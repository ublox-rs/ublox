use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, GnssFixType, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

#[ubx_packet_recv]
#[ubx(class = 0x28, id = 0x00, fixed_payload_len = 72)]
#[derive(Debug)]
struct HnrPvt {
    /// GPS Millisecond Time of Week
    itow: u32,

    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,

    #[ubx(map_type = HnrPvtValidFlags)]
    valid: u8,

    nanosec: i32,

    #[ubx(map_type = GnssFixType)]
    fix_type: u8,

    #[ubx(map_type = HnrPvtFlags)]
    flags: u8,

    reserved1: [u8; 2],

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
    #[ubx(map_type = f64, scale = 1e-3, alias = height_msl)]
    height_msl: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = ground_speed_2d)]
    g_speed: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = speed_3d)]
    speed: i32,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_motion)]
    head_motion: i32,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_vehicle)]
    head_vehicle: i32,

    /// Horizontal accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3, alias = horizontal_accuracy )]
    h_acc: u32,

    /// Vertical accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3, alias = vertical_accuracy )]
    v_acc: u32,

    /// Speed accuracy in [m/s]
    #[ubx(map_type = f64, scale = 1e-3, alias = speed_accuracy )]
    s_acc: u32,

    /// Heading accuracy estimate (for both vehicle and motion) [deg]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_accuracy)]
    head_acc: u32,

    reserved2: [u8; 4],
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    /// Fix status flags for `HnrPvt`
    pub struct HnrPvtFlags: u8 {
        /// position and velocity valid and within DOP and ACC Masks
        const GPS_FIX_OK = 0x01;
        /// DGPS used
        const DIFF_SOLN = 0x02;
        /// 1 = heading of vehicle is valid
        const WKN_SET = 0x04;
        const TOW_SET = 0x08;
        const HEAD_VEH_VALID = 0x10;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct HnrPvtValidFlags: u8 {
        const VALID_DATE = 0x01;
        const VALID_TIME = 0x02;
        const FULLY_RESOLVED = 0x04;
    }
}
