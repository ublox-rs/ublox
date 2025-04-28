#![cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31"
))]
use bitflags::bitflags;
use core::fmt;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv};

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x10, max_payload_len = 1240)]
struct EsfStatus {
    itow: u32,
    /// Version is 2 for M8L spec
    version: u8,

    #[ubx(map_type = EsfInitStatus1, from = EsfInitStatus1)]
    init_status1: u8,

    #[ubx(map_type = EsfInitStatus2, from = EsfInitStatus2)]
    init_status2: u8,

    reserved1: [u8; 5],

    #[ubx(map_type = EsfStatusFusionMode)]
    fusion_mode: u8,

    reserved2: [u8; 2],

    num_sens: u8,

    #[ubx(
        map_type = EsfSensorStatusIter,
        from = EsfSensorStatusIter::new,
        is_valid = EsfSensorStatusIter::is_valid,
        may_fail,
    )]
    data: [u8; 0],
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EsfStatusFusionMode {
    Initializing = 0,
    Fusion = 1,
    Suspended = 2,
    Disabled = 3,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfInitStatus1(u8);

impl EsfInitStatus1 {
    const WHEEL_TICK_MASK: u8 = 0x03;
    const MOUNTING_ANGLE_STATUS_MASK: u8 = 0x07;
    const INS_STATUS_MASK: u8 = 0x03;

    pub fn wheel_tick_init_status(self) -> EsfStatusWheelTickInit {
        let bits = (self.0) & Self::WHEEL_TICK_MASK;
        match bits {
            0 => EsfStatusWheelTickInit::Off,
            1 => EsfStatusWheelTickInit::Initializing,
            2 => EsfStatusWheelTickInit::Initialized,
            _ => EsfStatusWheelTickInit::Invalid,
        }
    }

    pub fn wheel_tick_init_status_raw(self) -> u8 {
        self.wheel_tick_init_status() as u8
    }

    pub fn mounting_angle_status(self) -> EsfStatusMountAngle {
        let bits = (self.0 >> 2) & Self::MOUNTING_ANGLE_STATUS_MASK;
        match bits {
            0 => EsfStatusMountAngle::Off,
            1 => EsfStatusMountAngle::Initializing,
            2 | 3 => EsfStatusMountAngle::Initialized,
            _ => EsfStatusMountAngle::Invalid,
        }
    }

    pub fn mount_angle_status_raw(self) -> u8 {
        self.mounting_angle_status() as u8
    }

    pub fn ins_initialization_status(self) -> EsfStatusInsInit {
        let bits = (self.0 >> 5) & Self::INS_STATUS_MASK;
        match bits {
            0 => EsfStatusInsInit::Off,
            1 => EsfStatusInsInit::Initializing,
            2 => EsfStatusInsInit::Initialized,
            _ => EsfStatusInsInit::Invalid,
        }
    }

    pub fn ins_initialization_status_raw(self) -> u8 {
        self.ins_initialization_status() as u8
    }
}

impl fmt::Debug for EsfInitStatus1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("initStatus1")
            .field("wtInitStatus", &self.wheel_tick_init_status())
            .field("mntAlgStatus", &self.mounting_angle_status())
            .field("insInitStatus", &self.ins_initialization_status())
            .finish()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusWheelTickInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusMountAngle {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusInsInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfInitStatus2(u8);

impl EsfInitStatus2 {
    pub fn imu_init_status_raw(self) -> u8 {
        self.imu_init_status() as u8
    }

    pub fn imu_init_status(self) -> EsfStatusImuInit {
        let bits = (self.0) & 0x02;
        match bits {
            0 => EsfStatusImuInit::Off,
            1 => EsfStatusImuInit::Initializing,
            2 => EsfStatusImuInit::Initialized,
            _ => EsfStatusImuInit::Invalid,
        }
    }
}

impl fmt::Debug for EsfInitStatus2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("initStatus2")
            .field("imuInitStatus", &self.imu_init_status())
            .finish()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfStatusImuInit {
    Off = 0,
    Initializing = 1,
    Initialized = 2,
    /// Only two bits are reserved for the init status
    Invalid = 3,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfSensorStatus {
    sens_status1: SensorStatus1,
    sens_status2: SensorStatus2,
    freq: u16,
    faults: EsfSensorFaults,
}

impl EsfSensorStatus {
    pub fn freq(&self) -> u16 {
        self.freq
    }

    pub fn faults(&self) -> EsfSensorFaults {
        self.faults
    }

    pub fn faults_raw(&self) -> u8 {
        self.faults().into_raw()
    }

    pub fn sensor_type(&self) -> EsfSensorType {
        self.sens_status1.sensor_type
    }

    pub fn sensor_type_raw(&self) -> u8 {
        self.sensor_type() as u8
    }

    pub fn sensor_used(&self) -> bool {
        self.sens_status1.used
    }

    pub fn sensor_ready(&self) -> bool {
        self.sens_status1.ready
    }

    pub fn calibration_status(&self) -> EsfSensorStatusCalibration {
        self.sens_status2.calibration_status
    }

    pub fn calibration_status_raw(&self) -> u8 {
        self.calibration_status() as u8
    }

    pub fn time_status(&self) -> EsfSensorStatusTime {
        self.sens_status2.time_status
    }

    pub fn time_status_raw(&self) -> u8 {
        self.time_status() as u8
    }
}

#[derive(Clone, Debug)]
pub struct EsfSensorStatusIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> EsfSensorStatusIter<'a> {
    const BLOCK_SIZE: usize = 4;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &[u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for EsfSensorStatusIter<'_> {
    type Item = EsfSensorStatus;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..Self::BLOCK_SIZE].try_into().unwrap());
        Some(EsfSensorStatus {
            sens_status1: ((data & 0xFF) as u8).into(),
            sens_status2: (((data >> 8) & 0xFF) as u8).into(),
            freq: ((data >> 16) & 0xFF).try_into().unwrap(),
            faults: (((data >> 24) & 0xFF) as u8).into(),
        })
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SensorStatus1 {
    sensor_type: EsfSensorType,
    used: bool,
    ready: bool,
}

impl From<u8> for SensorStatus1 {
    fn from(s: u8) -> Self {
        let sensor_type: EsfSensorType = (s & 0x3F).into();
        Self {
            sensor_type,
            used: (s >> 6) != 0,
            ready: (s >> 7) != 0,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorType {
    None = 0,
    /// Angular acceleration in [deg/s]
    GyroZ = 5,
    /// Unitless (counter)
    FrontLeftWheelTicks = 6,
    /// Unitless (counter)
    FrontRightWheelTicks = 7,
    /// Unitless (counter)
    RearLeftWheelTicks = 8,
    /// Unitless (counter)
    RearRightWheelTicks = 9,
    /// Unitless (counter)
    SpeedTick = 10,
    /// Speed in [m/s]
    Speed = 11,
    /// Temperature Celsius \[deg\]
    GyroTemp = 12,
    /// Angular acceleration in [deg/s]
    GyroY = 13,
    /// Angular acceleration in [deg/s]
    GyroX = 14,
    /// Specific force in [m/s^2]
    AccX = 16,
    /// Specific force in [m/s^2]
    AccY = 17,
    /// Specific force in [m/s^2]
    AccZ = 18,
    Invalid = 19,
}

impl From<u8> for EsfSensorType {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorType::None,
            5 => EsfSensorType::GyroZ,
            6 => EsfSensorType::FrontLeftWheelTicks,
            7 => EsfSensorType::FrontRightWheelTicks,
            8 => EsfSensorType::RearLeftWheelTicks,
            9 => EsfSensorType::RearRightWheelTicks,
            10 => EsfSensorType::SpeedTick,
            11 => EsfSensorType::Speed,
            12 => EsfSensorType::GyroTemp,
            13 => EsfSensorType::GyroY,
            14 => EsfSensorType::GyroX,
            16 => EsfSensorType::AccX,
            17 => EsfSensorType::AccY,
            18 => EsfSensorType::AccZ,
            _ => EsfSensorType::Invalid,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SensorStatus2 {
    pub(crate) calibration_status: EsfSensorStatusCalibration,
    pub(crate) time_status: EsfSensorStatusTime,
}

impl From<u8> for SensorStatus2 {
    fn from(s: u8) -> Self {
        let calibration_status: EsfSensorStatusCalibration = (s & 0x03).into();
        let time_status: EsfSensorStatusTime = ((s >> 2) & 0x03).into();
        Self {
            calibration_status,
            time_status,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorStatusCalibration {
    NotCalibrated = 0,
    Calibrating = 1,
    Calibrated = 2,
    Invalid = 4,
}

impl From<u8> for EsfSensorStatusCalibration {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorStatusCalibration::NotCalibrated,
            1 => EsfSensorStatusCalibration::Calibrating,
            2 | 3 => EsfSensorStatusCalibration::Calibrated,
            _ => EsfSensorStatusCalibration::Invalid,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EsfSensorStatusTime {
    NoData = 0,
    OnReceptionFirstByte = 1,
    OnEventInput = 2,
    TimeTagFromData = 3,
    Invalid = 4,
}

impl From<u8> for EsfSensorStatusTime {
    fn from(orig: u8) -> Self {
        match orig {
            0 => EsfSensorStatusTime::NoData,
            1 => EsfSensorStatusTime::OnReceptionFirstByte,
            2 => EsfSensorStatusTime::OnEventInput,
            3 => EsfSensorStatusTime::TimeTagFromData,
            _ => EsfSensorStatusTime::Invalid,
        }
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
#[derive(Debug, Default, Clone, Copy)]
pub struct EsfSensorFaults: u8 {
    const BAD_MEASUREMENT = 1;
    const BAD_TIME_TAG = 2;
    const MISSING_MEASUREMENT = 4;
    const NOISY_MEASUREMENT = 8;
}
}

impl From<u8> for EsfSensorFaults {
    fn from(s: u8) -> Self {
        Self::from_bits(s).unwrap_or(EsfSensorFaults::empty())
    }
}
