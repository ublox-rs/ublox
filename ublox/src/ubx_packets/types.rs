use super::packets::*;
use chrono::prelude::*;

#[derive(Debug)]
pub struct Position {
    pub lon: f64,
    pub lat: f64,
    pub alt: f64,
}

#[derive(Debug)]
pub struct Velocity {
    pub speed: f64,   // m/s over ground
    pub heading: f64, // degrees
}

impl<'a> From<&NavPosLlhRef<'a>> for Position {
    fn from(packet: &NavPosLlhRef<'a>) -> Self {
        Position {
            lon: packet.lon_degrees(),
            lat: packet.lat_degrees(),
            alt: packet.height_msl(),
        }
    }
}

impl<'a> From<&NavVelNedRef<'a>> for Velocity {
    fn from(packet: &NavVelNedRef<'a>) -> Self {
        Velocity {
            speed: packet.ground_speed(),
            heading: packet.heading_degrees(),
        }
    }
}

impl<'a> From<&NavPosVelTimeRef<'a>> for Position {
    fn from(packet: &NavPosVelTimeRef<'a>) -> Self {
        Position {
            lon: packet.lon_degrees(),
            lat: packet.lat_degrees(),
            alt: packet.height_msl(),
        }
    }
}

impl<'a> From<&NavPosVelTimeRef<'a>> for Velocity {
    fn from(packet: &NavPosVelTimeRef<'a>) -> Self {
        Velocity {
            speed: packet.ground_speed(),
            heading: packet.heading_degrees(),
        }
    }
}

impl<'a> From<&NavPosVelTimeRef<'a>> for DateTime<Utc> {
    fn from(sol: &NavPosVelTimeRef<'a>) -> Self {
        let ns = if sol.nanosecond().is_negative() {
            0
        } else {
            sol.nanosecond()
        } as u32;
        Utc.ymd(sol.year() as i32, sol.month().into(), sol.day().into())
            .and_hms_nano(sol.hour().into(), sol.min().into(), sol.sec().into(), ns)
    }
}
