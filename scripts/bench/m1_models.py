from typing import Literal

from report import Count, Document, Hash, Positive, Rate, Stats


class FreezeTiming(Document):
    gaps: Count
    nak_observed: Count
    repaired: Count
    censored: Count
    first_nak_ms: Positive | None
    recovery_ms: Positive | None
    pcap_sha256: Hash
    recovery_lower_bound_ms: Positive
    recovery_upper_bound_ms: Positive | None


class Outcome(Document):
    run_index: Count
    settled: bool
    reason: str | None
    goodput_bps: Positive
    retrans_ratio: Rate
    fingerprint: Hash
    freeze: FreezeTiming | None = None


class CellResult(Document):
    candidate: str
    scenario: str
    receiver: str
    ttl: Literal[40, 200, 500]
    freeze_enabled: bool
    offered_mbit: int | None
    outcomes: tuple[Outcome, ...]
    goodput: Stats
    retransmit: Stats

    @property
    def core(self) -> bool:
        return self.candidate in ("classic", "enhanced") and self.offered_mbit is None

    @property
    def foreign(self) -> bool:
        return self.candidate in ("belabox-c", "irlserver-rust")

    @property
    def passing_runs(self) -> int:
        limit = 0.1 if self.foreign else 0.05
        return sum(run.settled and run.retrans_ratio <= limit for run in self.outcomes)

    @property
    def passes(self) -> bool:
        return self.passing_runs >= (2 if self.foreign else 3)


class M1Summary(Document):
    schema_version: Literal[1] = 1
    spike: Literal["M1"] = "M1"
    groups: tuple[()] = ()
    manifest_sha256: Hash
    cells: tuple[CellResult, ...]
    warnings: tuple[str, ...]
    freeze_scope: str = "Entire receiver capture, including warmup; gap detection to first NAK / first repair; unrepaired gaps retained as right-censored lower bounds"


class Failure(Document):
    candidate: str
    scenario: str
    ttl: int
    passing_runs: int
    settled_runs: int
    retransmit_percent: tuple[float, ...]
    goodput_mbit: tuple[float, ...]


class FreezeComparison(Document):
    ttl: int
    offered_mbit: int
    enabled_ms: float
    disabled_ms: float | None
    penalty_ms: float | None
    censored: int


class Decision(Document):
    spike: Literal["M1"] = "M1"
    outcome: str
    ttl_star: int
    controller: bool
    freeze_penalty_ms: float
    selected_alternative: str
    rule_branch: str
    sweep_ttl: int
    passing_cells: dict[str, int]
    passing_core_cells: dict[str, int]
    failing_cells: tuple[Failure, ...]
    best_24_mbit_ttl: int
    goodput_gap_percent: float
    controller_nonoverlapping_ci: bool
    freeze_comparisons: tuple[FreezeComparison, ...]
    freeze_cap_applied: bool
    freeze_penalty_interpretation: str = "Maximum identifiable lower bound on median penalty; unresolved censored comparisons remain null, never zero"
    consequences: tuple[str, ...]
