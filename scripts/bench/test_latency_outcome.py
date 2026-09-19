"""Latency-as-outcome reduction: receiver-CSV fields, metric directions, LOSSMAXTTL time-equivalence.

Given/When/Then over synthetic run fixtures. The receiver `-statsout` capture is the
normalised copy the runner attaches at `raw.stats_csv` for the CSV at
`raw.stats_csv_path`; nothing here reads a live campaign tree.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest

from report import (
    LATENCY_FIELDS,
    Manifest,
    ReportTests,
    RunRecord,
    Summary,
    build_summary,
    load_records,
    paired,
)

REPO: Path = Path(__file__).resolve().parents[2]
WINDOW_S: float = 60.0
LATENCY_MS: float = 2000.0
LOSSMAXTTL: int = 40


def document(**overrides: Any) -> dict[str, Any]:
    """A synthetic ok run with the default production profile (latency 2000ms, lossmaxttl 40)."""
    base: dict[str, Any] = json.loads(ReportTests.RECORD)
    return base | overrides


def rows(*, packets: int, buffers: tuple[float, ...]) -> list[dict[str, Any]]:
    """Cumulative receiver rows spread across the whole 0..60000ms window."""
    steps = len(buffers)
    return [
        {
            "t_ms": int(index * 60000 / (steps - 1)),
            "socket_id": 1,
            "pkt_recv_total": int(index * packets / (steps - 1)),
            "ms_rcv_buf": buffer,
            "ms_rcv_tsbpd_delay": LATENCY_MS,
        }
        for index, buffer in enumerate(buffers)
    ]


BUFFERS: tuple[float, ...] = (100.0, 200.0, 300.0, 400.0, 1000.0)
# numpy linear percentiles over BUFFERS: p50 = 300, p95 = 400 + 0.8 * (1000 - 400).
P50: float = 300.0
P95: float = 880.0


def captured(
    *, packets: int = 120_000, buffers: tuple[float, ...] = BUFFERS, **overrides: Any
) -> dict[str, Any]:
    record = document(**overrides)
    record["raw"] = dict(record["raw"]) | {
        "stats_csv_path": "/synthetic/receiver.csv",
        "stats_csv": {"rows": rows(packets=packets, buffers=buffers)},
    }
    return record


def reduced(
    tmp_path: Path, *records: dict[str, Any], floor: float | None = None
) -> tuple[Manifest, tuple[RunRecord, ...]]:
    """Writes the records into a results tree and loads them through the real reducer."""
    results = tmp_path / "results"
    results.mkdir(parents=True, exist_ok=True)
    for index, record in enumerate(records):
        _ = (results / f"run-{index}.json").write_text(json.dumps(record))
    manifest = ReportTests().manifest().model_copy(update={"latency_outcomes": True})
    floor_path = tmp_path / "negotiated-floor.json"
    if floor is not None:
        _ = floor_path.write_text(
            json.dumps({"requested_ms": 50.0, "negotiated_ms": floor})
        )
    loaded = load_records(manifest, (results,), negotiated_floor=floor_path)
    return manifest, loaded.by_cell["adaptive--A--ceralive--production"]


def arm(
    goodput: tuple[float, ...], diagnostics: tuple[dict[str, float], ...]
) -> tuple[RunRecord, ...]:
    """A matched arm whose per-run values differ, so a ratio CI has two distinct bounds."""
    base = ReportTests().record()
    return tuple(
        base.model_copy(
            update={
                "run_index": index,
                "useful_goodput_bps": value,
                "msrcvbuf_p95": P95,
                "diagnostics": dict(base.diagnostics) | extra,
            }
        )
        for index, (value, extra) in enumerate(
            zip(goodput, diagnostics, strict=True)
        )
    )


def series(metric: str, *values: float) -> tuple[dict[str, float], ...]:
    return tuple({metric: value} for value in values)


REFERENCE_GOODPUT: tuple[float, ...] = (900.0, 1000.0, 1200.0)
CANDIDATE_GOODPUT: tuple[float, ...] = (1080.0, 1100.0, 1260.0)


def test_msrcvbuf_percentiles_retained(tmp_path: Path) -> None:
    # Given a receiver capture whose buffer occupancy varies across the window.
    _, records = reduced(tmp_path, captured())
    # When the tree is reduced.
    record = records[0]
    # Then both percentiles survive as first-class fields, not a single collapsed minimum.
    assert record.msrcvbuf_p50 == pytest.approx(P50)
    assert record.msrcvbuf_p95 == pytest.approx(P95)


def test_headroom_and_starvation_margin(tmp_path: Path) -> None:
    # Given a capture and the reported in-window buffer minimum.
    _, records = reduced(
        tmp_path,
        captured(
            diagnostics={
                "pkt_belated_sum": 0,
                "reorder_distance_max": None,
                "ms_rcv_buf_min": 400.0,
            }
        ),
    )
    record = records[0]
    # Then headroom is latency minus p95, and the margin is the minimum over latency.
    assert record.headroom_min_ms == pytest.approx(LATENCY_MS - P95)
    assert record.starvation_margin == 400.0 / LATENCY_MS


def test_lower_is_better_uses_ci_upper() -> None:
    # Given a candidate that roughly halves belated packets while raising goodput,
    # with per-run spread so the two CI bounds are distinct and cannot coincide.
    reference = arm(REFERENCE_GOODPUT, series("pkt_belated_delta", 100.0, 200.0, 400.0))
    candidate = arm(CANDIDATE_GOODPUT, series("pkt_belated_delta", 40.0, 100.0, 220.0))
    # When the two arms are compared.
    comparison = paired(candidate, reference).metrics
    belated, goodput = comparison["pkt_belated_delta"], comparison["useful_goodput_bps"]
    assert belated != "unavailable" and goodput != "unavailable"
    # Then belated is judged by its upper bound, and the halving clears a 0.7 threshold.
    assert belated.direction == "lower_is_better"
    assert belated.ci_lower == pytest.approx(0.4)
    assert belated.ci_upper == pytest.approx(0.55)
    assert belated.reported_bound == belated.ci_upper
    assert belated.reported_bound <= 0.7
    # And goodput keeps the higher-is-better lower bound. Reporting the wrong bound
    # would claim 1.2 for goodput and 0.4 for belated: both assertions below fail.
    assert goodput.direction == "higher_is_better"
    assert goodput.ci_lower == pytest.approx(1.05)
    assert goodput.ci_upper == pytest.approx(1.2)
    assert goodput.reported_bound == goodput.ci_lower


def test_lossmaxttl_ms_equivalent_and_tolerance_saturated(tmp_path: Path) -> None:
    # Given one run at 2000 pps and one at 20 pps over the same 60s window.
    _, fast = reduced(tmp_path / "fast", captured(packets=120_000))
    _, slow = reduced(tmp_path / "slow", captured(packets=1_200))
    # Then LOSSMAXTTL is converted from packets to milliseconds at the observed rate.
    assert fast[0].packets_per_second == 120_000 / WINDOW_S
    assert fast[0].lossmaxttl_ms_equivalent == LOSSMAXTTL / (120_000 / WINDOW_S) * 1000
    assert fast[0].tolerance_saturated is False
    # And a rate low enough to make the tolerance reach the latency budget is labelled saturated.
    assert slow[0].lossmaxttl_ms_equivalent == LATENCY_MS
    assert slow[0].tolerance_saturated is True


def test_floor_clamped_label(tmp_path: Path) -> None:
    # Given a receiver that refused the request and negotiated above the rung's latency.
    _, clamped = reduced(tmp_path / "clamped", captured(), floor=2500.0)
    # And one that refused it but stayed below the rung.
    _, free = reduced(tmp_path / "free", captured(), floor=1500.0)
    # And no floor document at all, because the producing todo may not have run.
    _, unknown = reduced(tmp_path / "unknown", captured())
    assert clamped[0].floor_clamped is True
    assert free[0].floor_clamped is False
    assert unknown[0].floor_clamped is False


def test_an_honoured_request_declares_no_floor(tmp_path: Path) -> None:
    # Given the shipped foldin-v2 document, where the receiver honoured the request.
    floor = json.loads((REPO / "docs/evidence/bpc/foldin-v2/negotiated-floor.json").read_text())
    assert floor["negotiated_ms"] == floor["requested_ms"]
    # When a rung far below both declared floors is reduced against it.
    _, records = reduced(tmp_path, captured(), floor=float(floor["requested_ms"]))
    # Then a declared-but-unenforced floor never clamps a rung.
    assert records[0].floor_clamped is False


def test_retransmit_unavailable_never_reconstructed() -> None:
    # Given both arms carrying a validated received-packet denominator.
    reference = arm(REFERENCE_GOODPUT, series("retrans_ratio", 0.02, 0.04, 0.08))
    candidate = arm(CANDIDATE_GOODPUT, series("retrans_ratio", 0.008, 0.02, 0.044))
    measured = paired(candidate, reference).metrics["retransmit_pct"]
    assert measured != "unavailable"
    assert measured.direction == "lower_is_better"
    assert measured.ci_lower == pytest.approx(0.4)
    assert measured.reported_bound == measured.ci_upper == pytest.approx(0.55)
    # When one arm has no denominator but does carry sender-side loss signals.
    blind = arm(REFERENCE_GOODPUT, series("pkt_belated_delta", 10.0, 20.0, 40.0))
    # Then the comparison states unavailability instead of reconstructing a percentage.
    assert paired(candidate, blind).metrics["retransmit_pct"] == "unavailable"
    assert paired(blind, candidate).metrics["retransmit_pct"] == "unavailable"


def test_metric_directions_published_for_a_latency_outcome_campaign(
    tmp_path: Path,
) -> None:
    # Given an opted-in campaign reduced end to end.
    results = tmp_path / "results"
    results.mkdir()
    _ = (results / "run-0.json").write_text(json.dumps(captured()))
    manifest = ReportTests().manifest().model_copy(update={"latency_outcomes": True})
    summary = build_summary(manifest, (results,))
    # Then the reduced document names each metric's direction.
    assert summary.metric_directions["pkt_belated_delta"] == "lower_is_better"
    assert summary.metric_directions["useful_goodput_bps"] == "higher_is_better"
    # And the cell metric table carries the latency-as-outcome fields.
    metrics = summary.groups[0].cells["adaptive"].metrics
    for name in (
        "msrcvbuf_p50",
        "msrcvbuf_p95",
        "headroom_min_ms",
        "packets_per_second",
        "lossmaxttl_ms_equivalent",
        "tolerance_saturated",
        "floor_clamped",
    ):
        assert name in metrics, name


def test_legacy_manifest_invents_no_fields(tmp_path: Path) -> None:
    # Given the same capture reduced under a manifest that does not opt in.
    results = tmp_path / "results"
    results.mkdir()
    _ = (results / "run-0.json").write_text(json.dumps(captured()))
    summary = build_summary(ReportTests().manifest(), (results,))
    payload = json.loads(summary.model_dump_json())
    # Then no new key appears anywhere in the reduced document.
    assert "metric_directions" not in payload
    assert "metrics" not in payload["groups"][0]["cells"]["adaptive"]["comparisons"]["adaptive"]
    assert "msrcvbuf_p95" not in payload["groups"][0]["cells"]["adaptive"]["metrics"]


def test_a_record_without_outcome_evidence_materialises_no_null_field() -> None:
    # Given a run carrying no receiver-CSV outcome evidence, as every frozen
    # conformance record in an existing summary does.
    absent = json.loads(ReportTests().record().model_dump_json())
    # Then not one additive field is written, because null is still a byte change.
    assert not set(absent) & set(LATENCY_FIELDS)
    # And a run that does carry the evidence keeps every field it measured.
    present = json.loads(
        ReportTests()
        .record()
        .model_copy(update={"msrcvbuf_p95": P95, "floor_clamped": False})
        .model_dump_json()
    )
    assert present["msrcvbuf_p95"] == pytest.approx(P95)
    assert present["floor_clamped"] is False


def test_m4_reduction_output_is_byte_identical() -> None:
    # Given the frozen M4 primary reduction, which predates every field added here.
    frozen = (REPO / "docs/evidence/bpc/m4/a/summary.json").read_text(encoding="utf-8")
    # When it is re-reduced through the current schema.
    republished = Summary.model_validate_json(frozen).model_dump_json(indent=2) + "\n"
    # Then the bytes are unchanged: absent additions are omitted, never written as null.
    assert republished == frozen
