use super::packets::{
    nav_hp_pos_ecef::{NavHpPosEcefOwned, NavHpPosEcefRef},
    nav_hp_pos_llh::{NavHpPosLlhOwned, NavHpPosLlhRef},
    nav_pos_llh::{NavPosLlhOwned, NavPosLlhRef},
    nav_pvt::{NavPvtOwned, NavPvtRef},
    nav_vel_ned::{NavVelNedOwned, NavVelNedRef},
};
use crate::error::DateTimeError;
use chrono::prelude::*;
use core::{convert::TryFrom, fmt};

/// Represents a world position, can be constructed from NavPosLlh and NavPvt packets.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Longitude in degrees
    pub lon: f64,

    /// Latitude in degrees
    pub lat: f64,

    /// Altitude in meters
    pub alt: f64,
}

/// Represents a world position in the ECEF coordinate system
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PositionECEF {
    /// x coordinates in meters
    pub x: f64,

    /// y coordinates in meters
    pub y: f64,

    /// z coordinates in meters
    pub z: f64,
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

impl From<&NavPosLlhOwned> for Position {
    fn from(packet: &NavPosLlhOwned) -> Self {
        Position {
            lon: packet.lon_degrees(),
            lat: packet.lat_degrees(),
            alt: packet.height_msl(),
        }
    }
}

impl<'a> From<&NavHpPosLlhRef<'a>> for Position {
    fn from(packet: &NavHpPosLlhRef<'a>) -> Self {
        Position {
            lon: packet.lon_degrees() + packet.lon_hp_degrees(),
            lat: packet.lat_degrees() + packet.lat_hp_degrees(),
            alt: packet.height_msl() + packet.height_hp_msl(),
        }
    }
}

impl From<&NavHpPosLlhOwned> for Position {
    fn from(packet: &NavHpPosLlhOwned) -> Self {
        Position {
            lon: packet.lon_degrees() + packet.lon_hp_degrees(),
            lat: packet.lat_degrees() + packet.lat_hp_degrees(),
            alt: packet.height_msl() + packet.height_hp_msl(),
        }
    }
}

fn ecef_from_cm_hp(cm: f64, hp_mm: f64) -> f64 {
    10e-2 * (cm + 0.1 * hp_mm)
}

impl<'a> From<&NavHpPosEcefRef<'a>> for PositionECEF {
    fn from(p: &NavHpPosEcefRef<'a>) -> Self {
        PositionECEF {
            x: ecef_from_cm_hp(p.ecef_x_cm(), p.ecef_x_hp_mm()),
            y: ecef_from_cm_hp(p.ecef_y_cm(), p.ecef_y_hp_mm()),
            z: ecef_from_cm_hp(p.ecef_z_cm(), p.ecef_z_hp_mm()),
        }
    }
}

impl From<&NavHpPosEcefOwned> for PositionECEF {
    fn from(p: &NavHpPosEcefOwned) -> Self {
        PositionECEF {
            x: ecef_from_cm_hp(p.ecef_x_cm(), p.ecef_x_hp_mm()),
            y: ecef_from_cm_hp(p.ecef_y_cm(), p.ecef_y_hp_mm()),
            z: ecef_from_cm_hp(p.ecef_z_cm(), p.ecef_z_hp_mm()),
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

impl From<&NavVelNedOwned> for Velocity {
    fn from(packet: &NavVelNedOwned) -> Self {
        Velocity {
            speed: packet.ground_speed(),
            heading: packet.heading_degrees(),
        }
    }
}

impl<'a> From<&NavPvtRef<'a>> for Position {
    fn from(packet: &NavPvtRef<'a>) -> Self {
        Position {
            lon: packet.longitude(),
            lat: packet.latitude(),
            alt: packet.height_msl(),
        }
    }
}

impl From<&NavPvtOwned> for Position {
    fn from(packet: &NavPvtOwned) -> Self {
        Position {
            lon: packet.longitude(),
            lat: packet.latitude(),
            alt: packet.height_msl(),
        }
    }
}

impl<'a> From<&NavPvtRef<'a>> for Velocity {
    fn from(packet: &NavPvtRef<'a>) -> Self {
        Velocity {
            speed: packet.ground_speed_2d(),
            heading: packet.heading_motion(),
        }
    }
}

impl From<&NavPvtOwned> for Velocity {
    fn from(packet: &NavPvtOwned) -> Self {
        Velocity {
            speed: packet.ground_speed_2d(),
            heading: packet.heading_motion(),
        }
    }
}

fn datetime_from_nav_pvt(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,
    nanos: i32,
) -> Result<DateTime<Utc>, DateTimeError> {
    let date = NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
        .ok_or(DateTimeError::InvalidDate)?;

    let time = NaiveTime::from_hms_opt(u32::from(hour), u32::from(min), u32::from(sec))
        .ok_or(DateTimeError::InvalidTime)?;

    const NANOS_LIM: u32 = 1_000_000_000;
    if (nanos.wrapping_abs() as u32) >= NANOS_LIM {
        return Err(DateTimeError::InvalidNanoseconds);
    }

    let dt = NaiveDateTime::new(date, time) + chrono::Duration::nanoseconds(i64::from(nanos));
    Ok(DateTime::from_naive_utc_and_offset(dt, Utc))
}

impl<'a> TryFrom<&NavPvtRef<'a>> for DateTime<Utc> {
    type Error = DateTimeError;
    fn try_from(sol: &NavPvtRef<'a>) -> Result<Self, Self::Error> {
        datetime_from_nav_pvt(
            sol.year(),
            sol.month(),
            sol.day(),
            sol.hour(),
            sol.min(),
            sol.sec(),
            sol.nanosec(),
        )
    }
}

impl TryFrom<&NavPvtOwned> for DateTime<Utc> {
    type Error = DateTimeError;
    fn try_from(sol: &NavPvtOwned) -> Result<Self, Self::Error> {
        datetime_from_nav_pvt(
            sol.year(),
            sol.month(),
            sol.day(),
            sol.hour(),
            sol.min(),
            sol.sec(),
            sol.nanosec(),
        )
    }
}

#[allow(dead_code, reason = "It is only dead code in some feature sets")]
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
