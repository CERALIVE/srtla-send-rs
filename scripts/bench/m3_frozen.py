"""Frozen M3 rule re-run with the retransmit-% predicate optionally marked unavailable.

M3's recorded predicates are (a) at least two of three runs jointly settled with a
received retransmission fraction <= 10 % and (b) same-sender median goodput ratio
>= 0.95. The per-run retransmission fraction has no validated denominator, so it
must be marked unavailable rather than reconstructed from any counter. This module
applies the recorded rule with that predicate dropped; the goodput predicate is
always retained.
"""

from __future__ import annotations

from pathlib import Path
from typing import Final, Literal

from m3_models import FOREIGN, CellResult, Comparison, M3Summary
from m3_provenance import (
    NEW_RECEIVER,
    OLD_RECEIVER,
    ROWS,
    Provenance,
    verify_raw,
)
from report import Document, EvidenceError, Positive

type RetransmitPolicy = Literal["required", "unavailable"]
type RowKey = tuple[str, str]
type Disposition = Literal[
    "pass", "known_limitation", "receiver_pr_blocker", "new_sender_failure"
]
type Predicate = Literal["retransmit_joint_pass_ge_2", "goodput_ratio_ge_0.95"]
type Verdict = Literal["measurement_artefact", "still_failing"]

GOODPUT_PREDICATE: Final[Predicate] = "goodput_ratio_ge_0.95"
JOINT_PASS_MIN: Final = 2
GOODPUT_FLOOR: Final = 0.95


class RescoreRow(Document):
    sender: str
    scenario: str
    m3_disposition: Disposition
    m3_goodput_ratio: Positive
    rescore_passes_without_retransmit: bool
    failing_predicates_remaining: tuple[Predicate, ...]
    provisional_verdict: Verdict
    raw_verified: bool


class Rescore(Document):
    schema_version: Literal[1] = 1
    rule: Literal["m3-frozen"] = "m3-frozen"
    retransmit: RetransmitPolicy
    provenance_source: str
    spike_verified: bool | None = None
    rows: tuple[RescoreRow, ...]
    m3_comparisons: tuple[Comparison, ...]


def disposition(sender: str, passed: bool) -> Disposition:
    if passed:
        return "pass"
    if sender in FOREIGN:
        return "known_limitation"
    if sender == "ours-3.3.0":
        return "receiver_pr_blocker"
    return "new_sender_failure"


def compare_with_policy(
    new: CellResult, old: CellResult, retransmit: RetransmitPolicy
) -> Comparison:
    if (new.sender, new.scenario, new.sender_sha256) != (
        old.sender,
        old.scenario,
        old.sender_sha256,
    ):
        raise EvidenceError("frozen comparison must use the same sender/scenario/binary")
    a, b = new.goodput.median, old.goodput.median
    if not isinstance(a, float) or not isinstance(b, float) or b <= 0:
        raise EvidenceError("frozen comparison requires finite positive old median goodput")
    retransmit_pass = new.passing_runs >= JOINT_PASS_MIN
    goodput_pass = a / b >= GOODPUT_FLOOR
    passed = goodput_pass and (retransmit_pass if retransmit == "required" else True)
    return Comparison(
        sender=new.sender,
        scenario=new.scenario,
        settled_runs=new.settled_runs,
        passing_runs=new.passing_runs,
        new_median_bps=a,
        old_median_bps=b,
        goodput_ratio=a / b,
        passed=passed,
        disposition=disposition(new.sender, passed),
    )


def row_cells(
    summary: M3Summary, sender: str, scenario: str
) -> tuple[CellResult, CellResult]:
    index = {(cell.sender, cell.receiver, cell.scenario): cell for cell in summary.cells}
    try:
        new = index[(sender, NEW_RECEIVER, scenario)]
        old = index[(sender, OLD_RECEIVER, scenario)]
    except KeyError as error:
        raise EvidenceError(
            f"missing M3 cell for frozen row {sender}/{scenario}: {error.args[0]}"
        ) from error
    return new, old


def comparisons_for(
    summary: M3Summary, retransmit: RetransmitPolicy
) -> tuple[Comparison, ...]:
    return tuple(
        compare_with_policy(*row_cells(summary, sender, scenario), retransmit)
        for sender, scenario in ROWS
    )


def failing_remaining(comparison: Comparison) -> tuple[Predicate, ...]:
    ratio = comparison.new_median_bps / comparison.old_median_bps
    return () if ratio >= GOODPUT_FLOOR else (GOODPUT_PREDICATE,)


def rescore(
    summary: M3Summary,
    retransmit: RetransmitPolicy,
    provenance: Provenance,
    verified: dict[RowKey, bool],
) -> Rescore:
    required = comparisons_for(summary, "required")
    policy = comparisons_for(summary, retransmit)
    rows: list[RescoreRow] = []
    for (sender, scenario), recorded, applied in zip(
        ROWS, required, policy, strict=True
    ):
        remaining = failing_remaining(applied)
        passes = not remaining
        rows.append(
            RescoreRow(
                sender=sender,
                scenario=scenario,
                m3_disposition=recorded.disposition,
                m3_goodput_ratio=applied.goodput_ratio,
                rescore_passes_without_retransmit=passes,
                failing_predicates_remaining=remaining,
                provisional_verdict=(
                    "measurement_artefact" if passes else "still_failing"
                ),
                raw_verified=verified[(sender, scenario)],
            )
        )
    return Rescore(
        retransmit=retransmit,
        provenance_source=provenance.source,
        rows=tuple(rows),
        m3_comparisons=required,
    )


def spike_comparisons_match(
    summary: M3Summary, spike_comparisons: tuple[Comparison, ...]
) -> bool:
    recorded = {
        (comparison.sender, comparison.scenario): comparison
        for comparison in spike_comparisons
    }
    return all(
        recorded.get((sender, scenario)) == produced
        for (sender, scenario), produced in zip(
            ROWS, comparisons_for(summary, "required"), strict=True
        )
    )


def build(
    summary: M3Summary,
    provenance: Provenance,
    *,
    retransmit: RetransmitPolicy,
    raw_root: Path | None = None,
    spike_comparisons: tuple[Comparison, ...] | None = None,
) -> tuple[Rescore, tuple[str, ...]]:
    verified, errors = verify_raw(summary, provenance, raw_root)
    result = rescore(summary, retransmit, provenance, verified)
    spike_verified = (
        spike_comparisons_match(summary, spike_comparisons)
        if spike_comparisons is not None
        else None
    )
    if spike_verified is False:
        errors = (
            *errors,
            "recomputed required-mode comparisons differ from the committed spike",
        )
    return result.model_copy(update={"spike_verified": spike_verified}), errors


def load_summary(path: Path) -> M3Summary:
    return M3Summary.model_validate_json(path.read_bytes())
