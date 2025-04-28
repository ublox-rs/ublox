use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

/// GNSS status monitoring,
/// gives currently selected constellations
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x28, fixed_payload_len = 8)]
struct MonGnss {
    /// Message version: 0x00
    version: u8,
    /// Supported major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    supported: u8,
    /// Default major GNSS constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    default: u8,
    /// Currently enabled major constellations bit mask
    #[ubx(map_type = MonGnssConstellMask)]
    enabled: u8,
    /// Maximum number of concurrent Major GNSS
    /// that can be supported by this receiver
    simultaneous: u8,
    reserved1: [u8; 3],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Selected / available Constellation Mask
    #[derive(Default, Debug)]
    pub struct MonGnssConstellMask: u8 {
        /// GPS constellation
        const GPS = 0x01;
        /// GLO constellation
        const GLO = 0x02;
        /// BDC constellation
        const BDC = 0x04;
        /// GAL constellation
        const GAL = 0x08;
    }
}
