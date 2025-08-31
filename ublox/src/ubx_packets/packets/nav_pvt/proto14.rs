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
    /// GPS Millisecond time of week of the navigation epoch.
    ///
    /// Messages with the same iTOW value can be assumed to have come from the same navigation solution.
    ///
    /// # Note
    ///
    /// iTOW values may not be valid (i.e. they may have been generated with insufficient
    /// conversion data) and therefore it is not recommended to use the iTOW field for any other purpose.
    itow: u32,

    /// Year (UTC)
    year: u16,
    /// Month, range 1..12 (UTC)
    month: u8,
    /// Day of month, range 1..31 (UTC)
    day: u8,
    /// Hour of day, range 0..23 (UTC)
    hour: u8,
    /// Minute of hour, range 0..59 (UTC)
    min: u8,
    /// Seconds of minute, range 0..60 (UTC)
    sec: u8,

    /// Validity flags, see [NavPvtValidFlags]
    #[ubx(map_type = NavPvtValidFlags)]
    valid: u8,

    /// Time accuracy estimate in nanoseconds (UTC)
    time_accuracy: u32,

    /// Fraction of second, range -1e9 .. 1e9 (UTC)
    nanosec: i32,

    /// GNSS Fix Type, see [GnssFixType]
    #[ubx(map_type = GnssFixType)]
    fix_type: u8,

    /// Fix status flags, see [NavPvtFlags]
    #[ubx(map_type = NavPvtFlags)]
    flags: u8,

    reserved1: u8,

    /// Number of satellites used in Nav Solution
    num_satellites: u8,

    /// Longitude in \[deg\]
    #[ubx(map_type = f64, scale = 1e-7, alias = longitude)]
    lon: i32,

    /// Latitude in \[deg\]
    #[ubx(map_type = f64, scale = 1e-7, alias = latitude)]
    lat: i32,

    /// Height above reference ellipsoid in \[m\]
    #[ubx(map_type = f64, scale = 1e-3, alias = height_above_ellipsoid)]
    height: i32,

    /// Height above Mean Sea Level in \[m\]
    #[ubx(map_type = f64, scale = 1e-3)]
    height_msl: i32,

    /// Horizontal accuracy in \[m\]
    #[ubx(map_type = f64, scale = 1e-3, alias = horizontal_accuracy )]
    h_acc: u32,

    /// Vertical accuracy in \[m\]
    #[ubx(map_type = f64, scale = 1e-3, alias = vertical_accuracy )]
    v_acc: u32,

    /// Velocity North component \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_north: i32,

    /// Velocity East component \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_east: i32,

    /// Velocity Down component \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_down: i32,

    /// Ground speed \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3, alias = ground_speed_2d)]
    g_speed: u32,

    /// Heading of motion 2-D \[deg\]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_motion)]
    head_motion: i32,

    /// Speed Accuracy Estimate \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3, alias = speed_accuracy)]
    s_acc: u32,

    /// Heading accuracy estimate (for both vehicle and motion) \[deg\]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_accuracy)]
    head_acc: u32,

    /// Position DOP
    #[ubx(map_type = f64, scale = 1e-2)]
    pdop: u16,

    reserved2: [u8; 2],
    reserved3: [u8; 4],
}
