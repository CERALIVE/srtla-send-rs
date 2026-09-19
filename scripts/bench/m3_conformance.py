from pathlib import Path
from typing import Literal

from m3_models import ConformanceRow
from m3_wire import classify, read_capture
from report import Count, Document, EvidenceError, RawPaths, RunRecord


class Bucket(Document):
    t_ms: int
    duration_ms: Count
    bytes: Count


class Raw(RawPaths):
    sink_series: tuple[Bucket, ...]


class SourceRate(Document):
    bps: Count


class OfferedRateAction(Document):
    OfferedRate: SourceRate


class Event(Document):
    action: Literal["ReceiverRestart"] | OfferedRateAction


class EventRecord(Document):
    t_ms: Count
    event: Event


class Captured(RunRecord):
    raw: Raw
    events: tuple[EventRecord, ...]


def recovery_ms(record: Captured, restart_ms: int) -> int | None:
    offered = record.load_intervals[0].offered_bps
    consecutive = 0
    for second in range((restart_ms + 999)//1000 + 1, record.window.end_ms//1000 + 1):
        buckets = [b for b in record.raw.sink_series if (second-1)*1000 < b.t_ms <= second*1000]
        complete = sum(b.duration_ms for b in buckets) == 1000
        passing = complete and sum(b.bytes for b in buckets)*8 >= 0.9*offered
        consecutive = consecutive + 1 if passing else 0
        if consecutive >= 3:
            return second*1000 - restart_ms
    return None


def evaluate(path: Path, idle_root: Path) -> tuple[ConformanceRow, ...]:
    record = Captured.model_validate_json(path.read_bytes())
    directory = record.raw.stats_csv_path.parent
    paths = tuple(sorted(directory.glob("interop-*.pcap")))
    if len(paths) != 2:
        raise EvidenceError("scenario I requires two conformance captures")
    wire = classify(tuple(p for i, path in enumerate(paths) for p in read_capture(path, i)), 2)
    idle_dir = idle_root / record.candidate.label
    idle_record = RunRecord.model_validate_json((idle_dir / "identity.json").read_bytes())
    if (idle_record.candidate, idle_record.receiver, idle_record.srt_live_transmit_sha256,
        idle_record.listener_uri_extra) != (record.candidate, record.receiver,
            record.srt_live_transmit_sha256, record.listener_uri_extra):
        raise EvidenceError("idle conformance must use identical sender/receiver binaries and options")
    idle_paths = tuple(sorted(idle_dir.glob("interop-*.pcap")))
    if len(idle_paths) != 2:
        raise EvidenceError("idle conformance requires two captures")
    idle = classify(tuple(p for i, path in enumerate(idle_paths) for p in read_capture(path, i)), 2)
    restarts = [event.t_ms for event in record.events if event.event.action == "ReceiverRestart"]
    if len(restarts) != 1 or not record.load_intervals:
        raise EvidenceError("scenario I restart/load evidence missing")
    restart = restarts[0]
    recovered = recovery_ms(record, restart)
    fresh = len(wire.complete_groups) >= 2 and wire.group_completion_s[-1] - wire.group_completion_s[0] > 20
    sender = record.candidate.label
    expected_length = 2 if sender == "belabox-c" else 38
    return (
        ConformanceRow(sender=sender, check="registration", passed=bool(wire.complete_groups),
                       details=f"Both-link complete receiver group handshakes={len(wire.complete_groups)}; group SHA256={wire.complete_groups}"),
        ConformanceRow(sender=sender, check="keepalive_echo",
                       passed=bool(idle.complete_groups) and all(idle.keepalive_echoes) and idle.mismatched_echoes == 0 and idle.keepalive_lengths == (expected_length,),
                       details=f"10s idle supplement: requests/link={idle.keepalive_requests}; exact echoes/link={idle.keepalive_echoes}; request lengths={idle.keepalive_lengths}; unmatched/mismatched echoes={idle.mismatched_echoes}; expected length={expected_length}. Loaded scenario-I requests/link={wire.keepalive_requests}, exact echoes/link={wire.keepalive_echoes}"),
        ConformanceRow(sender=sender, check="receiver_restart", passed=fresh and recovered is not None and 20000 <= restart <= 22000,
                       details=f"Restart event completed at measurement {restart}ms (scheduled20000ms; completion lag upper bound={restart-20000}ms); fresh receiver group={fresh}; three-second sink recovery after completion={recovered}ms"),
    )
