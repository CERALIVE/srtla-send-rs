#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic>=2,<3"]
# ///
# ─── How to run ───
# uv run scripts/bench/check_rexmit_bit.py caller-srt.pcap
# Exit 0: visible; 1: not visible; 2: capture/tool error (not a visibility result).
from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

from pydantic import BaseModel, ConfigDict


@dataclass(frozen=True, slots=True)
class Packet:
    source_port: int
    payload: bytes


class Visibility(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    data_packets: int
    originals: int
    retransmissions: int
    flagged_originals: int
    unflagged_retransmissions: int
    passed: bool
    pcap_sha256: str | None = None


def classify(packets: Iterable[Packet]) -> Visibility:
    seen: set[tuple[int, int, int]] = set()
    originals = repeats = bad_originals = bad_repeats = 0
    for packet in packets:
        data = packet.payload
        if len(data) < 16 or data[0] & 0x80:
            continue
        key = (packet.source_port, int.from_bytes(data[12:16], "big"),
               int.from_bytes(data[:4], "big"))
        flag = bool(data[4] & 0x04)
        if key in seen:
            repeats += 1
            bad_repeats += int(not flag)
        else:
            seen.add(key)
            originals += 1
            bad_originals += int(flag)
    return Visibility(
        data_packets=originals + repeats, originals=originals,
        retransmissions=repeats, flagged_originals=bad_originals,
        unflagged_retransmissions=bad_repeats,
        passed=originals + repeats >= 1000 and repeats > 0
        and bad_originals == 0 and bad_repeats == 0,
    )


@dataclass(frozen=True, slots=True)
class CaptureError(Exception):
    detail: str

    def __str__(self) -> str:
        return self.detail


def analyze(path: Path) -> Visibility:
    drops = re.findall(r"(?m)^(\d+) packets dropped by kernel\s*$",
                       path.with_suffix(".capture.log").read_text())
    if drops != ["0"]:
        raise CaptureError(f"zero capture drops unproven: {path}")
    result = subprocess.run(
        ["tshark", "-n", "-r", str(path), "-Y", "udp.dstport == 5555 && udp.payload",
         "-T", "fields", "-e", "udp.srcport", "-e", "udp.payload"],
        capture_output=True, text=True, timeout=120, check=False,
    )
    if result.returncode:
        raise CaptureError(result.stderr)
    packets: list[Packet] = []
    for line in result.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) != 2:
            raise CaptureError(f"invalid tshark row: {line[:100]}")
        packets.append(Packet(int(parts[0]), bytes.fromhex(parts[1].replace(":", ""))))
    return classify(packets).model_copy(
        update={"pcap_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    )


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_rexmit_bit.py caller-srt.pcap", file=sys.stderr)
        return 2
    try:
        result = analyze(Path(sys.argv[1]))
    except (CaptureError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(str(error), file=sys.stderr)
        return 2
    print(result.model_dump_json(indent=2))
    return int(not result.passed)


if __name__ == "__main__":
    raise SystemExit(main())
