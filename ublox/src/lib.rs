#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;
#[cfg(feature = "serde")]
extern crate serde;

pub use crate::{
    error::{DateTimeError, MemWriterError, ParserError},
    parser::{FixedLinearBuffer, Parser, ParserIter, UnderlyingBuffer},
    ubx_packets::*,
};

mod error;
mod parser;
mod ubx_packets;
