#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Extended Hardware Status
///
/// Deprecated after protocol version 23, use `UBX-MON-HW3` and `UBX-MON-RF` instead.
///
/// Status of different aspects of the hardware such as Imbalance, Low-Level Configuration and POST Results.
/// The first four parameters of this message represent the complex signal from the RF front end. The following
/// rules of thumb apply:
/// • The smaller the absolute value of the variable ofs_i and ofs_q, the better.
/// • Ideally, the magnitude of the I-part (mag_i) and the Q-part (mag_q) of the complex signal should be the
/// same.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x0b, fixed_payload_len = 28)]
struct MonHw2 {
    /// Imbalance of I-part of complex signal.
    ///
    /// scaled (-128 = max. negative imbalance, 127 = max. positive imbalance)
    ofs_i: i8,
    /// Magnitude of I-part of complex signal.
    ///
    /// scaled (0 = no signal, 255 = max. magnitude)
    mag_i: u8,
    /// Imbalance of Q-part of complex signal
    ///
    /// scaled (-128 = max. negative imbalance, 127 = max. positive imbalance)
    ofs_q: i8,
    /// Magnitude of Q-part of complex signal
    ///
    /// scaled (0 = no signal, 255 = max. magnitude)
    mag_q: u8,
    /// Source of low-level configuration
    #[ubx(map_type = ConfigSource)]
    cfg_source: u8,
    /// Reserved bytes
    reserved0: [u8; 3],
    /// Low-level configuration (obsolete for protocol versions greater than 15.00)
    low_lev_cfg: u32,
    /// Reserved bytes
    reserved1: [u8; 8],
    /// POST (Power On Self Test) status word
    post_status: u32,
    /// Reserved bytes
    reserved2: [u8; 4],
}

/// Source of low-level configuration
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// Configuration source undefined (default for unknown values)
    Undefined = 0,
    /// Configuration source is flash image
    Flash = 102,
    /// Configuration source is OTP (One-Time Programmable)
    Otp = 111,
    /// Configuration source is config pins
    ConfigPins = 112,
    /// Configuration source is ROM
    Rom = 114,
}
