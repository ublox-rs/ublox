//! MON-TXBUF: Transmitter Buffer Status
//!
//! Reports transmitter buffer usage statistics per target.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Number of targets (ports) reported in MON-TXBUF
const NUM_TARGETS: usize = 6;

/// Transmitter Buffer Status
///
/// Reports the number of bytes pending in the transmitter buffer for each target,
/// buffer usage statistics, and error flags.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x08, fixed_payload_len = 28)]
struct MonTxbuf {
    /// Number of bytes pending in transmitter buffer for each target
    #[ubx(map_type = &[u16], from = pending_from_bytes, is_valid = pending_is_valid, get_as_ref)]
    pending: [u8; 12],

    /// Maximum buffer usage during the last sysmon period for each target (%)
    #[ubx(map_type = &[u8], from = usage_from_bytes, is_valid = usage_is_valid, get_as_ref)]
    usage: [u8; 6],

    /// Maximum buffer usage for each target (%)
    #[ubx(map_type = &[u8], from = peak_usage_from_bytes, is_valid = peak_usage_is_valid, get_as_ref)]
    peak_usage: [u8; 6],

    /// Maximum usage of transmitter buffer during last sysmon period for all targets (%)
    t_usage: u8,

    /// Maximum usage of transmitter buffer for all targets (%)
    t_peak_usage: u8,

    /// Error flags
    #[ubx(map_type = MonTxbufErrors)]
    errors: u8,

    /// Reserved
    reserved0: u8,
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

/// Error flags for MON-TXBUF
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonTxbufErrors(u8);

impl MonTxbufErrors {
    /// Returns a bitmask indicating which targets have reached their buffer limit.
    /// Bit N corresponds to target N (bits 0-5 for 6 targets).
    pub fn limit(&self) -> u8 {
        self.0 & 0x3F
    }

    /// Returns true if the specified target (0-5) has reached its buffer limit.
    pub fn limit_reached(&self, target: usize) -> bool {
        target < NUM_TARGETS && (self.0 & (1 << target)) != 0
    }

    /// Returns true if a memory allocation error occurred.
    pub fn mem(&self) -> bool {
        self.0 & 0x40 != 0
    }

    /// Returns true if an allocation error occurred (TX buffer full).
    pub fn alloc(&self) -> bool {
        self.0 & 0x80 != 0
    }

    /// Returns the raw error byte.
    pub fn raw(&self) -> u8 {
        self.0
    }
}

impl From<u8> for MonTxbufErrors {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
