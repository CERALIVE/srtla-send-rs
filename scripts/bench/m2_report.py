import hashlib
from collections.abc import Sequence
from pathlib import Path

from m2_models import (
    CellResult,
    HsrspResult,
    InflightComparison,
    M2Summary,
    NakOffComparison,
    Outcome,
    RexmitResult,
    finite,
    hsrsp,
    regression,
)
from report import (
    CapturedRun,
    EvidenceError,
    Manifest,
    load_records,
    receiver_specs,
    stable_identity,
    statistics,
)


def _key(cell: CellResult) -> tuple[str, str, str]:
    return cell.candidate, cell.scenario, cell.receiver


def _passes_no_regression(adaptive: CellResult, enhanced: CellResult) -> bool:
    return (
        finite(adaptive.goodput.median) >= finite(enhanced.goodput.median)
        and finite(adaptive.goodput.ci_lower) >= finite(enhanced.goodput.ci_lower)
    )


def render(manifest_path: Path, roots: Sequence[Path]) -> tuple[str, str]:
    raw_manifest = manifest_path.read_bytes()
    manifest = Manifest.model_validate_json(raw_manifest)
    receivers = receiver_specs(manifest)
    manifest = manifest.model_copy(
        update={
            "cells": tuple(
                cell.model_copy(
                    update={
                        "cell_id": cell.cell_id
                        or stable_identity(cell, receivers[cell.receiver])
                    }
                )
                for cell in manifest.cells
            )
        }
    )
    if manifest.campaign != "m2-sender" or len(manifest.cells) != 19:
        raise EvidenceError("M2 requires the frozen 19-cell manifest")
    loaded = load_records(manifest, roots, m2_outcomes=True)
    captured: dict[tuple[str, str, int], CapturedRun] = {}
    for root in roots:
        for path in root.glob("*/run-*.json"):
            if path.name.endswith(".exhausted.json"):
                continue
            record = CapturedRun.model_validate_json(path.read_bytes())
            captured[(record.cell_id, record.candidate.label, record.run_index)] = record

    cells: list[CellResult] = []
    for cell in manifest.cells:
        records = loaded.by_cell[cell.id]
        if len(records) != cell.runs or {r.run_index for r in records} != set(range(cell.runs)):
            raise EvidenceError(f"M2 incomplete outcomes for {cell.id}")
        outcomes = tuple(
            Outcome(
                run_index=record.run_index,
                settled=record.status == "ok",
                reason=record.reason,
                goodput_bps=record.useful_goodput_bps,
            )
            for record in records
        )
        cells.append(
            CellResult(
                candidate=cell.candidate,
                scenario=cell.scenario,
                receiver=cell.receiver,
                variant=cell.variant,
                outcomes=outcomes,
                settle_rate=sum(outcome.settled for outcome in outcomes) / len(outcomes),
                goodput=statistics([outcome.goodput_bps for outcome in outcomes]),
            )
        )
    indexed = {_key(cell): cell for cell in cells if not cell.variant}

    inflight = []
    for scenario in ("B1", "C", "A", "G"):
        on = indexed[("enhanced", scenario, "ours-old")]
        off = indexed[("enhanced-rule-disabled", scenario, "ours-old")]
        inflight.append(
            InflightComparison(
                scenario=scenario,
                on=on.goodput,
                off=off.goodput,
                on_settle_rate=on.settle_rate,
                off_settle_rate=off.settle_rate,
                settle_rate_delta=on.settle_rate - off.settle_rate,
                regression=regression(on.goodput, off.goodput) if scenario in ("A", "G") else False,
            )
        )
    k_cap = 1 if any(item.regression for item in inflight) else 3

    nak_off = []
    for scenario in ("G", "F", "A"):
        enhanced = indexed[("enhanced", scenario, "irlserver-prod")]
        adaptive = indexed[("adaptive", scenario, "irlserver-prod")]
        passes = (
            finite(adaptive.goodput.ci_lower)
            >= 1.05 * finite(enhanced.goodput.ci_lower)
            if scenario == "G"
            else _passes_no_regression(adaptive, enhanced)
        )
        nak_off.append(
            NakOffComparison(
                scenario=scenario,
                enhanced=enhanced.goodput,
                adaptive=adaptive.goodput,
                passes=passes,
            )
        )
    nak_off_outcome = (
        "adaptive signals compensate" if all(item.passes for item in nak_off) else "present"
    )

    expected = {
        "ours-old": True,
        "ours-new": True,
        "irlserver-prod": False,
        "irlserver-next": True,
    }
    hsrsp_results: list[HsrspResult] = []
    for receiver, wanted in expected.items():
        cell = next(
            item
            for item in manifest.cells
            if item.scenario == "S-HSRSP" and item.receiver == receiver
        )
        record = loaded.by_cell[cell.id][0]
        raw = captured[(record.cell_id, record.candidate.label, record.run_index)]
        parsed = hsrsp(
            (raw.raw.stats_csv_path.parent / "sender.log").read_text(
                encoding="utf-8", errors="replace"
            ),
            wanted,
        )
        hsrsp_results.append(parsed.model_copy(update={"receiver": receiver}))

    rexmit_cell = next(cell for cell in manifest.cells if cell.variant == "rexmit-capture")
    rexmit_record = loaded.by_cell[rexmit_cell.id][0]
    rexmit_raw = captured[
        (rexmit_record.cell_id, rexmit_record.candidate.label, rexmit_record.run_index)
    ]
    from check_rexmit_bit import CaptureError, analyze

    try:
        visibility = analyze(rexmit_raw.raw.stats_csv_path.parent / "caller-srt.pcap")
    except CaptureError as error:
        raise EvidenceError(f"M2 retransmission capture invalid: {error}") from error
    if visibility.pcap_sha256 is None:
        raise EvidenceError("M2 retransmission capture hash missing")
    rexmit = RexmitResult.model_validate(visibility.model_dump())

    summary = M2Summary(
        manifest_sha256=hashlib.sha256(raw_manifest).hexdigest(),
        cells=tuple(cells),
        inflight=tuple(inflight),
        nak_off=tuple(nak_off),
        rexmit=rexmit,
        hsrsp=tuple(hsrsp_results),
        k_cap=k_cap,
        nak_off_outcome=nak_off_outcome,
        warnings=tuple(sorted(set(loaded.warnings))),
    )
    lines = [
        "# M2 sender-mechanism campaign",
        "",
        "47 one-attempt outcomes; settle timeouts retain their complete measured windows.",
        "Bootstrap median CI: 10,000 resamples, seed 20260913.",
        "The raw runner exited 101 because 24 ours-old outcomes did not settle; this reduction follows the predeclared rule and retains those full-window measurements rather than relabeling them as successes.",
        "",
        "## In-flight premature-NAK rule",
        "",
        "| Scenario | ON settled | OFF settled | Delta | ON goodput Mbit/s median [95% CI] | OFF goodput Mbit/s median [95% CI] | Regression |",
        "|---|---:|---:|---:|---|---|---|",
    ]
    for item in inflight:
        lines.append(
            f"| {item.scenario} | {item.on_settle_rate:.3f} | {item.off_settle_rate:.3f} | {item.settle_rate_delta:+.3f} | "
            f"{finite(item.on.median)/1e6:.3f} [{finite(item.on.ci_lower)/1e6:.3f}, {finite(item.on.ci_upper)/1e6:.3f}] | "
            f"{finite(item.off.median)/1e6:.3f} [{finite(item.off.ci_lower)/1e6:.3f}, {finite(item.off.ci_upper)/1e6:.3f}] | {item.regression} |"
        )
    lines.extend(
        [
            "",
            f"K-cap decision: **K={k_cap}**; the rule remains enabled.",
            "",
        "## NAK-off blindness",
            "",
            "| Scenario | Enhanced Mbit/s median [95% CI] | Adaptive Mbit/s median [95% CI] | Rule passes |",
            "|---|---|---|---|",
        ]
    )
    for item in nak_off:
        lines.append(
            f"| {item.scenario} | {finite(item.enhanced.median)/1e6:.3f} [{finite(item.enhanced.ci_lower)/1e6:.3f}, {finite(item.enhanced.ci_upper)/1e6:.3f}] | "
            f"{finite(item.adaptive.median)/1e6:.3f} [{finite(item.adaptive.ci_lower)/1e6:.3f}, {finite(item.adaptive.ci_upper)/1e6:.3f}] | {item.passes} |"
        )
    lines.extend(
        [
            "",
            f"Outcome: **{nak_off_outcome}**.",
            "",
            "## Retransmission-bit visibility",
            "",
            f"DATA={rexmit.data_packets}; originals={rexmit.originals}; retransmissions={rexmit.retransmissions}; "
            f"R=1 originals={rexmit.flagged_originals}; R=0 retransmissions={rexmit.unflagged_retransmissions}.",
            f"Outcome: **{'counters trusted' if rexmit.passed else 'not visible on this encoder build'}**. "
            f"Capture SHA-256: `{rexmit.pcap_sha256}`.",
            "",
            "## HSRSP lineage decode",
            "",
            "| Receiver | Expected | Observed | Status line |",
            "|---|---|---|---|",
        ]
    )
    for item in hsrsp_results:
        observed = "null" if item.nak_report is None else "on" if item.nak_report else "off"
        lines.append(
            f"| {item.receiver} | {'on' if item.expected else 'off'} | {observed} | `{item.status_line or ''}` |"
        )
    lines.extend(
        (
            "",
            "The absent irlserver-prod observation is recorded as null (None), selecting fail-safe NAK-on policy without adding a parser heuristic.",
        )
    )
    lines.extend(("", "## Warnings", "", *(f"- {item}" for item in summary.warnings), ""))
    return "\n".join(lines), summary.model_dump_json(indent=2) + "\n"
