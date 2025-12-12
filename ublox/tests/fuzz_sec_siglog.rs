#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox SEC-SIGLOG messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a SEC-SIGLOG message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload with repeated event blocks, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents a single 8-byte event entry within a SEC-SIGLOG message.
#[derive(Debug, Clone)]
pub struct SecSiglogEventPayload {
    pub time_elapsed_s: u32, // Seconds elapsed since this event
    pub detection_type: u8,  // Type of spoofing/jamming detection
    pub event_type: u8,      // Type of the event
}

/// Represents the payload of a UBX-SEC-SIGLOG message.
///
/// The fields are ordered as they appear in the u-blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
#[derive(Debug, Clone)]
pub struct SecSiglogPayload {
    pub version: u8,           // Message version (0x01 for this version)
    pub reserved0: [u8; 6],    // Reserved
    pub events: Vec<SecSiglogEventPayload>, // Repeated event blocks
}

impl SecSiglogPayload {
    /// Serializes the SecSiglogPayload into a vector.
    /// Size is 8 + (number of events * 8) bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let num_events = self.events.len() as u8;
        let mut wtr = Vec::with_capacity(8 + (self.events.len() * 8));
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(num_events).unwrap();
        wtr.extend_from_slice(&self.reserved0);

        for e in &self.events {
            wtr.write_u32::<LittleEndian>(e.time_elapsed_s).unwrap();
            wtr.write_u8(e.detection_type).unwrap();
            wtr.write_u8(e.event_type).unwrap();
            // bytes 6..8 are reserved
            wtr.extend_from_slice(&[0u8; 2]);
        }

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

/// A proptest strategy for generating a single `SecSiglogEventPayload`.
fn sec_siglog_event_strategy() -> impl Strategy<Value = SecSiglogEventPayload> {
    (
        any::<u32>(),
        any::<u8>(),
        any::<u8>(),
    )
        .prop_map(|(time_elapsed_s, detection_type, event_type)| SecSiglogEventPayload {
            time_elapsed_s,
            detection_type,
            event_type,
        })
}

/// A proptest strategy for generating a `SecSiglogPayload` struct.
fn sec_siglog_payload_strategy() -> impl Strategy<Value = SecSiglogPayload> {
    (
        Just(1u8),
        Just([0u8; 6]),
        // Max 16 events (per device documentation)
        prop::collection::vec(sec_siglog_event_strategy(), 0..=16),
    )
        .prop_map(|(version, reserved0, events)| SecSiglogPayload { version, reserved0, events })
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a SEC-SIGLOG message, along with the source payload struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(SecSiglogPayload, Vec<u8>)`.
pub fn ubx_sec_siglog_frame_strategy() -> impl Strategy<Value = (SecSiglogPayload, Vec<u8>)> {
    sec_siglog_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x27;
        let message_id = 0x10;
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
    fn test_parser_proto27_with_generated_sec_siglog_frames((expected, frame) in ubx_sec_siglog_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<4096>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::SecSiglog(p)))) = it.next() else {
            panic!("Parser failed to parse a SEC-SIGLOG valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.num_events(), expected.events.len() as u8);
        prop_assert_eq!(p.events().count(), expected.events.len());

        let mut parsed = p.events();
        for expected_event in &expected.events {
            let parsed_event = parsed.next().unwrap();

            prop_assert_eq!(parsed_event.time_elapsed_s, expected_event.time_elapsed_s);
            prop_assert_eq!(parsed_event.detection_type, expected_event.detection_type);
            prop_assert_eq!(parsed_event.event_type, expected_event.event_type);
        }
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_sec_siglog_frames((expected, frame) in ubx_sec_siglog_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<4096>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::SecSiglog(p)))) = it.next() else {
            panic!("Parser failed to parse a SEC-SIGLOG valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.num_events(), expected.events.len() as u8);
        prop_assert_eq!(p.events().count(), expected.events.len());

        let mut parsed = p.events();
        for expected_event in &expected.events {
            let parsed_event = parsed.next().unwrap();

            prop_assert_eq!(parsed_event.time_elapsed_s, expected_event.time_elapsed_s);
            prop_assert_eq!(parsed_event.detection_type, expected_event.detection_type);
            prop_assert_eq!(parsed_event.event_type, expected_event.event_type);
        }
    }
}
