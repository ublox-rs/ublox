#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x03, max_payload_len = 1240)]
struct EsfRaw {
    msss: u32,
    #[ubx(
        map_type = EsfRawDataIter,
        from = EsfRawDataIter::new,
        is_valid = EsfRawDataIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfRawData {
    pub data_type: u8,
    pub data_field: u32,
    pub sensor_time_tag: u32,
}

#[derive(Debug, Clone)]
pub struct EsfRawDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfRawDataIter<'a> {
    const BLOCK_SIZE: usize = 8;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfRawDataIter<'_> {
    type Item = EsfRawData;

    fn next(&mut self) -> Option<Self::Item> {
        const HALF_BLOCK: usize = 4;
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..HALF_BLOCK].try_into().unwrap());
        let sensor_time_tag =
            u32::from_le_bytes(chunk[HALF_BLOCK..Self::BLOCK_SIZE].try_into().unwrap());
        Some(EsfRawData {
            data_type: ((data >> 24) & 0xFF).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
            sensor_time_tag,
        })
    }
}
