#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::cfg_val::CfgVal;

use bitflags::bitflags;

use super::SerializeUbxPacketFields;

use ublox_derive::{ubx_extend, ubx_packet_send, ubx_packet_recv, ubx_extend_bitflags, ubx_packet_recv_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ScaleBack,
    ubx_checksum, MemWriter, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1,
    SYNC_CHAR_2,
    nav::NavBbrMask,
};

/// Time mode survey-in status
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x04, fixed_payload_len = 28)]
pub struct TimSvin {
    /// Passed survey-in minimum duration
    /// Units: s
    pub dur: u32,
    /// Current survey-in mean position ECEF X coordinate
    pub mean_x: i32,
    /// Current survey-in mean position ECEF Y coordinate
    pub mean_y: i32,
    /// Current survey-in mean position ECEF Z coordinate
    pub mean_z: i32,
    /// Current survey-in mean position 3D variance
    pub mean_v: i32,
    /// Number of position observations used during survey-in
    pub obs: u32,
    /// Survey-in position validity flag, 1 = valid, otherwise 0
    pub valid: u8,
    /// Survey-in in progress flag, 1 = in-progress, otherwise 0
    pub active: u8,
    pub reserved: [u8; 2],
}

/// Time pulse time data
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x01, fixed_payload_len = 16)]
pub struct TimTp {
    /// Time pulse time of week according to time base
    pub tow_ms: u32,
    /// Submillisecond part of towMS (scaling: 2^(-32))
    pub tow_sub_ms: u32,
    /// Quantization error of time pulse
    pub q_err: i32,
    /// Time pulse week number according to time base
    pub week: u16,
    /// Flags
    #[ubx(map_type = TimTpFlags, from = TimTpFlags)]
    pub flags: u8,
    /// Time reference information
    #[ubx(map_type = TimTpRefInfo, from = TimTpRefInfo)]
    pub ref_info: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTpFlags(u8);

impl TimTpFlags {
    /// Time base
    pub fn time_base(&self) -> TimTpTimeBase {
        if self.0 & 0b1 == 0 {
            TimTpTimeBase::Gnss
        } else {
            TimTpTimeBase::Utc
        }
    }

    /// UTC availability
    pub fn utc_available(&self) -> bool {
        self.0 & 0b10 != 0
    }

    /// (T)RAIM state
    ///
    /// Returns `None` if unavailale.
    pub fn raim_active(&self) -> Option<bool> {
        match (self.0 >> 2) & 0b11 {
            // Inactive.
            0b01 => Some(false),
            // Active.
            0b10 => Some(true),
            // Unavailable.
            _ => None,
        }
    }

    /// Quantization error validity
    pub fn q_err_valid(&self) -> bool {
        self.0 & 0b10000 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimTpTimeBase {
    Gnss,
    Utc,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTpRefInfo(u8);

impl TimTpRefInfo {
    /// GNSS reference information. Only valid if time base is GNSS.
    pub fn time_ref_gnss(&self) -> Option<TimTpRefInfoTimeRefGnss> {
        Some(match self.0 & 0b1111 {
            0 => TimTpRefInfoTimeRefGnss::Gps,
            1 => TimTpRefInfoTimeRefGnss::Glo,
            2 => TimTpRefInfoTimeRefGnss::Bds,
            3 => TimTpRefInfoTimeRefGnss::Gal,
            4 => TimTpRefInfoTimeRefGnss::NavIc,
            _ => return None,
        })
    }

    /// UTC standard identifier. Only valid if time base is UTC.
    pub fn utc_standard(&self) -> Option<TimTpRefInfoUtcStandard> {
        Some(match self.0 >> 4 {
            1 => TimTpRefInfoUtcStandard::Crl,
            2 => TimTpRefInfoUtcStandard::Nist,
            3 => TimTpRefInfoUtcStandard::Usno,
            4 => TimTpRefInfoUtcStandard::Bipm,
            5 => TimTpRefInfoUtcStandard::Eu,
            6 => TimTpRefInfoUtcStandard::Su,
            7 => TimTpRefInfoUtcStandard::Ntsc,
            8 => TimTpRefInfoUtcStandard::Npli,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimTpRefInfoTimeRefGnss {
    Gps,
    Glo,
    Bds,
    Gal,
    NavIc,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimTpRefInfoUtcStandard {
    Crl,
    Nist,
    Usno,
    Bipm,
    Eu,
    Su,
    Ntsc,
    Npli,
}

/// Time mark data
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x03, fixed_payload_len = 28)]
pub struct TimTm2 {
    /// Channel (i.e. EXTINT) upon which the pulse was measured
    pub ch: u8,
    /// Flags
    #[ubx(map_type = TimTm2Flags, from = TimTm2Flags)]
    pub flags: u8,
    /// Rising edge counter
    pub count: u16,
    /// Week number of last rising edge
    pub wn_r: u16,
    /// Week number of last falling edge
    pub wn_f: u16,
    /// Tow of rising edge
    pub tow_ms_r: u32,
    /// Millisecond fraction of tow of rising edge in nanoseconds
    pub tow_sub_ms_r: u32,
    /// Tow of falling edge
    pub tow_ms_f: u32,
    /// Millisecond fraction of tow of falling edge in nanoseconds
    pub tow_sub_ms_f: u32,
    /// Accuracy estimate
    pub acc_est: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TimTm2Flags(u8);

impl TimTm2Flags {
    pub fn mode(&self) -> TimTm2Mode {
        if self.0 & 0b1 == 0 {
            TimTm2Mode::Single
        } else {
            TimTm2Mode::Running
        }
    }

    pub fn run(&self) -> TimTm2Run {
        if self.0 & 0b10 == 0 {
            TimTm2Run::Armed
        } else {
            TimTm2Run::Stopped
        }
    }

    pub fn new_falling_edge(&self) -> bool {
        self.0 & 0b100 != 0
    }

    pub fn new_rising_edge(&self) -> bool {
        self.0 & 0b10000000 != 0
    }

    pub fn time_base(&self) -> TimTm2TimeBase {
        match self.0 & 0b11000 {
            0 => TimTm2TimeBase::Receiver,
            1 => TimTm2TimeBase::Gnss,
            2 => TimTm2TimeBase::Utc,
            _ => unreachable!(),
        }
    }

    /// UTC availability
    pub fn utc_available(&self) -> bool {
        self.0 & 0b100000 != 0
    }

    pub fn time_valid(&self) -> bool {
        self.0 & 0b1000000 != 0
    }
}

pub enum TimTm2Mode {
    Single,
    Running,
}

pub enum TimTm2Run {
    Armed,
    Stopped,
}

pub enum TimTm2TimeBase {
    Receiver,
    Gnss,
    Utc,
}
