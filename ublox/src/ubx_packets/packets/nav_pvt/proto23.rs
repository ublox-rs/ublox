#[cfg(feature = "serde")]
use super::super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::common::*;
use crate::{error::ParserError, GnssFixType, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Navigation Position Velocity Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x07, fixed_payload_len = 92)]
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

    /// Additional flags, see [NavPvtFlags2]
    #[ubx(map_type = NavPvtFlags2)]
    flags2: u8,

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
    g_speed: i32,

    /// Heading of motion 2-D \[deg\]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_motion)]
    head_motion: i32,

    /// Speed Accuracy Estimate \[m/s\]
    #[ubx(map_type = f64, scale = 1e-3, alias = speed_accuracy)]
    s_acc: u32,

    /// Heading accuracy estimate (for both vehicle and motion) [deg]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_accuracy)]
    head_acc: u32,

    /// Position DOP
    #[ubx(map_type = f64, scale = 1e-2)]
    pdop: u16,

    /// Additional flags
    #[ubx(map_type = flags::NavPvtFlags3)]
    flags3: u16,

    reserved1: [u8; 4],

    /// Heading of vehicle (2-D), this is only valid when [HEAD_VEH_VALID](NavPvtFlags::HEAD_VEH_VALID) is set,
    /// otherwise the output is set to the heading of motion
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_vehicle)]
    head_vehicle: i32,

    /// Magnetic declination. Only supported in ADR 4.10 and later.
    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination)]
    magnetic_declination: i16,

    /// Magnetic declination accuracy. Only supported in ADR 4.10 and later.
    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_accuracy)]
    magnetic_declination_accuracy: u16,
}

pub(crate) mod flags {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct NavPvtFlags3 {
        invalid_llh: bool,
        last_correction_age: u8,
    }

    impl NavPvtFlags3 {
        /// 1 = Invalid lon, lat, height and hMSL
        pub fn invalid_llh(&self) -> bool {
            self.invalid_llh
        }

        /// Age of the most recently received differential correction
        ///
        /// Values:
        /// - `0`: Not available
        /// - `1`: Age between 0 and 1 second
        /// - `2`: Age between 1 (inclusive) and 2 seconds
        /// - `3`: Age between 2 (inclusive) and 5 seconds
        /// - `4`: Age between 5 (inclusive) and 10 seconds
        /// - `5`: Age between 10 (inclusive) and 15 seconds
        /// - `6`: Age between 15 (inclusive) and 20 seconds
        /// - `7`: Age between 20 (inclusive) and 30 seconds
        /// - `8`: Age between 30 (inclusive) and 45 seconds
        /// - `9`: Age between 45 (inclusive) and 60 seconds
        /// - `10`: Age between 60 (inclusive) and 90 seconds
        /// - `11`: Age between 90 (inclusive) and 120 seconds
        /// - `>=12`: Age greater or equal than 120 seconds
        pub fn last_correction_age(&self) -> u8 {
            self.last_correction_age
        }
    }

    impl From<u16> for NavPvtFlags3 {
        fn from(val: u16) -> Self {
            const LAST_CORRECTION_AGE_MASK: u16 = 0b0000_0000_0001_1110;
            let invalid_llh = val & 0x01 == 1;
            let last_correction_age = ((val & LAST_CORRECTION_AGE_MASK) >> 1) as u8; // bits 1–4
            Self {
                invalid_llh,
                last_correction_age,
            }
        }
    }
}
