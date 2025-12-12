#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox NAV-POSECEF messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a NAV-POSECEF message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents the payload of a UBX-NAV-POSECEF message.
///
/// The fields are ordered as they appear in the u-blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
///
/// NAV-POSECEF payload is 20 bytes.
#[derive(Debug, Clone)]
pub struct NavPosEcefPayload {
    pub itow: u32,      // GPS time of week of the navigation epoch [ms]
    pub ecef_x: i32,    // ECEF X coordinate [cm]
    pub ecef_y: i32,    // ECEF Y coordinate [cm]
    pub ecef_z: i32,    // ECEF Z coordinate [cm]
    pub p_acc: u32,     // Position accuracy estimate [cm]
}

impl NavPosEcefPayload {
    /// Serializes the NavPosEcefPayload into a 20-byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(20);
        wtr.write_u32::<LittleEndian>(self.itow).unwrap();
        wtr.write_i32::<LittleEndian>(self.ecef_x).unwrap();
        wtr.write_i32::<LittleEndian>(self.ecef_y).unwrap();
        wtr.write_i32::<LittleEndian>(self.ecef_z).unwrap();
        wtr.write_u32::<LittleEndian>(self.p_acc).unwrap();
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

/// A proptest strategy for generating a `NavPosEcefPayload` struct.
fn nav_pos_ecef_payload_strategy() -> impl Strategy<Value = NavPosEcefPayload> {
    (
        any::<u32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<u32>(),
    )
        .prop_map(|(itow, ecef_x, ecef_y, ecef_z, p_acc)| NavPosEcefPayload {
            itow,
            ecef_x,
            ecef_y,
            ecef_z,
            p_acc,
        })
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a NAV-POSECEF message, along with the source payload struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(NavPosEcefPayload, Vec<u8>)`.
pub fn ubx_nav_pos_ecef_frame_strategy() -> impl Strategy<Value = (NavPosEcefPayload, Vec<u8>)> {
    nav_pos_ecef_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x01;
        let message_id = 0x01;
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
    fn test_parser_proto27_with_generated_nav_pos_ecef_frames((expected, frame) in ubx_nav_pos_ecef_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::NavPosEcef(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-POSECEF valid packet");
        };

        prop_assert_eq!(p.itow(), expected.itow);
        prop_assert_eq!(p.ecef_x_meters_raw(), expected.ecef_x);
        prop_assert_eq!(p.ecef_y_meters_raw(), expected.ecef_y);
        prop_assert_eq!(p.ecef_z_meters_raw(), expected.ecef_z);
        prop_assert_eq!(p.p_acc_meters_raw(), expected.p_acc);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_nav_pos_ecef_frames((expected, frame) in ubx_nav_pos_ecef_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::NavPosEcef(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-POSECEF valid packet");
        };

        prop_assert_eq!(p.itow(), expected.itow);
        prop_assert_eq!(p.ecef_x_meters_raw(), expected.ecef_x);
        prop_assert_eq!(p.ecef_y_meters_raw(), expected.ecef_y);
        prop_assert_eq!(p.ecef_z_meters_raw(), expected.ecef_z);
        prop_assert_eq!(p.p_acc_meters_raw(), expected.p_acc);
    }
}
