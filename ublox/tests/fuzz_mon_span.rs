//! A proptest generator for U-Blox MON-SPAN messages.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{constants::UBX_SYNC_CHAR_1, constants::UBX_SYNC_CHAR_2, ParserBuilder, UbxPacket};

const SPECTRUM_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct MonSpanRfBlock {
    pub spectrum: [u8; SPECTRUM_SIZE],
    pub span: u32,
    pub res: u32,
    pub center: u32,
    pub pga: u8,
    pub reserved1: [u8; 3],
}

impl MonSpanRfBlock {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(272);
        wtr.extend_from_slice(&self.spectrum);
        wtr.write_u32::<LittleEndian>(self.span).unwrap();
        wtr.write_u32::<LittleEndian>(self.res).unwrap();
        wtr.write_u32::<LittleEndian>(self.center).unwrap();
        wtr.push(self.pga);
        wtr.extend_from_slice(&self.reserved1);
        wtr
    }
}

#[derive(Debug, Clone)]
pub struct MonSpanPayload {
    pub version: u8,
    pub reserved0: [u8; 2],
    pub rf_blocks: Vec<MonSpanRfBlock>,
}

impl MonSpanPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(4 + self.rf_blocks.len() * 272);
        wtr.push(self.version);
        wtr.push(self.rf_blocks.len() as u8);
        wtr.extend_from_slice(&self.reserved0);
        for block in &self.rf_blocks {
            wtr.extend_from_slice(&block.to_bytes());
        }
        wtr
    }
}

fn mon_span_rf_block_strategy() -> impl Strategy<Value = MonSpanRfBlock> {
    (
        prop::array::uniform32(any::<u8>()).prop_flat_map(|arr| {
            // Create 256-byte array from 8x 32-byte arrays
            let mut spectrum = [0u8; SPECTRUM_SIZE];
            for (i, chunk) in arr.chunks(4).enumerate() {
                for (j, &byte) in chunk.iter().enumerate() {
                    if i * 4 + j < SPECTRUM_SIZE {
                        spectrum[i * 4 + j] = byte;
                    }
                }
            }
            Just(spectrum)
        }),
        any::<u32>(),
        1..=10000u32, // res must be > 0 for valid spectrum
        any::<u32>(),
        any::<u8>(),
        prop::array::uniform3(any::<u8>()),
    )
        .prop_map(
            |(spectrum, span, res, center, pga, reserved1)| MonSpanRfBlock {
                spectrum,
                span,
                res,
                center,
                pga,
                reserved1,
            },
        )
}

fn mon_span_payload_strategy() -> impl Strategy<Value = MonSpanPayload> {
    (
        Just(0x00u8), // version is always 0x00
        prop::array::uniform2(any::<u8>()),
        prop::collection::vec(mon_span_rf_block_strategy(), 1..=2),
    )
        .prop_map(|(version, reserved0, rf_blocks)| MonSpanPayload {
            version,
            reserved0,
            rf_blocks,
        })
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

pub fn ubx_mon_span_frame_strategy() -> impl Strategy<Value = (MonSpanPayload, Vec<u8>)> {
    mon_span_payload_strategy().prop_map(|payload_data| {
        let payload = payload_data.to_bytes();

        let class_id = 0x0a;
        let message_id = 0x31;
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

        (payload_data, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))] // Fewer cases due to large payloads
    #[test]
    fn test_parser_proto27_with_generated_mon_span_frames(
        (expected, frame) in ubx_mon_span_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonSpan(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-SPAN packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.num_rf_blocks(), expected.rf_blocks.len() as u8);

        let parsed_blocks: Vec<_> = p.rf_blocks().collect();
        prop_assert_eq!(parsed_blocks.len(), expected.rf_blocks.len());

        for (parsed, expected_block) in parsed_blocks.iter().zip(expected.rf_blocks.iter()) {
            prop_assert_eq!(parsed.spectrum_raw(), &expected_block.spectrum);
            prop_assert_eq!(parsed.span, expected_block.span);
            prop_assert_eq!(parsed.res, expected_block.res);
            prop_assert_eq!(parsed.center, expected_block.center);
            prop_assert_eq!(parsed.pga, expected_block.pga);
        }
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn test_parser_proto31_with_generated_mon_span_frames(
        (expected, frame) in ubx_mon_span_frame_strategy()
    ) {
        use ublox::proto31::{Proto31, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::MonSpan(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-SPAN packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.num_rf_blocks(), expected.rf_blocks.len() as u8);

        let parsed_blocks: Vec<_> = p.rf_blocks().collect();
        prop_assert_eq!(parsed_blocks.len(), expected.rf_blocks.len());

        for (parsed, expected_block) in parsed_blocks.iter().zip(expected.rf_blocks.iter()) {
            prop_assert_eq!(parsed.spectrum_raw(), &expected_block.spectrum);
            prop_assert_eq!(parsed.span, expected_block.span);
            prop_assert_eq!(parsed.res, expected_block.res);
            prop_assert_eq!(parsed.center, expected_block.center);
            prop_assert_eq!(parsed.pga, expected_block.pga);
        }
    }
}
