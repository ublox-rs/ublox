use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitflags::bitflags;

use super::{FixStatusInfo, GpsFix, SerializeUbxPacketFields};

use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ubx_checksum, MemWriter, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1,
    SYNC_CHAR_2,
};

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x32, fixed_payload_len = 8)]
pub struct RxmRtcm {
    pub version: u8,
    pub flags: u8,
    pub sub_type: u16,
    pub ref_station: u16,
    pub msg_type: u16,
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

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x15, max_payload_len = 8176)] // 16 + 255 * 32
pub struct RxmRawx {
    /// Measurement time of week in receiver local time approximately aligned to the GPS time system.
    pub rcv_tow: f64,
    /// GPS week number in receiver local time.
    pub week: u16,
    /// GPS leap seconds (GPS-UTC)
    pub leap_s: i8,
    /// Number of measurements to follow
    pub num_meas: u8,
    /// Receiver tracking status bitfield
    #[ubx(map_type = RecStatFlags)]
    pub rec_stat: u8,
    /// Message version
    pub version: u8,
    pub reserved1: [u8; 2],
    /// Extended software information strings
    #[ubx(
        map_type = RxmRawxInfoIter,
        from = RxmRawxInfoIter::new,
        may_fail,
        is_valid = RxmRawxInfoIter::is_valid,
    )]
    pub measurements: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct DwrdIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> DwrdIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        DwrdIter(bytes.chunks_exact(4))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 4 == 0
    }
}

impl<'a> core::iter::Iterator for DwrdIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x13, max_payload_len = 72)]
pub struct RxmSfrbx {
    pub gnss_id: u8,
    pub sv_id: u8,
    pub reserved1: u8,
    pub freq_id: u8,
    pub num_words: u8,
    pub reserved2: u8,
    pub version: u8,
    pub reserved3: u8,
    #[ubx(
        map_type = DwrdIter,
        from = DwrdIter::new,
        is_valid = DwrdIter::is_valid,
        may_fail,
    )]
    pub dwrd: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct RxmRawxInfoIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> RxmRawxInfoIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(data.chunks_exact(32))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 32 == 0
    }
}

impl<'a> core::iter::Iterator for RxmRawxInfoIter<'a> {
    type Item = RxmRawxInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(RxmRawxInfoRef)
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x15, fixed_payload_len = 32)]
#[derive(Debug)]
pub struct RxmRawxInfo {
    pub pr_mes: f64,
    pub cp_mes: f64,
    pub do_mes: f32,
    pub gnss_id: u8,
    pub sv_id: u8,
    pub reserved2: u8,
    pub freq_id: u8,
    pub lock_time: u16,
    pub cno: u8,
    #[ubx(map_type = StdevFlags)]
    pub pr_stdev: u8,
    #[ubx(map_type = StdevFlags)]
    pub cp_stdev: u8,
    #[ubx(map_type = StdevFlags)]
    pub do_stdev: u8,
    #[ubx(map_type = TrkStatFlags)]
    pub trk_stat: u8,
    pub reserved3: u8,
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
