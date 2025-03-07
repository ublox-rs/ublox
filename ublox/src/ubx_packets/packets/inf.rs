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
#[ubx(
    class = 0x4,
    id = 0x0,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfError {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x2,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfNotice {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x3,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfTest {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x1,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfWarning {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

#[ubx_packet_recv]
#[ubx(
    class = 0x4,
    id = 0x4,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct InfDebug {
    #[ubx(map_type = Option<&str>,
        may_fail,
        is_valid = inf::is_valid,
        from = inf::convert_to_str,
        get_as_ref)]
    message: [u8; 0],
}

mod inf {
    pub(crate) fn convert_to_str(bytes: &[u8]) -> Option<&str> {
        match core::str::from_utf8(bytes) {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }

    pub(crate) fn is_valid(_bytes: &[u8]) -> bool {
        // Validity is checked in convert_to_str
        true
    }
}
