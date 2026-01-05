//! MON-MSGPP: Message Parse and Process Status
//!
//! Reports message parsing statistics per port and protocol.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Number of I/O ports reported in MON-MSGPP
pub const NUM_PORTS: usize = 6;
/// Number of protocols per port
pub const NUM_PROTOCOLS: usize = 8;

/// Message Parse and Process Status
///
/// Reports the number of successfully parsed messages for each protocol
/// on each I/O port, as well as the number of skipped bytes per port.
///
/// Protocol indices (0-7): UBX, NMEA, RTCM2, RTCM3, SPARTN, and reserved.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x06, fixed_payload_len = 120)]
struct MonMsgpp {
    /// Message counts for port 0 (one per protocol, raw bytes)
    msg1: [u8; 16],

    /// Message counts for port 1 (one per protocol, raw bytes)
    msg2: [u8; 16],

    /// Message counts for port 2 (one per protocol, raw bytes)
    msg3: [u8; 16],

    /// Message counts for port 3 (one per protocol, raw bytes)
    msg4: [u8; 16],

    /// Message counts for port 4 (one per protocol, raw bytes)
    msg5: [u8; 16],

    /// Message counts for port 5 (one per protocol, raw bytes)
    msg6: [u8; 16],

    /// Number of skipped bytes for each port (raw bytes)
    skipped: [u8; 24],
}

/// Helper function to convert raw bytes to u16 array
pub fn parse_port_msg(bytes: &[u8; 16]) -> [u16; NUM_PROTOCOLS] {
    let mut result = [0u16; NUM_PROTOCOLS];
    for i in 0..NUM_PROTOCOLS {
        result[i] = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
    }
    result
}

/// Helper function to convert raw bytes to u32 array
pub fn parse_skipped(bytes: &[u8; 24]) -> [u32; NUM_PORTS] {
    let mut result = [0u32; NUM_PORTS];
    for i in 0..NUM_PORTS {
        result[i] = u32::from_le_bytes([
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]);
    }
    result
}
