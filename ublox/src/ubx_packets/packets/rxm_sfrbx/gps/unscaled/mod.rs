pub(crate) mod frame1;
pub(crate) mod frame2;
pub(crate) mod frame3;

pub(crate) use frame1::*;
pub(crate) use frame2::*;
pub(crate) use frame3::*;

use super::{scaled::*, GpsHowWord, GpsTelemetryWord};

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaled1 {
    pub word3: GpsUnscaled1Word3,
    pub word4: GpsUnscaled1Word4,
    pub word5: GpsUnscaled1Word5,
    pub word6: GpsUnscaled1Word6,
    pub word7: GpsUnscaled1Word7,
    pub word8: GpsUnscaled1Word8,
    pub word9: GpsUnscaled1Word9,
    pub word10: GpsUnscaled1Word10,
}

impl GpsUnscaled1 {
    pub fn scale(&self) -> GpsSubframe1 {
        GpsSubframe1::default()
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaled2 {
    pub word3: GpsUnscaled2Word3,
    pub word4: GpsUnscaled2Word4,
    pub word5: GpsUnscaled2Word5,
    pub word6: GpsUnscaled2Word6,
    pub word7: GpsUnscaled2Word7,
    pub word8: GpsUnscaled2Word8,
    pub word9: GpsUnscaled2Word9,
    pub word10: GpsUnscaled2Word10,
}

impl GpsUnscaled2 {
    pub fn scale(&self) -> GpsSubframe2 {
        GpsSubframe2::default()
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaled3 {
    pub word3: GpsUnscaled3Word3,
    pub word4: GpsUnscaled3Word4,
    pub word5: GpsUnscaled3Word5,
    pub word6: GpsUnscaled3Word6,
    pub word7: GpsUnscaled3Word7,
    pub word8: GpsUnscaled3Word8,
    pub word9: GpsUnscaled3Word9,
    pub word10: GpsUnscaled3Word10,
}

impl GpsUnscaled3 {
    pub fn scale(&self) -> GpsSubframe3 {
        GpsSubframe3::default()
    }
}

/// Interprated [GpsUnscaledSubframe]s (not scaled yet)
#[derive(Debug, Clone)]
pub(crate) enum GpsUnscaledSubframe {
    /// GPS - Unscaled Subframe #1
    Subframe1(GpsUnscaled1),

    /// GPS - Unscaled Subframe #2
    Subframe2(GpsUnscaled2),

    /// GPS - Unscaled Subframe #3
    Subframe3(GpsUnscaled3),
}

impl Default for GpsUnscaledSubframe {
    fn default() -> Self {
        Self::Subframe1(Default::default())
    }
}

impl GpsUnscaledSubframe {
    pub fn scale(&self) -> GpsSubframe {
        match self {
            Self::Subframe1(subframe) => GpsSubframe::Subframe1(subframe.scale()),
            Self::Subframe2(subframe) => GpsSubframe::Subframe2(subframe.scale()),
            Self::Subframe3(subframe) => GpsSubframe::Subframe3(subframe.scale()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaledFrame {
    pub telemetry: GpsTelemetryWord,
    pub how: GpsHowWord,
    pub subframe: GpsUnscaledSubframe,
}

impl GpsUnscaledFrame {
    pub(crate) fn scale(&self) -> GpsFrame {
        GpsFrame {
            telemetry: self.telemetry.clone(),
            how: self.how.clone(),
            subframe: self.subframe.scale(),
        }
    }
}
