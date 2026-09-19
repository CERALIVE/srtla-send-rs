import pytest
from pathlib import Path

from report import EvidenceError
from twinport_models import Comparison, decide, paired_delta
from twinport_models import Measurement, PlayerLeg, Telemetry
from twinport_inputs import validate_player


def test_paired_delta_preserves_correlated_indices() -> None:
    result = paired_delta((0.2, 0.4, 0.6), (0.1, 0.3, 0.5))
    assert result.d_goodput_pct == pytest.approx(10)
    assert result.ci_lower_pct == pytest.approx(10)
    assert result.ci_upper_pct == pytest.approx(10)
    assert not result.delivery_equal


def test_identical_pairs_are_equal_but_missing_publisher_denominator_is_distinct() -> None:
    same = paired_delta((0.2, 0.4, 0.6), (0.2, 0.4, 0.6))
    assert same.delivery_equal
    assert same.d_loss_pp is None
    verdict = decide((same, same))
    assert not verdict.alias_equivalent
    assert verdict.outcome == "distinct"
    assert verdict.selected_alternative == "run the reserved 4003 block in M4"


def test_loss_boundary_is_percentage_points_and_absolute() -> None:
    same = paired_delta((0.2, 0.4, 0.6), (0.2, 0.4, 0.6))
    passing = same.model_copy(update={"d_loss_pp": 0.05})
    assert decide((passing, passing)).alias_equivalent
    failing = same.model_copy(update={"d_loss_pp": -0.050001})
    assert not decide((passing, failing)).alias_equivalent


def test_incomplete_pairs_have_no_inference() -> None:
    incomplete = Comparison(n=0)
    assert decide((incomplete, incomplete)).d_goodput_pct is None
    with pytest.raises(EvidenceError):
        paired_delta((0.9, 0.9), (0.9, 0.9))


def test_spike_has_exact_requested_keys() -> None:
    result = decide((Comparison(n=0), Comparison(n=0)))
    assert set(result.model_dump()) == {
        "spike", "alias_equivalent", "d_goodput_pct", "d_loss_pp",
        "outcome", "selected_alternative",
    }


def test_player_csv_must_prove_full_window_and_actual_latency(tmp_path: Path) -> None:
    measurement = Measurement(settled=True, player_leg_valid=True,
        player_leg=PlayerLeg(loss=0, drop=0, mbps_recv_rate=9.6), player_latency_ms=200,
        offered_bps=9_600_000, player_bytes=108_000_000, sender_bytes=110_000_000,
        sender_start=Telemetry(schema_version=1, last_updated_ms=1000, bytes_sent_total=0),
        sender_end=Telemetry(schema_version=1, last_updated_ms=91000, bytes_sent_total=110_000_000),
        player_csv_start_ms=1000, player_csv_end_ms=91000, elapsed_ms=90000, sls_override=None)
    path = tmp_path / "player.csv"
    header = "Time,SocketID,Time,pktRcvLoss,pktRcvDrop,mbpsRecvRate,msRcvTsbPdDelay\n"
    path.write_text(header + "now,7,91000,0,0,9.6,200\n")
    with pytest.raises(EvidenceError):
        validate_player(path, measurement)
    path.write_text(header + "".join(f"now,7,{t},0,0,9.6,200\n" for t in range(1000, 91001, 100)))
    validate_player(path, measurement)
    path.write_text(path.read_text().replace(",9.6,200", ",9.6,100"))
    with pytest.raises(EvidenceError):
        validate_player(path, measurement)


@pytest.mark.parametrize("loss,drop,rate", [(1,0,10), (0,1,10), (0,0,9.407999)])
def test_player_rate_or_counter_failure_cannot_count(loss: int, drop: int, rate: float) -> None:
    assert not PlayerLeg(loss=loss, drop=drop, mbps_recv_rate=rate).valid(9_600_000)
