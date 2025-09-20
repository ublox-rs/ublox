#![cfg(feature = "ubx_proto14")]
//! Protocol 17 specific types

use crate::ubx_packets::packetref_proto17;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packetref_proto17::PacketRef;

impl<'a> From<PacketRef<'a>> for crate::UbxPacket<'a> {
    fn from(packet: PacketRef<'a>) -> Self {
        crate::UbxPacket::Proto17(packet)
    }
}

/// Tag for protocol 17 packets
pub struct Proto17;

impl crate::UbxProtocol for Proto17 {
    type PacketRef<'a> = PacketRef<'a>;
    const MAX_PAYLOAD_LEN: usize = packetref_proto17::MAX_PAYLOAD_LEN as usize;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
        packetref_proto17::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto17> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
