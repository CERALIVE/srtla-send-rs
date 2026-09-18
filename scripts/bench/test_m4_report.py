from pathlib import Path

import pytest

from report import EvidenceError, ReportTests, build_summary


def test_m4_retains_measured_timeout_but_cannot_pass_settling(tmp_path: Path) -> None:
    fixture = ReportTests()
    manifest = fixture.manifest().model_copy(update={"campaign": "m4a-ours-new"})
    record = fixture.record().model_copy(
        update={
            "campaign": "m4a-ours-new",
            "status": "failed",
            "reason": "settle_timeout",
        }
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    summary = build_summary(manifest, (tmp_path,))
    assert summary.groups[0].cells["adaptive"].n == 1
    assert summary.groups[0].cells["adaptive"].checks["settled_rate"] == 0


def test_ordinary_campaign_still_requires_success(tmp_path: Path) -> None:
    fixture = ReportTests()
    record = fixture.record().model_copy(
        update={"status": "failed", "reason": "settle_timeout"}
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    with pytest.raises(EvidenceError):
        build_summary(fixture.manifest(), (tmp_path,))
