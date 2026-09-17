import hashlib
import math
import statistics
import subprocess
from dataclasses import dataclass, replace
from pathlib import Path

from m1_models import FreezeTiming
from report import EvidenceError


@dataclass(frozen=True, slots=True)
class Gap:
    detected_ms: float
    nak_ms: float | None = None
    repaired_ms: float | None = None


@dataclass(frozen=True, slots=True)
class Packet:
    at_ms: float
    source: int
    destination: int
    payload: bytes


def gaps_from_packets(packets: tuple[Packet, ...]) -> tuple[Gap, ...]:
    gaps: dict[int, Gap] = {}
    highest: int | None = None
    for packet in packets:
        data = packet.payload
        if len(data) < 16:
            continue
        word = int.from_bytes(data[:4], "big")
        if packet.destination == 4001 and word < 0x80000000:
            seq = word
            if seq in gaps and gaps[seq].repaired_ms is None:
                gaps[seq] = replace(gaps[seq], repaired_ms=packet.at_ms)
            if highest is None:
                highest = seq
            distance = (seq - highest) & 0x7FFFFFFF
            if 0 < distance < 0x40000000:
                if distance > 100_000:
                    raise EvidenceError(
                        "freeze capture has an unbounded sequence discontinuity"
                    )
                for offset in range(1, distance):
                    gaps[(highest + offset) & 0x7FFFFFFF] = Gap(packet.at_ms)
                highest = seq
        elif packet.source == 4001 and data[:2] == b"\x80\x03":
            words = [
                int.from_bytes(data[i : i + 4], "big") for i in range(16, len(data), 4)
            ]
            index = 0
            while index < len(words):
                start = words[index] & 0x7FFFFFFF
                end = start
                if words[index] & 0x80000000:
                    index += 1
                    if index == len(words):
                        raise EvidenceError("truncated NAK loss range")
                    end = words[index]
                span = (end - start) & 0x7FFFFFFF
                for seq, gap in tuple(gaps.items()):
                    if (seq - start) & 0x7FFFFFFF <= span and gap.nak_ms is None:
                        gaps[seq] = replace(gap, nak_ms=packet.at_ms)
                index += 1
    return tuple(gaps.values())


def analyze(path: Path) -> FreezeTiming:
    log = path.with_suffix(".capture.log")
    if "0 packets dropped by kernel" not in log.read_text():
        raise EvidenceError(f"freeze capture drops unproven: {path}")
    completed = subprocess.run(
        [
            "tshark",
            "-n",
            "-r",
            str(path),
            "-Y",
            "udp && udp.payload",
            "-T",
            "fields",
            "-e",
            "frame.time_epoch",
            "-e",
            "udp.srcport",
            "-e",
            "udp.dstport",
            "-e",
            "udp.payload",
        ],
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    if completed.returncode:
        raise EvidenceError(f"tshark failed: {completed.stderr}")
    packets = tuple(
        Packet(
            float(parts[0]) * 1000,
            int(parts[1]),
            int(parts[2]),
            bytes.fromhex(parts[3].replace(":", "")),
        )
        for line in completed.stdout.splitlines()
        if len(parts := line.split("\t")) == 4
    )
    return summarize(packets, hashlib.sha256(path.read_bytes()).hexdigest())


def summarize(packets: tuple[Packet, ...], digest: str) -> FreezeTiming:
    gaps = gaps_from_packets(packets)
    nak = [gap.nak_ms - gap.detected_ms for gap in gaps if gap.nak_ms is not None]
    repair = [
        gap.repaired_ms - gap.detected_ms for gap in gaps if gap.repaired_ms is not None
    ]
    if not gaps:
        raise EvidenceError("freeze capture lacks observed gaps")
    lower_bounds = [
        (gap.repaired_ms if gap.repaired_ms is not None else packets[-1].at_ms)
        - gap.detected_ms
        for gap in gaps
    ]
    return FreezeTiming(
        gaps=len(gaps),
        nak_observed=len(nak),
        repaired=len(repair),
        censored=len(gaps) - len(repair),
        first_nak_ms=statistics.median(nak) if nak else None,
        recovery_ms=statistics.median(repair) if repair else None,
        recovery_lower_bound_ms=statistics.median(lower_bounds),
        recovery_upper_bound_ms=upper
        if math.isfinite(
            upper := statistics.median(
                gap.repaired_ms - gap.detected_ms
                if gap.repaired_ms is not None
                else math.inf
                for gap in gaps
            )
        )
        else None,
        pcap_sha256=digest,
    )
