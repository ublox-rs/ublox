use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

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
