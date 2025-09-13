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

/// UBX-MGA-GPS UTC frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 20)]
struct MgaGpsUtc {
    /// Message type. 0x01 for this type.
    msg_type: u8,

    /// Message version.
    version: u8,

    /// Reserved
    reserved1: [u8; 2],

    /// First parameter of UTC polynomial (in seconds)
    #[ubx(map_type = f64, scale = 2e-30)]
    utc_a0: i32,

    /// Second parameter of UTC polynomial (in s/second)
    #[ubx(map_type = f64, scale = 2e-50)]
    utc_a1: i32,

    /// Delta time due to current leap seconds
    utc_dt_ls: i8,

    /// UTC parameter reference time of week
    utc_tot: u8,

    /// UTC parameters reference week number
    utc_wn_t: u8,

    /// Week number at the end of which the future leap
    /// second becomes effective (the 8-bit WNLSF field, in weeks)
    utc_wn_lsf: u8,

    /// Day number at the end of which the future leap
    /// second becomes effective (in days)
    utc_dn: u8,

    /// Delta time to future leap seconds (in seconds)
    utc_dt_lsf: u8,

    /// Reserved
    reserved2: [u8; 2],
}
