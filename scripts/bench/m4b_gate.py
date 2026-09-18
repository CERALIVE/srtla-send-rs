#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/m4b_gate.py --verdict BASE --manifest MATRIX --results RESULTS --out FINAL
import argparse
from pathlib import Path
from typing import assert_never

from pydantic import ValidationError

from lineage_rule import LineageVerdict, SacrificedCell
from m4_manifest import M4Manifest
from m4b_manifest import ROOT, build, members
from m4b_models import Failure, FinalVerdict, Gate, LineageCell, Observation, Score, Swap
from report import EvidenceError


def score(runs: tuple[Observation, ...]) -> Score:
    if (
        len(runs) not in (3, 5)
        or {r.run_index for r in runs} != set(range(len(runs)))
        or len({r.sink for r in runs}) != 1
    ):
        raise EvidenceError("complete homogeneous N=3 or N=5 lineage outcomes required")
    required = 4 if len(runs) == 5 else 2
    joint = sum(
        r.settled and r.retransmit_ratio is not None and r.retransmit_ratio <= 0.10
        for r in runs
    )
    failures: list[str] = []
    if any(r.retransmit_ratio is None for r in runs):
        failures.append("retransmit_unknown")
    if joint < required:
        failures.append("joint_settle_retransmit_quota")
    match runs[0].sink:
        case "slt":
            if not all(r.zero_drop_belated is True for r in runs):
                failures.append("post_settle_zero_drop_belated")
            if not all(r.starvation_floor is True for r in runs):
                failures.append("post_settle_starvation_floor")
        case "sls":
            if not all(r.conformance is True for r in runs):
                failures.append("sls_conformance")
        case unreachable:
            assert_never(unreachable)
    return Score(
        n=len(runs),
        required=required,
        settled=sum(r.settled for r in runs),
        joint_passes=joint,
        passed=not failures,
        failing_gates=tuple(failures),
        observations=tuple(sorted(runs, key=lambda r: r.run_index)),
    )


def finalize(base: LineageVerdict, cells: tuple[LineageCell, ...]) -> FinalVerdict:
    selected = members(base)
    if (
        not cells
        or len({c.cell_id for c in cells}) != len(cells)
        or any(set(c.members) != set(selected) for c in cells)
    ):
        raise EvidenceError("lineage cells must contain exactly the selected members")
    passes = {name: sum(c.members[name].passed for c in cells) for name in selected}
    coverage = {
        name: sum(name in c.covered_by for c in base.cells.values() if not c.exempt)
        for name in selected
    }
    winner = min(
        selected,
        key=lambda name: (-passes[name], -coverage[name], name != base.base_mode, name),
    )
    swap = (
        Swap(previous=base.base_mode, selected=winner)
        if winner != base.base_mode
        else None
    )
    failures = tuple(
        Failure(
            cell_id=c.cell_id,
            scenario=c.scenario,
            lineage=c.lineage,
            sink=c.sink,
            port=c.port,
            failing_gates={name: s.failing_gates for name, s in c.members.items()},
        )
        for c in cells
        if not any(s.passed for s in c.members.values())
    )
    obligations = sum(not c.exempt for c in base.cells.values())
    primary_sacrifices = (
        tuple(
            SacrificedCell(
                cell_id=key,
                scenario=c.scenario,
                lineage=c.lineage,
                winner=c.winner,
                base_mode_ratio=c.candidates[winner].goodput_ratio,
                base_mode_ci_lower=c.candidates[winner].ci_lower,
                failing_gates=c.candidates[winner].failing_gates,
                reason="not_covered_by_base" if c.covered_by else "uncovered",
            )
            for key, c in base.cells.items()
            if not c.exempt and winner not in c.covered_by
        )
        if swap
        else base.sacrificed_cells
    )
    return FinalVerdict(
        ship_set=base.ship_set,
        base_mode=winner,
        covered_by_base_pct=100 * coverage[winner] / obligations
        if obligations
        else 100.0,
        sacrificed_cells=(*primary_sacrifices, *failures),
        cells=base.cells,
        lineage_gate=Gate(
            cells=cells, passes_per_member=passes, swap=swap, failures=failures
        ),
        lineage_gate_swap=swap,
    )


class Arguments(argparse.Namespace):
    verdict: Path
    manifest: Path
    results: Path
    out: Path


def main() -> int:
    from m4b_inputs import load

    parser = argparse.ArgumentParser()
    for name in ("verdict", "manifest", "results", "out"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args(namespace=Arguments())
    try:
        base = LineageVerdict.model_validate_json(args.verdict.read_bytes())
        manifest = M4Manifest.model_validate_json(args.manifest.read_bytes())
        primary = M4Manifest.model_validate_json(
            (ROOT / "scripts/bench/manifests/m4a-ours-new.json").read_bytes()
        )
        if manifest != build(base, primary):
            raise EvidenceError("lineage manifest differs from frozen matrix")
        result = finalize(base, load(manifest, args.results))
        args.out.write_text(result.model_dump_json(indent=2) + "\n")
        print(result.lineage_gate.model_dump_json(exclude={"cells"}))
    except (OSError, ValidationError, EvidenceError) as error:
        parser.exit(1, f"{error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
