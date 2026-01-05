//! A proptest generator for U-Blox MON-MSGPP messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-MSGPP message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// UBX Sync Character 1 (0xB5 = 'µ')
const SYNC_CHAR_1: u8 = 0xB5;
/// UBX Sync Character 2 (0x62 = 'b')
const SYNC_CHAR_2: u8 = 0x62;

/// Number of I/O ports
const NUM_PORTS: usize = 6;
/// Number of protocols per port
const NUM_PROTOCOLS: usize = 8;

/// Represents the MON-MSGPP payload (120 bytes fixed).
#[derive(Debug, Clone)]
pub struct MonMsgppPayload {
    /// Message counts per protocol for ports 0-5 (8 protocols each)
    pub msg: [[u16; NUM_PROTOCOLS]; NUM_PORTS],
    /// Skipped bytes per port
    pub skipped: [u32; NUM_PORTS],
}

impl MonMsgppPayload {
    /// Serializes this payload into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(120);
        for port_msgs in &self.msg {
            for &count in port_msgs {
                wtr.write_u16::<LittleEndian>(count).unwrap();
            }
        }
        for &skip in &self.skipped {
            wtr.write_u32::<LittleEndian>(skip).unwrap();
        }
        wtr
    }
}

/// A proptest strategy for generating a MonMsgppPayload.
fn mon_msgpp_payload_strategy() -> impl Strategy<Value = MonMsgppPayload> {
    (
        prop::array::uniform6(prop::array::uniform8(any::<u16>())),
        prop::array::uniform6(any::<u32>()),
    )
        .prop_map(|(msg, skipped)| MonMsgppPayload { msg, skipped })
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
/// containing a MON-MSGPP message, along with the source payload data.
pub fn ubx_mon_msgpp_frame_strategy() -> impl Strategy<Value = (MonMsgppPayload, Vec<u8>)> {
    mon_msgpp_payload_strategy().prop_map(|payload_data| {
        let payload = payload_data.to_bytes();

        let class_id = 0x0a;
        let message_id = 0x06;
        let length = payload.len() as u16;

        // Build the frame core (class, id, length, payload)
        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        // Assemble the final frame
        let mut final_frame = Vec::with_capacity(8 + payload.len());
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
    fn test_parser_proto14_with_generated_mon_msgpp_frames(
        (expected, frame) in ubx_mon_msgpp_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};
        use ublox::mon_msgpp::{parse_port_msg, parse_skipped};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<256>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonMsgpp(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-MSGPP packet");
        };

        // Parse raw bytes using helper functions
        let msg1 = parse_port_msg(&p.msg1());
        let msg2 = parse_port_msg(&p.msg2());
        let msg3 = parse_port_msg(&p.msg3());
        let msg4 = parse_port_msg(&p.msg4());
        let msg5 = parse_port_msg(&p.msg5());
        let msg6 = parse_port_msg(&p.msg6());
        let skipped = parse_skipped(&p.skipped());

        // Verify parsed data matches expected
        prop_assert_eq!(msg1, expected.msg[0], "msg1 mismatch");
        prop_assert_eq!(msg2, expected.msg[1], "msg2 mismatch");
        prop_assert_eq!(msg3, expected.msg[2], "msg3 mismatch");
        prop_assert_eq!(msg4, expected.msg[3], "msg4 mismatch");
        prop_assert_eq!(msg5, expected.msg[4], "msg5 mismatch");
        prop_assert_eq!(msg6, expected.msg[5], "msg6 mismatch");
        prop_assert_eq!(skipped, expected.skipped, "skipped mismatch");
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_msgpp_frames(
        (expected, frame) in ubx_mon_msgpp_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};
        use ublox::mon_msgpp::{parse_port_msg, parse_skipped};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<256>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonMsgpp(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-MSGPP packet");
        };

        // Parse raw bytes using helper functions
        let msg1 = parse_port_msg(&p.msg1());
        let msg2 = parse_port_msg(&p.msg2());
        let msg3 = parse_port_msg(&p.msg3());
        let msg4 = parse_port_msg(&p.msg4());
        let msg5 = parse_port_msg(&p.msg5());
        let msg6 = parse_port_msg(&p.msg6());
        let skipped = parse_skipped(&p.skipped());

        // Verify parsed data matches expected
        prop_assert_eq!(msg1, expected.msg[0], "msg1 mismatch");
        prop_assert_eq!(msg2, expected.msg[1], "msg2 mismatch");
        prop_assert_eq!(msg3, expected.msg[2], "msg3 mismatch");
        prop_assert_eq!(msg4, expected.msg[3], "msg4 mismatch");
        prop_assert_eq!(msg5, expected.msg[4], "msg5 mismatch");
        prop_assert_eq!(msg6, expected.msg[5], "msg6 mismatch");
        prop_assert_eq!(skipped, expected.skipped, "skipped mismatch");
    }
}
