# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2025-03-17

### 💼 Changed

- Add undocumented L5 command (#79)
- Add new ESF, HNR packages and other updates  (#86)
  - add new ubx packets and refactor
  - add HNR-ATT, HNR-INS, ESF-ALG, ESF-STATUS and CFG-ESFALG
  - rename NAV-PVT message as per ublox protocol description
  - make HNR-PVT and NAV-PVT fields uniform
  - add extra NAV-PVT mapping functions
  - make itow field naming uniform across packages
  - implemented decoding of sensor measurement for ESF-MEAS
  - bump MSRV to 1.81
- Packet.rs: introduce CfgSmgr synchronization core configuration frame (#46)
- Packets.rs: introduce NavClock and TimTos (#45)
- Add NavRelPosNed (#24)
- Introduce feature flags for UBX protocol versions (#87)
  - differentiate between uBlox prototocol/series
  - add build.rs to force single feature for protocol version
  - duplicate PacketRef enum per protocol version
  - add CFG-ESFWT message
  - fix bug in ESF MEAS decoding
  - refactor ublox_device into a lib to be used by all examples
  - add more examples: a TUI based on `ratatui` to show NavPvt similar to uCenter and DDS (Data Distribution Service middleware) example
- Create a release script (#89)
- Separate PacketRef enum into own file and CI improvements (#94)
  - set rust-version in primary workspace
  - add CI check for msrv
- Added comments and scaling for NavRelPosNed packets (#93)
- Prepare for next release (#95)
  - remove duplicate CI file
  - cherry-picked NavSig from PR #73
  - add semver to CI
