use super::constants::*;

/// Connection info data for extended keepalive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionInfo {
    pub conn_id: u32,
    pub window: i32,
    pub in_flight: i32,
    pub rtt_ms: u32,
    pub nak_count: u32,
    pub bitrate_bytes_per_sec: u32,
}

/// Helper functions for packet type checking (used in tests)
#[allow(dead_code)]
pub fn is_srtla_reg1(buf: &[u8]) -> bool {
    buf.len() == SRTLA_TYPE_REG1_LEN && get_packet_type(buf) == Some(SRTLA_TYPE_REG1)
}

#[allow(dead_code)]
pub fn is_srtla_reg2(buf: &[u8]) -> bool {
    buf.len() == SRTLA_TYPE_REG2_LEN && get_packet_type(buf) == Some(SRTLA_TYPE_REG2)
}

#[allow(dead_code)]
pub fn is_srtla_reg3(buf: &[u8]) -> bool {
    buf.len() == SRTLA_TYPE_REG3_LEN && get_packet_type(buf) == Some(SRTLA_TYPE_REG3)
}

#[allow(dead_code)]
pub fn is_srtla_keepalive(buf: &[u8]) -> bool {
    get_packet_type(buf) == Some(SRTLA_TYPE_KEEPALIVE)
}

#[allow(dead_code)]
pub fn is_srt_ack(buf: &[u8]) -> bool {
    get_packet_type(buf) == Some(SRT_TYPE_ACK)
}

#[inline]
pub fn get_packet_type(buf: &[u8]) -> Option<u16> {
    if buf.len() < 2 {
        return None;
    }
    Some(u16::from_be_bytes([buf[0], buf[1]]))
}

#[inline]
pub fn get_srt_sequence_number(buf: &[u8]) -> Option<u32> {
    if buf.len() < 4 {
        return None;
    }
    let sn = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if (sn & 0x8000_0000) == 0 {
        Some(sn)
    } else {
        None
    }
}

/// Extract the SRT retransmit flag (R bit, bit 26) from the message number field.
/// Returns `Some(true)` if the R bit is set, `Some(false)` if clear, or `None` if
/// the packet is not a DATA packet (control packets have no message number).
///
/// The R bit is located at bit 26 of the 32-bit message number field (header word 2,
/// bytes 4-7). For DATA packets, the top bit (bit 31) is 0; for control packets it's 1.
/// This function returns `None` for control packets.
#[inline]
pub fn get_srt_rexmit_flag(buf: &[u8]) -> Option<bool> {
    if buf.len() < 4 {
        return None;
    }
    let msgno = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    // Control packets have bit 31 set; DATA packets have it clear
    if (msgno & 0x8000_0000) != 0 {
        return None; // Control packet, no R bit
    }
    // R bit is at bit 26: 0x0400_0000
    Some((msgno & 0x0400_0000) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rexmit_flag_set_on_data_packet() {
        // DATA packet with R bit set (bit 26 = 1)
        let msgno = 0x0400_0001u32; // R bit set, seq = 1
        let buf = msgno.to_be_bytes();
        assert_eq!(get_srt_rexmit_flag(&buf), Some(true));
    }

    #[test]
    fn rexmit_flag_clear_on_data_packet() {
        // DATA packet with R bit clear (bit 26 = 0)
        let msgno = 0x0000_0001u32; // R bit clear, seq = 1
        let buf = msgno.to_be_bytes();
        assert_eq!(get_srt_rexmit_flag(&buf), Some(false));
    }

    #[test]
    fn rexmit_flag_none_on_control_packet() {
        // Control packet (bit 31 = 1)
        let msgno = 0x8000_0001u32; // Control packet
        let buf = msgno.to_be_bytes();
        assert_eq!(get_srt_rexmit_flag(&buf), None);
    }

    #[test]
    fn rexmit_flag_none_on_short_buffer() {
        // Buffer too short
        let buf = [0u8; 3];
        assert_eq!(get_srt_rexmit_flag(&buf), None);
    }

    #[test]
    fn rexmit_flag_never_panics_on_malformed() {
        // Arbitrary bytes should never panic
        let test_cases = vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 0, 0],
            vec![0xff, 0xff, 0xff, 0xff],
            vec![0x04, 0x00, 0x00, 0x00],
        ];
        for buf in test_cases {
            let _ = get_srt_rexmit_flag(&buf); // Should not panic
        }
    }
}
