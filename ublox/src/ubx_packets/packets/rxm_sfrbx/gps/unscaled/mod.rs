pub(crate) mod frame1;
pub(crate) mod frame2;
pub(crate) mod frame3;
// pub(crate) mod frame5; // almanac: not supported yet

pub(crate) use frame1::*;
pub(crate) use frame2::*;
pub(crate) use frame3::*;
// pub(crate) use frame5::*; // almanac: not supported yet

use super::{scaled::*, GpsHowWord, GpsTelemetryWord};

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
    pub fn scale(&self) -> GpsEphFrame1 {
        GpsEphFrame1 {
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
    pub fn scale(&self) -> GpsEphFrame2 {
        GpsEphFrame2 {
            iode: self.word3.iode,
            toe: (self.word10.toe as u32) * 2_u32.pow(4),
            crs: (self.word3.crs as f64) / 2.0_f64.powi(5),
            delta_n: (self.word4.delta_n as f64) / 2.0_f64.powi(43),
            m0: {
                let mut m0 = (self.word4.m0_msb as u32) << 25;
                m0 += self.word5.m0_lsb;

                (m0 as i32) as f64 / 2.0_f64.powi(31)
            },
            cus: (self.word8.cus as f64) / 2.0_f64.powi(29),
            cuc: (self.word6.cuc as f64) / 2.0_f64.powi(29),
            e: {
                // form u32 word
                let mut e = (self.word6.e_msb as u32) << 25;
                e += self.word7.e_lsb;

                (e as i32) as f64 / 2.0_f64.powi(33)
            },
            sqrt_a: {
                let mut sqrt_a = self.word9.sqrt_a_lsb;
                sqrt_a += (self.word8.sqrt_a_msb as u32) << 25;
                (sqrt_a as f64) / 2.0_f64.powi(19) 
            },
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
    pub fn scale(&self) -> GpsEphFrame3 {
        GpsEphFrame3 {
            cis: (self.word5.cis as f64) / 2.0_f64.powi(29),
            i0: {
                // form the u32 raw word
                let mut i0 = (self.word5.i0_msb as u32) << 24;
                i0 += self.word6.i0_lsb;

                (i0 as i32) as f64 / 2.0_f64.powi(31)
            },
            cic: (self.word3.cic as f64) / 2.0_f64.powi(29),
            omega0: {
                // form the u32 raw word
                let mut omega0 = (self.word3.omega0_msb as u32) << 24;
                omega0 += self.word4.omega0_lsb;

                (omega0 as i32) as f64 / 2.0_f64.powi(31)
            },
            crc: (self.word7.crc as f64) / 2.0_f64.powi(5),
            iode: self.word10.iode,
            idot: (self.word10.idot as f64) / 2.0_f64.powi(43),

            omega_dot: (self.word9.omega_dot as f64) / 2.0_f64.powi(43),

            omega: {
                // form the u32 raw word
                let mut omega = (self.word7.omega_msb as u32) << 24;
                omega += self.word8.omega_lsb;

                (omega as i32) as f64 / 2.0_f64.powi(31)
            },
        }
    }
}

// Almanac Pages 1-24 not supported yet
// /// Interpratation that is valid for Page ID 1-24 included.
// /// Afterwards, there is one final 25th page.
// #[derive(Debug, Default, Clone)]
// pub(crate) struct GpsUnscaled5Page1Thru24 {
//     pub word3: GpsUnscaled5Page1Thru24Word3,
//     pub word4: GpsUnscaled5Page1Thru24Word4,
//     pub word5: GpsUnscaled5Page1Thru24Word5,
//     pub word6: GpsUnscaled5Page1Thru24Word6,
//     pub word7: GpsUnscaled5Page1Thru24Word7,
//     pub word8: GpsUnscaled5Page1Thru24Word8,
//     pub word9: GpsUnscaled5Page1Thru24Word9,
//     pub word10: GpsUnscaled5Page1Thru24Word10,
// }

// impl GpsUnscaled5Page1Thru24 {
//     pub fn scale(&self) -> GpsSubframe5Page1Thru24 {
//         GpsSubframe5Page1Thru24 {
//             data_id: 0,
//             sv_id: 0,
//             e: {
//                 // form u32 word
//                 let e = (self.word3.e as u32) as f64;
//                 e / 2.0_f64.powi(21)
//             },
//             toa: (self.word4.toa as u32) * 2_u32.pow(12),
//             delta_i: 0.0,
//             omega_dot: 0.0,
//             sv_health: 0,
//             sqrt_a: 0.0,
//             omega_0: 0.0,
//             omega: 0.0,
//             m0: 0.0,
//         }
//     }
// }

// Almanac Page 25 not supported yet
// /// Interpratation that is valid for Page ID 25 of frame #5,
// /// following the first 24 pages.
// #[derive(Debug, Default, Clone)]
// pub(crate) struct GpsUnscaled5Page25 {
//     pub word3: GpsUnscaled5Page25Word3,
//     pub sv1_4_health: GpsUnscaled5Page25HealthWord,
//     pub sv5_8_health: GpsUnscaled5Page25HealthWord,
//     pub sv9_12_health: GpsUnscaled5Page25HealthWord,
//     pub sv13_16_health: GpsUnscaled5Page25HealthWord,
//     pub sv17_20_health: GpsUnscaled5Page25HealthWord,
//     pub sv21_24_health: GpsUnscaled5Page25HealthWord,
// }

// impl GpsUnscaled5Page25 {
//     pub fn scale(&self) -> GpsSubframe5Page25 {
//         GpsSubframe5Page25 {
//             data_id: self.word3.data_id,
//             page_id: self.word3.page_id,
//             toa: (self.word3.toa as u32) * 2_u32.pow(12),
//             wna: self.word3.wna,
//             sv1_health: self.sv1_4_health.sv_1msb_health,
//             sv2_health: self.sv1_4_health.sv_2_health,
//             sv3_health: self.sv1_4_health.sv_3_health,
//             sv4_health: self.sv1_4_health.sv_4lsb_health,

//             sv5_health: self.sv5_8_health.sv_1msb_health,
//             sv6_health: self.sv5_8_health.sv_2_health,
//             sv7_health: self.sv5_8_health.sv_3_health,
//             sv8_health: self.sv5_8_health.sv_4lsb_health,

//             sv9_health: self.sv9_12_health.sv_1msb_health,
//             sv10_health: self.sv9_12_health.sv_2_health,
//             sv11_health: self.sv9_12_health.sv_3_health,
//             sv12_health: self.sv9_12_health.sv_4lsb_health,

//             sv13_health: self.sv13_16_health.sv_1msb_health,
//             sv14_health: self.sv13_16_health.sv_2_health,
//             sv15_health: self.sv13_16_health.sv_3_health,
//             sv16_health: self.sv13_16_health.sv_4lsb_health,

//             sv17_health: self.sv17_20_health.sv_1msb_health,
//             sv18_health: self.sv17_20_health.sv_2_health,
//             sv19_health: self.sv17_20_health.sv_3_health,
//             sv20_health: self.sv17_20_health.sv_4lsb_health,

//             sv21_health: self.sv21_24_health.sv_1msb_health,
//             sv22_health: self.sv21_24_health.sv_2_health,
//             sv23_health: self.sv21_24_health.sv_3_health,
//             sv24_health: self.sv21_24_health.sv_4lsb_health,
//         }
//     }
// }

/// Interprated [GpsUnscaledSubframe]s (not scaled yet)
#[derive(Debug, Clone)]
pub(crate) enum GpsUnscaledSubframe {
    /// GPS Ephemeris #1 frame
    Eph1(GpsUnscaledEph1),

    /// GPS - Unscaled Subframe #2
    Eph2(GpsUnscaledEph2),

    /// GPS - Unscaled Subframe #3
    Eph3(GpsUnscaledEph3),
    // GPS Almanac Page 1-24 not supported yet
    // /// GPS - Unscaled Subframe #5 Pages 1-24 (included).
    // /// Use the page_id field to determine which one it is.
    // Subframe5Page1Thru24(GpsUnscaled5Page1Thru24),

    // GPS Almanac Page 25 not supported yet
    // /// GPS - Unscaled Subframe #5 Page 25 (last one).
    // Subframe5Page25(GpsUnscaled5Page25),
}

impl Default for GpsUnscaledSubframe {
    fn default() -> Self {
        Self::Eph1(Default::default())
    }
}

impl GpsUnscaledSubframe {
    pub fn scale(&self) -> GpsSubframe {
        match self {
            Self::Eph1(subframe) => GpsSubframe::Eph1(subframe.scale()),
            Self::Eph2(subframe) => GpsSubframe::Eph2(subframe.scale()),
            Self::Eph3(subframe) => GpsSubframe::Eph3(subframe.scale()),
            // Almanac Page 1-24 not supported yet
            // Self::Subframe5Page1Thru24(subframe) => GpsSubframe::Subframe5Page1Thru24(subframe.scale()),
            // Almanac Page 25 not supported yet
            // Self::Subframe5Page25(subframe) => GpsSubframe::Subframe5Page25(subframe.scale()),
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
