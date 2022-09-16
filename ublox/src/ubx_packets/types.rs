use super::packets::*;
use crate::error::DateTimeError;
use chrono::prelude::*;
use core::{convert::TryFrom, fmt};

/// Represents a world position, can be constructed from NavPosLlh and NavPosVelTime packets.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Logitude in degrees
    pub lon: f64,

    /// Latitude in degrees
    pub lat: f64,

    /// Altitude in meters
    pub alt: f64,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Velocity {
    /// m/s over the ground
    pub speed: f64,

    /// Heading in degrees
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

pub(crate) struct FieldIter<I>(pub(crate) I);

impl<I> fmt::Debug for FieldIter<I>
where
    I: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<I> serde::Serialize for FieldIter<I>
where
    I: Iterator + Clone,
    I::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.clone())
    }
}
