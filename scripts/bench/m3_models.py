from typing import Final, Literal

from pydantic import Field

from report import Count, Document, EvidenceError, Hash, Positive, Rate, Stats

SENDERS: Final = ("belabox-c", "irlserver-rust-classic", "irlserver-rust-enhanced", "ours-3.3.0", "ours-new")
FOREIGN: Final = SENDERS[:3]
SCENARIOS: Final = ("B1", "G", "C", "M1")
RECEIVERS: Final = ("ours-old", "ours-new-200")


class Outcome(Document):
    run_index: Count
    settled: bool
    goodput_bps: Positive
    retransmit_ratio: Rate


class CellResult(Document):
    sender: str
    scenario: str
    receiver: str
    outcomes: tuple[Outcome, ...]
    settled_runs: Count
    passing_runs: Count
    goodput: Stats
    sender_sha256: Hash
    receiver_sha256: Hash
    libsrt_tool_sha256: Hash


class Comparison(Document):
    sender: str
    scenario: str
    settled_runs: Count
    passing_runs: Count
    new_median_bps: Positive
    old_median_bps: Positive
    goodput_ratio: Positive
    passed: bool
    disposition: Literal["pass", "known_limitation", "receiver_pr_blocker", "new_sender_failure"]


class SenderVerdict(Document):
    passed: bool = Field(alias="pass")
    failing_scenarios: tuple[str, ...]


class Quadrant(Document):
    sender_population: Literal["existing", "new"]
    receiver: str
    senders: tuple[str, ...]
    measured_runs: Count
    settled_runs: Count
    passing_runs: Count


class Spike(Document):
    spike: Literal["M3"] = "M3"
    ttl_star: Literal[200] = 200
    quadrants: tuple[Quadrant, ...]
    senders: dict[str, SenderVerdict]
    comparisons: tuple[Comparison, ...]
    receiver_pr_blocker: bool
    selected_alternative: str
    foreign_mitigation: str = "the receiver cannot detect sender lineage; the only lever is TTL*/gate, which M1 chose"


class ConformanceRow(Document):
    sender: str
    check: Literal["registration", "keepalive_echo", "receiver_restart"]
    passed: bool
    details: str


class M3Summary(Document):
    schema_version: Literal[1] = 1
    campaign: Literal["m3-interop"] = "m3-interop"
    groups: tuple[()] = ()
    manifest_sha256: Hash
    cells: tuple[CellResult, ...]
    decision: Spike
    conformance: tuple[ConformanceRow, ...]
    input_sha256: dict[str, Hash]
    warnings: tuple[str, ...]


def compare(new: CellResult, old: CellResult) -> Comparison:
    if (new.sender, new.scenario, new.sender_sha256) != (old.sender, old.scenario, old.sender_sha256):
        raise EvidenceError("rollout comparison must use the same sender/scenario/binary")
    a, b = new.goodput.median, old.goodput.median
    if not isinstance(a, float) or not isinstance(b, float) or b <= 0:
        raise EvidenceError("rollout requires finite positive old median goodput")
    passed = new.passing_runs >= 2 and a / b >= 0.95
    disposition: Literal["pass", "known_limitation", "receiver_pr_blocker", "new_sender_failure"] = "pass"
    if not passed:
        if new.sender in FOREIGN:
            disposition = "known_limitation"
        elif new.sender == "ours-3.3.0":
            disposition = "receiver_pr_blocker"
        else:
            disposition = "new_sender_failure"
    return Comparison(sender=new.sender, scenario=new.scenario, settled_runs=new.settled_runs,
                      passing_runs=new.passing_runs, new_median_bps=a, old_median_bps=b,
                      goodput_ratio=a/b, passed=passed, disposition=disposition)
