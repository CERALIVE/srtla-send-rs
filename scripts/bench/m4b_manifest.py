#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/m4b_manifest.py --verdict VERDICT --out MANIFEST
import argparse
from pathlib import Path
from typing import Final

from pydantic import ValidationError

from lineage_rule import CLI, LineageVerdict
from m4_manifest import (
    M4Cell,
    M4Manifest,
    M4Receiver,
    Priority,
    validate as validate_primary,
)
from report import EvidenceError, stable_identity

ROOT: Final = Path(__file__).resolve().parents[2]
DYNAMIC: Final = ("M1", "M4", "M6")


def members(verdict: LineageVerdict) -> tuple[str, ...]:
    if (
        not verdict.ship_set
        or len(set(verdict.ship_set)) != len(verdict.ship_set)
        or verdict.base_mode not in verdict.ship_set
        or not set(verdict.ship_set) <= set(CLI)
    ):
        raise EvidenceError("invalid CLI covering set/base")
    return tuple(
        sorted(
            verdict.ship_set,
            key=lambda name: (
                name != verdict.base_mode,
                -sum(
                    name in c.covered_by for c in verdict.cells.values() if not c.exempt
                ),
                name,
            ),
        )
    )[:2]


def build(verdict: LineageVerdict, primary: M4Manifest) -> M4Manifest:
    validate_primary(primary)
    selected = members(verdict)
    receivers = tuple(
        M4Receiver(
            name=name,
            lineage=name,
            kind=kind,
            bin=f"lock:{name}",
            srt_live_transmit_bin=f"lock:{name}",
            listener_uri_extra=options,
        )
        for name, kind, options in (
            ("irlserver-prod", "irlserver", "&srtlapatches=1&lossmaxttl=200"),
            ("ours-old", "ceralive", "&nakreport=1&lossmaxttl=40&reorderfreeze=0"),
            ("irlserver-next", "irlserver", "&srtlapatches=1&lossmaxttl=200"),
            ("belabox", "belabox", "&srtlapatches=1&lossmaxttl=200"),
            ("ours-new", "ceralive", ""),
        )
    )
    cells: list[M4Cell] = []
    for receiver in receivers:
        reserved = receiver.name == "ours-new"
        prod = receiver.name == "irlserver-prod"
        for scenario in ("A", "B1", "G", *DYNAMIC) if prod else DYNAMIC:
            for name in selected:
                cell = M4Cell(
                    candidate=name,
                    scenario=scenario,
                    receiver=receiver.name,
                    srt_profile="production",
                    runs=5 if prod else 3,
                    covering=False,
                    sink="sls" if reserved else "slt",
                    metrics="none" if reserved else "full",
                    port=4003 if reserved else 4001,
                    priority_sidecar=(
                        Priority(link=0, priority=0.2),
                        Priority(link=1, priority=-0.2),
                    )
                    if scenario == "M6"
                    else None,
                )
                cells.append(
                    cell.model_copy(update={"cell_id": stable_identity(cell, receiver)})
                )
    if len(primary.cells) + len(cells) > 150:
        raise EvidenceError("m4a + m4b exceeds 150 cells")
    return M4Manifest(
        campaign="m4b-lineages",
        seed=primary.seed,
        caller_bin=primary.caller_bin,
        candidates=tuple(
            next(c for c in primary.candidates if c.label == name) for name in selected
        ),
        receivers=receivers,
        cells=tuple(cells),
    )


class Arguments(argparse.Namespace):
    verdict: Path
    out: Path | None = None
    validate: Path | None = None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verdict", type=Path, required=True)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--out", type=Path)
    group.add_argument("--validate", type=Path)
    args = parser.parse_args(namespace=Arguments())
    try:
        verdict = LineageVerdict.model_validate_json(args.verdict.read_bytes())
        primary = M4Manifest.model_validate_json(
            (ROOT / "scripts/bench/manifests/m4a-ours-new.json").read_bytes()
        )
        manifest = build(verdict, primary)
        if args.validate is not None:
            if M4Manifest.model_validate_json(args.validate.read_bytes()) != manifest:
                raise EvidenceError(
                    "lineage matrix differs from frozen verdict-derived scope"
                )
        if args.out is not None:
            args.out.write_text(
                manifest.model_dump_json(indent=2, exclude_none=True) + "\n"
            )
        print(
            f"m4a=110 m4b={len(manifest.cells)} total={110 + len(manifest.cells)} cap=150 runs={sum(c.runs for c in manifest.cells)}"
        )
    except (OSError, ValidationError, EvidenceError) as error:
        parser.exit(1, f"{error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
