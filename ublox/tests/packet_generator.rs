//! A proptest generator for U-Blox NAV-PVT messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a NAV-PVT message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

#[derive(Debug, Clone, Copy)]
pub enum ProtocolVersion {
    V14,
    V23,
    V27,
    V31,
}

/// Represents the payload of a UBX-NAV-PVT message.
///
/// The fields are ordered as they appear in the U-Blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
#[derive(Debug, Clone)]
pub struct NavPvt {
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub t_acc: u32,
    pub nano: i32,
    pub fix_type: u8,
    pub flags: u8,
    pub flags2: u8,
    pub num_sv: u8,
    pub lon: i32,
    pub lat: i32,
    pub height: i32,
    pub h_msl: i32,
    pub h_acc: u32,
    pub v_acc: u32,
    pub vel_n: i32,
    pub vel_e: i32,
    pub vel_d: i32,
    pub g_speed: i32,
    pub head_mot: i32,
    pub s_acc: u32,
    pub head_acc: u32,
    pub p_dop: u16,
    pub flags3: u8,
    pub reserved1: [u8; 5],
    pub head_veh: i32,
    pub mag_dec: i16,
    pub mag_acc: u16,
}

impl NavPvt {
    /// Serializes the NavPvt payload into a 92-byte vector.
    pub fn to_bytes(&self, version: ProtocolVersion) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(92);
        wtr.write_u32::<LittleEndian>(self.itow).unwrap();
        wtr.write_u16::<LittleEndian>(self.year).unwrap();
        wtr.write_u8(self.month).unwrap();
        wtr.write_u8(self.day).unwrap();
        wtr.write_u8(self.hour).unwrap();
        wtr.write_u8(self.min).unwrap();
        wtr.write_u8(self.sec).unwrap();
        wtr.write_u8(self.valid).unwrap();
        wtr.write_u32::<LittleEndian>(self.t_acc).unwrap();
        wtr.write_i32::<LittleEndian>(self.nano).unwrap();
        wtr.write_u8(self.fix_type).unwrap();
        wtr.write_u8(self.flags).unwrap();
        wtr.write_u8(self.flags2).unwrap();
        wtr.write_u8(self.num_sv).unwrap();
        wtr.write_i32::<LittleEndian>(self.lon).unwrap();
        wtr.write_i32::<LittleEndian>(self.lat).unwrap();
        wtr.write_i32::<LittleEndian>(self.height).unwrap();
        wtr.write_i32::<LittleEndian>(self.h_msl).unwrap();
        wtr.write_u32::<LittleEndian>(self.h_acc).unwrap();
        wtr.write_u32::<LittleEndian>(self.v_acc).unwrap();
        wtr.write_i32::<LittleEndian>(self.vel_n).unwrap();
        wtr.write_i32::<LittleEndian>(self.vel_e).unwrap();
        wtr.write_i32::<LittleEndian>(self.vel_d).unwrap();
        wtr.write_i32::<LittleEndian>(self.g_speed).unwrap();
        wtr.write_i32::<LittleEndian>(self.head_mot).unwrap();
        wtr.write_u32::<LittleEndian>(self.s_acc).unwrap();
        wtr.write_u32::<LittleEndian>(self.head_acc).unwrap();
        wtr.write_u16::<LittleEndian>(self.p_dop).unwrap();
        wtr.write_u8(self.flags3).unwrap();
        wtr.extend_from_slice(&self.reserved1);
        wtr.write_i32::<LittleEndian>(self.head_veh).unwrap();
        wtr.write_i16::<LittleEndian>(self.mag_dec).unwrap();
        wtr.write_u16::<LittleEndian>(self.mag_acc).unwrap();

        // TRUNCATE the payload for older versions
        match version {
            ProtocolVersion::V14 => {
                // Proto 14 NAV-PVT payload is 84 bytes. The last 8 bytes
                // (head_veh, mag_dec, mag_acc) are not part of it.
                wtr.truncate(84);
            },
            // Newer versions use the full 92 bytes
            ProtocolVersion::V23 | ProtocolVersion::V27 | ProtocolVersion::V31 => {},
        }
        wtr
    }
}

/// A proptest strategy for generating a `NavPvt` payload struct.
///
/// This strategy is parameterized by the protocol version to generate
/// a payload that is valid for that specific version.
pub fn nav_pvt_payload_strategy(version: ProtocolVersion) -> impl Strategy<Value = NavPvt> {
    // Base strategies for fields common to all versions
    let time_and_date = (
        any::<u32>(),                        // itow
        (1999..=2099u16),                    // year
        (1..=12u8),                          // month
        (1..=31u8),                          // day
        (0..=23u8),                          // hour
        (0..=59u8),                          // min
        (0..=60u8),                          // sec
        any::<u8>(),                         // valid
        any::<u32>(),                        // t_acc
        (-1_000_000_000..=1_000_000_000i32), // nano
    );

    let fix_and_pos = (
        (0..=5u8),                     // fix_type
        any::<u8>(),                   // flags (bitfield)
        any::<u8>(),                   // flags2 (bitfield)
        (0..=99u8),                    // num_sv
        (-1800000000..=1800000000i32), // lon
        (-900000000..=900000000i32),   // lat
        any::<i32>(),                  // height
        any::<i32>(),                  // h_msl
        any::<u32>(),                  // h_acc
        any::<u32>(),                  // v_acc
    );

    let velocity_and_heading = (
        any::<i32>(),                // vel_n
        any::<i32>(),                // vel_e
        any::<i32>(),                // vel_d
        any::<i32>(),                // g_speed
        (-180000000..=360000000i32), // head_mot
        any::<u32>(),                // s_acc
        any::<u32>(),                // head_acc
        any::<u16>(),                // p_dop
    );

    // Conditionally define strategies for version-specific fields
    match version {
        ProtocolVersion::V14 => (
            time_and_date,
            fix_and_pos,
            velocity_and_heading,
            // Group 4: Fields for proto14 (older)
            (
                Just(0u8),      // flags3 is reserved
                Just([0u8; 5]), // reserved1
                Just(0i32),     // head_veh is reserved
                Just(0i16),     // mag_dec is reserved
                Just(0u16),     // mag_acc is reserved
            ),
        )
            .sboxed(), // Use .sboxed() to unify the return types
        ProtocolVersion::V23 | ProtocolVersion::V27 | ProtocolVersion::V31 => (
            time_and_date,
            fix_and_pos,
            velocity_and_heading,
            // Group 4: Fields for proto23+ (newer)
            (
                any::<u8>(),                        // flags3
                prop::array::uniform5(any::<u8>()), // reserved1
                (-180000000..=360000000i32),        // head_veh
                any::<i16>(),                       // mag_dec
                any::<u16>(),                       // mag_acc
            ),
        )
            .sboxed(),
    }
    .prop_map(
        |(
            (itow, year, month, day, hour, min, sec, valid, t_acc, nano),
            (fix_type, flags, flags2, num_sv, lon, lat, height, h_msl, h_acc, v_acc),
            (vel_n, vel_e, vel_d, g_speed, head_mot, s_acc, head_acc, p_dop),
            (flags3, reserved1, head_veh, mag_dec, mag_acc),
        )| NavPvt {
            itow,
            year,
            month,
            day,
            hour,
            min,
            sec,
            valid,
            t_acc,
            nano,
            fix_type,
            flags,
            flags2,
            num_sv,
            lon,
            lat,
            height,
            h_msl,
            h_acc,
            v_acc,
            vel_n,
            vel_e,
            vel_d,
            g_speed,
            head_mot,
            s_acc,
            head_acc,
            p_dop,
            flags3,
            reserved1,
            head_veh,
            mag_dec,
            mag_acc,
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
/// containing a NAV-PVT message, along with the source `NavPvt` struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(NavPvt, Vec<u8>)`.
pub fn ubx_nav_pvt_frame_strategy(
    version: ProtocolVersion,
) -> impl Strategy<Value = (NavPvt, Vec<u8>)> {
    nav_pvt_payload_strategy(version).prop_map(move |nav_pvt| {
        let payload = nav_pvt.to_bytes(version);
        let class_id = 0x01;
        let message_id = 0x07;
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
        final_frame.push(0xB5); // Sync Char 1
        final_frame.push(0x62); // Sync Char 2
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (nav_pvt, final_frame)
    })
}

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_nav_pvt_frames(
        (expected_pvt, frame)  in ubx_nav_pvt_frame_strategy(ProtocolVersion::V14)
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::NavPvt(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PVT valid packet");
        };

        // Assert that most of the the parsed fields match the generated values.
        prop_assert_eq!(p.itow(), expected_pvt.itow);
        prop_assert_eq!(p.day(), expected_pvt.day);
        prop_assert_eq!(p.ground_speed_2d_raw(), expected_pvt.g_speed);
        prop_assert_eq!(p.heading_motion_raw(), expected_pvt.head_mot);
        prop_assert_eq!(p.longitude_raw(), expected_pvt.lon);
        prop_assert_eq!(p.latitude_raw(), expected_pvt.lat);
        prop_assert_eq!(p.height_above_ellipsoid_raw(), expected_pvt.height);
        prop_assert_eq!(p.pdop_raw(), expected_pvt.p_dop);
        prop_assert_eq!(p.vel_down_raw(), expected_pvt.vel_d);
        prop_assert_eq!(p.vel_east_raw(), expected_pvt.vel_e);
        prop_assert_eq!(p.vel_north_raw(), expected_pvt.vel_n);
        prop_assert_eq!(p.height_msl_raw(), expected_pvt.h_msl);
        prop_assert_eq!(p.fix_type_raw(), expected_pvt.fix_type);
        prop_assert_eq!(p.flags_raw(), expected_pvt.flags);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_pvt.v_acc);
        prop_assert_eq!(p.magnetic_declination_raw(), expected_pvt.mag_dec);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
    }
}

#[cfg(feature = "ubx_proto23")]
proptest! {
    #[test]
    fn test_parser_proto23_with_generated_nav_pvt_frames(
        (expected_pvt, frame) in ubx_nav_pvt_frame_strategy(ProtocolVersion::V23)
    ) {
        use ublox::proto23::{Proto23, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto23>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto23(PacketRef::NavPvt(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PVT valid packet");
        };

        // Assert that most of the the parsed fields match the generated values.
        prop_assert_eq!(p.itow(), expected_pvt.itow);
        prop_assert_eq!(p.day(), expected_pvt.day);
        prop_assert_eq!(p.ground_speed_2d_raw(), expected_pvt.g_speed);
        prop_assert_eq!(p.heading_motion_raw(), expected_pvt.head_mot);
        prop_assert_eq!(p.longitude_raw(), expected_pvt.lon);
        prop_assert_eq!(p.latitude_raw(), expected_pvt.lat);
        prop_assert_eq!(p.height_above_ellipsoid_raw(), expected_pvt.height);
        prop_assert_eq!(p.pdop_raw(), expected_pvt.p_dop);
        prop_assert_eq!(p.vel_down_raw(), expected_pvt.vel_d);
        prop_assert_eq!(p.vel_east_raw(), expected_pvt.vel_e);
        prop_assert_eq!(p.vel_north_raw(), expected_pvt.vel_n);
        prop_assert_eq!(p.height_msl_raw(), expected_pvt.h_msl);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_pvt.v_acc);
        prop_assert_eq!(p.magnetic_declination_raw(), expected_pvt.mag_dec);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.fix_type_raw(), expected_pvt.fix_type);
        prop_assert_eq!(p.flags_raw(), expected_pvt.flags);
        prop_assert_eq!(p.flags2_raw(), expected_pvt.flags2);
        prop_assert_eq!(p.flags3_raw(), expected_pvt.flags3, "Invalid flags3_raw = {:?}, flags3 = {:?}", p.flags3_raw(), p.flags3());


    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_nav_pvt_frames(
        (expected_pvt, frame) in ubx_nav_pvt_frame_strategy(ProtocolVersion::V27)
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);
        let Some(Ok(UbxPacket::Proto27(PacketRef::NavPvt(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PVT valid packet");
        };

        // Assert that most of the the parsed fields match the generated values.
        prop_assert_eq!(p.itow(), expected_pvt.itow);
        prop_assert_eq!(p.day(), expected_pvt.day);
        prop_assert_eq!(p.ground_speed_2d_raw(), expected_pvt.g_speed);
        prop_assert_eq!(p.heading_motion_raw(), expected_pvt.head_mot);
        prop_assert_eq!(p.longitude_raw(), expected_pvt.lon);
        prop_assert_eq!(p.latitude_raw(), expected_pvt.lat);
        prop_assert_eq!(p.height_above_ellipsoid_raw(), expected_pvt.height);
        prop_assert_eq!(p.pdop_raw(), expected_pvt.p_dop);
        prop_assert_eq!(p.vel_down_raw(), expected_pvt.vel_d);
        prop_assert_eq!(p.vel_east_raw(), expected_pvt.vel_e);
        prop_assert_eq!(p.vel_north_raw(), expected_pvt.vel_n);
        prop_assert_eq!(p.height_msl_raw(), expected_pvt.h_msl);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_pvt.v_acc);
        prop_assert_eq!(p.magnetic_declination_raw(), expected_pvt.mag_dec);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.fix_type_raw(), expected_pvt.fix_type);
        prop_assert_eq!(p.flags_raw(), expected_pvt.flags);
        prop_assert_eq!(p.flags2_raw(), expected_pvt.flags2);
        prop_assert_eq!(p.flags3_raw(), expected_pvt.flags3);
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_nav_pvt_frames(
        (expected_pvt, frame)  in ubx_nav_pvt_frame_strategy(ProtocolVersion::V31)
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);
        let Some(Ok(UbxPacket::Proto31(PacketRef::NavPvt(p)))) = it.next() else {
            panic!("Parser failed to parse a NAV-PVT valid packet");
        };

        // Assert that most of the the parsed fields match the generated values.
        prop_assert_eq!(p.itow(), expected_pvt.itow);
        prop_assert_eq!(p.day(), expected_pvt.day);
        prop_assert_eq!(p.ground_speed_2d_raw(), expected_pvt.g_speed);
        prop_assert_eq!(p.heading_motion_raw(), expected_pvt.head_mot);
        prop_assert_eq!(p.longitude_raw(), expected_pvt.lon);
        prop_assert_eq!(p.latitude_raw(), expected_pvt.lat);
        prop_assert_eq!(p.height_above_ellipsoid_raw(), expected_pvt.height);
        prop_assert_eq!(p.pdop_raw(), expected_pvt.p_dop);
        prop_assert_eq!(p.vel_down_raw(), expected_pvt.vel_d);
        prop_assert_eq!(p.vel_east_raw(), expected_pvt.vel_e);
        prop_assert_eq!(p.vel_north_raw(), expected_pvt.vel_n);
        prop_assert_eq!(p.height_msl_raw(), expected_pvt.h_msl);
        prop_assert_eq!(p.vertical_accuracy_raw(), expected_pvt.v_acc);
        prop_assert_eq!(p.magnetic_declination_raw(), expected_pvt.mag_dec);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.magnetic_declination_accuracy_raw(), expected_pvt.mag_acc);
        prop_assert_eq!(p.fix_type_raw(), expected_pvt.fix_type);
        prop_assert_eq!(p.flags_raw(), expected_pvt.flags);
        prop_assert_eq!(p.flags2_raw(), expected_pvt.flags2);
        prop_assert_eq!(p.flags3_raw(), expected_pvt.flags3);
    }
}
