use super::common::NavRelPosNedFlags;

#[cfg(feature = "serde")]
use crate::{serde::ser::SerializeMap, SerializeUbxPacketFields};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x3c, fixed_payload_len = 64)]
struct NavRelPosNed {
    /// Message version (0x01 for this version)
    version: u8,

    _reserved0: u8,

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
    /// Length of the relative position vector
    #[ubx(map_type = f64, alias = rel_pos_length_cm)]
    rel_pos_length: i32,
    /// Heading of the relative position vector
    #[ubx(map_type = f64, scale = 1e-5, alias = rel_pos_heading_degrees)]
    rel_pos_heading: i32,

    _reserved1: u32,

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
    /// High-precision Length of the relative position vector.
    /// Must be in the range -99 to +99.
    /// Full Length of the relative position vector in cm = rel_pos_length + (rel_pos_hp_length * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_length_mm)]
    rel_pos_hp_length: i8,

    /// Accuracy of relative position North component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_n_mm)]
    acc_n: u32,
    /// Accuracy of relative position East component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_e_mm)]
    acc_e: u32,
    /// Accuracy of relative position Down component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_d_mm)]
    acc_d: u32,
    /// Accuracy of Length of relative position vector
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_length_mm)]
    acc_length: u32,
    /// Accuracy of Heading of relative position vector
    #[ubx(map_type = f64, scale = 1e-5, alias = acc_heading_degrees)]
    acc_heading: u32,

    _reserved2: u32,

    #[ubx(map_type = NavRelPosNedFlags)]
    flags: u32,
}
