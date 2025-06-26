#![cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31"
))]
use core::fmt;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use num_traits::float::FloatCore;

#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, EsfSensorType, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x02, max_payload_len = 1240)]
struct EsfMeas {
    itow: u32,
    #[ubx(map_type = EsfMeasFlags, from = EsfMeasFlags)]
    flags: u16,
    id: u16,
    #[ubx(
        map_type = EsfMeasDataIter,
        from = EsfMeasDataIter::new,
        size_fn = data_len,
        is_valid = EsfMeasDataIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
    #[ubx(
        map_type = Option<u32>,
        from = EsfMeas::calib_tag,
        size_fn = calib_tag_len,
    )]
    calib_tag: [u8; 0],
}

impl EsfMeas {
    fn calib_tag(bytes: &[u8]) -> Option<u32> {
        bytes.try_into().ok().map(u32::from_le_bytes)
    }
}

impl EsfMeasRef<'_> {
    fn data_len(&self) -> usize {
        self.flags().num_meas() as usize * 4
    }

    fn calib_tag_len(&self) -> usize {
        if self.flags().calib_tag_valid() {
            4
        } else {
            0
        }
    }
}

impl EsfMeasOwned {
    fn data_len(&self) -> usize {
        self.flags().num_meas() as usize * 4
    }

    fn calib_tag_len(&self) -> usize {
        if self.flags().calib_tag_valid() {
            4
        } else {
            0
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasData {
    pub data_type: EsfSensorType,
    pub data_field: i32,
}

#[derive(Debug)]
pub enum SensorData {
    Tick(i32),
    Value(f32),
}

impl EsfMeasData {
    pub fn direction(&self) -> i8 {
        if self.data_field.is_negative() {
            -1
        } else {
            1
        }
    }

    pub fn value(&self) -> SensorData {
        match self.data_type {
            EsfSensorType::FrontLeftWheelTicks
            | EsfSensorType::FrontRightWheelTicks
            | EsfSensorType::RearLeftWheelTicks
            | EsfSensorType::RearRightWheelTicks
            | EsfSensorType::SpeedTick => {
                let tick = (self.data_field & 0x7FFFFF) * (self.direction() as i32);
                SensorData::Tick(tick)
            },
            EsfSensorType::Speed => {
                let value = (self.data_field & 0x7FFFFF) as f32 * (self.direction() as f32) * 1e-3;
                SensorData::Value(value)
            },
            EsfSensorType::GyroX | EsfSensorType::GyroY | EsfSensorType::GyroZ => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 2_f32.powi(-12);
                SensorData::Value(value)
            },
            EsfSensorType::AccX | EsfSensorType::AccY | EsfSensorType::AccZ => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 2_f32.powi(-10);
                SensorData::Value(value)
            },
            EsfSensorType::GyroTemp => {
                let value = (self.data_field & 0x7FFFFF) as f32 * 1e-2;
                SensorData::Value(value)
            },
            _ => SensorData::Value(0f32),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EsfMeasDataIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfMeasDataIter<'a> {
    const BLOCK_SIZE: usize = 4;
    const DIRECTION_INDICATOR_BIT: usize = 23;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfMeasDataIter<'_> {
    type Item = EsfMeasData;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..Self::BLOCK_SIZE].try_into().unwrap());
        let mut data_field = (data & 0x7FFFFF) as i32;
        let backward = ((data >> Self::DIRECTION_INDICATOR_BIT) & 0x01) == 1;
        // Turn value into valid negative integer representation
        if backward {
            data_field ^= 0x800000;
            data_field = data_field.wrapping_neg();
        }

        Some(EsfMeasData {
            data_type: (((data >> 24) & 0x3F) as u8).into(),
            data_field,
        })
    }
}

/// UBX-ESF-MEAS flags
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasFlags(u16);

impl EsfMeasFlags {
    pub fn time_mark_sent(self) -> u8 {
        ((self.0) & 0x2) as u8
    }

    pub fn time_mark_edge(self) -> bool {
        (self.0 >> 2) & 0x01 != 0
    }

    pub fn calib_tag_valid(self) -> bool {
        (self.0 >> 3) & 0x01 != 0
    }

    pub fn num_meas(self) -> u8 {
        ((self.0 >> 11) & 0x1F) as u8
    }
}

impl fmt::Debug for EsfMeasFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("timeMarkSent", &self.time_mark_sent())
            .field("timeMarkEdge", &self.time_mark_edge())
            .field("calibTagValid", &self.calib_tag_valid())
            .field("numMeas", &self.num_meas())
            .finish()
    }
}
