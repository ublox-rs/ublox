//! SEC-SIG: Signal Security Status
//!
//! Provides real-time jamming and spoofing detection status.
//! Critical for safety-critical autonomous systems.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Signal Security Status
///
/// Provides real-time jamming and spoofing detection status from
/// the receiver's built-in interference detection capabilities.
/// Essential for safety-critical GNSS applications.
#[ubx_packet_recv]
#[ubx(class = 0x27, id = 0x09, max_payload_len = 1028)] // 4 + jamNumCentFreqs*4 (max UBX payload length)
struct SecSig {
    /// Message version
    version: u8,

    /// Signal security flags
    #[ubx(map_type = SecSigFlags)]
    sig_sec_flags: u8,

    /// Reserved
    reserved0: u8,

    /// The number of center frequencies we provide jamming information for
    jam_num_cent_freqs: u8,

    /// Jamming state of signals sharing a given center frequency (repeated jam_num_cent_freqs times)
    #[ubx(map_type = SecSigJamStateCentFreqIter, may_fail,
          from = SecSigJamStateCentFreqIter::new,
          is_valid = SecSigJamStateCentFreqIter::is_valid)]
    jam_state_cent_freqs: [u8; 0],
}

/// Jamming state for a given center frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SecSigJamStateCentFreq {
    /// Center frequency in kHz (floored to nearest kHz)
    pub cent_freq_khz: u32,
    /// Flag indicates whether signals on the given center frequency are considered jammed
    pub jammed: bool,
}

impl From<u32> for SecSigJamStateCentFreq {
    fn from(value: u32) -> Self {
        Self {
            cent_freq_khz: value & 0x00ff_ffff,
            jammed: (value & 0x0100_0000) != 0,
        }
    }
}

/// Iterator for SEC-SIG jam state entries.
#[derive(Debug, Clone)]
pub struct SecSigJamStateCentFreqIter<'d> {
    data: &'d [u8],
    offset: usize,
}

impl<'d> SecSigJamStateCentFreqIter<'d> {
    fn new(data: &'d [u8]) -> Self {
        Self { data, offset: 0 }
    }

    #[allow(
        dead_code,
        reason = "Used by ubx_packet_recv macro for validation, but may appear unused in some feature configurations"
    )]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 4 == 0
    }
}

impl core::iter::Iterator for SecSigJamStateCentFreqIter<'_> {
    type Item = SecSigJamStateCentFreq;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 4)?;
        let raw = u32::from_le_bytes(chunk.try_into().ok()?);
        self.offset += 4;
        Some(SecSigJamStateCentFreq::from(raw))
    }
}

/// Jamming detection state
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

/// Spoofing detection state
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpoofingState {
    /// Unknown or feature disabled
    Unknown = 0,
    /// OK - no spoofing indicated
    Ok = 1,
    /// Indicated - spoofing indicated by single source
    Indicated = 2,
    /// Multiple - spoofing indicated by multiple sources
    Multiple = 3,
}

/// Flags from SEC-SIG message
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SecSigFlags {
    /// Jamming detection is enabled
    pub jam_det_enabled: bool,
    /// Spoofing detection is enabled
    pub spf_det_enabled: bool,
    /// Current jamming state
    pub jamming_state: JammingState,
    /// Current spoofing state
    pub spoofing_state: SpoofingState,
}

impl From<u8> for SecSigFlags {
    fn from(value: u8) -> Self {
        Self {
            jam_det_enabled: (value & 0x01) != 0,
            spf_det_enabled: (value & 0x08) != 0,
            jamming_state: JammingState::from((value >> 1) & 0x03),
            spoofing_state: SpoofingState::from((value >> 4) & 0x07),
        }
    }
}
