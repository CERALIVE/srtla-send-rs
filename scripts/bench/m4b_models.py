from typing import Literal

from lineage_rule import CellResult, SacrificedCell
from report import Count, Document, Positive, Rate


class Observation(Document):
    run_index: Count
    settled: bool
    retransmit_ratio: Rate | None
    zero_drop_belated: bool | None
    starvation_floor: bool | None
    sink: Literal["slt", "sls"] = "slt"
    conformance: bool | None = None
    goodput_bps: Positive = 0.0
    source: str = ""


class Score(Document):
    n: Count
    required: Count
    settled: Count
    joint_passes: Count
    passed: bool
    failing_gates: tuple[str, ...]
    observations: tuple[Observation, ...]


class LineageCell(Document):
    cell_id: str
    scenario: str
    lineage: str
    sink: str
    port: int
    members: dict[str, Score]


class Failure(Document):
    cell_id: str
    scenario: str
    lineage: str
    sink: str
    port: int
    reason: Literal["lineage gate"] = "lineage gate"
    failing_gates: dict[str, tuple[str, ...]]


class Swap(Document):
    previous: str
    selected: str


class Gate(Document):
    cells: tuple[LineageCell, ...]
    passes_per_member: dict[str, int]
    swap: Swap | None
    failures: tuple[Failure, ...]


class FinalVerdict(Document):
    ship_set: tuple[str, ...]
    base_mode: str
    covered_by_base_pct: float
    sacrificed_cells: tuple[SacrificedCell | Failure, ...]
    cells: dict[str, CellResult]
    refold: Literal[False] = False
    gate: None = None
    lineage_gate: Gate
    lineage_gate_swap: Swap | None
