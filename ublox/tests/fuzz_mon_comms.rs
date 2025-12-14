#![cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]

//! A proptest generator for U-Blox MON-COMMS messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-COMMS message. The generated data is
//! structurally correct, including sync words, class/ID, length,
//! a randomized valid payload with repeating port blocks, and a correct checksum.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

const SYNC_CHAR_1: u8 = 0xB5;
const SYNC_CHAR_2: u8 = 0x62;

/// Represents a single 40-byte port block within a MON-COMMS message.
#[derive(Debug, Clone)]
pub struct MonCommsPortPayload {
    pub port_id: u16,      // Port identifier (see u-blox docs)
    pub tx_pending: u16,   // Bytes pending in TX buffer
    pub tx_bytes: u32,     // Total bytes transmitted
    pub tx_usage: u8,      // TX buffer usage (0-255)
    pub tx_peak_usage: u8, // Peak TX buffer usage (0-255)
    pub rx_pending: u16,   // Bytes pending in RX buffer
    pub rx_bytes: u32,     // Total bytes received
    pub rx_usage: u8,      // RX buffer usage (0-255)
    pub rx_peak_usage: u8, // Peak RX buffer usage (0-255)
    pub overrun_errs: u16, // Overrun errors
    pub msgs: [u16; 4],    // Message counts per protocol
    pub skipped: u32,      // Skipped bytes
}

impl MonCommsPortPayload {
    /// Serializes the MonCommsPortPayload into a 40-byte block.
    pub fn to_bytes(&self, wtr: &mut Vec<u8>) {
        wtr.write_u16::<LittleEndian>(self.port_id).unwrap();
        wtr.write_u16::<LittleEndian>(self.tx_pending).unwrap();
        wtr.write_u32::<LittleEndian>(self.tx_bytes).unwrap();
        wtr.write_u8(self.tx_usage).unwrap();
        wtr.write_u8(self.tx_peak_usage).unwrap();
        wtr.write_u16::<LittleEndian>(self.rx_pending).unwrap();
        wtr.write_u32::<LittleEndian>(self.rx_bytes).unwrap();
        wtr.write_u8(self.rx_usage).unwrap();
        wtr.write_u8(self.rx_peak_usage).unwrap();
        wtr.write_u16::<LittleEndian>(self.overrun_errs).unwrap();
        for m in &self.msgs {
            wtr.write_u16::<LittleEndian>(*m).unwrap();
        }
        // reserved1: 8 bytes
        wtr.extend_from_slice(&[0u8; 8]);
        wtr.write_u32::<LittleEndian>(self.skipped).unwrap();
    }
}

/// Represents the payload of a UBX-MON-COMMS message.
///
/// MON-COMMS payload is 8 + nPorts * 40 bytes.
#[derive(Debug, Clone)]
pub struct MonCommsPayload {
    pub version: u8,                     // Message version (0x00 for this version)
    pub n_ports: u8,                     // Number of ports
    pub tx_errors: u8,                   // TX error bitmask
    pub reserved0: u8,                   // Reserved
    pub prot_ids: [u8; 4],               // Protocol identifiers
    pub ports: Vec<MonCommsPortPayload>, // Repeated port blocks
}

impl MonCommsPayload {
    /// Serializes the MonCommsPayload into a vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(8 + (self.ports.len() * 40));
        wtr.write_u8(self.version).unwrap();
        wtr.write_u8(self.n_ports).unwrap();
        wtr.write_u8(self.tx_errors).unwrap();
        wtr.write_u8(self.reserved0).unwrap();
        wtr.extend_from_slice(&self.prot_ids);
        for p in &self.ports {
            p.to_bytes(&mut wtr);
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

fn port_id_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![Just(0u16), Just(1u16), Just(2u16), Just(3u16), Just(5u16)]
}

/// A proptest strategy for generating a single `MonCommsPortPayload`.
fn mon_comms_port_strategy() -> impl Strategy<Value = MonCommsPortPayload> {
    // Split into smaller tuples to avoid proptest tuple size limits
    let header_and_tx = (
        port_id_strategy(),
        any::<u16>(),
        any::<u32>(),
        any::<u8>(),
        any::<u8>(),
    );

    let rx_and_err = (
        any::<u16>(),
        any::<u32>(),
        any::<u8>(),
        any::<u8>(),
        any::<u16>(),
    );

    let msgs_and_skipped = (
        any::<u16>(),
        any::<u16>(),
        any::<u16>(),
        any::<u16>(),
        any::<u32>(),
    );

    (header_and_tx, rx_and_err, msgs_and_skipped).prop_map(
        |(
            (port_id, tx_pending, tx_bytes, tx_usage, tx_peak_usage),
            (rx_pending, rx_bytes, rx_usage, rx_peak_usage, overrun_errs),
            (msgs0, msgs1, msgs2, msgs3, skipped),
        )| {
            MonCommsPortPayload {
                port_id,
                tx_pending,
                tx_bytes,
                tx_usage,
                tx_peak_usage,
                rx_pending,
                rx_bytes,
                rx_usage,
                rx_peak_usage,
                overrun_errs,
                msgs: [msgs0, msgs1, msgs2, msgs3],
                skipped,
            }
        },
    )
}

/// A proptest strategy for generating a `MonCommsPayload`.
fn mon_comms_payload_strategy() -> impl Strategy<Value = MonCommsPayload> {
    (
        Just(0u8),
        any::<u8>(),
        any::<u8>(),
        Just(0u8),
        prop::array::uniform4(any::<u8>()),
        prop::collection::vec(mon_comms_port_strategy(), 0..=6),
    )
        .prop_map(
            |(version, _n_ports, tx_errors, reserved0, prot_ids, mut ports)| {
                // Keep header n_ports consistent with number of repeated blocks.
                let n_ports = ports.len().min(6) as u8;
                ports.truncate(n_ports as usize);
                MonCommsPayload {
                    version,
                    n_ports,
                    tx_errors,
                    reserved0,
                    prot_ids,
                    ports,
                }
            },
        )
}

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a MON-COMMS message, along with the source payload struct.
pub fn ubx_mon_comms_frame_strategy() -> impl Strategy<Value = (MonCommsPayload, Vec<u8>)> {
    mon_comms_payload_strategy().prop_map(|payload_struct| {
        let payload = payload_struct.to_bytes();
        let class_id = 0x0A;
        let message_id = 0x36;
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

        (payload_struct, final_frame)
    })
}

// Proptest will run this test case many times with different generated frames.
#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_comms_frames((expected, frame) in ubx_mon_comms_frame_strategy()) {
        use ublox::proto27::{PacketRef, Proto27};
        use ublox::mon_comms::PortId;

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<4096>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonComms(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-COMMS valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_ports(), expected.ports.len() as u8);
        prop_assert_eq!(p.ports().count(), expected.ports.len());

        let mut parsed_ports = p.ports();
        for expected_port in &expected.ports {
            let parsed_port = parsed_ports.next().unwrap();
            prop_assert_eq!(parsed_port.port_id, PortId::from(expected_port.port_id));
            prop_assert_eq!(parsed_port.tx_pending, expected_port.tx_pending);
            prop_assert_eq!(parsed_port.tx_bytes, expected_port.tx_bytes);
            prop_assert_eq!(parsed_port.tx_usage, expected_port.tx_usage);
            prop_assert_eq!(parsed_port.tx_peak_usage, expected_port.tx_peak_usage);
            prop_assert_eq!(parsed_port.rx_pending, expected_port.rx_pending);
            prop_assert_eq!(parsed_port.rx_bytes, expected_port.rx_bytes);
            prop_assert_eq!(parsed_port.rx_usage, expected_port.rx_usage);
            prop_assert_eq!(parsed_port.rx_peak_usage, expected_port.rx_peak_usage);
            prop_assert_eq!(parsed_port.overrun_errs, expected_port.overrun_errs);
            prop_assert_eq!(parsed_port.msgs, expected_port.msgs);
            prop_assert_eq!(parsed_port.skipped, expected_port.skipped);
        }
    }
}

#[cfg(feature = "ubx_proto31")]
proptest! {
    #[test]
    fn test_parser_proto31_with_generated_mon_comms_frames((expected, frame) in ubx_mon_comms_frame_strategy()) {
        use ublox::proto31::{PacketRef, Proto31};
        use ublox::mon_comms::PortId;

        let mut parser = ParserBuilder::new().with_protocol::<Proto31>().with_fixed_buffer::<4096>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto31(PacketRef::MonComms(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-COMMS valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_ports(), expected.ports.len() as u8);
        prop_assert_eq!(p.ports().count(), expected.ports.len());

        let mut parsed_ports = p.ports();
        for expected_port in &expected.ports {
            let parsed_port = parsed_ports.next().unwrap();
            prop_assert_eq!(parsed_port.port_id, PortId::from(expected_port.port_id));
            prop_assert_eq!(parsed_port.tx_pending, expected_port.tx_pending);
            prop_assert_eq!(parsed_port.tx_bytes, expected_port.tx_bytes);
            prop_assert_eq!(parsed_port.tx_usage, expected_port.tx_usage);
            prop_assert_eq!(parsed_port.tx_peak_usage, expected_port.tx_peak_usage);
            prop_assert_eq!(parsed_port.rx_pending, expected_port.rx_pending);
            prop_assert_eq!(parsed_port.rx_bytes, expected_port.rx_bytes);
            prop_assert_eq!(parsed_port.rx_usage, expected_port.rx_usage);
            prop_assert_eq!(parsed_port.rx_peak_usage, expected_port.rx_peak_usage);
            prop_assert_eq!(parsed_port.overrun_errs, expected_port.overrun_errs);
            prop_assert_eq!(parsed_port.msgs, expected_port.msgs);
            prop_assert_eq!(parsed_port.skipped, expected_port.skipped);
        }
    }
}
