use super::packets::{
    nav_hp_pos_ecef::{NavHpPosEcefOwned, NavHpPosEcefRef},
    nav_hp_pos_llh::{NavHpPosLlhOwned, NavHpPosLlhRef},
    nav_pos_llh::{NavPosLlhOwned, NavPosLlhRef},
    nav_vel_ned::{NavVelNedOwned, NavVelNedRef},
};
use crate::error::DateTimeError;
use chrono::prelude::*;
use core::fmt;

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

/// A common trait for accessing NavPvt fields across all protocol versions
pub(crate) trait NavPvtFields {
    fn longitude(&self) -> f64;
    fn latitude(&self) -> f64;
    fn height_msl(&self) -> f64;
    fn ground_speed_2d(&self) -> f64;
    fn heading_motion(&self) -> f64;
    fn year(&self) -> u16;
    fn month(&self) -> u8;
    fn day(&self) -> u8;
    fn hour(&self) -> u8;
    fn min(&self) -> u8;
    fn sec(&self) -> u8;
    fn nanosec(&self) -> i32;
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

impl<T> From<&T> for Position
where
    T: NavPvtFields,
{
    fn from(packet: &T) -> Self {
        Position {
            lon: packet.longitude(),
            lat: packet.latitude(),
            alt: packet.height_msl(),
        }
    }
}

impl<T> From<&T> for Velocity
where
    T: NavPvtFields,
{
    fn from(packet: &T) -> Self {
        Velocity {
            speed: packet.ground_speed_2d(),
            heading: packet.heading_motion(),
        }
    }
}

pub(crate) fn datetime_from_nav_pvt<T: NavPvtFields>(
    sol: &T,
) -> Result<DateTime<Utc>, DateTimeError> {
    let year = sol.year();
    let month = sol.month();
    let day = sol.day();
    let hour = sol.hour();
    let min = sol.min();
    let sec = sol.sec();
    let nanos = sol.nanosec();
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
