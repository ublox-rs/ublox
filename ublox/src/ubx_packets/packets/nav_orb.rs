#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use core::fmt;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x34, max_payload_len = 908)]
struct NavOrb {
    /// GPS time of week in ms
    itow: u32,

    /// Message version (0x01)
    version: u8,

    num_svs: u8,

    reserved0: [u8; 2],

    #[ubx(
        map_type = NavOrbIter,
        from = NavOrbIter::new,
        is_valid = NavOrbIter::is_valid,
        may_fail,
        get_as_ref,
    )]
    svs: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct NavOrbIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NavOrbIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len().is_multiple_of(6)
    }
}

impl<'a> core::iter::Iterator for NavOrbIter<'a> {
    type Item = NavOrbSvInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 6];
            self.offset += 6;
            Some(NavOrbSvInfoRef(data))
        } else {
            None
        }
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x34, fixed_payload_len = 6)]
struct NavOrbSvInfo {
    gnss_id: u8,
    sv_id: u8,

    #[ubx(map_type = NavOrbSvFlag)]
    sv_flag: u8,

    #[ubx(map_type = NavOrbEph)]
    eph: u8,

    #[ubx(map_type = NavOrbAlm)]
    alm: u8,

    #[ubx(map_type = NavOrbOtherOrb)]
    other_orb: u8,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavOrbSvFlag(u8);

impl NavOrbSvFlag {
    pub fn health(self) -> NavOrbHealth {
        match self.0 & 0x3 {
            1 => NavOrbHealth::Healthy,
            2 => NavOrbHealth::Unhealthy,
            _ => NavOrbHealth::Unknown,
        }
    }

    pub fn visibility(self) -> NavOrbVisibility {
        match (self.0 >> 2) & 0x3 {
            1 => NavOrbVisibility::BelowHorizon,
            2 => NavOrbVisibility::AboveHorizon,
            3 => NavOrbVisibility::AboveElevationMask,
            _ => NavOrbVisibility::Unknown,
        }
    }

    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavOrbSvFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavOrbSvFlag")
            .field("health", &self.health())
            .field("visibility", &self.visibility())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavOrbEph(u8);

impl NavOrbEph {
    pub fn usability(self) -> u8 {
        self.0 & 0x1F
    }

    pub fn source(self) -> NavOrbSource {
        match (self.0 >> 5) & 0x7 {
            0 => NavOrbSource::NotAvailable,
            1 => NavOrbSource::GnssTransmission,
            2 => NavOrbSource::ExternalAiding,
            x => NavOrbSource::Other(x),
        }
    }

    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavOrbEph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavOrbEph")
            .field("usability", &self.usability())
            .field("source", &self.source())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavOrbAlm(u8);

impl NavOrbAlm {
    pub fn usability(self) -> u8 {
        self.0 & 0x1F
    }

    pub fn source(self) -> NavOrbSource {
        match (self.0 >> 5) & 0x7 {
            0 => NavOrbSource::NotAvailable,
            1 => NavOrbSource::GnssTransmission,
            2 => NavOrbSource::ExternalAiding,
            x => NavOrbSource::Other(x),
        }
    }

    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavOrbAlm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavOrbAlm")
            .field("usability", &self.usability())
            .field("source", &self.source())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavOrbOtherOrb(u8);

impl NavOrbOtherOrb {
    pub fn usability(self) -> u8 {
        self.0 & 0x1F
    }

    pub fn orb_type(self) -> NavOrbType {
        match (self.0 >> 5) & 0x7 {
            0 => NavOrbType::NoOrbitData,
            1 => NavOrbType::AssistNowOffline,
            2 => NavOrbType::AssistNowAutonomous,
            x => NavOrbType::Unknown(x),
        }
    }

    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavOrbOtherOrb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavOrbOtherOrb")
            .field("usability", &self.usability())
            .field("orb_type", &self.orb_type())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavOrbHealth {
    Unknown,
    Healthy,
    Unhealthy,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavOrbVisibility {
    Unknown,
    BelowHorizon,
    AboveHorizon,
    AboveElevationMask,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavOrbSource {
    NotAvailable,
    GnssTransmission,
    ExternalAiding,
    Other(u8),
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavOrbType {
    NoOrbitData,
    AssistNowOffline,
    AssistNowAutonomous,
    Unknown(u8),
}
