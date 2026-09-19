from pathlib import Path
import subprocess
import sys

import pytest

import m4b_gate as module
from lineage_rule import LineageVerdict
from m4b_manifest import ROOT
from m4b_inputs import DURATIONS
import m4b_inputs
from report import ReportTests, Window


def test_reference_g_uses_catalog_45_seconds_not_privileged_test_60() -> None:
    assert DURATIONS["G"] == 45_000


def test_sls_group_when_elapsed_milliseconds_differ_retains_actual_windows() -> None:
    fixture = ReportTests()
    runs = tuple(
        fixture.record().model_copy(
            update={
                "sink": "sls",
                "run_index": i,
                "window": Window(start_ms=0, end_ms=45_000 + i),
            }
        )
        for i in range(3)
    )
    m4b_inputs.check_integrity(runs, fixture.manifest().candidates[0])
    assert tuple(r.window.end_ms for r in runs) == (45_000, 45_001, 45_002)


def test_slt_group_when_windows_differ_still_rejects() -> None:
    fixture = ReportTests()
    runs = tuple(
        fixture.record().model_copy(
            update={
                "run_index": i,
                "window": Window(start_ms=0, end_ms=45_000 + i),
            }
        )
        for i in range(3)
    )
    with pytest.raises(module.EvidenceError):
        m4b_inputs.check_integrity(runs, fixture.manifest().candidates[0])


def test_gate_when_joint_runs_at_quota_passes() -> None:
    runs = tuple(
        module.Observation(
            run_index=i,
            settled=i < 2,
            retransmit_ratio=0.10,
            zero_drop_belated=True,
            starvation_floor=True,
        )
        for i in range(3)
    )
    result = module.score(runs)
    assert result.passed
    assert result.joint_passes == 2


@pytest.mark.parametrize(
    "field,value",
    [
        ("zero_drop_belated", False),
        ("starvation_floor", False),
        ("zero_drop_belated", None),
        ("starvation_floor", None),
    ],
)
def test_gate_when_any_applicable_metric_fails_cannot_pass(
    field: str, value: bool | None
) -> None:
    runs = tuple(
        module.Observation(
            run_index=i,
            settled=True,
            retransmit_ratio=0.01,
            zero_drop_belated=True,
            starvation_floor=True,
        )
        for i in range(3)
    )
    runs = (runs[0].model_copy(update={field: value}), *runs[1:])
    assert not module.score(runs).passed


def test_gate_when_sls_denominator_absent_never_invents_zero() -> None:
    runs = tuple(
        module.Observation(
            run_index=i,
            settled=True,
            retransmit_ratio=None,
            zero_drop_belated=None,
            starvation_floor=None,
            sink="sls",
            conformance=True,
        )
        for i in range(3)
    )
    result = module.score(runs)
    assert not result.passed
    assert "retransmit_unknown" in result.failing_gates
    assert result.joint_passes == 0


def test_gate_when_n5_requires_four_joint_successes() -> None:
    runs = tuple(
        module.Observation(
            run_index=i,
            settled=True,
            retransmit_ratio=0.10 if i < 3 else 0.100001,
            zero_drop_belated=True,
            starvation_floor=True,
        )
        for i in range(5)
    )
    assert not module.score(runs).passed
    assert module.score(
        (
            runs[3].model_copy(update={"retransmit_ratio": 0.10}),
            runs[0],
            runs[1],
            runs[2],
            runs[4],
        )
    ).passed


def test_gate_when_indices_incomplete_refuses_verdict() -> None:
    run = module.Observation(
        run_index=0,
        settled=True,
        retransmit_ratio=0.01,
        zero_drop_belated=True,
        starvation_floor=True,
    )
    with pytest.raises(module.EvidenceError):
        module.score((run, run, run))


def test_finalize_when_all_fail_retains_single_base_and_sacrifices() -> None:
    base = LineageVerdict.model_validate_json(
        (ROOT / "docs/evidence/bpc/m4/verdict-provisional.json").read_bytes()
    )
    result = module.score(
        tuple(
            module.Observation(
                run_index=i,
                settled=True,
                retransmit_ratio=0.01,
                zero_drop_belated=False,
                starvation_floor=True,
            )
            for i in range(3)
        )
    )
    cell = module.LineageCell(
        cell_id="test",
        scenario="M1",
        lineage="belabox",
        sink="slt",
        port=4001,
        members={"enhanced": result},
    )
    final = module.finalize(base, (cell,))
    assert final.ship_set == ("enhanced",)
    assert final.base_mode == "enhanced"
    assert final.covered_by_base_pct == 0.0
    assert final.lineage_gate.passes_per_member == {"enhanced": 0}
    assert final.lineage_gate.swap is None
    assert len(final.sacrificed_cells) == 20
    assert final.sacrificed_cells[-1].reason == "lineage gate"
    assert final.model_dump_json() == module.finalize(base, (cell,)).model_dump_json()


def test_cli_when_measurements_absent_refuses_output(tmp_path: Path) -> None:
    destination = tmp_path / "verdict.json"
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts/bench/m4b_gate.py"),
            "--verdict",
            str(ROOT / "docs/evidence/bpc/m4/verdict-provisional.json"),
            "--manifest",
            str(ROOT / "scripts/bench/manifests/m4b-lineages.json"),
            "--results",
            str(tmp_path),
            "--out",
            str(destination),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 1
    assert "incomplete cell" in result.stderr
    assert not destination.exists()


@pytest.mark.parametrize(
    "second_passes,expected", [(False, "enhanced"), (True, "edpf")]
)
def test_finalize_when_two_members_swaps_only_on_better_result(
    second_passes: bool, expected: str
) -> None:
    base = LineageVerdict.model_validate_json(
        (ROOT / "docs/evidence/bpc/m4/verdict-provisional.json").read_bytes()
    )
    base = base.model_copy(update={"ship_set": ("enhanced", "edpf")})
    scores = {
        name: module.score(
            tuple(
                module.Observation(
                    run_index=i,
                    settled=True,
                    retransmit_ratio=0.01,
                    zero_drop_belated=name == "edpf" and second_passes,
                    starvation_floor=True,
                )
                for i in range(3)
            )
        )
        for name in base.ship_set
    }
    cell = module.LineageCell(
        cell_id="test",
        scenario="M1",
        lineage="belabox",
        sink="slt",
        port=4001,
        members=scores,
    )
    final = module.finalize(base, (cell,))
    assert final.base_mode == expected
    assert final.ship_set == base.ship_set
    assert (final.lineage_gate.swap is not None) == second_passes
