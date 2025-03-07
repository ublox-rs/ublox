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
