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

mod error;
mod parser;
mod ubx_packets;

pub mod proto17;
pub mod proto23;
pub mod proto27;
pub mod proto31;

/// Encapsulates all the UBX packets of each protocol.
///
/// This enum provides a unified interface for handling UBX packets across different protocol versions.
/// Each variant corresponds to a specific UBX protocol version and contains the appropriate packet
/// reference for that protocol.
///
/// # Protocol Versions
///
/// The available variants depend on which feature flags are enabled:
/// - `ubx_proto14`: Enables Protocol 17 support
/// - `ubx_proto23`: Enables Protocol 23 support
/// - `ubx_proto27`: Enables Protocol 27 support
/// - `ubx_proto31`: Enables Protocol 31 support
///
/// # Note
///
/// Most users will only need one protocol, so with only one protocol feature enabled
/// the enum will contain a single variant which is the UBX packet variant of the selected protocol.
///
/// # Example
///
/// ```rust,ignore
/// # use ublox::UbxPacket;
/// match packet {
///     #[cfg(feature = "ubx_proto14")]
///     UbxPacket::Proto17(p) => { /* handle proto17 */ }
///     #[cfg(feature = "ubx_proto23")]
///     UbxPacket::Proto23(p) => { /* handle proto23 */ }
///     #[cfg(feature = "ubx_proto27")]
///     UbxPacket::Proto27(p) => { /* handle proto27 */ }
///     #[cfg(feature = "ubx_proto31")]
///     UbxPacket::Proto31(p) => { /* handle proto31 */ }
/// }
/// ```
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
