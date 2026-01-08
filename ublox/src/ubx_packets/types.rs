use crate::error::DateTimeError;
use chrono::prelude::*;
use core::fmt;

/// Represents a geodetic Position in the form of Longitude, Latitude, and Altitude
/// This can be constructed for example from NavPosLlh and NavPvt uBlox packets.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PositionLLA {
    /// Longitude in degrees
    pub lon: f64,

    /// Latitude in degrees
    pub lat: f64,

    /// Altitude in meters
    pub alt: f64,
}

/// A trait for types that can provide LLA (Longitude, Latitude, Altitude) Position information.
///
/// This trait is implemented by uBlox packets that contain LLA Position data,
/// allowing them to be converted to a [`PositionLLA`] struct.
pub(crate) trait ToLLA {
    fn to_lla(&self) -> PositionLLA;
}

/// Represents a Cartesian Position in the ECEF (Earth-Centered, Earth-Fixed) coordinate system.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PositionECEF {
    /// X coordinate in meters
    pub x: f64,

    /// Y coordinate in meters
    pub y: f64,

    /// Z coordinate in meters
    pub z: f64,
}

/// A trait for types that can provide ECEF (Earth-Centered, Earth-Fixed) Position information.
///
/// This trait is implemented by uBlox packets that contain ECEF Position data,
/// allowing them to be converted to a [`PositionECEF`] struct.
pub(crate) trait ToECEF {
    fn to_ecef(&self) -> PositionECEF;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Velocity {
    /// Speed in meters per second over the ground
    pub speed: f64,

    /// Heading in degrees
    pub heading: f64,
}

/// A trait for types that can provide velocity (speed and heading) information.
///
/// This trait is implemented by uBlox packets that contain velocity data,
/// allowing them to be converted to a [`Velocity`] struct.
pub(crate) trait ToVelocity {
    fn to_velocity(&self) -> Velocity;
}

impl<T> From<&T> for PositionLLA
where
    T: ToLLA,
{
    fn from(packet: &T) -> Self {
        packet.to_lla()
    }
}

impl<T> From<&T> for Velocity
where
    T: ToVelocity,
{
    fn from(packet: &T) -> Self {
        packet.to_velocity()
    }
}

impl<T> From<&T> for PositionECEF
where
    T: ToECEF,
{
    fn from(packet: &T) -> Self {
        packet.to_ecef()
    }
}

/// A trait for types that can provide Date Time information and convert it to a DateTime<Utc>
///
/// This trait is implemented by uBlox packets that contain Date Time data,
/// allowing them to be converted to a [`DateTime<Utc>`] struct.
pub(crate) trait ToDateTime {
    /// Convert to a DateTime<Utc> struct
    fn to_datetime(&self) -> Result<DateTime<Utc>, DateTimeError>;
}

/// Helper function to convert date & time components to DateTime<Utc>
pub(crate) fn datetime_from_components(
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
