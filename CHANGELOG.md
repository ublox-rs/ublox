# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2025-03-17

### 💼 What's Changed

- Prepare for next release ([#95](https://github.com/ublox-rs/ublox/pull/95))
  - remove duplicate CI file
  - cherry-picked NavSig from PR [#73](https://github.com/ublox-rs/ublox/pull/73)
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
  - bump MSRV to 1.81
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


