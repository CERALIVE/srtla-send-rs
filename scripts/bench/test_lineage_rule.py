"""Synthetic frozen-rule tests; these are never measurement evidence."""

from pathlib import Path
import subprocess
import sys

import pytest

from decide import CANDIDATES, Group, Summary, SYNTHETIC_SUMMARIES


SCENARIOS = (
    "A",
    "B1",
    "B2",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    *(f"M{i}" for i in range(1, 8)),
)


def fixture() -> Summary:
    source = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0]).groups[0]
    cell = source.cells["adaptive"]
    pair = cell.comparisons["adaptive"].model_copy(update={"n": 5})
    tied = cell.model_copy(
        update={
            "n": 5,
            "run_indices": tuple(range(5)),
            "comparisons": {name: pair for name in CANDIDATES},
            "checks": {
                "settled_rate": 1.0,
                "post_settle_zero_drop_belated_rate": 1.0,
                "post_settle_starvation_floor_rate": 1.0,
            },
        }
    )
    return Summary(
        schema_version=1,
        groups=tuple(
            Group(
                campaign="m4a-ours-new",
                scenario=scenario,
                receiver="ours-new",
                lineage="ours-new",
                profile="production",
                cell_id=f"ours-new@--{scenario}@--production--slt:4001--fec:off",
                cells={name: tied for name in CANDIDATES},
            )
            for scenario in SCENARIOS
        ),
    )


def test_cli_freezes_new_shape_and_deterministic_bytes(tmp_path: Path) -> None:
    # Given a complete tied CLI matrix.
    source = tmp_path / "summary.json"
    source.write_text(fixture().model_dump_json())
    outputs = [tmp_path / f"verdict-{i}.json" for i in range(2)]
    # When the real CLI executes twice.
    for output in outputs:
        result = subprocess.run(
            [
                sys.executable,
                str(Path(__file__).with_name("decide.py")),
                "--rule",
                "lineage-d1",
                "--summaries",
                str(source),
                "--out",
                str(output),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0, result.stderr
    # Then classic wins the tunable tie, but enhanced is the base coverage tie winner.
    from lineage_rule import LineageVerdict

    verdict = LineageVerdict.model_validate_json(outputs[0].read_bytes())
    assert verdict.ship_set == ("classic",)
    assert verdict.base_mode == "enhanced"
    assert verdict.covered_by_base_pct == 100
    assert outputs[0].read_bytes() == outputs[1].read_bytes()


@pytest.mark.parametrize(
    "gate",
    [
        "post_settle_zero_drop_belated_rate",
        "post_settle_starvation_floor_rate",
        "settled_rate",
    ],
)
def test_gate_failure_cannot_cover(gate: str) -> None:
    # Given a complete matrix with a failed candidate gate in A.
    from lineage_rule import decide_lineage

    summary = fixture()
    group = summary.groups[0]
    cells = {
        name: cell.model_copy(update={"checks": {**cell.checks, gate: 0.0}})
        for name, cell in group.cells.items()
    }
    changed = summary.model_copy(
        update={
            "groups": (group.model_copy(update={"cells": cells}), *summary.groups[1:])
        }
    )
    # When applying the rule, then the empty-set terminal is explicit.
    verdict = decide_lineage(changed)
    assert verdict.ship_set == ("enhanced",)
    assert verdict.covered_by_base_pct == 95
    assert verdict.sacrificed_cells[0].reason == "uncovered"
    assert gate in verdict.sacrificed_cells[0].failing_gates


def test_baseline_and_noncovering_rows_cannot_choose_base() -> None:
    # Given reference evidence with arbitrarily greater goodput.
    from lineage_rule import decide_lineage

    summary = fixture()
    reference = summary.groups[0].model_copy(
        update={
            "covering": False,
            "cell_id": "reference",
            "cells": {
                "upstream-classic": summary.groups[0]
                .cells["classic"]
                .model_copy(update={"goodput_median": 1e12})
            },
        }
    )
    # When reduced, then references have no influence on the verdict.
    assert decide_lineage(
        summary.model_copy(update={"groups": (*summary.groups, reference)})
    ) == decide_lineage(summary)


def test_d_is_exempt_only_when_all_cli_candidates_fail() -> None:
    from lineage_rule import decide_lineage

    summary = fixture()
    groups = tuple(
        g.model_copy(
            update={
                "cells": {
                    name: cell.model_copy(update={"checks": {}})
                    for name, cell in g.cells.items()
                }
            }
        )
        if g.scenario == "D"
        else g
        for g in summary.groups
    )
    verdict = decide_lineage(summary.model_copy(update={"groups": groups}))
    assert [cell.scenario for cell in verdict.cells.values() if cell.exempt] == ["D"]
    assert verdict.covered_by_base_pct == 100


def test_sacrifices_remain_when_another_mode_covers_the_cell() -> None:
    from lineage_rule import decide_lineage

    summary = fixture()
    group = summary.groups[0]
    cells = {
        name: cell if name == "adaptive" else cell.model_copy(update={"checks": {}})
        for name, cell in group.cells.items()
    }
    other_groups = tuple(
        g.model_copy(
            update={
                "cells": {
                    name: cell
                    if name == "enhanced"
                    else cell.model_copy(update={"checks": {}})
                    for name, cell in g.cells.items()
                }
            }
        )
        for g in summary.groups[1:]
    )
    verdict = decide_lineage(
        summary.model_copy(
            update={
                "groups": (group.model_copy(update={"cells": cells}), *other_groups)
            }
        )
    )
    assert verdict.ship_set == ("enhanced", "adaptive")
    assert verdict.base_mode == "enhanced"
    assert verdict.covered_by_base_pct == 95
    assert verdict.sacrificed_cells[0].scenario == "A"
    assert verdict.sacrificed_cells[0].reason == "not_covered_by_base"


@pytest.mark.parametrize("mutation", ["missing", "duplicate", "incomplete"])
def test_incomplete_primary_matrix_is_rejected(mutation: str) -> None:
    # Given a malformed primary matrix.
    from lineage_rule import decide_lineage
    from report import EvidenceError

    summary = fixture()
    groups = summary.groups
    match mutation:
        case "missing":
            groups = groups[1:]
        case "duplicate":
            groups = (*groups, groups[0])
        case "incomplete":
            groups = (groups[0].model_copy(update={"cells": {}}), *groups[1:])
        case _:
            raise AssertionError(mutation)
    # When evaluated, then missing evidence cannot become the terminal fallback.
    with pytest.raises(EvidenceError):
        decide_lineage(summary.model_copy(update={"groups": groups}))
