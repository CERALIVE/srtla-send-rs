import csv
import hashlib
import math
import re
from pathlib import Path
from typing import Literal

from pydantic import Field

from report import CapturedRun, Count, Document, EvidenceError, Hash, Positive, SlsAssertions
from twinport_models import ApiSnapshot, Measurement


class LockedCandidate(Document):
    srtla_send_sha256: Hash


class LockedReceiver(Document):
    srtla_rec_sha256: Hash
    srt_live_transmit_sha256: Hash


class LockedSls(Document):
    sha256: Hash


class Lock(Document):
    candidates: dict[str, dict[str, LockedCandidate]]
    receivers: dict[str, LockedReceiver]
    conformance_sinks: dict[str, LockedSls]


class Attempt(CapturedRun):
    attempt: int = Field(ge=1, le=2)
    reason: str | None = None


class PlayerRow(Document):
    time: int = Field(alias="Time")
    socket: int = Field(alias="SocketID")
    loss: Count = Field(alias="pktRcvLoss")
    drop: Count = Field(alias="pktRcvDrop")
    rate: Positive = Field(alias="mbpsRecvRate")
    latency: Positive = Field(alias="msRcvTsbPdDelay")


class Observation(Document):
    profile: Literal["default", "legacy-l2"]
    scenario: Literal["M1", "SLS"]
    port: Literal[4002, 4003]
    run_index: Count
    attempt: int
    status: str
    reason: str | None
    accepted: bool
    measurement: Measurement
    assertions: SlsAssertions
    publisher_keys: tuple[str, ...]
    publisher_drop_start: Count | None
    publisher_drop_end: Count | None
    publisher_received_start: Count | None
    publisher_received_end: Count | None
    publisher_latency_ms: Count
    profile_proof: tuple[str, ...]
    source: str


def validate_player(path: Path, measurement: Measurement) -> None:
    with path.open() as stream:
        rows = [PlayerRow.model_validate(row, strict=False) for row in csv.DictReader(stream)]
    selected = [row for row in rows if measurement.player_csv_start_ms < row.time <= measurement.player_csv_end_ms]
    boundary = [row for row in rows if row.time == measurement.player_csv_start_ms]
    span = measurement.player_csv_end_ms - measurement.player_csv_start_ms
    if (not selected or len(boundary) != 1 or len({row.socket for row in selected + boundary}) != 1
            or selected[-1].time != measurement.player_csv_end_ms or abs(span - measurement.elapsed_ms) > 2000):
        raise EvidenceError(f"missing/discontinuous player stats: {path}")
    previous = measurement.player_csv_start_ms
    weighted = 0.0
    for row in selected:
        if row.time < previous or row.latency != 200:
            raise EvidenceError(f"wrong player clock or latency: {path}")
        weighted += row.rate * (row.time - previous)
        previous = row.time
    rate = weighted / (previous - measurement.player_csv_start_ms)
    leg = measurement.player_leg
    if (sum(row.loss for row in selected) != leg.loss or sum(row.drop for row in selected) != leg.drop
            or not math.isclose(rate, leg.mbps_recv_rate, abs_tol=1e-9)):
        raise EvidenceError(f"player CSV disagrees with measurement: {path}")
    if measurement.player_leg_valid != leg.valid(measurement.offered_bps):
        raise EvidenceError(f"incorrect player-leg acceptance: {path}")


def load(path: Path) -> Observation:
    run = Attempt.model_validate_json(path.read_bytes())
    lock = Lock.model_validate_json((Path(__file__).parent / "receivers.lock.json").read_bytes())
    if (run.candidate.bin_sha256 != lock.candidates[run.campaign]["enhanced"].srtla_send_sha256
            or run.srtla_rec_sha256 != lock.receivers["ours-new"].srtla_rec_sha256
            or run.srt_live_transmit_sha256 != lock.receivers["ours-new"].srt_live_transmit_sha256
            or run.sls_identity is None or run.sls_identity.binary_sha256 != lock.conformance_sinks["sls"].sha256):
        raise EvidenceError(f"locked binary mismatch: {path}")
    directory = run.raw.stats_csv_path.parent
    measurement = Measurement.model_validate_json((directory / "twinport.json").read_bytes())
    scenario = run.scenario.id
    profile = run.campaign.removeprefix("twinport-")
    if scenario not in ("M1", "SLS") or profile not in ("default", "legacy-l2"):
        raise EvidenceError(f"unexpected twinport cell: {path}")
    match_port = re.search(r"--sls:(4002|4003)--", run.cell_id)
    if not match_port or run.seed != 20260913 or run.candidate.label != "enhanced":
        raise EvidenceError(f"wrong cell identity: {path}")
    if run.sink != "sls" or run.metrics != "none" or run.covering or run.receiver_lineage != "ours-new":
        raise EvidenceError(f"wrong measurement path: {path}")
    if run.sls_conformance is None or run.sls_identity is None:
        raise EvidenceError(f"missing SLS evidence: {path}")
    expected_rate, expected_ms = (9_600_000, 90_000) if scenario == "M1" else (1_000_000, 20_000)
    expected_override = None if profile == "default" else "legacy-l2"
    if (measurement.offered_bps != expected_rate or measurement.sls_override != expected_override
            or not expected_ms <= run.sls_conformance.duration_ms < expected_ms + 2000):
        raise EvidenceError(f"wrong rate, override, or window: {path}")
    if (measurement.sender_end.bytes_sent_total - measurement.sender_start.bytes_sent_total != measurement.sender_bytes
            or measurement.player_bytes != run.sls_conformance.player_bytes):
        raise EvidenceError(f"incoherent byte counters: {path}")
    validate_player(directory / "player.csv", measurement)
    start = ApiSnapshot.model_validate_json((directory / "sls-start.json").read_bytes()).publishers["publish/live/conformance"]
    end = ApiSnapshot.model_validate_json((directory / "sls-end.json").read_bytes()).publishers["publish/live/conformance"]
    expected_latency = 2000 if scenario == "M1" else 500
    if start.latency != expected_latency or end.latency != expected_latency:
        raise EvidenceError(f"wrong negotiated publisher latency: {path}")
    policy = ("freeze=1 nakreport=1 periodic_nak_gate=1 lossmaxttl=200 floor=100 fec_accept=1"
              if profile == "default" else "freeze=1 nakreport=0 periodic_nak_gate=0 lossmaxttl=40 floor=100 fec_accept=0")
    log = (directory / "listener.log").read_text()
    proof = tuple(line for line in log.splitlines() if policy in line and "profile=L" in line)
    if not all(any(f"profile={name} " in line for line in proof) for name in ("L1-bonded", "L2-bonded-alias")):
        raise EvidenceError(f"rollback does not reach both listener profiles: {path}")
    if "failed to read latency" in log:
        raise EvidenceError(f"latency getter failure: {path}")
    assertions = run.sls_conformance.assertions
    if not measurement.player_leg_valid and run.reason != "player_leg_invalid":
        raise EvidenceError(f"invalid leg not discarded: {path}")
    accepted = (run.status == "ok" and measurement.player_leg_valid and measurement.settled
                and assertions.registered and assertions.carry and assertions.latency)
    if run.status == "ok" and not accepted:
        raise EvidenceError(f"false success: {path}")
    keys = tuple(sorted(end.model_fields_set))
    return Observation.model_validate({
        "profile": profile, "scenario": scenario, "port": int(match_port[1]),
        "run_index": run.run_index, "attempt": run.attempt, "status": run.status, "reason": run.reason,
        "accepted": accepted, "measurement": measurement, "assertions": assertions,
        "publisher_keys": keys, "publisher_drop_start": start.pktRcvDrop, "publisher_drop_end": end.pktRcvDrop,
        "publisher_received_start": start.pktRecv, "publisher_received_end": end.pktRecv,
        "publisher_latency_ms": end.latency, "profile_proof": proof, "source": str(path),
    })


def digest(path: Path) -> Hash:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()
