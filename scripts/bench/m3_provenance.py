"""Committed-hash verification for the frozen M3 rows.

The M3 raw root is machine-local; only its recorded sha256 map is committed. The
committed record is `summary.json.input_sha256` (the M3 README names it directly);
a sibling `provenance.json` in the M1 shape is also honoured when present. A run
that is present in the hash map but missing or changed on disk marks its row
`raw_verified: false` and is never re-scored on the remaining runs.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Final

from m3_models import M3Summary
from report import EvidenceError

type RowKey = tuple[str, str]

NEW_RECEIVER: Final = "ours-new-200"
OLD_RECEIVER: Final = "ours-old"
EXHAUSTED_SUFFIX: Final = ".exhausted.json"
ROWS: Final[tuple[RowKey, ...]] = (
    ("belabox-c", "C"),
    ("irlserver-rust-enhanced", "C"),
    ("ours-3.3.0", "C"),
    ("ours-new", "C"),
    ("ours-new", "M1"),
)


@dataclass(frozen=True, slots=True)
class Provenance:
    source: str
    root: Path | None
    hashes: dict[str, str]


def load_provenance(summary: M3Summary, provenance_path: Path | None) -> Provenance:
    if provenance_path is not None and provenance_path.is_file():
        raw = json.loads(provenance_path.read_text(encoding="utf-8"))
        if not raw.get("raw_root"):
            raise EvidenceError("provenance has no raw_root")
        root = Path(raw["raw_root"])
        hashes = {
            str(root / entry["path"]): entry["sha256"] for entry in raw["input_files"]
        }
        return Provenance(f"provenance:{provenance_path}", root, hashes)
    return Provenance(
        "summary.input_sha256 (m3-interop/provenance.json absent)",
        None,
        dict(summary.input_sha256),
    )


def recorded_runs(
    provenance: Provenance, sender: str, scenario: str, receiver: str
) -> dict[int, list[tuple[str, str]]]:
    entries: dict[int, list[tuple[str, str]]] = {}
    for path, digest in provenance.hashes.items():
        posix = PurePosixPath(path)
        if posix.parent.parent.name != "results":
            continue
        directory = posix.parent.name
        if not (
            directory.startswith(f"{receiver}@")
            and f"--{scenario}@" in directory
            and directory.endswith(f"--{sender}")
        ):
            continue
        if posix.name.endswith(EXHAUSTED_SUFFIX):
            continue
        run = posix.name.removesuffix(".json").split(".failed-")[0]
        if not run.startswith("run-"):
            continue
        entries.setdefault(int(run.removeprefix("run-")), []).append((path, digest))
    return entries


def resolve(provenance: Provenance, raw_root: Path | None, recorded: str) -> Path:
    root = raw_root if raw_root is not None else provenance.root
    if root is None:
        return Path(recorded)
    posix = PurePosixPath(recorded)
    return root / "results" / posix.parent.name / posix.name


def verify_raw(
    summary: M3Summary, provenance: Provenance, raw_root: Path | None = None
) -> tuple[dict[RowKey, bool], tuple[str, ...]]:
    verified: dict[RowKey, bool] = {}
    errors: list[str] = []
    index = {(cell.sender, cell.receiver, cell.scenario): cell for cell in summary.cells}
    for sender, scenario in ROWS:
        ok = True
        for receiver in (NEW_RECEIVER, OLD_RECEIVER):
            cell = index.get((sender, receiver, scenario))
            if cell is None:
                raise EvidenceError(f"missing M3 cell {sender}/{scenario}/{receiver}")
            entries = recorded_runs(provenance, sender, scenario, receiver)
            for run_index in sorted({outcome.run_index for outcome in cell.outcomes}):
                if run_index not in entries:
                    ok = False
                    errors.append(
                        f"{sender}/{scenario}/{receiver}: provenance records no run-{run_index}"
                    )
            for run_index, recorded in sorted(entries.items()):
                for path, digest in recorded:
                    label = f"{sender}/{scenario}/{receiver}/run-{run_index}"
                    actual = resolve(provenance, raw_root, path)
                    if not actual.is_file():
                        ok = False
                        errors.append(f"{label}: missing raw run {actual}")
                    elif hashlib.sha256(actual.read_bytes()).hexdigest() != digest:
                        ok = False
                        errors.append(f"{label}: raw sha256 mismatch for {actual}")
        verified[(sender, scenario)] = ok
    return verified, tuple(errors)
