import math
from pathlib import Path
from typing import Final, Literal, assert_never

from m4_manifest import M4Manifest
from m4b_gate import score
from m4b_models import LineageCell, Observation
from m4b_manifest import ROOT
from report import (
    Candidate,
    CapturedRun,
    EvidenceError,
    RunRecord,
    metrics_v2_checks,
    summarize_cell,
)
from twinport_inputs import Lock, validate_player
from twinport_models import Measurement

DURATIONS: Final = {
    "A": 45_000,
    "B1": 45_000,
    "G": 45_000,
    "M1": 90_000,
    "M4": 60_000,
    "M6": 45_000,
}


class Attempt(CapturedRun):
    attempt: Literal[1]
    live: Literal[True] = True


def check_integrity(records: tuple[RunRecord, ...], candidate: Candidate) -> None:
    match records[0].sink:
        case "sls":
            if (
                len(
                    {
                        (r.fingerprint, r.scenario, r.receiver, r.srt_profile)
                        for r in records
                    }
                )
                != 1
            ):
                raise EvidenceError("mixed SLS scenario/receiver/profile/fingerprint")
            groups = tuple((record,) for record in records)
        case "slt":
            groups = (records,)
        case unreachable:
            assert_never(unreachable)
    for group in groups:
        integrity = summarize_cell(group, candidate)
        if integrity.integrity_errors:
            raise EvidenceError(str(integrity.integrity_errors))


def observation(record: RunRecord, path: Path) -> Observation:
    match record.sink:
        case "slt":
            checks = metrics_v2_checks((record,))
            return Observation(
                run_index=record.run_index,
                settled=record.status == "ok",
                retransmit_ratio=record.diagnostics.get("retrans_ratio"),
                zero_drop_belated=checks["post_settle_zero_drop_belated_rate"] == 1.0,
                starvation_floor=checks["post_settle_starvation_floor_rate"] == 1.0,
                goodput_bps=record.useful_goodput_bps,
                source=str(path),
            )
        case "sls":
            captured = Attempt.model_validate_json(path.read_bytes())
            directory = captured.raw.stats_csv_path.parent
            measured = Measurement.model_validate_json(
                (directory / "twinport.json").read_bytes()
            )
            validate_player(directory / "player.csv", measured)
            proof = record.sls_conformance
            if (
                proof is None
                or record.sls_stats is None
                or record.sls_identity is None
                or proof.offered_bytes <= 0
                or measured.sls_override is not None
                or measured.player_bytes != proof.player_bytes
                or measured.sender_end.bytes_sent_total
                - measured.sender_start.bytes_sent_total
                != measured.sender_bytes
                or proof.duration_ms != record.window.end_ms - record.window.start_ms
                or not math.isclose(
                    record.useful_goodput_bps,
                    proof.player_bytes * 8000 / proof.duration_ms,
                )
            ):
                raise EvidenceError(f"incomplete SLS measurement: {path}")
            assertions = proof.assertions
            passed = all((assertions.registered, assertions.carry, assertions.latency))
            if (
                assertions.carry != (proof.player_bytes * 10 >= proof.offered_bytes * 9)
                or proof.device_preset_ms != 2000
                or record.sls_stats.get("latency") != 2000
                or record.status == "ok"
                and not (measured.settled and passed)
            ):
                raise EvidenceError(f"inconsistent SLS conformance: {path}")
            return Observation(
                run_index=record.run_index,
                settled=measured.settled,
                retransmit_ratio=None,
                zero_drop_belated=None,
                starvation_floor=None,
                sink="sls",
                conformance=passed,
                goodput_bps=record.useful_goodput_bps,
                source=str(path),
            )
        case unreachable:
            assert_never(unreachable)


def load(manifest: M4Manifest, results: Path) -> tuple[LineageCell, ...]:
    lock = Lock.model_validate_json(
        (ROOT / "scripts/bench/receivers.lock.json").read_bytes()
    )
    expected = {(c.cell_id, c.candidate): c for c in manifest.cells}
    receivers = {r.name: r for r in manifest.receivers}
    candidates = {c.label: c for c in manifest.candidates}
    records: dict[tuple[str, str], dict[int, tuple[Path, Attempt]]] = {
        key: {} for key in expected
    }
    for path in sorted(results.glob("*/run-*.json")):
        if path.name.endswith(".exhausted.json"):
            continue
        run = Attempt.model_validate_json(path.read_bytes())
        key = (run.cell_id, run.candidate.label)
        if key not in expected:
            raise EvidenceError(f"unexpected cell: {path}")
        cell = expected[key]
        receiver = receivers[cell.receiver]
        locked = lock.receivers[cell.receiver]
        if (
            run.campaign != manifest.campaign
            or run.seed != manifest.seed
            or run.covering
            or run.receiver_label != cell.receiver
            or run.receiver_lineage != receiver.lineage
            or run.receiver.kind != receiver.kind
            or run.listener_uri_extra != receiver.listener_uri_extra
            or run.sink != cell.sink
            or run.metrics != cell.metrics
            or run.scenario.id != cell.scenario
            or run.srt_profile.name != cell.srt_profile
            or run.srt_profile.latency_ms != 2000
            or run.srt_profile.lossmaxttl != 40
            or run.srtla_rec_sha256 != locked.srtla_rec_sha256
            or run.receiver.sha256 != locked.srtla_rec_sha256
            or run.srt_live_transmit_sha256 != locked.srt_live_transmit_sha256
            or run.candidate.bin_sha256
            != lock.candidates["m4a-ours-new"][cell.candidate].srtla_send_sha256
            or run.run_index >= cell.runs
            or run.run_index in records[key]
            or run.supersedes is not None
        ):
            raise EvidenceError(f"identity/provenance/index mismatch: {path}")
        duration = run.window.end_ms - run.window.start_ms
        if not DURATIONS[cell.scenario] <= duration < DURATIONS[cell.scenario] + 2000:
            raise EvidenceError(f"short/incomplete measurement: {path}")
        match run.status:
            case "ok":
                if run.reason is not None:
                    raise EvidenceError(
                        f"successful outcome with failure reason: {path}"
                    )
            case "failed":
                if not (
                    run.reason == "settle_timeout"
                    or run.sink == "sls"
                    and run.reason == "execution_error"
                    and run.sls_conformance is not None
                    and not all(
                        (
                            run.sls_conformance.assertions.registered,
                            run.sls_conformance.assertions.carry,
                            run.sls_conformance.assertions.latency,
                        )
                    )
                ):
                    raise EvidenceError(f"infrastructure failure: {path}: {run.reason}")
            case "exhausted":
                raise EvidenceError(f"exhaustion is not an independent outcome: {path}")
            case unreachable:
                assert_never(unreachable)
        if (
            run.sls_identity is not None
            and run.sls_identity.binary_sha256 != lock.conformance_sinks["sls"].sha256
        ):
            raise EvidenceError(f"SLS binary mismatch: {path}")
        records[key][run.run_index] = (path, run)
    grouped: dict[str, LineageCell] = {}
    for cell in manifest.cells:
        rows = records[(cell.cell_id, cell.candidate)]
        if set(rows) != set(range(cell.runs)):
            raise EvidenceError(f"incomplete cell: {cell.id}: {sorted(rows)}")
        ordered = tuple(rows[i] for i in range(cell.runs))
        check_integrity(tuple(run for _, run in ordered), candidates[cell.candidate])
        result = score(tuple(observation(run, path) for path, run in ordered))
        previous = grouped.get(cell.cell_id)
        grouped[cell.cell_id] = LineageCell(
            cell_id=cell.cell_id,
            scenario=cell.scenario,
            lineage=receivers[cell.receiver].lineage or cell.receiver,
            sink=cell.sink,
            port=cell.port,
            members={**(previous.members if previous else {}), cell.candidate: result},
        )
    return tuple(grouped[key] for key in sorted(grouped))
