#![cfg(any(
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]

//! A proptest generator for U-Blox SEC-SIGLOG messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a SEC-SIGLOG message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload with repeated event blocks, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

mod common;
use common::build_ubx_frame;

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
    pub version: u8,                        // Message version (0x01 for this version)
    pub reserved0: [u8; 6],                 // Reserved
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

/// A proptest strategy for generating a single `SecSiglogEventPayload`.
fn sec_siglog_event_strategy() -> impl Strategy<Value = SecSiglogEventPayload> {
    (any::<u32>(), any::<u8>(), any::<u8>()).prop_map(
        |(time_elapsed_s, detection_type, event_type)| SecSiglogEventPayload {
            time_elapsed_s,
            detection_type,
            event_type,
        },
    )
}

/// A proptest strategy for generating a `SecSiglogPayload` struct.
fn sec_siglog_payload_strategy() -> impl Strategy<Value = SecSiglogPayload> {
    (
        Just(1u8),
        Just([0u8; 6]),
        // Max 16 events (per device documentation)
        prop::collection::vec(sec_siglog_event_strategy(), 0..=16),
    )
        .prop_map(|(version, reserved0, events)| SecSiglogPayload {
            version,
            reserved0,
            events,
        })
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a SEC-SIGLOG message, along with the source payload struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(SecSiglogPayload, Vec<u8>)`.
pub fn ubx_sec_siglog_frame_strategy() -> impl Strategy<Value = (SecSiglogPayload, Vec<u8>)> {
    sec_siglog_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let final_frame = build_ubx_frame(0x27, 0x10, &payload);

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

#[cfg(feature = "ubx_proto33")]
proptest! {
    #[test]
    fn test_parser_proto33_with_generated_sec_siglog_frames((expected, frame) in ubx_sec_siglog_frame_strategy()) {
        use ublox::proto33::{PacketRef, Proto33};

        let mut parser = ParserBuilder::new().with_protocol::<Proto33>().with_fixed_buffer::<4096>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto33(PacketRef::SecSiglog(p)))) = it.next() else {
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
