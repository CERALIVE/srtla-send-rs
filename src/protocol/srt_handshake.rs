//! Passive SRT v5 HSRSP decoding; offsets pinned by captured receiver traffic.

use super::{SRT_TYPE_HANDSHAKE, get_packet_type};

/// Receiver TSBPD delay in milliseconds from a complete v5 conclusion HSRSP.
/// Unknown extensions are skipped; malformed/truncated input silently returns `None`.
pub fn parse_hsrsp_tsbpd_delay_ms(buf: &[u8]) -> Option<u32> {
    let header = buf.get(..64)?;
    if get_packet_type(header)? != SRT_TYPE_HANDSHAKE
        || header[16..20] != 5_u32.to_be_bytes()
        || u16::from_be_bytes([header[22], header[23]]) & 1 == 0
        || header[36..40] != u32::MAX.to_be_bytes()
    {
        return None;
    }

    let mut extensions = buf.get(64..)?;
    let mut delay = None;
    while !extensions.is_empty() {
        let descriptor = extensions.get(..4)?;
        let command = u16::from_be_bytes([descriptor[0], descriptor[1]]);
        let words = usize::from(u16::from_be_bytes([descriptor[2], descriptor[3]]));
        let end = 4 + 4 * words;
        let block = extensions.get(..end)?;
        if command == 2 {
            // SRT version at E+4, flags at E+8, latency at E+12 (receiver high half).
            let latency = block.get(12..16)?;
            delay = Some(u32::from(u16::from_be_bytes([latency[0], latency[1]])));
        }
        extensions = extensions.get(end..)?;
    }
    delay
}

#[cfg(test)]
mod tests {
    use super::parse_hsrsp_tsbpd_delay_ms;

    const HSRSP: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/srt-hsrsp-latency2000.bin"
    ));

    #[test]
    fn captured_srt_handshake_decodes_receiver_latency() {
        // Given the real 2000 ms conclusion, When sniffed, Then decode milliseconds.
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(HSRSP), Some(2000));
    }

    #[test]
    fn truncated_srt_handshake_is_rejected_at_every_boundary() {
        // Given every proper prefix, When sniffed, Then no incomplete frame is accepted.
        for end in 0..HSRSP.len() {
            assert_eq!(parse_hsrsp_tsbpd_delay_ms(&HSRSP[..end]), None, "{end}");
        }
    }

    #[test]
    fn garbage_srt_handshake_is_rejected() {
        // Given garbage, When sniffed, Then no delay is invented.
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(&[0xff; 128]), None);
    }

    #[test]
    fn non_handshake_packet_is_rejected() {
        // Given a captured frame with DATA type, When sniffed, Then reject it.
        let mut packet = HSRSP.to_vec();
        packet[0] = 0;
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), None);
    }

    #[test]
    fn flipped_hsrsp_extension_type_is_rejected() {
        // Given a one-byte mutation at the proven descriptor, When sniffed, Then no HSRSP.
        let mut packet = HSRSP.to_vec();
        packet[65] ^= 1;
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), None);
    }

    #[test]
    fn non_v5_or_non_conclusion_or_missing_hs_flag_is_rejected() {
        // Given each invalid handshake header, When sniffed, Then reject it.
        for (offset, value) in [(19, 4), (39, 1), (23, 0)] {
            let mut packet = HSRSP.to_vec();
            packet[offset] = value;
            assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), None);
        }
    }

    #[test]
    fn hsrsp_after_unrelated_extension_decodes_high_half() {
        // Given the real HSRSP after a one-word unrelated block and unequal delays.
        let mut packet = HSRSP[..64].to_vec();
        packet.extend_from_slice(&[0, 5, 0, 1, 0, 0, 0, 0]);
        packet.extend_from_slice(&HSRSP[64..]);
        packet[86..88].copy_from_slice(&500_u16.to_be_bytes());
        // When sniffed, Then use the receiver half at E+12, not the sender half.
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), Some(2000));
    }

    #[test]
    fn hsrsp_body_must_contain_latency_and_fit_the_packet() {
        // Given short and oversized declared bodies, When sniffed, Then reject them.
        for words in [0_u16, 1, 2, 4, u16::MAX] {
            let mut packet = HSRSP.to_vec();
            packet[66..68].copy_from_slice(&words.to_be_bytes());
            assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), None);
        }
    }

    #[test]
    fn truncated_extension_after_hsrsp_is_rejected() {
        // Given a complete HSRSP followed by a partial descriptor, When sniffed, Then reject.
        let mut packet = HSRSP.to_vec();
        packet.push(0);
        assert_eq!(parse_hsrsp_tsbpd_delay_ms(&packet), None);
    }
}
