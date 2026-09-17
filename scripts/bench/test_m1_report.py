# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest", "numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run --with pytest --with numpy --with pydantic -m pytest scripts/bench/test_m1_report.py
from pathlib import Path

import pytest

import report
from report import EvidenceError, load_records


def test_m1_outcomes_keep_failed_settle_without_claiming_success(
    tmp_path: Path,
) -> None:
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m1-ttl"})
    record = (
        report.ReportTests()
        .record()
        .model_copy(
            update={
                "campaign": "m1-ttl",
                "status": "failed",
                "reason": "settle_timeout",
            }
        )
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    loaded = load_records(manifest, (tmp_path,), m1_outcomes=True)
    assert loaded.by_cell[manifest.cells[0].id][0].status == "failed"


def test_normal_report_still_refuses_failed_only_cell(tmp_path: Path) -> None:
    manifest = report.ReportTests().manifest()
    record = (
        report.ReportTests()
        .record()
        .model_copy(update={"status": "failed", "reason": "settle_timeout"})
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    with pytest.raises(EvidenceError, match="missing/insufficient"):
        load_records(manifest, (tmp_path,))


def test_m1_outcomes_refuse_infrastructure_failures(tmp_path: Path) -> None:
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m1-ttl"})
    record = (
        report.ReportTests()
        .record()
        .model_copy(
            update={"campaign": "m1-ttl", "status": "failed", "reason": "worker_failed"}
        )
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    with pytest.raises(EvidenceError, match="measurement"):
        load_records(manifest, (tmp_path,), m1_outcomes=True)


def test_m1_variants_normalize_before_duplicate_check(tmp_path: Path) -> None:
    from m1_report import render

    manifest = Path(__file__).parent / "manifests" / "m1-ttl.json"
    with pytest.raises(EvidenceError, match="results directory not found"):
        render(manifest, (tmp_path / "absent",))
