#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox MON-RF messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-RF message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload with a variable number of repeating
//! blocks, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{
    constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2},
    ParserBuilder, UbxPacket,
};

/// Represents a single RF block within a MON-RF message payload.
///
/// This struct holds raw integer values that are easy for proptest to
/// generate, which are then compared against the parsed output.
#[derive(Debug, Clone)]
pub struct RfBlock {
    pub block_id: u8,
    pub flags: u8,
    pub ant_status: u8,
    pub ant_power: u8,
    pub post_status: u32,
    pub reserved1: [u8; 4],
    pub noise_per_ms: u16,
    pub agc_cnt: u16,
    pub jam_ind: u8,
    pub ofs_i: i8,
    pub mag_i: u8,
    pub ofs_q: i8,
    pub mag_q: u8,
    pub reserved2: [u8; 3],
}

impl RfBlock {
    /// Serializes the RfBlock struct into a 24-byte vector.
    pub fn to_bytes(&self, wtr: &mut Vec<u8>) {
        wtr.write_u8(self.block_id).unwrap();
        wtr.write_u8(self.flags).unwrap();
        wtr.write_u8(self.ant_status).unwrap();
        wtr.write_u8(self.ant_power).unwrap();
        wtr.write_u32::<LittleEndian>(self.post_status).unwrap();
        wtr.extend_from_slice(&self.reserved1);
        wtr.write_u16::<LittleEndian>(self.noise_per_ms).unwrap();
        wtr.write_u16::<LittleEndian>(self.agc_cnt).unwrap();
        wtr.write_u8(self.jam_ind).unwrap();
        wtr.write_i8(self.ofs_i).unwrap();
        wtr.write_u8(self.mag_i).unwrap();
        wtr.write_i8(self.ofs_q).unwrap();
        wtr.write_u8(self.mag_q).unwrap();
        wtr.extend_from_slice(&self.reserved2);
    }
}

/// Represents the payload of a UBX-MON-RF message.
#[derive(Debug, Clone)]
pub struct MonRf {
    pub version: u8,
    pub reserved0: [u8; 2],
    pub blocks: Vec<RfBlock>,
}

impl MonRf {
    /// Serializes the MonRf payload into a vector.
    /// The size is 4 + (number of blocks * 24) bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let n_blocks = self.blocks.len() as u8;
        let capacity = 4 + (n_blocks as usize * 24);
        let mut wtr = Vec::with_capacity(capacity);
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(n_blocks).unwrap();
        wtr.extend_from_slice(&self.reserved0);
        for block in &self.blocks {
            block.to_bytes(&mut wtr);
        }
        wtr
    }
}

/// A proptest strategy for generating a single `RfBlock` struct.
pub fn rf_block_strategy() -> impl Strategy<Value = RfBlock> {
    (
        (
            any::<u8>(),                                                // block_id
            (0..=3u8), // flags (only jammingState bits 0-1 are used)
            prop_oneof![Just(0u8), Just(1), Just(2), Just(3), Just(4)], // ant_status
            prop_oneof![Just(0u8), Just(1), Just(2)], // ant_power
            any::<u32>(), // post_status
            Just([0u8; 4]), // reserved1
            any::<u16>(), // noise_per_ms
        ),
        (
            (0..=8191u16),  // agc_cnt (valid range 0-8191)
            any::<u8>(),    // jam_ind
            any::<i8>(),    // ofs_i
            any::<u8>(),    // mag_i
            any::<i8>(),    // ofs_q
            any::<u8>(),    // mag_q
            Just([0u8; 3]), // reserved2
        ),
    )
        .prop_map(
            |(
                (block_id, flags, ant_status, ant_power, post_status, reserved1, noise_per_ms),
                (agc_cnt, jam_ind, ofs_i, mag_i, ofs_q, mag_q, reserved2),
            )| {
                RfBlock {
                    block_id,
                    flags,
                    ant_status,
                    ant_power,
                    post_status,
                    reserved1,
                    noise_per_ms,
                    agc_cnt,
                    jam_ind,
                    ofs_i,
                    mag_i,
                    ofs_q,
                    mag_q,
                    reserved2,
                }
            },
        )
}

/// A proptest strategy for generating a `MonRf` payload struct.
pub fn mon_rf_payload_strategy() -> impl Strategy<Value = MonRf> {
    (
        Just(0u8),      // version
        Just([0u8; 2]), // reserved0
        // Max 42 blocks to fit within a 1028-byte max payload length.
        prop::collection::vec(rf_block_strategy(), 0..=42),
    )
        .prop_map(|(version, reserved0, blocks)| MonRf {
            version,
            reserved0,
            blocks,
        })
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
/// containing a MON-RF message, along with the source `MonRf` struct.
pub fn ubx_mon_rf_frame_strategy() -> impl Strategy<Value = (MonRf, Vec<u8>)> {
    mon_rf_payload_strategy().prop_map(|mon_rf| {
        let payload = mon_rf.to_bytes();
        let class_id = 0x0A; // MON class
        let message_id = 0x38; // RF message ID
        let length = payload.len() as u16;

        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(UBX_SYNC_CHAR_1);
        final_frame.push(UBX_SYNC_CHAR_2);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (mon_rf, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_rf_frames(
        (expected_mon_rf, frame) in ubx_mon_rf_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonRf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RF valid packet");
        };

        prop_assert_eq!(p.version(), expected_mon_rf.version);
        prop_assert_eq!(p.n_blocks(), expected_mon_rf.blocks.len() as u8);
        prop_assert_eq!(p.blocks().count(), expected_mon_rf.blocks.len());

        let mut parsed_blocks = p.blocks();
        for expected_block in &expected_mon_rf.blocks {
            let parsed_block = parsed_blocks.next().unwrap();

            prop_assert_eq!(parsed_block.block_id, expected_block.block_id);
            prop_assert_eq!(parsed_block.flags, expected_block.flags.into());
            prop_assert_eq!(parsed_block.ant_status as u8, expected_block.ant_status);
            prop_assert_eq!(parsed_block.ant_power as u8, expected_block.ant_power);
            prop_assert_eq!(parsed_block.post_status, expected_block.post_status);
            prop_assert_eq!(parsed_block.noise_per_ms, expected_block.noise_per_ms);
            prop_assert_eq!(parsed_block.agc_cnt, expected_block.agc_cnt);
            prop_assert_eq!(parsed_block.jam_ind, expected_block.jam_ind);
            prop_assert_eq!(parsed_block.ofs_i, expected_block.ofs_i);
            prop_assert_eq!(parsed_block.mag_i, expected_block.mag_i);
            prop_assert_eq!(parsed_block.ofs_q, expected_block.ofs_q);
            prop_assert_eq!(parsed_block.mag_q, expected_block.mag_q);
        }
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_mon_rf_frames(
        (expected_mon_rf, frame) in ubx_mon_rf_frame_strategy()
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::MonRf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RF valid packet");
        };

        prop_assert_eq!(p.version(), expected_mon_rf.version);
        prop_assert_eq!(p.n_blocks(), expected_mon_rf.blocks.len() as u8);
        prop_assert_eq!(p.blocks().count(), expected_mon_rf.blocks.len());

        let mut parsed_blocks = p.blocks();
        for expected_block in &expected_mon_rf.blocks {
            let parsed_block = parsed_blocks.next().unwrap();

            prop_assert_eq!(parsed_block.block_id, expected_block.block_id);
            prop_assert_eq!(parsed_block.flags, expected_block.flags.into());
            prop_assert_eq!(parsed_block.ant_status as u8, expected_block.ant_status);
            prop_assert_eq!(parsed_block.ant_power as u8, expected_block.ant_power);
            prop_assert_eq!(parsed_block.post_status, expected_block.post_status);
            prop_assert_eq!(parsed_block.noise_per_ms, expected_block.noise_per_ms);
            prop_assert_eq!(parsed_block.agc_cnt, expected_block.agc_cnt);
            prop_assert_eq!(parsed_block.jam_ind, expected_block.jam_ind);
            prop_assert_eq!(parsed_block.ofs_i, expected_block.ofs_i);
            prop_assert_eq!(parsed_block.mag_i, expected_block.mag_i);
            prop_assert_eq!(parsed_block.ofs_q, expected_block.ofs_q);
            prop_assert_eq!(parsed_block.mag_q, expected_block.mag_q);
        }
    }
}
