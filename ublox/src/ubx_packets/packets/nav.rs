use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitflags::bitflags;

use super::{FixStatusInfo, GpsFix, SerializeUbxPacketFields};

use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ubx_checksum, MemWriter, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1,
    SYNC_CHAR_2,
};

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct NavHpPosEcefFlags: u8 {
        const INVALID_ECEF = 1;

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

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x11, fixed_payload_len = 20)]
pub struct NavVelECEF {
    pub itow: u32,
    pub ecef_vx: i32,
    pub ecef_vy: i32,
    pub ecef_vz: u32,
    pub s_acc: u32,
}

/// Geodetic Position Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 2, fixed_payload_len = 28)]
pub struct NavPosLlh {
    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// Longitude
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    pub lon: i32,

    /// Latitude
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    pub lat: i32,

    /// Height above Ellipsoid
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_meters: i32,

    /// Height above mean sea level
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_msl: i32,

    /// Horizontal Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-3)]
    pub h_ack: u32,

    /// Vertical Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-3)]
    pub v_acc: u32,
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

/// Velocity Solution in NED
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x12, fixed_payload_len = 36)]
struct NavVelNed {
    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// north velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub vel_north: i32,

    /// east velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub vel_east: i32,

    /// down velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub vel_down: i32,

    /// Speed 3-D (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub speed_3d: u32,

    /// Ground speed (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ground_speed: u32,

    /// Heading of motion 2-D (degrees)
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_degrees)]
    pub heading: i32,

    /// Speed Accuracy Estimate (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub speed_accuracy_estimate: u32,

    /// Course / Heading Accuracy Estimate (degrees)
    #[ubx(map_type = f64, scale = 1e-5)]
    pub course_heading_accuracy_estimate: u32,
}

/// High Precision Geodetic Position Solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x14, fixed_payload_len = 36)]
struct NavHpPosLlh {
    /// Message version (0 for protocol version 27)
    pub version: u8,

    pub reserved1: [u8; 3],

    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// Longitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    pub lon: i32,

    /// Latitude (deg)
    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    pub lat: i32,

    /// Height above Ellipsoid (m)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_meters: i32,

    /// Height above mean sea level (m)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_msl: i32,

    /// High precision component of longitude
    /// Must be in the range -99..+99
    /// Precise longitude in deg * 1e-7 = lon + (lonHp * 1e-2)
    #[ubx(map_type = f64, scale = 1e-9, alias = lon_hp_degrees)]
    pub lon_hp: i8,

    /// High precision component of latitude
    /// Must be in the range -99..+99
    /// Precise latitude in deg * 1e-7 = lat + (latHp * 1e-2)
    #[ubx(map_type = f64, scale = 1e-9, alias = lat_hp_degrees)]
    pub lat_hp: i8,

    /// High precision component of height above ellipsoid
    /// Must be in the range -9..+9
    /// Precise height in mm = height + (heightHp * 0.1)
    #[ubx(map_type = f64, scale = 1e-1)]
    pub height_hp_meters: i8,

    /// High precision component of height above mean sea level
    /// Must be in range -9..+9
    /// Precise height in mm = hMSL + (hMSLHp * 0.1)
    #[ubx(map_type = f64, scale = 1e-1)]
    pub height_hp_msl: i8,

    /// Horizontal accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    pub horizontal_accuracy: u32,

    /// Vertical accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    pub vertical_accuracy: u32,
}

/// High Precision Geodetic Position Solution (ECEF)
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x13, fixed_payload_len = 28)]
struct NavHpPosEcef {
    /// Message version (0 for protocol version 27)
    pub version: u8,

    pub reserved1: [u8; 3],

    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// ECEF X coordinate
    #[ubx(map_type = f64, alias = ecef_x_cm)]
    pub ecef_x: i32,

    /// ECEF Y coordinate
    #[ubx(map_type = f64, alias = ecef_y_cm)]
    pub ecef_y: i32,

    /// ECEF Z coordinate
    #[ubx(map_type = f64, alias = ecef_z_cm)]
    pub ecef_z: i32,

    /// High precision component of X
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_x + (ecef_x_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_x_hp_mm)]
    pub ecef_x_hp: i8,

    /// High precision component of Y
    /// Must be in the range -99..+99
    /// 9. Precise coordinate in cm = ecef_y + (ecef_y_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_y_hp_mm)]
    pub ecef_y_hp: i8,

    /// High precision component of Z
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_z + (ecef_z_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_z_hp_mm)]
    pub ecef_z_hp: i8,

    #[ubx(map_type = NavHpPosEcefFlags)]
    pub flags: u8,

    /// Horizontal accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    pub p_acc: u32,
}

/// Navigation Position Velocity Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x07, fixed_payload_len = 92)]
pub struct NavPvt {
    /// GPS Millisecond Time of Week
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub time_accuracy: u32,
    pub nanosecond: i32,

    /// GNSS fix Type
    #[ubx(map_type = GpsFix)]
    pub fix_type: u8,

    #[ubx(map_type = NavPvtFlags)]
    pub flags: u8,

    #[ubx(map_type = NavPvtFlags2)]
    pub flags2: u8,

    pub num_satellites: u8,

    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    pub lon: i32,

    #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
    pub lat: i32,

    /// Height above Ellipsoid
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_meters: i32,

    /// Height above mean sea level
    #[ubx(map_type = f64, scale = 1e-3)]
    pub height_msl: i32,

    pub horiz_accuracy: u32,
    pub vert_accuracy: u32,

    /// north velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub vel_north: i32,

    /// east velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub vel_east: i32,

    /// down velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub vel_down: i32,

    /// Ground speed (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub ground_speed: u32,

    /// Heading of motion 2-D (degrees)
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_degrees)]
    pub heading: i32,

    /// Speed Accuracy Estimate (m/s)
    #[ubx(map_type = f64, scale = 1e-3)]
    pub speed_accuracy_estimate: u32,

    /// Heading accuracy estimate (both motionand vehicle) (degrees)
    #[ubx(map_type = f64, scale = 1e-5)]
    pub heading_accuracy_estimate: u32,

    /// Position DOP
    pub pdop: u16,
    pub reserved1: [u8; 6],

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_of_vehicle_degrees)]
    pub heading_of_vehicle: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_degrees)]
    pub magnetic_declination: i16,

    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_accuracy_degrees)]
    pub magnetic_declination_accuracy: u16,
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
pub struct NavStatus {
    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// GPS fix Type, this value does not qualify a fix as
    /// valid and within the limits
    #[ubx(map_type = GpsFix)]
    pub fix_type: u8,

    /// Navigation Status Flags
    #[ubx(map_type = NavStatusFlags)]
    pub flags: u8,

    /// Fix Status Information
    #[ubx(map_type = FixStatusInfo)]
    pub fix_stat: u8,

    /// further information about navigation output
    #[ubx(map_type = NavStatusFlags2)]
    pub flags2: u8,

    /// Time to first fix (millisecond time tag)
    pub time_to_first_fix: u32,

    /// Milliseconds since Startup / Reset
    pub uptime_ms: u32,
}

/// Dilution of precision
#[ubx_packet_recv]
#[ubx(class = 1, id = 4, fixed_payload_len = 18)]
pub struct NavDop {
    /// GPS Millisecond Time of Week
    pub itow: u32,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub geometric_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub position_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub time_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub vertical_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub horizontal_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub northing_dop: u16,

    #[ubx(map_type = f32, scale = 1e-2)]
    pub easting_dop: u16,
}

/// End of Epoch Marker
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x61, fixed_payload_len = 4)]
pub struct NavEoe {
    /// GPS time of week for navigation epoch
    pub itow: u32,
}

/// Navigation Solution Information
#[ubx_packet_recv]
#[ubx(class = 1, id = 6, fixed_payload_len = 52)]
pub struct NavSolution {
    /// GPS Millisecond Time of Week
    pub itow: u32,

    /// Fractional part of iTOW (range: +/-500000).
    pub ftow_ns: i32,

    /// GPS week number of the navigation epoch
    pub week: i16,

    /// GPS fix Type
    #[ubx(map_type = GpsFix)]
    pub fix_type: u8,

    /// Navigation Status Flags
    #[ubx(map_type = NavStatusFlags)]
    pub flags: u8,

    /// ECEF X coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_x: i32,

    /// ECEF Y coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_y: i32,

    /// ECEF Z coordinate (meters)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_z: i32,

    /// 3D Position Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-2)]
    pub position_accuracy_estimate: u32,

    /// ECEF X velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_vx: i32,

    /// ECEF Y velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_vy: i32,

    /// ECEF Z velocity (m/s)
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_vz: i32,

    /// Speed Accuracy Estimate
    #[ubx(map_type = f64, scale = 1e-2)]
    pub speed_accuracy_estimate: u32,

    /// Position DOP
    #[ubx(map_type = f32, scale = 1e-2)]
    pub pdop: u16,

    pub reserved1: u8,
    /// Number of SVs used in Nav Solution
    pub num_sv: u8,
    pub reserved2: [u8; 4],
}

/// Further information about navigation output
/// Only for FW version >= 7.01; undefined otherwise
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum NavStatusFlags2 {
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
pub struct NavSatSvInfo {
    pub gnss_id: u8,
    pub sv_id: u8,
    pub cno: u8,
    pub elev: i8,
    pub azim: i16,
    pub pr_res: i16,

    #[ubx(map_type = NavSatSvFlags)]
    pub flags: u32,
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
pub struct NavSat {
    /// GPS time of week in ms
    pub itow: u32,

    /// Message version, should be 1
    pub version: u8,

    pub num_svs: u8,

    pub reserved: [u8; 2],

    #[ubx(
        map_type = NavSatIter,
        from = NavSatIter::new,
        is_valid = NavSatIter::is_valid,
        may_fail,
        get_as_ref,
    )]
    pub svs: [u8; 0],
}

/// Odometer solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x09, fixed_payload_len = 20)]
struct NavOdo {
    pub version: u8,
    pub reserved: [u8; 3],
    pub itow: u32,
    pub distance: u32,
    pub total_distance: u32,
    pub distance_std: u32,
}

/// Reset odometer
#[ubx_packet_send]
#[ubx(class = 0x01, id = 0x10, fixed_payload_len = 0)]
struct NavResetOdo {}

/// UTC Time Solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x21, fixed_payload_len = 20)]
pub struct NavTimeUTC {
    /// GPS Millisecond Time of Week
    pub itow: u32,
    pub time_accuracy_estimate_ns: u32,

    /// Nanoseconds of second, range -1e9 .. 1e9
    pub nanos: i32,

    /// Year, range 1999..2099
    pub year: u16,

    /// Month, range 1..12
    pub month: u8,

    /// Day of Month, range 1..31
    pub day: u8,

    /// Hour of Day, range 0..23
    pub hour: u8,

    /// Minute of Hour, range 0..59
    pub min: u8,

    /// Seconds of Minute, range 0..59
    pub sec: u8,

    /// Validity Flags
    #[ubx(map_type = NavTimeUtcFlags)]
    pub valid: u8,
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x05, fixed_payload_len = 32)]
pub struct NavAtt {
    pub itow: u32,
    pub version: u8,
    pub reserved1: [u8; 3],
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_roll)]
    pub roll: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_pitch)]
    pub pitch: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_heading)]
    pub heading: i32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_roll_accuracy)]
    pub acc_roll: u32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_pitch_accuracy)]
    pub acc_pitch: u32,
    #[ubx(map_type = f64, scale = 1e-5, alias = vehicle_heading_accuracy)]
    pub acc_heading: u32,
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x22, fixed_payload_len = 20)]
pub struct NavClock {
    pub itow: u32,
    pub clk_b: i32,
    pub clk_d: i32,
    pub t_acc: u32,
    pub f_acc: u32,
}

/// Leap second event information
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x26, fixed_payload_len = 24)]
pub struct NavTimeLs {
    /// GPS time of week of the navigation epoch in ms.
    pub itow: u32,
    ///Message version (0x00 for this version)
    pub version: u8,
    pub reserved_1: [u8; 3],
    /// Information source for the current number of leap seconds.
    /// 0: Default (hardcoded in the firmware, can be outdated)
    /// 1: Derived from time difference between GPS and GLONASS time
    /// 2: GPS
    /// 3: SBAS
    /// 4: BeiDou
    /// 5: Galileo
    /// 6: Aided data 7: Configured 8: NavIC
    /// 255: Unknown
    pub src_of_curr_ls: u8,
    /// Current number of leap seconds since start of GPS time (Jan 6, 1980). It reflects how much
    /// GPS time is ahead of UTC time. Galileo number of leap seconds is the same as GPS. BeiDou
    /// number of leap seconds is 14 less than GPS. GLONASS follows UTC time, so no leap seconds.
    pub current_ls: i8,
    /// Information source for the future leap second event.
    /// 0: No source
    /// 2: GPS
    /// 3: SBAS
    /// 4: BeiDou
    /// 5: Galileo
    /// 6: GLONASS 7: NavIC
    pub src_of_ls_change: u8,
    /// Future leap second change if one is scheduled. +1 = positive leap second, -1 = negative
    /// leap second, 0 = no future leap second event scheduled or no information available.
    pub ls_change: i8,
    /// Number of seconds until the next leap second event, or from the last leap second event if
    /// no future event scheduled. If > 0 event is in the future, = 0 event is now, < 0 event is in
    /// the past. Valid only if validTimeToLsEvent = 1.
    pub time_to_ls_event: i32,
    /// GPS week number (WN) of the next leap second event or the last one if no future event
    /// scheduled. Valid only if validTimeToLsEvent = 1.
    pub date_of_ls_gps_wn: u16,
    /// GPS day of week number (DN) for the next leap second event or the last one if no future
    /// event scheduled. Valid only if validTimeToLsEvent = 1. (GPS and Galileo DN: from 1 = Sun to
    /// 7 = Sat. BeiDou DN: from 0 = Sun to 6 = Sat.)
    pub date_of_ls_gps_dn: u16,
    pub reserved_2: [u8; 3],
    /// Validity flags see `NavTimeLsFlags`
    #[ubx(map_type = NavTimeLsFlags)]
    pub valid: u8,
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
