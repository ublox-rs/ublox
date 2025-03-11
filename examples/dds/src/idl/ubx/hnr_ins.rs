use rustdds::Keyed;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Ins {
    pub x_ang_rate: f64,
    pub y_ang_rate: f64,
    pub z_ang_rate: f64,
    pub x_accel: f64,
    pub y_accel: f64,
    pub z_accel: f64,
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "x_angrate: {}, y_angrate: {}, z_angrate: {}, x_accel: {}, y_accel: {}, z_accel: {}",
            self.x_ang_rate,
            self.y_ang_rate,
            self.z_ang_rate,
            self.x_accel,
            self.y_accel,
            self.z_accel
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HnrIns {
    pub key: String,
    pub itow: u32,
    pub ins: Ins,
}

impl fmt::Display for HnrIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: HnrIns, @key: '{}'\nitow:{}, ins: {}",
            self.key, self.itow, self.ins,
        )
    }
}

impl Keyed for HnrIns {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
