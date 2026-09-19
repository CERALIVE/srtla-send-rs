from pathlib import Path
import subprocess
import sys

import pytest

from lineage_rule import LineageVerdict
from m4_manifest import M4Manifest
from m4b_manifest import build
from report import EvidenceError

ROOT = Path(__file__).resolve().parents[2]


def test_generate_when_single_fallback_has_exact_cells(tmp_path: Path) -> None:
    # Given the unchanged, single-member authoritative verdict.
    destination = tmp_path / "manifest.json"
    # When the real generator CLI runs.
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts/bench/m4b_manifest.py"),
            "--verdict",
            str(ROOT / "docs/evidence/bpc/m4/verdict-provisional.json"),
            "--out",
            str(destination),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then all 18 cells, including reserved 4003, are generated, not a fake pass.
    assert result.returncode == 0, result.stderr
    manifest = M4Manifest.model_validate_json(destination.read_bytes())
    assert len(manifest.cells) == 18
    assert sum(c.runs for c in manifest.cells) == 66
    assert {c.candidate for c in manifest.cells} == {"enhanced"}
    assert sum(c.sink == "sls" and c.port == 4003 for c in manifest.cells) == 3
    assert (
        sum(c.receiver == "irlserver-prod" and c.runs == 5 for c in manifest.cells) == 6
    )
    assert all(c.priority_sidecar for c in manifest.cells if c.scenario == "M6")
    assert all(not c.covering for c in manifest.cells)


def test_manifest_when_two_members_stays_below_total_cap() -> None:
    verdict = LineageVerdict.model_validate_json(
        (ROOT / "docs/evidence/bpc/m4/verdict-provisional.json").read_bytes()
    )
    primary = M4Manifest.model_validate_json(
        (ROOT / "scripts/bench/manifests/m4a-ours-new.json").read_bytes()
    )
    manifest = build(
        verdict.model_copy(update={"ship_set": ("enhanced", "edpf")}), primary
    )
    assert len(manifest.cells) == 36
    assert len(primary.cells) + len(manifest.cells) == 146
    assert sum(c.runs for c in manifest.cells) == 132


def test_manifest_when_ship_set_contains_non_cli_refuses() -> None:
    verdict = LineageVerdict.model_validate_json(
        (ROOT / "docs/evidence/bpc/m4/verdict-provisional.json").read_bytes()
    )
    primary = M4Manifest.model_validate_json(
        (ROOT / "scripts/bench/manifests/m4a-ours-new.json").read_bytes()
    )
    with pytest.raises(EvidenceError):
        build(
            verdict.model_copy(update={"ship_set": ("enhanced", "upstream-classic")}),
            primary,
        )
