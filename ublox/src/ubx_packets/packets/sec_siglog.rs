//! SEC-SIGLOG: Signal Security Event Log
//!
//! Provides a log of security-related events (jamming/spoofing detections).
//! Complements SEC-SIG with historical event data.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Signal Security Event Log
///
/// Provides a log of security-related events (jamming/spoofing detections).
/// Works with SEC-SIG for complete security monitoring.
#[ubx_packet_recv]
#[ubx(class = 0x27, id = 0x10, max_payload_len = 136)] // 8 + 8 * 16 events max
struct SecSiglog {
    /// Message version (0x01 for this version)
    version: u8,

    /// Number of events in log
    num_events: u8,

    /// Reserved
    reserved0: [u8; 6],

    /// Event log entries (repeated num_events times)
    #[ubx(map_type = SecSiglogEventIter, may_fail,
          from = SecSiglogEventIter::new,
          is_valid = SecSiglogEventIter::is_valid)]
    events: [u8; 0],
}

/// A single security event from the log
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SecSiglogEvent {
    /// Seconds elapsed since this event
    pub time_elapsed_s: u32,
    /// Type of spoofing/jamming detection
    pub detection_type: u8,
    /// Type of the event
    pub event_type: u8,
}

/// Iterator for SEC-SIGLOG events
#[derive(Debug, Clone)]
pub struct SecSiglogEventIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> SecSiglogEventIter<'a> {
    /// Construct iterator from raw event payload bytes.
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 8 bytes.
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 8 == 0
    }
}

impl core::iter::Iterator for SecSiglogEventIter<'_> {
    type Item = SecSiglogEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 8)?;

        let event = SecSiglogEvent {
            time_elapsed_s: u32::from_le_bytes(chunk[0..4].try_into().ok()?),
            detection_type: chunk[4],
            event_type: chunk[5],
            // bytes 6..8 are reserved
        };

        self.offset += 8;
        Some(event)
    }
}
