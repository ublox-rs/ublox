uBlox for Rust
==============

[![Rust](https://github.com/ublox-rs/ublox/actions/workflows/build.yml/badge.svg)](https://github.com/ublox-rs/ublox/actions/workflows/build.yml)
[![ublox on docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![rustc v1.65][mrvs-badge]][mrvs-url]

[docs-badge]: https://docs.rs/ublox/badge.svg
[docs-url]: https://docs.rs/ublox
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/lkolbly/ublox/blob/master/LICENSE.md
[mrvs-url]: https://www.whatrustisit.com
[mrvs-badge]: https://img.shields.io/badge/minimum%20rustc-1.65-blue?logo=rust

This project aims to build a pure-rust I/O library for uBlox GPS devices, specifically using the UBX protocol.

Examples of usage of the library can be found in the [examples/](./examples) directory. A basic CLI for interacting with an uBlox device can be found in [examples/ublox-cli](./examples/ublox-cli/) directory. See the specific [examples/README](./examples/README.md).

Constructing Packets
====================

Constructing packets happens using the `Builder` variant of the packet, for example:
```
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

For variable-size packet like `CfgValSet`, you can construct it into a new `Vec<u8>`:
```
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
```
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

Parsing Packets
===============

Parsing packets happens by instantiating a `Parser` object and then adding data into it using its `consume()` method. The parser contains an internal buffer of data, and when `consume()` is called that data is copied into the internal buffer and an iterator-like object is returned to access the packets. For example:
```
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

no_std Support
==============

This library supports no_std environments with a deterministic-size `Parser`. See the documentation for more information.

Minimum Supported Rust Version
==============================

The library crate will support at least the previous year's worth of Rust compilers. Currently, the MSRV is `1.65.0`. Note that, as we are pre-1.0, breaking the MSRV will not force a minor update - the MSRV can change in a patch update.
