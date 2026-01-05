#![cfg(any(
    feature = "ubx_proto27",
    feature = "ubx_proto31",
    feature = "ubx_proto33",
))]
use core::fmt;

#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::nav_sat::NavSatSvHealth;
#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x43, max_payload_len = 1240)]
struct NavSig {
    /// GPS time of week in ms
    itow: u32,

    /// Message version, should be 0
    version: u8,

    num_sigs: u8,

    reserved: u16,

    #[ubx(map_type = NavSigIter,
        may_fail,
        is_valid = NavSigIter::is_valid,
        from = NavSigIter::new,
        get_as_ref)]
    sigs: [u8; 0],
}

impl fmt::Debug for NavSigIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSigIter").finish()
    }
}

#[derive(Clone)]
pub struct NavSigIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NavSigIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % 16 == 0
    }
}

impl<'a> core::iter::Iterator for NavSigIter<'a> {
    type Item = NavSigInfoRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let data = &self.data[self.offset..self.offset + 16];
            self.offset += 16;
            Some(NavSigInfoRef(data))
        } else {
            None
        }
    }
}

/// This packet is not actually received as such, it is a block of the `NavSig` message
/// The `ubx_packet_recv` macro is used here as a shortcut to generate the needed code required for the repeated block.
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x35, fixed_payload_len = 16)]
pub(crate) struct NavSigInfo {
    gnss_id: u8,
    sv_id: u8,
    sig_id: u8,
    freq_id: u8,
    pr_res: i16,
    cno: u8,
    quality_ind: u8,
    corr_source: u8,
    ion_model: u8,
    #[ubx(map_type = NavSigFlags)]
    flags: u16,
    reserved: [u8; 4],
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NavSigFlags(u16);

impl NavSigFlags {
    /* Re-use the NavSatHealth enum for the signal health */
    pub fn health(self) -> NavSatSvHealth {
        let bits = self.0 & 0x3;
        match bits {
            1 => NavSatSvHealth::Healthy,
            2 => NavSatSvHealth::Unhealthy,
            x => NavSatSvHealth::Unknown(x as u8),
        }
    }

    pub fn pr_smoothed(self) -> bool {
        (self.0 >> 2) & 0x1 != 0
    }

    pub fn pr_used(self) -> bool {
        (self.0 >> 3) & 0x1 != 0
    }

    pub fn cr_used(self) -> bool {
        (self.0 >> 4) & 0x1 != 0
    }

    pub fn do_used(self) -> bool {
        (self.0 >> 5) & 0x1 != 0
    }

    pub fn pr_corr_used(self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn cr_corr_used(self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    pub fn do_corr_used(self) -> bool {
        (self.0 >> 8) & 0x1 != 0
    }

    pub const fn from(x: u16) -> Self {
        Self(x)
    }
}

impl fmt::Debug for NavSigFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavSatSvFlags")
            .field("health", &self.health())
            .field("pr_smoothed", &self.pr_smoothed())
            .field("pr__used", &self.pr_used())
            .field("cr__used", &self.cr_used())
            .field("do__used", &self.do_used())
            .field("pr_corr_used", &self.pr_corr_used())
            .field("cr_corr_used", &self.cr_corr_used())
            .field("do_corr_used", &self.do_corr_used())
            .finish()
    }
}
