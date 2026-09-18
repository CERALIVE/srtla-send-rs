# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest>=8", "pydantic==2.*", "typer>=0.16,<1"]
# ///
# Run: uv run --with pytest --with pydantic --with typer pytest scripts/bench/test_defect_evidence.py
"""Synthetic evidence fixtures only; no campaign records are modified."""

import json
import subprocess
import sys
from pathlib import Path
from typing import Final

import pytest
from pydantic import JsonValue

SCRIPT: Final = Path(__file__).with_name("pr_description.py")


def invoke(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--check-defects", str(root)],
        capture_output=True,
        text=True,
        check=False,
        timeout=20,
    )


@pytest.fixture
def empty_round(tmp_path: Path) -> Path:
    (tmp_path / "reruns").mkdir()
    (tmp_path / "reruns/summary.json").write_text('{"cells": [], "runs": 0}')
    (tmp_path / "none.json").write_text(
        '{"outcome":"none admitted","selected_alternative":"retain frozen data"}'
    )
    return tmp_path


@pytest.fixture
def admitted_round(empty_round: Path) -> Path:
    (empty_round / "none.json").unlink()
    record: dict[str, JsonValue] = {
        "mode": "enhanced",
        "design_claim_citation": "README.md:96",
        "failing_test": "sustained_loss_claim",
        "pre_fix_source_sha": "a" * 40,
        "pre_fix_binary_sha256": "b" * 64,
        "pre_fix_failure_log": "test result: FAILED. 0 passed; 1 failed",
        "post_fix_source_sha": "c" * 40,
        "post_fix_binary_sha256": "d" * 64,
        "cells_rerun": ["F"],
        "before": {"F": 0},
        "after": {"F": 1},
        "coverage_delta": 1,
        "admitted_by": [True, True, True, True, True],
        "outcome": "retained",
        "selected_alternative": "one correction, no retuning",
    }
    (empty_round / "defect-enhanced.json").write_text(json.dumps(record))
    return empty_round


def test_accepts_explicit_none_with_empty_summary(empty_round: Path) -> None:
    # Given the explicit no-admission alternative, when checking its directory.
    result = invoke(empty_round)
    # Then the real CLI accepts the evidence without inventing reruns.
    assert result.returncode == 0, result.stderr


def test_accepts_complete_admission_shape(admitted_round: Path) -> None:
    # Given a synthetic complete record, when checking the five assertions.
    result = invoke(admitted_round)
    # Then the structural gate accepts it (historical proof is a separate audit).
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    "field",
    [
        "design_claim_citation",
        "failing_test",
        "pre_fix_source_sha",
        "pre_fix_binary_sha256",
        "pre_fix_failure_log",
        "post_fix_source_sha",
        "post_fix_binary_sha256",
        "cells_rerun",
        "before",
        "after",
        "coverage_delta",
        "admitted_by",
        "outcome",
        "selected_alternative",
    ],
)
def test_rejects_missing_admission_field(admitted_round: Path, field: str) -> None:
    # Given one missing predicate field, when checking the mutated fixture.
    from pydantic import TypeAdapter

    path = admitted_round / "defect-enhanced.json"
    record = TypeAdapter(dict[str, JsonValue]).validate_json(path.read_text())
    del record[field]
    path.write_text(json.dumps(record))
    result = invoke(admitted_round)
    # Then an incomplete record cannot pass.
    assert result.returncode == 1, result.stderr


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("admitted_by", [True] * 4),
        ("admitted_by", [True, True, False, True, True]),
        ("admitted_by", [1] * 5),
        ("pre_fix_failure_log", "test result: ok. 1 passed"),
        ("pre_fix_source_sha", "87328c6-dirty"),
        ("outcome", "   "),
        ("cells_rerun", []),
        ("mode", "classic"),
        ("mode", "adaptive"),
        ("coverage_delta", "not-a-number"),
    ],
)
def test_rejects_invalid_admission_value(
    admitted_round: Path, field: str, value: JsonValue
) -> None:
    # Given malformed proof, when checking its typed boundary.
    from pydantic import TypeAdapter

    path = admitted_round / "defect-enhanced.json"
    record = TypeAdapter(dict[str, JsonValue]).validate_json(path.read_text())
    record[field] = value
    path.write_text(json.dumps(record))
    result = invoke(admitted_round)
    # Then the record fails closed.
    assert result.returncode == 1, result.stderr


def test_rejects_none_beside_admitted_record(admitted_round: Path) -> None:
    # Given contradictory dispositions, when checking the directory.
    (admitted_round / "none.json").write_text(
        '{"outcome":"none","selected_alternative":"unchanged"}'
    )
    result = invoke(admitted_round)
    # Then none cannot conceal an invalid or admitted record.
    assert result.returncode == 1, result.stderr


@pytest.mark.parametrize(
    "payload", ["{}", "[]", "{", '{"cells":{},"runs":0}', '{"cells":[],"runs":1}']
)
def test_rejects_invalid_empty_summary(empty_round: Path, payload: str) -> None:
    # Given an unusable second input, when checking a no-admission round.
    (empty_round / "reruns/summary.json").write_text(payload)
    result = invoke(empty_round)
    # Then the absence of reruns cannot mask malformed summary evidence.
    assert result.returncode == 1, result.stderr


def test_rejects_missing_summary(empty_round: Path) -> None:
    # Given no summary, when the real CLI runs.
    (empty_round / "reruns/summary.json").unlink()
    result = invoke(empty_round)
    # Then the missing required artifact is fatal.
    assert result.returncode == 1, result.stderr


def test_rejects_empty_directory(tmp_path: Path) -> None:
    # Given no disposition, when the real CLI runs.
    result = invoke(tmp_path)
    # Then an empty directory is not a no-defects assertion.
    assert result.returncode == 1, result.stderr
