use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, GnssFixType, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

/// Navigation Solution Information
#[ubx_packet_recv]
#[ubx(class = 1, id = 6, fixed_payload_len = 52)]
struct NavSol {
    /// GPS Millisecond Time of Week
    itow: u32,

    /// Fractional part of iTOW (range: +/-500000).
    ftow_ns: i32,

    /// GPS week number of the navigation epoch
    week: i16,

    /// GPS fix Type
    #[ubx(map_type = GnssFixType)]
    fix_type: u8,

    /// Navigation Status Flags
    #[ubx(map_type = NavSolFlags)]
    flags: u8,

    /// ECEF X coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_x: i32,

    /// ECEF Y coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_y: i32,

    /// ECEF Z coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_z: i32,

    /// 3D Position Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-2)]
    position_accuracy_estimate: u32,

    /// ECEF X velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_vx: i32,

    /// ECEF Y velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_vy: i32,

    /// ECEF Z velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_vz: i32,

    /// Speed Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-2)]
    speed_accuracy_estimate: u32,

    /// Position DOP
    #[ubx(map_type = f32, scale = 1e-2)]
    pdop: u16,
    reserved1: u8,

    /// Number of SVs used in Nav Solution
    num_sv: u8,
    reserved2: [u8; 4],
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Navigation Status Flags
    #[derive(Debug)]
    pub struct NavSolFlags: u8 {
        /// position and velocity valid and within DOP and ACC Masks
        const GPS_FIX_OK = 1;
        /// DGPS used
        const DIFF_SOLN = 2;
        /// Week Number valid
        const WKN_SET = 4;
        /// Time of Week valid
        const TOW_SET = 8;
    }
}
