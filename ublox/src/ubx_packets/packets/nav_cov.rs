#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Position/Velocity Covariance Matrix Solution (NED frame)
///
/// Provides full 3×3 covariance matrices for position and velocity
/// in the local NED (North-East-Down) frame. Essential for sensor fusion
/// and safety-critical applications requiring proper uncertainty quantification.
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x36, fixed_payload_len = 64)]
struct NavCov {
    /// GPS time of week (ms)
    itow: u32,

    /// Message version (0 for this version)
    version: u8,

    /// Position covariance valid flag (0 = invalid, 1 = valid)
    pos_cov_valid: u8,

    /// Velocity covariance valid flag (0 = invalid, 1 = valid)
    vel_cov_valid: u8,

    /// Reserved bytes
    reserved0: [u8; 9],

    /// Position covariance North-North (m²)
    pos_cov_nn: f32,

    /// Position covariance North-East (m²)
    pos_cov_ne: f32,

    /// Position covariance North-Down (m²)
    pos_cov_nd: f32,

    /// Position covariance East-East (m²)
    pos_cov_ee: f32,

    /// Position covariance East-Down (m²)
    pos_cov_ed: f32,

    /// Position covariance Down-Down (m²)
    pos_cov_dd: f32,

    /// Velocity covariance North-North (m²/s²)
    vel_cov_nn: f32,

    /// Velocity covariance North-East (m²/s²)
    vel_cov_ne: f32,

    /// Velocity covariance North-Down (m²/s²)
    vel_cov_nd: f32,

    /// Velocity covariance East-East (m²/s²)
    vel_cov_ee: f32,

    /// Velocity covariance East-Down (m²/s²)
    vel_cov_ed: f32,

    /// Velocity covariance Down-Down (m²/s²)
    vel_cov_dd: f32,
}
