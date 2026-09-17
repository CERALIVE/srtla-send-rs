from itertools import product
from statistics import median
from typing import Final

from m1_models import Decision, Failure, FreezeComparison, M1Summary
from report import EvidenceError, Number

TTLS: Final = (40, 200, 500)


def finite(value: Number) -> float:
    if value is None or value == "+inf":
        raise EvidenceError("M1 requires finite measured statistics")
    return value


def validate_matrix(summary: M1Summary) -> None:
    expected = {
        (sender, scenario, receiver, None)
        for sender, scenario, receiver in product(
            ("classic", "enhanced"),
            ("A", "B1", "C", "G"),
            ("ours-old", "new-40", "new-200", "new-500"),
        )
    }
    expected.update(("enhanced", "B1", f"new-{ttl}", 24) for ttl in TTLS)
    expected.update(
        ("enhanced", "S-FREEZE-NORDR", f"{prefix}-{ttl}", rate)
        for prefix, ttl, rate in product(("new", "thaw"), TTLS, (2, 6))
    )
    expected.update(
        (sender, scenario, f"new-{ttl}", None)
        for sender, scenario, ttl in product(
            ("belabox-c", "irlserver-rust"), ("B1", "G", "C"), TTLS
        )
    )
    actual = {
        (c.candidate, c.scenario, c.receiver, c.offered_mbit) for c in summary.cells
    }
    if actual != expected or len(summary.cells) != len(expected):
        raise EvidenceError("M1 matrix must contain exactly all 65 declared cells")
    for cell in summary.cells:
        ttl = 40 if cell.receiver == "ours-old" else int(cell.receiver.split("-")[1])
        if cell.ttl != ttl or cell.freeze_enabled != cell.receiver.startswith("new-"):
            raise EvidenceError("M1 receiver options disagree with cell identity")
        if len(cell.outcomes) != 3 or {r.run_index for r in cell.outcomes} != {0, 1, 2}:
            raise EvidenceError(
                "M1 requires three distinct one-attempt outcomes per cell"
            )
        for run in cell.outcomes:
            if run.settled != (run.reason is None) or run.reason not in (
                None,
                "settle_timeout",
            ):
                raise EvidenceError("M1 outcome is not a measured settling result")
            if (run.freeze is not None) != (cell.scenario == "S-FREEZE-NORDR"):
                raise EvidenceError(
                    "M1 freeze timing missing or assigned to wrong scenario"
                )


def decide(summary: M1Summary) -> Decision:
    validate_matrix(summary)
    sweep = tuple(
        c
        for c in summary.cells
        if c.receiver.startswith("new-") and (c.core or c.foreign)
    )
    core_counts = {
        str(ttl): sum(c.ttl == ttl and c.core and c.passes for c in sweep)
        for ttl in TTLS
    }
    counts = {str(ttl): sum(c.ttl == ttl and c.passes for c in sweep) for ttl in TTLS}
    complete = [ttl for ttl in TTLS if counts[str(ttl)] == 14]
    if complete:
        selected = min(complete)
        branch = "smallest_joint_pass"
    elif not any(count == 8 for count in core_counts.values()):
        selected = 200
        branch = "core_failure_upstream_parity"
    else:
        selected = max(
            TTLS, key=lambda ttl: (counts[str(ttl)], core_counts[str(ttl)], -ttl)
        )
        branch = "maximum_passing_cells"
    freeze = {
        (c.ttl, c.offered_mbit, c.freeze_enabled): c
        for c in summary.cells
        if c.scenario == "S-FREEZE-NORDR"
    }
    comparisons = []
    for ttl, rate in product(TTLS, (2, 6)):
        enabled, disabled = freeze[(ttl, rate, True)], freeze[(ttl, rate, False)]
        on = median(
            r.freeze.recovery_lower_bound_ms
            for r in enabled.outcomes
            if r.freeze is not None
        )
        off_bounds = tuple(
            r.freeze.recovery_upper_bound_ms
            for r in disabled.outcomes
            if r.freeze is not None
        )
        off = median(
            value if value is not None else float("inf") for value in off_bounds
        )
        identifiable = off != float("inf")
        comparisons.append(
            FreezeComparison(
                ttl=ttl,
                offered_mbit=rate,
                enabled_ms=on,
                disabled_ms=off if identifiable else None,
                penalty_ms=on - off if identifiable else None,
                censored=sum(
                    r.freeze.censored
                    for c in (enabled, disabled)
                    for r in c.outcomes
                    if r.freeze is not None
                ),
            )
        )
    identified = tuple(c.penalty_ms for c in comparisons if c.penalty_ms is not None)
    if not identified:
        raise EvidenceError("M1 freeze penalty is unidentifiable in every arm")
    penalty = max(0.0, *identified)
    capped = penalty > 250
    ttl_star = min(selected, 200) if capped else selected
    diagnostic = {
        c.ttl: c for c in summary.cells if c.scenario == "B1" and c.offered_mbit == 24
    }
    best = max(TTLS, key=lambda ttl: (finite(diagnostic[ttl].goodput.median), -ttl))
    baseline = finite(diagnostic[ttl_star].goodput.median)
    gap = (
        100 * (finite(diagnostic[best].goodput.median) - baseline) / baseline
        if baseline > 0
        else 0.0
    )
    separate = finite(diagnostic[best].goodput.ci_lower) > finite(
        diagnostic[ttl_star].goodput.ci_upper
    )
    failures = tuple(
        Failure(
            candidate=c.candidate,
            scenario=c.scenario,
            ttl=c.ttl,
            passing_runs=c.passing_runs,
            settled_runs=sum(r.settled for r in c.outcomes),
            retransmit_percent=tuple(100 * r.retrans_ratio for r in c.outcomes),
            goodput_mbit=tuple(r.goodput_bps / 1e6 for r in c.outcomes),
        )
        for c in sweep
        if not c.passes
    )
    alternative = f"static TTL {ttl_star}; periodic NAK gate on; freeze decided jointly"
    if branch == "core_failure_upstream_parity":
        alternative += "; upstream-parity fallback; Todo 8 in-flight-aware NAK protection is the compensating mechanism, not a proven cure"
    return Decision(
        outcome=f"{branch}: TTL*={ttl_star}",
        ttl_star=ttl_star,
        controller=best != ttl_star and gap >= 5 and separate,
        freeze_penalty_ms=penalty,
        selected_alternative=alternative,
        rule_branch=branch,
        sweep_ttl=selected,
        passing_cells=counts,
        passing_core_cells=core_counts,
        failing_cells=failures,
        best_24_mbit_ttl=best,
        goodput_gap_percent=gap,
        controller_nonoverlapping_ci=separate,
        freeze_comparisons=tuple(comparisons),
        freeze_cap_applied=capped,
        consequences=(
            "L3 keeps freeze OFF: justified by the measured recovery penalty",
            "A one-link bond inherits the penalty: accepted",
            "Freeze and TTL are coupled and must be decided jointly",
        )
        if penalty >= 60
        else (),
    )
