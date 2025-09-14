#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;
#[cfg(feature = "serde")]
extern crate serde;

pub use crate::{
    error::{DateTimeError, MemWriterError, ParserError},
    parser::{
        AnyPacketRef, FixedLinearBuffer, Parser, RtcmPacketRef, UbxParserIter, UnderlyingBuffer,
    },
    ubx_packets::*,
};

#[derive(Debug)]
pub enum UbxPacket<'a> {
    #[cfg(feature = "ubx_proto14")]
    Proto17(proto17::PacketRef<'a>),
    #[cfg(feature = "ubx_proto23")]
    Proto23(proto23::PacketRef<'a>),
    #[cfg(feature = "ubx_proto27")]
    Proto27(proto27::PacketRef<'a>),
    #[cfg(feature = "ubx_proto31")]
    Proto31(proto31::PacketRef<'a>),
}

#[cfg(feature = "ubx_proto14")]
impl<'a> From<proto17::PacketRef<'a>> for UbxPacket<'a> {
    fn from(packet: proto17::PacketRef<'a>) -> Self {
        UbxPacket::Proto17(packet)
    }
}
#[cfg(feature = "ubx_proto23")]
impl<'a> From<proto23::PacketRef<'a>> for UbxPacket<'a> {
    fn from(packet: proto23::PacketRef<'a>) -> Self {
        UbxPacket::Proto23(packet)
    }
}
#[cfg(feature = "ubx_proto27")]
impl<'a> From<proto27::PacketRef<'a>> for UbxPacket<'a> {
    fn from(packet: proto27::PacketRef<'a>) -> Self {
        UbxPacket::Proto27(packet)
    }
}
#[cfg(feature = "ubx_proto31")]
impl<'a> From<proto31::PacketRef<'a>> for UbxPacket<'a> {
    fn from(packet: proto31::PacketRef<'a>) -> Self {
        UbxPacket::Proto31(packet)
    }
}

#[cfg(all(feature = "ubx_proto14", any(feature = "std", feature = "alloc")))]
pub use parser::proto14;
#[cfg(feature = "ubx_proto14")]
pub use parser::proto14_with_buffer;
#[cfg(all(feature = "ubx_proto23", any(feature = "std", feature = "alloc")))]
pub use parser::proto23;
#[cfg(feature = "ubx_proto23")]
pub use parser::proto23_with_buffer;
#[cfg(all(feature = "ubx_proto27", any(feature = "std", feature = "alloc")))]
pub use parser::proto27;
#[cfg(feature = "ubx_proto27")]
pub use parser::proto27_with_buffer;
#[cfg(all(feature = "ubx_proto31", any(feature = "std", feature = "alloc")))]
pub use parser::proto31;
#[cfg(feature = "ubx_proto31")]
pub use parser::proto31_with_buffer;

use core::marker::Sized;

/// Trait for parsing UBX protocol version.
pub trait UbxProtocol: Send + Sized {
    /// The protocol-specific PacketRef type. The `'a` lifetime is tied to the input buffer.
    type PacketRef<'a>: Into<UbxPacket<'a>>;

    /// The maximum payload length supported by this protocol version.
    const MAX_PAYLOAD_LEN: usize;

    /// Matches a Class ID, Message ID, and payload to a specific packet type.
    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, ParserError>;
}

#[cfg(feature = "ubx_proto14")]
pub mod proto17 {
    pub use crate::parser::Proto17;
    pub use crate::ubx_packets::packetref_proto17::PacketRef;
}
#[cfg(feature = "ubx_proto23")]
pub mod proto23 {
    pub use crate::parser::Proto23;
    pub use crate::ubx_packets::packetref_proto23::PacketRef;
}
#[cfg(feature = "ubx_proto27")]
pub mod proto27 {
    pub use crate::parser::Proto27;
    pub use crate::ubx_packets::packetref_proto27::PacketRef;
}
#[cfg(feature = "ubx_proto31")]
pub mod proto31 {
    pub use crate::parser::Proto31;
    pub use crate::ubx_packets::packetref_proto31::PacketRef;
}

mod error;
mod parser;
mod ubx_packets;
