#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

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
