ublox for Rust
==============

[![ublox on Travis CI][travis-image]][travis]

[travis-image]: https://api.travis-ci.com/lkolbly/ublox.svg?branch=master
[travis]: https://travis-ci.com/lkolbly/ublox

This project aims to build a pure-rust I/O library for ublox GPS devices, specifically using the UBX protocol.

An example of using this library to talk to a device can be seen in the ublox_cli subfolder of this project.

Constructing Packets
====================

Constructing packets happens using the `Builder` variant of the packet, for example:
```
let packet: Vec<u8> = CfgPrtUartBuilder {
   portid: UartPortId::Uart1,
   ...
}.into_packet_bytes();
```
See the documentation for the individual `Builder` structs for information on the fields.

Parsing Packets
===============

Parsing packets happens by instantiating a `Parser` object and then adding data into it using its `consume()` method. The parser contains an internal buffer of data, and when `consume()` is called that data is copied into the internal buffer and an iterator-like object is returned to access the packets. For example:
```
let mut parser = Parser::default();
let my_raw_data = ...;
let it = parser.consume(my_raw_data);
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
