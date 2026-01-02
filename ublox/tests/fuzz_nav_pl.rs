#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox NAV-PL messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a NAV-PL message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

const SYNC_CHAR_1: u8 = 0xB5;
const SYNC_CHAR_2: u8 = 0x62;

/// Represents the payload of a UBX-NAV-PL message.
///
/// NAV-PL payload is 52 bytes.
#[derive(Debug, Clone)]
pub struct NavPlPayload {
    pub version: u8,                   // Message version (0x01 for this version)
    pub tmir_coeff: u8,                // TMIR coefficient
    pub tmir_exp: i8,                  // TMIR exponent
    pub pl_pos_valid: u8,              // Position PL validity
    pub pl_pos_frame: u8,              // Position PL frame
    pub pl_vel_valid: u8,              // Velocity PL validity
    pub pl_vel_frame: u8,              // Velocity PL frame
    pub pl_time_valid: u8,             // Time PL validity
    pub pl_pos_invalidity_reason: u8,  // Position invalidity reason
    pub pl_vel_invalidity_reason: u8,  // Velocity invalidity reason
    pub pl_time_invalidity_reason: u8, // Time invalidity reason
    pub reserved0: u8,                 // Reserved
    pub itow: u32,                     // GPS time of week [ms]
    pub pl_pos1: u32,                  // Position PL axis 1 [mm]
    pub pl_pos2: u32,                  // Position PL axis 2 [mm]
    pub pl_pos3: u32,                  // Position PL axis 3 [mm]
    pub pl_vel1: u32,                  // Velocity PL axis 1 [mm/s]
    pub pl_vel2: u32,                  // Velocity PL axis 2 [mm/s]
    pub pl_vel3: u32,                  // Velocity PL axis 3 [mm/s]
    pub pl_pos_horiz_orient: u16,      // Position ellipse orientation [1e-2 deg]
    pub pl_vel_horiz_orient: u16,      // Velocity ellipse orientation [1e-2 deg]
    pub pl_time: u32,                  // Time PL [ns]
    pub reserved1: [u8; 4],            // Reserved
}

impl NavPlPayload {
    /// Serializes the NavPlPayload into a 52-byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(52);
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.tmir_coeff).unwrap();
        wtr.write_i8(self.tmir_exp).unwrap();
        wtr.write_u8(self.pl_pos_valid).unwrap();
        wtr.write_u8(self.pl_pos_frame).unwrap();
        wtr.write_u8(self.pl_vel_valid).unwrap();
        wtr.write_u8(self.pl_vel_frame).unwrap();
        wtr.write_u8(self.pl_time_valid).unwrap();
        wtr.write_u8(self.pl_pos_invalidity_reason).unwrap();
        wtr.write_u8(self.pl_vel_invalidity_reason).unwrap();
        wtr.write_u8(self.pl_time_invalidity_reason).unwrap();
        wtr.write_u8(self.reserved0).unwrap();
        wtr.write_u32::<LittleEndian>(self.itow).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos1).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos2).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos3).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel1).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel2).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel3).unwrap();
        wtr.write_u16::<LittleEndian>(self.pl_pos_horiz_orient)
            .unwrap();
        wtr.write_u16::<LittleEndian>(self.pl_vel_horiz_orient)
            .unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_time).unwrap();
        wtr.extend_from_slice(&self.reserved1);
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

/// A proptest strategy for generating a `NavPlPayload` struct.
fn nav_pl_payload_strategy() -> impl Strategy<Value = NavPlPayload> {
    // Split into smaller tuples to avoid proptest tuple size limits
    let header = (
        Just(0x01u8), // version
        any::<u8>(),  // tmir_coeff
        any::<i8>(),  // tmir_exp
        0u8..=1u8,    // pl_pos_valid
        0u8..=3u8,    // pl_pos_frame
        0u8..=1u8,    // pl_vel_valid
        0u8..=3u8,    // pl_vel_frame
        0u8..=1u8,    // pl_time_valid
    );

    let invalidity = (
        any::<u8>(), // pl_pos_invalidity_reason
        any::<u8>(), // pl_vel_invalidity_reason
        any::<u8>(), // pl_time_invalidity_reason
        Just(0u8),   // reserved0
    );

    let timing_and_pos = (
        any::<u32>(), // itow
        any::<u32>(), // pl_pos1
        any::<u32>(), // pl_pos2
        any::<u32>(), // pl_pos3
    );

    let velocity = (
        any::<u32>(), // pl_vel1
        any::<u32>(), // pl_vel2
        any::<u32>(), // pl_vel3
    );

    let orient_and_time = (
        any::<u16>(),   // pl_pos_horiz_orient
        any::<u16>(),   // pl_vel_horiz_orient
        any::<u32>(),   // pl_time
        Just([0u8; 4]), // reserved1
    );

    (
        header,
        invalidity,
        timing_and_pos,
        velocity,
        orient_and_time,
    )
        .prop_map(
            |(
                (
                    version,
                    tmir_coeff,
                    tmir_exp,
                    pl_pos_valid,
                    pl_pos_frame,
                    pl_vel_valid,
                    pl_vel_frame,
                    pl_time_valid,
                ),
                (
                    pl_pos_invalidity_reason,
                    pl_vel_invalidity_reason,
                    pl_time_invalidity_reason,
                    reserved0,
                ),
                (itow, pl_pos1, pl_pos2, pl_pos3),
                (pl_vel1, pl_vel2, pl_vel3),
                (pl_pos_horiz_orient, pl_vel_horiz_orient, pl_time, reserved1),
            )| {
                NavPlPayload {
                    version,
                    tmir_coeff,
                    tmir_exp,
                    pl_pos_valid,
                    pl_pos_frame,
                    pl_vel_valid,
                    pl_vel_frame,
                    pl_time_valid,
                    pl_pos_invalidity_reason,
                    pl_vel_invalidity_reason,
                    pl_time_invalidity_reason,
                    reserved0,
                    itow,
                    pl_pos1,
                    pl_pos2,
                    pl_pos3,
                    pl_vel1,
                    pl_vel2,
                    pl_vel3,
                    pl_pos_horiz_orient,
                    pl_vel_horiz_orient,
                    pl_time,
                    reserved1,
                }
            },
        )
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a NAV-PL message, along with the source payload struct.
pub fn ubx_nav_pl_frame_strategy() -> impl Strategy<Value = (NavPlPayload, Vec<u8>)> {
    nav_pl_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x01;
        let message_id = 0x62;
        let length = payload.len() as u16;

        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(SYNC_CHAR_1);
        final_frame.push(SYNC_CHAR_2);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (payload_struct, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_nav_pl_frames((expected, frame) in ubx_nav_pl_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::NavPl(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PL valid packet");
        };

        // Header fields
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.tmir_coeff(), expected.tmir_coeff);
        prop_assert_eq!(p.tmir_exp(), expected.tmir_exp);

        // Validity and frame fields (raw values)
        prop_assert_eq!(p.pl_pos_valid_raw(), expected.pl_pos_valid);
        prop_assert_eq!(p.pl_pos_frame_raw(), expected.pl_pos_frame);
        prop_assert_eq!(p.pl_vel_valid_raw(), expected.pl_vel_valid);
        prop_assert_eq!(p.pl_vel_frame_raw(), expected.pl_vel_frame);
        prop_assert_eq!(p.pl_time_valid_raw(), expected.pl_time_valid);

        // Invalidity reason fields
        prop_assert_eq!(p.pl_pos_invalidity_reason_raw(), expected.pl_pos_invalidity_reason);
        prop_assert_eq!(p.pl_vel_invalidity_reason_raw(), expected.pl_vel_invalidity_reason);
        prop_assert_eq!(p.pl_time_invalidity_reason_raw(), expected.pl_time_invalidity_reason);

        // Timing and position fields
        prop_assert_eq!(p.itow(), expected.itow);
        // Position PLs now return f64 in meters (scaled by 0.001 from mm)
        prop_assert!((p.pl_pos1() - (expected.pl_pos1 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_pos2() - (expected.pl_pos2 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_pos3() - (expected.pl_pos3 as f64 * 0.001)).abs() < 1e-9);

        // Velocity PLs now return f64 in m/s (scaled by 0.001 from mm/s)
        prop_assert!((p.pl_vel1() - (expected.pl_vel1 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_vel2() - (expected.pl_vel2 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_vel3() - (expected.pl_vel3 as f64 * 0.001)).abs() < 1e-9);

        // Orientation and time fields
        prop_assert_eq!(p.pl_pos_horiz_orient_raw(), expected.pl_pos_horiz_orient);
        prop_assert_eq!(p.pl_vel_horiz_orient_raw(), expected.pl_vel_horiz_orient);
        prop_assert_eq!(p.pl_time(), expected.pl_time);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_nav_pl_frames((expected, frame) in ubx_nav_pl_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::NavPl(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PL valid packet");
        };

        // Header fields
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.tmir_coeff(), expected.tmir_coeff);
        prop_assert_eq!(p.tmir_exp(), expected.tmir_exp);

        // Validity and frame fields (raw values)
        prop_assert_eq!(p.pl_pos_valid_raw(), expected.pl_pos_valid);
        prop_assert_eq!(p.pl_pos_frame_raw(), expected.pl_pos_frame);
        prop_assert_eq!(p.pl_vel_valid_raw(), expected.pl_vel_valid);
        prop_assert_eq!(p.pl_vel_frame_raw(), expected.pl_vel_frame);
        prop_assert_eq!(p.pl_time_valid_raw(), expected.pl_time_valid);

        // Invalidity reason fields
        prop_assert_eq!(p.pl_pos_invalidity_reason_raw(), expected.pl_pos_invalidity_reason);
        prop_assert_eq!(p.pl_vel_invalidity_reason_raw(), expected.pl_vel_invalidity_reason);
        prop_assert_eq!(p.pl_time_invalidity_reason_raw(), expected.pl_time_invalidity_reason);

        // Timing and position fields
        prop_assert_eq!(p.itow(), expected.itow);
        // Position PLs now return f64 in meters (scaled by 0.001 from mm)
        prop_assert!((p.pl_pos1() - (expected.pl_pos1 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_pos2() - (expected.pl_pos2 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_pos3() - (expected.pl_pos3 as f64 * 0.001)).abs() < 1e-9);

        // Velocity PLs now return f64 in m/s (scaled by 0.001 from mm/s)
        prop_assert!((p.pl_vel1() - (expected.pl_vel1 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_vel2() - (expected.pl_vel2 as f64 * 0.001)).abs() < 1e-9);
        prop_assert!((p.pl_vel3() - (expected.pl_vel3 as f64 * 0.001)).abs() < 1e-9);

        // Orientation and time fields
        prop_assert_eq!(p.pl_pos_horiz_orient_raw(), expected.pl_pos_horiz_orient);
        prop_assert_eq!(p.pl_vel_horiz_orient_raw(), expected.pl_vel_horiz_orient);
        prop_assert_eq!(p.pl_time(), expected.pl_time);
    }
}
