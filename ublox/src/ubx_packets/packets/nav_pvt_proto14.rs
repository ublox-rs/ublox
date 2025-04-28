#![cfg(feature = "ubx_proto14")]
use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

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

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Fix status flags for `NavPvt`
    #[derive(Debug)]
    pub struct NavPvtFlags: u8 {
        /// Position and velocity valid and within DOP and ACC Masks
        const GPS_FIX_OK = 1;
        /// Differential corrections were applied; DGPS used
        const DIFF_SOLN = 2;
        /// Heading of vehicle is valid
        const HEAD_VEH_VALID = 0x20;
        const CARR_SOLN_FLOAT = 0x40;
        const CARR_SOLN_FIXED = 0x80;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Additional flags for `NavPvt`
    #[derive(Debug)]
    pub struct NavPvtFlags2: u8 {
        /// 1 = information about UTC Date and Time of Day validity confirmation
        /// is available. This flag is only supported in Protocol Versions
        /// 19.00, 19.10, 20.10, 20.20, 20.30, 22.00, 23.00, 23.01,27 and 28.
        const CONFIRMED_AVAI = 0x20;
        /// 1 = UTC Date validity could be confirmed
        /// (confirmed by using an additional independent source)
        const CONFIRMED_DATE = 0x40;
        /// 1 = UTC Time of Day could be confirmed
        /// (confirmed by using an additional independent source)
        const CONFIRMED_TIME = 0x80;
    }
}

#[cfg(feature = "ubx_proto23")]
pub(crate) mod flags {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct NavPvtFlags3 {
        invalid_llh: bool,
        age_differential_correction: u8,
    }

    impl NavPvtFlags3 {
        pub fn invalid_llh(&self) -> bool {
            self.invalid_llh
        }

        pub fn age_differential_correction(&self) -> u8 {
            self.age_differential_correction
        }
    }

    impl From<u8> for NavPvtFlags3 {
        fn from(val: u8) -> Self {
            const AGE_DIFFERENTIAL_CORRECTION_MASK: u8 = 0b11110;
            let invalid = val & 0x01 == 1;
            // F9R interface description document specifies that this byte is unused
            // We can read it ... but we don't expose it
            let age_differential_correction = val & AGE_DIFFERENTIAL_CORRECTION_MASK;
            Self {
                invalid_llh: invalid,
                age_differential_correction,
            }
        }
    }
}
