import hashlib
import re
import subprocess
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

from report import Document, EvidenceError


@dataclass(frozen=True, slots=True)
class Packet:
    link: int
    time_s: float
    source_port: int
    destination_port: int
    payload: bytes


class WireEvidence(Document):
    complete_groups: tuple[str, ...]
    group_completion_s: tuple[float, ...]
    keepalive_requests: tuple[int, ...]
    keepalive_echoes: tuple[int, ...]
    keepalive_lengths: tuple[int, ...]
    mismatched_echoes: int


def classify(packets: Sequence[Packet], links: int) -> WireEvidence:
    seeds: dict[tuple[int, int], bytes] = {}
    groups: dict[bytes, set[int]] = {}
    pending: dict[tuple[int, int], bytes] = {}
    keepalives: dict[tuple[int, int], list[bytes]] = {}
    completed: dict[bytes, float] = {}
    requests, echoes = [0]*links, [0]*links
    lengths: set[int] = set()
    mismatched = 0
    for packet in sorted(packets, key=lambda p: p.time_s):
        data = packet.payload
        outgoing = packet.destination_port == 5000
        peer = (packet.link, packet.source_port if outgoing else packet.destination_port)
        match (data[:2], outgoing):
            case (b"\x92\x00", True) if len(data) == 258:
                seeds[peer] = data[2:130]
            case (b"\x92\x01", False) if len(data) == 258:
                if seeds.get(peer) == data[2:130]:
                    groups.setdefault(data[2:], set())
            case (b"\x92\x01", True) if len(data) == 258:
                if data[2:] in groups:
                    pending[peer] = data[2:]
            case (b"\x92\x02", False) if not any(data[2:]):
                group = pending.pop(peer, None)
                if group is not None:
                    groups[group].add(packet.link)
                    if len(groups[group]) == links:
                        completed.setdefault(group, packet.time_s)
            case (b"\x90\x00", True):
                requests[packet.link] += 1
                lengths.add(len(data))
                keepalives.setdefault(peer, []).append(data)
            case (b"\x90\x00", False):
                sent = keepalives.get(peer, [])
                if data in sent:
                    sent.remove(data)
                    echoes[packet.link] += 1
                else:
                    mismatched += 1
            case _:
                continue
    return WireEvidence(complete_groups=tuple(hashlib.sha256(group).hexdigest() for group in completed),
                        group_completion_s=tuple(completed.values()), keepalive_requests=tuple(requests),
                        keepalive_echoes=tuple(echoes), keepalive_lengths=tuple(sorted(lengths)),
                        mismatched_echoes=mismatched)


def read_capture(path: Path, link: int) -> tuple[Packet, ...]:
    drops = re.findall(r"(?m)^(\d+) packets dropped by kernel\s*$", path.with_suffix(".capture.log").read_text())
    if drops != ["0"]:
        raise EvidenceError(f"zero capture drops unproven: {path}")
    result = subprocess.run(["tshark", "-n", "-r", str(path), "-Y", "udp.payload",
                             "-T", "fields", "-e", "frame.time_epoch", "-e", "udp.srcport",
                             "-e", "udp.dstport", "-e", "udp.length", "-e", "udp.payload"],
                            capture_output=True, text=True, timeout=120, check=False)
    if result.returncode:
        raise EvidenceError(f"tshark failed: {result.stderr}")
    packets = []
    for line in result.stdout.splitlines():
        fields = line.split("\t")
        if len(fields) != 5:
            raise EvidenceError(f"malformed capture row: {line}")
        payload = bytes.fromhex(fields[4].replace(":", ""))
        if len(payload) != int(fields[3]) - 8:
            raise EvidenceError("truncated control capture")
        packets.append(Packet(link, float(fields[0]), int(fields[1]), int(fields[2]), payload))
    return tuple(packets)
