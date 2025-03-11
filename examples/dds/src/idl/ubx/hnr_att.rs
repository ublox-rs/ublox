use rustdds::Keyed;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Attitude {
    pub roll: f64,
    pub pitch: f64,
    pub heading: f64,
    pub acc_roll: f64,
    pub acc_pitch: f64,
    pub acc_heading: f64,
}

impl fmt::Display for Attitude {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "roll: {}, pitch: {}, heading: {}, acc_roll: {}, acc_pitch: {}, acc_heading: {}",
            self.roll, self.pitch, self.heading, self.acc_roll, self.acc_pitch, self.acc_heading
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HnrAtt {
    pub key: String,
    pub itow: u32,
    pub attitude: Attitude,
}

impl fmt::Display for HnrAtt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: HnrAtt, @key: '{}'\nitow:{}, attitude: {}",
            self.key, self.itow, self.attitude
        )
    }
}

impl Keyed for HnrAtt {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
