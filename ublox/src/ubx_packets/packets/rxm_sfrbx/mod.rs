#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

mod gps;
pub use gps::*;

#[ubx_packet_recv]
#[ubx(class = 0x02, id = 0x13, max_payload_len = 72)]
struct RxmSfrbx {
    gnss_id: u8,
    sv_id: u8,
    reserved1: u8,
    freq_id: u8,
    num_words: u8,
    reserved2: u8,
    version: u8,
    reserved3: u8,
    #[ubx(
        map_type = DwrdIter,
        from = DwrdIter::new,
        is_valid = DwrdIter::is_valid,
        may_fail,
    )]
    dwrd: [u8; 0],
}

#[derive(Debug, Clone)]
pub struct DwrdIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> DwrdIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        DwrdIter(bytes.chunks_exact(4))
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % 4 == 0
    }
}

impl core::iter::Iterator for DwrdIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

#[derive(Debug, Clone)]
pub enum RxmSfrbxInterprated {
    /// GPS Subframe
    Gps(GpsDataWord),
}

pub struct RxmSfrbxInterprator<'a> {
    dword: usize,
    gnss_id: u8,
    iter: DwrdIter<'a>,
}

impl core::iter::Iterator for RxmSfrbxInterprator<'_> {
    type Item = RxmSfrbxInterprated;

    fn next(&mut self) -> Option<Self::Item> {
        let dword = self.iter.next()?;

        let ret = match self.gnss_id {
            0 => {
                // GPS interpretation
                match self.dword {
                    0 => {
                        let tlm = gps::GpsTelemetryWord::decode(dword)?;
                        Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Telemetry(tlm)))
                    },
                    1 => {
                        let how = gps::GpsHowWord::decode(dword)?;
                        Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::How(how)))
                    },
                    _ => None,
                }
            },
            _ => {
                // Not interprated yet
                None
            },
        };

        self.dword += 1;
        ret
    }
}

impl RxmSfrbxRef<'_> {
    pub fn interprator(&self) -> RxmSfrbxInterprator {
        RxmSfrbxInterprator {
            dword: 0,
            iter: self.dwrd(),
            gnss_id: self.gnss_id(),
        }
    }
}
