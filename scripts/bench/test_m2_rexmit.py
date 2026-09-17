from pathlib import Path

import pytest

from check_rexmit_bit import CaptureError, Packet, analyze, classify


def packet(seq: int, rexmit: bool, socket: int = 7) -> Packet:
    data = (seq.to_bytes(4, "big") + (int(rexmit) << 26).to_bytes(4, "big")
            + bytes(4) + socket.to_bytes(4, "big"))
    return Packet(1234, data)


def test_visible_flag_requires_independently_observed_repeated_sequence() -> None:
    packets = tuple(packet(i, False) for i in range(1000)) + (packet(2, True),)
    result = classify(packets)
    assert result.passed
    assert (result.originals, result.retransmissions) == (1000, 1)


def test_flag_absent_on_repeat_is_not_visible() -> None:
    packets = tuple(packet(i, False) for i in range(1000)) + (packet(2, False),)
    result = classify(packets)
    assert not result.passed
    assert result.unflagged_retransmissions == 1


def test_all_originals_cannot_vacuously_prove_visibility() -> None:
    assert not classify(tuple(packet(i, False) for i in range(1000))).passed


def test_flagged_original_fails_and_sockets_have_separate_sequence_spaces() -> None:
    result = classify((packet(1, False), packet(1, True, socket=8)))
    assert (result.originals, result.retransmissions, result.flagged_originals) == (2, 0, 1)
    assert not result.passed


def test_control_and_short_packets_do_not_count_as_data() -> None:
    result = classify((Packet(1234, bytes(8)), packet(0x80000000, True)))
    assert result.data_packets == 0


def test_ten_capture_drops_cannot_match_zero_drop_suffix(tmp_path: Path) -> None:
    capture = tmp_path / "absent.pcap"
    capture.with_suffix(".capture.log").write_text("10 packets dropped by kernel\n")
    with pytest.raises(CaptureError, match="zero capture drops"):
        analyze(capture)


def test_source_ports_and_sequence_wrap_keep_originals_distinct() -> None:
    first = packet(0x7FFFFFFF, False)
    result = classify((first, packet(0, False), Packet(4321, first.payload), packet(0, True)))
    assert (result.originals, result.retransmissions) == (3, 1)
    assert (result.flagged_originals, result.unflagged_retransmissions) == (0, 0)
