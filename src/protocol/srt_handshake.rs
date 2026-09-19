//! Passive SRT v5 HSRSP decoding; offsets pinned by captured receiver traffic.

use super::{SRT_TYPE_HANDSHAKE, get_packet_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HsrspInfo {
    pub srt_version: u32,
    pub flags: u32,
    pub tsbpd_delay_ms: u32,
}

/// Receiver capabilities from a complete v5 conclusion HSRSP.
/// Unknown extensions are skipped; malformed/truncated input silently returns `None`.
pub fn parse_hsrsp(buf: &[u8]) -> Option<HsrspInfo> {
    let header = buf.get(..64)?;
    if get_packet_type(header)? != SRT_TYPE_HANDSHAKE
        || header[16..20] != 5_u32.to_be_bytes()
        || u16::from_be_bytes([header[22], header[23]]) & 1 == 0
        || header[36..40] != u32::MAX.to_be_bytes()
    {
        return None;
    }

    let mut extensions = buf.get(64..)?;
    let mut info = None;
    while !extensions.is_empty() {
        let descriptor = extensions.get(..4)?;
        let command = u16::from_be_bytes([descriptor[0], descriptor[1]]);
        let words = usize::from(u16::from_be_bytes([descriptor[2], descriptor[3]]));
        let end = 4 + 4 * words;
        let block = extensions.get(..end)?;
        if command == 2 {
            // SRT version at E+4, flags at E+8, latency at E+12 (receiver high half).
            let latency = block.get(12..16)?;
            info = Some(HsrspInfo {
                srt_version: u32::from_be_bytes(block.get(4..8)?.try_into().ok()?),
                flags: u32::from_be_bytes(block.get(8..12)?.try_into().ok()?),
                tsbpd_delay_ms: u32::from(u16::from_be_bytes([latency[0], latency[1]])),
            });
        }
        extensions = extensions.get(end..)?;
    }
    info
}

pub fn parse_hsrsp_tsbpd_delay_ms(buf: &[u8]) -> Option<u32> {
    parse_hsrsp(buf).map(|info| info.tsbpd_delay_ms)
}

#[cfg(test)]
mod tests {
    use super::parse_hsrsp_tsbpd_delay_ms;

    #[test]
    fn captured_hsrsp_decodes_version_and_nak_on_flags() {
        let info = super::parse_hsrsp(HSRSP).unwrap();
        assert_eq!(info.srt_version, 0x010505);
        assert_eq!(info.flags, 0xbf);
        assert_ne!(info.flags & crate::protocol::SRT_OPT_NAKREPORT, 0);
        assert_eq!(info.tsbpd_delay_ms, 2000);
    }

    #[test]
    fn captured_hsrsp_decodes_nak_off_flags() {
        let packet = include_bytes!("../../tests/fixtures/srt-hsrsp-nak-off.bin");
        let info = super::parse_hsrsp(packet).unwrap();
        assert_eq!(info.flags, 0xaf);
        assert_eq!(info.flags & crate::protocol::SRT_OPT_NAKREPORT, 0);
        assert_eq!(info.srt_version, 0x010505);
        assert_eq!(info.tsbpd_delay_ms, 2000);
    }

    #[test]
    fn clearing_nak_bit_preserves_other_hsrsp_fields() {
        let mut packet = HSRSP.to_vec();
        packet[72..76].copy_from_slice(&0xaf_u32.to_be_bytes());
        let info = super::parse_hsrsp(&packet).unwrap();
        assert_eq!(info.flags & crate::protocol::SRT_OPT_NAKREPORT, 0);
        assert_eq!(info.flags, 0xaf);
        assert_eq!(info.srt_version, 0x010505);
        assert_eq!(info.tsbpd_delay_ms, 2000);
    }

    #[test]
    fn hsrsp_fields_follow_extension_walk_and_preserve_unknown_bits() {
        let mut packet = HSRSP[..64].to_vec();
        packet.extend_from_slice(&[0, 99, 0, 1, 0, 0, 0, 0]);
        packet.extend_from_slice(&HSRSP[64..]);
        packet[76..80].copy_from_slice(&0x010506_u32.to_be_bytes());
        packet[80..84].copy_from_slice(&0x8000_00bf_u32.to_be_bytes());
        assert_eq!(
            super::parse_hsrsp(&packet),
            Some(super::HsrspInfo {
                srt_version: 0x010506,
                flags: 0x8000_00bf,
                tsbpd_delay_ms: 2000,
            })
        );
    }

    #[test]
    fn hsrsp_flag_constants_match_srt_wire_bits() {
        use crate::protocol::*;
        assert_eq!(
            [
                SRT_OPT_TSBPDSND,
                SRT_OPT_TSBPDRCV,
                SRT_OPT_TLPKTDROP,
                SRT_OPT_NAKREPORT,
                SRT_OPT_REXMITFLG,
                SRT_OPT_STREAM,
                SRT_OPT_FILTERCAP
            ],
            [1, 2, 8, 16, 32, 64, 128]
        );
    }

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
