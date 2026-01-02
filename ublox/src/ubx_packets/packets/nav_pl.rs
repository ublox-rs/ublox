#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Protection Level Information
///
/// This message provides protection level (PL) values per protection level state
/// (e.g. position ECEF X/Y/Z) and w.r.t. the given target misleading information
/// risk (TMIR) per coordinate axis.
///
/// Target misleading information risk is expressed as X [%MI/epoch] (read: X%
/// probability of having an MI per epoch). Misleading information (MI) occurs
/// when the Protection Level value is smaller than the true position error.
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x62, fixed_payload_len = 52)]
struct NavPl {
    /// Message version (0x01 for this version)
    version: u8,

    /// Target misleading information risk (TMIR) [%MI/epoch], coefficient
    /// integer number of base 10 scientific notation.
    /// TMIR = tmirCoeff * 10^tmirExp
    tmir_coeff: u8,

    /// Target misleading information risk (TMIR) [%MI/epoch], exponent
    /// integer number of base 10 scientific notation.
    /// TMIR = tmirCoeff * 10^tmirExp
    tmir_exp: i8,

    /// Position protection level validity
    /// 0: Invalid (Protection level should not be used)
    /// 1: Protection level is valid
    #[ubx(map_type = PlPosValid)]
    pl_pos_valid: u8,

    /// Position protection level frame
    #[ubx(map_type = PlPosFrame)]
    pl_pos_frame: u8,

    /// Velocity protection level validity
    /// 0: Invalid (Protection level should not be used)
    /// 1: Protection level is valid
    #[ubx(map_type = PlVelValid)]
    pl_vel_valid: u8,

    /// Velocity protection level frame
    #[ubx(map_type = PlVelFrame)]
    pl_vel_frame: u8,

    /// Time protection level validity
    /// 0: Invalid (Protection level should not be used)
    /// 1: Protection level is valid
    #[ubx(map_type = PlTimeValid)]
    pl_time_valid: u8,

    /// Position protection level invalidity reason
    #[ubx(map_type = PlInvalidityReason)]
    pl_pos_invalidity_reason: u8,

    /// Velocity protection level invalidity reason
    #[ubx(map_type = PlInvalidityReason)]
    pl_vel_invalidity_reason: u8,

    /// Time protection level invalidity reason
    #[ubx(map_type = PlInvalidityReason)]
    pl_time_invalidity_reason: u8,

    /// Reserved
    reserved0: u8,

    /// GPS time of week (ms)
    itow: u32,

    /// First axis of position protection level value (m),
    /// given in coordinate frame of plPosFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_pos1: u32,

    /// Second axis of position protection level value (m),
    /// given in coordinate frame of plPosFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_pos2: u32,

    /// Third axis of position protection level value (m),
    /// given in coordinate frame of plPosFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_pos3: u32,

    /// First axis of velocity protection level value (m/s),
    /// given in coordinate frame of plVelFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_vel1: u32,

    /// Second axis of velocity protection level value (m/s),
    /// given in coordinate frame of plVelFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_vel2: u32,

    /// Third axis of velocity protection level value (m/s),
    /// given in coordinate frame of plVelFrame
    #[ubx(map_type = f64, scale = 0.001)]
    pl_vel3: u32,

    /// Orientation of HorizSemiMajorAxis of horizontal ellipse position
    /// protection level (clockwise degrees from true North),
    /// if plPosFrame==3; zero otherwise.
    /// Scale: 1e-2 degrees
    #[ubx(map_type = f64, scale = 0.01)]
    pl_pos_horiz_orient: u16,

    /// Orientation of HorizSemiMajorAxis of horizontal ellipse velocity
    /// protection level (clockwise degrees from true North),
    /// if plVelFrame==3; zero otherwise.
    /// Scale: 1e-2 degrees
    #[ubx(map_type = f64, scale = 0.01)]
    pl_vel_horiz_orient: u16,

    /// Time protection level value (ns), w.r.t. the given target
    /// misleading information risk (TMIR) of [tmirCoeff * 10^(tmirExp)]
    pl_time: u32,

    /// Reserved
    reserved1: [u8; 4],
}

/// Position protection level validity
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlPosValid {
    /// Invalid - Protection level should not be used
    Invalid = 0,
    /// Protection level is valid
    Valid = 1,
}

/// Velocity protection level validity
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlVelValid {
    /// Invalid - Protection level should not be used
    Invalid = 0,
    /// Protection level is valid
    Valid = 1,
}

/// Time protection level validity
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlTimeValid {
    /// Invalid - Protection level should not be used
    Invalid = 0,
    /// Protection level is valid
    Valid = 1,
}

/// Position protection level reference frame
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlPosFrame {
    /// Invalid (not possible to calculate frame conversion)
    Invalid = 0,
    /// North-East-Down
    Ned = 1,
    /// Longitudinal-Lateral-Vertical
    LongLatVert = 2,
    /// HorizSemiMajorAxis-HorizSemiMinorAxis-Vertical
    HorizSemiMajorMinorVert = 3,
}

/// Velocity protection level reference frame
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlVelFrame {
    /// Invalid (not possible to calculate frame conversion)
    Invalid = 0,
    /// North-East-Down
    Ned = 1,
    /// Longitudinal-Lateral-Vertical
    LongLatVert = 2,
    /// HorizSemiMajorAxis-HorizSemiMinorAxis-Vertical
    HorizSemiMajorMinorVert = 3,
}

/// Protection level invalidity reason
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlInvalidityReason {
    /// Not available
    NotAvailable = 0,
    /// Solution not trustworthy (values 1-29 map to this)
    SolutionNotTrustworthy = 1,
    /// PL not verified for this receiver configuration (values 30-100)
    NotVerifiedForConfig = 30,
}
