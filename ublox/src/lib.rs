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
        AnyPacketRef, FixedBuffer, FixedLinearBuffer, NmeaPacketRef, Parser, ParserBuilder,
        RtcmPacketRef, UbxParserIter, UnderlyingBuffer,
    },
    ubx_packets::*,
};

mod error;
mod parser;
mod ubx_packets;

pub mod constants;
pub mod proto14;
pub mod proto23;
pub mod proto27;
pub mod proto31;

/// Unified interface for UBX packets across different protocol versions.
///
/// Each variant corresponds to a UBX protocol version (14, 23, 27, 31).
///
/// Most users will only need one protocol, so enable only the relevant feature flag.
///
/// # Example
///
/// ```rust,ignore
/// # use ublox::{UbxPacket, proto23::PacketRef};
///
/// match packet {
///     UbxPacket::Proto23(p) => match p {
///         PacketRef::NavPvt(nav_pvt) => {
///             println!("Speed: {} m/s", nav_pvt.ground_speed_2d());
///         },
///         _ => {} // Other packet types
///     }
///     // Handle other protocol versions if needed
///     _ => {}
/// }
/// ```
#[derive(Debug)]
pub enum UbxPacket<'a> {
    #[cfg(feature = "ubx_proto14")]
    Proto14(proto14::PacketRef<'a>),
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
    const MAX_PAYLOAD_LEN: u16;

    /// Matches a Class ID, Message ID, and payload to a specific packet type.
    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, ParserError>;
}
