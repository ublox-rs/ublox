#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// UBX-MGA-BDS UTC frame.
#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x03, fixed_payload_len = 20)]
struct MgaBdsUtc {
    /// Message type. 0x01 for this type.
    msg_type: u8,

    /// Message version.
    version: u8,

    /// Reserved
    reserved1: [u8; 2],

    /// BDT-UTC (in seconds)
    #[ubx(map_type = f64, scale = 2e-30)]
    utc_a0: i32,

    /// BDT-UTC rate of change (in s/second)
    #[ubx(map_type = f64, scale = 2e-50)]
    utc_a1: i32,

    /// Delta time due leap seconds before the new leap second
    /// is effective
    dt_ls: i8,

    /// Reserved
    reserved2: u8,

    /// BeiDou week number of reception of this
    /// UTC parameter set (8-bit truncated)
    wn_rec: u8,

    /// Week number of the new leap second
    wn_lsf: u8,

    /// Day number of the new leap second
    dn: u8,

    /// Delta time due to leap seconds after the new
    /// leap second is effective
    dt_lsf: i8,

    /// Reserved
    reserved3: [u8; 2],
}
