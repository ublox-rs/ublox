//! MON-SPAN: Spectrum Analyzer
//!
//! Reports signal characteristics and spectrum data for each RF path.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Size of each RF block in bytes
const RF_BLOCK_SIZE: usize = 272;
/// Size of spectrum data array
const SPECTRUM_SIZE: usize = 256;

/// Spectrum Analyzer
///
/// Reports signal characteristics and spectrum data for each RF path.
/// Each RF block contains a 256-point spectrum array and metadata.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x31, max_payload_len = 548)] // 4 + 2*272
struct MonSpan {
    /// Message version (0x00 for this version)
    version: u8,

    /// Number of RF blocks included
    num_rf_blocks: u8,

    /// Reserved
    reserved0: [u8; 2],

    /// RF blocks (repeated num_rf_blocks times, 272 bytes each)
    #[ubx(map_type = MonSpanRfBlockIter, may_fail,
          from = MonSpanRfBlockIter::new,
          is_valid = MonSpanRfBlockIter::is_valid)]
    rf_blocks: [u8; 0],
}

/// Information about a single RF block in MON-SPAN
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonSpanRfBlock<'a> {
    /// Spectrum data (256 points, 0.25 dB resolution)
    spectrum: &'a [u8; SPECTRUM_SIZE],
    /// Spectrum span in Hz
    pub span: u32,
    /// Spectrum resolution in Hz
    pub res: u32,
    /// Center frequency in Hz
    pub center: u32,
    /// Programmable gain amplifier setting in dB
    pub pga: u8,
}

impl<'a> MonSpanRfBlock<'a> {
    /// Returns the raw spectrum data (256 bytes).
    /// Values are in units of 0.25 dB (scale factor 2^-2).
    pub fn spectrum_raw(&self) -> &[u8; SPECTRUM_SIZE] {
        self.spectrum
    }

    /// Returns the spectrum value at the given index in dB.
    /// Index must be less than 256.
    pub fn spectrum_db(&self, index: usize) -> Option<f32> {
        self.spectrum.get(index).map(|&v| v as f32 * 0.25)
    }

    /// Returns an iterator over spectrum values in dB.
    pub fn spectrum_db_iter(&self) -> impl Iterator<Item = f32> + 'a {
        self.spectrum.iter().map(|&v| v as f32 * 0.25)
    }

    /// Returns the number of valid spectrum points.
    /// Calculated as span / res when res > 0.
    pub fn num_points(&self) -> Option<u32> {
        if self.res > 0 {
            Some(self.span / self.res)
        } else {
            None
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MonSpanRfBlock<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MonSpanRfBlock", 5)?;
        state.serialize_field("spectrum", self.spectrum.as_slice())?;
        state.serialize_field("span", &self.span)?;
        state.serialize_field("res", &self.res)?;
        state.serialize_field("center", &self.center)?;
        state.serialize_field("pga", &self.pga)?;
        state.end()
    }
}

/// Iterator for MON-SPAN RF blocks
#[derive(Debug, Clone)]
pub struct MonSpanRfBlockIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MonSpanRfBlockIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    #[allow(dead_code, reason = "Used by ubx_packet_recv macro for validation")]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % RF_BLOCK_SIZE == 0
    }
}

impl<'a> core::iter::Iterator for MonSpanRfBlockIter<'a> {
    type Item = MonSpanRfBlock<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + RF_BLOCK_SIZE)?;

        let spectrum: &[u8; SPECTRUM_SIZE] = chunk[0..SPECTRUM_SIZE].try_into().ok()?;
        let block = MonSpanRfBlock {
            spectrum,
            span: u32::from_le_bytes(chunk[256..260].try_into().ok()?),
            res: u32::from_le_bytes(chunk[260..264].try_into().ok()?),
            center: u32::from_le_bytes(chunk[264..268].try_into().ok()?),
            pga: chunk[268],
            // bytes 269..272 are reserved1
        };

        self.offset += RF_BLOCK_SIZE;
        Some(block)
    }
}
