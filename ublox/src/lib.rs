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
