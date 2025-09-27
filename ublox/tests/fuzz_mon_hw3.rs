//! A proptest generator for U-Blox MON-HW3 messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-HW3 message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents a single pin in the UBX-MON-HW3 message.
#[derive(Debug, Clone)]
pub struct MonHw3Pin {
    pub pin_id: u16,
    pub pin_mask: u16,
    pub vp: u8,
    pub reserved1: u8,
}

/// Represents the payload of a UBX-MON-HW3 message.
///
/// The payload contains a fixed header (version, flags, hwVersion, reserved0)
/// and a variable number of pin records.
#[derive(Debug, Clone)]
pub struct MonHw3 {
    pub version: u8,          // always 0x00
    pub n_pins: u8,           // number of pins
    pub flags: u8,            // flags
    pub hw_version: [u8; 10], // zero-terminated string
    pub reserved0: [u8; 9],   // reserved
    pub pins: Vec<MonHw3Pin>, // repeated group
}

impl MonHw3 {
    /// Serializes the MON-HW3 payload into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(22 + self.pins.len() * 6);

        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.n_pins).unwrap();
        wtr.write_u8(self.flags).unwrap();
        wtr.extend_from_slice(&self.hw_version);
        wtr.extend_from_slice(&self.reserved0);

        for pin in &self.pins {
            wtr.write_u16::<LittleEndian>(pin.pin_id).unwrap();
            wtr.write_u16::<LittleEndian>(pin.pin_mask).unwrap();
            wtr.write_u8(pin.vp).unwrap();
            wtr.write_u8(pin.reserved1).unwrap();
        }

        wtr
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

fn hw_version_strategy() -> impl Strategy<Value = [u8; 10]> {
    // Generate a string length 0..=9 (last byte reserved for zero)
    (0..=9).prop_flat_map(|len| {
        prop::collection::vec(
            prop::char::range('0', 'z'), // ASCII printable characters
            len as usize,
        )
        .prop_map(move |chars| {
            let mut hw = [0u8; 10];
            for (i, c) in chars.iter().enumerate() {
                hw[i] = *c as u8;
            }
            hw
        })
    })
}

/// Proptest strategy for generating a single MON-HW3 pin.
fn mon_hw3_pin_strategy() -> impl Strategy<Value = MonHw3Pin> {
    (any::<u16>(), any::<u16>(), any::<u8>(), any::<u8>()).prop_map(
        |(pin_id, pin_mask, vp, reserved1)| MonHw3Pin {
            pin_id,
            pin_mask,
            vp,
            reserved1,
        },
    )
}

/// Proptest strategy for generating a complete MON-HW3 payload.
pub fn mon_hw3_payload_strategy() -> impl Strategy<Value = MonHw3> {
    (
        Just(0u8),                                            // version
        any::<u8>(),                                          // flags
        hw_version_strategy(),                                // hw_version
        prop::array::uniform9(any::<u8>()),                   // reserved0
        prop::collection::vec(mon_hw3_pin_strategy(), 0..=8), // pins
    )
        .prop_map(|(version, flags, hw_version, reserved0, pins)| MonHw3 {
            version,
            n_pins: pins.len() as u8,
            flags,
            hw_version,
            reserved0,
            pins,
        })
}

/// Proptest strategy for generating a complete UBX frame containing a MON-HW3 message.
pub fn ubx_mon_hw3_frame_strategy() -> impl Strategy<Value = (MonHw3, Vec<u8>)> {
    mon_hw3_payload_strategy().prop_map(|mon_hw3| {
        let payload = mon_hw3.to_bytes();
        let class_id = 0x0A;
        let message_id = 0x37;
        let length = payload.len() as u16;

        // Build the frame core
        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        // Assemble final frame
        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(0xB5);
        final_frame.push(0x62);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (mon_hw3, final_frame)
    })
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_mon_hw3_frames(
        (expected, frame) in ubx_mon_hw3_frame_strategy()
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto31>()
            .with_fixed_buffer::<2048>();

        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::MonHw3(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW3 valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_pins(), expected.n_pins);
        prop_assert_eq!(p.flags_raw(), expected.flags);
        prop_assert_eq!(p.hw_version_raw(), &expected.hw_version);
        prop_assert_eq!(p.reserved0(), expected.reserved0);

        for (pin_expected, pin_parsed) in expected.pins.iter().zip(p.pins()) {
            prop_assert_eq!(pin_parsed.pin_id, pin_expected.pin_id);
            prop_assert_eq!(pin_parsed.vp, pin_expected.vp);
            prop_assert_eq!(pin_parsed.reserved1, pin_expected.reserved1);
        }
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_hw3_frames(
        (expected, frame) in ubx_mon_hw3_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto27>()
            .with_fixed_buffer::<2048>();

        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonHw3(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-HW3 valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_pins(), expected.n_pins);
        prop_assert_eq!(p.flags_raw(), expected.flags);
        prop_assert_eq!(p.hw_version_raw(), &expected.hw_version);
        prop_assert_eq!(p.reserved0(), expected.reserved0);

        for (pin_expected, pin_parsed) in expected.pins.iter().zip(p.pins()) {
            prop_assert_eq!(pin_parsed.pin_id, pin_expected.pin_id);
            prop_assert_eq!(pin_parsed.vp, pin_expected.vp);
            prop_assert_eq!(pin_parsed.reserved1, pin_expected.reserved1);
        }
    }
}
