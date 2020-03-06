//! # ublox
//!
//! `ublox` is a library to talk to u-blox GPS devices using the UBX protocol.
//! At time of writing this library is developed for a device which behaves like
//! a NEO-6M device.

pub use crate::segmenter::Segmenter;
#[cfg(feature = "serial")]
pub use crate::serialport::{Device, ResetType};
pub use crate::ubx_packets::*;

mod error;
mod segmenter;
#[cfg(feature = "serial")]
mod serialport;
mod ubx_packets;
