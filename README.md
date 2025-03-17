uBlox for Rust
==============

[![Rust](https://github.com/ublox-rs/ublox/actions/workflows/build.yml/badge.svg)](https://github.com/ublox-rs/ublox/actions/workflows/build.yml)
[![ublox on docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![rustc v1.81][mrvs-badge]][mrvs-url]

[docs-badge]: https://docs.rs/ublox/badge.svg
[docs-url]: https://docs.rs/ublox
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/lkolbly/ublox/blob/master/LICENSE.md
[mrvs-url]: https://www.whatrustisit.com
[mrvs-badge]: https://img.shields.io/badge/minimum%20rustc-1.81-blue?logo=rust

# Table of Contents

* [Introduction](#introduction)
* [Basic Usage](#basic-usage)
    * [Constructing Packets](#constructing-packets)
    * [Parsing Packets](#parsing-packets)
* [Examples](#examples)
* [Feature Flags](#feature-flags)
* [Minimum Supported Rust Version](#minimum-supported-rust-version)
* [Contributing](#contributing)
* [License](#license)

# Introduction

This project aims to build a pure-rust I/O library for uBlox GPS devices, specifically using the UBX protocol.

The crate has originally been developed for Series 8 uBlox devices, but it is being currently adapted to support other protocol specifications and uBlox devices.

# Basic Usage

## Constructing Packets

Constructing packets happens using the `Builder` variant of the packet, for example:

```rust
use ublox::{CfgPrtUartBuilder, UartPortId, UartMode, DataBits, Parity, StopBits, InProtoMask, OutProtoMask};
let packet: [u8; 28] = CfgPrtUartBuilder {
   portid: UartPortId::Uart1,
   reserved0: 0,
   tx_ready: 0,
   mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
   baud_rate: 9600,
   in_proto_mask: InProtoMask::all(),
   out_proto_mask: OutProtoMask::UBLOX,
   flags: 0,
   reserved5: 0,
}.into_packet_bytes();
```

For variable-size packets like `CfgValSet`, you can construct it into a new `Vec<u8>`:

```rust
use ublox::{cfg_val::CfgVal::*, CfgLayer, CfgValSetBuilder};
let packet_vec: Vec<u8> = CfgValSetBuilder {
    version: 1,
    layers: CfgLayer::RAM,
    reserved1: 0,
    cfg_data: &[UsbOutProtNmea(true), UsbOutProtRtcm3x(true), UsbOutProtUbx(true)],
}
.into_packet_vec();
let packet: &[u8] = packet_vec.as_slice();
```

Or by extending to an existing one:

```rust
let mut packet_vec = Vec::new();
CfgValSetBuilder {
    version: 1,
    layers: CfgLayer::RAM,
    reserved1: 0,
    cfg_data: &[UsbOutProtNmea(true), UsbOutProtRtcm3x(true), UsbOutProtUbx(true)],
}
.extend_to(&mut packet_vec);
let packet = packet_vec.as_slice();
```
See the documentation for the individual `Builder` structs for information on the fields.

## Parsing Packets

Parsing packets happens by instantiating a `Parser` object and then adding data into it using its `consume()` method. The parser contains an internal buffer of data, and when `consume()` is called that data is copied into the internal buffer and an iterator-like object is returned to access the packets. For example:

```rust
use ublox::Parser;
let mut parser = Parser::default();
let my_raw_data = vec![1, 2, 3, 4]; // From your serial port
let mut it = parser.consume(&my_raw_data);
loop {
    match it.next() {
        Some(Ok(packet)) => {
            // We've received a &PacketRef, we can handle it
        }
        Some(Err(_)) => {
            // Received a malformed packet
        }
        None => {
            // The internal buffer is now empty
            break;
        }
    }
}
```

# Examples

For a list of examples and their description see the [examples/](./examples/README.md) directory. 

# Feature Flags

The following feature flags are available:

### `alloc`
Enable usage of heap allocated Vectors from `core::vec`. 

### `serde`

Enable `serde` support. 

### `std`

Enable `std` support. 

This library supports no_std environments with a deterministic-size `Parser`. See the documentation for more information.

### `ubx_proto23`
Enable support for uBlox protocol 23 messages (default).

### `ubx_proto27`
Enable support for uBlox protocol 27 messages. 

### `ubx_proto31`
Enable support for uBlox protocol 31 messages. 

# Minimum Supported Rust Version

The library crate will support at least the previous year's Rust compilers. Currently, the MSRV is `1.81.0`. Note that, as we are pre-1.0, breaking the MSRV will not force a minor update - the MSRV can change in a patch update.

# Contributing

* If you have noticed a bug or would like to see a new feature added, please submit an issue on the [issue tracker](https://github.com/ublox-rs/ublox/issues) and preferably a Pull Request.

# License

`ublox.rs` is distributed under the terms of the MIT license, see [LICENSE](https://github.com/ublox-rs/ublox/tree/master/LICENSE).
