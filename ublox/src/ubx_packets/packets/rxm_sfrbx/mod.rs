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
    /// GNSS identifier
    gnss_id: u8,

    /// Satellite identifier
    sv_id: u8,

    /// Reserved
    reserved1: u8,

    /// Only used for GLonass: this is the frequency slot +7
    freq_id: u8,

    /// Number of data words
    num_words: u8,

    /// Reserved
    reserved2: u8,

    /// Message version
    version: u8,

    /// Reserved
    reserved3: u8,

    /// Data words
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
pub enum RxmSfrbxInterpreted {
    /// [gps::RxmSfrbxGpsQzssFrame], applies to both Constellations.
    /// We support:
    /// - Ephemeris #1
    /// - Ephemeris #2
    /// - Ephemeris #3
    GpsQzss(gps::RxmSfrbxGpsQzssFrame),
}

struct RxmSfrbxInterpretor<'a> {
    /// Data word counter, to know where we are within current frame
    ptr: usize,

    /// GNSS-ID because frame interpretation is GNSS dependent
    gnss_id: u8,

    /// u32 (uninterpreated) words iterator
    iter: DwrdIter<'a>,
}

impl RxmSfrbxInterpretor<'_> {
    /// Consumes all data words, trying to obtain an [RxmSfrbxInterpreted],
    /// for supported frames.
    pub fn interpret(&mut self) -> Option<RxmSfrbxInterpreted> {
        // GPS Frame possibly constructed
        let mut gps = Option::<gps::GpsUnscaledFrame>::None;

        while let Some(dword) = self.iter.next() {
            self.ptr += 1; // increment position within frame

            match self.gnss_id {
                0 | 5 => {
                    // 0: GPS
                    // 5: QZSS applies similarly
                    if self.gps_decoding(dword, &mut gps).is_none() {
                        // no need to continue interpretation prcess
                        break;
                    }
                },
                _ => {}, // not applicable
            }
        }

        // final scaling & wrapping
        match self.gnss_id {
            0 | 5 => {
                // 0: GPS
                // 5: QZSS applies similarly
                let gps = gps?; // decoding went well
                let scaled = gps.scale()?; // scaling went well
                Some(RxmSfrbxInterpreted::GpsQzss(scaled))
            },
            _ => {
                // not supported yet
                None
            },
        }
    }

    fn gps_decoding(
        &mut self,
        dword: u32,
        interpreted: &mut Option<gps::GpsUnscaledFrame>,
    ) -> Option<()> {
        match self.ptr {
            1 => {
                // TLM word (must be valid)
                let telemetry = gps::RxmSfrbxGpsQzssTelemetry::decode(dword)?;
                let mut frame = GpsUnscaledFrame::default();
                frame.telemetry = telemetry;
                *interpreted = Some(frame);
            },
            2 => {
                // HOW word (must follow TLM).
                // After this step, the interpretation cannot fail.
                // It just must be wrapped correctly (many cases, basically indexed on frame_id & ptr).
                let how = gps::RxmSfrbxGpsQzssHow::decode(dword);
                if let Some(interpreted) = interpreted {
                    interpreted.how = how;
                } else {
                    // should never happen
                    return None;
                }
            },
            3 => {
                // Frame dependent construction
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            // GPS Ephemeris #1
                            let word3 = gps::GpsUnscaledEph1Word3::decode(dword);
                            let mut subframe1 = gps::GpsUnscaledEph1::default();
                            subframe1.word3 = word3;

                            interpreted.subframe = gps::GpsUnscaledSubframe::Eph1(subframe1);
                        },
                        2 => {
                            let word3 = gps::GpsUnscaledEph2Word3::decode(dword);
                            let mut subframe2 = gps::GpsUnscaledEph2::default();
                            subframe2.word3 = word3;

                            interpreted.subframe = gps::GpsUnscaledSubframe::Eph2(subframe2);
                        },
                        3 => {
                            let word3 = gps::GpsUnscaledEph3Word3::decode(dword);
                            let mut subframe3 = gps::GpsUnscaledEph3::default();
                            subframe3.word3 = word3;

                            interpreted.subframe = gps::GpsUnscaledSubframe::Eph3(subframe3);
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            4 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word4 = gps::GpsUnscaledEph1Word4::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word4 = gps::GpsUnscaledEph2Word4::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word4 = gps::GpsUnscaledEph3Word4::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            5 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word5 = gps::GpsUnscaledEph1Word5::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word5 = gps::GpsUnscaledEph2Word5::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word5 = gps::GpsUnscaledEph3Word5::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            6 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word6 = gps::GpsUnscaledEph1Word6::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word6 = gps::GpsUnscaledEph2Word6::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word6 = gps::GpsUnscaledEph3Word6::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            7 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word7 = gps::GpsUnscaledEph1Word7::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word7 = gps::GpsUnscaledEph2Word7::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word7 = gps::GpsUnscaledEph3Word7::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            8 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word8 = gps::GpsUnscaledEph1Word8::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word8 = gps::GpsUnscaledEph2Word8::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word8 = gps::GpsUnscaledEph3Word8::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            9 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word9 = gps::GpsUnscaledEph1Word9::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word9 = gps::GpsUnscaledEph2Word9::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word9 = gps::GpsUnscaledEph3Word9::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            10 => {
                // Frame dependent continuation
                if let Some(interpreted) = interpreted {
                    match interpreted.how.frame_id {
                        1 => {
                            let word10 = gps::GpsUnscaledEph1Word10::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph1(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        2 => {
                            let word10 = gps::GpsUnscaledEph2Word10::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph2(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        3 => {
                            let word10 = gps::GpsUnscaledEph3Word10::decode(dword);
                            match &mut interpreted.subframe {
                                gps::GpsUnscaledSubframe::Eph3(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {
                                    // not applicable
                                    return None;
                                },
                            }
                        },
                        // Almanac #4 not supported yet
                        // 4 => {
                        //     // frame # 4 is paginated.
                        // },
                        // Almanac #5 not supported yet
                        // 5 => {
                        //     // frame # 5 is paginated.
                        // },
                        _ => return None, // does not exist
                    }
                }
            },
            _ => {
                // does not exist in GPS
                return None;
            },
        }

        Some(())
    }
}

impl RxmSfrbxRef<'_> {
    /// Try to interprete the RXM-SFRBX inner frame (when supported/known).
    pub fn interpret(&self) -> Option<RxmSfrbxInterpreted> {
        self.interpretor().interpret()
    }

    /// Builds the [RxmSfrbxInterpretor] that can interpreta
    /// some of the inner words we support.
    fn interpretor(&self) -> RxmSfrbxInterpretor<'_> {
        RxmSfrbxInterpretor {
            ptr: 0,
            iter: self.dwrd(),
            gnss_id: self.gnss_id(),
        }
    }
}
