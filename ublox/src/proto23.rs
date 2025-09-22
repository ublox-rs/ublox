#![cfg(feature = "ubx_proto23")]
//! Protocol 23 specific types

use crate::ubx_packets::packetref_proto23;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packetref_proto23::PacketRef;

impl<'a> From<PacketRef<'a>> for crate::UbxPacket<'a> {
    fn from(packet: PacketRef<'a>) -> Self {
        crate::UbxPacket::Proto23(packet)
    }
}

/// Tag for protocol 23 packets
pub struct Proto23;

impl crate::UbxProtocol for Proto23 {
    type PacketRef<'a> = PacketRef<'a>;

    const MAX_PAYLOAD_LEN: usize = packetref_proto23::MAX_PAYLOAD_LEN as usize;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
        packetref_proto23::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto23> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
