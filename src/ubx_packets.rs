use serde_derive::{Serialize, Deserialize};
use std::vec::Vec;
use chrono::prelude::*;

#[derive(Debug)]
pub struct Position {
    pub lon: f32,
    pub lat: f32,

    /// In meters above sea level.
    pub alt: f32,
}

#[derive(Debug)]
pub struct Velocity {
    /// In meters/second over the ground
    pub speed: f32,

    /// Degrees
    pub heading: f32, // degrees
}

#[derive(Debug)]
pub struct UbxPacket {
    pub class: u8,
    pub id: u8,
    pub payload: Vec<u8>,
}

impl UbxPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(0xB5);
        v.push(0x62);
        v.push(self.class);
        v.push(self.id);

        let length = self.payload.len() as u16;
        v.push((length & 0xFF) as u8);
        v.push(((length >> 8) & 0xFF) as u8);

        for b in self.payload.iter() {
            v.push(*b);
        }

        // Calculate the checksum
        let mut cka = 0;
        let mut ckb = 0;
        for i in 0..self.payload.len()+4 {
            cka = ((cka as usize + v[i + 2] as usize) & 0xFF) as u8;
            ckb = ((cka as usize + ckb as usize) & 0xFF) as u8;
        }
        v.push(cka);
        v.push(ckb);
        v
    }

    fn compute_checksum(&self) -> (u8, u8) {
        let s = self.serialize();
        let cka = s[s.len() - 2];
        let ckb = s[s.len() - 1];
        return (cka, ckb);
    }

    pub fn check_checksum(&self, test_cka: u8, test_ckb: u8) -> bool {
        let (cka, ckb) = self.compute_checksum();
        cka == test_cka && ckb == test_ckb
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavPosLLH {
    pub itow: u32,
    pub lon: i32,
    pub lat: i32,
    pub height: i32,
    pub height_msl: i32,
    pub horizontal_accuracy: u32,
    pub vertical_accuracy: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavVelNED {
    pub itow: u32,
    pub vel_north: i32, // cm/s
    pub vel_east: i32,
    pub vel_down: i32,
    pub speed: u32,
    pub ground_speed: u32,
    pub heading: i32, // 1e-5 degrees
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavPosVelTime {
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub time_accuracy: u32,
    pub nanosecond: i32,
    pub fix_type: u8,
    pub flags: u8,
    pub reserved1: u8,
    pub num_satellites: u8,
    pub lon: i32,
    pub lat: i32,
    pub height: i32,
    pub height_msl: i32,
    pub horiz_accuracy: u32,
    pub vert_accuracy: u32,
    pub vel_north: i32, // mm/s
    pub vel_east: i32,
    pub vel_down: i32,
    pub ground_speed: i32, // mm/s
    pub heading: i32, // 1e-5 deg
    pub speed_accuracy: u32,
    pub heading_accuracy: u32,
    pub pos_dop: u16,
    pub reserved2: u16,
    pub reserved3: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavStatus {
    pub itow: u32,
    pub gps_fix: u8,
    pub flags: u8,
    pub fix_status: u8,
    pub flags2: u8,
    pub time_to_first_fix: u32,
    pub uptime_ms: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AidIni {
    ecef_x_or_lat: i32,
    ecef_y_or_lon: i32,
    ecef_z_or_alt: i32,
    pos_accuracy: u32,
    time_cfg: u16,
    week_or_ym: u16,
    tow_or_hms: u32,
    tow_ns: i32,
    tm_accuracy_ms: u32,
    tm_accuracy_ns: u32,
    clk_drift_or_freq: i32,
    clk_drift_or_freq_accuracy: u32,
    flags: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlpSrv {
    pub id_size: u8,
    pub data_type: u8,
    pub offset: u16,
    pub size: u16,
    pub file_id: u16,
    pub data_size: u16,
    pub id1: u8,
    pub id2: u8,
    pub id3: u32,
}

impl AidIni {
    pub fn new() -> AidIni {
        AidIni {
            ecef_x_or_lat: 0,
            ecef_y_or_lon: 0,
            ecef_z_or_alt: 0,
            pos_accuracy: 0,
            time_cfg: 0,
            week_or_ym: 0,
            tow_or_hms: 0,
            tow_ns: 0,
            tm_accuracy_ms: 0,
            tm_accuracy_ns: 0,
            clk_drift_or_freq: 0,
            clk_drift_or_freq_accuracy: 0,
            flags: 0,
        }
    }

    pub fn set_position(&mut self, pos: Position) {
        self.ecef_x_or_lat = (pos.lat * 10_000_000.0) as i32;
        self.ecef_y_or_lon = (pos.lon * 10_000_000.0) as i32;
        self.ecef_z_or_alt = (pos.alt * 100.0) as i32; // Height is in centimeters, here
        self.flags |= (1 << 0) | (1 << 5);
    }

    pub fn set_time(&mut self, tm: DateTime<Utc>) {
        self.week_or_ym = (match tm.year_ce() {
            (true, yr) => { yr - 2000 },
            (false, _) => { panic!("Jesus must have been born for this method to work"); },
        } * 100 + tm.month0()) as u16;
        self.tow_or_hms = tm.hour() * 10000 + tm.minute() * 100 + tm.second();
        self.tow_ns = tm.nanosecond() as i32;
        self.flags |= (1 << 1) | (1 << 10);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AckAck {
    pub classid: u8,
    pub msgid: u8,
}
