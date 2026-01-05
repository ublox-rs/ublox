//! MON-RXBUF: Receiver Buffer Status
//!
//! Reports receiver buffer usage statistics per target.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Number of targets (ports) reported in MON-RXBUF
const NUM_TARGETS: usize = 6;

/// Receiver Buffer Status
///
/// Reports the number of bytes pending in the receiver buffer for each target,
/// as well as buffer usage statistics (current period and peak).
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    /// Number of bytes pending in receiver buffer for each target
    #[ubx(map_type = &[u16], from = pending_from_bytes, is_valid = pending_is_valid, get_as_ref)]
    pending: [u8; 12],

    /// Maximum buffer usage during the last sysmon period for each target (%)
    #[ubx(map_type = &[u8], from = usage_from_bytes, is_valid = usage_is_valid, get_as_ref)]
    usage: [u8; 6],

    /// Maximum buffer usage for each target (%)
    #[ubx(map_type = &[u8], from = peak_usage_from_bytes, is_valid = peak_usage_is_valid, get_as_ref)]
    peak_usage: [u8; 6],
}

fn pending_from_bytes(bytes: &[u8]) -> &[u16] {
    let ptr = bytes.as_ptr() as *const u16;
    unsafe { core::slice::from_raw_parts(ptr, NUM_TARGETS) }
}

#[allow(dead_code, reason = "Used by ubx_packet_recv macro for validation")]
fn pending_is_valid(bytes: &[u8]) -> bool {
    bytes.len() == NUM_TARGETS * 2
}

fn usage_from_bytes(bytes: &[u8]) -> &[u8] {
    bytes
}

#[allow(dead_code, reason = "Used by ubx_packet_recv macro for validation")]
fn usage_is_valid(bytes: &[u8]) -> bool {
    bytes.len() == NUM_TARGETS
}

fn peak_usage_from_bytes(bytes: &[u8]) -> &[u8] {
    bytes
}

#[allow(dead_code, reason = "Used by ubx_packet_recv macro for validation")]
fn peak_usage_is_valid(bytes: &[u8]) -> bool {
    bytes.len() == NUM_TARGETS
}
