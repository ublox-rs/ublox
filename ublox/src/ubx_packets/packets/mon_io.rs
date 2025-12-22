//! MON-IO: I/O Subsystem Status
//!
//! Provides I/O port statistics including byte counts and error statistics.
//! This message is deprecated; use UBX-MON-COMMS instead for newer receivers.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// I/O Subsystem Status
///
/// Provides I/O port statistics including byte counts and error statistics.
/// The number of ports varies by receiver (typically 6 on u-blox 5/6/7/8).
///
/// This message is deprecated. Use UBX-MON-COMMS instead for newer receivers.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x02, max_payload_len = 200)] // 20 * 10 ports max
struct MonIo {
    /// Port information blocks (repeated N times, 20 bytes each)
    #[ubx(map_type = MonIoPortIter, may_fail,
          from = MonIoPortIter::new,
          is_valid = MonIoPortIter::is_valid)]
    ports: [u8; 0],
}

/// Information for a single I/O port
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonIoPort {
    /// Number of bytes ever received
    pub rx_bytes: u32,
    /// Number of bytes ever sent
    pub tx_bytes: u32,
    /// Number of 100ms timeslots with parity errors
    pub parity_errs: u16,
    /// Number of 100ms timeslots with framing errors
    pub framing_errs: u16,
    /// Number of 100ms timeslots with overrun errors
    pub overrun_errs: u16,
    /// Number of 100ms timeslots with break conditions
    pub break_cond: u16,
}

/// Iterator for MON-IO port blocks
#[derive(Debug, Clone)]
pub struct MonIoPortIter<'d> {
    data: &'d [u8],
    offset: usize,
}

impl<'d> MonIoPortIter<'d> {
    /// Construct iterator from raw port block payload bytes.
    fn new(data: &'d [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 20 bytes.
    #[allow(
        dead_code,
        reason = "Used by ubx_packet_recv macro for validation, but may appear unused in some feature configurations"
    )]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 20 == 0
    }
}

impl core::iter::Iterator for MonIoPortIter<'_> {
    type Item = MonIoPort;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 20)?;

        let port = MonIoPort {
            rx_bytes: u32::from_le_bytes(chunk[0..4].try_into().ok()?),
            tx_bytes: u32::from_le_bytes(chunk[4..8].try_into().ok()?),
            parity_errs: u16::from_le_bytes(chunk[8..10].try_into().ok()?),
            framing_errs: u16::from_le_bytes(chunk[10..12].try_into().ok()?),
            overrun_errs: u16::from_le_bytes(chunk[12..14].try_into().ok()?),
            break_cond: u16::from_le_bytes(chunk[14..16].try_into().ok()?),
            // bytes 16..20 are reserved0
        };

        self.offset += 20;
        Some(port)
    }
}
