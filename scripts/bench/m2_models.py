import re
from typing import Literal

from report import Document, Stats


class Outcome(Document):
    run_index: int
    settled: bool
    reason: str | None
    goodput_bps: float


class CellResult(Document):
    candidate: str
    scenario: str
    receiver: str
    variant: str
    outcomes: tuple[Outcome, ...]
    settle_rate: float
    goodput: Stats


class InflightComparison(Document):
    scenario: Literal["A", "B1", "C", "G"]
    on: Stats
    off: Stats
    on_settle_rate: float
    off_settle_rate: float
    settle_rate_delta: float
    regression: bool


class NakOffComparison(Document):
    scenario: Literal["A", "F", "G"]
    enhanced: Stats
    adaptive: Stats
    passes: bool


class HsrspResult(Document):
    receiver: str = ""
    expected: bool
    nak_report: bool | None
    status_line: str | None


class RexmitResult(Document):
    data_packets: int
    originals: int
    retransmissions: int
    flagged_originals: int
    unflagged_retransmissions: int
    passed: bool
    pcap_sha256: str


class M2Summary(Document):
    schema_version: Literal[1] = 1
    spike: Literal["M2"] = "M2"
    manifest_sha256: str
    cells: tuple[CellResult, ...]
    inflight: tuple[InflightComparison, ...]
    nak_off: tuple[NakOffComparison, ...]
    rexmit: RexmitResult
    hsrsp: tuple[HsrspResult, ...]
    k_cap: Literal[1, 3]
    nak_off_outcome: Literal["adaptive signals compensate", "present"]
    warnings: tuple[str, ...]


def finite(value: float | str | None) -> float:
    if not isinstance(value, float):
        raise ValueError("M2 requires finite statistics")
    return value


def regression(on: Stats, off: Stats) -> bool:
    return (
        finite(on.median) < 0.95 * finite(off.median)
        and finite(on.ci_upper) < finite(off.ci_lower)
    )


def hsrsp(text: str, expected: bool) -> HsrspResult:
    lines = tuple(
        match.group(0)
        for line in text.splitlines()
        if (match := re.search(r"receiver: nak_report=(?:on|off)\b.*", line))
    )
    observed = {
        match.group(1) == "on"
        for line in lines
        if (match := re.search(r"receiver: nak_report=(on|off)\b", line))
    }
    value = next(iter(observed)) if len(observed) == 1 else None
    if value != expected:
        value = None
    return HsrspResult(
        expected=expected,
        nak_report=value,
        status_line=lines[-1] if len(lines) == 1 and value is not None else None,
    )
