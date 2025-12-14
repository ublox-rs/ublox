#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox NAV-COV messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a NAV-COV message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

const SYNC_CHAR_1: u8 = 0xB5;
const SYNC_CHAR_2: u8 = 0x62;

/// Represents the payload of a UBX-NAV-COV message.
///
/// The fields are ordered as they appear in the u-blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
///
/// NAV-COV payload is 64 bytes.
#[derive(Debug, Clone)]
pub struct NavCovPayload {
    pub itow: u32,          // GPS time of week of the navigation epoch [ms]
    pub version: u8,        // Message version (0x00 for this version)
    pub pos_cov_valid: u8,  // Position covariance matrix validity flag (0 = invalid, 1 = valid)
    pub vel_cov_valid: u8,  // Velocity covariance matrix validity flag (0 = invalid, 1 = valid)
    pub reserved0: [u8; 9], // Reserved
    pub pos_cov_nn: f32,    // Position covariance matrix value p_NN [m^2]
    pub pos_cov_ne: f32,    // Position covariance matrix value p_NE [m^2]
    pub pos_cov_nd: f32,    // Position covariance matrix value p_ND [m^2]
    pub pos_cov_ee: f32,    // Position covariance matrix value p_EE [m^2]
    pub pos_cov_ed: f32,    // Position covariance matrix value p_ED [m^2]
    pub pos_cov_dd: f32,    // Position covariance matrix value p_DD [m^2]
    pub vel_cov_nn: f32,    // Velocity covariance matrix value v_NN [m^2/s^2]
    pub vel_cov_ne: f32,    // Velocity covariance matrix value v_NE [m^2/s^2]
    pub vel_cov_nd: f32,    // Velocity covariance matrix value v_ND [m^2/s^2]
    pub vel_cov_ee: f32,    // Velocity covariance matrix value v_EE [m^2/s^2]
    pub vel_cov_ed: f32,    // Velocity covariance matrix value v_ED [m^2/s^2]
    pub vel_cov_dd: f32,    // Velocity covariance matrix value v_DD [m^2/s^2]
}

impl NavCovPayload {
    /// Serializes the NavCovPayload into a 64-byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(64);
        wtr.write_u32::<LittleEndian>(self.itow).unwrap();
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.pos_cov_valid).unwrap();
        wtr.write_u8(self.vel_cov_valid).unwrap();
        wtr.extend_from_slice(&self.reserved0);
        wtr.write_f32::<LittleEndian>(self.pos_cov_nn).unwrap();
        wtr.write_f32::<LittleEndian>(self.pos_cov_ne).unwrap();
        wtr.write_f32::<LittleEndian>(self.pos_cov_nd).unwrap();
        wtr.write_f32::<LittleEndian>(self.pos_cov_ee).unwrap();
        wtr.write_f32::<LittleEndian>(self.pos_cov_ed).unwrap();
        wtr.write_f32::<LittleEndian>(self.pos_cov_dd).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_nn).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_ne).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_nd).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_ee).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_ed).unwrap();
        wtr.write_f32::<LittleEndian>(self.vel_cov_dd).unwrap();
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

/// Generates finite `f32` values (excludes NaN and +/-Inf).
fn finite_f32() -> impl Strategy<Value = f32> {
    any::<u32>().prop_filter_map("finite f32", |bits| {
        let f = f32::from_bits(bits);
        if f.is_finite() {
            Some(f)
        } else {
            None
        }
    })
}

/// A proptest strategy for generating a `NavCovPayload` struct.
fn nav_cov_payload_strategy() -> impl Strategy<Value = NavCovPayload> {
    // Split into smaller tuples to avoid proptest tuple size limits
    let header = (
        any::<u32>(),
        Just(0u8),
        any::<u8>(),
        any::<u8>(),
        Just([0u8; 9]),
    );

    let pos_cov = (
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
    );

    let vel_cov = (
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
    );

    (header, pos_cov, vel_cov).prop_map(
        |(
            (itow, version, pos_cov_valid, vel_cov_valid, reserved0),
            (pos_cov_nn, pos_cov_ne, pos_cov_nd, pos_cov_ee, pos_cov_ed, pos_cov_dd),
            (vel_cov_nn, vel_cov_ne, vel_cov_nd, vel_cov_ee, vel_cov_ed, vel_cov_dd),
        )| {
            NavCovPayload {
                itow,
                version,
                pos_cov_valid,
                vel_cov_valid,
                reserved0,
                pos_cov_nn,
                pos_cov_ne,
                pos_cov_nd,
                pos_cov_ee,
                pos_cov_ed,
                pos_cov_dd,
                vel_cov_nn,
                vel_cov_ne,
                vel_cov_nd,
                vel_cov_ee,
                vel_cov_ed,
                vel_cov_dd,
            }
        },
    )
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a NAV-COV message, along with the source payload struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(NavCovPayload, Vec<u8>)`.
pub fn ubx_nav_cov_frame_strategy() -> impl Strategy<Value = (NavCovPayload, Vec<u8>)> {
    nav_cov_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x01;
        let message_id = 0x36;
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

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_nav_cov_frames((expected, frame) in ubx_nav_cov_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::NavCov(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-COV valid packet");
        };

        prop_assert_eq!(p.itow(), expected.itow);
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.pos_cov_valid(), expected.pos_cov_valid);
        prop_assert_eq!(p.vel_cov_valid(), expected.vel_cov_valid);

        prop_assert_eq!(p.pos_cov_nn().to_bits(), expected.pos_cov_nn.to_bits());
        prop_assert_eq!(p.pos_cov_ne().to_bits(), expected.pos_cov_ne.to_bits());
        prop_assert_eq!(p.pos_cov_nd().to_bits(), expected.pos_cov_nd.to_bits());
        prop_assert_eq!(p.pos_cov_ee().to_bits(), expected.pos_cov_ee.to_bits());
        prop_assert_eq!(p.pos_cov_ed().to_bits(), expected.pos_cov_ed.to_bits());
        prop_assert_eq!(p.pos_cov_dd().to_bits(), expected.pos_cov_dd.to_bits());

        prop_assert_eq!(p.vel_cov_nn().to_bits(), expected.vel_cov_nn.to_bits());
        prop_assert_eq!(p.vel_cov_ne().to_bits(), expected.vel_cov_ne.to_bits());
        prop_assert_eq!(p.vel_cov_nd().to_bits(), expected.vel_cov_nd.to_bits());
        prop_assert_eq!(p.vel_cov_ee().to_bits(), expected.vel_cov_ee.to_bits());
        prop_assert_eq!(p.vel_cov_ed().to_bits(), expected.vel_cov_ed.to_bits());
        prop_assert_eq!(p.vel_cov_dd().to_bits(), expected.vel_cov_dd.to_bits());
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_nav_cov_frames((expected, frame) in ubx_nav_cov_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::NavCov(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-COV valid packet");
        };

        prop_assert_eq!(p.itow(), expected.itow);
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.pos_cov_valid(), expected.pos_cov_valid);
        prop_assert_eq!(p.vel_cov_valid(), expected.vel_cov_valid);

        prop_assert_eq!(p.pos_cov_nn().to_bits(), expected.pos_cov_nn.to_bits());
        prop_assert_eq!(p.pos_cov_ne().to_bits(), expected.pos_cov_ne.to_bits());
        prop_assert_eq!(p.pos_cov_nd().to_bits(), expected.pos_cov_nd.to_bits());
        prop_assert_eq!(p.pos_cov_ee().to_bits(), expected.pos_cov_ee.to_bits());
        prop_assert_eq!(p.pos_cov_ed().to_bits(), expected.pos_cov_ed.to_bits());
        prop_assert_eq!(p.pos_cov_dd().to_bits(), expected.pos_cov_dd.to_bits());

        prop_assert_eq!(p.vel_cov_nn().to_bits(), expected.vel_cov_nn.to_bits());
        prop_assert_eq!(p.vel_cov_ne().to_bits(), expected.vel_cov_ne.to_bits());
        prop_assert_eq!(p.vel_cov_nd().to_bits(), expected.vel_cov_nd.to_bits());
        prop_assert_eq!(p.vel_cov_ee().to_bits(), expected.vel_cov_ee.to_bits());
        prop_assert_eq!(p.vel_cov_ed().to_bits(), expected.vel_cov_ed.to_bits());
        prop_assert_eq!(p.vel_cov_dd().to_bits(), expected.vel_cov_dd.to_bits());
    }
}
