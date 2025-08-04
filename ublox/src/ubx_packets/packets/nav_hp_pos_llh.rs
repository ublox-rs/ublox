#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// High Precision Geodetic Position Solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x14, fixed_payload_len = 36)]
struct NavHpPosLlh {
    /// Message version (0 for protocol version 27)
    version: u8,

    reserved1: [u8; 2],

    #[ubx(map_type = flags::NavHpPosLlhFlags)]
    flags: u8,

    /// GPS Millisecond Time of Week
    itow: u32,

    /// Longitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    lon: i32,

    /// Latitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    lat: i32,

    /// Height above Ellipsoid [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    height_meters: i32,

    /// Height above mean sea level [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    height_msl: i32,

    /// High precision component of longitude
    /// Must be in the range -99..+99
    /// Precise longitude in deg * 1e-7 = lon + (lonHp * 1e-2)
    #[ubx(map_type = f64, scale = 1e-9, alias = lon_hp_degrees)]
    lon_hp: i8,

    /// High precision component of latitude
    /// Must be in the range -99..+99
    /// Precise latitude in deg * 1e-7 = lat + (latHp * 1e-2)
    #[ubx(map_type = f64, scale = 1e-9, alias = lat_hp_degrees)]
    lat_hp: i8,

    /// High precision component of height above ellipsoid
    /// Must be in the range -9..+9
    /// Precise height in mm = height + (heightHp * 0.1)
    #[ubx(map_type = f64, scale = 1e-1)]
    height_hp_meters: i8,

    /// High precision component of height above mean sea level
    /// Must be in range -9..+9
    /// Precise height in mm = hMSL + (hMSLHp * 0.1)
    #[ubx(map_type = f64, scale = 1e-1)]
    height_hp_msl: i8,

    /// Horizontal accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    horizontal_accuracy: u32,

    /// Vertical accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    vertical_accuracy: u32,
}

#[cfg(not(any(feature = "ubx_proto27", feature = "ubx_proto31")))]
pub(crate) mod flags {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct NavHpPosLlhFlags {}

    impl From<u8> for NavHpPosLlhFlags {
        fn from(_val: u8) -> Self {
            Self {}
        }
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
pub(crate) mod flags {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct NavHpPosLlhFlags {
        invalid_llh: bool,
    }

    impl NavHpPosLlhFlags {
        /// 1 = Invalid lon, lat, height, hMSL, lonHp, latHp, heightHp and hMSLHp
        pub fn invalid_llh(&self) -> bool {
            self.invalid_llh
        }
    }

    impl From<u8> for NavHpPosLlhFlags {
        fn from(val: u8) -> Self {
            let invalid = val & 0x01 == 1;
            Self {
                invalid_llh: invalid,
            }
        }
    }
}
