#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///

# ─── How to run ───
# 1. Install uv (if not installed):
#      curl -LsSf https://astral.sh/uv/install.sh | sh
# 2. Run directly (no venv, no pip install needed):
#      uv run scripts/bench/decide.py --self-test
# 3. Or make executable and run:
#      chmod +x decide.py && ./decide.py --help
# ──────────────────

from __future__ import annotations

import argparse
import math
import subprocess
import sys
import tempfile
import unittest
from dataclasses import dataclass
from itertools import combinations
from pathlib import Path
from typing import Annotated, ClassVar, Final, Literal, assert_never

from pydantic import BaseModel, ConfigDict, Field, ValidationError

# allow: SIZE_OK — mandated standalone script with embedded literal fixtures/tests;
# splitting boundary models from the rule would introduce a local runtime dependency.
type Count = Annotated[int, Field(ge=0, strict=True)]
type Finite = Annotated[float, Field(allow_inf_nan=False)]
type Rate = Annotated[Finite, Field(ge=0, le=1)]
LEGACY: Final = ("classic", "enhanced", "rtt-threshold", "edpf")
CANDIDATES: Final = ("adaptive", *LEGACY)
SCENARIOS: Final = ("A", "B1", "B2", "C", "D", "E", "F", "G", "H", "I", "K")
TUNABLES: Final[dict[str, int]] = dict(
    zip(LEGACY + ("adaptive",), (0, 11, 12, 14, 32), strict=True)
)


class Document(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(
        frozen=True, strict=True, allow_inf_nan=False
    )


class Comparison(Document):
    n: Count
    dropped_indices: tuple[Count, ...]
    ci_lower: Finite | None
    ci_upper: Finite | None
    viewer_loss_delta_pp: Finite
    recovery_ratio: Annotated[Finite, Field(ge=0)] | None
    nonfinite_recovery: bool
    episode_mismatch: bool


class Evidence(Document):
    n: Count
    run_indices: tuple[Count, ...]
    goodput_median: Annotated[Finite, Field(ge=0)]
    viewer_loss_median: Rate
    graded_episodes: Count
    nonrecovered_rate: Rate | None
    comparisons: dict[str, Comparison]
    integrity_errors: tuple[str, ...] = ()
    switch_count: Annotated[Finite, Field(ge=0)] | None = None
    checks: dict[str, Rate | None] = Field(default_factory=dict)


class Group(Document):
    sink: Literal["slt", "sls"] = "slt"
    cell_id: str = ""
    lineage: str = ""
    covering: bool = True
    campaign: str
    scenario: str
    receiver: str
    profile: str
    cells: dict[str, Evidence]

    @property
    def key(self) -> str:
        if self.cell_id:
            return "/".join((self.campaign, self.cell_id))
        return "/".join((self.campaign, self.scenario, self.receiver, self.profile))


class Summary(Document):
    schema_version: Literal[1]
    groups: tuple[Group, ...]


def metric_groups(summary: Summary) -> tuple[Group, ...]:
    return tuple(
        g for g in summary.groups if g.sink == "slt" and "--sls:" not in g.cell_id
    )


@dataclass(frozen=True, slots=True)
class RuleRequest:
    candidates: tuple[str, ...] = CANDIDATES
    scenarios: tuple[str, ...] = SCENARIOS
    n: int = 10


class ScenarioVerdict(Document):
    best: str | None
    covered_by: tuple[str, ...]
    ci_lower: dict[str, float | None]
    viewer_loss_delta_pp: dict[str, float | None]
    recovery_ratio: dict[str, float | None]
    nonrecovered_rate: dict[str, float | None]


class Verdict(Document):
    verdict: Literal["d1", "ablation"] | None
    reason: Literal["insufficient_evidence"] | None = None
    uncovered_scenarios: tuple[str, ...] = ()
    errors: tuple[str, ...] = ()
    scenarios: dict[str, ScenarioVerdict] = Field(default_factory=dict)
    reported_checks: dict[str, dict[str, dict[str, float | None]]] = Field(
        default_factory=dict
    )
    shipped_modes: tuple[str, ...] | None = None
    retired_modes: tuple[str, ...] | None = None
    default_candidate: str | None = None
    objectives: dict[str, float] = Field(default_factory=dict)
    rejected_configurations: tuple[str, ...] = ()

    def to_json(self) -> str:
        excluded: set[str] = set()
        if self.verdict is None or self.verdict == "ablation":
            excluded.update(("shipped_modes", "retired_modes", "default_candidate"))
        return self.model_dump_json(indent=2, exclude=excluded) + "\n"


def evidence_errors(group: Group, request: RuleRequest) -> tuple[str, ...]:
    errors: list[str] = []
    for name in request.candidates:
        cell = group.cells.get(name)
        label = f"{group.key}/{name}"
        if cell is None:
            errors.append(f"missing cell {label}")
        elif cell.n != request.n:
            errors.append(f"{label}: n={cell.n}, required n={request.n}")
        elif len(cell.run_indices) != cell.n or len(set(cell.run_indices)) != cell.n:
            errors.append(f"{label}: duplicate/missing run_index")
        elif set(cell.run_indices) != set(range(request.n)):
            errors.append(f"{label}: run_indices must cover 0..{request.n - 1}")
    return tuple(errors)


def covers(cell: Evidence, best: Evidence, pair: Comparison) -> bool:
    if (
        cell.n != best.n
        or pair.n != best.n
        or pair.dropped_indices
        or cell.integrity_errors
        or best.integrity_errors
        or pair.episode_mismatch
        or pair.nonfinite_recovery
        or set(cell.run_indices) != set(best.run_indices)
        or pair.ci_lower is None
        or pair.ci_upper is None
        or pair.ci_lower > pair.ci_upper
        or pair.ci_lower < 0.95
        or pair.viewer_loss_delta_pp > 0.1
        or best.goodput_median <= 0
        or 100 * (cell.viewer_loss_median - best.viewer_loss_median) > 0.1
    ):
        return False
    if cell.graded_episodes != best.graded_episodes:
        return False
    if best.graded_episodes:
        return (
            pair.recovery_ratio is not None
            and pair.recovery_ratio <= 1.10
            and cell.nonrecovered_rate is not None
            and best.nonrecovered_rate is not None
            and cell.nonrecovered_rate <= best.nonrecovered_rate
        )
    return True


def evaluate_group(group: Group, request: RuleRequest) -> ScenarioVerdict:
    eligible = [
        name
        for name in request.candidates
        if name in group.cells and group.cells[name].n == request.n
    ]
    best = max(
        eligible, key=lambda name: group.cells[name].goodput_median, default=None
    )
    pairs = (
        {name: group.cells[name].comparisons.get(best) for name in eligible}
        if best is not None
        else {}
    )
    covered = tuple(
        name
        for name in eligible
        if best is not None
        and (pair := pairs.get(name)) is not None
        and (
            group.scenario not in ("C", "D", "E", "H", "K")
            or group.cells[name].graded_episodes > 0
        )
        and covers(group.cells[name], group.cells[best], pair)
    )
    return ScenarioVerdict(
        best=best,
        covered_by=covered,
        ci_lower={
            name: pair.ci_lower if pair else None for name, pair in pairs.items()
        },
        viewer_loss_delta_pp={
            name: pair.viewer_loss_delta_pp if pair else None
            for name, pair in pairs.items()
        },
        recovery_ratio={
            name: pair.recovery_ratio if pair else None for name, pair in pairs.items()
        },
        nonrecovered_rate={
            name: group.cells[name].nonrecovered_rate for name in eligible
        },
    )


def decide(summary: Summary, request: RuleRequest) -> Verdict:
    scenarios = tuple(s for s in request.scenarios if s not in ("J", "L"))
    eligible_groups = metric_groups(summary)
    groups = tuple(g for g in eligible_groups if g.scenario in scenarios)
    errors = [error for g in groups for error in evidence_errors(g, request)]
    if (
        request.n <= 0
        or not scenarios
        or not request.candidates
        or len(set(request.candidates)) != len(request.candidates)
        or len(set(request.scenarios)) != len(request.scenarios)
        or any(c not in TUNABLES for c in request.candidates)
    ):
        errors.append("invalid candidates/scenarios/n for d1")
    if len({g.key for g in eligible_groups}) != len(eligible_groups):
        errors.append("duplicate campaign/scenario/receiver/profile group")
    missing = set(scenarios) - {g.scenario for g in groups}
    errors.extend(f"missing scenario {s}" for s in sorted(missing))
    outcomes = {g.key: evaluate_group(g, request) for g in groups}
    uncovered = missing | {g.scenario for g in groups if not outcomes[g.key].covered_by}
    uncovered.update(g.scenario for g in groups if evidence_errors(g, request))
    reported = {
        g.key: {
            c: dict(e.checks) for c, e in g.cells.items() if c in request.candidates
        }
        for g in eligible_groups
        if g.scenario in ("J", "L")
    }
    if errors or uncovered:
        return Verdict(
            verdict=None,
            reason="insufficient_evidence",
            errors=tuple(errors),
            uncovered_scenarios=tuple(sorted(uncovered)),
            scenarios=outcomes,
            reported_checks=reported,
        )
    choices = [
        subset
        for size in range(1, len(request.candidates) + 1)
        for subset in combinations(request.candidates, size)
        if all(set(subset).intersection(v.covered_by) for v in outcomes.values())
    ]

    def rank(subset: tuple[str, ...]) -> tuple[int, bool, int, float, tuple[str, ...]]:
        switches = sum(
            switch if (switch := g.cells[c].switch_count) is not None else math.inf
            for g in groups
            for c in subset
        )
        return (
            len(subset),
            "adaptive" not in subset,
            sum(TUNABLES[c] for c in subset),
            switches,
            subset,
        )

    shipped = min(choices, key=rank)
    default = min(shipped, key=lambda c: (c != "adaptive", TUNABLES[c]))
    return Verdict(
        verdict="d1",
        shipped_modes=shipped,
        retired_modes=tuple(
            c for c in LEGACY if c in request.candidates and c not in shipped
        ),
        default_candidate=default,
        scenarios=outcomes,
        reported_checks=reported,
    )


def ablation(summary: Summary, request: RuleRequest, baseline: str) -> Verdict:
    targets = tuple(s for s in request.scenarios if s != "A")
    objectives: dict[str, float] = {}
    rejected: list[str] = []
    for name in request.candidates:
        selected = tuple(
            g
            for g in metric_groups(summary)
            if name in g.cells and g.scenario in (*targets, "A")
        )
        contexts = {(g.campaign, g.receiver, g.profile) for g in selected}
        ratios: list[float] = []
        valid = bool(targets and contexts and request.n > 0)
        for context in contexts:
            for scenario in (*targets, "A"):
                matches = [
                    g
                    for g in selected
                    if (g.campaign, g.receiver, g.profile) == context
                    and g.scenario == scenario
                ]
                if len(matches) != 1:
                    valid = False
                    continue
                g = matches[0]
                cell, best = g.cells[name], g.cells.get(baseline)
                pair = cell.comparisons.get(baseline)
                if (
                    best is None
                    or pair is None
                    or evidence_errors(
                        g, RuleRequest((name, baseline), (scenario,), request.n)
                    )
                    or cell.integrity_errors
                    or best.integrity_errors
                    or pair.n != request.n
                    or pair.dropped_indices
                    or pair.episode_mismatch
                    or pair.nonfinite_recovery
                    or set(cell.run_indices) != set(best.run_indices)
                    or best.goodput_median <= 0
                    or cell.goodput_median <= 0
                ):
                    valid = False
                elif scenario == "A":
                    valid = (
                        valid and pair.ci_lower is not None and pair.ci_lower >= 0.98
                    )
                else:
                    ratios.append(cell.goodput_median / best.goodput_median)
        if valid and ratios:
            objectives[name] = math.exp(sum(math.log(r) for r in ratios) / len(ratios))
        else:
            rejected.append(name)
    return Verdict(
        verdict="ablation" if objectives else None,
        reason=None if objectives else "insufficient_evidence",
        objectives=objectives,
        rejected_configurations=tuple(rejected),
    )


# Literal synthetic summary documents, independent of campaign/report generation.
# Each test explicitly selects the scenarios present in its fixture.
SYNTHETIC_SUMMARIES: Final = (
    r"""{"schema_version":1,"groups":[{"campaign":"synthetic","scenario":"A","receiver":"ceralive","profile":"production","cells":{
      "adaptive":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":100,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{"adaptive":{"n":10,"dropped_indices":[],"ci_lower":1,"ci_upper":1,"viewer_loss_delta_pp":0,"recovery_ratio":null,"nonfinite_recovery":false,"episode_mismatch":false}}},
      "classic":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":90,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{"adaptive":{"n":10,"dropped_indices":[],"ci_lower":0.9,"ci_upper":0.9,"viewer_loss_delta_pp":0,"recovery_ratio":null,"nonfinite_recovery":false,"episode_mismatch":false}}},
      "enhanced":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":90,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "rtt-threshold":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":90,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "edpf":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":90,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}}
    }}]}""",
    r"""{"schema_version":1,"groups":[{"campaign":"synthetic","scenario":"A","receiver":"ceralive","profile":"production","cells":{
      "adaptive":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":100,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{"adaptive":{"n":10,"dropped_indices":[],"ci_lower":1,"ci_upper":1,"viewer_loss_delta_pp":0,"recovery_ratio":null,"nonfinite_recovery":false,"episode_mismatch":false}}},
      "classic":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "enhanced":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "rtt-threshold":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "edpf":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}}
    }},{"campaign":"synthetic","scenario":"G","receiver":"ceralive","profile":"production","cells":{
      "adaptive":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "classic":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":100,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{"classic":{"n":10,"dropped_indices":[],"ci_lower":1,"ci_upper":1,"viewer_loss_delta_pp":0,"recovery_ratio":null,"nonfinite_recovery":false,"episode_mismatch":false}}},
      "enhanced":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "rtt-threshold":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}},
      "edpf":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":0,"nonrecovered_rate":null,"comparisons":{}}
    }}]}""",
    r"""{"schema_version":1,"groups":[{"campaign":"synthetic","scenario":"E","receiver":"ceralive","profile":"production","cells":{
      "adaptive":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":100,"viewer_loss_median":0,"graded_episodes":10,"nonrecovered_rate":0,"comparisons":{"adaptive":{"n":9,"dropped_indices":[9],"ci_lower":1,"ci_upper":1,"viewer_loss_delta_pp":0,"recovery_ratio":1,"nonfinite_recovery":false,"episode_mismatch":false}}},
      "classic":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":10,"nonrecovered_rate":0,"comparisons":{}},
      "enhanced":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":10,"nonrecovered_rate":0,"comparisons":{}},
      "rtt-threshold":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":10,"nonrecovered_rate":0,"comparisons":{}},
      "edpf":{"n":10,"run_indices":[0,1,2,3,4,5,6,7,8,9],"goodput_median":80,"viewer_loss_median":0,"graded_episodes":10,"nonrecovered_rate":0,"comparisons":{}}
    }}]}""",
)


class DecisionTests(unittest.TestCase):
    def test_sls_cannot_supply_a_missing_covering_scenario(self) -> None:
        # Given complete metric A and an SLS-only G, even incorrectly marked covering.
        original = Summary.model_validate_json(SYNTHETIC_SUMMARIES[1])
        for covering in (True, False):
            sls = original.groups[1].model_copy(
                update={
                    "sink": "sls",
                    "covering": covering,
                    "cell_id": "ours-new@--G@--production--sls:4002--fec:off",
                }
            )
            summary = original.model_copy(update={"groups": (original.groups[0], sls)})
            # When D-1 runs, then SLS cannot satisfy the missing G obligation.
            verdict = decide(summary, RuleRequest(scenarios=("A", "G")))
            self.assertIsNone(verdict.verdict)
            self.assertIn("G", verdict.uncovered_scenarios)
            self.assertNotIn(sls.key, verdict.scenarios)

    def test_loader_preserves_lineage_cell_identity_and_covering(self) -> None:
        raw = """{"schema_version":1,"groups":[{"campaign":"unit","scenario":"A",
          "receiver":"arbitrary-label","profile":"production","lineage":"ours-old",
          "cell_id":"ours-old@--A@--production--slt:4002--fec:off","covering":false,"cells":{}}]}"""
        summary = Summary.model_validate_json(raw)
        self.assertEqual(summary.groups[0].lineage, "ours-old")
        self.assertFalse(summary.groups[0].covering)
        self.assertEqual(
            summary.groups[0].key, "unit/ours-old@--A@--production--slt:4002--fec:off"
        )

    def test_all_eleven_scenarios_when_adaptive_covers_full_matrix(self) -> None:
        # Given the literal input repeated across the complete D-1 scenario matrix.
        source = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0]).groups[0]
        cells = {
            name: cell.model_copy(
                update={
                    "graded_episodes": 10,
                    "nonrecovered_rate": 0.0,
                    "comparisons": {
                        reference: pair.model_copy(update={"recovery_ratio": 1.0})
                        for reference, pair in cell.comparisons.items()
                    },
                }
            )
            for name, cell in source.cells.items()
        }
        summary = Summary(
            schema_version=1,
            groups=tuple(
                source.model_copy(update={"scenario": scenario, "cells": cells})
                for scenario in SCENARIOS
            ),
        )
        # When the default request checks all eleven scenarios with all five arms.
        verdict = decide(summary, RuleRequest())
        # Then the complete matrix, not merely the A fixture, retires all legacy modes.
        self.assertEqual(len(verdict.scenarios), 11)
        self.assertEqual(verdict.retired_modes, LEGACY)

    def test_literal_summary_clis_match_the_three_retention_outcomes(self) -> None:
        # Given the three hand-authored JSON summaries.
        cases = (
            ("A", 0, ("adaptive",)),
            ("A,G", 0, ("adaptive", "classic")),
            ("E", 1, None),
        )
        for raw, (scenarios, exit_code, shipped) in zip(
            SYNTHETIC_SUMMARIES, cases, strict=True
        ):
            with (
                self.subTest(scenarios=scenarios),
                tempfile.TemporaryDirectory() as directory,
            ):
                source, output = (
                    Path(directory) / "summary.json",
                    Path(directory) / "verdict.json",
                )
                _ = source.write_text(raw, encoding="utf-8")
                # When the real CLI applies D-1 to that fixture.
                process = subprocess.run(
                    [
                        sys.executable,
                        __file__,
                        "--summary",
                        str(source),
                        "--out",
                        str(output),
                        "--rule",
                        "d1",
                        "--scenarios",
                        scenarios,
                        "--n",
                        "10",
                    ],
                    capture_output=True,
                    text=True,
                    timeout=30,
                    check=False,
                )
                # Then both the exit status and serialized retention decision agree.
                self.assertEqual(process.returncode, exit_code, process.stderr)
                verdict = Verdict.model_validate_json(
                    output.read_text(encoding="utf-8")
                )
                self.assertEqual(verdict.shipped_modes, shipped)
                if exit_code:
                    self.assertIsNone(verdict.retired_modes)
                    self.assertEqual(verdict.reason, "insufficient_evidence")

    def test_partial_summary_refuses_default_matrix(self) -> None:
        # Given a valid A-only document, not a full campaign.
        summary = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0])
        # When the normal D-1 defaults are used, then absent scenarios block retirement.
        verdict = decide(summary, RuleRequest())
        self.assertIsNone(verdict.verdict)
        self.assertIn("K", verdict.uncovered_scenarios)

    def test_tie_prefers_adaptive_then_fewer_tunables(self) -> None:
        # Given tied arms with identical complete evidence.
        group = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0]).groups[0]
        cell = group.cells["adaptive"]
        pair = cell.comparisons["adaptive"]
        tied = cell.model_copy(update={"comparisons": {c: pair for c in CANDIDATES}})
        summary = Summary(
            schema_version=1,
            groups=(group.model_copy(update={"cells": {c: tied for c in CANDIDATES}}),),
        )
        # When adaptive is eligible, then it wins the singleton tie.
        self.assertEqual(
            decide(summary, RuleRequest(scenarios=("A",))).shipped_modes, ("adaptive",)
        )
        # When considering only legacy arms, then the zero-tunable classic wins.
        self.assertEqual(
            decide(summary, RuleRequest(LEGACY, ("A",))).shipped_modes, ("classic",)
        )

    def test_boundaries_when_ratio_loss_or_recovery_changes(self) -> None:
        # Given complete paired evidence with graded episodes.
        cell = (
            Summary.model_validate_json(SYNTHETIC_SUMMARIES[2])
            .groups[0]
            .cells["adaptive"]
        )
        pair = cell.comparisons["adaptive"].model_copy(
            update={"n": 10, "dropped_indices": ()}
        )
        cases = (
            ({"ci_lower": 0.949}, False),
            ({"ci_lower": 0.951}, True),
            ({"ci_lower": 0.95}, True),
            ({"viewer_loss_delta_pp": 0.1}, True),
            ({"viewer_loss_delta_pp": 0.10001}, False),
            ({"recovery_ratio": 1.10}, True),
            ({"recovery_ratio": 1.10001}, False),
            ({"recovery_ratio": None}, False),
            ({"nonfinite_recovery": True}, False),
            ({"episode_mismatch": True}, False),
            ({"n": 9}, False),
        )
        for change, expected in cases:
            with self.subTest(change=change):
                # When one boundary is changed, then coverage follows D-1.
                self.assertEqual(
                    covers(cell, cell, pair.model_copy(update=change)), expected
                )

    def test_failure_rate_when_candidate_loses_one_episode(self) -> None:
        # Given valid recovery ratios but a new failed episode.
        cell = (
            Summary.model_validate_json(SYNTHETIC_SUMMARIES[2])
            .groups[0]
            .cells["adaptive"]
        )
        pair = cell.comparisons["adaptive"].model_copy(
            update={"n": 10, "dropped_indices": ()}
        )
        # When compared to a fully recovered best, then coverage is refused.
        self.assertFalse(
            covers(cell.model_copy(update={"nonrecovered_rate": 0.1}), cell, pair)
        )

    def test_cli_refuses_when_one_cell_has_nine_runs(self) -> None:
        # Given a literal summary with one insufficient cell.
        summary = SYNTHETIC_SUMMARIES[0].replace('"n":10', '"n":9', 1)
        with tempfile.TemporaryDirectory() as directory:
            source, output = (
                Path(directory) / "summary.json",
                Path(directory) / "verdict.json",
            )
            _ = source.write_text(summary, encoding="utf-8")
            # When invoked through the real standalone CLI.
            process = subprocess.run(
                [
                    sys.executable,
                    __file__,
                    "--summary",
                    str(source),
                    "--out",
                    str(output),
                    "--rule",
                    "d1",
                    "--scenarios",
                    "A",
                    "--n",
                    "10",
                ],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Then the exit and artifact refuse retirement and identify the cell.
            self.assertNotEqual(process.returncode, 0)
            self.assertIn("synthetic/A/ceralive/production/adaptive", process.stderr)
            verdict = Verdict.model_validate_json(output.read_text(encoding="utf-8"))
            self.assertIsNone(verdict.verdict)
            self.assertIsNone(verdict.retired_modes)

    def test_ablation_rejects_when_a_is_missing(self) -> None:
        # Given a configuration measured on G but not its regression control A.
        source = Summary.model_validate_json(SYNTHETIC_SUMMARIES[1])
        summary = Summary(schema_version=1, groups=(source.groups[1],))
        # When ablation evaluates the configuration.
        result = ablation(summary, RuleRequest(("classic",), ("G",)), "classic")
        # Then the configuration is rejected, not scored on its targets alone.
        self.assertIsNone(result.verdict)
        self.assertEqual(result.rejected_configurations, ("classic",))

    def test_ablation_rejects_when_own_a_regresses_beyond_predeclared_guard(
        self,
    ) -> None:
        # Given C2's predeclared 0.98 A guard and an otherwise improving target.
        group = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0]).groups[0]
        baseline = group.cells["adaptive"]
        pair = baseline.comparisons["adaptive"].model_copy(update={"ci_lower": 0.979})
        sweep = baseline.model_copy(
            update={"goodput_median": 97.9, "comparisons": {"adaptive": pair}}
        )
        summary = Summary(
            schema_version=1,
            groups=(
                group.model_copy(
                    update={"cells": {"adaptive": baseline, "sweep": sweep}}
                ),
                group.model_copy(
                    update={
                        "scenario": "G",
                        "cells": {
                            "adaptive": baseline,
                            "sweep": baseline.model_copy(
                                update={"goodput_median": 110}
                            ),
                        },
                    }
                ),
            ),
        )
        # When the sweep is scored, then its own measured A prevents acceptance.
        self.assertIsNone(
            ablation(summary, RuleRequest(("sweep",), ("G",)), "adaptive").verdict
        )

    def test_geometric_objective_when_two_targets_disagree(self) -> None:
        # Given hand-selected ratios 1.21 and 0.81, with an independent A control.
        source = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0]).groups[0]
        baseline = source.cells["adaptive"]
        groups = tuple(
            source.model_copy(
                update={
                    "scenario": scenario,
                    "cells": {
                        "adaptive": baseline,
                        "sweep": baseline.model_copy(
                            update={"goodput_median": goodput}
                        ),
                    },
                }
            )
            for scenario, goodput in (("A", 100), ("D", 121), ("G", 81))
        )
        # When the configuration is scored.
        verdict = ablation(
            Summary(schema_version=1, groups=groups),
            RuleRequest(("sweep",), ("D", "G")),
            "adaptive",
        )
        # Then the geometric mean, not the arithmetic mean, determines the objective.
        self.assertAlmostEqual(verdict.objectives["sweep"], 0.99)

    def test_retires_legacy_when_adaptive_covers_every_scenario(self) -> None:
        # Given a literal synthetic summary, not measurements.
        summary = Summary.model_validate_json(SYNTHETIC_SUMMARIES[0])
        # When D-1 evaluates every scenario in this fixture.
        verdict = decide(summary, RuleRequest(scenarios=("A",)))
        # Then only adaptive ships.
        self.assertEqual(verdict.retired_modes, LEGACY)

    def test_retains_classic_when_it_uniquely_covers_g(self) -> None:
        # Given independent A/G measurements.
        summary = Summary.model_validate_json(SYNTHETIC_SUMMARIES[1])
        # When the covering set is selected.
        verdict = decide(summary, RuleRequest(scenarios=("A", "G")))
        # Then both indispensable candidates ship.
        self.assertEqual(verdict.shipped_modes, ("adaptive", "classic"))

    def test_refuses_when_no_candidate_covers_e(self) -> None:
        # Given an incomplete pairing for the highest-goodput arm.
        summary = Summary.model_validate_json(SYNTHETIC_SUMMARIES[2])
        # When E is evaluated.
        verdict = decide(summary, RuleRequest(scenarios=("E",)))
        # Then no retirement decision exists.
        self.assertIsNone(verdict.verdict)
        self.assertEqual(verdict.reason, "insufficient_evidence")
        self.assertEqual(verdict.uncovered_scenarios, ("E",))
        self.assertNotIn('"retired_modes"', verdict.to_json())


class Arguments(argparse.Namespace):
    self_test: bool = False
    summary: Path | None = None
    rule: Literal["d1", "lineage-d1", "ablation"] = "d1"
    candidates: str = ",".join(CANDIDATES)
    scenarios: str = ",".join(SCENARIOS)
    n: int = 10
    baseline: str = "adaptive"
    out: Path | None = None


def main() -> int:
    parser = argparse.ArgumentParser(description="Fail-closed scheduler retention rule")
    _ = parser.add_argument("--self-test", action="store_true")
    _ = parser.add_argument("--summary", type=Path)
    _ = parser.add_argument(
        "--rule", choices=("d1", "lineage-d1", "ablation"), default="d1"
    )
    _ = parser.add_argument("--candidates", default=",".join(CANDIDATES))
    _ = parser.add_argument("--scenarios", default=",".join(SCENARIOS))
    _ = parser.add_argument("--n", type=int, default=10)
    _ = parser.add_argument("--baseline", default="adaptive")
    _ = parser.add_argument("--out", type=Path)
    args = parser.parse_args(namespace=Arguments())
    if args.self_test:
        result = unittest.TextTestRunner(verbosity=2).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(DecisionTests)
        )
        return int(not result.wasSuccessful())
    if args.summary is None or args.out is None:
        parser.error("--summary and --out are required")
    try:
        summary = Summary.model_validate_json(args.summary.read_text(encoding="utf-8"))
        request = RuleRequest(
            tuple(args.candidates.split(",")), tuple(args.scenarios.split(",")), args.n
        )
        match args.rule:
            case "d1" | "lineage-d1":
                verdict = decide(summary, request)
            case "ablation":
                verdict = ablation(summary, request, args.baseline)
            case _:
                assert_never(args.rule)
    except (OSError, ValidationError) as error:
        verdict = Verdict(
            verdict=None, reason="insufficient_evidence", errors=(str(error),)
        )
    try:
        _ = args.out.write_text(verdict.to_json(), encoding="utf-8")
    except OSError as error:
        print(str(error), file=sys.stderr)
        return 1
    if verdict.verdict is None:
        print(verdict.to_json(), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
