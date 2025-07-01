use bitflags::bitflags;

#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

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

#[derive(Debug, Clone)]
pub struct RxmRawxInfoIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> RxmRawxInfoIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(data.chunks_exact(32))
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % 32 == 0
    }
}

impl<'a> core::iter::Iterator for RxmRawxInfoIter<'a> {
    type Item = RxmRawxInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(RxmRawxInfoRef)
    }
}

/// This packet is not actually received as such, it is a block of the `RxmRawx` message
/// The `ubx_packet_recv` macro is used here as a shortcut to generate the needed code required for the repeated block.
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
