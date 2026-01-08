use super::common::NavRelPosNedFlags;

#[cfg(feature = "serde")]
use super::super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x3c, fixed_payload_len = 40)]
struct NavRelPosNed {
    /// Message version (0x00 for this version)
    version: u8,

    reserved1: u8,

    /// Reference station ID. Must be in the range 0..4095
    ref_station_id: u16,

    /// GPS Millisecond time of week of the navigation epoch.
    itow: u32,

    /// North component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_n_cm)]
    rel_pos_n: i32,

    /// East component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_e_cm)]
    rel_pos_e: i32,

    /// Down component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_d_cm)]
    rel_pos_d: i32,

    /// High-precision North component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full North component of relative position vector in cm = rel_pos_n + (rel_pos_hpn * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_n_mm)]
    rel_pos_hpn: i8,

    /// High-precision East component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full East component of relative position vector in cm = rel_pos_e + (rel_pos_hpe * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_e_mm)]
    rel_pos_hpe: i8,

    /// High-precision Down component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full Down component of relative position vector in cm = rel_pos_d + (rel_pos_hpd * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_d_mm)]
    rel_pos_hpd: i8,

    reserved2: u8,
    /// Accuracy of relative position North component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_n_mm)]
    acc_n: u32,

    /// Accuracy of relative position East component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_e_mm)]
    acc_e: u32,

    /// Accuracy of relative position Down component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_d_mm)]
    acc_d: u32,

    #[ubx(map_type = NavRelPosNedFlags)]
    flags: u32,
}
