#![cfg(any(
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, mon_ver, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// I/O pin status
///
/// This message contains information specific to each HW I/O pin, for example whether the pin is set as Input
/// or Output.
/// For the antenna supervisor status and other RF status information, see the `UBX-MON-RF` message.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x37, max_payload_len = 1024)]
pub struct MonHw3 {
    /// Message version (0x00 for this version)
    version: u8,
    /// The number of I/O pins included
    n_pins: u8,
    /// Flags
    #[ubx(map_type = Flags)]
    flags: u8,
    /// Zero-terminated hardware version string (same as that returned in the UBX-MON-VER message)
    #[ubx(map_type = &str, may_fail, from = mon_ver::convert_to_str_unchecked,
          is_valid = mon_ver::is_cstr_valid, get_as_ref)]
    hw_version: [u8; 10],
    /// Reserved bytes
    reserved0: [u8; 9],
    /// Pin information (repeated n_pins times)
    #[ubx(map_type = PinInfoIter, may_fail,
          from = PinInfoIter::new,
          is_valid = PinInfoIter::is_valid)]
    pins: [u8; 0],
}

/// Flags for MON-HW3
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Flags {
    /// RTC is calibrated
    pub rtc_calib: bool,
    /// Safeboot mode (0 = inactive, 1 = active)
    pub safe_boot: bool,
    /// RTC xtal has been determined to be absent
    pub xtal_absent: bool,
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self {
            rtc_calib: (value & 0x01) != 0,
            safe_boot: (value & 0x02) != 0,
            xtal_absent: (value & 0x04) != 0,
        }
    }
}

/// Pin information structure
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PinInfo {
    /// Identifier for the pin, including both external and internal pins
    pub pin_id: u16,
    /// Pin mask containing various pin configuration flags
    pub pin_mask: PinMask,
    /// Virtual pin mapping
    pub vp: u8,
    /// Reserved byte
    pub reserved1: u8,
}

/// Pin mask with bit fields
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PinMask {
    /// Pin is set to peripheral or PIO? 0=Peripheral 1=PIO
    pub periph_pio: bool,
    /// Bank the pin belongs to, where 0=A 1=B 2=C 3=D 4=E 5=F 6=G 7=H
    pub pin_bank: u8,
    /// Pin direction? 0=Input 1=Output
    pub direction: bool,
    /// Pin value? 0=Low 1=High
    pub value: bool,
    /// Used by virtual pin manager? 0=No 1=Yes
    pub vp_manager: bool,
    /// Interrupt enabled? 0=No 1=Yes
    pub pio_irq: bool,
    /// Using pull high resistor? 0=No 1=Yes
    pub pio_pull_high: bool,
    /// Using pull low resistor? 0=No 1=Yes
    pub pio_pull_low: bool,
}

impl From<u16> for PinMask {
    fn from(value: u16) -> Self {
        Self {
            periph_pio: (value & 0x0001) != 0,
            // bits 1..3 (mask 0x000E) shifted right by 1
            pin_bank: ((value & 0x000E) >> 1) as u8,
            direction: (value & 0x0010) != 0,
            value: (value & 0x0020) != 0,
            vp_manager: (value & 0x0040) != 0,
            pio_irq: (value & 0x0080) != 0,
            pio_pull_high: (value & 0x0100) != 0,
            pio_pull_low: (value & 0x0200) != 0,
        }
    }
}

/// Iterator for pin information
#[derive(Debug, Clone)]
pub struct PinInfoIter<'a> {
    data: &'a [u8],
    offset: usize,
    // keep for future use / debugging; not strictly required for iteration here
    _pins_total: usize,
}

impl<'a> PinInfoIter<'a> {
    /// Construct iterator from raw pin payload bytes (should be `nPins * 6` bytes).
    /// Note: the derive macro will call `PinInfoIter::is_valid` before using this where `may_fail` is set.
    fn new(data: &'a [u8]) -> Self {
        let total = data.len() / 6;
        Self {
            data,
            offset: 0,
            _pins_total: total,
        }
    }

    /// Validate raw repeated-group payload: must be a multiple of 6 bytes (each pin entry is 6 bytes)
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 6 == 0
    }
}

impl core::iter::Iterator for PinInfoIter<'_> {
    type Item = PinInfo;

    fn next(&mut self) -> Option<Self::Item> {
        // each entry must be exactly 6 bytes
        if self.offset + 6 <= self.data.len() {
            let b0 = self.data[self.offset];
            let b1 = self.data[self.offset + 1];
            let pin_id = u16::from_le_bytes([b0, b1]);

            let m0 = self.data[self.offset + 2];
            let m1 = self.data[self.offset + 3];
            let pin_mask_raw = u16::from_le_bytes([m0, m1]);
            let pin_mask = PinMask::from(pin_mask_raw);

            let vp = self.data[self.offset + 4];
            let reserved1 = self.data[self.offset + 5];

            self.offset += 6;

            Some(PinInfo {
                pin_id,
                pin_mask,
                vp,
                reserved1,
            })
        } else {
            None
        }
    }
}
