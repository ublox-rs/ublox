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

/// UBX-MGA-GAL TIMEOFFSET frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x02, fixed_payload_len = 12)]
struct MgaGalTime {
    /// Message type. 0x01 for this type.
    msg_type: u8,

    /// Message version.
    version: u8,

    /// Reserved
    reserved1: [u8; 2],

    /// Constant term of the polynomial describing the offset
    #[ubx(map_type = f64, scale = 2e-35)]
    a0g: i16,

    /// Rate of change of the offset
    #[ubx(map_type = f64, scale = 2e-51)]
    a1g: i16,

    /// GGTO reference time in seconds
    #[ubx(map_type = f64, scale = 3600.0)]
    t0g: u8,
    
    /// GGTO reference week number
    #[ubx(map_type = f64, scale = 1.0)]
    wn0g: u8,

    /// Reserved
    reserved2: [u8; 2],
}
