//! RXM-COR: Differential Correction Status
//!
//! Provides status information about received differential corrections.
//! Critical for RTK/PPP monitoring and SPARTN/SSR support.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Differential Correction Status
///
/// Provides status information about received differential corrections.
/// Important for RTK/PPP monitoring and verifying correction source/quality.
#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x34, fixed_payload_len = 12)]
struct RxmCor {
    /// Message version (0x01 for this version)
    version: u8,

    /// Eb/N0 (signal quality), 0.125 dB/LSB, 0 = unknown
    #[ubx(map_type = f32, scale = 0.125)]
    ebno: u8,

    /// Reserved
    reserved0: [u8; 2],

    /// Status information bitfield
    #[ubx(map_type = RxmCorStatusInfo)]
    status_info: u32,

    /// Correction message type
    msg_type: u16,

    /// Correction message subtype
    msg_sub_type: u16,
}

/// Status information from RXM-COR
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RxmCorStatusInfo {
    /// Input correction data protocol
    pub protocol: CorrectionProtocol,
    /// Error status of received correction message content
    pub err_status: CorrectionErrStatus,
    /// Status of receiver using the input message
    pub msg_used: CorrectionMsgUsed,
    /// Identifier for the correction stream
    pub correction_id: u16,
    /// Validity of the msg_type field
    pub msg_type_valid: bool,
    /// Validity of the msg_sub_type field
    pub msg_sub_type_valid: bool,
    /// Input handling support of the input message
    pub msg_input_handle: MsgInputHandle,
    /// Encryption status of the input message
    pub msg_encrypted: MsgEncrypted,
    /// Decryption status of the input message
    pub msg_decrypted: MsgDecrypted,
}

impl From<u32> for RxmCorStatusInfo {
    fn from(value: u32) -> Self {
        Self {
            protocol: CorrectionProtocol::from((value & 0x1f) as u8),
            err_status: CorrectionErrStatus::from(((value >> 5) & 0x03) as u8),
            msg_used: CorrectionMsgUsed::from(((value >> 7) & 0x03) as u8),
            correction_id: ((value >> 9) & 0xffff) as u16,
            msg_type_valid: (value & (1 << 25)) != 0,
            msg_sub_type_valid: (value & (1 << 26)) != 0,
            msg_input_handle: MsgInputHandle::from(((value >> 27) & 0x01) as u8),
            msg_encrypted: MsgEncrypted::from(((value >> 28) & 0x03) as u8),
            msg_decrypted: MsgDecrypted::from(((value >> 30) & 0x03) as u8),
        }
    }
}

/// Input correction data protocol
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CorrectionProtocol {
    Unknown = 0,
    Rtcm3 = 1,
    Spartn = 2,
    UbxRxmPmp = 29,
    UbxRxmQzssl6 = 30,
}

/// Error status of the received correction message content
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CorrectionErrStatus {
    Unknown = 0,
    ErrorFree = 1,
    Erroneous = 2,
}

/// Status of receiver using the input message
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CorrectionMsgUsed {
    Unknown = 0,
    NotUsed = 1,
    Used = 2,
}

/// Input handling support of the input message
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MsgInputHandle {
    NoSupport = 0,
    Supported = 1,
}

/// Encryption status of the input message
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MsgEncrypted {
    Unknown = 0,
    NotEncrypted = 1,
    Encrypted = 2,
}

/// Decryption status of the input message
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MsgDecrypted {
    Unknown = 0,
    NotDecrypted = 1,
    Decrypted = 2,
}
