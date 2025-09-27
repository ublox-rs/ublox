#![cfg(feature = "ubx_proto31")]
//! Protocol 31 specific types

use crate::ubx_packets::packetref_proto31;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packetref_proto31::PacketRef;

impl<'a> From<PacketRef<'a>> for crate::UbxPacket<'a> {
    fn from(packet: PacketRef<'a>) -> Self {
        crate::UbxPacket::Proto31(packet)
    }
}

/// Tag for protocol 31 packets
pub struct Proto31;

impl crate::UbxProtocol for Proto31 {
    type PacketRef<'a> = PacketRef<'a>;
    const MAX_PAYLOAD_LEN: u16 = packetref_proto31::MAX_PAYLOAD_LEN;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
        packetref_proto31::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto31> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
