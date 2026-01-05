//! A proptest generator for U-Blox NAV-HPPOSLLH messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a NAV-HPPOSLLH message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{
    constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2},
    ParserBuilder, UbxPacket,
};

#[allow(dead_code, reason = "dead variants for SOME feature flag combination")]
#[derive(Debug, Clone, Copy)]
pub enum ProtocolVersion {
    V14,
    V23,
    V27,
    V31,
    V33,
}

/// Represents the payload of a UBX-NAV-HPPOSLLH message.
///
/// The fields are ordered as they appear in the U-Blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
///
/// NAV-HPPOSLLH payload is 36 bytes for all protocol versions.
#[derive(Debug, Clone)]
pub struct NavHpPosLlh {
    pub version: u8,    // Message version (0x00 for current version)
    pub reserved1: u16, // Reserved
    pub flags: u8,      // flags
    pub itow: u32,      // GPS time of week [ms]
    pub lon: i32,       // Longitude [1e-7 deg]
    pub lat: i32,       // Latitude [1e-7 deg]
    pub height: i32,    // Height above ellipsoid [mm]
    pub h_msl: i32,     // Height above MSL [mm]
    pub lon_hp: i8,     // High precision component of longitude [1e-9 deg]
    pub lat_hp: i8,     // High precision component of latitude [1e-9 deg]
    pub height_hp: i8,  // High precision component of height above ellipsoid [0.1 mm]
    pub h_msl_hp: i8,   // High precision component of height above MSL [0.1 mm]
    pub h_acc: u32,     // Horizontal accuracy estimate [0.1 mm]
    pub v_acc: u32,     // Vertical accuracy estimate [0.1 mm]
}

impl NavHpPosLlh {
    /// Serializes the NavHpPosLlh payload into a vector.
    /// Always 36 bytes for all protocol versions.
    pub fn to_bytes(&self, _version: ProtocolVersion) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(36);
        wtr.write_u8(self.version).unwrap();
        wtr.write_u16::<LittleEndian>(self.reserved1).unwrap();
        wtr.write_u8(self.flags).unwrap();
        wtr.write_u32::<LittleEndian>(self.itow).unwrap();
        wtr.write_i32::<LittleEndian>(self.lon).unwrap();
        wtr.write_i32::<LittleEndian>(self.lat).unwrap();
        wtr.write_i32::<LittleEndian>(self.height).unwrap();
        wtr.write_i32::<LittleEndian>(self.h_msl).unwrap();
        wtr.write_i8(self.lon_hp).unwrap();
        wtr.write_i8(self.lat_hp).unwrap();
        wtr.write_i8(self.height_hp).unwrap();
        wtr.write_i8(self.h_msl_hp).unwrap();
        wtr.write_u32::<LittleEndian>(self.h_acc).unwrap();
        wtr.write_u32::<LittleEndian>(self.v_acc).unwrap();
        wtr
    }
}

/// A proptest strategy for generating a `NavHpPosLlh` payload struct.
///
/// This strategy is parameterized by the protocol version but the payload
/// format is the same across all versions for NAV-HPPOSLLH.
pub fn nav_hpposllh_payload_strategy(
    _version: ProtocolVersion,
) -> impl Strategy<Value = NavHpPosLlh> {
    // Split into smaller tuples to avoid proptest tuple size limits
    let header_and_time = (
        Just(0u8),    // version (always 0x00)
        Just(0u16),   // reserved1
        any::<u8>(),  // flags, only bit 1 is used
        any::<u32>(), // itow
    );

    let position_data = (
        (-1800000000..=1800000000i32), // lon (longitude in 1e-7 degrees)
        (-900000000..=900000000i32),   // lat (latitude in 1e-7 degrees)
        any::<i32>(),                  // height (height above ellipsoid in mm)
        any::<i32>(),                  // h_msl (height above MSL in mm)
    );

    let high_precision_data = (
        (-99..=99i8), // lon_hp (high precision longitude component)
        (-99..=99i8), // lat_hp (high precision latitude component)
        (-9..=9i8),   // height_hp (high precision height component)
        (-9..=9i8),   // h_msl_hp (high precision MSL height component)
    );

    let accuracy_data = (
        any::<u32>(), // h_acc (horizontal accuracy in 0.1 mm)
        any::<u32>(), // v_acc (vertical accuracy in 0.1 mm)
    );

    (
        header_and_time,
        position_data,
        high_precision_data,
        accuracy_data,
    )
        .prop_map(
            |(
                (version, reserved1, flags, itow),
                (lon, lat, height, h_msl),
                (lon_hp, lat_hp, height_hp, h_msl_hp),
                (h_acc, v_acc),
            )| {
                NavHpPosLlh {
                    version,
                    reserved1,
                    flags,
                    itow,
                    lon,
                    lat,
                    height,
                    h_msl,
                    lon_hp,
                    lat_hp,
                    height_hp,
                    h_msl_hp,
                    h_acc,
                    v_acc,
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
/// containing a NAV-HPPOSLLH message, along with the source `NavHpPosLlh` struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(NavHpPosLlh, Vec<u8>)`.
pub fn ubx_nav_hpposllh_frame_strategy(
    version: ProtocolVersion,
) -> impl Strategy<Value = (NavHpPosLlh, Vec<u8>)> {
    nav_hpposllh_payload_strategy(version).prop_map(move |nav_hpposllh| {
        let payload = nav_hpposllh.to_bytes(version);
        let class_id = 0x01; // NAV class
        let message_id = 0x14; // HPPOSLLH message ID
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

        (nav_hpposllh, final_frame)
    })
}

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_nav_hpposllh_frames(
        (expected_hpposllh, frame) in ubx_nav_hpposllh_frame_strategy(ProtocolVersion::V14)
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::NavHpPosLlh(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-HPPOSLLH valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.version(), expected_hpposllh.version);
        prop_assert_eq!(p.itow(), expected_hpposllh.itow);
        prop_assert_eq!(p.flags_raw(), expected_hpposllh.flags);
        prop_assert_eq!(p.lon_degrees_raw(), expected_hpposllh.lon);
        prop_assert_eq!(p.lat_degrees_raw(), expected_hpposllh.lat);
        prop_assert_eq!(p.height_meters_raw(), expected_hpposllh.height);
        prop_assert_eq!(p.height_msl_raw(), expected_hpposllh.h_msl);
        prop_assert_eq!(p.lon_hp_degrees_raw(), expected_hpposllh.lon_hp);
        prop_assert_eq!(p.lat_hp_degrees_raw(), expected_hpposllh.lat_hp);
        prop_assert_eq!(p.height_hp_meters_raw(), expected_hpposllh.height_hp);
        prop_assert_eq!(p.height_hp_msl_raw(), expected_hpposllh.h_msl_hp);
        prop_assert_eq!(p.horizontal_accuracy_raw(), expected_hpposllh.h_acc);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_hpposllh.v_acc);
    }
}

#[cfg(feature = "ubx_proto23")]
proptest! {
    #[test]
    fn test_parser_proto23_with_generated_nav_hpposllh_frames(
        (expected_hpposllh, frame) in ubx_nav_hpposllh_frame_strategy(ProtocolVersion::V23)
    ) {
        use ublox::proto23::{Proto23, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto23>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto23(PacketRef::NavHpPosLlh(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-HPPOSLLH valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.version(), expected_hpposllh.version);
        prop_assert_eq!(p.itow(), expected_hpposllh.itow);
        prop_assert_eq!(p.flags_raw(), expected_hpposllh.flags);
        prop_assert_eq!(p.lon_degrees_raw(), expected_hpposllh.lon);
        prop_assert_eq!(p.lat_degrees_raw(), expected_hpposllh.lat);
        prop_assert_eq!(p.height_meters_raw(), expected_hpposllh.height);
        prop_assert_eq!(p.height_msl_raw(), expected_hpposllh.h_msl);
        prop_assert_eq!(p.lon_hp_degrees_raw(), expected_hpposllh.lon_hp);
        prop_assert_eq!(p.lat_hp_degrees_raw(), expected_hpposllh.lat_hp);
        prop_assert_eq!(p.height_hp_meters_raw(), expected_hpposllh.height_hp);
        prop_assert_eq!(p.height_hp_msl_raw(), expected_hpposllh.h_msl_hp);
        prop_assert_eq!(p.horizontal_accuracy_raw(), expected_hpposllh.h_acc);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_hpposllh.v_acc);
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_nav_hpposllh_frames(
        (expected_hpposllh, frame) in ubx_nav_hpposllh_frame_strategy(ProtocolVersion::V27)
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::NavHpPosLlh(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-HPPOSLLH valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.version(), expected_hpposllh.version);
        prop_assert_eq!(p.itow(), expected_hpposllh.itow);
        prop_assert_eq!(p.flags_raw(), expected_hpposllh.flags);
        prop_assert_eq!(p.lon_degrees_raw(), expected_hpposllh.lon);
        prop_assert_eq!(p.lat_degrees_raw(), expected_hpposllh.lat);
        prop_assert_eq!(p.height_meters_raw(), expected_hpposllh.height);
        prop_assert_eq!(p.height_msl_raw(), expected_hpposllh.h_msl);
        prop_assert_eq!(p.lon_hp_degrees_raw(), expected_hpposllh.lon_hp);
        prop_assert_eq!(p.lat_hp_degrees_raw(), expected_hpposllh.lat_hp);
        prop_assert_eq!(p.height_hp_meters_raw(), expected_hpposllh.height_hp);
        prop_assert_eq!(p.height_hp_msl_raw(), expected_hpposllh.h_msl_hp);
        prop_assert_eq!(p.horizontal_accuracy_raw(), expected_hpposllh.h_acc);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_hpposllh.v_acc);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_nav_hpposllh_frames(
        (expected_hpposllh, frame) in ubx_nav_hpposllh_frame_strategy(ProtocolVersion::V31)
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::NavHpPosLlh(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-HPPOSLLH valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.version(), expected_hpposllh.version);
        prop_assert_eq!(p.itow(), expected_hpposllh.itow);
        prop_assert_eq!(p.flags_raw(), expected_hpposllh.flags);
        prop_assert_eq!(p.lon_degrees_raw(), expected_hpposllh.lon);
        prop_assert_eq!(p.lat_degrees_raw(), expected_hpposllh.lat);
        prop_assert_eq!(p.height_meters_raw(), expected_hpposllh.height);
        prop_assert_eq!(p.height_msl_raw(), expected_hpposllh.h_msl);
        prop_assert_eq!(p.lon_hp_degrees_raw(), expected_hpposllh.lon_hp);
        prop_assert_eq!(p.lat_hp_degrees_raw(), expected_hpposllh.lat_hp);
        prop_assert_eq!(p.height_hp_meters_raw(), expected_hpposllh.height_hp);
        prop_assert_eq!(p.height_hp_msl_raw(), expected_hpposllh.h_msl_hp);
        prop_assert_eq!(p.horizontal_accuracy_raw(), expected_hpposllh.h_acc);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_hpposllh.v_acc);
    }
}

#[cfg(feature = "ubx_proto33")]
proptest! {
    #[test]
    fn test_parser_proto33_with_generated_nav_hpposllh_frames(
        (expected_hpposllh, frame) in ubx_nav_hpposllh_frame_strategy(ProtocolVersion::V33)
    ) {
        use ublox::proto33::{Proto33, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto33>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto33(PacketRef::NavHpPosLlh(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-HPPOSLLH valid packet");
        };

        // Assert that the parsed fields match the generated values.
        prop_assert_eq!(p.version(), expected_hpposllh.version);
        prop_assert_eq!(p.itow(), expected_hpposllh.itow);
        prop_assert_eq!(p.flags_raw(), expected_hpposllh.flags);
        prop_assert_eq!(p.lon_degrees_raw(), expected_hpposllh.lon);
        prop_assert_eq!(p.lat_degrees_raw(), expected_hpposllh.lat);
        prop_assert_eq!(p.height_meters_raw(), expected_hpposllh.height);
        prop_assert_eq!(p.height_msl_raw(), expected_hpposllh.h_msl);
        prop_assert_eq!(p.lon_hp_degrees_raw(), expected_hpposllh.lon_hp);
        prop_assert_eq!(p.lat_hp_degrees_raw(), expected_hpposllh.lat_hp);
        prop_assert_eq!(p.height_hp_meters_raw(), expected_hpposllh.height_hp);
        prop_assert_eq!(p.height_hp_msl_raw(), expected_hpposllh.h_msl_hp);
        prop_assert_eq!(p.horizontal_accuracy_raw(), expected_hpposllh.h_acc);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_hpposllh.v_acc);
    }
}
