# Changelog

All notable changes to this project will be documented in this file.

## [0.9.0]

### Added

- Add SEC-SIG and SEC-SIGLOG packets
- Add NAV-COV and NAV-POSECEF packets
- Add RXM-COR and MON-COMMS packets
- Implementation for From\<CfgVal\> for CfgKey and derives for CfgVal/CfgKey

### Changed

 - Marked PacketRef enums as `#[non_exhaustive]`


## [0.8.0]

### Added

- Add support for extracting NMEA packets from byte stream

### Changed

 - Derive `Clone` trait for owned packets 

## [0.7.0]

### Added

- Add remaining `UBX-CFG-NAVSPG-*` messages for protocol 27
- Add `UBX-MON-HW2` and `UBX-MON-HW3`
- Add `UBX-RF`
- Add API to get a packet's payload length
- Add fuzz testing for `UBX-NAV-HPPOSLLH` and `UBX-NAV-PVT` 

### Changed

 - Bump MSRV from 1.82 to 1.83 for const `.to_le_bytes()` for f32 + f64
 - Enable all features for building and publishing docs 
 - Add a FixedBuffer implementation that owns the byte array 
 - Add capability to send `UBX-CFG-GNSS` packets 
 - Add capability to send `UBX-ESF-MEAS` packets 
 - Renamed `CfgValGetSend` and `CfgValGetRecv` to `CfgValGetRequest` and `CfgValGetResponse`

### Misc

- Fix Rust 1.89.0 clippy lints

## [0.6.0]

### Added

- Add owned variants of `PacketRef` ([#103](https://github.com/ublox-rs/ublox/pull/103))
- Split large file that defined all packets definition into individual files ([#129])(https://github.com/ublox-rs/ublox/pull/129)
- Some of the new `CFG-NAVSPG-*` messages
- Add new `invalidLlh` bitflag for `UBX-NAV-HPPOSLLH` protocol 27 & 31

### Breaking

- Fixed typo: `NavBbrMask::OSCILATOR_PARAMETER` to `NavBbrMask::OSCILLATOR_PARAMETER` ([#118](https://github.com/ublox-rs/ublox/pull/118))
- Fixed typo: `CfgTmode2Ref::survery_in_accur_limit_raw` to `CfgTmode2Ref::survey_in_accur_limit_raw` ([#118](https://github.com/ublox-rs/ublox/pull/118))
- Fixed typo: `CfgTmode2Ref::survery_in_accur_limit` to `CfgTmode2Ref::survey_in_accur_limit` ([#118](https://github.com/ublox-rs/ublox/pull/118))
- Fixed typo: `HnrPvtRef::heading_accurracy_raw` to `HnrPvtRef::heading_accuracy_raw` ([#118](https://github.com/ublox-rs/ublox/pull/118))
- Fixed typo: `HnrPvtRef::heading_accurracy` to `HnrPvtRef::heading_accuracy` ([#118](https://github.com/ublox-rs/ublox/pull/118))
- Fixed typo: `CfgTmode2Builder.survery_in_accur_limit` to `CfgTmode2Builder.survey_in_accur_limit` ([#118](https://github.com/ublox-rs/ublox/pull/118))

- Almost all `CfgVal` enum variants have been renamed to have a consistent CamelCase representation ([#106](https://github.com/ublox-rs/ublox/pull/106))
- Added new Packet variant `CfgValGetRecv` for UBX Proto 27 and UBX Proto 31 ([#106](https://github.com/ublox-rs/ublox/pull/106))
- Replaced `CfgLayer` by `CfgLayerSet` to differentiate it from `CfgLayerGet` ([#106](https://github.com/ublox-rs/ublox/pull/106))
- Renamed `NavSolution` to `NavSol` ([#129])(https://github.com/ublox-rs/ublox/pull/129)
- Removed `AlpSrv` from protocol versions 27 and 31 ([#129])(https://github.com/ublox-rs/ublox/pull/129)
- Rename `CfgNav5FixMode` to `NavFixMode` as it is also used in `CfgVal` messages
- Rename `CfgNav5DynModel` to `NavDynamicModel` as it is also used in `CfgVal` messages, and changed the default to `Portable` as specified in the documentation

### CI

- Add typo checking workflow ([#122](https://github.com/ublox-rs/ublox/pull/122))
- Add link checking workflow ([#117](https://github.com/ublox-rs/ublox/pull/117))

## [0.5.0] - 2025-03-17

### 💼 What's Changed

- Prepare for next release ([#95](https://github.com/ublox-rs/ublox/pull/95))
  - remove duplicate CI file
  - cherry-picked NavSig from PR ([#73](https://github.com/ublox-rs/ublox/pull/73))
  - add semver to CI
- Added comments and scaling for NavRelPosNed packets ([#93](https://github.com/ublox-rs/ublox/pull/93))
- Separate PacketRef enum into own file and CI improvements ([#94](https://github.com/ublox-rs/ublox/pull/94))
  - set rust-version in primary workspace
  - add CI check for msrv
- Create a release script ([#89](https://github.com/ublox-rs/ublox/pull/89))
- Introduce feature flags for UBX protocol versions ([#87](https://github.com/ublox-rs/ublox/pull/87))
  - differentiate between uBlox prototocol/series
  - add build.rs to force single feature for protocol version
  - duplicate PacketRef enum per protocol version
  - add CFG-ESFWT message
  - fix bug in ESF MEAS decoding
  - refactor ublox_device into a lib to be used by all examples
  - add more examples: a TUI based on `ratatui` to show NavPvt similar to uCenter and DDS (Data Distribution Service middleware) example
- Add new ESF, HNR packages and other updates  ([#86](https://github.com/ublox-rs/ublox/pull/86))
  - add new ubx packets and refactor
  - add HNR-ATT, HNR-INS, ESF-ALG, ESF-STATUS and CFG-ESFALG
  - rename NAV-PVT message as per ublox protocol description
  - make HNR-PVT and NAV-PVT fields uniform
  - add extra NAV-PVT mapping functions
  - make itow field naming uniform across packages
  - implemented decoding of sensor measurement for ESF-MEAS
  - bump MSRV to 1.82
- Packets.rs: introduce NavClock and TimTos ([#45](https://github.com/ublox-rs/ublox/pull/45))
- Packets.rs: introduce CfgSmgr synchronization core configuration frame ([#46](https://github.com/ublox-rs/ublox/pull/46))
- Add NavRelPosNed ([#24](https://github.com/ublox-rs/ublox/pull/24))
- Add undocumented L5 command ([#79](https://github.com/ublox-rs/ublox/pull/79))

### Breaking Changes

---

 - `NavPvt` packet: the majority of getters & aliases have been renamed (check the new packet definition for the complete list)
 - `HnrPvt` packet: renamed getters & aliases to align them with the similar `NavPvt` packet
 - `NavSatSvInfo` packet: added extra `Invalid` enum variant to the `NavSatQualityIndicator`
 - `EsfMeas` packet: renamed `time_tag` field & getter to `itow` to align with other packets and introduced `EsfSensorType` for sensor data type
 - `EsfIns` packet: renamed field & getter `bit_field` to `bitfield`
 - `HnrIns` packet: renamed field & getter `bit_field` to `bitfield`
 - `NavClock` packet: renamed fields & getters
