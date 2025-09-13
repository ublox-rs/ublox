#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError,
    ubx_checksum,
    ubx_packets::{packets::ScaleBack, UbxChecksumCalc},
    MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};

use ublox_derive::ubx_packet_recv_send;

/// UBX-MGA-BDS IONO frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x03, fixed_payload_len = 16)]
struct MgaBdsIono {
    /// Message type: 0x06 for this type
    msg_type: u8,

    /// Message version: 0x00 for this version
    version: u8,

    /// Reserved
    reserved1: [u8; 2],

    /// Ionospheric parameter alpha0 (in seconds)
    #[ubx(map_type = f64, scale = 2.0e-30)]
    alpha0: i8,

    /// Ionospheric parameter alpha1 (in seconds per radians)
    #[ubx(map_type = f64, scale = 2.0e-27)]
    alpha1: i8,

    /// Ionospheric parameter alpha1 (in seconds per squared radians)
    #[ubx(map_type = f64, scale = 2.0e-24)]
    alpha2: i8,

    /// Ionospheric parameter alpha1 (in seconds per cubic radians)
    #[ubx(map_type = f64, scale = 2.0e-24)]
    alpha3: i8,

    /// Ionospheric parameter beta0 (in seconds)
    #[ubx(map_type = f64, scale = 2.0e11)]
    beta0: i8,

    /// Ionospheric parameter beta0 (in seconds per radians)
    #[ubx(map_type = f64, scale = 2.0e14)]
    beta1: i8,

    /// Ionospheric parameter beta0 (in seconds per squared radians)
    #[ubx(map_type = f64, scale = 2.0E16)]
    beta2: i8,

    /// Ionospheric parameter beta0 (in seconds per cubic radians)
    #[ubx(map_type = f64, scale = 2.0e16)]
    beta3: i8,

    /// Reserved
    reserved2: [u8; 4],
}
