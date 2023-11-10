//! # ublox
//!
//! This project aims to build a pure-rust I/O library for ublox GPS devices, specifically using the UBX protocol.
//!
//! An example of using this library to talk to a device can be seen in the examples/ublox-cli subfolder of this project.
//! More examples are available in the examples subfolder.
//!
//! Constructing Packets
//! ====================
//!
//! Constructing packets happens using the `Builder` variant of the packet, for example:
//! ```
//! use ublox::{CfgPrtUartBuilder, UartPortId, UartMode, DataBits, Parity, StopBits, InProtoMask, OutProtoMask};
//!
//! let packet: [u8; 28] = CfgPrtUartBuilder {
//!    portid: UartPortId::Uart1,
//!    reserved0: 0,
//!    tx_ready: 0,
//!    mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
//!    baud_rate: 9600,
//!    in_proto_mask: InProtoMask::all(),
//!    out_proto_mask: OutProtoMask::UBLOX,
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
//! # #[cfg(any(feature = "alloc", feature = "std"))] {
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
//!             // The internal buffer is now empty
//!             break;
//!         }
//!     }
//! }
//! # }
//! ```
//!
//! no_std Support
//! ==============
//!
//! This library additionally supports no_std environments with a deterministic-size parser. To use this parser, simply create a FixedLinearBuffer and use it to construct a `Parser` object:
//! ```
//! let mut buf = vec![0; 256];
//! let buf = ublox::FixedLinearBuffer::new(&mut buf[..]);
//! let mut parser = ublox::Parser::new(buf);
//! ```
//! The resulting parser can be used like normal. The absolute smallest recommended buffer size is 36 bytes, large enough to contain a NavPosLlh packet.

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
