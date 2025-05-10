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
    /// Data word counter, to know where we are within current frame
    dword: usize,

    /// GNSS-ID because frame interpretation is GNSS dependent
    gnss_id: u8,

    /// Stored frame_id to continue the parsing process.
    /// GPS uses 3 bits, we'll see if this need to change as others are introduced.
    frame_id: u8,

    /// u32 (uninterpreated) words iterator
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
                        let how = gps::GpsHowWord::decode(dword);

                        // store frame_id for later
                        self.frame_id = how.frame_id;

                        Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::How(how)))
                    },
                    2 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 3
                                let decoded = gps::GpsSubframe2Word3::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word3(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #2 word 3
                                let decoded = gps::GpsSubframe3Word3::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word3(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    3 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 4
                                let decoded = gps::GpsSubframe2Word4::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word4(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #2 word 4
                                let decoded = gps::GpsSubframe3Word4::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word4(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    4 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 5
                                let decoded = gps::GpsSubframe2Word5::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word5(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 5
                                let decoded = gps::GpsSubframe3Word5::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word5(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    5 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 6
                                let decoded = gps::GpsSubframe2Word6::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word6(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 6
                                let decoded = gps::GpsSubframe3Word6::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word6(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    6 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 7
                                let decoded = gps::GpsSubframe2Word7::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word7(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 7
                                let decoded = gps::GpsSubframe3Word7::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word7(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    7 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 8
                                let decoded = gps::GpsSubframe2Word8::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word8(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 8
                                let decoded = gps::GpsSubframe3Word8::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word8(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    8 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 9
                                let decoded = gps::GpsSubframe2Word9::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word9(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 9
                                let decoded = gps::GpsSubframe3Word9::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word9(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    9 => {
                        // Frame dependent
                        match self.frame_id {
                            2 => {
                                // GPS Frame #2 word 10
                                let decoded = gps::GpsSubframe2Word10::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe2Word10(
                                    decoded,
                                )))
                            },
                            3 => {
                                // GPS Frame #3 word 10
                                let decoded = gps::GpsSubframe3Word10::decode(dword);
                                Some(RxmSfrbxInterprated::Gps(gps::GpsDataWord::Subframe3Word10(
                                    decoded,
                                )))
                            },
                            _ => None, // not supported yet
                        }
                    },
                    _ => {
                        // following GPS dword not interprated yet
                        None
                    },
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
            frame_id: 0,
            iter: self.dwrd(),
            gnss_id: self.gnss_id(),
        }
    }
}
