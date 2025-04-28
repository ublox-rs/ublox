use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta, UtcStandardIdentifier};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

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
    #[ubx(map_type = UtcStandardIdentifier, may_fail)]
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
        /// External oscillator is within tolerance limit (Ubx-CfgSmgr)
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
