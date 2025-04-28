use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

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
