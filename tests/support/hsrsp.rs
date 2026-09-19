use srtla_send::protocol::SRT_TYPE_HANDSHAKE;

#[derive(Debug, PartialEq, Eq)]
pub struct Hsrsp {
    pub extension_offset: usize,
    pub latency_offset: usize,
    pub high_ms: u16,
    pub low_ms: u16,
}

fn be_word(packet: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        packet.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

// Clean-room layout from Haivision's handshake.md: 16-byte header, 48-byte CIF,
// then descriptors {type:u16, length:u16}, with lengths in body-only u32 words.
pub fn decode_hsrsp(packet: &[u8]) -> Option<Hsrsp> {
    if be_word(packet, 0)? != u32::from(SRT_TYPE_HANDSHAKE) << 16
        || be_word(packet, 16)? != 5
        || be_word(packet, 36)? != u32::MAX
        || be_word(packet, 20)? & 1 == 0
    {
        return None;
    }
    let mut offset = 64;
    while offset < packet.len() {
        let descriptor = be_word(packet, offset)?;
        let body_size = usize::from(u16::try_from(descriptor & 0xffff).ok()?) * 4;
        let body = packet.get(offset + 4..offset + 4 + body_size)?;
        if descriptor >> 16 == 2 {
            let latency = be_word(body, 8)?;
            return Some(Hsrsp {
                extension_offset: offset,
                latency_offset: offset + 12,
                high_ms: u16::try_from(latency >> 16).ok()?,
                low_ms: u16::try_from(latency & 0xffff).ok()?,
            });
        }
        offset += 4 + body_size;
    }
    None
}

#[test]
fn decodes_distinct_halves_after_an_unrelated_extension() {
    // Given: a v5 conclusion, an unrelated one-word extension, then HSRSP.
    let mut packet = vec![0; 88];
    packet[0..2].copy_from_slice(&0x8000_u16.to_be_bytes());
    packet[16..20].copy_from_slice(&5_u32.to_be_bytes());
    packet[22..24].copy_from_slice(&1_u16.to_be_bytes());
    packet[36..40].copy_from_slice(&u32::MAX.to_be_bytes());
    packet[64..68].copy_from_slice(&[0, 99, 0, 1]);
    packet[72..76].copy_from_slice(&[0, 2, 0, 3]);
    packet[84..88].copy_from_slice(&[0x07, 0xd0, 0, 120]);
    // When: walking extension lengths, not assuming HSRSP is first.
    let decoded = decode_hsrsp(&packet);
    // Then: offsets and both halves retain their independent wire values.
    assert_eq!(
        decoded,
        Some(Hsrsp {
            extension_offset: 72,
            latency_offset: 84,
            high_ms: 2000,
            low_ms: 120,
        })
    );
}
