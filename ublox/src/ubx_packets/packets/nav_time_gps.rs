use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

///  GPS time solution
#[ubx_packet_recv]
#[ubx(class = 1, id = 0x20, fixed_payload_len = 16)]
struct NavTimeGps {
    /// GPS time of week of the navigation epoch (ms).
    itow: u32,

    /// Fractional part of iTOW (range: +/- 500000) (ns).
    ftow: i32,

    /// GPS week number of the navigation epoch.
    week: i16,

    /// GPS leap seconds (GPS-UTC) (s).
    leap_s: i8,

    /// Validity Flags.
    #[ubx(map_type = NavTimeGpsFlags)]
    valid: u8,

    /// Time Accuracy Estimate (ns).
    t_acc: u32,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Validity flags of `NavTimeGps`
    #[derive(Default, Debug)]
    pub struct NavTimeGpsFlags: u8 {
        ///  Valid GPS time of week (itow + ftow).
        const VALID_TOW = 1;
        /// Valid GPS week number.
        const VALID_WKN = 2;
        /// Valid GPS leap seconds.
        const VALID_LEAP_S = 4;
    }
}
