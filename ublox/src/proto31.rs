#![cfg(feature = "ubx_proto31")]
//! Protocol 31 specific types

use crate::ubx_packets::packet_proto31;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[doc(inline)]
pub use crate::ubx_packets::packet_proto31::Packet;

impl<'a> From<Packet<'a>> for crate::UbxPacket<'a> {
    fn from(packet: Packet<'a>) -> Self {
        crate::UbxPacket::Proto31(packet)
    }
}

/// Tag for protocol 31 packets
pub struct Proto31;

impl crate::UbxProtocol for Proto31 {
    type Packet<'a> = Packet<'a>;
    const MAX_PAYLOAD_LEN: u16 = packet_proto31::MAX_PAYLOAD_LEN;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::Packet<'_>, crate::ParserError> {
        packet_proto31::match_packet(class_id, msg_id, payload)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto31> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
