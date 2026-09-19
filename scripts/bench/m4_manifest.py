#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/m4_manifest.py --generate | --validate MANIFEST

import argparse
from pathlib import Path
from typing import Final, Literal

from pydantic import ValidationError

from lineage_rule import CLI, N, SCENARIOS
from report import (
    Candidate,
    Cell,
    Document,
    EvidenceError,
    Manifest,
    ManifestReceiver,
    stable_identity,
)

OPTIONS: Final = "&nakreport=1&periodicnakgate=1&lossmaxttl=200&reorderfreeze=1"
BASELINES: Final = ("upstream-classic", "upstream-enhanced")
REFERENCES: Final = ("A", "G", "M1", "M4")
FEATURES: Final = (
    "stall",
    "loss",
    "queue",
    "deadline",
    "rejoin",
    "sole",
    "pref",
    "ratecap",
    "quality",
)


class Priority(Document):
    link: int
    priority: float


class M4Cell(Cell):
    priority_sidecar: tuple[Priority, ...] | None = None


class M4Candidate(Candidate):
    bin: str
    candidate_class: Literal["cli", "baseline"]
    stats_file: bool = False
    control_socket: bool = True


class M4Receiver(ManifestReceiver):
    bin: str = "lock:ours-new"
    srt_live_transmit_bin: str = "lock:ours-new"


class M4Manifest(Manifest):
    caller_bin: str
    candidates: tuple[M4Candidate, ...]
    receivers: tuple[M4Receiver, ...]
    cells: tuple[M4Cell, ...]
    window_secs_override: None = None


def build(binary: Path) -> M4Manifest:
    receiver = M4Receiver(
        name="ours-new-200",
        lineage="ours-new",
        kind="ceralive",
        listener_uri_extra=OPTIONS,
    )
    config = {
        "adaptive_features": list(FEATURES),
        "adaptive_tuning": {
            "stall_attempts": 32,
            "loss_enter": 0.1,
            "deadline_hold_fraction": 0.5,
            "ratecap_loss_backoff": 0.85,
        },
    }
    candidates = tuple(
        M4Candidate(
            label=name,
            bin=str(binary),
            args=("--mode", name),
            candidate_class="cli",
            stats_file=True,
            effective_config=config,
        )
        for name in CLI
    )
    candidates += tuple(
        M4Candidate(
            label=name,
            bin="/home/andres/.cache/opencode/tmp/srtla-bench/lineage/irlserver-srtla_send",
            args=("--mode", name.removeprefix("upstream-")),
            candidate_class="baseline",
        )
        for name in BASELINES
    )
    cells = [
        M4Cell(
            candidate=name,
            scenario=scenario,
            receiver=receiver.name,
            srt_profile="production",
            runs=N,
            priority_sidecar=(
                Priority(link=0, priority=0.2),
                Priority(link=1, priority=-0.2),
            )
            if scenario == "M6"
            else None,
        )
        for scenario in SCENARIOS
        for name in CLI
    ]
    cells += [
        M4Cell(
            candidate=name,
            scenario=scenario,
            receiver=receiver.name,
            srt_profile="production",
            runs=N,
            covering=False,
            variant="baseline",
        )
        for scenario in REFERENCES
        for name in BASELINES
    ]
    cells += [
        M4Cell(
            candidate="enhanced",
            scenario="M4",
            receiver=receiver.name,
            srt_profile="production",
            runs=3,
            covering=False,
            variant="fec-pair",
            fec=fec,
        )
        for fec in (False, True)
    ]
    return M4Manifest(
        campaign="m4a-ours-new",
        seed=20260913,
        caller_bin="/home/andres/.cache/opencode/tmp/srtla-bench/lineage/ours-new/srt-live-transmit",
        candidates=candidates,
        receivers=(receiver,),
        cells=tuple(
            cell.model_copy(update={"cell_id": stable_identity(cell, receiver)})
            for cell in cells
        ),
    )


def validate(manifest: M4Manifest) -> None:
    expected = build(
        Path(
            next(
                (c.bin for c in manifest.candidates if c.label == "enhanced"), "missing"
            )
        )
    )
    if (
        manifest.campaign != expected.campaign
        or manifest.seed != expected.seed
        or manifest.window_secs_override is not None
    ):
        raise EvidenceError("M4 campaign, seed and full windows are frozen")
    if (
        manifest.receivers != expected.receivers
        or manifest.caller_bin != expected.caller_bin
    ):
        raise EvidenceError("M4 requires the locked ours-new TTL200 policy and caller")
    if sorted(manifest.candidates, key=lambda c: c.label) != sorted(
        expected.candidates, key=lambda c: c.label
    ):
        raise EvidenceError(
            "M4 candidate classes/default flags/configuration are frozen"
        )
    if len(manifest.cells) != 110 or sorted(
        manifest.cells, key=lambda c: c.id
    ) != sorted(expected.cells, key=lambda c: c.id):
        raise EvidenceError(
            "M4 requires 20 unique five-CLI primary groups at N=5, eight baseline references, two FEC N=3 cells"
        )


class Arguments(argparse.Namespace):
    generate: bool = False
    validate: Path | None = None
    smoke_out: Path | None = None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate", action="store_true")
    parser.add_argument("--validate", type=Path)
    parser.add_argument("--smoke-out", type=Path)
    args = parser.parse_args(namespace=Arguments())
    try:
        if args.smoke_out:
            manifest = build(Path("target/m4-srtla_send"))
            selected = tuple(
                c
                for c in manifest.cells
                if c.candidate == "enhanced"
                and (c.scenario == "A" or c.variant == "fec-pair")
            )
            smoke = manifest.model_copy(
                update={
                    "campaign": "m4-resume-smoke",
                    "candidates": tuple(
                        c for c in manifest.candidates if c.label == "enhanced"
                    ),
                    "cells": tuple(c.model_copy(update={"runs": 1}) for c in selected),
                }
            )
            args.smoke_out.write_text(
                smoke.model_dump_json(indent=2, exclude_none=True) + "\n"
            )
            print("resume_smoke_runs=3 (separate QA; not M4 measurement evidence)")
            return 0
        if args.generate:
            root = Path(__file__).parent / "manifests"
            manifest = build(Path("target/m4-srtla_send"))
            validate(manifest)
            (root / "m4a-ours-new.json").write_text(
                manifest.model_dump_json(indent=2, exclude_none=True) + "\n"
            )
            receiver = manifest.receivers[0]
            cells = tuple(
                M4Cell(
                    candidate=name,
                    scenario="M8",
                    receiver=receiver.name,
                    srt_profile="production",
                    runs=1,
                    covering=False,
                )
                for name in ("enhanced", "adaptive")
            )
            soak = manifest.model_copy(
                update={
                    "campaign": "m4-soak",
                    "candidates": tuple(
                        c
                        for c in manifest.candidates
                        if c.label in ("enhanced", "adaptive")
                    ),
                    "cells": tuple(
                        c.model_copy(update={"cell_id": stable_identity(c, receiver)})
                        for c in cells
                    ),
                }
            )
            (root / "m4-soak.json").write_text(
                soak.model_dump_json(indent=2, exclude_none=True) + "\n"
            )
        elif args.validate:
            validate(M4Manifest.model_validate_json(args.validate.read_bytes()))
        else:
            parser.error("choose --generate or --validate")
    except (OSError, ValidationError, EvidenceError) as error:
        parser.exit(1, f"{error}\n")
    print(
        "metric_cells=110 (cap 150; lineage gate + reserved block ≤ 36 reserved for todo 34)"
    )
    print(
        "primary_groups=20 cli_cells=100 baseline_cells=8 fec_cells=2 metric_runs=546 soak_runs=2"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
