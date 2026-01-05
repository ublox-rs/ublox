//! MON-RXR: Receiver Status Information
//!
//! Reports receiver wake/sleep status after power state changes.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Receiver Status Information
///
/// This message is sent when the receiver changes from or to backup mode.
/// It indicates whether the receiver is currently awake or in backup mode.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x21, fixed_payload_len = 1)]
struct MonRxr {
    /// Receiver status flags
    #[ubx(map_type = MonRxrFlags)]
    flags: u8,
}

/// Receiver status flags for MON-RXR
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonRxrFlags(u8);

impl MonRxrFlags {
    /// Returns true if the receiver is awake (not in backup mode)
    pub fn awake(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// Returns the raw flags byte
    pub fn raw(&self) -> u8 {
        self.0
    }
}

impl From<u8> for MonRxrFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
