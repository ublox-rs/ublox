#![cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31"
))]
use core::fmt;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use num_traits::float::FloatCore;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::ubx_packets::packets::esf_status::EsfSensorType;
use crate::{error::ParserError, ubx_checksum, UbxPacketMeta};
use ublox_derive::ubx_packet_recv_send;

const DATA_BITMASK: u32 = 0x7FFFFF;
const DIRECTION_INDICATOR_BIT: usize = 23;
const SIGN_BIT: u32 = 0x800000;

/// External Sensor Fusion (ESF) Measurement Data
///
/// This message contains external sensor measurements for dead reckoning applications.
/// It supports various sensor types including accelerometers, gyroscopes, wheel tick sensors,
/// and speed sensors.
///
/// # Time Marking
///
/// The receiver can use two sensor timestamping approaches:
/// - **First Byte Reception** (default): Uses reception time of first byte
/// - **External Time Mark**: Uses time mark signal on external input pin
///
/// See EsfMeasFlags and the uBlox specification specific to your device on how to use the time mark signal.
///
/// # Example
///
/// ```rust
/// use ublox::esf_meas::{EsfMeas, EsfMeasBuilder, EsfMeasData, EsfMeasFlagsBuilder};
/// use ublox::esf_status::EsfSensorType;
///
/// // Create sensor measurement data
/// let sensor_data = vec![
///     EsfMeasData {
///         data_type: EsfSensorType::AccX,
///         data_field: 1000, // Acceleration value
///     },
///     EsfMeasData {
///         data_type: EsfSensorType::GyroZ,
///         data_field: -500, // Gyroscope value
///     },
/// ];
///
/// // Build ESF measurement packet
/// let mut esf_packet = EsfMeasBuilder::default()
///     .with_measurement_data(&sensor_data)
///     .with_calib_tag(Some(0x12345678)); // Calibration tag (optional)
///
/// esf_packet.id = 1; // Identifier of data provider  
/// esf_packet.itow = 123456; // Time of week in milliseconds
///
/// // Convert to bytes for transmission
/// let mut buffer = Vec::new();
/// esf_packet.extend_to(&mut buffer);
/// ```
#[ubx_packet_recv_send]
#[ubx(
    class = 0x10,
    id = 0x02,
    max_payload_len = 1240,
    flags = "default_for_builder"
)]
struct EsfMeas<'a> {
    itow: u32,
    #[ubx(map_type = EsfMeasFlags, from = EsfMeasFlags, into = EsfMeasFlags::into_raw)]
    flags: u16,
    id: u16,
    #[ubx(
        map_type = EsfMeasDataIter<'a>,
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
        into = EsfMeas::calib_tag_into_bytes,
    )]
    calib_tag: [u8; 0],
}

impl EsfMeas {
    fn calib_tag(bytes: &[u8]) -> Option<u32> {
        bytes.try_into().ok().map(u32::from_le_bytes)
    }

    fn calib_tag_into_bytes(x: Option<u32>) -> CalibTagBytes {
        match x {
            Some(v) => CalibTagBytes {
                buf: v.to_le_bytes(),
                len: 4,
            },
            None => CalibTagBytes {
                buf: [0; 4],
                len: 0,
            },
        }
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

#[derive(Clone)]
pub enum EsfMeasDataIter<'a> {
    Bytes(core::slice::ChunksExact<'a, u8>),
    Slice(core::slice::Iter<'a, EsfMeasData>),
}

impl<'a> EsfMeasDataIter<'a> {
    const BLOCK_SIZE: usize = 4;
    fn new(bytes: &'a [u8]) -> Self {
        Self::Bytes(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn from_slice(data: &'a [EsfMeasData]) -> Self {
        Self::Slice(data.iter())
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::fmt::Debug for EsfMeasDataIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        let it = self.clone();
        for item in it {
            list.entry(&item);
        }
        list.finish()
    }
}

impl core::iter::Iterator for EsfMeasDataIter<'_> {
    type Item = EsfMeasData;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Bytes(iter) => {
                let chunk = iter.next()?;
                let data = u32::from_le_bytes(chunk[0..Self::BLOCK_SIZE].try_into().ok()?);
                let mut data_field = (data & DATA_BITMASK) as i32;
                let backward = ((data >> DIRECTION_INDICATOR_BIT) & 0x01) == 1;
                // Turn value into valid negative integer representation
                // This is to handle the case where the data field is negative
                // and retrieve the correct value as the two's complement
                if backward {
                    data_field ^= SIGN_BIT as i32;
                    data_field = data_field.wrapping_neg();
                }

                Some(EsfMeasData {
                    data_type: (((data >> 24) & 0x3F) as u8).into(),
                    data_field,
                })
            },
            Self::Slice(iter) => iter.next().cloned(),
        }
    }
}

impl Default for EsfMeasDataIter<'_> {
    fn default() -> Self {
        Self::from_slice(&[])
    }
}

#[derive(Debug, Clone)]
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
    const TYPE_MASK_6: u32 = 0x3F;
    const TYPE_SHIFT: u32 = 24;

    #[inline]
    fn encode_magnitude_and_sign(value: i32) -> u32 {
        let magnitude = (value as u32) & DATA_BITMASK;
        if value.is_negative() {
            magnitude ^ SIGN_BIT
        } else {
            magnitude
        }
    }

    #[inline]
    fn encode_type_bits(t: EsfSensorType) -> u32 {
        (((t as u8) as u32) & Self::TYPE_MASK_6) << Self::TYPE_SHIFT
    }

    #[inline]
    fn encode_data_block(&self) -> u32 {
        Self::encode_magnitude_and_sign(self.data_field) | Self::encode_type_bits(self.data_type)
    }

    pub fn extend_to<T>(&self, out: &mut T) -> usize
    where
        T: core::iter::Extend<u8>,
    {
        let bytes = self.encode_data_block().to_le_bytes();
        out.extend(bytes);
        bytes.len()
    }

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
                let tick = (self.data_field & DATA_BITMASK as i32) * (self.direction() as i32);
                SensorData::Tick(tick)
            },
            EsfSensorType::Speed => {
                let value = (self.data_field & DATA_BITMASK as i32) as f32
                    * (self.direction() as f32)
                    * 1e-3;
                SensorData::Value(value)
            },
            EsfSensorType::GyroX | EsfSensorType::GyroY | EsfSensorType::GyroZ => {
                let value = (self.data_field & DATA_BITMASK as i32) as f32 * 2_f32.powi(-12);
                SensorData::Value(value)
            },
            EsfSensorType::AccX | EsfSensorType::AccY | EsfSensorType::AccZ => {
                let value = (self.data_field & DATA_BITMASK as i32) as f32 * 2_f32.powi(-10);
                SensorData::Value(value)
            },
            EsfSensorType::GyroTemp => {
                let value = (self.data_field & DATA_BITMASK as i32) as f32 * 1e-2;
                SensorData::Value(value)
            },
            _ => SensorData::Value(0f32),
        }
    }
}

/// UBX-ESF-MEAS flags
#[repr(transparent)]
#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfMeasFlags(u16);

impl EsfMeasFlags {
    const NUM_MEAS_SHIFT: u16 = 11;
    const NUM_MEAS_MASK_5: u16 = 0x1F;
    const CALIB_TAG_VALID_BIT: u16 = 1 << 3;

    pub const fn into_raw(self) -> u16 {
        self.0
    }
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

    pub fn enable_calibration_tag(self, valid: bool) -> Self {
        if valid {
            Self(self.0 | Self::CALIB_TAG_VALID_BIT)
        } else {
            Self(self.0 & !Self::CALIB_TAG_VALID_BIT)
        }
    }

    fn set_measurements_count(self, n: u8) -> Self {
        let new_meas_count = (n as u16) & Self::NUM_MEAS_MASK_5;
        let clear_meas_count = self.0 & !(Self::NUM_MEAS_MASK_5 << Self::NUM_MEAS_SHIFT);
        Self(clear_meas_count | (new_meas_count << Self::NUM_MEAS_SHIFT))
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

#[derive(Debug, Clone)]
struct CalibTagBytes {
    buf: [u8; 4],
    len: usize,
}

impl AsRef<[u8]> for CalibTagBytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// Builder for EsfMeasFlags
pub struct EsfMeasFlagsBuilder {
    time_mark_sent: u8,
    time_mark_edge: bool,
    calib_tag_valid: bool,
    num_meas: u8,
}

impl EsfMeasFlagsBuilder {
    pub fn new() -> Self {
        Self {
            time_mark_sent: 0,
            time_mark_edge: false,
            calib_tag_valid: false,
            num_meas: 0,
        }
    }

    pub fn time_mark_sent(mut self, sent: u8) -> Self {
        self.time_mark_sent = sent & 0x3; // Ensure only 2 bits
        self
    }

    pub fn time_mark_edge(mut self, edge: bool) -> Self {
        self.time_mark_edge = edge;
        self
    }

    pub fn calib_tag_valid(mut self, valid: bool) -> Self {
        self.calib_tag_valid = valid;
        self
    }

    pub fn num_meas(mut self, num: u8) -> Self {
        self.num_meas = num & 0x1F; // Ensure only 5 bits
        self
    }

    pub fn build(self) -> EsfMeasFlags {
        let mut flags = 0u16;
        flags |= (self.time_mark_sent as u16) << 1; // bits 1-2
        flags |= (self.time_mark_edge as u16) << 2; // bit 2
        flags |= (self.calib_tag_valid as u16) << 3; // bit 3
        flags |= (self.num_meas as u16) << 11; // bits 11-15
        EsfMeasFlags(flags)
    }
}

impl Default for EsfMeasFlagsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for the send-side builder
impl<'a> EsfMeasBuilder<'a> {
    pub fn with_measurement_data(mut self, data: &'a [EsfMeasData]) -> Self {
        self.data = EsfMeasDataIter::from_slice(data);
        self.flags = self.flags.set_measurements_count(data.len() as u8);
        self
    }

    pub fn with_calib_tag(mut self, tag: Option<u32>) -> Self {
        self.calib_tag = tag;
        self.flags = self.flags.enable_calibration_tag(tag.is_some());
        self
    }
}
