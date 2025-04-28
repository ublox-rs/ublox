#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

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
