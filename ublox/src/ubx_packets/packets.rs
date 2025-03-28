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
    ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_recv_send, ubx_packet_send,
};

use crate::error::{MemWriterError, ParserError};
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
use crate::ubx_packets::packets::mon_ver::is_cstr_valid;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;

use super::{
    ubx_checksum, MemWriter, Position, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta,
    SYNC_CHAR_1, SYNC_CHAR_2,
};

pub use packetref::PacketRef;
pub mod packetref;

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

    /// north velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_north: i32,

    /// east velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_east: i32,

    /// down velocity [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    vel_down: i32,

    /// Speed 3-D [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    speed_3d: u32,

    /// Ground speed [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    ground_speed: u32,

    /// Heading of motion 2-D [deg]
    #[ubx(map_type = f64, scale = 1e-5, alias = heading_degrees)]
    heading: i32,

    /// Speed Accuracy Estimate [m/s]
    #[ubx(map_type = f64, scale = 1e-2)]
    speed_accuracy_estimate: u32,

    /// Course / Heading Accuracy Estimate [deg]
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

/// Navigation clock solution,
/// current receiver clock bias and drift estimates
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x22, fixed_payload_len = 20)]
struct NavClock {
    /// GPS time of week, in s
    #[ubx(map_type = f64, scale = 1e-3)]
    itow: u32,
    /// Receiver clock bias (offset) in s
    #[ubx(map_type = f64, scale = 1.0E-9)]
    clk_bias: i32,
    /// Clock drift (offset variations) [s/s]
    #[ubx(map_type = f64, scale = 1.0E-9)]
    clk_drift: i32,
    /// time accuracy estimate
    #[ubx(map_type = f64, scale = 1.0E-9)]
    time_acc: u32,
    /// frequency accuracy estimate [s/s]
    #[ubx(map_type = f64, scale = 1.0E-12)]
    freq_acc: u32,
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

    nanosec: i32,

    /// GNSS Fix Type
    #[ubx(map_type = GpsFix)]
    fix_type: u8,

    #[ubx(map_type = NavPvtFlags)]
    flags: u8,

    #[ubx(map_type = NavPvtFlags2)]
    flags2: u8,

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

    reserved1: [u8; 5],
    #[ubx(map_type = NavPvtFlags3)]
    flags3: u8,

    #[ubx(map_type = f64, scale = 1e-5, alias = heading_vehicle)]
    head_vehicle: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination)]
    magnetic_declination: i16,

    #[ubx(map_type = f64, scale = 1e-2, alias = magnetic_declination_accuracy)]
    magnetic_declination_accuracy: u16,
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

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NavPvtFlags3 {
    invalid_llh: bool,
    #[cfg(feature = "ubx_proto23")]
    age_differential_correction: u8,
}

impl NavPvtFlags3 {
    pub fn invalid_llh(&self) -> bool {
        self.invalid_llh
    }

    /// F9R interface descritpion document specifies that this byte is unused
    #[cfg(feature = "ubx_proto23")]
    pub fn age_differential_correction(&self) -> u8 {
        self.age_differential_correction
    }
}

impl From<u8> for NavPvtFlags3 {
    #[cfg(feature = "ubx_proto23")]
    fn from(val: u8) -> Self {
        const AGE_DIFFERENTIAL_CORRECTION_MASK: u8 = 0b11110;
        let invalid = val & 0x01 == 1;
        // F9R interface descritpion document specifies that this byte is unused
        // We can read it ... but we don't expose it
        let age_differential_correction = val & AGE_DIFFERENTIAL_CORRECTION_MASK;
        Self {
            invalid_llh: invalid,
            age_differential_correction,
        }
    }

    #[cfg(not(feature = "ubx_proto23"))]
    fn from(val: u8) -> Self {
        let invalid = val & 0x01 == 1;
        Self {
            invalid_llh: invalid,
        }
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

#[cfg(feature = "ubx_proto23")]
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x3c, fixed_payload_len = 40)]
struct NavRelPosNed {
    /// Message version (0x00 for this version)
    version: u8,

    reserved1: u8,

    /// Reference station ID. Must be in the range 0..4095
    ref_station_id: u16,

    /// GPS Millisecond time of week of the navigation epoch.
    itow: u32,

    /// North component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_n_cm)]
    rel_pos_n: i32,

    /// East component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_e_cm)]
    rel_pos_e: i32,

    /// Down component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_d_cm)]
    rel_pos_d: i32,

    /// High-precision North component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full North component of relative position vector in cm = rel_pos_n + (rel_pos_hpn * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_n_mm)]
    rel_pos_hpn: i8,

    /// High-precision East component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full East component of relative position vector in cm = rel_pos_e + (rel_pos_hpe * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_e_mm)]
    rel_pos_hpe: i8,

    /// High-precision Down component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full Down component of relative position vector in cm = rel_pos_d + (rel_pos_hpd * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_d_mm)]
    rel_pos_hpd: i8,

    reserved2: u8,
    /// Accuracy of relative position North component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_n_mm)]
    acc_n: u32,

    /// Accuracy of relative position East component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_e_mm)]
    acc_e: u32,

    /// Accuracy of relative position Down component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_d_mm)]
    acc_d: u32,

    #[ubx(map_type = NavRelPosNedFlags)]
    flags: u32,
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x3c, fixed_payload_len = 64)]
struct NavRelPosNed {
    /// Message version (0x01 for this version)
    version: u8,

    _reserved0: u8,

    /// Reference station ID. Must be in the range 0..4095
    ref_station_id: u16,

    /// GPS Millisecond time of week of the navigation epoch.
    itow: u32,

    /// North component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_n_cm)]
    rel_pos_n: i32,
    /// East component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_e_cm)]
    rel_pos_e: i32,
    /// Down component of relative position vector
    #[ubx(map_type = f64, alias = rel_pos_d_cm)]
    rel_pos_d: i32,
    /// Length of the relative position vector
    #[ubx(map_type = f64, alias = rel_pos_length_cm)]
    rel_pos_length: i32,
    /// Heading of the relative position vector
    #[ubx(map_type = f64, scale = 1e-5, alias = rel_pos_heading_degrees)]
    rel_pos_heading: i32,

    _reserved1: u32,

    /// High-precision North component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full North component of relative position vector in cm = rel_pos_n + (rel_pos_hpn * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_n_mm)]
    rel_pos_hpn: i8,
    /// High-precision East component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full East component of relative position vector in cm = rel_pos_e + (rel_pos_hpe * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_e_mm)]
    rel_pos_hpe: i8,
    /// High-precision Down component of relative position vector.
    /// Must be in the range -99 to +99.
    /// Full Down component of relative position vector in cm = rel_pos_d + (rel_pos_hpd * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_d_mm)]
    rel_pos_hpd: i8,
    /// High-precision Length of the relative position vector.
    /// Must be in the range -99 to +99.
    /// Full Length of the relative position vector in cm = rel_pos_length + (rel_pos_hp_length * 1e-2)
    #[ubx(map_type = f64, scale = 1e-1, alias = rel_pos_hp_length_mm)]
    rel_pos_hp_length: i8,

    /// Accuracy of relative position North component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_n_mm)]
    acc_n: u32,
    /// Accuracy of relative position East component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_e_mm)]
    acc_e: u32,
    /// Accuracy of relative position Down component
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_d_mm)]
    acc_d: u32,
    /// Accuracy of Length of relative position vector
    #[ubx(map_type = f64, scale = 1e-1, alias = acc_length_mm)]
    acc_length: u32,
    /// Accuracy of Heading of relative position vector
    #[ubx(map_type = f64, scale = 1e-5, alias = acc_heading_degrees)]
    acc_heading: u32,

    _reserved2: u32,

    #[ubx(map_type = NavRelPosNedFlags)]
    flags: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum CarrierPhaseRangeSolutionStatus {
    /// No carrier phase range solution
    NoSolution,
    /// Carrier phase range solution with floating ambiguities
    SolutionWithFloatingAmbiguities,
    /// Carrier phase range solution with fixed ambiguities
    SolutionWithFixedAmbiguities,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NavRelPosNedFlags(u32);

impl NavRelPosNedFlags {
    pub fn gnss_fix_ok(&self) -> bool {
        self.0 & 0x1 != 0
    }

    pub fn diff_soln(&self) -> bool {
        (self.0 >> 1) & 0x1 != 0
    }

    pub fn rel_pos_valid(&self) -> bool {
        (self.0 >> 2) & 0x1 != 0
    }

    pub fn carr_soln(&self) -> CarrierPhaseRangeSolutionStatus {
        match (self.0 >> 3) & 0x3 {
            0 => CarrierPhaseRangeSolutionStatus::NoSolution,
            1 => CarrierPhaseRangeSolutionStatus::SolutionWithFloatingAmbiguities,
            2 => CarrierPhaseRangeSolutionStatus::SolutionWithFixedAmbiguities,
            unknown => panic!("Unexpected 2-bit bitfield value {}!", unknown),
        }
    }

    pub fn is_moving(&self) -> bool {
        (self.0 >> 5) & 0x1 != 0
    }

    pub fn ref_pos_miss(&self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn ref_obs_miss(&self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    #[cfg(feature = "ubx_proto27")]
    pub fn rel_pos_heading_valid(&self) -> bool {
        (self.0 >> 8) & 0x1 != 0
    }

    #[cfg(feature = "ubx_proto27")]
    pub fn rel_pos_normalized(&self) -> bool {
        (self.0 >> 9) & 0x1 != 0
    }

    pub const fn from(x: u32) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavRelPosNedFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg_struct = f.debug_struct("NavRelPosNedFlags");
        dbg_struct
            .field("gnss_fix_ok", &self.gnss_fix_ok())
            .field("diff_soln", &self.diff_soln())
            .field("rel_pos_valid", &self.rel_pos_valid())
            .field("carr_soln", &self.carr_soln())
            .field("is_moving", &self.is_moving())
            .field("ref_pos_miss", &self.ref_pos_miss())
            .field("ref_obs_miss", &self.ref_obs_miss());

        #[cfg(feature = "ubx_proto27")]
        dbg_struct
            .field("rel_pos_heading_valid", &self.rel_pos_heading_valid())
            .field("rel_pos_normalized", &self.rel_pos_normalized());

        dbg_struct.finish()
    }
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
            _ => NavSatQualityIndicator::Invalid,
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
    Invalid,
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

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavSigFlags(u16);

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
impl NavSigFlags {
    /* Re-use the NavSatHealth enum for the signal health */
    pub fn health(self) -> NavSatSvHealth {
        let bits = self.0 & 0x3;
        match bits {
            1 => NavSatSvHealth::Healthy,
            2 => NavSatSvHealth::Unhealthy,
            x => NavSatSvHealth::Unknown(x as u8),
        }
    }

    pub fn pr_smoothed(self) -> bool {
        (self.0 >> 2) & 0x1 != 0
    }

    pub fn pr_used(self) -> bool {
        (self.0 >> 3) & 0x1 != 0
    }

    pub fn cr_used(self) -> bool {
        (self.0 >> 4) & 0x1 != 0
    }

    pub fn do_used(self) -> bool {
        (self.0 >> 5) & 0x1 != 0
    }

    pub fn pr_corr_used(self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn cr_corr_used(self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    pub fn do_corr_used(self) -> bool {
        (self.0 >> 8) & 0x1 != 0
    }

    pub const fn from(x: u16) -> Self {
        Self(x)
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
impl fmt::Debug for NavSigFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSatSvFlags")
            .field("health", &self.health())
            .field("pr_smoothed", &self.pr_smoothed())
            .field("pr__used", &self.pr_used())
            .field("cr__used", &self.cr_used())
            .field("do__used", &self.do_used())
            .field("pr_corr_used", &self.pr_corr_used())
            .field("cr_corr_used", &self.cr_corr_used())
            .field("do_corr_used", &self.do_corr_used())
            .finish()
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, fixed_payload_len = 16)]
struct NavSigInfo {
    gnss_id: u8,
    sv_id: u8,
    sig_id: u8,
    freq_id: u8,
    pr_res: i16,
    cno: u8,
    quality_ind: u8,
    corr_source: u8,
    ion_model: u8,
    #[ubx(map_type = NavSigFlags)]
    flags: u16,
    reserved: [u8; 4],
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
#[derive(Clone)]
pub struct NavSigIter<'a> {
    data: &'a [u8],
    offset: usize,
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
impl<'a> NavSigIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % 16 == 0
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
impl<'a> core::iter::Iterator for NavSigIter<'a> {
    type Item = NavSigInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 16];
            self.offset += 16;
            Some(NavSigInfoRef(data))
        } else {
            None
        }
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
impl fmt::Debug for NavSigIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSigIter").finish()
    }
}

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x43, max_payload_len = 1240)]
struct NavSig {
    /// GPS time of week in ms
    itow: u32,

    /// Message version, should be 0
    version: u8,

    num_sigs: u8,

    reserved: u16,

    #[ubx(map_type = NavSigIter,
        may_fail,
        is_valid = NavSigIter::is_valid,
        from = NavSigIter::new,
        get_as_ref)]
    sigs: [u8; 0],
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

/// Synchronization management configuration frame
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x62,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
#[derive(Debug)]
struct CfgSmgr {
    version: u8,
    /// Minimum # of GNSS fixes before we
    /// commit to use it as a source
    min_gnss_fix: u8,
    /// Maximum frequency rate change, in ppb/sec,
    /// when disciplining. Must be < 30 ppb/s.
    #[ubx(map_type = f32, scale = 1.0)]
    max_freq_change_rate: u16,
    /// Maximum phase correction rate, in ns/s
    /// in coherent time pulse mode.
    /// Must be < 100 ns/s
    max_phase_corr_rate: u16,
    reserved1: u16,
    /// Limit possible deviation in ppb,
    /// before UBX-TIM-TOS indicates that frequency
    /// is out of tolerance
    #[ubx(map_type = f32, scale = 1.0)]
    freq_tolerance: u16,
    /// Limit possible deviation, in ns,
    /// before UBX-TIM-TOS indicates that pulse
    /// is out of tolerance
    #[ubx(map_type = f32, scale = 1.0)]
    time_tolerance: u16,
    /// Message configuration, see [CfgSmgrMsgFlags]
    #[ubx(map_type = CfgSmgrMsgFlags)]
    msg: u16,
    /// Maximum slew rate, in s/s
    #[ubx(map_type = f32, scale = 1.0E-6)]
    max_slew_rate: u16,
    /// Configuration flags, see [CfgSmgrFlags]
    #[ubx(map_type = CfgSmgrFlags)]
    flags: u32,
}

/// Synchronization Manager message flags
#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Sync manager message flags
    #[derive(Default, Debug)]
    pub struct CfgSmgrMsgFlags: u16 {
        /// Report internal oscillator offset estimate from oscillator model
        const MEAS_INTERNAL1 = 0x01;
        /// Report internal oscillator offset relative to GNSS
        const MEAS_GNSS = 0x02;
        /// Report internal oscillator offset relative to EXTINT0 source
        const MEAS_EXTINT0 = 0x04;
        /// Report internal oscillator offset relative to EXTINT1 source
        const MEAS_EXTINT1 = 0x08;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Synchronization Manager config flags
    #[derive(Default, Debug)]
    pub struct CfgSmgrFlags: u32 {
        /// Disable internal Osc. disciplining
        const DISABLE_INTERNAL = 0x01;
        /// Disable external Osc. disciplining
        const DISABLE_EXTERNAL = 0x02;
        /// Reference selection preference,
        /// `Best Phase accuracy` when set,
        /// `Best frequency accuracy` when unset
        const BEST_PHASE_ACCURACY_PREFERENCE = 0x04;
        /// Enables GNSS as sync source
        const ENABLE_GNSS = 0x08;
        /// Enables ExtInt0 as sync source
        const ENABLE_EXTINT0 = 0x10;
        /// Enables ExtInt1 as sync source
        const ENABLE_EXTINT1 = 0x20;
        /// Enable host measurements of the internal
        /// oscillator as sync source.
        /// TimSmeasData0 frame should be used
        /// to send measurements data
        const ENABLE_HOST_MEAS_INT = 0x40;
        /// Enable host measurements of the external
        /// oscillator as sync source.
        /// TimSmeasData1 frame should be used
        /// to send measurements data
        const ENABLE_HOST_MEAS_EXT = 0x80;
        /// Uses any fix when asserted,
        /// otherwise, only `over determined` navigation
        /// solutions are used
        const USE_ANY_FIX = 0x100;
        /// MaxSlewRate field is discarded when asserted,
        /// otherwise MaxSlewRate field is used for
        /// maximum time correction, in corrective fime pulse mode
        const DISABLE_MAX_SLEW_RATE = 0x200;
        /// Issues UBX-TIME-TOS warning when frequency uncertainty
        /// exceeds `freq_tolerance`
        const ISSUE_FREQ_WARNING = 0x400;
        /// Issues UBX-TIME-TOS warning when time uncertainty
        /// exceeds `time_tolerance`
        const ISSUE_TIME_WARNING = 0x800;
        /// Coherence Pulses. Time phase offsets will be corrected
        /// gradually by varying the GNSS oscillator rate within
        /// freq. tolerance limits.
        const TP_COHERENT_PULSES = 0x1000;
        /// Non coherence Pulses. Time phase offsets will be corrected
        /// as quickly as allowed by specified `max_slew_rate`
        const TP_NON_COHERENCE_PULSES = 0x2000;
        /// Post init. coherent pulses.
        /// Starts off in non coherent mode, then automatically switches
        /// to coherent pulse mode, when PLL is locked
        const TP_POST_INIT_COHERENT_PULSES = 0x4000;
        /// Disable automatic storage of oscillator offset
        const DISABLE_OFFSET_STORAGE = 0x8000;
    }
}

/// Information message config
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

impl AckAckRef<'_> {
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

impl AckNakRef<'_> {
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

impl core::iter::Iterator for CfgValIter<'_> {
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
        if x < T::from_i8(i8::MIN).unwrap() {
            i8::MIN
        } else if x > T::from_i8(i8::MAX).unwrap() {
            i8::MAX
        } else {
            x.to_i8().unwrap()
        }
    }

    fn as_i16(self, x: T) -> i16 {
        let x = (x * self.0).round();
        if x < T::from_i16(i16::MIN).unwrap() {
            i16::MIN
        } else if x > T::from_i16(i16::MAX).unwrap() {
            i16::MAX
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

/// Get/set IMU-mount misalignment configuration
/// Only available for ADR products
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x56,
    fixed_payload_len = 12,
    flags = "default_for_builder"
)]
struct CfgEsfAlg {
    #[ubx(map_type = CfgEsfAlgFlags)]
    flags: u32,

    /// IMU mount yaw angle [0, 360]
    #[ubx(map_type = f64, scale = 1e-2, alias = yaw)]
    yaw: u32,

    /// IMU mount pitch angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = pitch)]
    pitch: i16,

    /// IMU mount roll angle [-180, 180]
    #[ubx(map_type = f64, scale = 1e-2, alias = roll)]
    roll: i16,
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CfgEsfAlgFlags {
    /// Not writable, only readable
    version: u8,
    /// Auto alignment status
    auto_alignment: bool,
}

impl From<u32> for CfgEsfAlgFlags {
    fn from(cfg: u32) -> Self {
        let version = cfg.to_le_bytes()[0];
        let auto_alignment = ((cfg >> 8) & 0x01) == 1;
        Self {
            version,
            auto_alignment,
        }
    }
}

impl CfgEsfAlgFlags {
    pub fn auto_imu_mount_alg_on(self) -> bool {
        self.auto_alignment
    }

    pub fn set_auto_imu_mount_alg(&mut self, enable: bool) {
        self.auto_alignment = enable
    }

    pub fn version(self) -> u8 {
        self.version
    }

    const fn into_raw(self) -> u32 {
        (self.auto_alignment as u32) << 8
    }
}

impl fmt::Debug for CfgEsfAlgFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("autoMntAlgOn", &self.auto_imu_mount_alg_on())
            .field("version", &self.version())
            .finish()
    }
}

/// Get/set wheel-tick configuration
/// Only available for ADR products
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x82,
    fixed_payload_len = 32,
    flags = "default_for_builder"
)]
struct CfgEsfWt {
    version: u8,

    #[ubx(map_type = CfgEsfWtFlags1)]
    flags1: u8,

    #[ubx(map_type = CfgEsfWtFlags2)]
    flags2: u8,
    reserved1: u8,

    /// Wheel tick scaling factor
    #[ubx(map_type = f64, scale = 1e-6)]
    wt_factor: u32,

    /// Wheel tick quantization
    #[ubx(map_type = f64, scale = 1e-6)]
    wt_quant_error: u32,

    /// Wheel tick counter maximum value
    wt_count_max: u32,

    /// Wheel tick latency due to e.g. CAN bus
    wt_latency: u16,

    /// Nominal wheel tick data frequency
    wt_frequency: u8,

    #[ubx(map_type = CfgEsfWtFlags3)]
    flags3: u8,

    /// Speed sensor dead band
    speed_dead_band: u16,

    reserved2: [u8; 10],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags1 : u8 {
        /// Use combined rear wheel-ticks
        const COMBINED_TICKS = 0x01;
        /// Low-speed COG filter enabled flag
        const USE_WHEEL_TICK_SPEED = 0x10;
        /// Direction pin polarity
        const DIR_PIN_POLARITY = 0x20;
        /// Use wheel tick pin for speed measurement
        const USE_WT_PIN = 0x40;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags2 : u8 {
        const AUTO_WT_COUNT_MAX_OFF = 0x01;
        const AUTO_DIR_PIN_POL_OFF = 0x02;
        const AUTO_SOFTWARE_WT_OFF = 0x04;
        const AUTO_USE_WT_SPEED_OFF = 0x08;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags3 : u8 {
        /// Count both rising and falling edges of wheel-tick
        const CNT_BOTH_EDGES = 0x01;
    }
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

#[cfg(feature = "ubx_proto23")]
/// Time pulse time & frequency data
#[ubx_packet_recv]
#[ubx(class = 0x0D, id = 0x12, fixed_payload_len = 56)]
#[derive(Debug)]
struct TimTos {
    version: u8,
    /// GNSS system used for reporting GNSS time
    gnss_id: u8,
    reserved1: [u8; 2],
    #[ubx(map_type = TimTosFlags)]
    flags: u32,
    /// Year of UTC time
    year: u16,
    /// Month of UTC time
    month: u8,
    /// Day of UTC time
    day: u8,
    /// Hour of UTC time
    hour: u8,
    /// Minute of UTC time
    minute: u8,
    /// Second of UTC time
    second: u8,
    /// UTC standard identifier
    #[ubx(map_type = CfgNav5UtcStandard, may_fail)]
    utc_standard: u8,
    /// Time offset between preceding pulse and UTC top of second
    utc_offset: i32,
    /// Uncertainty of UTC offset
    utc_uncertainty: u32,
    /// GNSS week number
    week: u32,
    /// GNSS time of week
    tow: u32,
    /// Time offset between the preceding pulse and GNSS top of second
    gnss_offset: i32,
    /// Uncertainty of GNSS offset
    gnss_uncertainty: u32,
    #[ubx(map_type = f64, scale = 2.0E-8)]
    int_osc_offset: i32,
    #[ubx(map_type = f64, scale = 2.0E-8)]
    int_osc_uncertainty: u32,
    #[ubx(map_type = f64, scale = 2.0E-8)]
    ext_osc_offset: i32,
    #[ubx(map_type = f64, scale = 2.0E-8)]
    ext_osc_uncertainty: u32,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct TimTosFlags: u32 {
        /// Currently in a leap second
        const LEAP_NOW = 0x01;
        /// Leap second in current minute
        const LEAP_CURRENT_MINUTE = 0x02;
        /// Positive leap second
        const POSITIVE_LEAP = 0x04;
        /// Time pulse is within tolerance limit (Ubx-CfgSmgr)
        const TIME_IN_LIMIT = 0x08;
        /// Internal oscillator is within tolerance limit (Ubx-CfgSmgr)
        const INT_OSC_IN_LIMIT = 0x10;
        /// Exteranl oscillator is within tolerance limit (Ubx-CfgSmgr)
        const EXT_OSC_IN_LIMIT = 0x20;
        /// GNSS Time is valid
        const GNSS_TIME_IS_VALID = 0x40;
        /// Disciplining source is GNSS
        const GNSS_DISCIPLINING = 0x80;
        /// Disciplining source is EXTINT0
        const EXTINT0_DISCIPLINING = 0x100;
        /// Disciplining source is EXTINT1
        const EXTINT1_DISCIPLINING = 0x200;
        /// Internal Osc measured by host
        const INT_MEAS_BY_HOST = 0x400;
        /// External Osc measured by host
        const EXT_MEAS_BY_HOST = 0x800;
        /// (T)RAIM system currently active
        const RAIM = 0x1000;
        /// Coherent pulse generation active
        const COHERENT_PULSE = 0x2000;
        /// Time pulse is locked
        const TIME_PULSE_LOCKED = 0x4000;
    }
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
    itow: u32,
    #[ubx(map_type = EsfMeasFlags, from = EsfMeasFlags)]
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

impl EsfMeasRef<'_> {
    fn data_len(&self) -> usize {
        self.flags().num_meas() as usize * 4
    }

    fn calib_tag_len(&self) -> usize {
        if self.flags().calib_tag_valid() {
            4
        } else {
            0
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasData {
    pub data_type: EsfSensorType,
    pub data_field: i32,
}

#[derive(Debug)]
pub enum SensorData {
    Tick(i32),
    Value(f32),
}

impl EsfMeasData {
    pub fn direction(&self) -> i8 {
        if self.data_field.is_negative() {
            -1
        } else {
            1
        }
    }

    pub fn value(&self) -> SensorData {
        match self.data_type {
            EsfSensorType::FrontLeftWheelTicks
            | EsfSensorType::FrontRightWheelTicks
            | EsfSensorType::RearLeftWheelTicks
            | EsfSensorType::RearRightWheelTicks
            | EsfSensorType::SpeedTick => {
                let tick = (self.data_field & 0x7FFFFF) * (self.direction() as i32);
                SensorData::Tick(tick)
            },
            EsfSensorType::Speed => {
                let value = (self.data_field & 0x7FFFFF) as f32 * (self.direction() as f32) * 1e-3;
                SensorData::Value(value)
            },
            EsfSensorType::GyroX | EsfSensorType::GyroY | EsfSensorType::GyroZ => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 2_f32.powi(-12);
                SensorData::Value(value)
            },
            EsfSensorType::AccX | EsfSensorType::AccY | EsfSensorType::AccZ => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 2_f32.powi(-10);
                SensorData::Value(value)
            },
            EsfSensorType::GyroTemp => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 1e-2;
                SensorData::Value(value)
            },
            _ => SensorData::Value(0f32),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EsfMeasDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfMeasDataIter<'a> {
    const BLOCK_SIZE: usize = 4;
    const DIRECTION_INDICATOR_BIT: usize = 23;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfMeasDataIter<'_> {
    type Item = EsfMeasData;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..Self::BLOCK_SIZE].try_into().unwrap());
        let mut data_field = (data & 0x7FFFFF) as i32;
        let backward = ((data >> Self::DIRECTION_INDICATOR_BIT) & 0x01) == 1;
        // Turn value into valid negative integer representation
        if backward {
            data_field ^= 0x800000;
            data_field = data_field.wrapping_neg();
        }

        Some(EsfMeasData {
            data_type: (((data >> 24) & 0x3F) as u8).into(),
            data_field,
        })
    }
}

/// UBX-ESF-MEAS flags
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasFlags(u16);

impl EsfMeasFlags {
    pub fn time_mark_sent(self) -> u8 {
        ((self.0) & 0x2) as u8
    }

    pub fn time_mark_edge(self) -> bool {
        (self.0 >> 2) & 0x01 != 0
    }

    pub fn calib_tag_valid(self) -> bool {
        (self.0 >> 3) & 0x01 != 0
    }

    pub fn num_meas(self) -> u8 {
        ((self.0 >> 11) & 0x1F) as u8
    }
}

impl fmt::Debug for EsfMeasFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("timeMarkSent", &self.time_mark_sent())
            .field("timeMarkEdge", &self.time_mark_edge())
            .field("calibTagValid", &self.calib_tag_valid())
            .field("numMeas", &self.num_meas())
            .finish()
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
    const BLOCK_SIZE: usize = 8;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfRawDataIter<'_> {
    type Item = EsfRawData;

    fn next(&mut self) -> Option<Self::Item> {
        const HALF_BLOCK: usize = 4;
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..HALF_BLOCK].try_into().unwrap());
        let sensor_time_tag =
            u32::from_le_bytes(chunk[HALF_BLOCK..Self::BLOCK_SIZE].try_into().unwrap());
        Some(EsfRawData {
            data_type: ((data >> 24) & 0xFF).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
            sensor_time_tag,
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfAlgStatus {
    UserDefinedAngles = 0,
    RollPitchAlignmentOngoing = 1,
    RollPitchYawAlignmentOngoing = 2,
    CoarseAlignment = 3,
    FineAlignment = 4,
    /// Only three bits are reserved for this field
    Invalid = 5,
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x14, fixed_payload_len = 16)]
struct EsfAlg {
    itow: u32,
    /// Message version: 0x01 for M8L
    version: u8,

    #[ubx(map_type = EsfAlgFlags, from = EsfAlgFlags)]
    flags: u8,

    #[ubx(map_type = EsfAlgError)]
    error: u8,

    reserved1: u8,

    /// IMU mount yaw angle [0, 360]
    #[ubx(map_type = f64, scale = 1e-2, alias = yaw)]
    yaw: u32,

    /// IMU mount pitch angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = pitch)]
    pitch: i16,

    /// IMU mount roll angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = roll)]
    roll: i16,
}

/// UBX-ESF-ALG flags
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfAlgFlags(u8);

impl EsfAlgFlags {
    pub fn flags_raw(&self) -> u8 {
        self.0
    }
    pub fn auto_imu_mount_alg_on(self) -> bool {
        (self.0) & 0x1 != 0
    }

    pub fn status(self) -> EsfAlgStatus {
        let bits = (self.0 >> 1) & 0x07;
        match bits {
            0 => EsfAlgStatus::UserDefinedAngles,
            1 => EsfAlgStatus::RollPitchAlignmentOngoing,
            2 => EsfAlgStatus::RollPitchYawAlignmentOngoing,
            3 => EsfAlgStatus::CoarseAlignment,
            4 => EsfAlgStatus::FineAlignment,
            _ => EsfAlgStatus::Invalid,
        }
    }
}

impl fmt::Debug for EsfAlgFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("autoMntAlgOn", &self.auto_imu_mount_alg_on())
            .field("status", &self.status())
            .finish()
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct EsfAlgError: u8 {
        const TILT_ALG_ERROR = 0x01;
        const YAW_ALG_ERROR = 0x02;
        const ANGLE_ERROR = 0x04;
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x15, fixed_payload_len = 36)]
struct EsfIns {
    #[ubx(map_type = EsfInsBitFlags)]
    bitfield: u32,
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
#[ubx(class = 0x10, id = 0x10, max_payload_len = 1240)]
struct EsfStatus {
    itow: u32,
    /// Version is 2 for M8L spec
    version: u8,

    #[ubx(map_type = EsfInitStatus1, from = EsfInitStatus1)]
    init_status1: u8,

    #[ubx(map_type = EsfInitStatus2, from = EsfInitStatus2)]
    init_status2: u8,

    reserved1: [u8; 5],

    #[ubx(map_type = EsfStatusFusionMode)]
    fusion_mode: u8,

    reserved2: [u8; 2],

    num_sens: u8,

    #[ubx(
        map_type = EsfSensorStatusIter,
        from = EsfSensorStatusIter::new,
        is_valid = EsfSensorStatusIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EsfStatusFusionMode {
    Initializing = 0,
    Fusion = 1,
    Suspended = 2,
    Disabled = 3,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfInitStatus1(u8);

impl EsfInitStatus1 {
    const WHEEL_TICK_MASK: u8 = 0x03;
    const MOUNTING_ANGLE_STATUS_MASK: u8 = 0x07;
    const INS_STATUS_MASK: u8 = 0x03;

    pub fn wheel_tick_init_status(self) -> EsfStatusWheelTickInit {
        let bits = (self.0) & Self::WHEEL_TICK_MASK;
        match bits {
            0 => EsfStatusWheelTickInit::Off,
            1 => EsfStatusWheelTickInit::Initializing,
            2 => EsfStatusWheelTickInit::Initialized,
            _ => EsfStatusWheelTickInit::Invalid,
        }
    }

    pub fn wheel_tick_init_status_raw(self) -> u8 {
        self.wheel_tick_init_status() as u8
    }

    pub fn mounting_angle_status(self) -> EsfStatusMountAngle {
        let bits = (self.0 >> 2) & Self::MOUNTING_ANGLE_STATUS_MASK;
        match bits {
            0 => EsfStatusMountAngle::Off,
            1 => EsfStatusMountAngle::Initializing,
            2 | 3 => EsfStatusMountAngle::Initialized,
            _ => EsfStatusMountAngle::Invalid,
        }
    }

    pub fn mount_angle_status_raw(self) -> u8 {
        self.mounting_angle_status() as u8
    }

    pub fn ins_initialization_status(self) -> EsfStatusInsInit {
        let bits = (self.0 >> 5) & Self::INS_STATUS_MASK;
        match bits {
            0 => EsfStatusInsInit::Off,
            1 => EsfStatusInsInit::Initializing,
            2 => EsfStatusInsInit::Initialized,
            _ => EsfStatusInsInit::Invalid,
        }
    }

    pub fn ins_initialization_status_raw(self) -> u8 {
        self.ins_initialization_status() as u8
    }
}

impl fmt::Debug for EsfInitStatus1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("initStatus1")
            .field("wtInitStatus", &self.wheel_tick_init_status())
            .field("mntAlgStatus", &self.mounting_angle_status())
            .field("insInitStatus", &self.ins_initialization_status())
            .finish()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusWheelTickInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusMountAngle {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusInsInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfInitStatus2(u8);

impl EsfInitStatus2 {
    pub fn imu_init_status_raw(self) -> u8 {
        self.imu_init_status() as u8
    }

    pub fn imu_init_status(self) -> EsfStatusImuInit {
        let bits = (self.0) & 0x02;
        match bits {
            0 => EsfStatusImuInit::Off,
            1 => EsfStatusImuInit::Initializing,
            2 => EsfStatusImuInit::Initialized,
            _ => EsfStatusImuInit::Invalid,
        }
    }
}

impl fmt::Debug for EsfInitStatus2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("initStatus2")
            .field("imuInitStatus", &self.imu_init_status())
            .finish()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusImuInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfSensorStatus {
    sens_status1: SensorStatus1,
    sens_status2: SensorStatus2,
    freq: u16,
    faults: EsfSensorFaults,
}

impl EsfSensorStatus {
    pub fn freq(&self) -> u16 {
        self.freq
    }

    pub fn faults(&self) -> EsfSensorFaults {
        self.faults
    }

    pub fn faults_raw(&self) -> u8 {
        self.faults().into_raw()
    }

    pub fn sensor_type(&self) -> EsfSensorType {
        self.sens_status1.sensor_type
    }

    pub fn sensor_type_raw(&self) -> u8 {
        self.sensor_type() as u8
    }

    pub fn sensor_used(&self) -> bool {
        self.sens_status1.used
    }

    pub fn sensor_ready(&self) -> bool {
        self.sens_status1.ready
    }

    pub fn calibration_status(&self) -> EsfSensorStatusCalibration {
        self.sens_status2.calibration_status
    }

    pub fn calibration_status_raw(&self) -> u8 {
        self.calibration_status() as u8
    }

    pub fn time_status(&self) -> EsfSensorStatusTime {
        self.sens_status2.time_status
    }

    pub fn time_status_raw(&self) -> u8 {
        self.time_status() as u8
    }
}

#[derive(Clone, Debug)]
pub struct EsfSensorStatusIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfSensorStatusIter<'a> {
    const BLOCK_SIZE: usize = 4;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfSensorStatusIter<'_> {
    type Item = EsfSensorStatus;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..Self::BLOCK_SIZE].try_into().unwrap());
        Some(EsfSensorStatus {
            sens_status1: ((data & 0xFF) as u8).into(),
            sens_status2: (((data >> 8) & 0xFF) as u8).into(),
            freq: ((data >> 16) & 0xFF).try_into().unwrap(),
            faults: (((data >> 24) & 0xFF) as u8).into(),
        })
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SensorStatus1 {
    sensor_type: EsfSensorType,
    used: bool,
    ready: bool,
}

impl From<u8> for SensorStatus1 {
    fn from(s: u8) -> Self {
        let sensor_type: EsfSensorType = (s & 0x3F).into();
        Self {
            sensor_type,
            used: (s >> 6) != 0,
            ready: (s >> 7) != 0,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorType {
    None = 0,
    /// Angular acceleration in [deg/s]
    GyroZ = 5,
    /// Unitless (counter)
    FrontLeftWheelTicks = 6,
    /// Unitless (counter)
    FrontRightWheelTicks = 7,
    /// Unitless (counter)
    RearLeftWheelTicks = 8,
    /// Unitless (counter)
    RearRightWheelTicks = 9,
    /// Unitless (counter)
    SpeedTick = 10,
    /// Speed in [m/s]
    Speed = 11,
    /// Temperature Celsius [deg]
    GyroTemp = 12,
    /// Angular acceleration in [deg/s]
    GyroY = 13,
    /// Angular acceleration in [deg/s]
    GyroX = 14,
    /// Specific force in [m/s^2]
    AccX = 16,
    /// Specific force in [m/s^2]
    AccY = 17,
    /// Specific force in [m/s^2]
    AccZ = 18,
    Invalid = 19,
}

impl From<u8> for EsfSensorType {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorType::None,
            5 => EsfSensorType::GyroZ,
            6 => EsfSensorType::FrontLeftWheelTicks,
            7 => EsfSensorType::FrontRightWheelTicks,
            8 => EsfSensorType::RearLeftWheelTicks,
            9 => EsfSensorType::RearRightWheelTicks,
            10 => EsfSensorType::SpeedTick,
            11 => EsfSensorType::Speed,
            12 => EsfSensorType::GyroTemp,
            13 => EsfSensorType::GyroY,
            14 => EsfSensorType::GyroX,
            16 => EsfSensorType::AccX,
            17 => EsfSensorType::AccY,
            18 => EsfSensorType::AccZ,
            _ => EsfSensorType::Invalid,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SensorStatus2 {
    pub(crate) calibration_status: EsfSensorStatusCalibration,
    pub(crate) time_status: EsfSensorStatusTime,
}

impl From<u8> for SensorStatus2 {
    fn from(s: u8) -> Self {
        let calibration_status: EsfSensorStatusCalibration = (s & 0x03).into();
        let time_status: EsfSensorStatusTime = ((s >> 2) & 0x03).into();
        Self {
            calibration_status,
            time_status,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorStatusCalibration {
    NotCalibrated = 0,
    Calibrating = 1,
    Calibrated = 2,
    Invalid = 4,
}

impl From<u8> for EsfSensorStatusCalibration {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorStatusCalibration::NotCalibrated,
            1 => EsfSensorStatusCalibration::Calibrating,
            2 | 3 => EsfSensorStatusCalibration::Calibrated,
            _ => EsfSensorStatusCalibration::Invalid,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorStatusTime {
    NoData = 0,
    OnReceptionFirstByte = 1,
    OnEventInput = 2,
    TimeTagFromData = 3,
    Invalid = 4,
}

impl From<u8> for EsfSensorStatusTime {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorStatusTime::NoData,
            1 => EsfSensorStatusTime::OnReceptionFirstByte,
            2 => EsfSensorStatusTime::OnEventInput,
            3 => EsfSensorStatusTime::TimeTagFromData,
            _ => EsfSensorStatusTime::Invalid,
        }
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
#[derive(Debug, Default, Clone, Copy)]
pub struct EsfSensorFaults: u8 {
    const BAD_MEASUREMENT = 1;
    const BAD_TIME_TAG = 2;
    const MISSING_MEASUREMENT = 4;
    const NOISY_MEASUREMENT = 8;
}
}

impl From<u8> for EsfSensorFaults {
    fn from(s: u8) -> Self {
        Self::from_bits(s).unwrap_or(EsfSensorFaults::empty())
    }
}

#[cfg(feature = "ubx_proto23")]
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

#[cfg(feature = "ubx_proto23")]
#[ubx_packet_recv]
#[ubx(class = 0x28, id = 0x02, fixed_payload_len = 36)]
struct HnrIns {
    #[ubx(map_type = HnrInsBitFlags)]
    bitfield: u32,
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

#[cfg(feature = "ubx_proto23")]
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

    #[ubx(map_type = GpsFix)]
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

impl core::iter::Iterator for DwrdIter<'_> {
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
struct MgaGpsEph {
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
    af1: i16,
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn mon_ver_rom_interpret() {
        let payload: [u8; 160] = [
            82, 79, 77, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41,
            0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 70, 87, 86, 69, 82, 61,
            83, 80, 71, 32, 51, 46, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 82,
            79, 84, 86, 69, 82, 61, 49, 56, 46, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 71, 80, 83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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

    #[test]
    fn mon_ver_flash_m8l_interpret() {
        let payload: [u8; 250] = [
            69, 88, 84, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 100, 49, 56, 57, 102, 102,
            41, 0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 82, 79, 77, 32, 66,
            65, 83, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41, 0, 0, 0, 0, 0, 0,
            0, 0, 70, 87, 86, 69, 82, 61, 65, 68, 82, 32, 52, 46, 49, 49, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 80, 82, 79, 84, 86, 69, 82, 61, 49, 57, 46, 49, 48, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 79, 68, 61, 78, 69, 79, 45, 77, 56, 76, 45,
            48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 70, 73, 83, 61, 48, 120, 69, 70,
            52, 48, 49, 53, 32, 40, 49, 48, 48, 49, 49, 49, 41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 80,
            83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(Ok(()), <MonVerRef>::validate(&payload));
        let ver = MonVerRef(&payload);
        assert_eq!("EXT CORE 3.01 (d189ff)", ver.software_version());
        assert_eq!("00080000", ver.hardware_version());
        let mut it = ver.extension();
        assert_eq!("ROM BASE 3.01 (107888)", it.next().unwrap());
        assert_eq!("FWVER=ADR 4.11", it.next().unwrap());
        assert_eq!("PROTVER=19.10", it.next().unwrap());
        assert_eq!("MOD=NEO-M8L-0", it.next().unwrap());
        assert_eq!("FIS=0xEF4015 (100111)", it.next().unwrap());
        assert_eq!("GPS;GLO;GAL;BDS", it.next().unwrap());
        assert_eq!("SBAS;IMES;QZSS", it.next().unwrap());
        assert_eq!(None, it.next());
    }
}
