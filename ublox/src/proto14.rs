#![cfg(feature = "ubx_proto14")]
//! Protocol 14 specific types

use crate::ubx_packets::packet_proto14;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packet_proto14::Packet;

impl<'a> From<Packet<'a>> for crate::UbxPacket<'a> {
    fn from(packet: Packet<'a>) -> Self {
        crate::UbxPacket::Proto14(packet)
    }
}

/// Tag for protocol 14 packets
pub struct Proto14;

impl crate::UbxProtocol for Proto14 {
    type Packet<'a> = Packet<'a>;
    const MAX_PAYLOAD_LEN: u16 = packet_proto14::MAX_PAYLOAD_LEN;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::Packet<'_>, crate::ParserError> {
        packet_proto14::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto14> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
