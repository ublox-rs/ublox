use rustdds::Keyed;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct EsfStatus {
    pub key: String,
    pub itow: u32,
    pub fusion_status: EsfFusionStatus,
    pub sensors: Vec<EsfSensorStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EsfFusionStatus {
    pub fusion_mode: u8,
    pub imu_status: u8,
    pub wheel_tick_sensor_status: u8,
    pub ins_status: u8,
    pub imu_mount_alignment_status: u8,
}

impl Default for EsfFusionStatus {
    fn default() -> Self {
        Self {
            fusion_mode: u8::MAX,
            imu_status: u8::MAX,
            wheel_tick_sensor_status: u8::MAX,
            ins_status: u8::MAX,
            imu_mount_alignment_status: u8::MAX,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EsfSensorStatus {
    pub sensor_type: u8,
    pub calibration_status: u8,
    pub timing_status: u8,
    pub freq: u16,
    pub faults: u8,
}

impl Default for EsfSensorStatus {
    fn default() -> Self {
        Self {
            sensor_type: u8::MAX,
            calibration_status: u8::MAX,
            timing_status: u8::MAX,
            freq: 0,
            faults: u8::MAX,
        }
    }
}

impl EsfStatus {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for EsfStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Keyed for EsfStatus {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
