#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

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

#[cfg(feature = "sfrbx-gps")]
use gnss_protos::{GpsDataByte, GpsQzssDecoder, GpsQzssFrame};

#[derive(Debug, Clone)]
pub enum RxmSfrbxInterpreted {
    /// [GpsQzssFrame]
    #[cfg(feature = "sfrbx-gps")]
    GpsQzss(GpsQzssFrame),
}

impl RxmSfrbxRef<'_> {
    /// Try to interpret the RXM-SFRBX inner frame (when supported/known).
    pub fn interpret(&self) -> Option<RxmSfrbxInterpreted> {
        // interpretation is GNSS dependent
        match self.gnss_id() {
            0 | 5 => {
                #[cfg(feature = "sfrbx-gps")]
                {
                    let decoded = self.gps_qzss_decoding()?;
                    Some(RxmSfrbxInterpreted::GpsQzss(decoded))
                }
                #[cfg(not(feature = "sfrbx-gps"))]
                {
                    None
                }
            },
            _ => {
                // either not supported or not applicable
                None
            },
        }
    }

    #[cfg(feature = "sfrbx-gps")]
    /// [GpsQzssFrame] interpretation attempt, from parsed UBX-SFRBX data.
    fn gps_qzss_decoding(&self) -> Option<GpsQzssFrame> {
        let mut decoder = GpsQzssDecoder::default().without_parity_verification();

        for byte in self.dwrd() {
            let bytes = byte.to_be_bytes();

            let gps_bytes = [
                GpsDataByte::MsbPadded(bytes[0]),
                GpsDataByte::Byte(bytes[1]),
                GpsDataByte::Byte(bytes[2]),
                GpsDataByte::Byte(bytes[3]),
            ];

            for gps_byte in gps_bytes {
                if let Some(decoded) = decoder.parse(gps_byte) {
                    return Some(decoded);
                }
            }
        }

        None
    }
}
