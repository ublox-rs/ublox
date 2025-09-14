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

#[cfg(feature = "ubx_proto14")]
pub mod proto17 {
    pub use crate::ubx_packets::packetref_proto17::PacketRef;
}
#[cfg(feature = "ubx_proto23")]
pub mod proto23 {
    pub use crate::ubx_packets::packetref_proto23::PacketRef;
}
#[cfg(feature = "ubx_proto27")]
pub mod proto27 {
    pub use crate::ubx_packets::packetref_proto27::PacketRef;
}
#[cfg(feature = "ubx_proto31")]
pub mod proto31 {
    pub use crate::ubx_packets::packetref_proto31::PacketRef;
}

mod error;
mod parser;
mod ubx_packets;
