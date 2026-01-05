#![cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]

//! A proptest generator for U-Blox MON-HW2 messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-HW2 message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{
    constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2},
    ParserBuilder, UbxPacket,
};

/// Represents the payload of a UBX-MON-HW2 message.
///
/// The fields are ordered as they appear in the U-Blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
///
/// MON-HW2 payload is 28 bytes.
#[derive(Debug, Clone)]
pub struct MonHw2 {
    pub ofs_i: i8,
    pub mag_i: u8,
    pub ofs_q: i8,
    pub mag_q: u8,
    pub cfg_source: u8,
    pub reserved0: [u8; 3],
    pub low_lev_cfg: u32,
    pub reserved1: [u8; 8],
    pub post_status: u32,
    pub reserved2: [u8; 4],
}

impl MonHw2 {
    /// Serializes the MonHw2 payload into a vector of 28 bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(28);
        wtr.write_i8(self.ofs_i).unwrap();
        wtr.write_u8(self.mag_i).unwrap();
        wtr.write_i8(self.ofs_q).unwrap();
        wtr.write_u8(self.mag_q).unwrap();
        wtr.write_u8(self.cfg_source).unwrap();
        wtr.extend_from_slice(&self.reserved0);
        wtr.write_u32::<LittleEndian>(self.low_lev_cfg).unwrap();
        wtr.extend_from_slice(&self.reserved1);
        wtr.write_u32::<LittleEndian>(self.post_status).unwrap();
        wtr.extend_from_slice(&self.reserved2);
        wtr
    }
}

/// A proptest strategy for generating a `MonHw2` payload struct.
pub fn mon_hw2_payload_strategy() -> impl Strategy<Value = MonHw2> {
    (
        any::<i8>(), // ofs_i
        any::<u8>(), // mag_i
        any::<i8>(), // ofs_q
        any::<u8>(), // mag_q
        prop_oneof![
            // cfg_source must be one of the specified values
            Just(114u8), // ROM
            Just(111u8), // OTP
            Just(112u8), // config pins
            Just(102u8), // flash image
        ],
        Just([0u8; 3]), // reserved0
        any::<u32>(),   // low_lev_cfg
        Just([0u8; 8]), // reserved1
        any::<u32>(),   // post_status
        Just([0u8; 4]), // reserved2
    )
        .prop_map(
            |(
                ofs_i,
                mag_i,
                ofs_q,
                mag_q,
                cfg_source,
                reserved0,
                low_lev_cfg,
                reserved1,
                post_status,
                reserved2,
            )| {
                MonHw2 {
                    ofs_i,
                    mag_i,
                    ofs_q,
                    mag_q,
                    cfg_source,
                    reserved0,
                    low_lev_cfg,
                    reserved1,
                    post_status,
                    reserved2,
                }
            },
        )
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
/// containing a MON-HW2 message, along with the source `MonHw2` struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(MonHw2, Vec<u8>)`.
pub fn ubx_mon_hw2_frame_strategy() -> impl Strategy<Value = (MonHw2, Vec<u8>)> {
    mon_hw2_payload_strategy().prop_map(|mon_hw2| {
        let payload = mon_hw2.to_bytes();
        let class_id = 0x0A; // MON class
        let message_id = 0x0B; // HW2 message ID
        let length = payload.len() as u16;

        // Start building the frame to be checksummed
        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        // Assemble the final frame
        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(UBX_SYNC_CHAR_1);
        final_frame.push(UBX_SYNC_CHAR_2);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (mon_hw2, final_frame)
    })
}

#[cfg(feature = "ubx_proto23")]
proptest! {
    #[test]
    fn test_parser_proto23_with_generated_mon_hw2_frames(
        (expected_hw2, frame) in ubx_mon_hw2_frame_strategy()
    ) {
        use ublox::proto23::{Proto23, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto23>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto23(PacketRef::MonHw2(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW2 valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.ofs_i(), expected_hw2.ofs_i);
        prop_assert_eq!(p.mag_i(), expected_hw2.mag_i);
        prop_assert_eq!(p.ofs_q(), expected_hw2.ofs_q);
        prop_assert_eq!(p.mag_q(), expected_hw2.mag_q);
        prop_assert_eq!(p.cfg_source_raw(), expected_hw2.cfg_source);
        prop_assert_eq!(p.low_lev_cfg(), expected_hw2.low_lev_cfg);
        prop_assert_eq!(p.post_status(), expected_hw2.post_status);
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_hw2_frames(
        (expected_hw2, frame) in ubx_mon_hw2_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonHw2(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW2 valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.ofs_i(), expected_hw2.ofs_i);
        prop_assert_eq!(p.mag_i(), expected_hw2.mag_i);
        prop_assert_eq!(p.ofs_q(), expected_hw2.ofs_q);
        prop_assert_eq!(p.mag_q(), expected_hw2.mag_q);
        prop_assert_eq!(p.cfg_source_raw(), expected_hw2.cfg_source);
        prop_assert_eq!(p.low_lev_cfg(), expected_hw2.low_lev_cfg);
        prop_assert_eq!(p.post_status(), expected_hw2.post_status);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_mon_hw2_frames(
        (expected_hw2, frame) in ubx_mon_hw2_frame_strategy()
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::MonHw2(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW2 valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.ofs_i(), expected_hw2.ofs_i);
        prop_assert_eq!(p.mag_i(), expected_hw2.mag_i);
        prop_assert_eq!(p.ofs_q(), expected_hw2.ofs_q);
        prop_assert_eq!(p.mag_q(), expected_hw2.mag_q);
        prop_assert_eq!(p.cfg_source_raw(), expected_hw2.cfg_source);
        prop_assert_eq!(p.low_lev_cfg(), expected_hw2.low_lev_cfg);
        prop_assert_eq!(p.post_status(), expected_hw2.post_status);
    }
}

#[cfg(feature = "ubx_proto33")]
proptest! {
    #[test]
    fn test_parser_proto33_with_generated_mon_hw2_frames(
        (expected_hw2, frame) in ubx_mon_hw2_frame_strategy()
    ) {
        use ublox::proto33::{Proto33, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto33>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto33(PacketRef::MonHw2(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW2 valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.ofs_i(), expected_hw2.ofs_i);
        prop_assert_eq!(p.mag_i(), expected_hw2.mag_i);
        prop_assert_eq!(p.ofs_q(), expected_hw2.ofs_q);
        prop_assert_eq!(p.mag_q(), expected_hw2.mag_q);
        prop_assert_eq!(p.cfg_source_raw(), expected_hw2.cfg_source);
        prop_assert_eq!(p.low_lev_cfg(), expected_hw2.low_lev_cfg);
        prop_assert_eq!(p.post_status(), expected_hw2.post_status);
    }
}
