#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Position Solution in ECEF
///
/// This message provides the position solution in Earth-Centered
/// Earth-Fixed (ECEF) coordinates.
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x01, fixed_payload_len = 20)]
struct NavPosEcef {
    /// GPS time of week (ms)
    itow: u32,

    /// ECEF X coordinate
    ///
    /// Raw UBX payload unit: centimeters (cm).
    /// This crate exposes this value as meters (m) via `scale = 1e-2`.
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_x_meters: i32,

    /// ECEF Y coordinate
    ///
    /// Raw UBX payload unit: centimeters (cm).
    /// This crate exposes this value as meters (m) via `scale = 1e-2`.
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_y_meters: i32,

    /// ECEF Z coordinate
    ///
    /// Raw UBX payload unit: centimeters (cm).
    /// This crate exposes this value as meters (m) via `scale = 1e-2`.
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_z_meters: i32,

    /// Position accuracy estimate
    ///
    /// Raw UBX payload unit: centimeters (cm).
    /// This crate exposes this value as meters (m) via `scale = 1e-2`.
    #[ubx(map_type = f64, scale = 1e-2)]
    p_acc_meters: u32,
}
