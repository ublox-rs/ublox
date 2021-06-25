//! # ublox
//!
//! This project aims to build a pure-rust I/O library for ublox GPS devices, specifically using the UBX protocol.
//!
//! An example of using this library to talk to a device can be seen in the ublox_cli subfolder of this project.
//!
//! Constructing Packets
//! ====================
//!
//! Constructing packets happens using the `Builder` variant of the packet, for example:
//! ```
//! use ublox::{CfgPrtUartBuilder, UartPortId};
//!
//! let packet: [u8; 28] = CfgPrtUartBuilder {
//!    portid: UartPortId::Uart1,
//!    reserved0: 0,
//!    tx_ready: 0,
//!    mode: 0x8d0,
//!    baud_rate: 9600,
//!    in_proto_mask: 0x07,
//!    out_proto_mask: 0x01,
//!    flags: 0,
//!    reserved5: 0,
//! }.into_packet_bytes();
//! ```
//! See the documentation for the individual `Builder` structs for information on the fields.
//!
//! Parsing Packets
//! ===============
//!
//! Parsing packets happens by instantiating a `Parser` object and then adding data into it using its `consume()` method. The parser contains an internal buffer of data, and when `consume()` is called that data is copied into the internal buffer and an iterator-like object is returned to access the packets. For example:
//! ```
//! use ublox::Parser;
//!
//! let mut parser = Parser::default();
//! let my_raw_data = vec![1, 2, 3, 4]; // From your serial port
//! let mut it = parser.consume(&my_raw_data);
//! loop {
//!     match it.next() {
//!         Some(Ok(packet)) => {
//!             // We've received a &PacketRef, we can handle it
//!         }
//!         Some(Err(_)) => {
//!             // Received a malformed packet
//!         }
//!         None => {
//! 	    // The internal buffer is now empty
//!             break;
//!         }
//!     }
//! }
//! ```
//!
//! no_std Support
//! ==============
//!
//! This library supports no_std environments, with two caveats about the `Parser` traits:
//! - You must have a global allocator configured, and
//! - If your device is "poorly behaved" (i.e. is not a real u-blox GPS device but a maliciously designed device), then the amount of memory allocated can grow up to 64K.
//!
//! A deterministic-memory `Parser` implementation is on the roadmap.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use crate::{
    error::{DateTimeError, MemWriterError, ParserError},
    parser::{FixedLinearBuffer, Parser, ParserIter, UnderlyingBuffer},
    ubx_packets::*,
};

mod error;
mod parser;
mod ubx_packets;
