//! A proptest generator for U-Blox MON-RXR messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-RXR message.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// UBX Sync Character 1 (0xB5 = 'µ')
const SYNC_CHAR_1: u8 = 0xB5;
/// UBX Sync Character 2 (0x62 = 'b')
const SYNC_CHAR_2: u8 = 0x62;

/// Represents the MON-RXR payload (1 byte).
#[derive(Debug, Clone)]
pub struct MonRxrPayload {
    pub flags: u8,
}

impl MonRxrPayload {
    pub fn awake(&self) -> bool {
        self.flags & 0x01 != 0
    }
}

/// Calculates the 8-bit Fletcher-16 checksum used by U-Blox.
fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a MON-RXR message.
pub fn ubx_mon_rxr_frame_strategy() -> impl Strategy<Value = (MonRxrPayload, Vec<u8>)> {
    any::<u8>().prop_map(|flags| {
        let payload_data = MonRxrPayload { flags };

        let class_id = 0x0a;
        let message_id = 0x21;
        let length: u16 = 1;

        let mut frame_core = Vec::with_capacity(5);
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.push(flags);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(9);
        final_frame.push(SYNC_CHAR_1);
        final_frame.push(SYNC_CHAR_2);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (payload_data, final_frame)
    })
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_rxr_frames(
        (expected, frame) in ubx_mon_rxr_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<64>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonRxr(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-RXR packet");
        };

        prop_assert_eq!(p.flags().awake(), expected.awake());
        prop_assert_eq!(p.flags().raw(), expected.flags);
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_rxr_frames(
        (expected, frame) in ubx_mon_rxr_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<64>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonRxr(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-RXR packet");
        };

        prop_assert_eq!(p.flags().awake(), expected.awake());
        prop_assert_eq!(p.flags().raw(), expected.flags);
    }
}
