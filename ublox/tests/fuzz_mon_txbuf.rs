//! A proptest generator for U-Blox MON-TXBUF messages.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

mod common;
use common::build_ubx_frame;

const NUM_TARGETS: usize = 6;

#[derive(Debug, Clone)]
pub struct MonTxbufPayload {
    pub pending: [u16; NUM_TARGETS],
    pub usage: [u8; NUM_TARGETS],
    pub peak_usage: [u8; NUM_TARGETS],
    pub t_usage: u8,
    pub t_peak_usage: u8,
    pub errors: u8,
    pub reserved0: u8,
}

impl MonTxbufPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(28);
        for &p in &self.pending {
            wtr.write_u16::<LittleEndian>(p).unwrap();
        }
        wtr.extend_from_slice(&self.usage);
        wtr.extend_from_slice(&self.peak_usage);
        wtr.push(self.t_usage);
        wtr.push(self.t_peak_usage);
        wtr.push(self.errors);
        wtr.push(self.reserved0);
        wtr
    }
}

fn mon_txbuf_payload_strategy() -> impl Strategy<Value = MonTxbufPayload> {
    (
        prop::array::uniform6(any::<u16>()),
        prop::array::uniform6(any::<u8>()),
        prop::array::uniform6(any::<u8>()),
        any::<u8>(),
        any::<u8>(),
        any::<u8>(),
        any::<u8>(),
    )
        .prop_map(
            |(pending, usage, peak_usage, t_usage, t_peak_usage, errors, reserved0)| {
                MonTxbufPayload {
                    pending,
                    usage,
                    peak_usage,
                    t_usage,
                    t_peak_usage,
                    errors,
                    reserved0,
                }
            },
        )
}

pub fn ubx_mon_txbuf_frame_strategy() -> impl Strategy<Value = (MonTxbufPayload, Vec<u8>)> {
    mon_txbuf_payload_strategy().prop_map(|payload_data| {
        let payload = payload_data.to_bytes();

        let final_frame = build_ubx_frame(0x0a, 0x08, &payload);

        (payload_data, final_frame)
    })
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_txbuf_frames(
        (expected, frame) in ubx_mon_txbuf_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<64>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonTxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-TXBUF packet");
        };

        for (i, &expected_val) in expected.pending.iter().enumerate() {
            prop_assert_eq!(p.pending()[i], expected_val, "pending[{}] mismatch", i);
        }
        for (i, &expected_val) in expected.usage.iter().enumerate() {
            prop_assert_eq!(p.usage()[i], expected_val, "usage[{}] mismatch", i);
        }
        for (i, &expected_val) in expected.peak_usage.iter().enumerate() {
            prop_assert_eq!(p.peak_usage()[i], expected_val, "peak_usage[{}] mismatch", i);
        }
        prop_assert_eq!(p.t_usage(), expected.t_usage);
        prop_assert_eq!(p.t_peak_usage(), expected.t_peak_usage);
        prop_assert_eq!(p.errors().raw(), expected.errors);
        prop_assert_eq!(p.errors().limit(), expected.errors & 0x3F);
        prop_assert_eq!(p.errors().mem(), expected.errors & 0x40 != 0);
        prop_assert_eq!(p.errors().alloc(), expected.errors & 0x80 != 0);
    }
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_txbuf_frames(
        (expected, frame) in ubx_mon_txbuf_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<64>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonTxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a valid MON-TXBUF packet");
        };

        for (i, &expected_val) in expected.pending.iter().enumerate() {
            prop_assert_eq!(p.pending()[i], expected_val, "pending[{}] mismatch", i);
        }
        for (i, &expected_val) in expected.usage.iter().enumerate() {
            prop_assert_eq!(p.usage()[i], expected_val, "usage[{}] mismatch", i);
        }
        for (i, &expected_val) in expected.peak_usage.iter().enumerate() {
            prop_assert_eq!(p.peak_usage()[i], expected_val, "peak_usage[{}] mismatch", i);
        }
        prop_assert_eq!(p.t_usage(), expected.t_usage);
        prop_assert_eq!(p.t_peak_usage(), expected.t_peak_usage);
        prop_assert_eq!(p.errors().raw(), expected.errors);
        prop_assert_eq!(p.errors().limit(), expected.errors & 0x3F);
        prop_assert_eq!(p.errors().mem(), expected.errors & 0x40 != 0);
        prop_assert_eq!(p.errors().alloc(), expected.errors & 0x80 != 0);
    }
}
