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
    //! Protocol 17 specific types

    use crate::ubx_packets::packetref_proto17::{match_packet, MAX_PAYLOAD_LEN};
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
        const MAX_PAYLOAD_LEN: usize = MAX_PAYLOAD_LEN as usize;

        fn match_packet(
            class_id: u8,
            msg_id: u8,
            payload: &[u8],
        ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
            match_packet(class_id, msg_id, payload)
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl core::default::Default for crate::Parser<Vec<u8>, Proto17> {
        fn default() -> Self {
            Self::new(Vec::new())
        }
    }
}

#[cfg(feature = "ubx_proto23")]
pub mod proto23 {
    //! Protocol 23 specific types

    use crate::ubx_packets::packetref_proto23::{match_packet, MAX_PAYLOAD_LEN};
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

        const MAX_PAYLOAD_LEN: usize = MAX_PAYLOAD_LEN as usize;

        fn match_packet(
            class_id: u8,
            msg_id: u8,
            payload: &[u8],
        ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
            match_packet(class_id, msg_id, payload)
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl core::default::Default for crate::Parser<Vec<u8>, Proto23> {
        fn default() -> Self {
            Self::new(Vec::new())
        }
    }
}

#[cfg(feature = "ubx_proto27")]
pub mod proto27 {
    //! Protocol 27 specific types

    use crate::ubx_packets::packetref_proto27::{match_packet, MAX_PAYLOAD_LEN};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[doc(inline)]
    pub use crate::ubx_packets::packetref_proto27::PacketRef;

    impl<'a> From<PacketRef<'a>> for crate::UbxPacket<'a> {
        fn from(packet: PacketRef<'a>) -> Self {
            crate::UbxPacket::Proto27(packet)
        }
    }

    /// Tag for protocol 27 packets
    pub struct Proto27;

    impl crate::UbxProtocol for Proto27 {
        type PacketRef<'a> = PacketRef<'a>;
        const MAX_PAYLOAD_LEN: usize = MAX_PAYLOAD_LEN as usize;

        fn match_packet(
            class_id: u8,
            msg_id: u8,
            payload: &[u8],
        ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
            match_packet(class_id, msg_id, payload)
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl core::default::Default for crate::Parser<Vec<u8>, Proto27> {
        fn default() -> Self {
            Self::new(Vec::new())
        }
    }
}

#[cfg(feature = "ubx_proto31")]
pub mod proto31 {
    //! Protocol 31 specific types

    use crate::ubx_packets::packetref_proto31::{match_packet, MAX_PAYLOAD_LEN};
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
        const MAX_PAYLOAD_LEN: usize = MAX_PAYLOAD_LEN as usize;

        fn match_packet(
            class_id: u8,
            msg_id: u8,
            payload: &[u8],
        ) -> Result<Self::PacketRef<'_>, crate::ParserError> {
            match_packet(class_id, msg_id, payload)
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl core::default::Default for crate::Parser<Vec<u8>, Proto31> {
        fn default() -> Self {
            Self::new(Vec::new())
        }
    }
}

mod error;
mod parser;
mod ubx_packets;
