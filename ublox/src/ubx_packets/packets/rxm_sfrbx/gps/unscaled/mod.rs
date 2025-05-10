pub(crate) mod frame1;
pub(crate) mod frame2;
pub(crate) mod frame3;
// frame4 contains mostly system reserved stuff
// but also the Kb model and A/S indication that could be useful..
pub(crate) mod frame5;

pub(crate) use frame1::*;
pub(crate) use frame2::*;
pub(crate) use frame3::*;
pub(crate) use frame5::*;

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
        GpsSubframe1 {
            week: self.word3.week,
            ca_or_p_l2: self.word3.ca_or_p_l2,
            ura: self.word3.ura,
            health: self.word3.health,
            iodc: { (self.word3.iodc_msb as u16) << 8 + self.word8.iodc_lsb as u16 },
            tgd: (self.word7.tgd as f64) / 2.0_f64.powi(31),
            toc: (self.word8.toc as u32) * 2_u32.pow(4),
            af2: (self.word9.af2 as f64) / 2.0_f64.powi(55),
            af1: (self.word9.af1 as f64) / 2.0_f64.powi(43),
            af0: (self.word10.af0 as f64) / 2.0_f64.powi(31),
        }
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
        GpsSubframe2 {
            iode: self.word3.iode,
            crs: (self.word3.crs as f64) / 2.0_f64.powi(5),
            delta_n: (self.word4.delta_n as f64) / 2.0_f64.powi(43),
            m0: 0.0,
            cuc: 0.0,
            e: 0.0,
        }
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
        GpsSubframe3 {
            cis: 0.0,
            i0: 0.0,
            cic: 0.0,
            omega0: 0.0,
            crc: 0.0,
            iode: 0.0,
            idot: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaled5 {
    pub word3: GpsUnscaled5Word3,
    pub word4: GpsUnscaled5Word4,
    pub word5: GpsUnscaled5Word5,
    pub word6: GpsUnscaled5Word6,
    pub word7: GpsUnscaled5Word7,
    pub word8: GpsUnscaled5Word8,
    pub word9: GpsUnscaled5Word9,
    pub word10: GpsUnscaled5Word10,
}

impl GpsUnscaled5 {
    pub fn scale(&self) -> GpsSubframe5 {
        GpsSubframe5 {
            data_id: 0,
            sv_id: 0,
            e: 0.0,
            p_dot: 0.0,
            sv_health: 0,
            sqrt_a: 0.0,
            omega0: 0.0,
            omega: 0.0,
            m0: 0.0,
            af0: 0.0,
            af1: 0.0,
        }
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

    /// GPS - Unscaled Paginated Subframe #5
    Subframe5(GpsUnscaled5),
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
            Self::Subframe5(subframe) => GpsSubframe::Subframe5(subframe.scale()),
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
