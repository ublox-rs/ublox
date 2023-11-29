use crate::cfg_val::CfgVal;
use core::convert::TryInto;
use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitflags::bitflags;
use chrono::prelude::*;
use num_traits::cast::{FromPrimitive, ToPrimitive};
use num_traits::float::FloatCore;

use ublox_derive::{
    define_recv_packets, ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_recv_send,
    ubx_packet_send,
};

use crate::error::{MemWriterError, ParserError};
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
use crate::ubx_packets::packets::mon_ver::is_cstr_valid;

use super::{
    ubx_checksum, MemWriter, Position, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta,
    UbxUnknownPacketRef, SYNC_CHAR_1, SYNC_CHAR_2,
};

/// Used to help serialize the packet's fields flattened within a struct containing the msg_id and class fields, but
/// without using the serde FlatMapSerializer which requires alloc.
#[cfg(feature = "serde")]
pub(crate) trait SerializeUbxPacketFields {
    fn serialize_fields<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeMap;
}

/// Geodetic Position Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 2, fixed_payload_len = 28)]
struct NavPosLlh {
    /// GPS Millisecond Time of Week
    itow: u32,

    /// Longitude
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    lon: i32,

    /// Latitude
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    lat: i32,

    /// Height above Ellipsoid
    #[ubx(map_type = f64, scale = 1e-3)]
    height_meters: i32,

    /// Height above mean sea level
    #[ubx(map_type = f64, scale = 1e-3)]
    height_msl: i32,

    /// Horizontal Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-3)]
    h_ack: u32,

    /// Vertical Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-3)]
    v_acc: u32,
}

/// Velocity Solution in NED
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x12, fixed_payload_len = 36)]
struct NavVelNed {
    /// GPS Millisecond Time of Week
    itow: u32,

    /// north velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_north: i32,

    /// east velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_east: i32,

    /// down velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_down: i32,

    /// Speed 3-D (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    speed_3d: u32,

    /// Ground speed (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    ground_speed: u32,

    /// Heading of motion 2-D (degrees)
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_degrees)]
    heading: i32,

    /// Speed Accuracy Estimate (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    speed_accuracy_estimate: u32,

    /// Course / Heading Accuracy Estimate (degrees)
    #[ubx(map_type = f64, scale = 1e-5)]
    course_heading_accuracy_estimate: u32,
}

/// High Precision Geodetic Position Solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x14, fixed_payload_len = 36)]
struct NavHpPosLlh {
    /// Message version (0 for protocol version 27)
    version: u8,

    reserved1: [u8; 3],

    /// GPS Millisecond Time of Week
    itow: u32,

    /// Longitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    lon: i32,

    /// Latitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    lat: i32,

    /// Height above Ellipsoid (m)
    #[ubx(map_type = f64, scale = 1e-3)]
    height_meters: i32,

    /// Height above mean sea level (m)
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

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct NavHpPosEcefFlags: u8 {
        const INVALID_ECEF = 1;

    }
}

/// High Precision Geodetic Position Solution (ECEF)
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x13, fixed_payload_len = 28)]
struct NavHpPosEcef {
    /// Message version (0 for protocol version 27)
    version: u8,

    reserved1: [u8; 3],

    /// GPS Millisecond Time of Week
    itow: u32,

    /// ECEF X coordinate
    #[ubx(map_type = f64, alias = ecef_x_cm)]
    ecef_x: i32,

    /// ECEF Y coordinate
    #[ubx(map_type = f64, alias = ecef_y_cm)]
    ecef_y: i32,

    /// ECEF Z coordinate
    #[ubx(map_type = f64, alias = ecef_z_cm)]
    ecef_z: i32,

    /// High precision component of X
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_x + (ecef_x_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_x_hp_mm)]
    ecef_x_hp: i8,

    /// High precision component of Y
    /// Must be in the range -99..+99
    /// 9. Precise coordinate in cm = ecef_y + (ecef_y_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_y_hp_mm)]
    ecef_y_hp: i8,

    /// High precision component of Z
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_z + (ecef_z_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_z_hp_mm)]
    ecef_z_hp: i8,

    #[ubx(map_type = NavHpPosEcefFlags)]
    flags: u8,

    /// Horizontal accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    p_acc: u32,
}

/// Navigation Position Velocity Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x07, fixed_payload_len = 92)]
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
    nanosecond: i32,

    /// GNSS fix Type
    #[ubx(map_type = GpsFix)]
    fix_type: u8,
    #[ubx(map_type = NavPvtFlags)]
    flags: u8,
    #[ubx(map_type = NavPvtFlags2)]
    flags2: u8,
    num_satellites: u8,
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    lon: i32,
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    lat: i32,

    /// Height above Ellipsoid
    #[ubx(map_type = f64, scale = 1e-3)]
    height_meters: i32,

    /// Height above mean sea level
    #[ubx(map_type = f64, scale = 1e-3)]
    height_msl: i32,
    horiz_accuracy: u32,
    vert_accuracy: u32,

    /// north velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_north: i32,

    /// east velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_east: i32,

    /// down velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    vel_down: i32,

    /// Ground speed (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    ground_speed: u32,

    /// Heading of motion 2-D (degrees)
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_degrees)]
    heading: i32,

    /// Speed Accuracy Estimate (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    speed_accuracy_estimate: u32,

    /// Heading accuracy estimate (both motionand vehicle) (degrees)
    #[ubx(map_type = f64, scale = 1e-5)]
    heading_accuracy_estimate: u32,

    /// Position DOP
    pdop: u16,
    reserved1: [u8; 6],
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_of_vehicle_degrees)]
    heading_of_vehicle: i32,
    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_degrees)]
    magnetic_declination: i16,
    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_accuracy_degrees)]
    magnetic_declination_accuracy: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Fix status flags for `NavPvt`
    #[derive(Debug)]
    pub struct NavPvtFlags: u8 {
        /// position and velocity valid and within DOP and ACC Masks
        const GPS_FIX_OK = 1;
        /// DGPS used
        const DIFF_SOLN = 2;
        /// 1 = heading of vehicle is valid
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

///  Receiver Navigation Status
#[ubx_packet_recv]
#[ubx(class = 1, id = 3, fixed_payload_len = 16)]
struct NavStatus {
    /// GPS Millisecond Time of Week
    itow: u32,

    /// GPS fix Type, this value does not qualify a fix as

    /// valid and within the limits
    #[ubx(map_type = GpsFix)]
    fix_type: u8,

    /// Navigation Status Flags
    #[ubx(map_type = NavStatusFlags)]
    flags: u8,

    /// Fix Status Information
    #[ubx(map_type = FixStatusInfo)]
    fix_stat: u8,

    /// further information about navigation output
    #[ubx(map_type = NavStatusFlags2)]
    flags2: u8,

    /// Time to first fix (millisecond time tag)
    time_to_first_fix: u32,

    /// Milliseconds since Startup / Reset
    uptime_ms: u32,
}

/// Dilution of precision
#[ubx_packet_recv]
#[ubx(class = 1, id = 4, fixed_payload_len = 18)]
struct NavDop {
    /// GPS Millisecond Time of Week
    itow: u32,
    #[ubx(map_type = f32, scale = 1e-2)]
    geometric_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    position_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    time_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    vertical_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    horizontal_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    northing_dop: u16,
    #[ubx(map_type = f32, scale = 1e-2)]
    easting_dop: u16,
}

/// End of Epoch Marker
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x61, fixed_payload_len = 4)]
struct NavEoe {
    /// GPS time of week for navigation epoch
    itow: u32,
}

/// Navigation Solution Information
#[ubx_packet_recv]
#[ubx(class = 1, id = 6, fixed_payload_len = 52)]
struct NavSolution {
    /// GPS Millisecond Time of Week
    itow: u32,

    /// Fractional part of iTOW (range: +/-500000).
    ftow_ns: i32,

    /// GPS week number of the navigation epoch
    week: i16,

    /// GPS fix Type
    #[ubx(map_type = GpsFix)]
    fix_type: u8,

    /// Navigation Status Flags
    #[ubx(map_type = NavStatusFlags)]
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

    /// ECEF X velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_vx: i32,

    /// ECEF Y velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_vy: i32,

    /// ECEF Z velocity (m/s)
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

/// GPS fix Type
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GpsFix {
    NoFix = 0,
    DeadReckoningOnly = 1,
    Fix2D = 2,
    Fix3D = 3,
    GPSPlusDeadReckoning = 4,
    TimeOnlyFix = 5,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Navigation Status Flags
    #[derive(Debug)]
    pub struct NavStatusFlags: u8 {
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

/// Fix Status Information
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FixStatusInfo(u8);

impl FixStatusInfo {
    pub const fn has_pr_prr_correction(self) -> bool {
        (self.0 & 1) == 1
    }
    pub fn map_matching(self) -> MapMatchingStatus {
        let bits = (self.0 >> 6) & 3;
        match bits {
            0 => MapMatchingStatus::None,
            1 => MapMatchingStatus::Valid,
            2 => MapMatchingStatus::Used,
            3 => MapMatchingStatus::Dr,
            _ => unreachable!(),
        }
    }
    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for FixStatusInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixStatusInfo")
            .field("has_pr_prr_correction", &self.has_pr_prr_correction())
            .field("map_matching", &self.map_matching())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MapMatchingStatus {
    None = 0,
    /// valid, i.e. map matching data was received, but was too old
    Valid = 1,
    /// used, map matching data was applied
    Used = 2,
    /// map matching was the reason to enable the dead reckoning
    /// gpsFix type instead of publishing no fix
    Dr = 3,
}

/// Further information about navigation output
/// Only for FW version >= 7.01; undefined otherwise
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum NavStatusFlags2 {
    Acquisition = 0,
    Tracking = 1,
    PowerOptimizedTracking = 2,
    Inactive = 3,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavSatSvFlags(u32);

impl NavSatSvFlags {
    pub fn quality_ind(self) -> NavSatQualityIndicator {
        let bits = self.0 & 0x7;
        match bits {
            0 => NavSatQualityIndicator::NoSignal,
            1 => NavSatQualityIndicator::Searching,
            2 => NavSatQualityIndicator::SignalAcquired,
            3 => NavSatQualityIndicator::SignalDetected,
            4 => NavSatQualityIndicator::CodeLock,
            5..=7 => NavSatQualityIndicator::CarrierLock,
            _ => {
                panic!("Unexpected 3-bit bitfield value {}!", bits);
            },
        }
    }

    pub fn sv_used(self) -> bool {
        (self.0 >> 3) & 0x1 != 0
    }

    pub fn health(self) -> NavSatSvHealth {
        let bits = (self.0 >> 4) & 0x3;
        match bits {
            1 => NavSatSvHealth::Healthy,
            2 => NavSatSvHealth::Unhealthy,
            x => NavSatSvHealth::Unknown(x as u8),
        }
    }

    pub fn differential_correction_available(self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn smoothed(self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    pub fn orbit_source(self) -> NavSatOrbitSource {
        let bits = (self.0 >> 8) & 0x7;
        match bits {
            0 => NavSatOrbitSource::NoInfoAvailable,
            1 => NavSatOrbitSource::Ephemeris,
            2 => NavSatOrbitSource::Almanac,
            3 => NavSatOrbitSource::AssistNowOffline,
            4 => NavSatOrbitSource::AssistNowAutonomous,
            x => NavSatOrbitSource::Other(x as u8),
        }
    }

    pub fn ephemeris_available(self) -> bool {
        (self.0 >> 11) & 0x1 != 0
    }

    pub fn almanac_available(self) -> bool {
        (self.0 >> 12) & 0x1 != 0
    }

    pub fn an_offline_available(self) -> bool {
        (self.0 >> 13) & 0x1 != 0
    }

    pub fn an_auto_available(self) -> bool {
        (self.0 >> 14) & 0x1 != 0
    }

    pub fn sbas_corr(self) -> bool {
        (self.0 >> 16) & 0x1 != 0
    }

    pub fn rtcm_corr(self) -> bool {
        (self.0 >> 17) & 0x1 != 0
    }

    pub fn slas_corr(self) -> bool {
        (self.0 >> 18) & 0x1 != 0
    }

    pub fn spartn_corr(self) -> bool {
        (self.0 >> 19) & 0x1 != 0
    }

    pub fn pr_corr(self) -> bool {
        (self.0 >> 20) & 0x1 != 0
    }

    pub fn cr_corr(self) -> bool {
        (self.0 >> 21) & 0x1 != 0
    }

    pub fn do_corr(self) -> bool {
        (self.0 >> 22) & 0x1 != 0
    }

    pub const fn from(x: u32) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavSatSvFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSatSvFlags")
            .field("quality_ind", &self.quality_ind())
            .field("sv_used", &self.sv_used())
            .field("health", &self.health())
            .field(
                "differential_correction_available",
                &self.differential_correction_available(),
            )
            .field("smoothed", &self.smoothed())
            .field("orbit_source", &self.orbit_source())
            .field("ephemeris_available", &self.ephemeris_available())
            .field("almanac_available", &self.almanac_available())
            .field("an_offline_available", &self.an_offline_available())
            .field("an_auto_available", &self.an_auto_available())
            .field("sbas_corr", &self.sbas_corr())
            .field("rtcm_corr", &self.rtcm_corr())
            .field("slas_corr", &self.slas_corr())
            .field("spartn_corr", &self.spartn_corr())
            .field("pr_corr", &self.pr_corr())
            .field("cr_corr", &self.cr_corr())
            .field("do_corr", &self.do_corr())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatQualityIndicator {
    NoSignal,
    Searching,
    SignalAcquired,
    SignalDetected,
    CodeLock,
    CarrierLock,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatSvHealth {
    Healthy,
    Unhealthy,
    Unknown(u8),
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatOrbitSource {
    NoInfoAvailable,
    Ephemeris,
    Almanac,
    AssistNowOffline,
    AssistNowAutonomous,
    Other(u8),
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, fixed_payload_len = 12)]
struct NavSatSvInfo {
    gnss_id: u8,
    sv_id: u8,
    cno: u8,
    elev: i8,
    azim: i16,
    pr_res: i16,

    #[ubx(map_type = NavSatSvFlags)]
    flags: u32,
}

#[derive(Debug, Clone)]
pub struct NavSatIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NavSatIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % 12 == 0
    }
}

impl<'a> core::iter::Iterator for NavSatIter<'a> {
    type Item = NavSatSvInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 12];
            self.offset += 12;
            Some(NavSatSvInfoRef(data))
        } else {
            None
        }
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, max_payload_len = 1240)]
struct NavSat {
    /// GPS time of week in ms
    itow: u32,

    /// Message version, should be 1
    version: u8,

    num_svs: u8,

    reserved: [u8; 2],

    #[ubx(
        map_type = NavSatIter,
        from = NavSatIter::new,
        is_valid = NavSatIter::is_valid,
        may_fail,
        get_as_ref,
    )]
    svs: [u8; 0],
}

/// Odometer solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x09, fixed_payload_len = 20)]
struct NavOdo {
    version: u8,
    reserved: [u8; 3],
    itow: u32,
    distance: u32,
    total_distance: u32,
    distance_std: u32,
}

/// Reset odometer
#[ubx_packet_send]
#[ubx(class = 0x01, id = 0x10, fixed_payload_len = 0)]
struct NavResetOdo {}

/// Configure odometer
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x1E,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
struct CfgOdo {
    version: u8,
    reserved: [u8; 3],
    /// Odometer COG filter flags. See [OdoCogFilterFlags] for details.
    #[ubx(map_type = OdoCogFilterFlags)]
    flags: u8,
    #[ubx(map_type = OdoProfile, may_fail)]
    odo_cfg: u8,
    reserved2: [u8; 6],
    cog_max_speed: u8,
    cog_max_pos_acc: u8,
    reserved3: [u8; 2],
    vel_lp_gain: u8,
    cog_lp_gain: u8,
    reserved4: [u8; 2],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct OdoCogFilterFlags: u8 {
        /// Odometer enabled flag
        const USE_ODO = 0x01;
        /// Low-speed COG filter enabled flag
        const USE_COG = 0x02;
        /// Output low-pass filtered velocity flag
        const OUT_LP_VEL = 0x04;
        /// Output low-pass filtered heading (COG) flag
        const OUT_LP_COG = 0x08;
    }
}

/// Odometer configuration profile
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OdoProfile {
    #[default]
    Running = 0,
    Cycling = 1,
    Swimming = 2,
    Car = 3,
    Custom = 4,
}

/// Configure Jamming interference monitoring
#[ubx_packet_recv_send]
#[ubx(class = 0x06, id = 0x39, fixed_payload_len = 8)]
struct CfgItfm {
    /// Interference config Word
    #[ubx(map_type = CfgItfmConfig)]
    config: u32,
    /// Extra settings
    #[ubx(map_type = CfgItfmConfig2)]
    config2: u32,
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmConfig {
    /// enable interference detection
    enable: bool,
    /// Broadband jamming detection threshold (dB)
    bb_threshold: CfgItfmBbThreshold,
    /// CW jamming detection threshold (dB)
    cw_threshold: CfgItfmCwThreshold,
    /// Reserved algorithm settings
    /// should be set to 0x16B156 default value
    /// for correct settings
    algorithm_bits: CfgItfmAlgoBits,
}

impl CfgItfmConfig {
    pub fn new(enable: bool, bb_threshold: u32, cw_threshold: u32) -> Self {
        Self {
            enable,
            bb_threshold: bb_threshold.into(),
            cw_threshold: cw_threshold.into(),
            algorithm_bits: CfgItfmAlgoBits::default(),
        }
    }

    const fn into_raw(self) -> u32 {
        (self.enable as u32) << 31
            | self.cw_threshold.into_raw()
            | self.bb_threshold.into_raw()
            | self.algorithm_bits.into_raw()
    }
}

impl From<u32> for CfgItfmConfig {
    fn from(cfg: u32) -> Self {
        let enable = (cfg & 0x80000000) > 0;
        let bb_threshold = CfgItfmBbThreshold::from(cfg);
        let cw_threshold = CfgItfmCwThreshold::from(cfg);
        let algorithm_bits = CfgItfmAlgoBits::from(cfg);
        Self {
            enable,
            bb_threshold,
            cw_threshold,
            algorithm_bits,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmBbThreshold(u32);

impl CfgItfmBbThreshold {
    const POSITION: u32 = 0;
    const LENGTH: u32 = 4;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmBbThreshold {
    fn default() -> Self {
        Self(3) // from UBX specifications
    }
}

impl From<u32> for CfgItfmBbThreshold {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmCwThreshold(u32);

impl CfgItfmCwThreshold {
    const POSITION: u32 = 4;
    const LENGTH: u32 = 5;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmCwThreshold {
    fn default() -> Self {
        Self(15) // from UBX specifications
    }
}

impl From<u32> for CfgItfmCwThreshold {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmAlgoBits(u32);

impl CfgItfmAlgoBits {
    const POSITION: u32 = 9;
    const LENGTH: u32 = 22;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmAlgoBits {
    fn default() -> Self {
        Self(0x16B156) // from UBX specifications
    }
}

impl From<u32> for CfgItfmAlgoBits {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmConfig2 {
    /// General settings, should be set to
    /// 0x31E default value, for correct setting
    general: CfgItfmGeneralBits,
    /// antenna settings
    antenna: CfgItfmAntennaSettings,
    /// Set to true to scan auxillary bands on ublox-M8,
    /// ignored otherwise
    scan_aux_bands: bool,
}

impl CfgItfmConfig2 {
    pub fn new(antenna: CfgItfmAntennaSettings, scan_aux_bands: bool) -> Self {
        Self {
            general: CfgItfmGeneralBits::default(),
            antenna,
            scan_aux_bands,
        }
    }

    const fn into_raw(self) -> u32 {
        ((self.scan_aux_bands as u32) << 14)
            | self.general.into_raw()
            | self.antenna.into_raw() as u32
    }
}

impl From<u32> for CfgItfmConfig2 {
    fn from(cfg: u32) -> Self {
        let scan_aux_bands = (cfg & 0x4000) > 0;
        let general = CfgItfmGeneralBits::from(cfg);
        let antenna = CfgItfmAntennaSettings::from(cfg);
        Self {
            scan_aux_bands,
            general,
            antenna,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmGeneralBits(u32);

impl CfgItfmGeneralBits {
    const POSITION: u32 = 0;
    const LENGTH: u32 = 12;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmGeneralBits {
    fn default() -> Self {
        Self(0x31E) // from UBX specifications
    }
}

impl From<u32> for CfgItfmGeneralBits {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

/// ITFM Antenna settings helps the interference
/// monitoring module
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum CfgItfmAntennaSettings {
    /// Type of Antenna is not known
    #[default]
    Unknown = 0,
    /// Active antenna
    Active = 1,
    /// Passive antenna
    Passive = 2,
}

impl From<u32> for CfgItfmAntennaSettings {
    fn from(cfg: u32) -> Self {
        let cfg = (cfg & 0x3000) >> 12;
        match cfg {
            1 => CfgItfmAntennaSettings::Active,
            2 => CfgItfmAntennaSettings::Passive,
            _ => CfgItfmAntennaSettings::Unknown,
        }
    }
}

/// Information message conifg
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x2,
    fixed_payload_len = 10,
    flags = "default_for_builder"
)]
struct CfgInf {
    protocol_id: u8,
    reserved: [u8; 3],
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_0: u8,
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_1: u8,
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_2: u8,
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_3: u8,
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_4: u8,
    #[ubx(map_type = CfgInfMask)]
    inf_msg_mask_5: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgInfMask` parameters bitmask
    #[derive(Default, Debug, Clone, Copy)]
    pub struct CfgInfMask: u8 {
        const ERROR = 0x1;
        const WARNING = 0x2;
        const NOTICE = 0x4;
        const TEST  = 0x08;
        const DEBUG = 0x10;
    }
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x0,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfError {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x2,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfNotice {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x3,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfTest {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x1,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfWarning {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x4,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfDebug {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

mod inf {
    pub(crate) fn convert_to_str(bytes: &[u8]) -> Option<&str> {
        match core::str::from_utf8(bytes) {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }

    pub(crate) fn is_valid(_bytes: &[u8]) -> bool {
        // Validity is checked in convert_to_str
        true
    }
}

#[ubx_packet_send]
#[ubx(
    class = 0x0B,
    id = 0x01,
    fixed_payload_len = 48,
    flags = "default_for_builder"
)]
struct AidIni {
    ecef_x_or_lat: i32,
    ecef_y_or_lon: i32,
    ecef_z_or_alt: i32,
    pos_accuracy: u32,
    time_cfg: u16,
    week_or_ym: u16,
    tow_or_hms: u32,
    tow_ns: i32,
    tm_accuracy_ms: u32,
    tm_accuracy_ns: u32,
    clk_drift_or_freq: i32,
    clk_drift_or_freq_accuracy: u32,
    flags: u32,
}

impl AidIniBuilder {
    pub fn set_position(mut self, pos: Position) -> Self {
        self.ecef_x_or_lat = (pos.lat * 10_000_000.0) as i32;
        self.ecef_y_or_lon = (pos.lon * 10_000_000.0) as i32;
        self.ecef_z_or_alt = (pos.alt * 100.0) as i32; // Height is in centimeters, here
        self.flags |= (1 << 0) | (1 << 5);
        self
    }

    pub fn set_time(mut self, tm: DateTime<Utc>) -> Self {
        self.week_or_ym = (match tm.year_ce() {
            (true, yr) => yr - 2000,
            (false, _) => {
                panic!("AID-INI packet only supports years after 2000");
            },
        } * 100
            + tm.month0()) as u16;
        self.tow_or_hms = tm.hour() * 10000 + tm.minute() * 100 + tm.second();
        self.tow_ns = tm.nanosecond() as i32;
        self.flags |= (1 << 1) | (1 << 10);
        self
    }
}

/// ALP client requests AlmanacPlus data from server
#[ubx_packet_recv]
#[ubx(class = 0x0B, id = 0x32, fixed_payload_len = 16)]
struct AlpSrv {
    pub id_size: u8,
    pub data_type: u8,
    pub offset: u16,
    pub size: u16,
    pub file_id: u16,
    pub data_size: u16,
    pub id1: u8,
    pub id2: u8,
    pub id3: u32,
}

/// Messages in this class are sent as a result of a CFG message being
/// received, decoded and processed by thereceiver.
#[ubx_packet_recv]
#[ubx(class = 5, id = 1, fixed_payload_len = 2)]
struct AckAck {
    /// Class ID of the Acknowledged Message
    class: u8,

    /// Message ID of the Acknowledged Message
    msg_id: u8,
}

impl<'a> AckAckRef<'a> {
    pub fn is_ack_for<T: UbxPacketMeta>(&self) -> bool {
        self.class() == T::CLASS && self.msg_id() == T::ID
    }
}

/// Message Not-Acknowledge
#[ubx_packet_recv]
#[ubx(class = 5, id = 0, fixed_payload_len = 2)]
struct AckNak {
    /// Class ID of the Acknowledged Message
    class: u8,

    /// Message ID of the Acknowledged Message
    msg_id: u8,
}

impl<'a> AckNakRef<'a> {
    pub fn is_nak_for<T: UbxPacketMeta>(&self) -> bool {
        self.class() == T::CLASS && self.msg_id() == T::ID
    }
}

/// Reset Receiver / Clear Backup Data Structures
#[ubx_packet_send]
#[ubx(class = 6, id = 4, fixed_payload_len = 4)]
struct CfgRst {
    /// Battery backed RAM sections to clear
    #[ubx(map_type = NavBbrMask)]
    nav_bbr_mask: u16,

    /// Reset Type
    #[ubx(map_type = ResetMode)]
    reset_mode: u8,
    reserved1: u8,
}

/// Reset Receiver / Clear Backup Data Structures
#[ubx_packet_recv_send]
#[ubx(class = 6, id = 0x13, fixed_payload_len = 4)]
struct CfgAnt {
    /// Antenna flag mask. See [AntFlags] for details.
    #[ubx(map_type = AntFlags)]
    flags: u16,
    /// Antenna pin configuration. See 32.10.1.1 in receiver spec for details.
    pins: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct AntFlags: u16 {
        /// Enable supply voltage control signal
        const SVCS = 0x01;
        /// Enable short circuit detection
        const SCD = 0x02;
        /// Enable open circuit detection
        const OCD = 0x04;
        /// Power down on short circuit detection
        const PDWN_ON_SCD = 0x08;
        /// Enable automatic recovery from short circuit state
        const RECOVERY = 0x10;
    }
}

/// TP5: "Time Pulse" Config frame (32.10.38.4)
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x31,
    fixed_payload_len = 32,
    flags = "default_for_builder"
)]
struct CfgTp5 {
    #[ubx(map_type = CfgTp5TimePulseMode, may_fail)]
    tp_idx: u8,
    version: u8,
    reserved1: [u8; 2],
    /// Antenna cable delay [ns]
    #[ubx(map_type = f32, scale = 1.0)]
    ant_cable_delay: i16,
    /// RF group delay [ns]
    #[ubx(map_type = f32, scale = 1.0)]
    rf_group_delay: i16,
    /// Frequency in Hz or Period in us,
    /// depending on `flags::IS_FREQ` bit
    #[ubx(map_type = f64, scale = 1.0)]
    freq_period: u32,
    /// Frequency in Hz or Period in us,
    /// when locked to GPS time.
    /// Only used when `flags::LOCKED_OTHER_SET` is set
    #[ubx(map_type = f64, scale = 1.0)]
    freq_period_lock: u32,
    /// Pulse length or duty cycle, [us] or [*2^-32],
    /// depending on `flags::LS_LENGTH` bit
    #[ubx(map_type = f64, scale = 1.0)]
    pulse_len_ratio: u32,
    /// Pulse Length in us or duty cycle (*2^-32),
    /// when locked to GPS time.
    /// Only used when `flags::LOCKED_OTHER_SET` is set
    #[ubx(map_type = f64, scale = 1.0)]
    pulse_len_ratio_lock: u32,
    /// User configurable time pulse delay in [ns]
    #[ubx(map_type = f64, scale = 1.0)]
    user_delay: i32,
    /// Configuration flags, see [CfgTp5Flags]
    #[ubx(map_type = CfgTp5Flags)]
    flags: u32,
}

/// Time pulse selection, used in CfgTp5 frame
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum CfgTp5TimePulseMode {
    #[default]
    TimePulse = 0,
    TimePulse2 = 1,
}

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
    /// Time transfer modes, see [CfgTmode2TimeXferModes] for details
    #[ubx(map_type = CfgTmode2TimeXferModes, may_fail)]
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
    survery_in_accur_limit: u32,
}

/// Time transfer modes (32.10.36)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgTmode2TimeXferModes {
    #[default]
    Disabled = 0,
    SurveyIn = 1,
    /// True position information required
    /// when using `fixed mode`
    FixedMode = 2,
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

/// Time mode survey-in status
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x04, fixed_payload_len = 28)]
struct TimSvin {
    /// Passed survey-in minimum duration
    /// Units: s
    dur: u32,
    /// Current survey-in mean position ECEF X coordinate
    mean_x: i32,
    /// Current survey-in mean position ECEF Y coordinate
    mean_y: i32,
    /// Current survey-in mean position ECEF Z coordinate
    mean_z: i32,
    /// Current survey-in mean position 3D variance
    mean_v: i32,
    /// Number of position observations used during survey-in
    obs: u32,
    /// Survey-in position validity flag, 1 = valid, otherwise 0
    valid: u8,
    /// Survey-in in progress flag, 1 = in-progress, otherwise 0
    active: u8,
    reserved: [u8; 2],
}

/// Leap second event information
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x26, fixed_payload_len = 24)]
struct NavTimeLs {
    /// GPS time of week of the navigation epoch in ms.
    itow: u32,
    ///Message version (0x00 for this version)
    version: u8,
    reserved_1: [u8; 3],
    /// Information source for the current number of leap seconds.
    /// 0: Default (hardcoded in the firmware, can be outdated)
    /// 1: Derived from time difference between GPS and GLONASS time
    /// 2: GPS
    /// 3: SBAS
    /// 4: BeiDou
    /// 5: Galileo
    /// 6: Aided data 7: Configured 8: NavIC
    /// 255: Unknown
    src_of_curr_ls: u8,
    /// Current number of leap seconds since start of GPS time (Jan 6, 1980). It reflects how much
    /// GPS time is ahead of UTC time. Galileo number of leap seconds is the same as GPS. BeiDou
    /// number of leap seconds is 14 less than GPS. GLONASS follows UTC time, so no leap seconds.
    current_ls: i8,
    /// Information source for the future leap second event.
    /// 0: No source
    /// 2: GPS
    /// 3: SBAS
    /// 4: BeiDou
    /// 5: Galileo
    /// 6: GLONASS 7: NavIC
    src_of_ls_change: u8,
    /// Future leap second change if one is scheduled. +1 = positive leap second, -1 = negative
    /// leap second, 0 = no future leap second event scheduled or no information available.
    ls_change: i8,
    /// Number of seconds until the next leap second event, or from the last leap second event if
    /// no future event scheduled. If > 0 event is in the future, = 0 event is now, < 0 event is in
    /// the past. Valid only if validTimeToLsEvent = 1.
    time_to_ls_event: i32,
    /// GPS week number (WN) of the next leap second event or the last one if no future event
    /// scheduled. Valid only if validTimeToLsEvent = 1.
    date_of_ls_gps_wn: u16,
    /// GPS day of week number (DN) for the next leap second event or the last one if no future
    /// event scheduled. Valid only if validTimeToLsEvent = 1. (GPS and Galileo DN: from 1 = Sun to
    /// 7 = Sat. BeiDou DN: from 0 = Sun to 6 = Sat.)
    date_of_ls_gps_dn: u16,
    reserved_2: [u8; 3],
    /// Validity flags see `NavTimeLsFlags`
    #[ubx(map_type = NavTimeLsFlags)]
    valid: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Fix status flags for `NavTimeLsFlags`
    #[derive(Debug)]
    pub struct NavTimeLsFlags: u8 {
        /// 1 = Valid current number of leap seconds value.
        const VALID_CURR_LS = 1;
        /// Valid time to next leap second event or from the last leap second event if no future
        /// event scheduled.
        const VALID_TIME_TO_LS_EVENT = 2;
    }
}

/// Time MODE3 Config Frame (32.10.37.1)
/// only available on `timing` receivers
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x71,
    fixed_payload_len = 40,
    flags = "default_for_builder"
)]
struct CfgTmode3 {
    version: u8,
    reserved1: u8,
    /// Receiver mode, see [CfgTmode3RcvrMode] enum
    #[ubx(map_type = CfgTmode3RcvrMode)]
    rcvr_mode: u8,
    #[ubx(map_type = CfgTmode3Flags)]
    flags: u8,
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
    /// High precision WGS84 ECEF.x coordinate in [tenths of mm],
    /// or high precision latitude, in nano degrees,
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_x_or_lat_hp: i8,
    /// High precision WGS84 ECEF.y coordinate in [tenths of mm]
    /// or high precision longitude, in nano degrees,
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_y_or_lon_hp: i8,
    /// High precision WGS84 ECEF.z coordinate or altitude,
    /// both if tenths of [mm],
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_z_or_alt_hp: i8,
    reserved2: u8,
    /// Fixed position 3D accuracy [0.1 mm]
    #[ubx(map_type = f64, scale = 1e-4)]
    fixed_pos_acc: u32,
    /// Survey in minimum duration [s]
    sv_in_min_duration: u32,
    /// Survey in position accuracy limit [0.1 mm]
    #[ubx(map_type = f64, scale = 1e-4)]
    sv_in_accur_limit: u32,
    reserved3: [u8; 8],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode3RcvrMode: u8 {
        const DISABLED = 0x01;
        const SURVEY_IN = 0x02;
        /// True ARP position is required in `FixedMode`
        const FIXED_MODE = 0x04;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTp5Flags: u32 {
        // Enables time pulse
        const ACTIVE = 0x01;
        /// Synchronize time pulse to GNSS as
        /// soon as GNSS time is valid.
        /// Uses local lock otherwise.
        const LOCK_GNSS_FREQ = 0x02;
        /// use `freq_period_lock` and `pulse_len_ratio_lock`
        /// fields as soon as GPS time is valid. Uses
        /// `freq_period` and `pulse_len_ratio` when GPS time is invalid.
        const LOCKED_OTHER_SET = 0x04;
        /// `freq_period` and `pulse_len_ratio` fields
        /// are interprated as frequency when this bit is set
        const IS_FREQ = 0x08;
        /// Interprate pulse lengths instead of duty cycle
        const IS_LENGTH = 0x10;
        /// Align pulse to top of second
        /// Period time must be integer fraction of `1sec`
        /// `LOCK_GNSS_FREQ` is expected, to unlock this feature
        const ALIGN_TO_TOW = 0x20;
        /// Pulse polarity,
        /// 0: falling edge @ top of second,
        /// 1: rising edge @ top of second,
        const POLARITY = 0x40;
        /// UTC time grid
        const UTC_TIME_GRID = 0x80;
        /// GPS time grid
        const GPS_TIME_GRID = 0x100;
        /// GLO time grid
        const GLO_TIME_GRID = 0x200;
        /// BDS time grid
        const BDS_TIME_GRID = 0x400;
        /// GAL time grid
        /// not supported in protocol < 18
        const GAL_TIME_GRID = 0x800;
        /// Switches to FreqPeriodLock and PulseLenRatio
        /// as soon as Sync Manager has an accurate time,
        /// never switches back
        const SYNC_MODE_0 = 0x1000;
        /// Switches to FreqPeriodLock and PulseLenRatioLock
        /// as soon as Sync Manager has an accurante time,
        /// and switch back to FreqPeriodLock and PulseLenRatio
        /// when time gets inaccurate
        const SYNC_MODE_1 = 0x2000;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode3Flags: u8 {
        /// Set if position is given in Lat/Lon/Alt,
        /// ECEF coordinates being used otherwise
        const LLA = 0x01;
    }
}

#[ubx_extend_bitflags]
#[ubx(into_raw, rest_reserved)]
bitflags! {
    /// Battery backed RAM sections to clear
    pub struct NavBbrMask: u16 {
        const EPHEMERIS = 1;
        const ALMANACH = 2;
        const HEALTH = 4;
        const KLOBUCHARD = 8;
        const POSITION = 16;
        const CLOCK_DRIFT = 32;
        const OSCILATOR_PARAMETER = 64;
        const UTC_CORRECTION_PARAMETERS = 0x80;
        const RTC = 0x100;
        const SFDR_PARAMETERS = 0x800;
        const SFDR_VEHICLE_MONITORING_PARAMETERS = 0x1000;
        const TCT_PARAMETERS = 0x2000;
        const AUTONOMOUS_ORBIT_PARAMETERS = 0x8000;
    }
}

/// Predefined values for `NavBbrMask`
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NavBbrPredefinedMask(u16);

impl From<NavBbrPredefinedMask> for NavBbrMask {
    fn from(x: NavBbrPredefinedMask) -> Self {
        Self::from_bits_truncate(x.0)
    }
}

impl NavBbrPredefinedMask {
    pub const HOT_START: NavBbrPredefinedMask = NavBbrPredefinedMask(0);
    pub const WARM_START: NavBbrPredefinedMask = NavBbrPredefinedMask(1);
    pub const COLD_START: NavBbrPredefinedMask = NavBbrPredefinedMask(0xFFFF);
}

/// Reset Type
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ResetMode {
    /// Hardware reset (Watchdog) immediately
    HardwareResetImmediately = 0,
    ControlledSoftwareReset = 0x1,
    ControlledSoftwareResetGpsOnly = 0x02,
    /// Hardware reset (Watchdog) after shutdown (>=FW6.0)
    HardwareResetAfterShutdown = 0x04,
    ControlledGpsStop = 0x08,
    ControlledGpsStart = 0x09,
}

impl ResetMode {
    const fn into_raw(self) -> u8 {
        self as u8
    }
}

#[ubx_packet_send]
#[ubx(
  class = 0x06,
  id = 0x8a,
  max_payload_len = 772, // 4 + (4 + 8) * 64
)]
struct CfgValSet<'a> {
    /// Message version
    version: u8,
    /// The layers from which the configuration items should be retrieved
    #[ubx(map_type = CfgLayer)]
    layers: u8,
    reserved1: u16,
    cfg_data: &'a [CfgVal],
}

#[derive(Debug, Clone)]
pub struct CfgValIter<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) offset: usize,
}

impl<'a> CfgValIter<'a> {
    pub fn new(data: &'a mut [u8], values: &[CfgVal]) -> Self {
        let mut offset = 0;

        for value in values {
            offset += value.write_to(&mut data[offset..]);
        }

        Self {
            data: &data[..offset],
            offset: 0,
        }
    }
}

impl<'a> core::iter::Iterator for CfgValIter<'a> {
    type Item = CfgVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let cfg_val = CfgVal::parse(&self.data[self.offset..]);

            self.offset += cfg_val.len();

            Some(cfg_val)
        } else {
            None
        }
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing where configuration is applied.
    pub struct CfgLayer: u8 {
        const RAM = 0b001;
        const BBR = 0b010;
        const FLASH = 0b100;
    }
}

impl Default for CfgLayer {
    fn default() -> Self {
        Self::RAM | Self::BBR | Self::FLASH
    }
}

/// Port Configuration for I2C
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x00,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
struct CfgPrtI2c {
    #[ubx(map_type = I2cPortId, may_fail)]
    portid: u8,
    reserved1: u8,
    /// TX ready PIN configuration
    tx_ready: u16,
    /// I2C Mode Flags
    mode: u32,
    reserved2: u32,
    #[ubx(map_type = InProtoMask)]
    in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    out_proto_mask: u16,
    flags: u16,
    reserved3: u16,
}

/// Port Identifier Number (= 0 for I2C ports)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum I2cPortId {
    #[default]
    I2c = 0,
}

/// Port Configuration for UART
#[ubx_packet_recv_send]
#[ubx(class = 0x06, id = 0x00, fixed_payload_len = 20)]
struct CfgPrtUart {
    #[ubx(map_type = UartPortId, may_fail)]
    portid: u8,
    reserved0: u8,
    tx_ready: u16,
    #[ubx(map_type = UartMode)]
    mode: u32,
    baud_rate: u32,
    #[ubx(map_type = InProtoMask)]
    in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    out_proto_mask: u16,
    flags: u16,
    reserved5: u16,
}

/// Port Identifier Number (= 1 or 2 for UART ports)
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum UartPortId {
    Uart1 = 1,
    Uart2 = 2,
    Usb = 3,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UartMode {
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
}

impl UartMode {
    pub const fn new(data_bits: DataBits, parity: Parity, stop_bits: StopBits) -> Self {
        Self {
            data_bits,
            parity,
            stop_bits,
        }
    }

    const fn into_raw(self) -> u32 {
        self.data_bits.into_raw() | self.parity.into_raw() | self.stop_bits.into_raw()
    }
}

impl From<u32> for UartMode {
    fn from(mode: u32) -> Self {
        let data_bits = DataBits::from(mode);
        let parity = Parity::from(mode);
        let stop_bits = StopBits::from(mode);

        Self {
            data_bits,
            parity,
            stop_bits,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataBits {
    Seven,
    Eight,
}

impl DataBits {
    const POSITION: u32 = 6;
    const MASK: u32 = 0b11;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::Seven => 0b10,
            Self::Eight => 0b11,
        }) << Self::POSITION
    }
}

impl From<u32> for DataBits {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b00 => unimplemented!("five data bits"),
            0b01 => unimplemented!("six data bits"),
            0b10 => Self::Seven,
            0b11 => Self::Eight,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parity {
    Even,
    Odd,
    None,
}

impl Parity {
    const POSITION: u32 = 9;
    const MASK: u32 = 0b111;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::Even => 0b000,
            Self::Odd => 0b001,
            Self::None => 0b100,
        }) << Self::POSITION
    }
}

impl From<u32> for Parity {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b000 => Self::Even,
            0b001 => Self::Odd,
            0b100 | 0b101 => Self::None,
            0b010 | 0b011 | 0b110 | 0b111 => unimplemented!("reserved"),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StopBits {
    One,
    OneHalf,
    Two,
    Half,
}

impl StopBits {
    const POSITION: u32 = 12;
    const MASK: u32 = 0b11;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::One => 0b00,
            Self::OneHalf => 0b01,
            Self::Two => 0b10,
            Self::Half => 0b11,
        }) << Self::POSITION
    }
}

impl From<u32> for StopBits {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b00 => Self::One,
            0b01 => Self::OneHalf,
            0b10 => Self::Two,
            0b11 => Self::Half,
            _ => unreachable!(),
        }
    }
}

/// Port Configuration for SPI Port
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x00,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
struct CfgPrtSpi {
    #[ubx(map_type = SpiPortId, may_fail)]
    portid: u8,
    reserved0: u8,
    /// TX ready PIN configuration
    tx_ready: u16,
    /// SPI Mode Flags
    mode: u32,
    reserved3: u32,
    #[ubx(map_type = InProtoMask)]
    in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    out_proto_mask: u16,
    flags: u16,
    reserved5: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing which input protocols are active
    /// Each bit of this mask is used for a protocol.
    /// Through that, multiple protocols can be defined on a single port
    /// Used in `CfgPrtSpi` and `CfgPrtI2c`
    #[derive(Default, Debug)]
    pub struct InProtoMask: u16 {
        const UBLOX = 1;
        const NMEA = 2;
        const RTCM = 4;
        /// The bitfield inRtcm3 is not supported in protocol
        /// versions less than 20
        const RTCM3 = 0x20;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing which output protocols are active.
    /// Each bit of this mask is used for a protocol.
    /// Through that, multiple protocols can be defined on a single port
    /// Used in `CfgPrtSpi` and `CfgPrtI2c`
    #[derive(Default, Debug)]
    pub struct OutProtoMask: u16 {
        const UBLOX = 1;
        const NMEA = 2;
        /// The bitfield outRtcm3 is not supported in protocol
        /// versions less than 20
        const RTCM3 = 0x20;
    }
}

/// Port Identifier Number (= 4 for SPI port)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum SpiPortId {
    #[default]
    Spi = 4,
}

/// UTC Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x21, fixed_payload_len = 20)]
struct NavTimeUTC {
    /// GPS Millisecond Time of Week
    itow: u32,
    time_accuracy_estimate_ns: u32,

    /// Nanoseconds of second, range -1e9 .. 1e9
    nanos: i32,

    /// Year, range 1999..2099
    year: u16,

    /// Month, range 1..12
    month: u8,

    /// Day of Month, range 1..31
    day: u8,

    /// Hour of Day, range 0..23
    hour: u8,

    /// Minute of Hour, range 0..59
    min: u8,

    /// Seconds of Minute, range 0..59
    sec: u8,

    /// Validity Flags
    #[ubx(map_type = NavTimeUtcFlags)]
    valid: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Validity Flags of `NavTimeUTC`
    #[derive(Default, Debug)]
    pub struct NavTimeUtcFlags: u8 {
        /// Valid Time of Week
        const VALID_TOW = 1;
        /// Valid Week Number
        const VALID_WKN = 2;
        /// Valid UTC (Leap Seconds already known)
        const VALID_UTC = 4;
    }
}

/// Navigation/Measurement Rate Settings
#[ubx_packet_send]
#[ubx(class = 6, id = 8, fixed_payload_len = 6)]
struct CfgRate {
    /// Measurement Rate, GPS measurements are taken every `measure_rate_ms` milliseconds
    measure_rate_ms: u16,

    /// Navigation Rate, in number of measurement cycles.

    /// On u-blox 5 and u-blox 6, this parametercannot be changed, and is always equals 1.
    nav_rate: u16,

    /// Alignment to reference time
    #[ubx(map_type = AlignmentToReferenceTime)]
    time_ref: u16,
}

/// Alignment to reference time
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum AlignmentToReferenceTime {
    Utc = 0,
    Gps = 1,
    Glo = 2,
    Bds = 3,
    Gal = 4,
}

impl AlignmentToReferenceTime {
    const fn into_raw(self) -> u16 {
        self as u16
    }
}

/// Set Message Rate the current port
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 3)]
struct CfgMsgSinglePort {
    msg_class: u8,
    msg_id: u8,

    /// Send rate on current Target
    rate: u8,
}

impl CfgMsgSinglePortBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rate: u8) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rate,
        }
    }
}

/// Set Message rate configuration
/// Send rate is relative to the event a message is registered on.
/// For example, if the rate of a navigation message is set to 2,
/// the message is sent every second navigation solution
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 8)]
struct CfgMsgAllPorts {
    msg_class: u8,
    msg_id: u8,

    /// Send rate on I/O Port (6 Ports)
    rates: [u8; 6],
}

impl CfgMsgAllPortsBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rates: [u8; 6]) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rates,
        }
    }
}

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
    #[ubx(map_type = CfgNav5DynModel, may_fail)]
    dyn_model: u8,
    #[ubx(map_type = CfgNav5FixMode, may_fail)]
    fix_mode: u8,

    /// Fixed altitude (mean sea level) for 2D fixmode (m)
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

    /// Position Accuracy Mask (m)
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
    #[ubx(map_type = CfgNav5UtcStandard, may_fail)]
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
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5DynModel {
    Portable = 0,
    Stationary = 2,
    Pedestrian = 3,
    Automotive = 4,
    Sea = 5,
    AirborneWithLess1gAcceleration = 6,
    AirborneWithLess2gAcceleration = 7,
    #[default]
    AirborneWith4gAcceleration = 8,
    /// not supported in protocol versions less than 18
    WristWornWatch = 9,
    /// supported in protocol versions 19.2
    Bike = 10,
}

/// Position Fixing Mode
#[derive(Default)] // default needs to be derived before ubx_extend
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5FixMode {
    Only2D = 1,
    Only3D = 2,
    #[default]
    Auto2D3D = 3,
}

/// UTC standard to be used
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5UtcStandard {
    /// receiver selects based on GNSS configuration (see GNSS timebases)
    #[default]
    Automatic = 0,
    /// UTC as operated by the U.S. NavalObservatory (USNO);
    /// derived from GPStime
    Usno = 3,
    /// UTC as operated by the former Soviet Union; derived from GLONASS time
    UtcSu = 6,
    /// UTC as operated by the National TimeService Center, China;
    /// derived from BeiDou time
    UtcChina = 7,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct ScaleBack<T: FloatCore + FromPrimitive + ToPrimitive>(T);

impl<T: FloatCore + FromPrimitive + ToPrimitive> ScaleBack<T> {
    fn as_i8(self, x: T) -> i8 {
        let x = (x * self.0).round();
        if x < T::from_i8(i8::min_value()).unwrap() {
            i8::min_value()
        } else if x > T::from_i8(i8::max_value()).unwrap() {
            i8::max_value()
        } else {
            x.to_i8().unwrap()
        }
    }

    fn as_i16(self, x: T) -> i16 {
        let x = (x * self.0).round();
        if x < T::from_i16(i16::min_value()).unwrap() {
            i16::min_value()
        } else if x > T::from_i16(i16::max_value()).unwrap() {
            i16::max_value()
        } else {
            x.to_i16().unwrap()
        }
    }

    fn as_i32(self, x: T) -> i32 {
        let x = (x * self.0).round();
        if x < T::from_i32(i32::MIN).unwrap() {
            i32::MIN
        } else if x > T::from_i32(i32::MAX).unwrap() {
            i32::MAX
        } else {
            x.to_i32().unwrap()
        }
    }

    fn as_u32(self, x: T) -> u32 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u32(u32::MAX).unwrap() {
                x.to_u32().unwrap()
            } else {
                u32::MAX
            }
        } else {
            0
        }
    }

    fn as_u16(self, x: T) -> u16 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u16(u16::MAX).unwrap() {
                x.to_u16().unwrap()
            } else {
                u16::MAX
            }
        } else {
            0
        }
    }

    fn as_u8(self, x: T) -> u8 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u8(u8::MAX).unwrap() {
                x.to_u8().unwrap()
            } else {
                u8::MAX
            }
        } else {
            0
        }
    }
}

/// Navigation Engine Expert Settings
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x23,
    fixed_payload_len = 40,
    flags = "default_for_builder"
)]
struct CfgNavX5 {
    /// Only version 2 supported
    version: u16,

    /// Only the masked parameters will be applied
    #[ubx(map_type = CfgNavX5Params1)]
    mask1: u16,

    #[ubx(map_type = CfgNavX5Params2)]
    mask2: u32,

    /// Reserved
    reserved1: [u8; 2],

    /// Minimum number of satellites for navigation
    min_svs: u8,

    ///Maximum number of satellites for navigation
    max_svs: u8,

    /// Minimum satellite signal level for navigation
    min_cno_dbhz: u8,

    /// Reserved
    reserved2: u8,

    /// initial fix must be 3D
    ini_fix_3d: u8,

    /// Reserved
    reserved3: [u8; 2],

    /// issue acknowledgements for assistance message input
    ack_aiding: u8,

    /// GPS week rollover number
    wkn_rollover: u16,

    /// Permanently attenuated signal compensation
    sig_atten_comp_mode: u8,

    /// Reserved
    reserved4: u8,
    reserved5: [u8; 2],
    reserved6: [u8; 2],

    /// Use Precise Point Positioning (only available with the PPP product variant)
    use_ppp: u8,

    /// AssistNow Autonomous configuration
    aop_cfg: u8,

    /// Reserved
    reserved7: [u8; 2],

    /// Maximum acceptable (modeled) AssistNow Autonomous orbit error
    aop_orb_max_err: u16,

    /// Reserved
    reserved8: [u8; 4],
    reserved9: [u8; 3],

    /// Enable/disable ADR/UDR sensor fusion
    use_adr: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNavX51` parameters bitmask
    #[derive(Default, Debug)]
    pub struct CfgNavX5Params1: u16 {
        /// apply min/max SVs settings
        const MIN_MAX = 0x4;
        /// apply minimum C/N0 setting
        const MIN_CNO = 0x8;
        /// apply initial 3D fix settings
        const INITIAL_3D_FIX = 0x40;
        /// apply GPS weeknumber rollover settings
        const WKN_ROLL = 0x200;
        /// apply assistance acknowledgement settings
        const AID_ACK = 0x400;
        /// apply usePPP flag
        const USE_PPP = 0x2000;
        /// apply aopCfg (useAOP flag) and aopOrbMaxErr settings (AssistNow Autonomous)
        const AOP_CFG = 0x4000;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNavX5Params2` parameters bitmask
    #[derive(Default, Debug)]
    pub struct CfgNavX5Params2: u32 {
        ///  apply ADR/UDR sensor fusion on/off setting
        const USE_ADR = 0x40;
        ///  apply signal attenuation compensation feature settings
        const USE_SIG_ATTEN_COMP = 0x80;
    }
}

/// GNSS Assistance ACK UBX-MGA-ACK
#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x60, fixed_payload_len = 8)]
struct MgaAck {
    /// Type of acknowledgment: 0 -> not used, 1 -> accepted
    ack_type: u8,

    /// Version 0
    version: u8,

    /// Provides greater information on what the receiver chose to do with the message contents.
    /// See [MsgAckInfoCode].
    #[ubx(map_type = MsgAckInfoCode)]
    info_code: u8,

    /// UBX message ID of the acknowledged message
    msg_id: u8,

    /// The first 4 bytes of the acknowledged message's payload
    msg_payload_start: [u8; 4],
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MsgAckInfoCode {
    Accepted = 0,
    RejectedNoTime = 1,
    RejectedBadVersion = 2,
    RejectedBadSize = 3,
    RejectedDBStoreFailed = 4,
    RejectedNotReady = 5,
    RejectedUnknownType = 6,
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x06, fixed_payload_len = 48)]
struct MgaGloEph {
    msg_type: u8,
    version: u8,
    sv_id: u8,
    reserved1: u8,
    ft: u8,
    b: u8,
    m: u8,
    h: i8,
    x: i32,
    y: i32,
    z: i32,
    dx: i32,
    dy: i32,
    dz: i32,
    ddx: i8,
    ddy: i8,
    ddz: i8,
    tb: u8,
    gamma: u16,
    e: u8,
    delta_tau: u8,
    tau: i32,
    reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 16)]
struct MgaGpsIono {
    /// Message type: 0x06 for this type
    msg_type: u8,
    /// Message version: 0x00 for this version
    version: u8,
    reserved1: [u8; 2],
    /// Ionospheric parameter alpha0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-30
    alpha0: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-27
    alpha1: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha2: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha3: i8,
    /// Ionospheric parameter beta0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-11
    beta0: i8,
    /// Ionospheric parameter beta0 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-14
    beta1: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta2: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta3: i8,
    reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
struct MgaGpsEph {
    msg_type: u8,
    version: u8,
    sv_id: u8,
    reserved1: u8,
    fit_interval: u8,
    ura_index: u8,
    sv_health: u8,
    tgd: i8,
    iodc: u16,
    toc: u16,
    reserved2: u8,
    af2: i8,
    af1: i16,
    af0: i32,
    crs: i16,
    delta_n: i16,
    m0: i32,
    cuc: i16,
    cus: i16,
    e: u32,
    sqrt_a: u32,
    toe: u16,
    cic: i16,
    omega0: i32,
    cis: i16,
    crc: i16,
    i0: i32,
    omega: i32,
    omega_dot: i32,
    idot: i16,
    reserved3: [u8; 2],
}

/// Time pulse time data
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x01, fixed_payload_len = 16)]
struct TimTp {
    /// Time pulse time of week according to time base
    tow_ms: u32,
    /// Submillisecond part of towMS (scaling: 2^(-32))
    tow_sub_ms: u32,
    /// Quantization error of time pulse
    q_err: i32,
    /// Time pulse week number according to time base
    week: u16,
    /// Flags
    #[ubx(map_type = TimTpFlags, from = TimTpFlags)]
    flags: u8,
    /// Time reference information
    #[ubx(map_type = TimTpRefInfo, from = TimTpRefInfo)]
    ref_info: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTpFlags(u8);

impl TimTpFlags {
    /// Time base
    pub fn time_base(&self) -> TimTpTimeBase {
        if self.0 & 0b1 == 0 {
            TimTpTimeBase::Gnss
        } else {
            TimTpTimeBase::Utc
        }
    }

    /// UTC availability
    pub fn utc_available(&self) -> bool {
        self.0 & 0b10 != 0
    }

    /// (T)RAIM state
    ///
    /// Returns `None` if unavailale.
    pub fn raim_active(&self) -> Option<bool> {
        match (self.0 >> 2) & 0b11 {
            // Inactive.
            0b01 => Some(false),
            // Active.
            0b10 => Some(true),
            // Unavailable.
            _ => None,
        }
    }

    /// Quantization error validity
    pub fn q_err_valid(&self) -> bool {
        self.0 & 0b10000 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimTpTimeBase {
    Gnss,
    Utc,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTpRefInfo(u8);

impl TimTpRefInfo {
    /// GNSS reference information. Only valid if time base is GNSS.
    pub fn time_ref_gnss(&self) -> Option<TimTpRefInfoTimeRefGnss> {
        Some(match self.0 & 0b1111 {
            0 => TimTpRefInfoTimeRefGnss::Gps,
            1 => TimTpRefInfoTimeRefGnss::Glo,
            2 => TimTpRefInfoTimeRefGnss::Bds,
            3 => TimTpRefInfoTimeRefGnss::Gal,
            4 => TimTpRefInfoTimeRefGnss::NavIc,
            _ => return None,
        })
    }

    /// UTC standard identifier. Only valid if time base is UTC.
    pub fn utc_standard(&self) -> Option<TimTpRefInfoUtcStandard> {
        Some(match self.0 >> 4 {
            1 => TimTpRefInfoUtcStandard::Crl,
            2 => TimTpRefInfoUtcStandard::Nist,
            3 => TimTpRefInfoUtcStandard::Usno,
            4 => TimTpRefInfoUtcStandard::Bipm,
            5 => TimTpRefInfoUtcStandard::Eu,
            6 => TimTpRefInfoUtcStandard::Su,
            7 => TimTpRefInfoUtcStandard::Ntsc,
            8 => TimTpRefInfoUtcStandard::Npli,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimTpRefInfoTimeRefGnss {
    Gps,
    Glo,
    Bds,
    Gal,
    NavIc,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimTpRefInfoUtcStandard {
    Crl,
    Nist,
    Usno,
    Bipm,
    Eu,
    Su,
    Ntsc,
    Npli,
}

/// Time mark data
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x03, fixed_payload_len = 28)]
struct TimTm2 {
    /// Channel (i.e. EXTINT) upon which the pulse was measured
    ch: u8,
    /// Flags
    #[ubx(map_type = TimTm2Flags, from = TimTm2Flags)]
    flags: u8,
    /// Rising edge counter
    count: u16,
    /// Week number of last rising edge
    wn_r: u16,
    /// Week number of last falling edge
    wn_f: u16,
    /// Tow of rising edge
    tow_ms_r: u32,
    /// Millisecond fraction of tow of rising edge in nanoseconds
    tow_sub_ms_r: u32,
    /// Tow of falling edge
    tow_ms_f: u32,
    /// Millisecond fraction of tow of falling edge in nanoseconds
    tow_sub_ms_f: u32,
    /// Accuracy estimate
    acc_est: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTm2Flags(u8);

impl TimTm2Flags {
    pub fn mode(&self) -> TimTm2Mode {
        if self.0 & 0b1 == 0 {
            TimTm2Mode::Single
        } else {
            TimTm2Mode::Running
        }
    }

    pub fn run(&self) -> TimTm2Run {
        if self.0 & 0b10 == 0 {
            TimTm2Run::Armed
        } else {
            TimTm2Run::Stopped
        }
    }

    pub fn new_falling_edge(&self) -> bool {
        self.0 & 0b100 != 0
    }

    pub fn new_rising_edge(&self) -> bool {
        self.0 & 0b10000000 != 0
    }

    pub fn time_base(&self) -> TimTm2TimeBase {
        match self.0 & 0b11000 {
            0 => TimTm2TimeBase::Receiver,
            1 => TimTm2TimeBase::Gnss,
            2 => TimTm2TimeBase::Utc,
            _ => unreachable!(),
        }
    }

    /// UTC availability
    pub fn utc_available(&self) -> bool {
        self.0 & 0b100000 != 0
    }

    pub fn time_valid(&self) -> bool {
        self.0 & 0b1000000 != 0
    }
}

pub enum TimTm2Mode {
    Single,
    Running,
}

pub enum TimTm2Run {
    Armed,
    Stopped,
}

pub enum TimTm2TimeBase {
    Receiver,
    Gnss,
    Utc,
}

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x15, max_payload_len = 8176)] // 16 + 255 * 32
struct RxmRawx {
    /// Measurement time of week in receiver local time approximately aligned to the GPS time system.
    rcv_tow: f64,
    /// GPS week number in receiver local time.
    week: u16,
    /// GPS leap seconds (GPS-UTC)
    leap_s: i8,
    /// Number of measurements to follow
    num_meas: u8,
    /// Receiver tracking status bitfield
    #[ubx(map_type = RecStatFlags)]
    rec_stat: u8,
    /// Message version
    version: u8,
    reserved1: [u8; 2],
    /// Extended software information strings
    #[ubx(
        map_type = RxmRawxInfoIter,
        from = RxmRawxInfoIter::new,
        may_fail,
        is_valid = RxmRawxInfoIter::is_valid,
    )]
    measurements: [u8; 0],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNavX5Params2` parameters bitmask
    #[derive(Default, Debug)]
    pub struct RecStatFlags: u8 {
        /// Leap seconds have been determined
        const LEAP_SEC = 0x1;
        /// Clock reset applied.
        const CLK_RESET = 0x2;
    }
}

/// Hardware status
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x09, fixed_payload_len = 60)]
struct MonHw {
    pin_sel: u32,
    pin_bank: u32,
    pin_dir: u32,
    pin_val: u32,
    noise_per_ms: u16,
    agc_cnt: u16,
    #[ubx(map_type = AntennaStatus)]
    a_status: u8,
    #[ubx(map_type = AntennaPower)]
    a_power: u8,
    flags: u8,
    reserved1: u8,
    used_mask: u32,
    vp: [u8; 17],
    jam_ind: u8,
    reserved2: [u8; 2],
    pin_irq: u32,
    pull_h: u32,
    pull_l: u32,
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaStatus {
    Init = 0,
    DontKnow = 1,
    Ok = 2,
    Short = 3,
    Open = 4,
}

/// GNSS status monitoring,
/// gives currently selected constellations
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x28, fixed_payload_len = 8)]
struct MonGnss {
    /// Message version: 0x00
    version: u8,
    /// Supported major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    supported: u8,
    /// Default major GNSS constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    default: u8,
    /// Currently enabled major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    enabled: u8,
    /// Maximum number of concurent Major GNSS
    /// that can be supported by this receiver
    simultaneous: u8,
    reserved1: [u8; 3],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Selected / available Constellation Mask
    #[derive(Default, Debug)]
    pub struct MonGnssConstellMask: u8 {
        /// GPS constellation
        const GPS = 0x01;
        /// GLO constellation
        const GLO = 0x02;
        /// BDC constellation
        const BDC = 0x04;
        /// GAL constellation
        const GAL = 0x08;
    }
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaPower {
    Off = 0,
    On = 1,
    DontKnow = 2,
}

#[derive(Debug, Clone)]
pub struct MonVerExtensionIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MonVerExtensionIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 30 == 0 && payload.chunks(30).all(is_cstr_valid)
    }
}

impl<'a> core::iter::Iterator for MonVerExtensionIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 30];
            self.offset += 30;
            Some(mon_ver::convert_to_str_unchecked(data))
        } else {
            None
        }
    }
}

/// Receiver/Software Version
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x04, max_payload_len = 1240)]
struct MonVer {
    #[ubx(map_type = &str, may_fail, from = mon_ver::convert_to_str_unchecked,
          is_valid = mon_ver::is_cstr_valid, get_as_ref)]
    software_version: [u8; 30],
    #[ubx(map_type = &str, may_fail, from = mon_ver::convert_to_str_unchecked,
          is_valid = mon_ver::is_cstr_valid, get_as_ref)]
    hardware_version: [u8; 10],

    /// Extended software information strings
    #[ubx(map_type = MonVerExtensionIter, may_fail,
          from = MonVerExtensionIter::new,
          is_valid = MonVerExtensionIter::is_valid)]
    extension: [u8; 0],
}

mod mon_ver {
    pub(crate) fn convert_to_str_unchecked(bytes: &[u8]) -> &str {
        let null_pos = bytes
            .iter()
            .position(|x| *x == 0)
            .expect("is_cstr_valid bug?");
        core::str::from_utf8(&bytes[0..null_pos])
            .expect("is_cstr_valid should have prevented this code from running")
    }

    pub(crate) fn is_cstr_valid(bytes: &[u8]) -> bool {
        let null_pos = match bytes.iter().position(|x| *x == 0) {
            Some(pos) => pos,
            None => {
                return false;
            },
        };
        core::str::from_utf8(&bytes[0..null_pos]).is_ok()
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x32, fixed_payload_len = 8)]
struct RxmRtcm {
    version: u8,
    flags: u8,
    sub_type: u16,
    ref_station: u16,
    msg_type: u16,
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x02, max_payload_len = 1240)]
struct EsfMeas {
    time_tag: u32,
    flags: u16,
    id: u16,
    #[ubx(
        map_type = EsfMeasDataIter,
        from = EsfMeasDataIter::new,
        size_fn = data_len,
        is_valid = EsfMeasDataIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
    #[ubx(
        map_type = Option<u32>,
        from = EsfMeas::calib_tag,
        size_fn = calib_tag_len,
    )]
    calib_tag: [u8; 0],
}

impl EsfMeas {
    fn calib_tag(bytes: &[u8]) -> Option<u32> {
        bytes.try_into().ok().map(u32::from_le_bytes)
    }
}

impl<'a> EsfMeasRef<'a> {
    fn data_len(&self) -> usize {
        ((self.flags() >> 11 & 0x1f) as usize) * 4
    }

    fn calib_tag_len(&self) -> usize {
        if self.flags() & 0x8 != 0 {
            4
        } else {
            0
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasData {
    pub data_type: u8,
    pub data_field: u32,
}

#[derive(Debug, Clone)]
pub struct EsfMeasDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfMeasDataIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(4))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 4 == 0
    }
}

impl<'a> core::iter::Iterator for EsfMeasDataIter<'a> {
    type Item = EsfMeasData;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.0.next()?.try_into().map(u32::from_le_bytes).unwrap();
        Some(EsfMeasData {
            data_type: ((data & 0x3F000000) >> 24).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
        })
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x03, max_payload_len = 1240)]
struct EsfRaw {
    msss: u32,
    #[ubx(
        map_type = EsfRawDataIter,
        from = EsfRawDataIter::new,
        is_valid = EsfRawDataIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfRawData {
    pub data_type: u8,
    pub data_field: u32,
    pub sensor_time_tag: u32,
}

#[derive(Debug, Clone)]
pub struct EsfRawDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfRawDataIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(8))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 8 == 0
    }
}

impl<'a> core::iter::Iterator for EsfRawDataIter<'a> {
    type Item = EsfRawData;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
        let sensor_time_tag = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        Some(EsfRawData {
            data_type: ((data >> 24) & 0xFF).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
            sensor_time_tag,
        })
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x15, fixed_payload_len = 36)]
struct EsfIns {
    #[ubx(map_type = EsfInsBitFlags)]
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
    pub struct EsfInsBitFlags: u32 {
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

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x05, fixed_payload_len = 32)]
struct NavAtt {
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
#[ubx(class = 0x01, id = 0x22, fixed_payload_len = 20)]
struct NavClock {
    itow: u32,
    clk_b: i32,
    clk_d: i32,
    t_acc: u32,
    f_acc: u32,
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

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x13, max_payload_len = 72)]
struct RxmSfrbx {
    gnss_id: u8,
    sv_id: u8,
    reserved1: u8,
    freq_id: u8,
    num_words: u8,
    reserved2: u8,
    version: u8,
    reserved3: u8,
    #[ubx(
        map_type = DwrdIter,
        from = DwrdIter::new,
        is_valid = DwrdIter::is_valid,
        may_fail,
    )]
    dwrd: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct DwrdIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> DwrdIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        DwrdIter(bytes.chunks_exact(4))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 4 == 0
    }
}

impl<'a> core::iter::Iterator for DwrdIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x11, fixed_payload_len = 20)]
struct NavVelECEF {
    itow: u32,
    ecef_vx: i32,
    ecef_vy: i32,
    ecef_vz: u32,
    s_acc: u32,
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
struct MgaGpsEPH {
    msg_type: u8,
    version: u8,
    sv_id: u8,
    reserved1: u8,
    fit_interval: u8,
    ura_index: u8,
    sv_health: u8,
    #[ubx(map_type = f64, scale = 2e-31)]
    tgd: i8,
    iodc: u16,
    #[ubx(map_type = f64, scale = 2e+4)]
    toc: u16,
    reserved2: u8,
    #[ubx(map_type = f64, scale = 2e-55)]
    af2: i8,
    #[ubx(map_type = f64, scale = 2e-43)]
    afl: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    af0: i32,
    #[ubx(map_type = f64, scale = 2e-5)]
    crs: i16,
    #[ubx(map_type = f64, scale = 2e-43)]
    delta_n: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    m0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    cuc: i16,
    #[ubx(map_type = f64, scale = 2e-29)]
    cus: i16,
    #[ubx(map_type = f64, scale = 2e-33)]
    e: u32,
    #[ubx(map_type = f64, scale = 2e-19)]
    sqrt_a: u32,
    #[ubx(map_type = f64, scale = 2e+4)]
    toe: u16,
    #[ubx(map_type = f64, scale = 2e-29)]
    cic: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    omega0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    cis: i16,
    #[ubx(map_type = f64, scale = 2e-5)]
    crc: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    i0: i32,
    #[ubx(map_type = f64, scale = 2e-31)]
    omega: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    omega_dot: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    idot: i16,
    reserved3: [u8; 2],
}

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x15, fixed_payload_len = 32)]
#[derive(Debug)]
pub struct RxmRawxInfo {
    pr_mes: f64,
    cp_mes: f64,
    do_mes: f32,
    gnss_id: u8,
    sv_id: u8,
    reserved2: u8,
    freq_id: u8,
    lock_time: u16,
    cno: u8,
    #[ubx(map_type = StdevFlags)]
    pr_stdev: u8,
    #[ubx(map_type = StdevFlags)]
    cp_stdev: u8,
    #[ubx(map_type = StdevFlags)]
    do_stdev: u8,
    #[ubx(map_type = TrkStatFlags)]
    trk_stat: u8,
    reserved3: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct StdevFlags: u8 {
        const STD_1 = 0x01;
        const STD_2 = 0x02;
        const STD_3 = 0x04;
        const STD_4 = 0x08;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct TrkStatFlags: u8 {
        const PR_VALID = 0x01;
        const CP_VALID = 0x02;
        const HALF_CYCLE = 0x04;
        const SUB_HALF_CYCLE = 0x08;
    }
}

#[derive(Debug, Clone)]
pub struct RxmRawxInfoIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> RxmRawxInfoIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(data.chunks_exact(32))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 32 == 0
    }
}

impl<'a> core::iter::Iterator for RxmRawxInfoIter<'a> {
    type Item = RxmRawxInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(RxmRawxInfoRef)
    }
}

/// This message is used to retrieve a unique chip identifier
#[ubx_packet_recv]
#[ubx(class = 0x27, id = 0x03, fixed_payload_len = 9)]
struct SecUniqId {
    version: u8,
    reserved1: [u8; 3],
    unique_id: [u8; 5],
}

define_recv_packets!(
    enum PacketRef {
        _ = UbxUnknownPacketRef,
        NavPosLlh,
        NavStatus,
        NavDop,
        NavPvt,
        NavSolution,
        NavVelNed,
        NavHpPosLlh,
        NavHpPosEcef,
        NavTimeUTC,
        NavTimeLs,
        NavSat,
        NavEoe,
        NavOdo,
        CfgOdo,
        MgaAck,
        MgaGpsIono,
        MgaGpsEph,
        MgaGloEph,
        AlpSrv,
        AckAck,
        AckNak,
        CfgItfm,
        CfgPrtI2c,
        CfgPrtSpi,
        CfgPrtUart,
        CfgNav5,
        CfgAnt,
        CfgTmode2,
        CfgTmode3,
        CfgTp5,
        InfError,
        InfWarning,
        InfNotice,
        InfTest,
        InfDebug,
        RxmRawx,
        TimTp,
        TimTm2,
        MonVer,
        MonGnss,
        MonHw,
        RxmRtcm,
        EsfMeas,
        EsfIns,
        HnrAtt,
        HnrIns,
        HnrPvt,
        NavAtt,
        NavClock,
        NavVelECEF,
        MgaGpsEPH,
        RxmSfrbx,
        EsfRaw,
        TimSvin,
        SecUniqId,
    }
);

#[test]
fn test_mon_ver_interpret() {
    let payload: [u8; 160] = [
        82, 79, 77, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41, 0,
        0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 70, 87, 86, 69, 82, 61, 83, 80,
        71, 32, 51, 46, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 82, 79, 84, 86,
        69, 82, 61, 49, 56, 46, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 80,
        83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];
    assert_eq!(Ok(()), <MonVerRef>::validate(&payload));
    let ver = MonVerRef(&payload);
    assert_eq!("ROM CORE 3.01 (107888)", ver.software_version());
    assert_eq!("00080000", ver.hardware_version());
    let mut it = ver.extension();
    assert_eq!("FWVER=SPG 3.01", it.next().unwrap());
    assert_eq!("PROTVER=18.00", it.next().unwrap());
    assert_eq!("GPS;GLO;GAL;BDS", it.next().unwrap());
    assert_eq!("SBAS;IMES;QZSS", it.next().unwrap());
    assert_eq!(None, it.next());
}
