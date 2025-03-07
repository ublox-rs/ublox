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

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Selected / available Constellation Mask
    #[derive(Default, Debug)]
    pub struct MonGnssConstellMask: u8 {
        /// GPS constellation
        const GPS = 0x01;
        /// GLO constellation
        const GLO = 0x02;
        /// BDC constellation
        const BDC = 0x04;
        /// GAL constellation
        const GAL = 0x08;
    }
}

/// GNSS status monitoring,
/// gives currently selected constellations
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x28, fixed_payload_len = 8)]
pub struct MonGnss {
    /// Message version: 0x00
    pub version: u8,
    /// Supported major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    pub supported: u8,
    /// Default major GNSS constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    pub default: u8,
    /// Currently enabled major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    pub enabled: u8,
    /// Maximum number of concurent Major GNSS
    /// that can be supported by this receiver
    pub simultaneous: u8,
    pub reserved1: [u8; 3],
}

/// Receiver/Software Version
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x04, max_payload_len = 1240)]
pub struct MonVer {
    #[ubx(map_type = &str, may_fail, from = mon_ver::convert_to_str_unchecked,
          is_valid = mon_ver::is_cstr_valid, get_as_ref)]
    pub software_version: [u8; 30],
    #[ubx(map_type = &str, may_fail, from = mon_ver::convert_to_str_unchecked,
          is_valid = mon_ver::is_cstr_valid, get_as_ref)]
    pub hardware_version: [u8; 10],

    /// Extended software information strings
    #[ubx(map_type = MonVerExtensionIter, may_fail,
          from = MonVerExtensionIter::new,
          is_valid = MonVerExtensionIter::is_valid)]
    pub extension: [u8; 0],
}

mod mon_ver {
    pub(crate) fn convert_to_str_unchecked(bytes: &[u8]) -> &str {
        let null_pos = bytes
            .iter()
            .position(|x| *x == 0)
            .expect("is_cstr_valid bug?");
        core::str::from_utf8(&bytes[0..null_pos])
            .expect("is_cstr_valid should have prevented this code from running")
    }

    pub(crate) fn is_cstr_valid(bytes: &[u8]) -> bool {
        let null_pos = match bytes.iter().position(|x| *x == 0) {
            Some(pos) => pos,
            None => {
                return false;
            },
        };
        core::str::from_utf8(&bytes[0..null_pos]).is_ok()
    }
}

/// Hardware status
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x09, fixed_payload_len = 60)]
pub struct MonHw {
    pub pin_sel: u32,
    pub pin_bank: u32,
    pub pin_dir: u32,
    pub pin_val: u32,
    pub noise_per_ms: u16,
    pub agc_cnt: u16,
    #[ubx(map_type = AntennaStatus)]
    pub a_status: u8,
    #[ubx(map_type = AntennaPower)]
    pub a_power: u8,
    pub flags: u8,
    pub reserved1: u8,
    pub used_mask: u32,
    pub vp: [u8; 17],
    pub jam_ind: u8,
    pub reserved2: [u8; 2],
    pub pin_irq: u32,
    pub pull_h: u32,
    pub pull_l: u32,
}

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

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AntennaPower {
    Off = 0,
    On = 1,
    DontKnow = 2,
}

#[derive(Debug, Clone)]
pub struct MonVerExtensionIter<'a> {
    data: &'a [u8],
    offset: usize,
}

use mon_ver::is_cstr_valid;

impl<'a> MonVerExtensionIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 30 == 0 && payload.chunks(30).all(is_cstr_valid)
    }
}

impl<'a> core::iter::Iterator for MonVerExtensionIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 30];
            self.offset += 30;
            Some(mon_ver::convert_to_str_unchecked(data))
        } else {
            None
        }
    }
}
