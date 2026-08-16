use smallvec::SmallVec;

use super::constants::*;
use super::srt_seq::SrtSeq;
use super::types::{ConnectionInfo, get_packet_type};

pub fn extract_keepalive_timestamp(buf: &[u8]) -> Option<u64> {
    if buf.len() < 10 {
        return None;
    }
    if get_packet_type(buf)? != SRTLA_TYPE_KEEPALIVE {
        return None;
    }
    let mut ts: u64 = 0;
    for i in 0..8 {
        ts = (ts << 8) | (buf[2 + i] as u64);
    }
    Some(ts)
}

/// Extract connection info from extended keepalive packet
///
/// Returns None if:
/// - Packet is too short (< 38 bytes)
/// - Not a KEEPALIVE packet
/// - Magic number doesn't match (not an extended keepalive)
/// - Version doesn't match
#[allow(dead_code)]
pub fn extract_keepalive_conn_info(buf: &[u8]) -> Option<ConnectionInfo> {
    if buf.len() < SRTLA_KEEPALIVE_EXT_LEN {
        return None;
    }
    if get_packet_type(buf)? != SRTLA_TYPE_KEEPALIVE {
        return None;
    }

    // Check magic number at bytes 10-11
    let magic = u16::from_be_bytes([buf[10], buf[11]]);
    if magic != SRTLA_KEEPALIVE_MAGIC {
        return None;
    }

    // Check version at bytes 12-13
    let version = u16::from_be_bytes([buf[12], buf[13]]);
    if version != SRTLA_KEEPALIVE_EXT_VERSION {
        return None;
    }

    // Parse connection info
    let conn_id = u32::from_be_bytes([buf[14], buf[15], buf[16], buf[17]]);
    let window = i32::from_be_bytes([buf[18], buf[19], buf[20], buf[21]]);
    let in_flight = i32::from_be_bytes([buf[22], buf[23], buf[24], buf[25]]);
    let rtt_ms = u32::from_be_bytes([buf[26], buf[27], buf[28], buf[29]]);
    let nak_count = u32::from_be_bytes([buf[30], buf[31], buf[32], buf[33]]);
    let bitrate_bytes_per_sec = u32::from_be_bytes([buf[34], buf[35], buf[36], buf[37]]);

    Some(ConnectionInfo {
        conn_id,
        window,
        in_flight,
        rtt_ms,
        nak_count,
        bitrate_bytes_per_sec,
    })
}

/// Decode the acknowledged sequence number of an SRT cumulative ACK.
///
/// The word is a bare sequence number, so bit 31 being set means the frame is
/// corrupt or hostile, not that a flag is present: such a frame is rejected
/// rather than masked. That keeps every value this function yields inside the
/// 31-bit domain, so the downstream `as i32` packet-log cast can never go
/// negative. Same rule [`parse_srt_nak`] applies to a range-end word.
#[inline]
pub fn parse_srt_ack(buf: &[u8]) -> Option<u32> {
    if buf.len() < 20 {
        return None;
    }
    if get_packet_type(buf)? != SRT_TYPE_ACK {
        return None;
    }
    let word = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    SrtSeq::from_u32_checked(word).map(SrtSeq::value)
}

/// Sequence numbers decoded from one SRT NAK frame, plus whether the decode hit
/// the [`SRT_NAK_MAX_ENTRIES`] cap.
///
/// `truncated` exists so the loss report is not silently incomplete: the parser
/// stays pure (it neither logs nor allocates unboundedly) and the caller decides
/// how to surface the condition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NakList {
    pub seqs: SmallVec<u32, 4>,
    pub truncated: bool,
}

impl NakList {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.seqs.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.seqs.len()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u32] {
        self.seqs.as_slice()
    }
}

impl IntoIterator for NakList {
    type Item = u32;
    type IntoIter = smallvec::IntoIter<u32, 4>;

    fn into_iter(self) -> Self::IntoIter {
        self.seqs.into_iter()
    }
}

/// Decode the loss list of an SRT NAK control packet.
///
/// The loss list begins at [`SRT_CONTROL_HEADER_LEN`] (16), NOT at 4 — see that
/// constant for what the intervening 12 bytes are. A frame shorter than
/// `16 + 4` therefore carries no loss list at all and yields an empty result.
///
/// Each 4-byte word is either a bare sequence number (bit 31 clear) or a range
/// start (bit 31 set) followed by a range end word. Ranges are expanded with
/// [`SrtSeq::next`] under 31-bit serial semantics, so a range that crosses the
/// `0x7FFF_FFFF → 0` wrap expands correctly instead of running away into
/// `0x8000_0000`+ values that are not sequence numbers.
///
/// Malformed pieces are skipped, not fatal: a range whose end word has bit 31
/// set is dropped and parsing continues with the next word; a descending range
/// (and the RFC1982-ambiguous antipodal case) emits nothing.
#[inline]
pub fn parse_srt_nak(buf: &[u8]) -> NakList {
    let mut out = NakList::default();
    if buf.len() < SRT_CONTROL_HEADER_LEN + 4 {
        return out;
    }
    if get_packet_type(buf) != Some(SRT_TYPE_NAK) {
        return out;
    }

    let mut i = SRT_CONTROL_HEADER_LEN;
    while i + 3 < buf.len() {
        let word = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
        i += 4;

        if word & 0x8000_0000 == 0 {
            if out.seqs.len() >= SRT_NAK_MAX_ENTRIES {
                out.truncated = true;
                return out;
            }
            out.seqs.push(word);
            continue;
        }

        // Range: the high bit is the marker, so masking it off always yields a
        // valid start. The end word is a bare sequence number and is rejected
        // outright if its high bit is set.
        let start = SrtSeq::new(word);
        if i + 3 >= buf.len() {
            break;
        }
        let end_word = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
        i += 4;
        let Some(end) = SrtSeq::from_u32_checked(end_word) else {
            continue;
        };
        // Descending ranges, and the antipodal pair where serial order is
        // undefined, expand to nothing rather than to a half-space of seqs.
        if !start.serial_le(end) {
            continue;
        }

        let mut seq = start;
        loop {
            if out.seqs.len() >= SRT_NAK_MAX_ENTRIES {
                out.truncated = true;
                return out;
            }
            out.seqs.push(seq.value());
            // Stop AFTER emitting `end` so an inclusive range ending at
            // SrtSeq::MAX emits exactly its members and never wraps past them.
            if seq == end {
                break;
            }
            seq = seq.next();
        }
    }
    out
}

/// Decode the acknowledged sequence numbers of an SRTLA ACK frame.
///
/// Every word is a bare sequence number; one with bit 31 set is malformed and
/// is DISCARDED while the rest of the frame still decodes, so a single corrupt
/// word does not throw away the valid acks beside it. Same 31-bit output
/// guarantee as [`parse_srt_ack`].
#[inline]
pub fn parse_srtla_ack(buf: &[u8]) -> SmallVec<u32, 4> {
    if buf.len() < 8 {
        return SmallVec::new();
    }
    if get_packet_type(buf) != Some(SRTLA_TYPE_ACK) {
        return SmallVec::new();
    }
    let mut out = SmallVec::new();

    // Match original C implementation behavior: skip first 4 bytes, not 2
    // The C code does: uint32_t *acks = (uint32_t *)buf; for (int i = 1; ...)
    // which effectively skips acks[0] (first 4 bytes)
    let mut i = 4usize; // Skip packet type + padding (4 bytes total)
    while i + 3 < buf.len() {
        let word = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
        i += 4;
        if let Some(ack) = SrtSeq::from_u32_checked(word) {
            out.push(ack.value());
        }
    }
    out
}
