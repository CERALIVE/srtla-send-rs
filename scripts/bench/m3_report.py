import hashlib
from collections.abc import Sequence
from pathlib import Path

from m3_conformance import evaluate
from m3_models import (FOREIGN, RECEIVERS, SCENARIOS, SENDERS, CellResult, M3Summary,
                       Outcome, Quadrant, SenderVerdict, Spike, compare)
from report import (CapturedRun, EvidenceError, Manifest, load_records, receiver_specs,
                    stable_identity, statistics, summarize_cell)


def reduce(manifest_path: Path, roots: Sequence[Path]) -> M3Summary:
    raw_manifest = manifest_path.read_bytes()
    manifest = Manifest.model_validate_json(raw_manifest)
    receivers = receiver_specs(manifest)
    manifest = manifest.model_copy(update={"cells": tuple(cell.model_copy(update={
        "cell_id": stable_identity(cell, receivers[cell.receiver])}) for cell in manifest.cells)})
    expected = {(s, r, c, 3, "") for s in SENDERS for r in RECEIVERS for c in SCENARIOS}
    expected |= {(s, "ours-new-200", "I", 1, "interop-conformance") for s in FOREIGN}
    actual = {(c.candidate, c.receiver, c.scenario, c.runs, c.variant) for c in manifest.cells}
    if manifest.campaign != "m3-interop" or actual != expected or len(manifest.cells) != 43:
        raise EvidenceError("M3 requires the frozen four-quadrant matrix and all foreign conformance cells")
    policies = {"ours-old": "&nakreport=1&lossmaxttl=40&reorderfreeze=0",
                "ours-new-200": "&nakreport=1&periodicnakgate=1&lossmaxttl=200&reorderfreeze=1"}
    if {name: r.listener_uri_extra for name, r in receivers.items()} != policies:
        raise EvidenceError("M3 receiver policies differ from M1 TTL*=200 and genuine old baseline")
    loaded = load_records(manifest, roots, m3_outcomes=True)
    if len(roots) != 1:
        raise EvidenceError("M3 uses one campaign root with its conformance-idle sibling")
    idle_root = roots[0].parent / "conformance-idle"
    candidates = {candidate.label: candidate for candidate in manifest.candidates}
    hashes = {str(manifest_path): hashlib.sha256(raw_manifest).hexdigest()}
    paths: dict[tuple[str, str, int], Path] = {}
    for root in roots:
        for path in sorted(root.glob("*/run-*.json")):
            if path.name.endswith(".exhausted.json"):
                continue
            raw = path.read_bytes()
            record = CapturedRun.model_validate_json(raw)
            paths[(record.cell_id, record.candidate.label, record.run_index)] = path
            hashes[str(path)] = hashlib.sha256(raw).hexdigest()
            directory = record.raw.stats_csv_path.parent
            for artifact in sorted(directory.iterdir()):
                if artifact.is_file() and artifact.suffix in (".pcap", ".csv", ".log", ".json"):
                    with artifact.open("rb") as stream:
                        hashes[str(artifact)] = hashlib.file_digest(stream, "sha256").hexdigest()
    cells = []
    conformance = []
    warnings = list(loaded.warnings)
    for cell in manifest.cells:
        records = loaded.by_cell[cell.id]
        integrity = summarize_cell(records, candidates[cell.candidate]).integrity_errors
        if integrity:
            raise EvidenceError(f"M3 configuration mismatch: {integrity}")
        for record in records:
            warnings.extend(record.warnings)
            if record.loadavg_1m > 2:
                warnings.append(f"{cell.id}/{record.run_index}: loadavg={record.loadavg_1m}")
        if cell.variant:
            record = records[0]
            conformance.extend(evaluate(paths[(record.cell_id, record.candidate.label, record.run_index)], idle_root))
            continue
        outcomes = []
        for record in records:
            ratio = record.diagnostics.get("retrans_ratio")
            if ratio is None or record.srt_live_transmit_sha256 is None:
                raise EvidenceError("M3 requires received retransmission fraction and receiver binary identity")
            outcomes.append(Outcome(run_index=record.run_index, settled=record.status == "ok",
                                    goodput_bps=record.useful_goodput_bps, retransmit_ratio=ratio))
        first = records[0]
        if first.srt_live_transmit_sha256 is None:
            raise EvidenceError("missing libsrt tool hash")
        cells.append(CellResult(sender=cell.candidate, scenario=cell.scenario, receiver=cell.receiver,
                                outcomes=tuple(outcomes), settled_runs=sum(o.settled for o in outcomes),
                                passing_runs=sum(o.settled and o.retransmit_ratio <= 0.10 for o in outcomes),
                                goodput=statistics([o.goodput_bps for o in outcomes]),
                                sender_sha256=first.candidate.bin_sha256, receiver_sha256=first.receiver.sha256,
                                libsrt_tool_sha256=first.srt_live_transmit_sha256))
    indexed = {(c.sender, c.receiver, c.scenario): c for c in cells}
    comparisons = tuple(compare(indexed[s, "ours-new-200", c], indexed[s, "ours-old", c]) for s in SENDERS for c in SCENARIOS)
    senders = {s: SenderVerdict.model_validate({"pass": all(c.passed for c in comparisons if c.sender == s),
                "failing_scenarios": tuple(c.scenario for c in comparisons if c.sender == s and not c.passed)}) for s in SENDERS}
    blocker = not senders["ours-3.3.0"].passed
    quadrants = []
    for population, names in (("existing", SENDERS[:-1]), ("new", ("ours-new",))):
        for receiver in RECEIVERS:
            selected = [c for c in cells if c.sender in names and c.receiver == receiver]
            quadrants.append(Quadrant(sender_population=population, receiver=receiver, senders=names,
                measured_runs=sum(len(c.outcomes) for c in selected), settled_runs=sum(c.settled_runs for c in selected),
                passing_runs=sum(c.passing_runs for c in selected)))
    decision = Spike(quadrants=tuple(quadrants), senders=senders, comparisons=comparisons, receiver_pr_blocker=blocker,
                     selected_alternative=("BLOCKER: Todo 24 must rerun M1's frozen rule with failing ours-3.3.0 scenarios added before TTL* is applied: " + ", ".join(senders["ours-3.3.0"].failing_scenarios)) if blocker else "ours-3.3.0 rollout passes; retain M1 TTL*=200; record foreign-only failures as known limitations for Todo 27")
    for artifact in sorted(idle_root.glob("*/*")):
        if artifact.is_file() and artifact.suffix in (".pcap", ".log", ".json"):
            hashes[str(artifact)] = hashlib.sha256(artifact.read_bytes()).hexdigest()
    return M3Summary(manifest_sha256=hashlib.sha256(raw_manifest).hexdigest(), cells=tuple(cells),
                     decision=decision, conformance=tuple(conformance), input_sha256=hashes, warnings=tuple(sorted(set(warnings))))


def markdown(summary: M3Summary) -> str:
    lines = ["# M3 four-quadrant interop", "", summary.decision.selected_alternative, "",
             "120 one-attempt performance outcomes; three separate foreign scenario-I conformance runs.",
             "New receiver: NAK-on, gate-on, freeze-on, TTL200. Old receiver: stock NAK, TTL40, freeze-off.",
             "Current/released own senders use enhanced. Settling failures retain full-window measurements, not successful status.",
             "Rule: at least2/3 jointly settled and retransmit≤10%, AND same-sender median goodput ratio≥0.95.", "",
             "| Sender | Scenario | Settled new | Joint pass new | New median Mbit/s | Old median Mbit/s | Ratio | Disposition |",
             "|---|---|---:|---:|---:|---:|---:|---|"]
    for c in summary.decision.comparisons:
        lines.append(f"| {c.sender} | {c.scenario} | {c.settled_runs}/3 | {c.passing_runs}/3 | {c.new_median_bps/1e6:.6f} | {c.old_median_bps/1e6:.6f} | {c.goodput_ratio:.6f} | {c.disposition} |")
    lines.extend(["", "## All per-run observations", "", "| Sender | Receiver | Scenario | Settled | Retransmit % by index | Goodput Mbit/s by index |",
                  "|---|---|---|---:|---|---|"])
    for c in summary.cells:
        ratios = ", ".join(f"{100*o.retransmit_ratio:.6f}" for o in c.outcomes)
        rates = ", ".join(f"{o.goodput_bps/1e6:.6f}" for o in c.outcomes)
        lines.append(f"| {c.sender} | {c.receiver} | {c.scenario} | {c.settled_runs}/3 | {ratios} | {rates} |")
    lines.extend(["", "## Foreign-sender limitations", "", summary.decision.foreign_mitigation,
                  "Foreign-only failures do not revert receiver policy. These are namespace/netem observations, not hardware certification.",
                  "", "## Warnings", "", *(f"- {warning}" for warning in summary.warnings), ""])
    return "\n".join(lines)


def conformance_markdown(summary: M3Summary) -> str:
    lines = ["# M3 foreign-sender conformance", "", "Both directions captured before registration; zero kernel capture drops required.",
             "Keepalive payload length/equality is measured, not inferred from protocol documentation.", "",
             "| Sender | Check | Result | Evidence |", "|---|---|---|---|"]
    for row in summary.conformance:
        lines.append(f"| {row.sender} | {row.check} | {'PASS' if row.passed else 'FAIL'} | {row.details} |")
    return "\n".join(lines) + "\n"
