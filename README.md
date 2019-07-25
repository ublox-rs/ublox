ublox for Rust
==============

[![ublox on Travis CI][travis-image]][travis]

[travis-image]: https://api.travis-ci.com/lkolbly/ublox.svg?branch=master
[travis]: https://travis-ci.com/lkolbly/ublox

This project aims to build a pure-rust I/O library for ublox GPS devices, specifically using the UBX protocol.

In order to use a device, you must open it, and then you must periodically call `poll` to process incoming messages from the device (or call `poll_for`, which will call poll for the given amount of time). As new navigation solutions are created by the device, you can fetch them using the `get_solution` method.

In order to speed up the time to first fix, you may use the `load_aid_data` method to send the current position and time to the device, if known.

An example program is in `src/main.rs`

Roadmap
=======

- Currently, loading ALP offline data is flaky at best. I want to fix this, so that you can load ALP data to further increase startup times.
