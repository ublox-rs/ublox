use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::common::*;
use crate::{error::ParserError, GnssFixType, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

/// Navigation Position Velocity Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x07, fixed_payload_len = 84)]
struct NavPvt {
    /// GPS Millisecond Time of Week
    itow: u32,

    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,

    valid: u8,
    time_accuracy: u32,

    nanosec: i32,

    /// GNSS Fix Type
    #[ubx(map_type = GnssFixType)]
    fix_type: u8,

    #[ubx(map_type = NavPvtFlags)]
    flags: u8,

    reserved1: u8,

    num_satellites: u8,

    /// Longitude in [deg]
    #[ubx(map_type = f64, scale = 1e-7, alias = longitude)]
    lon: i32,

    /// Latitude in [deg]
    #[ubx(map_type = f64, scale = 1e-7, alias = latitude)]
    lat: i32,

    /// Height above reference ellipsoid in [m]
    #[ubx(map_type = f64, scale = 1e-3, alias = height_above_ellipsoid)]
    height: i32,

    /// Height above Mean Sea Level in [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    height_msl: i32,

    /// Horizontal accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3, alias = horizontal_accuracy )]
    h_acc: u32,

    /// Vertical accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3, alias = vertical_accuracy )]
    v_acc: u32,

    /// Velocity North component [m/s]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_north: i32,

    /// Velocity East component [m/s]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_east: i32,

    /// Velocity Down component [m/s]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_down: i32,

    /// Ground speed [m/s]
    #[ubx(map_type = f64, scale = 1e-3, alias = ground_speed_2d)]
    g_speed: u32,

    /// Heading of motion 2-D [deg]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_motion)]
    head_motion: i32,

    /// Speed Accuracy Estimate [m/s]
    #[ubx(map_type = f64, scale = 1e-3, alias = speed_accuracy)]
    s_acc: u32,

    /// Heading accuracy estimate (for both vehicle and motion) [deg]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_accuracy)]
    head_acc: u32,

    /// Position DOP
    #[ubx(map_type = f64, scale = 1e-2)]
    pdop: u16,

    reserved2: [u8; 2],
    reserved3: [u8; 4],
}
