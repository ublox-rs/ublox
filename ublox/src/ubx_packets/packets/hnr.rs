#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::cfg_val::CfgVal;

use bitflags::bitflags;

use super::SerializeUbxPacketFields;

use ublox_derive::{ubx_extend, ubx_packet_send, ubx_packet_recv, ubx_extend_bitflags, ubx_packet_recv_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ScaleBack,
    ubx_checksum, MemWriter, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1,
    SYNC_CHAR_2,
    nav::NavBbrMask,
    GpsFix,
};

#[ubx_packet_recv]
#[ubx(class = 0x28, id = 0x01, fixed_payload_len = 32)]
struct HnrAtt {
    itow: u32,
    version: u8,
    reserved1: [u8; 3],
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_roll)]
    roll: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_pitch)]
    pitch: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_heading)]
    heading: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_roll_accuracy)]
    acc_roll: u32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_pitch_accuracy)]
    acc_pitch: u32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_heading_accuracy)]
    acc_heading: u32,
}

#[ubx_packet_recv]
#[ubx(class = 0x28, id = 0x02, fixed_payload_len = 36)]
pub struct HnrIns {
    #[ubx(map_type = HnrInsBitFlags)]
    bit_field: u32,
    reserved: [u8; 4],
    itow: u32,

    #[ubx(map_type = f64, scale = 1e-3, alias = x_angular_rate)]
    x_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = y_angular_rate)]
    y_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = z_angular_rate)]
    z_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = x_acceleration)]
    x_accel: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = y_acceleration)]
    y_accel: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = z_acceleration)]
    z_accel: i32,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct HnrInsBitFlags: u32 {
        const VERSION = 1;
        const X_ANG_RATE_VALID = 0x100;
        const Y_ANG_RATE_VALID = 0x200;
        const Z_ANG_RATE_VALID = 0x400;
        const X_ACCEL_VALID = 0x800;
        const Y_ACCEL_VALID = 0x1000;
        const Z_ACCEL_VALID = 0x2000;
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x28, id = 0x00, fixed_payload_len = 72)]
#[derive(Debug)]
struct HnrPvt {
    itow: u32,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,

    #[ubx(map_type = HnrPvtValidFlags)]
    valid: u8,
    nano: i32,
    #[ubx(map_type = GpsFix)]
    gps_fix: u8,

    #[ubx(map_type = HnrPvtFlags)]
    flags: u8,

    reserved1: [u8; 2],

    #[ubx(map_type = f64, scale = 1e-7, alias = longitude)]
    lon: i32,

    #[ubx(map_type = f64, scale = 1e-7, alias = latitude)]
    lat: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = height_above_ellipsoid)]
    height: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = height_msl)]
    height_msl: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = ground_speed_2d)]
    g_speed: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = speed_3d)]
    speed: i32,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_motion)]
    head_mot: i32,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_vehicle)]
    head_veh: i32,

    h_acc: u32,
    v_acc: u32,
    s_acc: u32,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_accurracy)]
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
