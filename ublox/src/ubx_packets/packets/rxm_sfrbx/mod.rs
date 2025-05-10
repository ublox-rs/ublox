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
    /// [gps::GpsFrame]
    GPS(gps::GpsFrame),
}

struct RxmSfrbxInterprator<'a> {
    /// Data word counter, to know where we are within current frame
    ptr: usize,

    /// GNSS-ID because frame interpretation is GNSS dependent
    gnss_id: u8,

    /// u32 (uninterpreated) words iterator
    iter: DwrdIter<'a>,
}

impl RxmSfrbxInterprator<'_> {
    /// Consumes all data words, trying to obtain an [RxmSfrbxInterprated],
    /// for supported frames.
    pub fn interprate(&mut self) -> Option<RxmSfrbxInterprated> {
        // GPS Frame possibly constructed
        let mut gps = Option::<gps::GpsFrame>::None;

        while let Some(dword) = self.iter.next() {
            self.ptr += 1; // increment position within frame

            match self.gnss_id {
                0 => {
                    if self.gps_interpration(dword, &mut gps).is_none() {
                        // no need to continue interpretation prcess
                        break;
                    }
                },
                _ => {}, // not applicable
            }
        }

        // final wrapping
        match self.gnss_id {
            0 => {
                let gps = gps?; // decoding went well
                Some(RxmSfrbxInterprated::GPS(gps))
            },
            _ => {
                // not supported yet
                None
            },
        }
    }

    fn gps_interpration(
        &mut self,
        dword: u32,
        interprated: &mut Option<gps::GpsFrame>,
    ) -> Option<()> {
        match self.ptr {
            1 => {
                // TLM word (must be valid)
                let telemetry = gps::GpsTelemetryWord::decode(dword)?;
                let mut frame = GpsFrame::default();
                frame.telemetry = telemetry;
                *interprated = Some(frame);
            },
            2 => {
                // HOW word (must follow TLM).
                // After this step, the interpratation cannot fail.
                // It just must be wrapped correctly (many cases, basically indexed on frame_id & ptr).
                let how = gps::GpsHowWord::decode(dword);
                if let Some(interprated) = interprated {
                    interprated.how = how;
                }
            },
            3 => {
                // Frame dependent construction
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word3 = gps::GpsSubframe1Word3::decode(dword);
                            let mut subframe1 = gps::GpsSubframe1::default();
                            subframe1.word3 = word3;

                            interprated.subframe = gps::GpsSubframe::Subframe1(subframe1);
                        },
                        2 => {
                            let word3 = gps::GpsSubframe2Word3::decode(dword);
                            let mut subframe2 = gps::GpsSubframe2::default();
                            subframe2.word3 = word3;

                            interprated.subframe = gps::GpsSubframe::Subframe2(subframe2);
                        },
                        3 => {
                            let word3 = gps::GpsSubframe3Word3::decode(dword);
                            let mut subframe3 = gps::GpsSubframe3::default();
                            subframe3.word3 = word3;

                            interprated.subframe = gps::GpsSubframe::Subframe3(subframe3);
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            4 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word4 = gps::GpsSubframe1Word4::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word4 = gps::GpsSubframe2Word4::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word4 = gps::GpsSubframe3Word4::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word4 = word4;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            5 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word5 = gps::GpsSubframe1Word5::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word5 = gps::GpsSubframe2Word5::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word5 = gps::GpsSubframe3Word5::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word5 = word5;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            6 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word6 = gps::GpsSubframe1Word6::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word6 = gps::GpsSubframe2Word6::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word6 = gps::GpsSubframe3Word6::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word6 = word6;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            7 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word7 = gps::GpsSubframe1Word7::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word7 = gps::GpsSubframe2Word7::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word7 = gps::GpsSubframe3Word7::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word7 = word7;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            8 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word8 = gps::GpsSubframe1Word8::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word8 = gps::GpsSubframe2Word8::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word8 = gps::GpsSubframe3Word8::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word8 = word8;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            9 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word9 = gps::GpsSubframe1Word9::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word9 = gps::GpsSubframe2Word9::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word9 = gps::GpsSubframe3Word9::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word9 = word9;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            10 => {
                // Frame dependent continuation
                if let Some(interprated) = interprated {
                    let frame_id = interprated.how.frame_id;
                    match frame_id {
                        1 => {
                            let word10 = gps::GpsSubframe1Word10::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe1(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        2 => {
                            let word10 = gps::GpsSubframe2Word10::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe2(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        3 => {
                            let word10 = gps::GpsSubframe3Word10::decode(dword);
                            match &mut interprated.subframe {
                                gps::GpsSubframe::Subframe3(subframe) => {
                                    subframe.word10 = word10;
                                },
                                _ => {}, // not applicable
                            }
                        },
                        _ => {}, // not supported yet
                    }
                }
            },
            _ => {}, // not applicable
        }

        Some(())
    }
}

impl RxmSfrbxRef<'_> {
    pub fn interprate(&self) -> Option<RxmSfrbxInterprated> {
        self.interprator().interprate()
    }

    fn interprator(&self) -> RxmSfrbxInterprator<'_> {
        RxmSfrbxInterprator {
            ptr: 0,
            iter: self.dwrd(),
            gnss_id: self.gnss_id(),
        }
    }
}
