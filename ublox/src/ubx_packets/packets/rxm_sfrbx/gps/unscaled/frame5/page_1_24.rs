/// Interpretation that prevails for Frame #5 page 1-24 (included)
use super::super::super::GPS_PARITY_SIZE;

const WORD3_DATAID_MASK: u32 = 0xc00000;
const WORD3_DATAID_SHIFT: u32 = 22;
const WORD3_SVID_MASK: u32 = 0x3c0000;
const WORD3_SVID_SHIFT: u32 = 20;
const WORD3_E_MASK: u32 = 0x03ffff;
const WORD3_E_SHIFT: u32 = 0;

const WORD4_TOA_MASK: u32 = 0xff0000;
const WORD4_TOA_SHIFT: u32 = 16;
const WORD4_DELTAI_MASK: u32 = 0x00ffff;
const WORD4_DELTAI_SHIFT: u32 = 0;

const WORD5_OMEGADOT_MASK: u32 = 0xffff00;
const WORD5_OMEGADOT_SHIFT: u32 = 8;
const WORD5_SVHEALTH_MASK: u32 = 0x0000ff;
const WORD5_SVHEALTH_SHIFT: u32 = 0;

const WORD6_SQRTA_MASK: u32 = 0xffffff;
const WORD6_SQRTA_SHIFT: u32 = 0;

const WORD7_OMEGA0_MASK: u32 = 0xffffff;
const WORD7_OMEGA0_SHIFT: u32 = 0;

const WORD8_OMEGA_MASK: u32 = 0xffffff;
const WORD8_OMEGA_SHIFT: u32 = 0;

const WORD9_M0_MASK: u32 = 0xffffff;
const WORD9_M0_SHIFT: u32 = 0;

const WORD10_AF0MSB_MASK: u32 = 0xff0000;
const WORD10_AF0MSB_SHIFT: u32 = 16 - 3;
const WORD10_AF0LSB_MASK: u32 = 0x000007;
const WORD10_AF1_MASK: u32 = 0x00fff8;
const WORD10_AF1_SHIFT: u32 = 3;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word3 {
    /// 2-bit data ID
    pub data_id: u8,

    /// 6-bit SV ID
    pub sv_id: u8,

    /// eccentricity
    pub e: u16,
}

impl GpsUnscaled5Page1Thru24Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let data_id = ((dword & WORD3_DATAID_MASK) >> WORD3_DATAID_SHIFT) as u8;
        let sv_id = ((dword & WORD3_SVID_MASK) >> WORD3_SVID_SHIFT) as u8;
        let e = ((dword & WORD3_E_MASK) >> WORD3_E_SHIFT) as u16;
        Self { data_id, sv_id, e }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word4 {
    /// Toa
    pub toa: u8,

    /// delta_i
    pub delta_i: u16,
}

impl GpsUnscaled5Page1Thru24Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let toa = ((dword & WORD4_TOA_MASK) >> WORD4_TOA_SHIFT) as u8;
        let delta_i = ((dword & WORD4_DELTAI_MASK) >> WORD4_DELTAI_SHIFT) as u16;
        Self { toa, delta_i }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word5 {
    pub omega_dot: u16,
    pub sv_health: u8,
}

impl GpsUnscaled5Page1Thru24Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega_dot = ((dword & WORD5_OMEGADOT_MASK) >> WORD5_OMEGADOT_SHIFT) as u16;
        let sv_health = ((dword & WORD5_SVHEALTH_MASK) >> WORD5_SVHEALTH_SHIFT) as u8;
        Self {
            omega_dot,
            sv_health,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word6 {
    /// 24-bit SQRT(a)
    pub sqrt_a: u32,
}

impl GpsUnscaled5Page1Thru24Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let sqrt_a = ((dword & WORD6_SQRTA_MASK) >> WORD6_SQRTA_SHIFT) as u32;
        Self { sqrt_a }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word7 {
    /// 24-bit omega0
    pub omega0: u32,
}

impl GpsUnscaled5Page1Thru24Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega0 = ((dword & WORD7_OMEGA0_MASK) >> WORD7_OMEGA0_SHIFT) as u32;
        Self { omega0 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word8 {
    /// 24-bit omega
    pub omega: u32,
}

impl GpsUnscaled5Page1Thru24Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega = ((dword & WORD8_OMEGA_MASK) >> WORD8_OMEGA_SHIFT) as u32;
        Self { omega }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word9 {
    /// 24-bit m0
    pub m0: u32,
}

impl GpsUnscaled5Page1Thru24Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let m0 = ((dword & WORD9_M0_MASK) >> WORD9_M0_SHIFT) as u32;
        Self { m0 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page1Thru24Word10 {
    pub af0: u16,
    pub af1: u16,
}

impl GpsUnscaled5Page1Thru24Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let mut af0 = (dword & WORD10_AF0LSB_MASK) as u16;
        let af1 = ((dword & WORD10_AF1_MASK) >> WORD10_AF1_SHIFT) as u16;
        af0 |= ((dword & WORD10_AF0MSB_MASK) as u16) >> WORD10_AF0MSB_SHIFT;
        Self { af0, af1 }
    }
}
