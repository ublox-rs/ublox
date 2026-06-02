#![cfg(any(
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]

//! A proptest generator for U-Blox SEC-SIG messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a SEC-SIG message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

mod common;
use common::build_ubx_frame;

/// Represents the payload of a UBX-SEC-SIG message.
///
/// The fields are ordered as they appear in the u-blox documentation.
/// This struct makes it easy for proptest to generate and shrink
/// meaningful values for each field before they are serialized into bytes.
///
/// SEC-SIG payload is variable length: 4 + jamNumCentFreqs * 4 bytes.
#[derive(Debug, Clone)]
pub struct SecSigPayload {
    pub version: u8,                    // Message version
    pub sig_sec_flags: u8,              // Signal security flags
    pub reserved0: u8,                  // Reserved
    pub jam_num_cent_freqs: u8,         // Number of center frequencies
    pub jam_state_cent_freqs: Vec<u32>, // Repeated jamStateCentFreq blocks
}

impl SecSigPayload {
    /// Serializes the SecSigPayload into a vector.
    /// Size is 4 + (jamNumCentFreqs * 4) bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(4 + (self.jam_state_cent_freqs.len() * 4));
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.sig_sec_flags).unwrap();
        wtr.write_u8(self.reserved0).unwrap();
        wtr.write_u8(self.jam_num_cent_freqs).unwrap();
        for v in &self.jam_state_cent_freqs {
            wtr.write_u32::<LittleEndian>(*v).unwrap();
        }
        wtr
    }
}

/// A proptest strategy for generating a `SecSigPayload` struct.
fn sec_sig_payload_strategy() -> impl Strategy<Value = SecSigPayload> {
    (
        // Version as per device spec (varies by receiver); allow any.
        any::<u8>(),
        any::<u8>(),
        Just(0u8),
        // Keep the number of repeated blocks bounded for test performance.
        prop::collection::vec(0u32..=0x01ff_ffffu32, 0..=32),
    )
        .prop_map(|(version, sig_sec_flags, reserved0, mut blocks)| {
            // Ensure jamNumCentFreqs matches the number of blocks.
            let jam_num_cent_freqs = blocks.len().min(32) as u8;
            blocks.truncate(jam_num_cent_freqs as usize);
            SecSigPayload {
                version,
                sig_sec_flags,
                reserved0,
                jam_num_cent_freqs,
                jam_state_cent_freqs: blocks,
            }
        })
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a SEC-SIG message, along with the source payload struct.
///
/// This is the main strategy to use in tests. It returns a tuple of
/// `(SecSigPayload, Vec<u8>)`.
pub fn ubx_sec_sig_frame_strategy() -> impl Strategy<Value = (SecSigPayload, Vec<u8>)> {
    sec_sig_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let final_frame = build_ubx_frame(0x27, 0x09, &payload);

        (payload_struct, final_frame)
    })
}

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_sec_sig_frames((expected, frame) in ubx_sec_sig_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::SecSig(p)))) = it.next() else {
            panic!("Parser failed to parse a SEC-SIG valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.sig_sec_flags_raw(), expected.sig_sec_flags);
        prop_assert_eq!(p.jam_num_cent_freqs(), expected.jam_num_cent_freqs);
        prop_assert_eq!(p.jam_state_cent_freqs().count(), expected.jam_state_cent_freqs.len());
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_sec_sig_frames((expected, frame) in ubx_sec_sig_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::SecSig(p)))) = it.next() else {
            panic!("Parser failed to parse a SEC-SIG valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.sig_sec_flags_raw(), expected.sig_sec_flags);
        prop_assert_eq!(p.jam_num_cent_freqs(), expected.jam_num_cent_freqs);
        prop_assert_eq!(p.jam_state_cent_freqs().count(), expected.jam_state_cent_freqs.len());
    }
}

#[cfg(feature = "ubx_proto33")]
proptest! {
    #[test]
    fn test_parser_proto33_with_generated_sec_sig_frames((expected, frame) in ubx_sec_sig_frame_strategy()) {
        use ublox::proto33::{PacketRef, Proto33};

        let mut parser = ParserBuilder::new().with_protocol::<Proto33>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto33(PacketRef::SecSig(p)))) = it.next() else {
            panic!("Parser failed to parse a SEC-SIG valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.sig_sec_flags_raw(), expected.sig_sec_flags);
        prop_assert_eq!(p.jam_num_cent_freqs(), expected.jam_num_cent_freqs);
        prop_assert_eq!(p.jam_state_cent_freqs().count(), expected.jam_state_cent_freqs.len());
    }
}
