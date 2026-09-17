import hashlib
from collections.abc import Sequence
from pathlib import Path
from urllib.parse import parse_qs

from m1_freeze import analyze
from m1_models import CellResult, M1Summary, Outcome
from report import (
    CapturedRun,
    EvidenceError,
    Manifest,
    load_records,
    receiver_specs,
    statistics,
    stable_identity,
    summarize_cell,
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
    loaded = load_records(manifest, roots, m1_outcomes=True)
    candidates = {candidate.label: candidate for candidate in manifest.candidates}
    captured = {}
    for root in roots:
        for path in root.glob("*/run-*.json"):
            if path.name.endswith(".exhausted.json"):
                continue
            record = CapturedRun.model_validate_json(path.read_bytes())
            captured[(record.cell_id, record.candidate.label, record.run_index)] = (
                record
            )
    cells: list[CellResult] = []
    for cell in manifest.cells:
        records = loaded.by_cell[cell.id]
        errors = summarize_cell(records, candidates[cell.candidate]).integrity_errors
        if errors:
            raise EvidenceError(f"M1 configuration mismatch {cell.id}: {errors}")
        outcomes: list[Outcome] = []
        for record in records:
            ratio = record.diagnostics.get("retrans_ratio")
            if ratio is None or not record.load_intervals:
                raise EvidenceError(f"M1 missing full-window metrics {cell.id}")
            freeze = None
            if cell.scenario == "S-FREEZE-NORDR":
                raw = captured[
                    (record.cell_id, record.candidate.label, record.run_index)
                ]
                freeze = analyze(raw.raw.stats_csv_path.parent / "receiver-srt.pcap")
            outcomes.append(
                Outcome(
                    run_index=record.run_index,
                    settled=record.status == "ok",
                    reason=record.reason,
                    goodput_bps=record.useful_goodput_bps,
                    retrans_ratio=ratio,
                    fingerprint=record.fingerprint,
                    freeze=freeze,
                )
            )
        options = parse_qs(receivers[cell.receiver].listener_uri_extra.lstrip("&"))
        cells.append(
            CellResult.model_validate(
                {
                    "candidate": cell.candidate,
                    "scenario": cell.scenario,
                    "receiver": cell.receiver,
                    "ttl": int(options["lossmaxttl"][0]),
                    "freeze_enabled": options["reorderfreeze"] == ["1"],
                    "offered_mbit": cell.offered_mbit_override,
                    "outcomes": tuple(outcomes),
                    "goodput": statistics([r.goodput_bps for r in outcomes]),
                    "retransmit": statistics([r.retrans_ratio for r in outcomes]),
                }
            )
        )
    warnings = list(loaded.warnings)
    for cell in manifest.cells:
        for record in loaded.by_cell[cell.id]:
            warnings.extend(
                f"{cell.id} run {record.run_index}: {warning}"
                for warning in record.warnings
            )
            if record.loadavg_1m > 2:
                warnings.append(
                    f"{cell.id} run {record.run_index}: loadavg>2 ({record.loadavg_1m})"
                )
    summary = M1Summary(
        manifest_sha256=hashlib.sha256(raw_manifest).hexdigest(),
        cells=tuple(cells),
        warnings=tuple(sorted(set(warnings))),
    )
    from m1_rule import validate_matrix

    validate_matrix(summary)
    lines = [
        "# M1 receiver TTL campaign",
        "",
        "65 cells; 195 planned one-attempt outcomes. Failed settling remains failed.",
        "",
        "Retransmission fraction = received retransmissions / received packets. Bootstrap median CI: 10,000 resamples, seed 20260913.",
        "",
        "Old lineage has stock periodic NAK and lacks the freeze URI; its default freeze-off behavior is explicitly recorded. No emulation or patch of that baseline.",
        "",
        f"Freeze timing scope: {summary.freeze_scope}.",
        "",
        "| Receiver | Sender | Scenario | Offered override | Settled | Joint passing | Goodput Mbit/s (95% CI) | Retransmissions % by run |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for cell in cells:
        goodput = cell.goodput
        numbers = (goodput.median, goodput.ci_lower, goodput.ci_upper)
        if not all(isinstance(value, float) for value in numbers):
            raise EvidenceError("M1 finite goodput confidence interval required")
        formatted = "/".join(
            f"{float(value) / 1e6:.3f}" for value in numbers if value is not None
        )
        ratios = ", ".join(f"{100 * r.retrans_ratio:.3f}" for r in cell.outcomes)
        lines.append(
            f"| {cell.receiver} | {cell.candidate} | {cell.scenario} | {cell.offered_mbit or 'catalog'} | {sum(r.settled for r in cell.outcomes)}/3 | {cell.passing_runs}/3 | {formatted} | {ratios} |"
        )
    lines.extend(
        [
            "",
            "## Freeze timing (per-run medians, milliseconds)",
            "",
            "| Receiver | Mbit/s | Index | Gaps | First NAK | Repair | Repair lower bound incl. censored | Censored |",
            "|---|---|---|---|---|---|---|---|",
        ]
    )
    for cell in cells:
        for outcome in cell.outcomes:
            if (timing := outcome.freeze) is not None:
                lines.append(
                    f"| {cell.receiver} | {cell.offered_mbit} | {outcome.run_index} | {timing.gaps} | {timing.first_nak_ms} | {timing.recovery_ms} | {timing.recovery_lower_bound_ms:.3f} | {timing.censored} |"
                )
    lines.extend(
        ["", "## Warnings", "", *[f"- {warning}" for warning in summary.warnings], ""]
    )
    return "\n".join(lines), summary.model_dump_json(indent=2) + "\n"
