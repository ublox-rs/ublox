use rustdds::Keyed;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HnrPvt {
    pub key: String,
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub nanosec: i32,
    pub gps_fix: u8,
    pub flags: u8,
    pub lon: f64,
    pub lat: f64,
    pub height: f64,
    pub height_msl: f64,
    pub ground_speed: f64,
    pub speed: f64,
    pub head_motion: f64,
    pub head_vehicle: f64,
    pub horizontal_accuracy: f64,
    pub vertical_accuracy: f64,
    pub speed_accuracy: f64,
    pub head_acc: f64,
}

impl fmt::Display for HnrPvt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: HnrPvt, @key: '{}'\n
            itow:{}, year: {}, month: {}, day: {},
            hour: {}, min: {}, sec: {},
            valid: {}, nanosec: {}, gps_fix: {},
            flags: {}, lon: {}, lat: {},
            height: {}, height_msl: {}, g_speed: {},
            speed: {}, head_mot: {}, head_veh: {},
            h_acc: {}, v_acc: {}, s_acc: {}, head_acc: {}",
            self.key,
            self.itow,
            self.year,
            self.month,
            self.day,
            self.hour,
            self.min,
            self.sec,
            self.valid,
            self.nanosec,
            self.gps_fix,
            self.flags,
            self.lon,
            self.lat,
            self.height,
            self.height_msl,
            self.ground_speed,
            self.speed,
            self.head_motion,
            self.head_vehicle,
            self.horizontal_accuracy,
            self.vertical_accuracy,
            self.speed_accuracy,
            self.head_acc
        )
    }
}

impl Keyed for HnrPvt {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
