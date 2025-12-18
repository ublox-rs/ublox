//! A proptest generator for U-Blox MON-IO messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-IO message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2};

/// Represents a single I/O port block in a MON-IO message (20 bytes).
#[derive(Debug, Clone)]
pub struct MonIoPort {
    pub rx_bytes: u32,
    pub tx_bytes: u32,
    pub parity_errs: u16,
    pub framing_errs: u16,
    pub overrun_errs: u16,
    pub break_cond: u16,
    pub reserved0: [u8; 4],
}

impl MonIoPort {
    /// Serializes this port block into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(20);
        wtr.write_u32::<LittleEndian>(self.rx_bytes).unwrap();
        wtr.write_u32::<LittleEndian>(self.tx_bytes).unwrap();
        wtr.write_u16::<LittleEndian>(self.parity_errs).unwrap();
        wtr.write_u16::<LittleEndian>(self.framing_errs).unwrap();
        wtr.write_u16::<LittleEndian>(self.overrun_errs).unwrap();
        wtr.write_u16::<LittleEndian>(self.break_cond).unwrap();
        wtr.extend_from_slice(&self.reserved0);
        wtr
    }
}

/// A proptest strategy for generating a single MonIoPort.
fn mon_io_port_strategy() -> impl Strategy<Value = MonIoPort> {
    (
        any::<u32>(),                       // rx_bytes
        any::<u32>(),                       // tx_bytes
        any::<u16>(),                       // parity_errs
        any::<u16>(),                       // framing_errs
        any::<u16>(),                       // overrun_errs
        any::<u16>(),                       // break_cond
        prop::array::uniform4(any::<u8>()), // reserved0
    )
        .prop_map(
            |(
                rx_bytes,
                tx_bytes,
                parity_errs,
                framing_errs,
                overrun_errs,
                break_cond,
                reserved0,
            )| {
                MonIoPort {
                    rx_bytes,
                    tx_bytes,
                    parity_errs,
                    framing_errs,
                    overrun_errs,
                    break_cond,
                    reserved0,
                }
            },
        )
}

/// A proptest strategy for generating a vector of MonIoPort blocks.
/// Typical receivers have 6 ports, but we test with 1-6 ports.
fn mon_io_payload_strategy() -> impl Strategy<Value = Vec<MonIoPort>> {
    prop::collection::vec(mon_io_port_strategy(), 1..=6)
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
/// containing a MON-IO message, along with the source port data.
///
/// Returns a tuple of `(Vec<MonIoPort>, Vec<u8>)`.
pub fn ubx_mon_io_frame_strategy() -> impl Strategy<Value = (Vec<MonIoPort>, Vec<u8>)> {
    mon_io_payload_strategy().prop_map(|ports| {
        // Serialize all port blocks
        let mut payload = Vec::new();
        for port in &ports {
            payload.extend_from_slice(&port.to_bytes());
        }

        let class_id = 0x0a;
        let message_id = 0x02;
        let length = payload.len() as u16;

        // Build the frame core (class, id, length, payload)
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

        (ports, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_io_frames(
        (expected_ports, frame) in ubx_mon_io_frame_strategy()
    ) {
        use ublox::{proto27::{Proto27, PacketRef}, ParserBuilder, UbxPacket};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonIo(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-IO valid packet");
        };

        // Verify parsed ports match expected
        let parsed_ports: Vec<_> = p.ports().collect();
        prop_assert_eq!(parsed_ports.len(), expected_ports.len());

        for (parsed, expected) in parsed_ports.iter().zip(expected_ports.iter()) {
            prop_assert_eq!(parsed.rx_bytes, expected.rx_bytes);
            prop_assert_eq!(parsed.tx_bytes, expected.tx_bytes);
            prop_assert_eq!(parsed.parity_errs, expected.parity_errs);
            prop_assert_eq!(parsed.framing_errs, expected.framing_errs);
            prop_assert_eq!(parsed.overrun_errs, expected.overrun_errs);
            prop_assert_eq!(parsed.break_cond, expected.break_cond);
        }
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_io_frames(
        (expected_ports, frame) in ubx_mon_io_frame_strategy()
    ) {
        use ublox::{proto14::{Proto14, PacketRef}, ParserBuilder, UbxPacket};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonIo(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-IO valid packet");
        };

        // Verify parsed ports match expected
        let parsed_ports: Vec<_> = p.ports().collect();
        prop_assert_eq!(parsed_ports.len(), expected_ports.len());

        for (parsed, expected) in parsed_ports.iter().zip(expected_ports.iter()) {
            prop_assert_eq!(parsed.rx_bytes, expected.rx_bytes);
            prop_assert_eq!(parsed.tx_bytes, expected.tx_bytes);
            prop_assert_eq!(parsed.parity_errs, expected.parity_errs);
            prop_assert_eq!(parsed.framing_errs, expected.framing_errs);
            prop_assert_eq!(parsed.overrun_errs, expected.overrun_errs);
            prop_assert_eq!(parsed.break_cond, expected.break_cond);
        }
    }
}
