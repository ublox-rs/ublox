#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use core::fmt;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, max_payload_len = 1240)]
struct NavSat {
    /// GPS time of week in ms
    itow: u32,

    /// Message version, should be 1
    version: u8,

    num_svs: u8,

    reserved: [u8; 2],

    #[ubx(
        map_type = NavSatIter,
        from = NavSatIter::new,
        is_valid = NavSatIter::is_valid,
        may_fail,
        get_as_ref,
    )]
    svs: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct NavSatIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NavSatIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len().is_multiple_of(12)
    }
}

impl<'a> core::iter::Iterator for NavSatIter<'a> {
    type Item = NavSatSvInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 12];
            self.offset += 12;
            Some(NavSatSvInfoRef(data))
        } else {
            None
        }
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, fixed_payload_len = 12)]
struct NavSatSvInfo {
    gnss_id: u8,
    sv_id: u8,
    cno: u8,
    elev: i8,
    azim: i16,
    pr_res: i16,

    #[ubx(map_type = NavSatSvFlags)]
    flags: u32,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavSatSvFlags(u32);

impl NavSatSvFlags {
    pub fn quality_ind(self) -> NavSatQualityIndicator {
        let bits = self.0 & 0x7;
        match bits {
            0 => NavSatQualityIndicator::NoSignal,
            1 => NavSatQualityIndicator::Searching,
            2 => NavSatQualityIndicator::SignalAcquired,
            3 => NavSatQualityIndicator::SignalDetected,
            4 => NavSatQualityIndicator::CodeLock,
            5..=7 => NavSatQualityIndicator::CarrierLock,
            _ => NavSatQualityIndicator::Invalid,
        }
    }

    pub fn sv_used(self) -> bool {
        (self.0 >> 3) & 0x1 != 0
    }

    pub fn health(self) -> NavSatSvHealth {
        let bits = (self.0 >> 4) & 0x3;
        match bits {
            1 => NavSatSvHealth::Healthy,
            2 => NavSatSvHealth::Unhealthy,
            x => NavSatSvHealth::Unknown(x as u8),
        }
    }

    pub fn differential_correction_available(self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn smoothed(self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    pub fn orbit_source(self) -> NavSatOrbitSource {
        let bits = (self.0 >> 8) & 0x7;
        match bits {
            0 => NavSatOrbitSource::NoInfoAvailable,
            1 => NavSatOrbitSource::Ephemeris,
            2 => NavSatOrbitSource::Almanac,
            3 => NavSatOrbitSource::AssistNowOffline,
            4 => NavSatOrbitSource::AssistNowAutonomous,
            x => NavSatOrbitSource::Other(x as u8),
        }
    }

    pub fn ephemeris_available(self) -> bool {
        (self.0 >> 11) & 0x1 != 0
    }

    pub fn almanac_available(self) -> bool {
        (self.0 >> 12) & 0x1 != 0
    }

    pub fn an_offline_available(self) -> bool {
        (self.0 >> 13) & 0x1 != 0
    }

    pub fn an_auto_available(self) -> bool {
        (self.0 >> 14) & 0x1 != 0
    }

    pub fn sbas_corr(self) -> bool {
        (self.0 >> 16) & 0x1 != 0
    }

    pub fn rtcm_corr(self) -> bool {
        (self.0 >> 17) & 0x1 != 0
    }

    pub fn slas_corr(self) -> bool {
        (self.0 >> 18) & 0x1 != 0
    }

    pub fn spartn_corr(self) -> bool {
        (self.0 >> 19) & 0x1 != 0
    }

    pub fn pr_corr(self) -> bool {
        (self.0 >> 20) & 0x1 != 0
    }

    pub fn cr_corr(self) -> bool {
        (self.0 >> 21) & 0x1 != 0
    }

    pub fn do_corr(self) -> bool {
        (self.0 >> 22) & 0x1 != 0
    }

    pub const fn from(x: u32) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavSatSvFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSatSvFlags")
            .field("quality_ind", &self.quality_ind())
            .field("sv_used", &self.sv_used())
            .field("health", &self.health())
            .field(
                "differential_correction_available",
                &self.differential_correction_available(),
            )
            .field("smoothed", &self.smoothed())
            .field("orbit_source", &self.orbit_source())
            .field("ephemeris_available", &self.ephemeris_available())
            .field("almanac_available", &self.almanac_available())
            .field("an_offline_available", &self.an_offline_available())
            .field("an_auto_available", &self.an_auto_available())
            .field("sbas_corr", &self.sbas_corr())
            .field("rtcm_corr", &self.rtcm_corr())
            .field("slas_corr", &self.slas_corr())
            .field("spartn_corr", &self.spartn_corr())
            .field("pr_corr", &self.pr_corr())
            .field("cr_corr", &self.cr_corr())
            .field("do_corr", &self.do_corr())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatQualityIndicator {
    NoSignal,
    Searching,
    SignalAcquired,
    SignalDetected,
    CodeLock,
    CarrierLock,
    Invalid,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatSvHealth {
    Healthy,
    Unhealthy,
    Unknown(u8),
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavSatOrbitSource {
    NoInfoAvailable,
    Ephemeris,
    Almanac,
    AssistNowOffline,
    AssistNowAutonomous,
    Other(u8),
}
