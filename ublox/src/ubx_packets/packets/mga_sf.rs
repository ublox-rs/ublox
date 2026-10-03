#![cfg(feature = "ubx_proto33")]
#[allow(unused_imports, reason = "It is only unused in some feature sets")]
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};
use crate::{
    ubx_checksum, MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta,
};
use crate::error::ParserError;
use ublox_derive::ubx_packet_recv_send;

#[ubx_packet_recv_send]
#[ubx(
    class = 0x13,
    id = 0x10,
    fixed_payload_len = 464,
)]
struct MgaSfIni2 {
    msg_type: u8,
    version: u8,
    reserved0: [u8; 462],
}

impl MgaSfIni2Builder {
    /// Initializes the builder with the correct type and version
    pub fn new() -> Self {
        Self {
            msg_type: 0x10,  // UBX_MGA_SF_INI2_TYPE
            version: 0x00,   // UBX_MGA_SF_INI2_VERSION
            reserved0: [0; 462],
        }
    }

    /// Populate the first data block
    pub fn set_data(&mut self, calib_data: &alloc::vec::Vec<u8>) {
        self.reserved0[..calib_data.len()].copy_from_slice(calib_data);
    }
}
