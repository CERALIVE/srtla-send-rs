# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic==2.*", "typer>=0.16,<1"]
# ///
# Run: uv run scripts/bench/pr_description.py --check-defects docs/evidence/bpc/defects/
"""Todo 33 structural gate; historical/causal proof still requires evidence review."""

from dataclasses import dataclass
from pathlib import Path
from typing import Annotated, Literal

import typer
from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    JsonValue,
    StrictBool,
    StrictInt,
    StringConstraints,
    ValidationError,
)

type Text = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]
type SourceSha = Annotated[str, StringConstraints(pattern=r"^[0-9a-f]{40}$")]
type BinarySha = Annotated[str, StringConstraints(pattern=r"^[0-9a-f]{64}$")]


class Disposition(BaseModel):
    model_config = ConfigDict(frozen=True, extra="ignore")

    outcome: Text
    selected_alternative: Text


class DefectRecord(Disposition):
    mode: Literal["enhanced", "rtt-threshold", "edpf"]
    design_claim_citation: Text
    failing_test: Text
    pre_fix_source_sha: SourceSha
    pre_fix_binary_sha256: BinarySha
    pre_fix_failure_log: Text
    post_fix_source_sha: SourceSha
    post_fix_binary_sha256: BinarySha
    cells_rerun: Annotated[tuple[Text, ...], Field(min_length=1)]
    before: dict[str, JsonValue]
    after: dict[str, JsonValue]
    coverage_delta: Annotated[float, Field(allow_inf_nan=False)]
    admitted_by: Annotated[tuple[StrictBool, ...], Field(min_length=5, max_length=5)]


class RerunSummary(BaseModel):
    model_config = ConfigDict(frozen=True, extra="ignore")

    cells: tuple[JsonValue, ...]
    runs: Annotated[StrictInt, Field(ge=0)]


@dataclass(frozen=True, slots=True)
class DefectEvidenceError(Exception):
    path: Path
    reason: str

    def __str__(self) -> str:
        return f"{self.path}: {self.reason}"


def check_defects(root: Path) -> int:
    """Check exclusive disposition, admission shape/caps and the required summary."""
    paths = tuple(sorted(root.glob("defect-*.json")))
    none = root / "none.json"
    if none.is_file() == bool(paths):
        raise DefectEvidenceError(
            root, "require none.json OR defect-<mode>.json records"
        )
    summary = RerunSummary.model_validate_json(
        (root / "reruns/summary.json").read_text(encoding="utf-8")
    )
    if none.is_file():
        Disposition.model_validate_json(none.read_text(encoding="utf-8"))
        if summary.cells or summary.runs:
            raise DefectEvidenceError(
                none, "no admitted defects requires zero rerun cells/runs"
            )
        return 0
    if len(paths) > 3:
        raise DefectEvidenceError(root, "at most three mode defects in one round")
    modes: set[str] = set()
    for path in paths:
        record = DefectRecord.model_validate_json(path.read_text(encoding="utf-8"))
        if path.name != f"defect-{record.mode}.json" or record.mode in modes:
            raise DefectEvidenceError(
                path, "require one correctly named record per mode"
            )
        if not all(record.admitted_by):
            raise DefectEvidenceError(
                path, "all five admission predicates must be true"
            )
        if "test result: FAILED" not in record.pre_fix_failure_log:
            raise DefectEvidenceError(path, "missing captured Rust test failure")
        modes.add(record.mode)
    return len(paths)


def main(check_defects_path: Annotated[Path, typer.Option("--check-defects")]) -> None:
    try:
        count = check_defects(check_defects_path)
    except (DefectEvidenceError, ValidationError, OSError) as error:
        typer.echo(str(error), err=True)
        raise typer.Exit(1) from error
    typer.echo(f"defect evidence OK: {count} admitted mode records")


if __name__ == "__main__":
    typer.run(main)
