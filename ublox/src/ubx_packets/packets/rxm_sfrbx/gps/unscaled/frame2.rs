use super::super::{gps_qzss_bitmask, twos_complement};

const WORD3_IODE_MASK: u32 = 0xff0000;
const WORD3_IODE_SHIFT: u32 = 16; // remaining payload bits
const WORD3_CRS_MASK: u32 = 0x00ffff;
const WORD3_CRS_SHIFT: u32 = 0;

const WORD4_DELTAN_MASK: u32 = 0xffff00;
const WORD4_DELTAN_SHIFT: u32 = 8;
const WORD4_M0_MSB_MASK: u32 = 0x0000ff;
const WORD4_M0_MSB_SHIFT: u32 = 0;

const WORD5_M0_LSB_MASK: u32 = 0xffffff;
const WORD5_M0_LSB_SHIFT: u32 = 0;

const WORD6_CUC_MASK: u32 = 0xffff00;
const WORD6_CUC_SHIFT: u32 = 8;
const WORD6_E_MSB_MASK: u32 = 0x0000ff;
const WORD6_E_MSB_SHIFT: u32 = 0;

const WORD7_E_LSB_MASK: u32 = 0xffffff;
const WORD7_E_LSB_SHIFT: u32 = 0;

const WORD8_CUS_MASK: u32 = 0xffff00;
const WORD8_CUS_SHIFT: u32 = 8;
const WORD8_SQRTA_MSB_MASK: u32 = 0x0000ff;
const WORD8_SQRTA_MSB_SHIFT: u32 = 0;

const WORD9_SQRTA_LSB_MASK: u32 = 0xffffff;
const WORD9_SQRTA_LSB_SHIFT: u32 = 0;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word3 {
    pub iode: u8,

    pub crs: i32,
}

impl GpsUnscaledEph2Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let iode = ((dword & WORD3_IODE_MASK) >> WORD3_IODE_SHIFT) as u8;

        let crs = ((dword & WORD3_CRS_MASK) >> WORD3_CRS_SHIFT) as u32;
        let crs = twos_complement(crs, 0xffff, 0x8000);

        Self { iode, crs }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word4 {
    pub dn: i16,

    /// M0 (8) msb, you need to associate this to Subframe #2 Word #5
    pub m0_msb: u8,
}

impl GpsUnscaledEph2Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let dn = ((dword & WORD4_DELTAN_MASK) >> WORD4_DELTAN_SHIFT) as i16;
        let m0_msb = ((dword & WORD4_M0_MSB_MASK) >> WORD4_M0_MSB_SHIFT) as u8;
        Self { dn, m0_msb }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word5 {
    /// M0 (24) lsb, you need to associate this to Subframe #2 Word #4
    pub m0_lsb: u32,
}

impl GpsUnscaledEph2Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let m0_lsb = ((dword & WORD5_M0_LSB_MASK) >> WORD5_M0_LSB_SHIFT) as u32;
        Self { m0_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word6 {
    pub cuc: i32,

    /// MSB(8) eccentricity, you need to associate this to Subframe #2 Word #7
    pub e_msb: u8,
}

impl GpsUnscaledEph2Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);

        let cuc = ((dword & WORD6_CUC_MASK) >> WORD6_CUC_SHIFT) as u32;
        let cuc = twos_complement(cuc, 0xffff, 0x8000);

        let e_msb = ((dword & WORD6_E_MSB_MASK) >> WORD6_E_MSB_SHIFT) as u8;

        Self { cuc, e_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word7 {
    /// LSB(24) eccentricity, you need to associate this to Subframe #2 Word #6
    pub e_lsb: u32,
}

impl GpsUnscaledEph2Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let e_lsb = ((dword & WORD7_E_LSB_MASK) >> WORD7_E_LSB_SHIFT) as u32;
        Self { e_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word8 {
    pub cus: i32,

    /// MSB(8) A⁻¹: you need to associate this to Subframe #2 Word #9
    pub sqrt_a_msb: u8,
}

impl GpsUnscaledEph2Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);

        let cus = ((dword & WORD8_CUS_MASK) >> WORD8_CUS_SHIFT) as u32;
        let cus = twos_complement(cus, 0xffff, 0x8000);

        let sqrt_a_msb = ((dword & WORD8_SQRTA_MSB_MASK) >> WORD8_SQRTA_MSB_SHIFT) as u8;
        Self { cus, sqrt_a_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word9 {
    /// LSB(24) A⁻¹: you need to associate this to Subframe #2 Word #8
    pub sqrt_a_lsb: u32,
}

impl GpsUnscaledEph2Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let sqrt_a_lsb = ((dword & WORD9_SQRTA_LSB_MASK) >> WORD9_SQRTA_LSB_SHIFT) as u32;
        Self { sqrt_a_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word10 {
    /// Time of issue of Ephemeris (u16)
    pub toe: u16,

    /// Fit interval. Differs between GPS and QZSS.
    pub fitint: bool,

    /// AODO
    pub aodo: u8,
}

impl GpsUnscaledEph2Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let toe = ((dword & 0x3fffc000) >> 14) as u16;
        let fitint = (dword & 0x00002000) > 0;
        let aodo = ((dword & 0x00001f00) >> 8) as u8;
        Self { toe, fitint, aodo }
    }
}
