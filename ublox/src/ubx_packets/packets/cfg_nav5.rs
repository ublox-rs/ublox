use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, ubx_packets::packets::ScaleBack, MemWriter, MemWriterError,
    UbxPacketCreator, UbxPacketMeta, UtcStandardIdentifier,
};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv_send};

/// Navigation Engine Settings
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x24,
    fixed_payload_len = 36,
    flags = "default_for_builder"
)]
struct CfgNav5 {
    /// Only the masked parameters will be applied
    #[ubx(map_type = CfgNav5Params)]
    mask: u16,
    #[ubx(map_type = NavDynamicModel, may_fail)]
    dyn_model: u8,
    #[ubx(map_type = NavFixMode, may_fail)]
    fix_mode: u8,

    /// Fixed altitude (mean sea level) for 2D fixmode [m]
    #[ubx(map_type = f64, scale = 0.01)]
    fixed_alt: i32,

    /// Fixed altitude variance for 2D mode (m^2)
    #[ubx(map_type = f64, scale = 0.0001)]
    fixed_alt_var: u32,

    /// Minimum Elevation for a GNSS satellite to be used in NAV (deg)
    min_elev_degrees: i8,

    /// Reserved
    dr_limit: u8,

    /// Position DOP Mask to use
    #[ubx(map_type = f32, scale = 0.1)]
    pdop: u16,

    /// Time DOP Mask to use
    #[ubx(map_type = f32, scale = 0.1)]
    tdop: u16,

    /// Position Accuracy Mask [m]
    pacc: u16,

    /// Time Accuracy Mask
    /// according to manual unit is "m", but this looks like typo
    tacc: u16,

    /// Static hold threshold
    #[ubx(map_type = f32, scale = 0.01)]
    static_hold_thresh: u8,

    /// DGNSS timeout (seconds)
    dgps_time_out: u8,

    /// Number of satellites required to have
    /// C/N0 above `cno_thresh` for a fix to be attempted
    cno_thresh_num_svs: u8,

    /// C/N0 threshold for deciding whether toattempt a fix (dBHz)
    cno_thresh: u8,
    reserved1: [u8; 2],

    /// Static hold distance threshold (beforequitting static hold)
    static_hold_max_dist: u16,

    /// UTC standard to be used
    #[ubx(map_type = UtcStandardIdentifier, may_fail)]
    utc_standard: u8,
    reserved2: [u8; 5],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNav5` parameters bitmask
    #[derive(Default, Debug, PartialEq, Eq)]
    pub struct CfgNav5Params: u16 {
        /// Apply dynamic model settings
        const DYN = 1;
        /// Apply minimum elevation settings
        const MIN_EL = 2;
        /// Apply fix mode settings
       const POS_FIX_MODE = 4;
        /// Reserved
        const DR_LIM = 8;
        /// position mask settings
       const POS_MASK_APPLY = 0x10;
        /// Apply time mask settings
        const TIME_MASK = 0x20;
        /// Apply static hold settings
        const STATIC_HOLD_MASK = 0x40;
        /// Apply DGPS settings
        const DGPS_MASK = 0x80;
        /// Apply CNO threshold settings (cnoThresh, cnoThreshNumSVs)
        const CNO_THRESHOLD = 0x100;
        /// Apply UTC settings (not supported in protocol versions less than 16)
        const UTC = 0x400;
    }
}

/// Dynamic platform model
#[non_exhaustive]
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NavDynamicModel {
    #[default]
    Portable = 0,
    Stationary = 2,
    Pedestrian = 3,
    Automotive = 4,
    Sea = 5,
    AirborneWithLess1gAcceleration = 6,
    AirborneWithLess2gAcceleration = 7,
    AirborneWithLess4gAcceleration = 8,
    #[cfg(any(
        feature = "ubx_proto27",
        feature = "ubx_proto31",
        feature = "ubx_proto33",
    ))]
    /// not supported in protocol versions less than 18
    WristWornWatch = 9,
    #[cfg(any(feature = "ubx_proto31", feature = "ubx_proto33"))]
    /// supported in protocol versions 19.2
    Bike = 10,
    #[cfg(feature = "ubx_proto33")]
    Mower = 11,
    #[cfg(feature = "ubx_proto33")]
    EScooter = 12,
    #[cfg(feature = "ubx_proto33")]
    Rail = 13,
}

/// Position Fixing Mode
#[derive(Default)] // default needs to be derived before ubx_extend
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NavFixMode {
    /// 2D only
    Only2D = 1,
    /// 3D only
    Only3D = 2,
    #[default]
    /// Auto 2D/3D
    Auto2D3D = 3,
}
