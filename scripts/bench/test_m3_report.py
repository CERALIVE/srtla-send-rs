# /// script
# requires-python = ">=3.12"
# dependencies = ["pytest", "numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run --with pytest --with numpy --with pydantic -m pytest scripts/bench/test_m3_report.py
from pathlib import Path
import subprocess
import sys

import pytest

import report
import m3_models


def test_m3_retains_one_failed_measurement_not_exhaustion(tmp_path: Path) -> None:
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m3-interop"})
    record = report.ReportTests().record().model_copy(
        update={"campaign": "m3-interop", "status": "failed", "reason": "settle_timeout"}
    )
    for name in ("run-0.failed-1.json", "run-0.exhausted.json"):
        (tmp_path / name).write_text(record.model_dump_json())
    loaded = report.load_records(manifest, (tmp_path,), m3_outcomes=True)
    assert [r.status for r in loaded.by_cell[manifest.cells[0].id]] == ["failed"]


@pytest.mark.parametrize("reason", ["worker_failed", "missing_metrics"])
def test_m3_refuses_infrastructure_failure(tmp_path: Path, reason: str) -> None:
    manifest = report.ReportTests().manifest().model_copy(update={"campaign": "m3-interop"})
    record = report.ReportTests().record().model_copy(
        update={"campaign": "m3-interop", "status": "failed", "reason": reason}
    )
    (tmp_path / "run-0.failed-1.json").write_text(record.model_dump_json())
    with pytest.raises(report.EvidenceError, match="measurement"):
        report.load_records(manifest, (tmp_path,), m3_outcomes=True)


def test_m3_permission_cannot_leak_to_ordinary_campaign(tmp_path: Path) -> None:
    with pytest.raises(report.EvidenceError, match="restricted"):
        report.load_records(report.ReportTests().manifest(), (tmp_path,), m3_outcomes=True)


@pytest.mark.parametrize("sender", ["belabox-c", "ours-3.3.0", "ours-new"])
@pytest.mark.parametrize("ratio", [0.95, 0.949999])
def test_rollout_boundary_and_sender_failure_disposition(sender: str, ratio: float) -> None:
    outcomes = tuple(m3_models.Outcome(run_index=i, settled=i < 2, goodput_bps=100.0 * ratio, retransmit_ratio=0.10) for i in range(3))
    base = m3_models.CellResult(sender=sender, scenario="B1", receiver="ours-new-200", outcomes=outcomes, settled_runs=2, passing_runs=2, goodput=report.statistics([100.0 * ratio] * 3), sender_sha256="a"*64, receiver_sha256="b"*64, libsrt_tool_sha256="c"*64)
    old = base.model_copy(update={"receiver":"ours-old", "goodput":report.statistics([100.0]*3)})
    result = m3_models.compare(base, old)
    assert result.passed is (ratio >= 0.95)
    expected = {"belabox-c":"known_limitation", "ours-3.3.0":"receiver_pr_blocker", "ours-new":"new_sender_failure"}
    assert result.disposition == ("pass" if ratio >= 0.95 else expected[sender])


def test_cli_refuses_incomplete_matrix_and_invalidates_summary(tmp_path: Path) -> None:
    manifest = tmp_path / "manifest.json"
    manifest.write_text(report.ReportTests().manifest().model_copy(update={"campaign": "m3-interop"}).model_dump_json())
    result_dir = tmp_path / "results"
    result_dir.mkdir()
    summary = tmp_path / "summary.json"
    summary.write_text('{"stale":true}')
    result = subprocess.run([sys.executable, str(Path(report.__file__)), "--m3-outcomes", "--manifest", str(manifest),
                             "--results", str(result_dir), "--out", str(tmp_path/"report.md"), "--json", str(summary)],
                            capture_output=True, text=True, timeout=30, check=False)
    assert result.returncode == 1
    assert not summary.exists()


@pytest.mark.parametrize("passing_runs", [0, 1, 2, 3])
def test_two_joint_passes_are_required_even_with_high_goodput(passing_runs: int) -> None:
    base = m3_models.CellResult(sender="ours-3.3.0", scenario="G", receiver="ours-new-200", outcomes=(),
        settled_runs=3, passing_runs=passing_runs, goodput=report.statistics([100.0]*3),
        sender_sha256="a"*64, receiver_sha256="b"*64, libsrt_tool_sha256="c"*64)
    assert m3_models.compare(base, base.model_copy(update={"receiver":"ours-old"})).passed is (passing_runs >= 2)
