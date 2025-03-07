#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::cfg_val::CfgVal;

use bitflags::bitflags;

use super::SerializeUbxPacketFields;

use ublox_derive::{
    ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_recv_send, ubx_packet_send,
};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    nav::NavBbrMask, ubx_checksum, MemWriter, ScaleBack, UbxChecksumCalc, UbxPacketCreator,
    UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x02, max_payload_len = 1240)]
pub struct EsfMeas {
    pub time_tag: u32,
    pub flags: u16,
    pub id: u16,
    #[ubx(
        map_type = EsfMeasDataIter,
        from = EsfMeasDataIter::new,
        size_fn = data_len,
        is_valid = EsfMeasDataIter::is_valid,
        may_fail,
    )]
    pub data: [u8; 0],
    #[ubx(
        map_type = Option<u32>,
        from = EsfMeas::calib_tag,
        size_fn = calib_tag_len,
    )]
    pub calib_tag: [u8; 0],
}

impl EsfMeas {
    fn calib_tag(bytes: &[u8]) -> Option<u32> {
        bytes.try_into().ok().map(u32::from_le_bytes)
    }
}

impl<'a> EsfMeasRef<'a> {
    fn data_len(&self) -> usize {
        ((self.flags() >> 11 & 0x1f) as usize) * 4
    }

    fn calib_tag_len(&self) -> usize {
        if self.flags() & 0x8 != 0 {
            4
        } else {
            0
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasData {
    pub data_type: u8,
    pub data_field: u32,
}

#[derive(Debug, Clone)]
pub struct EsfMeasDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfMeasDataIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(4))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 4 == 0
    }
}

impl<'a> core::iter::Iterator for EsfMeasDataIter<'a> {
    type Item = EsfMeasData;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.0.next()?.try_into().map(u32::from_le_bytes).unwrap();
        Some(EsfMeasData {
            data_type: ((data & 0x3F000000) >> 24).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
        })
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x03, max_payload_len = 1240)]
pub struct EsfRaw {
    pub msss: u32,
    #[ubx(
        map_type = EsfRawDataIter,
        from = EsfRawDataIter::new,
        is_valid = EsfRawDataIter::is_valid,
        may_fail,
    )]
    pub data: [u8; 0],
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
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(8))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % 8 == 0
    }
}

impl<'a> core::iter::Iterator for EsfRawDataIter<'a> {
    type Item = EsfRawData;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
        let sensor_time_tag = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        Some(EsfRawData {
            data_type: ((data >> 24) & 0xFF).try_into().unwrap(),
            data_field: data & 0xFFFFFF,
            sensor_time_tag,
        })
    }
}

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x15, fixed_payload_len = 36)]
pub struct EsfIns {
    #[ubx(map_type = EsfInsBitFlags)]
    pub bit_field: u32,
    pub reserved: [u8; 4],
    pub itow: u32,

    #[ubx(map_type = f64, scale = 1e-3, alias = x_angular_rate)]
    pub x_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = y_angular_rate)]
    pub y_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-3, alias = z_angular_rate)]
    pub z_ang_rate: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = x_acceleration)]
    pub x_accel: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = y_acceleration)]
    pub y_accel: i32,

    #[ubx(map_type = f64, scale = 1e-2, alias = z_acceleration)]
    pub z_accel: i32,
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct EsfInsBitFlags: u32 {
        const VERSION = 1;
        const X_ANG_RATE_VALID = 0x100;
        const Y_ANG_RATE_VALID = 0x200;
        const Z_ANG_RATE_VALID = 0x400;
        const X_ACCEL_VALID = 0x800;
        const Y_ACCEL_VALID = 0x1000;
        const Z_ACCEL_VALID = 0x2000;
    }
}
