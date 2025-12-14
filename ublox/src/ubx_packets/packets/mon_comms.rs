//! MON-COMMS: Communication Port Status
//!
//! Provides detailed status of all communication ports including
//! buffer usage, data flow statistics, and protocol information.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Communication Port Status
///
/// Provides detailed status of all communication ports including
/// data flow statistics and buffer monitoring.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x36, max_payload_len = 248)] // 8 + 40 * 6 ports max
struct MonComms {
    /// Message version (0x00 for this version)
    version: u8,

    /// Number of ports included
    n_ports: u8,

    /// TX error bitmask
    tx_errors: u8,

    /// Reserved
    reserved0: u8,

    /// Protocol identifiers (indexed 0-3)
    /// 0=UBX, 1=NMEA, 2=RTCM3, 5=SPARTN
    prot_ids: [u8; 4],

    /// Port information blocks (repeated n_ports times)
    #[ubx(map_type = MonCommsPortIter, may_fail,
          from = MonCommsPortIter::new,
          is_valid = MonCommsPortIter::is_valid)]
    ports: [u8; 0],
}

/// Port identifier values
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PortId {
    /// I2C (DDC)
    I2c,
    /// UART1
    Uart1,
    /// UART2
    Uart2,
    /// USB
    Usb,
    /// SPI
    Spi,
    /// Unknown port
    Unknown(u16),
}

impl From<u16> for PortId {
    fn from(value: u16) -> Self {
        match value {
            0 => PortId::I2c,
            1 => PortId::Uart1,
            2 => PortId::Uart2,
            3 => PortId::Usb,
            5 => PortId::Spi,
            other => PortId::Unknown(other),
        }
    }
}

/// Information for a single communication port
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonCommsPort {
    /// Port identifier
    pub port_id: PortId,
    /// Bytes pending in TX buffer
    pub tx_pending: u16,
    /// Total bytes transmitted
    pub tx_bytes: u32,
    /// TX buffer usage (0-255 = 0-100%)
    pub tx_usage: u8,
    /// Peak TX buffer usage
    pub tx_peak_usage: u8,
    /// Bytes pending in RX buffer
    pub rx_pending: u16,
    /// Total bytes received
    pub rx_bytes: u32,
    /// RX buffer usage (0-255 = 0-100%)
    pub rx_usage: u8,
    /// Peak RX buffer usage
    pub rx_peak_usage: u8,
    /// Number of overrun errors
    pub overrun_errs: u16,
    /// Message counts per protocol (indexed by protIds)
    pub msgs: [u16; 4],
    /// Skipped bytes
    pub skipped: u32,
}

/// Iterator for MON-COMMS port blocks
#[derive(Debug, Clone)]
pub struct MonCommsPortIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MonCommsPortIter<'a> {
    /// Construct iterator from raw port block payload bytes.
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 40 bytes.
    #[allow(dead_code)]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 40 == 0
    }
}

impl core::iter::Iterator for MonCommsPortIter<'_> {
    type Item = MonCommsPort;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 40)?;

        let port = MonCommsPort {
            port_id: PortId::from(u16::from_le_bytes(chunk[0..2].try_into().ok()?)),
            tx_pending: u16::from_le_bytes(chunk[2..4].try_into().ok()?),
            tx_bytes: u32::from_le_bytes(chunk[4..8].try_into().ok()?),
            tx_usage: chunk[8],
            tx_peak_usage: chunk[9],
            rx_pending: u16::from_le_bytes(chunk[10..12].try_into().ok()?),
            rx_bytes: u32::from_le_bytes(chunk[12..16].try_into().ok()?),
            rx_usage: chunk[16],
            rx_peak_usage: chunk[17],
            overrun_errs: u16::from_le_bytes(chunk[18..20].try_into().ok()?),
            msgs: [
                u16::from_le_bytes(chunk[20..22].try_into().ok()?),
                u16::from_le_bytes(chunk[22..24].try_into().ok()?),
                u16::from_le_bytes(chunk[24..26].try_into().ok()?),
                u16::from_le_bytes(chunk[26..28].try_into().ok()?),
            ],
            // bytes 28..36 are reserved
            skipped: u32::from_le_bytes(chunk[36..40].try_into().ok()?),
        };

        self.offset += 40;
        Some(port)
    }
}
