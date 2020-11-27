//! # ublox
//!
//! `ublox` is a library to talk to u-blox GPS devices using the UBX protocol.
//! At time of writing this library is developed for a device which behaves like
//! a NEO-6M device.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use crate::{
    error::{DateTimeError, MemWriterError, ParserError},
    parser::{Parser, ParserIter},
    ubx_packets::*,
};

mod error;
mod parser;
mod ubx_packets;
