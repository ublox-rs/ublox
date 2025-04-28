use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError,
    ubx_checksum,
    ubx_packets::{packets::ScaleBack, UbxChecksumCalc},
    MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv_send};

/// Time MODE2 Config Frame (32.10.36.1)
/// only available on `timing` receivers
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x3d,
    fixed_payload_len = 28,
    flags = "default_for_builder"
)]
struct CfgTmode2 {
    /// Time transfer modes, see [CfgTModeModes] for details
    #[ubx(map_type = CfgTModeModes, may_fail)]
    time_transfer_mode: u8,
    reserved1: u8,
    #[ubx(map_type = CfgTmode2Flags)]
    flags: u16,
    /// WGS84 ECEF.x coordinate in [m] or latitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_x_or_lat: i32,
    /// WGS84 ECEF.y coordinate in [m] or longitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_y_or_lon: i32,
    /// WGS84 ECEF.z coordinate or altitude, both in [m],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_z_or_alt: i32,
    /// Fixed position 3D accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    fixed_pos_acc: u32,
    /// Survey in minimum duration in [s]
    survey_in_min_duration: u32,
    /// Survey in position accuracy limit in [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    survey_in_accur_limit: u32,
}

/// Time transfer modes (32.10.36)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum CfgTModeModes {
    #[default]
    Disabled = 0,
    SurveyIn = 1,
    /// True Antenna Reference Point (ARP) position information required
    /// when using `Fixed`
    Fixed = 2,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode2Flags :u16 {
        /// Position given in LAT/LON/ALT
        /// default being WGS84 ECEF
        const LLA = 0x01;
        /// In case LLA was set, Altitude value is not valid
        const ALT_INVALID = 0x02;
    }
}
