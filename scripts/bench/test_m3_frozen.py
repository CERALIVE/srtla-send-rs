# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest", "numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run pytest scripts/bench/ -q -k m3_frozen
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

import m3_frozen
import m3_models
import m3_provenance
from m3_models import CellResult, M3Summary, Outcome, Spike
from report import statistics

ROOT = Path(__file__).resolve().parents[2]
SPIKE_PATH = ROOT / "docs/evidence/bpc/m3-interop/spike.json"
RUN_FILES = 3


def cell(
    sender: str,
    scenario: str,
    receiver: str,
    median: float,
    passing_runs: int,
    settled_runs: int,
) -> CellResult:
    return CellResult(
        sender=sender,
        scenario=scenario,
        receiver=receiver,
        outcomes=tuple(
            Outcome(
                run_index=index,
                settled=index < settled_runs,
                goodput_bps=median,
                retransmit_ratio=0.5,
            )
            for index in range(RUN_FILES)
        ),
        settled_runs=settled_runs,
        passing_runs=passing_runs,
        goodput=statistics([median] * RUN_FILES),
        sender_sha256="a" * 64,
        receiver_sha256="b" * 64,
        libsrt_tool_sha256="c" * 64,
    )


def cell_dir(receiver: str, scenario: str, sender: str) -> str:
    return f"{receiver}@synthetic--{scenario}@--production--slt:4001--fec:off--{sender}"


def synthetic_summary(root: Path) -> M3Summary:
    results = root / "results"
    cells: list[CellResult] = []
    hashes: dict[str, str] = {}
    for sender, scenario in m3_frozen.ROWS:
        for receiver, median in (
            (m3_frozen.NEW_RECEIVER, 110.0),
            (m3_frozen.OLD_RECEIVER, 100.0),
        ):
            directory = results / cell_dir(receiver, scenario, sender)
            directory.mkdir(parents=True)
            for index in range(RUN_FILES):
                run = directory / f"run-{index}.json"
                run.write_text(json.dumps({"run": index}), encoding="utf-8")
                hashes[str(run)] = hashlib.sha256(run.read_bytes()).hexdigest()
            cells.append(cell(sender, scenario, receiver, median, 0, RUN_FILES))
    decision = Spike(
        quadrants=(),
        senders={},
        comparisons=(),
        receiver_pr_blocker=False,
        selected_alternative="synthetic",
    )
    return M3Summary(
        manifest_sha256="a" * 64,
        cells=tuple(cells),
        decision=decision,
        conformance=(),
        input_sha256=hashes,
        warnings=(),
    )


def spike_fixture(summary: M3Summary) -> Spike:
    return Spike(
        quadrants=(),
        senders={},
        comparisons=m3_frozen.comparisons_for(summary, "required"),
        receiver_pr_blocker=False,
        selected_alternative="synthetic",
    )


def test_m3_frozen_required_is_byte_identical_to_committed_spike() -> None:
    spike = Spike.model_validate_json(SPIKE_PATH.read_bytes())
    committed = {(c.sender, c.scenario): c for c in spike.comparisons}
    for sender, scenario in m3_frozen.ROWS:
        fixture = committed[(sender, scenario)]
        new = cell(
            sender,
            scenario,
            m3_frozen.NEW_RECEIVER,
            fixture.new_median_bps,
            fixture.passing_runs,
            fixture.settled_runs,
        )
        old = cell(
            sender,
            scenario,
            m3_frozen.OLD_RECEIVER,
            fixture.old_median_bps,
            fixture.passing_runs,
            fixture.settled_runs,
        )
        produced = m3_frozen.compare_with_policy(new, old, "required")
        assert produced == m3_models.compare(new, old)
        assert produced.model_dump_json() == fixture.model_dump_json()


def test_m3_frozen_unavailable_drops_only_the_retransmit_predicate() -> None:
    # Given a row whose only failure is the joint retransmit predicate.
    retransmit_only = (
        cell("belabox-c", "C", m3_frozen.NEW_RECEIVER, 100.0, 0, 3),
        cell("belabox-c", "C", m3_frozen.OLD_RECEIVER, 100.0, 0, 3),
    )
    required = m3_frozen.compare_with_policy(*retransmit_only, "required")
    unavailable = m3_frozen.compare_with_policy(*retransmit_only, "unavailable")
    # Then required fails and unavailable passes: the predicate was dropped.
    assert required.passed is False
    assert required.disposition == "known_limitation"
    assert unavailable.passed is True
    assert unavailable.disposition == "pass"
    # Given a row that fails the retained goodput predicate.
    goodput_only = (
        cell("belabox-c", "C", m3_frozen.NEW_RECEIVER, 90.0, 3, 3),
        cell("belabox-c", "C", m3_frozen.OLD_RECEIVER, 100.0, 3, 3),
    )
    # Then unavailable still fails on goodput: only retransmit was dropped.
    assert m3_frozen.compare_with_policy(*goodput_only, "unavailable").passed is False
    assert m3_frozen.failing_remaining(
        m3_frozen.compare_with_policy(*goodput_only, "unavailable")
    ) == ("goodput_ratio_ge_0.95",)


def test_m3_frozen_build_scores_and_verifies_every_row(tmp_path: Path) -> None:
    summary = synthetic_summary(tmp_path)
    provenance = m3_provenance.load_provenance(summary, None)
    result, errors = m3_frozen.build(
        summary,
        provenance,
        retransmit="unavailable",
        raw_root=tmp_path,
        spike_comparisons=spike_fixture(summary).comparisons,
    )
    assert errors == ()
    assert result.spike_verified is True
    assert len(result.rows) == 5
    assert all(row.raw_verified for row in result.rows)
    assert all(row.rescore_passes_without_retransmit for row in result.rows)
    assert all(row.provisional_verdict == "measurement_artefact" for row in result.rows)
    assert {row.m3_disposition for row in result.rows} <= {
        "known_limitation",
        "receiver_pr_blocker",
        "new_sender_failure",
    }


def test_m3_frozen_missing_run_fails_closed(tmp_path: Path) -> None:
    summary = synthetic_summary(tmp_path)
    missing = (
        tmp_path / "results" / cell_dir("ours-new-200", "C", "ours-new") / "run-1.json"
    )
    missing.unlink()
    provenance = m3_provenance.load_provenance(summary, None)
    result, errors = m3_frozen.build(
        summary,
        provenance,
        retransmit="unavailable",
        raw_root=tmp_path,
        spike_comparisons=spike_fixture(summary).comparisons,
    )
    assert any(
        "missing raw run" in error and "ours-new/C/ours-new-200/run-1" in error
        for error in errors
    )
    status = {(row.sender, row.scenario): row.raw_verified for row in result.rows}
    assert status[("ours-new", "C")] is False
    assert all(value for key, value in status.items() if key != ("ours-new", "C"))


def test_m3_frozen_cli_writes_rows_and_exits_nonzero_on_missing_run(
    tmp_path: Path,
) -> None:
    summary = synthetic_summary(tmp_path)
    spike = tmp_path / "spike.json"
    spike.write_text(spike_fixture(summary).model_dump_json(), encoding="utf-8")
    summary_path = tmp_path / "summary.json"
    summary_path.write_text(summary.model_dump_json(), encoding="utf-8")
    out = tmp_path / "rescore.json"
    command = [
        sys.executable,
        str(Path(__file__).with_name("decide.py")),
        "--rule",
        "m3-frozen",
        "--retransmit",
        "unavailable",
        "--summary",
        str(summary_path),
        "--spike",
        str(spike),
        "--m3-raw-root",
        str(tmp_path),
        "--out",
        str(out),
    ]
    happy = subprocess.run(command, capture_output=True, text=True, timeout=60)
    assert happy.returncode == 0, happy.stderr
    document = json.loads(out.read_text(encoding="utf-8"))
    assert len(document["rows"]) == 5
    assert all(row["raw_verified"] for row in document["rows"])
    # When one C-cell raw run is deleted, then the rule refuses that row.
    (
        tmp_path / "results" / cell_dir("ours-old", "C", "ours-3.3.0") / "run-0.json"
    ).unlink()
    failure = subprocess.run(command, capture_output=True, text=True, timeout=60)
    assert failure.returncode != 0
    assert "ours-3.3.0/C/ours-old/run-0" in failure.stderr
    refused = json.loads(out.read_text(encoding="utf-8"))
    assert {row["sender"]: row["raw_verified"] for row in refused["rows"]}[
        "ours-3.3.0"
    ] is False
