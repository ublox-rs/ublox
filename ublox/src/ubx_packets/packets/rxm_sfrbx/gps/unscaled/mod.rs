#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;

pub(crate) mod frame1;
pub(crate) mod frame2;
pub(crate) mod frame3;

pub(crate) use frame1::*;
pub(crate) use frame2::*;
pub(crate) use frame3::*;

use super::{scaled::*, RxmSfrbxGpsQzssHow, RxmSfrbxGpsQzssTelemetry};

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaledEph1 {
    pub word3: GpsUnscaledEph1Word3,
    pub word4: GpsUnscaledEph1Word4,
    pub word5: GpsUnscaledEph1Word5,
    pub word6: GpsUnscaledEph1Word6,
    pub word7: GpsUnscaledEph1Word7,
    pub word8: GpsUnscaledEph1Word8,
    pub word9: GpsUnscaledEph1Word9,
    pub word10: GpsUnscaledEph1Word10,
}

impl GpsUnscaledEph1 {
    pub fn scale(&self) -> RxmSfrbxGpsQzssFrame1 {
        RxmSfrbxGpsQzssFrame1 {
            week: self.word3.week,
            ca_or_p_l2: self.word3.ca_or_p_l2,
            ura: self.word3.ura,
            health: self.word3.health,
            reserved_word4: self.word4.reserved,
            reserved_word5: self.word5.reserved,
            reserved_word6: self.word6.reserved,
            reserved_word7: self.word7.reserved,

            iodc: {
                let mut iodc = self.word3.iodc_msb as u16;
                iodc <<= 8;
                iodc |= self.word8.iodc_lsb as u16;
                iodc
            },

            l2_p_data_flag: self.word4.l2_p_data_flag,

            tgd_s: (self.word7.tgd as f64) / 2.0_f64.powi(31),
            toc_s: (self.word8.toc as u32) * 16,
            af2_s_s2: (self.word9.af2 as f64) / 2.0_f64.powi(55),
            af1_s_s: (self.word9.af1 as f64) / 2.0_f64.powi(43),
            af0_s: (self.word10.af0 as f64) / 2.0_f64.powi(31),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaledEph2 {
    pub word3: GpsUnscaledEph2Word3,
    pub word4: GpsUnscaledEph2Word4,
    pub word5: GpsUnscaledEph2Word5,
    pub word6: GpsUnscaledEph2Word6,
    pub word7: GpsUnscaledEph2Word7,
    pub word8: GpsUnscaledEph2Word8,
    pub word9: GpsUnscaledEph2Word9,
    pub word10: GpsUnscaledEph2Word10,
}

impl GpsUnscaledEph2 {
    pub fn scale(&self) -> RxmSfrbxGpsQzssFrame2 {
        RxmSfrbxGpsQzssFrame2 {
            iode: self.word3.iode,
            toe_s: (self.word10.toe as u32) * 16,
            crs: (self.word3.crs as f64) / 2.0_f64.powi(5),
            cus: (self.word8.cus as f64) / 2.0_f64.powi(29),
            cuc: (self.word6.cuc as f64) / 2.0_f64.powi(29),

            dn: {
                let dn = self.word4.dn as f64;
                dn / 2.0_f64.powi(43)
            },

            m0: {
                let mut m0 = self.word4.m0_msb as u32;
                m0 <<= 24;
                m0 |= self.word5.m0_lsb as u32;

                let m0 = (m0 as i32) as f64;
                m0 / 2.0_f64.powi(31)
            },

            e: {
                let mut e = self.word6.e_msb as u32;
                e <<= 24;
                e |= self.word7.e_lsb;

                (e as f64) / 2.0_f64.powi(33)
            },

            sqrt_a: {
                let mut sqrt_a = self.word8.sqrt_a_msb as u32;
                sqrt_a <<= 24;
                sqrt_a |= self.word9.sqrt_a_lsb;

                (sqrt_a as f64) / 2.0_f64.powi(19)
            },

            aodo: self.word10.aodo,
            fit_int_flag: self.word10.fitint,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaledEph3 {
    pub word3: GpsUnscaledEph3Word3,
    pub word4: GpsUnscaledEph3Word4,
    pub word5: GpsUnscaledEph3Word5,
    pub word6: GpsUnscaledEph3Word6,
    pub word7: GpsUnscaledEph3Word7,
    pub word8: GpsUnscaledEph3Word8,
    pub word9: GpsUnscaledEph3Word9,
    pub word10: GpsUnscaledEph3Word10,
}

impl GpsUnscaledEph3 {
    pub fn scale(&self) -> RxmSfrbxGpsQzssFrame3 {
        RxmSfrbxGpsQzssFrame3 {
            iode: self.word10.iode,
            cic: (self.word3.cic as f64) / 2.0_f64.powi(29),
            cis: (self.word5.cis as f64) / 2.0_f64.powi(29),
            crc: (self.word7.crc as f64) / 2.0_f64.powi(5),

            i0: {
                let mut i0 = self.word5.i0_msb as u32;
                i0 <<= 24;
                i0 |= self.word6.i0_lsb;

                let i0 = (i0 as i32) as f64;
                i0 / 2.0_f64.powi(31)
            },

            omega0: {
                let mut omega0 = self.word3.omega0_msb as u32;
                omega0 <<= 24;
                omega0 |= self.word4.omega0_lsb;

                let omega0 = (omega0 as i32) as f64;
                omega0 / 2.0_f64.powi(31)
            },

            idot: {
                let idot = self.word10.idot as f64;
                idot / 2.0_f64.powi(43)
            },

            omega_dot: {
                let omega_dot = self.word9.omega_dot as f64;
                omega_dot / 2.0_f64.powi(43)
            },

            omega: {
                // form the u32 raw word
                let mut omega = self.word7.omega_msb as u32;
                omega <<= 24;
                omega |= self.word8.omega_lsb;

                let omega = (omega as i32) as f64;
                omega / 2.0_f64.powi(31)
            },
        }
    }
}

/// Interpreted [GpsUnscaledSubframe]s (not scaled yet)
#[derive(Debug, Clone)]
pub(crate) enum GpsUnscaledSubframe {
    /// GPS Ephemeris #1 frame
    Eph1(GpsUnscaledEph1),

    /// GPS - Unscaled Subframe #2
    Eph2(GpsUnscaledEph2),

    /// GPS - Unscaled Subframe #3
    Eph3(GpsUnscaledEph3),

    /// Non supported subframe
    NonSupported,
}

impl GpsUnscaledSubframe {
    pub fn scale(&self) -> Option<RxmSfrbxGpsQzssSubframe> {
        match self {
            Self::Eph1(subframe) => Some(RxmSfrbxGpsQzssSubframe::Eph1(subframe.scale())),
            Self::Eph2(subframe) => Some(RxmSfrbxGpsQzssSubframe::Eph2(subframe.scale())),
            Self::Eph3(subframe) => Some(RxmSfrbxGpsQzssSubframe::Eph3(subframe.scale())),
            Self::NonSupported => None,
        }
    }
}

impl Default for GpsUnscaledSubframe {
    fn default() -> Self {
        Self::NonSupported
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct GpsUnscaledFrame {
    pub how: RxmSfrbxGpsQzssHow,
    pub subframe: GpsUnscaledSubframe,
    pub telemetry: RxmSfrbxGpsQzssTelemetry,
}

impl GpsUnscaledFrame {
    /// Scale this [GpsUnscaledFrame] into [RxmSfrbxGpsQzssFrame], if it is correctly supported.
    pub(crate) fn scale(&self) -> Option<RxmSfrbxGpsQzssFrame> {
        Some(RxmSfrbxGpsQzssFrame {
            how: self.how.clone(),
            subframe: self.subframe.scale()?,
            telemetry: self.telemetry.clone(),
        })
    }
}
