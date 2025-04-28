#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

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
