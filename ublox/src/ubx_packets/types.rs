use super::packets::*;
use crate::error::DateTimeError;
use chrono::prelude::*;
use std::convert::TryFrom;

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

impl<'a> TryFrom<&NavPosVelTimeRef<'a>> for DateTime<Utc> {
    type Error = DateTimeError;
    fn try_from(sol: &NavPosVelTimeRef<'a>) -> Result<Self, Self::Error> {
        let date = NaiveDate::from_ymd_opt(
            i32::from(sol.year()),
            u32::from(sol.month()),
            u32::from(sol.day()),
        )
        .ok_or(DateTimeError::InvalidDate)?;
        let time = NaiveTime::from_hms_opt(
            u32::from(sol.hour()),
            u32::from(sol.min()),
            u32::from(sol.sec()),
        )
        .ok_or(DateTimeError::InvalidTime)?;
        const NANOS_LIM: u32 = 1_000_000_000;
        if (sol.nanosecond().wrapping_abs() as u32) >= NANOS_LIM {
            return Err(DateTimeError::InvalidNanoseconds);
        }

        let dt = NaiveDateTime::new(date, time)
            + chrono::Duration::nanoseconds(i64::from(sol.nanosecond()));

        Ok(DateTime::from_utc(dt, Utc))
    }
}
