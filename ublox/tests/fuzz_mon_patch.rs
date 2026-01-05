//! A proptest generator for U-Blox MON-PATCH messages.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

const SYNC_CHAR_1: u8 = 0xB5;
const SYNC_CHAR_2: u8 = 0x62;

#[derive(Debug, Clone)]
pub struct MonPatchEntry {
    pub patch_info: u32,
    pub comparator_number: u32,
    pub patch_address: u32,
    pub patch_data: u32,
}

impl MonPatchEntry {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(16);
        wtr.write_u32::<LittleEndian>(self.patch_info).unwrap();
        wtr.write_u32::<LittleEndian>(self.comparator_number)
            .unwrap();
        wtr.write_u32::<LittleEndian>(self.patch_address).unwrap();
        wtr.write_u32::<LittleEndian>(self.patch_data).unwrap();
        wtr
    }
}

#[derive(Debug, Clone)]
pub struct MonPatchPayload {
    pub version: u16,
    pub entries: Vec<MonPatchEntry>,
}

impl MonPatchPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(4 + self.entries.len() * 16);
        wtr.write_u16::<LittleEndian>(self.version).unwrap();
        wtr.write_u16::<LittleEndian>(self.entries.len() as u16)
            .unwrap();
        for entry in &self.entries {
            wtr.extend_from_slice(&entry.to_bytes());
        }
        wtr
    }
}

fn mon_patch_entry_strategy() -> impl Strategy<Value = MonPatchEntry> {
    (any::<u32>(), any::<u32>(), any::<u32>(), any::<u32>()).prop_map(
        |(patch_info, comparator_number, patch_address, patch_data)| MonPatchEntry {
            patch_info,
            comparator_number,
            patch_address,
            patch_data,
        },
    )
}

fn mon_patch_payload_strategy() -> impl Strategy<Value = MonPatchPayload> {
    (
        Just(0x0001u16), // version is always 0x0001
        prop::collection::vec(mon_patch_entry_strategy(), 0..=8),
    )
        .prop_map(|(version, entries)| MonPatchPayload { version, entries })
}

fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

pub fn ubx_mon_patch_frame_strategy() -> impl Strategy<Value = (MonPatchPayload, Vec<u8>)> {
    mon_patch_payload_strategy().prop_map(|payload_data| {
        let payload = payload_data.to_bytes();

        let class_id = 0x0a;
        let message_id = 0x27;
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

        (payload_data, final_frame)
    })
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_patch_frames(
        (expected, frame) in ubx_mon_patch_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonPatch(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-PATCH packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_entries(), expected.entries.len() as u16);

        let parsed_entries: Vec<_> = p.patches().collect();
        prop_assert_eq!(parsed_entries.len(), expected.entries.len());

        for (parsed, expected_entry) in parsed_entries.iter().zip(expected.entries.iter()) {
            prop_assert_eq!(parsed.patch_info.raw(), expected_entry.patch_info);
            prop_assert_eq!(parsed.comparator_number, expected_entry.comparator_number);
            prop_assert_eq!(parsed.patch_address, expected_entry.patch_address);
            prop_assert_eq!(parsed.patch_data, expected_entry.patch_data);
        }
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_patch_frames(
        (expected, frame) in ubx_mon_patch_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonPatch(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-PATCH packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_entries(), expected.entries.len() as u16);

        let parsed_entries: Vec<_> = p.patches().collect();
        prop_assert_eq!(parsed_entries.len(), expected.entries.len());

        for (parsed, expected_entry) in parsed_entries.iter().zip(expected.entries.iter()) {
            prop_assert_eq!(parsed.patch_info.raw(), expected_entry.patch_info);
            prop_assert_eq!(parsed.comparator_number, expected_entry.comparator_number);
            prop_assert_eq!(parsed.patch_address, expected_entry.patch_address);
            prop_assert_eq!(parsed.patch_data, expected_entry.patch_data);
        }
    }
}
