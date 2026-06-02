//! A proptest generator for U-Blox MON-RXR messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-RXR message.

use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

mod common;
use common::build_ubx_frame;

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

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a MON-RXR message.
pub fn ubx_mon_rxr_frame_strategy() -> impl Strategy<Value = (MonRxrPayload, Vec<u8>)> {
    any::<u8>().prop_map(|flags| {
        let payload_data = MonRxrPayload { flags };

        let final_frame = build_ubx_frame(0x0a, 0x21, &[flags]);

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
