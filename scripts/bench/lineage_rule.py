"""Frozen lineage-d1 rule: CLI-only primary coverage, independent base selection."""

from itertools import combinations
from typing import Final, Literal

from decide import (
    Document,
    Evidence,
    Group,
    RuleRequest,
    Summary,
    TUNABLES,
    evidence_errors,
)
from report import EvidenceError

CLI: Final = ("classic", "enhanced", "rtt-threshold", "edpf", "adaptive")
SCENARIOS: Final = (
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
N: Final = 5
COVERAGE_THRESHOLD: Final = 0.95
GATES: Final = (
    "settled_rate",
    "post_settle_zero_drop_belated_rate",
    "post_settle_starvation_floor_rate",
)


class CandidateResult(Document):
    goodput_ratio: float | None
    ci_lower: float | None
    failing_gates: tuple[str, ...]


class CellResult(Document):
    scenario: str
    lineage: str
    winner: str
    covered_by: tuple[str, ...]
    candidates: dict[str, CandidateResult]
    exempt: bool = False
    exemption_reason: str | None = None


class SacrificedCell(Document):
    cell_id: str
    scenario: str
    lineage: str
    winner: str
    base_mode_ratio: float | None
    base_mode_ci_lower: float | None
    failing_gates: tuple[str, ...]
    reason: Literal["uncovered", "not_covered_by_base"]


class LineageVerdict(Document):
    ship_set: tuple[str, ...]
    base_mode: str
    covered_by_base_pct: float
    sacrificed_cells: tuple[SacrificedCell, ...]
    cells: dict[str, CellResult]
    refold: Literal[False] = False
    gate: None = None


def candidate_result(cell: Evidence, best: Evidence, winner: str) -> CandidateResult:
    failures = [gate for gate in GATES if cell.checks.get(gate) != 1.0]
    pair = cell.comparisons.get(winner)
    if cell.integrity_errors:
        failures.append("integrity")
    if best.goodput_median <= 0:
        failures.append("winner_zero_goodput")
    if pair is None:
        failures.append("missing_pair")
    else:
        if (
            pair.n != N
            or pair.dropped_indices
            or pair.episode_mismatch
            or set(cell.run_indices) != set(best.run_indices)
        ):
            failures.append("pair_integrity")
        if (
            pair.ci_lower is None
            or pair.ci_upper is None
            or pair.ci_lower > pair.ci_upper
            or pair.ci_lower < COVERAGE_THRESHOLD
        ):
            failures.append("goodput_ci_lower")
        if (
            pair.viewer_loss_delta_pp > 0.1
            or 100 * (cell.viewer_loss_median - best.viewer_loss_median) > 0.1
        ):
            failures.append("loss")
        if cell.graded_episodes != best.graded_episodes:
            failures.append("episode_count")
        if best.graded_episodes:
            if (
                pair.nonfinite_recovery
                or pair.recovery_ratio is None
                or pair.recovery_ratio > 1.10
            ):
                failures.append("recovery_ratio")
            if (
                cell.nonrecovered_rate is None
                or best.nonrecovered_rate is None
                or cell.nonrecovered_rate > best.nonrecovered_rate
            ):
                failures.append("nonrecovery_rate")
    return CandidateResult(
        goodput_ratio=cell.goodput_median / best.goodput_median
        if best.goodput_median > 0
        else None,
        ci_lower=pair.ci_lower if pair else None,
        failing_gates=tuple(failures),
    )


def primary_groups(summary: Summary) -> tuple[Group, ...]:
    groups = tuple(g for g in summary.groups if g.covering)
    if any(
        g.sink != "slt"
        or "--sls:" in g.cell_id
        or "--fec:on" in g.cell_id
        or g.lineage != "ours-new"
        or g.scenario not in SCENARIOS
        for g in groups
    ):
        raise EvidenceError("covering rows must be primary ours-new SLT cells")
    if len(groups) != len(SCENARIOS) or {g.scenario for g in groups} != set(SCENARIOS):
        raise EvidenceError(
            "exactly one primary ours-new group per declared scenario required"
        )
    errors = [
        error
        for g in groups
        for error in evidence_errors(g, RuleRequest(CLI, SCENARIOS, N))
    ]
    if errors or any(set(g.cells) != set(CLI) for g in groups):
        raise EvidenceError(
            "candidate-complete CLI N=5 primary matrix required: " + "; ".join(errors)
        )
    return tuple(sorted(groups, key=lambda g: (g.lineage, g.scenario, g.key)))


def decide_lineage(summary: Summary) -> LineageVerdict:
    groups = primary_groups(summary)
    outcomes: dict[str, CellResult] = {}
    for group in groups:
        winner = min(
            CLI,
            key=lambda name: (
                -group.cells[name].goodput_median,
                name != "enhanced",
                name,
            ),
        )
        results = {
            name: candidate_result(group.cells[name], group.cells[winner], winner)
            for name in CLI
        }
        exempt = group.scenario == "D" and all(
            result.failing_gates for result in results.values()
        )
        outcomes[group.key] = CellResult(
            scenario=group.scenario,
            lineage=group.lineage,
            winner=winner,
            covered_by=tuple(name for name in CLI if not results[name].failing_gates),
            candidates=results,
            exempt=exempt,
            exemption_reason="all_cli_candidates_fail" if exempt else None,
        )
    obligations = {key: cell for key, cell in outcomes.items() if not cell.exempt}
    counts = {
        name: sum(name in cell.covered_by for cell in obligations.values())
        for name in CLI
    }
    base = min(CLI, key=lambda name: (-counts[name], name != "enhanced", name))
    choices = [
        subset
        for size in range(1, len(CLI) + 1)
        for subset in combinations(CLI, size)
        if all(
            set(subset).intersection(cell.covered_by) for cell in obligations.values()
        )
    ]
    shipped = (
        min(
            choices,
            key=lambda subset: (
                len(subset),
                sum(TUNABLES[name] for name in subset),
                "enhanced" not in subset,
                subset,
            ),
        )
        if choices
        else (base,)
    )
    sacrificed = tuple(
        SacrificedCell(
            cell_id=key,
            scenario=cell.scenario,
            lineage=cell.lineage,
            winner=cell.winner,
            base_mode_ratio=cell.candidates[base].goodput_ratio,
            base_mode_ci_lower=cell.candidates[base].ci_lower,
            failing_gates=cell.candidates[base].failing_gates,
            reason="not_covered_by_base" if choices else "uncovered",
        )
        for key, cell in obligations.items()
        if base not in cell.covered_by
    )
    return LineageVerdict(
        ship_set=shipped,
        base_mode=base,
        covered_by_base_pct=100 * counts[base] / len(obligations)
        if obligations
        else 100,
        sacrificed_cells=sacrificed,
        cells=outcomes,
    )
