# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest>=8", "pydantic>=2,<3"]
# ///
# Run: uv run --with pytest --with pydantic pytest scripts/bench/test_apply_verdict_precondition.py
"""Default selection requires both independent gates, never a pending canary."""

import json
import subprocess
import sys
from pathlib import Path

import pytest


@pytest.mark.parametrize(
    ("base", "gate", "reason", "canary", "default", "branch"),
    [
        (
            "enhanced",
            "skipped",
            "base mode is enhanced",
            "pending",
            "enhanced",
            "fallback",
        ),
        ("adaptive", "passed", None, "pass", "adaptive", "verdict"),
        ("adaptive", "passed", None, "pending", "enhanced", "fallback"),
        ("adaptive", "failed", None, "pass", "enhanced", "fallback"),
        (
            "adaptive",
            "skipped",
            "base mode is enhanced",
            "pass",
            "enhanced",
            "fallback",
        ),
        ("enhanced", "skipped", "base mode is enhanced", "pass", "enhanced", "verdict"),
    ],
)
def test_precondition_cli(
    tmp_path: Path,
    base: str,
    gate: str,
    reason: str | None,
    canary: str,
    default: str,
    branch: str,
) -> None:
    # Given independently recorded verdict, M5, and canary documents.
    verdict_path = tmp_path / "verdict.json"
    m5_path = tmp_path / "m5.json"
    canary_path = tmp_path / "canary.json"
    verdict_path.write_text(json.dumps({"ship_set": [base], "base_mode": base}))
    m5_path.write_text(json.dumps({"gate": gate, "reason": reason}))
    canary_path.write_text(json.dumps({"outcome": canary}))
    # When the real precondition CLI resolves the release branch.
    result = subprocess.run(
        [
            sys.executable,
            str(Path(__file__).with_name("apply_verdict_precondition.py")),
            "--verdict",
            str(verdict_path),
            "--m5",
            str(m5_path),
            "--canary",
            str(canary_path),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then pending is never pass and the fallback is added to the final set.
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == {
        "default": default,
        "branch": branch,
        "final_set": sorted({base, default}),
    }
