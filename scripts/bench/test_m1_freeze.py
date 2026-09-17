from m1_freeze import Packet, gaps_from_packets


def data(seq: int, at: float) -> Packet:
    return Packet(at, 30000, 4001, seq.to_bytes(4, "big") + bytes(12))


def nak(words: tuple[int, ...], at: float) -> Packet:
    return Packet(
        at,
        4001,
        30000,
        b"\x80\x03" + bytes(14) + b"".join(w.to_bytes(4, "big") for w in words),
    )


def test_gap_to_first_nak_and_repair_are_distinct() -> None:
    packets = (
        data(99, 0),
        data(101, 10),
        nak((100,), 50),
        nak((100,), 55),
        data(100, 110),
    )
    (gap,) = gaps_from_packets(packets)
    assert (gap.detected_ms, gap.nak_ms, gap.repaired_ms) == (10, 50, 110)


def test_unrepaired_gap_is_retained_as_censored() -> None:
    packets = (data(99, 0), data(101, 10), nak((100,), 50), data(102, 110))
    (gap,) = gaps_from_packets(packets)
    assert gap.repaired_ms is None


def test_nak_range_and_sequence_wrap() -> None:
    packets = (data(0x7FFFFFFD, 0), data(1, 10), nak((0xFFFFFFFE, 0), 50), data(0, 110))
    gaps = gaps_from_packets(packets)
    assert len(gaps) == 3
    assert all(gap.nak_ms == 50 for gap in gaps)
    assert sum(gap.repaired_ms is not None for gap in gaps) == 1


def test_duplicate_data_does_not_invent_a_gap() -> None:
    assert gaps_from_packets((data(99, 0), data(99, 5), data(100, 10))) == ()


def test_no_nak_or_repair_stays_unknown_with_censored_bounds() -> None:
    from m1_freeze import summarize

    result = summarize((data(99, 0), data(101, 10), data(102, 1010)), "a" * 64)
    assert result.first_nak_ms is None
    assert result.recovery_ms is None
    assert result.recovery_lower_bound_ms == 1000
    assert result.recovery_upper_bound_ms is None
    assert result.censored == 1
