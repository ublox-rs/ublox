use rustdds::Keyed;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct NavPvt {
    pub key: String,
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub nanosec: i32,
    pub utc_time_accuracy: u32,

    pub lat: f32,
    pub lon: f32,
    pub height: f32,
    pub msl: f32,
    pub horizontal_accuracy: f32,
    pub vertical_accuracy: f32,

    pub vel_n: f32,
    pub vel_e: f32,
    pub vel_d: f32,
    pub speed_over_ground: f32,
    pub velocity_accuracy: f32,

    pub heading_motion: f32,
    pub heading_vehicle: f32,
    pub heading_accuracy: f32,

    pub magnetic_declination: f32,
    pub magnetic_declination_accuracy: f32,

    pub pdop: f32,
    pub satellites_used: u8,

    pub gps_fix_type: u8,
    pub fix_flags: u8,
    pub llh_validity: bool,
    pub time_confirmation_flags: u8,
}

impl NavPvt {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for NavPvt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Keyed for NavPvt {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
