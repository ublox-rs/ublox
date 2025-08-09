use rustdds::Keyed;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct EsfAlg {
    pub key: String,
    pub itow: u32,
    pub flags: u8,
    pub errors: u8,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl EsfAlg {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for EsfAlg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Keyed for EsfAlg {
    type K = String;
    fn key(&self) -> String {
        self.key.clone()
    }
}
