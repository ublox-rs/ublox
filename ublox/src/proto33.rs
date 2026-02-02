#![cfg(feature = "ubx_proto33")]
//! Protocol 33 specific types

use crate::ubx_packets::packet_proto33;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packet_proto33::Packet;

impl<'a> From<Packet<'a>> for crate::UbxPacket<'a> {
    fn from(packet: Packet<'a>) -> Self {
        crate::UbxPacket::Proto33(packet)
    }
}

/// Tag for protocol 33 packets
pub struct Proto33;

impl crate::UbxProtocol for Proto33 {
    type Packet<'a> = Packet<'a>;
    const MAX_PAYLOAD_LEN: u16 = packet_proto33::MAX_PAYLOAD_LEN;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::Packet<'_>, crate::ParserError> {
        packet_proto33::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto33> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
