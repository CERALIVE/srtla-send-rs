#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/m4_resume_check.py snapshot|compare SMOKE_ROOT

import hashlib
import sys
from pathlib import Path

from report import Document, EvidenceError, Manifest, build_summary


class Snapshot(Document):
    checkpoints: dict[str, str]
    artifacts: tuple[str, ...]


def snapshot(root: Path) -> Snapshot:
    manifest = Manifest.model_validate_json(
        (root.parent / "smoke-manifest.json").read_bytes()
    )
    summary = build_summary(manifest, (root / "results",))
    if sum(cell.n for group in summary.groups for cell in group.cells.values()) != 3:
        raise EvidenceError("resume smoke requires three complete measured outcomes")
    if any(
        cell.integrity_errors
        for group in summary.groups
        for cell in group.cells.values()
    ):
        raise EvidenceError("resume smoke configuration mismatch")
    return Snapshot(
        checkpoints={
            str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted((root / "results").rglob("run-*.json"))
        },
        artifacts=tuple(sorted(path.name for path in (root / "artifacts").iterdir())),
    )


def main() -> int:
    phase, directory = sys.argv[1:]
    root = Path(directory)
    current = snapshot(root)
    receipt = root / "resume-snapshot.json"
    match phase:
        case "snapshot":
            receipt.write_text(current.model_dump_json(indent=2) + "\n")
        case "compare":
            if current != Snapshot.model_validate_json(receipt.read_bytes()):
                raise EvidenceError(
                    "resume changed checkpoints or created new attempts"
                )
            print(
                "resume PASS: three complete outcomes, identical checkpoint bytes, zero new attempts"
            )
        case _:
            raise EvidenceError("expected snapshot or compare")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
