#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 16)]
struct MgaGpsIono {
    /// Message type: 0x06 for this type
    msg_type: u8,
    /// Message version: 0x00 for this version
    version: u8,
    reserved1: [u8; 2],
    /// Ionospheric parameter alpha0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-30
    alpha0: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-27
    alpha1: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha2: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha3: i8,
    /// Ionospheric parameter beta0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-11
    beta0: i8,
    /// Ionospheric parameter beta0 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-14
    beta1: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta2: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta3: i8,
    reserved2: [u8; 4],
}
