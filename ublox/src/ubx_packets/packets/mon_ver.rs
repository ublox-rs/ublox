#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Receiver/Software Version
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x04, max_payload_len = 1240)]
pub struct MonVer {
    #[ubx(map_type = &str, may_fail, from = convert_to_str_unchecked,
          is_valid = is_cstr_valid, get_as_ref)]
    software_version: [u8; 30],
    #[ubx(map_type = &str, may_fail, from = convert_to_str_unchecked,
          is_valid = is_cstr_valid, get_as_ref)]
    hardware_version: [u8; 10],

    /// Extended software information strings
    #[ubx(map_type = MonVerExtensionIter, may_fail,
          from = MonVerExtensionIter::new,
          is_valid = MonVerExtensionIter::is_valid)]
    extension: [u8; 0],
}

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

#[derive(Debug, Clone)]
pub struct MonVerExtensionIter<'a> {
    data: &'a [u8],
    offset: usize,
}

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
            Some(convert_to_str_unchecked(data))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn mon_ver_rom_interpret() {
        let payload: [u8; 160] = [
            82, 79, 77, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41,
            0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 70, 87, 86, 69, 82, 61,
            83, 80, 71, 32, 51, 46, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 82,
            79, 84, 86, 69, 82, 61, 49, 56, 46, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 71, 80, 83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(Ok(()), <MonVerRef>::validate(&payload));
        let ver = MonVerRef(&payload);
        assert_eq!("ROM CORE 3.01 (107888)", ver.software_version());
        assert_eq!("00080000", ver.hardware_version());
        let mut it = ver.extension();
        assert_eq!("FWVER=SPG 3.01", it.next().unwrap());
        assert_eq!("PROTVER=18.00", it.next().unwrap());
        assert_eq!("GPS;GLO;GAL;BDS", it.next().unwrap());
        assert_eq!("SBAS;IMES;QZSS", it.next().unwrap());
        assert_eq!(None, it.next());
    }

    #[test]
    fn mon_ver_flash_m8l_interpret() {
        let payload: [u8; 250] = [
            69, 88, 84, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 100, 49, 56, 57, 102, 102,
            41, 0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 82, 79, 77, 32, 66,
            65, 83, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41, 0, 0, 0, 0, 0, 0,
            0, 0, 70, 87, 86, 69, 82, 61, 65, 68, 82, 32, 52, 46, 49, 49, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 80, 82, 79, 84, 86, 69, 82, 61, 49, 57, 46, 49, 48, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 79, 68, 61, 78, 69, 79, 45, 77, 56, 76, 45,
            48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 70, 73, 83, 61, 48, 120, 69, 70,
            52, 48, 49, 53, 32, 40, 49, 48, 48, 49, 49, 49, 41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 80,
            83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(Ok(()), <MonVerRef>::validate(&payload));
        let ver = MonVerRef(&payload);
        assert_eq!("EXT CORE 3.01 (d189ff)", ver.software_version());
        assert_eq!("00080000", ver.hardware_version());
        let mut it = ver.extension();
        assert_eq!("ROM BASE 3.01 (107888)", it.next().unwrap());
        assert_eq!("FWVER=ADR 4.11", it.next().unwrap());
        assert_eq!("PROTVER=19.10", it.next().unwrap());
        assert_eq!("MOD=NEO-M8L-0", it.next().unwrap());
        assert_eq!("FIS=0xEF4015 (100111)", it.next().unwrap());
        assert_eq!("GPS;GLO;GAL;BDS", it.next().unwrap());
        assert_eq!("SBAS;IMES;QZSS", it.next().unwrap());
        assert_eq!(None, it.next());
    }
}
