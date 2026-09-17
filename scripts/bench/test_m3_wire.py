from pathlib import Path

import m3_wire
import pytest
import report
from m3_conformance import Bucket, Captured, EventRecord, Raw, recovery_ms


def test_group_identity_and_literal_echo_are_required() -> None:
    seed = bytes(range(256))
    group = seed[:128] + bytes([77])*128
    packets = [m3_wire.Packet(0, 0, 1234, 5000, b"\x92\x00" + seed),
               m3_wire.Packet(0, 1, 5000, 1234, b"\x92\x01" + group)]
    for link in (0, 1):
        packets.extend([m3_wire.Packet(link, 2, 1234, 5000, b"\x92\x01" + group),
                        m3_wire.Packet(link, 3, 5000, 1234, b"\x92\x02"),
                        m3_wire.Packet(link, 4, 1234, 5000, b"\x90\x00"),
                        m3_wire.Packet(link, 5, 5000, 1234, b"\x90\x00")])
    proof = m3_wire.classify(packets, 2)
    assert len(proof.complete_groups) == 1
    assert proof.keepalive_echoes == (1, 1)
    assert proof.keepalive_lengths == (2,)
    assert proof.mismatched_echoes == 0
    broken = [p for p in packets if p.payload[:2] != b"\x92\x01" or p.source_port != 5000]
    assert m3_wire.classify(broken, 2).complete_groups == ()
    corrupt = [m3_wire.Packet(p.link, p.time_s, p.source_port, p.destination_port,
               p.payload+b"\x00" if p.payload == b"\x90\x00" and p.source_port == 5000 else p.payload) for p in packets]
    assert m3_wire.classify(corrupt, 2).mismatched_echoes == 2


def test_replayed_grants_cannot_create_a_new_receiver_group() -> None:
    packets = [m3_wire.Packet(0, float(i), 5000, 1234, b"\x92\x02") for i in range(10)]
    proof = m3_wire.classify(packets, 2)
    assert proof.complete_groups == ()
    assert proof.keepalive_echoes == (0, 0)


@pytest.mark.parametrize("byte_rate,expected", [(112500, 4000), (112499, None)])
def test_restart_needs_three_full_seconds_at_ninety_percent(byte_rate: int, expected: int | None) -> None:
    original = report.ReportTests().record()
    record = Captured.model_validate(original.model_dump() | {
        "raw": Raw(stats_csv_path=Path("/synthetic/receiver.csv"), sink_series=tuple(
            Bucket(t_ms=t, duration_ms=1000, bytes=byte_rate) for t in range(22000, 61000, 1000)
        )),
        "events": (),
        "load_intervals": (report.LoadInterval(start_ms=0, end_ms=60000, offered_bps=1000000,
            target_bps=900000.0, graded=False, no_collapse=True, reached_ms=None, recovered=False),),
    })
    assert recovery_ms(record, 20000) == expected


def test_capture_rejects_kernel_drops_before_tshark(tmp_path: Path) -> None:
    path = tmp_path / "interop-0.pcap"
    path.with_suffix(".capture.log").write_text("1 packets dropped by kernel\n")
    with pytest.raises(report.EvidenceError, match="capture drops"):
        m3_wire.read_capture(path, 0)


def test_scenario_i_parses_real_externally_tagged_offered_rate() -> None:
    event = EventRecord.model_validate_json('{"t_ms":0,"event":{"action":{"OfferedRate":{"bps":12800000}}}}')
    assert event.t_ms == 0
    assert event.event.action != "ReceiverRestart"
