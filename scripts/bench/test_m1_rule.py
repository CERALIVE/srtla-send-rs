from itertools import product
from pathlib import Path
import subprocess
import sys

import pytest

from m1_models import CellResult, FreezeTiming, M1Summary, Outcome
from report import EvidenceError, statistics


def fixture() -> M1Summary:
    specs = [
        (sender, scenario, receiver, None)
        for sender, scenario, receiver in product(
            ("classic", "enhanced"),
            ("A", "B1", "C", "G"),
            ("ours-old", "new-40", "new-200", "new-500"),
        )
    ]
    specs += [("enhanced", "B1", f"new-{ttl}", 24) for ttl in (40, 200, 500)]
    specs += [
        ("enhanced", "S-FREEZE-NORDR", f"{prefix}-{ttl}", rate)
        for prefix, ttl, rate in product(("new", "thaw"), (40, 200, 500), (2, 6))
    ]
    specs += [
        (sender, scenario, f"new-{ttl}", None)
        for sender, scenario, ttl in product(
            ("belabox-c", "irlserver-rust"), ("B1", "G", "C"), (40, 200, 500)
        )
    ]
    cells = []
    for sender, scenario, receiver, rate in specs:
        freeze = (
            FreezeTiming(
                gaps=10,
                nak_observed=10,
                repaired=10,
                censored=0,
                first_nak_ms=10.0,
                recovery_ms=70.0,
                recovery_lower_bound_ms=70.0,
                recovery_upper_bound_ms=70.0,
                pcap_sha256="a" * 64,
            )
            if scenario == "S-FREEZE-NORDR"
            else None
        )
        cells.append(
            CellResult.model_validate(
                {
                    "candidate": sender,
                    "scenario": scenario,
                    "receiver": receiver,
                    "ttl": 40
                    if receiver == "ours-old"
                    else int(receiver.split("-")[1]),
                    "freeze_enabled": receiver.startswith("new"),
                    "offered_mbit": rate,
                    "outcomes": tuple(
                        Outcome(
                            run_index=i,
                            settled=True,
                            reason=None,
                            goodput_bps=10e6,
                            retrans_ratio=0.01,
                            fingerprint="a" * 64,
                            freeze=freeze,
                        )
                        for i in range(3)
                    ),
                    "goodput": statistics([10e6] * 3),
                    "retransmit": statistics([0.01] * 3),
                }
            )
        )
    return M1Summary(manifest_sha256="a" * 64, cells=tuple(cells), warnings=())


def test_smallest_ttl_when_every_cell_passes() -> None:
    from m1_rule import decide

    result = decide(fixture())
    assert result.ttl_star == 40
    assert not result.controller
    assert result.rule_branch == "smallest_joint_pass"


def test_core_failure_uses_explicit_parity_fallback() -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(
            update={
                "outcomes": tuple(
                    run.model_copy(
                        update={"settled": False, "reason": "settle_timeout"}
                    )
                    for run in cell.outcomes
                )
            }
        )
        if cell.core and cell.scenario == "C"
        else cell
        for cell in original.cells
    )
    result = decide(original.model_copy(update={"cells": cells}))
    assert result.ttl_star == 200
    assert result.rule_branch == "core_failure_upstream_parity"
    assert len(result.failing_cells) == 6


def test_foreign_two_of_three_joint_boundary() -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(
            update={
                "outcomes": tuple(
                    run.model_copy(
                        update={"retrans_ratio": 0.1 if run.run_index < 2 else 0.11}
                    )
                    for run in cell.outcomes
                )
            }
        )
        if cell.foreign
        else cell
        for cell in original.cells
    )
    assert decide(original.model_copy(update={"cells": cells})).ttl_star == 40


def test_foreign_failures_use_passing_cell_score_not_hand_selection() -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(
            update={
                "outcomes": tuple(
                    run.model_copy(update={"retrans_ratio": 0.11})
                    for run in cell.outcomes
                )
            }
        )
        if cell.foreign and (cell.ttl != 500 or cell.scenario == "C")
        else cell
        for cell in original.cells
    )
    result = decide(original.model_copy(update={"cells": cells}))
    assert result.ttl_star == 500
    assert result.rule_branch == "maximum_passing_cells"


@pytest.mark.parametrize(
    ("penalty", "capped"), [(60.0, False), (250.0, False), (251.0, True)]
)
def test_freeze_cap_strictly_exceeds_250(penalty: float, capped: bool) -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(
            update={
                "outcomes": tuple(
                    run.model_copy(
                        update={
                            "freeze": run.freeze.model_copy(
                                update={"recovery_lower_bound_ms": 70.0 + penalty}
                            )
                        }
                    )
                    for run in cell.outcomes
                    if run.freeze is not None
                )
            }
        )
        if cell.scenario == "S-FREEZE-NORDR"
        and cell.freeze_enabled
        and cell.offered_mbit == 2
        else cell
        for cell in original.cells
    )
    result = decide(original.model_copy(update={"cells": cells}))
    assert result.freeze_penalty_ms == penalty
    assert result.freeze_cap_applied == capped
    assert len(result.consequences) == 3


def test_controller_requires_gap_and_disjoint_ci() -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(update={"goodput": statistics([11e6] * 3)})
        if cell.offered_mbit == 24 and cell.ttl == 500
        else cell
        for cell in original.cells
    )
    assert decide(original.model_copy(update={"cells": cells})).controller
    cells = tuple(
        cell.model_copy(update={"goodput": statistics([9e6, 11e6, 13e6])})
        if cell.offered_mbit == 24 and cell.ttl == 500
        else cell
        for cell in original.cells
    )
    assert not decide(original.model_copy(update={"cells": cells})).controller


def test_missing_foreign_arm_refuses_decision() -> None:
    from m1_rule import decide

    original = fixture()
    with pytest.raises(EvidenceError, match="matrix"):
        decide(original.model_copy(update={"cells": original.cells[:-1]}))


def test_censored_disabled_median_cannot_be_claimed_as_penalty() -> None:
    from m1_rule import decide

    original = fixture()
    cells = tuple(
        cell.model_copy(
            update={
                "outcomes": tuple(
                    run.model_copy(
                        update={
                            "freeze": run.freeze.model_copy(
                                update={"recovery_upper_bound_ms": None}
                            )
                        }
                    )
                    for run in cell.outcomes
                    if run.freeze is not None
                )
            }
        )
        if cell.scenario == "S-FREEZE-NORDR"
        and cell.ttl == 500
        and not cell.freeze_enabled
        else cell
        for cell in original.cells
    )
    result = decide(original.model_copy(update={"cells": cells}))
    assert all(
        pair.penalty_ms is None for pair in result.freeze_comparisons if pair.ttl == 500
    )


def test_cli_decision_reproduces_and_rejects_incomplete_matrix(tmp_path: Path) -> None:
    summary = tmp_path / "summary.json"
    first, second = tmp_path / "first.json", tmp_path / "second.json"
    summary.write_text(fixture().model_dump_json())
    command = [
        sys.executable,
        str(Path(__file__).with_name("decide.py")),
        "--rule",
        "m1-ttl",
        "--summary",
        str(summary),
        "--out",
    ]
    for path in (first, second):
        result = subprocess.run(
            [*command, str(path)],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        assert result.returncode == 0, result.stderr
    assert first.read_bytes() == second.read_bytes()
    original = fixture()
    summary.write_text(
        original.model_copy(update={"cells": original.cells[:-1]}).model_dump_json()
    )
    result = subprocess.run(
        [*command, str(second)], capture_output=True, text=True, timeout=30, check=False
    )
    assert result.returncode == 1
    assert not second.exists()
