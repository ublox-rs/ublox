#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox RXM-COR messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a RXM-COR message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents the payload of a UBX-RXM-COR message.
///
/// RXM-COR payload is 12 bytes.
#[derive(Debug, Clone)]
pub struct RxmCorPayload {
    pub version: u8,     // Message version (0x01 for this version)
    pub ebno: u8,        // Eb/N0, 0.125 dB/LSB (raw)
    pub reserved0: [u8; 2], // Reserved
    pub status_info: u32, // Status information bitfield
    pub msg_type: u16,    // Message type
    pub msg_sub_type: u16,// Message subtype
}

impl RxmCorPayload {
    /// Serializes the RxmCorPayload into a 12-byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(12);
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.ebno).unwrap();
        wtr.extend_from_slice(&self.reserved0);
        wtr.write_u32::<LittleEndian>(self.status_info).unwrap();
        wtr.write_u16::<LittleEndian>(self.msg_type).unwrap();
        wtr.write_u16::<LittleEndian>(self.msg_sub_type).unwrap();
        wtr
    }
}

/// Calculates the 8-bit Fletcher-16 checksum used by u-blox.
fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

/// A proptest strategy for generating a `RxmCorPayload` struct.
fn rxm_cor_payload_strategy() -> impl Strategy<Value = RxmCorPayload> {
    (
        Just(1u8),
        any::<u8>(),
        Just([0u8; 2]),
        any::<u32>(),
        any::<u16>(),
        any::<u16>(),
    )
        .prop_map(|(version, ebno, reserved0, status_info, msg_type, msg_sub_type)| RxmCorPayload {
            version,
            ebno,
            reserved0,
            status_info,
            msg_type,
            msg_sub_type,
        })
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a RXM-COR message, along with the source payload struct.
pub fn ubx_rxm_cor_frame_strategy() -> impl Strategy<Value = (RxmCorPayload, Vec<u8>)> {
    rxm_cor_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x02;
        let message_id = 0x34;
        let length = payload.len() as u16;

        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(0xB5);
        final_frame.push(0x62);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (payload_struct, final_frame)
    })
}

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_rxm_cor_frames((expected, frame) in ubx_rxm_cor_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::RxmCor(p)))) = it.next() else {
            panic!("Parser failed to parse a RXM-COR valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.ebno_raw(), expected.ebno);
        prop_assert_eq!(p.status_info_raw(), expected.status_info);
        prop_assert_eq!(p.msg_type(), expected.msg_type);
        prop_assert_eq!(p.msg_sub_type(), expected.msg_sub_type);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_rxm_cor_frames((expected, frame) in ubx_rxm_cor_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::RxmCor(p)))) = it.next() else {
            panic!("Parser failed to parse a RXM-COR valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.ebno_raw(), expected.ebno);
        prop_assert_eq!(p.status_info_raw(), expected.status_info);
        prop_assert_eq!(p.msg_type(), expected.msg_type);
        prop_assert_eq!(p.msg_sub_type(), expected.msg_sub_type);
    }
}
