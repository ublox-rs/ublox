#![cfg(any(
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// RF information
///
/// This message contains information for each RF block.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x38, max_payload_len = 1028)] // 4 bytes header + 42 blocks * 24 bytes
pub struct MonRf {
    /// Message version (0x00 for this version)
    version: u8,
    /// The number of RF blocks included
    n_blocks: u8,
    /// Reserved bytes
    reserved0: [u8; 2],
    /// RF block information (repeated n_blocks times)
    #[ubx(map_type = RfBlockIter, may_fail,
          from = RfBlockIter::new,
          is_valid = RfBlockIter::is_valid)]
    blocks: [u8; 0],
}

/// Jamming/Interference Monitor State
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JammingState {
    /// Unknown or feature disabled
    Unknown = 0,
    /// OK - no significant jamming
    Ok = 1,
    /// Warning - interference visible but fix is OK
    Warning = 2,
    /// Critical - interference visible and no fix
    Critical = 3,
}

/// Antenna Supervisor State Machine Status
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaStatus {
    /// Status is INIT
    Init = 0,
    /// Status is DONTKNOW
    DontKnow = 1,
    /// Status is OK
    Ok = 2,
    /// Status is SHORT
    Short = 3,
    /// Status is OPEN
    Open = 4,
}

/// Antenna Power Status
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaPowerStatus {
    /// Power is OFF
    Off = 0,
    /// Power is ON
    On = 1,
    /// Power is DONTKNOW
    DontKnow = 2,
}

/// Flags for an RF block
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Flags {
    /// Output from Jamming/Interference Monitor
    pub jamming_state: JammingState,
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self {
            // bits 1..0
            jamming_state: JammingState::from(value & 0x03),
        }
    }
}

/// Information for a single RF block
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RfBlock {
    /// RF block ID
    pub block_id: u8,
    /// Flags
    pub flags: Flags,
    /// Status of the antenna supervisor state machine
    pub ant_status: AntennaStatus,
    /// Current power status of antenna
    pub ant_power: AntennaPowerStatus,
    /// POST (Power On Self Test) status word
    pub post_status: u32,
    /// Reserved bytes
    pub reserved1: [u8; 4],
    /// Noise level as measured by the GPS core
    pub noise_per_ms: u16,
    /// AGC Monitor (counts SIGHI xor SIGLO, range 0 to 8191)
    pub agc_cnt: u16,
    /// CW jamming indicator, scaled (0=no CW jamming, 255=strong CW jamming)
    pub jam_ind: u8,
    /// Imbalance of I-part of complex signal, scaled
    pub ofs_i: i8,
    /// Magnitude of I-part of complex signal, scaled
    pub mag_i: u8,
    /// Imbalance of Q-part of complex signal, scaled
    pub ofs_q: i8,
    /// Magnitude of Q-part of complex signal, scaled
    pub mag_q: u8,
    /// Reserved bytes
    pub reserved2: [u8; 3],
}

/// Iterator for RF block information
#[derive(Debug, Clone)]
pub struct RfBlockIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> RfBlockIter<'a> {
    /// Construct iterator from raw RF block payload bytes.
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 24 bytes.
    fn is_valid(payload: &[u8]) -> bool {
        payload.len().is_multiple_of(24)
    }
}

impl core::iter::Iterator for RfBlockIter<'_> {
    type Item = RfBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 24)?;

        let block = RfBlock {
            block_id: chunk[0],
            flags: Flags::from(chunk[1]),
            ant_status: AntennaStatus::from(chunk[2]),
            ant_power: AntennaPowerStatus::from(chunk[3]),
            post_status: u32::from_le_bytes(chunk[4..8].try_into().ok()?),
            reserved1: chunk[8..12].try_into().ok()?,
            noise_per_ms: u16::from_le_bytes(chunk[12..14].try_into().ok()?),
            agc_cnt: u16::from_le_bytes(chunk[14..16].try_into().ok()?),
            jam_ind: chunk[16],
            ofs_i: chunk[17] as i8,
            mag_i: chunk[18],
            ofs_q: chunk[19] as i8,
            mag_q: chunk[20],
            reserved2: chunk[21..24].try_into().ok()?,
        };

        // Only advance the offset after a successful parse.
        self.offset += 24;
        Some(block)
    }
}
