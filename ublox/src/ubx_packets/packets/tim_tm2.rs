#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Time mark data
#[ubx_packet_recv]
#[ubx(class = 0x0d, id = 0x03, fixed_payload_len = 28)]
struct TimTm2 {
    /// Channel (e.g. EXTINT) upon which the pulse was measured
    ch: u8,
    /// Flags
    #[ubx(map_type = TimTm2Flags, from = TimTm2Flags)]
    flags: u8,
    /// Rising edge counter
    count: u16,
    /// Week number of last rising edge
    wn_r: u16,
    /// Week number of last falling edge
    wn_f: u16,
    /// Tow of rising edge
    tow_ms_r: u32,
    /// Millisecond fraction of tow of rising edge in nanoseconds
    tow_sub_ms_r: u32,
    /// Tow of falling edge
    tow_ms_f: u32,
    /// Millisecond fraction of tow of falling edge in nanoseconds
    tow_sub_ms_f: u32,
    /// Accuracy estimate
    acc_est: u32,
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
            _ => unreachable!("TimeBase value not supported by protocol specification"),
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
