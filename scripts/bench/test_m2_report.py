# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest", "numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run --with pytest --with numpy --with pydantic -m pytest scripts/bench/test_m2_report.py
from pathlib import Path

import pytest

import report
from report import EvidenceError, load_records


def test_m2_keeps_measured_settle_failure_without_counting_exhaustion_twice(tmp_path: Path) -> None:
    # Given a measured failed attempt and its duplicate exhaustion receipt.
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m2-sender"})
    record = report.ReportTests().record().model_copy(
        update={"campaign": "m2-sender", "status": "failed", "reason": "settle_timeout"}
    )
    for name in ("run-0.failed-1.json", "run-0.exhausted.json"):
        (tmp_path / name).write_text(record.model_dump_json())
    # When explicitly reducing M2 outcomes.
    loaded = load_records(manifest, (tmp_path,), m2_outcomes=True)
    # Then one failed observation survives, never a fabricated success.
    assert [r.status for r in loaded.by_cell[manifest.cells[0].id]] == ["failed"]


@pytest.mark.parametrize("reason", ["worker_failed", "missing_metrics"])
def test_m2_refuses_infrastructure_failure(tmp_path: Path, reason: str) -> None:
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m2-sender"})
    record = report.ReportTests().record().model_copy(
        update={"campaign": "m2-sender", "status": "failed", "reason": reason}
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    with pytest.raises(EvidenceError, match="measurement"):
        load_records(manifest, (tmp_path,), m2_outcomes=True)


def test_m2_outcome_permission_cannot_leak_to_ordinary_campaign(tmp_path: Path) -> None:
    with pytest.raises(EvidenceError, match="restricted"):
        load_records(report.ReportTests().manifest(), (tmp_path,), m2_outcomes=True)


@pytest.mark.parametrize(
    ("on", "off", "expected"),
    [([90., 91., 92.], [100., 101., 102.], True),
     ([90., 91., 103.], [100., 101., 102.], False),
     ([95., 95., 95.], [100., 100., 100.], False)],
)
def test_k_cap_requires_strict_five_percent_and_disjoint_ci(
    on: list[float], off: list[float], expected: bool,
) -> None:
    from m2_models import regression

    assert regression(report.statistics(on), report.statistics(off)) is expected


@pytest.mark.parametrize(
    ("text", "expected", "value"),
    [("receiver: nak_report=unknown srt=unknown\nreceiver: nak_report=off srt=1.5.5", False, False),
     ("receiver: nak_report=on srt=1.5.5", False, None),
     ("receiver: nak_report=on srt=1.5.5\nreceiver: nak_report=off srt=1.5.5", True, None),
     ("", True, None)],
)
def test_hsrsp_mismatch_or_absence_stays_unknown(
    text: str, expected: bool, value: bool | None,
) -> None:
    from m2_models import hsrsp

    assert hsrsp(text, expected).nak_report is value
