#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Hardware status
///
/// Deprecated after protocol version 23, use `UBX-MON-HW3` and `UBX-MON-RF` instead.
///
/// Status of different aspect of the hardware, such as antenna, PIO/peripheral pins, noise level, automatic gain control (AGC)
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x09, fixed_payload_len = 60)]
struct MonHw {
    /// Mask of pins set as peripheral/PIO
    pin_sel: u32,
    /// Mask of pins set as bank A/B
    pin_bank: u32,
    /// Mask of pins set as input/output
    pin_dir: u32,
    /// Mask of pins value low/high
    pin_val: u32,
    /// Noise level as measured by the GPS core
    noise_per_ms: u16,
    /// AGC monitor (counts SIGHI xor SIGLO, range 0 to 8191)
    agc_cnt: u16,
    /// Status of the antenna supervisor state machine (0=INIT, 1=DONTKNOW, 2=OK, 3=SHORT, 4=OPEN)
    #[ubx(map_type = AntennaStatus)]
    a_status: u8,
    /// Current power status of antenna (0=OFF, 1=ON, 2=DONTKNOW)
    #[ubx(map_type = AntennaPower)]
    a_power: u8,
    /// Hardware status flags
    #[ubx(map_type = HardwareFlags)]
    flags: u8,
    reserved1: u8,
    /// Mask of pins that are used by the virtual pin manager
    used_mask: u32,
    /// Array of pin mappings for each of the 17 physical pins
    vp: [u8; 17],
    /// CW jamming indicator, scaled (0 = no CW jamming, 255 = strong CW jamming)
    jam_ind: u8,
    reserved2: [u8; 2],
    /// Mask of pins value using the PIO Irq
    pin_irq: u32,
    /// Mask of pins value using the PIO pull high resistor
    pull_h: u32,
    /// Mask of pins value using the PIO pull low resistor
    pull_l: u32,
}

/// Status of the antenna supervisor state machine (0=INIT, 1=DONTKNOW, 2=OK, 3=SHORT, 4=OPEN)
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaStatus {
    Init = 0,
    DontKnow = 1,
    Ok = 2,
    Short = 3,
    Open = 4,
}

/// Current power status of antenna (0=OFF, 1=ON, 2=DONTKNOW)
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaPower {
    Off = 0,
    On = 1,
    DontKnow = 2,
}

/// Hardware status flags
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HardwareFlags {
    /// RTC is calibrated
    RtcCalib = 0b0000_0001,
    /// Safeboot mode active
    SafeBoot = 0b0000_0010,
    /// RTC xtal has been determined to be absent
    XtalAbsent = 0b0001_0000,
}

impl HardwareFlags {
    /// Check if RTC is calibrated
    pub fn rtc_calib(self) -> bool {
        (self as u8 & Self::RtcCalib as u8) != 0
    }

    /// Check if safeboot mode is active
    pub fn safe_boot(self) -> bool {
        (self as u8 & Self::SafeBoot as u8) != 0
    }

    /// Check if RTC xtal is absent (not supported in protocol versions less than 18)
    pub fn xtal_absent(self) -> bool {
        (self as u8 & Self::XtalAbsent as u8) != 0
    }

    /// Get jamming/interference monitor status from bits 3..2
    pub fn jamming_state(self) -> JammingState {
        match (self as u8 >> 2) & 0b11 {
            0 => JammingState::Unknown,
            1 => JammingState::Ok,
            2 => JammingState::Warning,
            3 => JammingState::Critical,
            _ => unreachable!(),
        }
    }

    /// Get raw flags value
    pub fn raw(self) -> u8 {
        self as u8
    }
}

/// Jamming/interference monitor status (bits 3..2)
///
/// This flag is deprecated in protocol versions that support UBX-SEC-SIG (version 0x02)
/// and always reported as 0; instead jammingState in UBX-SEC-SIG should be monitored.
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JammingState {
    /// Unknown, or feature disabled, or flag unavailable
    Unknown = 0,
    /// OK - no significant jamming
    Ok = 1,
    /// Warning - interference visible but fix OK
    Warning = 2,
    /// Critical - interference visible and no fix
    Critical = 3,
}
