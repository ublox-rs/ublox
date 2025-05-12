use super::super::{twos_complement, GPS_PARITY_SIZE};

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

const WORD10_TOE_MASK: u32 = 0xffff00;
const WORD10_TOE_SHIFT: u32 = 8;
const WORD10_FITINT_MASK: u32 = 0x000020;
const WORD10_AODO_MASK: u32 = 0x00001f;
const WORD10_AODO_SHIFT: u32 = 0;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word3 {
    pub iode: u8,

    pub crs: i32,
}

impl GpsUnscaledEph2Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let iode = ((dword & WORD3_IODE_MASK) >> WORD3_IODE_SHIFT) as u8;

        let crs = ((dword & WORD3_CRS_MASK) >> WORD3_CRS_SHIFT) as u32;
        let crs = twos_complement(crs, 0xffff, 0x8000);

        Self { iode, crs }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word4 {
    pub delta_n: i16,

    /// M0 (8) msb, you need to associate this to Subframe #2 Word #5
    pub m0_msb: u8,
}

impl GpsUnscaledEph2Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let delta_n = ((dword & WORD4_DELTAN_MASK) >> WORD4_DELTAN_SHIFT) as i16;
        let m0_msb = ((dword & WORD4_M0_MSB_MASK) >> WORD4_M0_MSB_SHIFT) as u8;
        Self { delta_n, m0_msb }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word5 {
    /// M0 (24) lsb, you need to associate this to Subframe #2 Word #4
    pub m0_lsb: u32,
}

impl GpsUnscaledEph2Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
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
        let dword = dword >> GPS_PARITY_SIZE;

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
        let dword = dword >> GPS_PARITY_SIZE;
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
        let dword = dword >> GPS_PARITY_SIZE;

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
        let dword = dword >> GPS_PARITY_SIZE;
        let sqrt_a_lsb = ((dword & WORD9_SQRTA_LSB_MASK) >> WORD9_SQRTA_LSB_SHIFT) as u32;
        Self { sqrt_a_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph2Word10 {
    /// Time of issue of Ephemeris (u16)
    pub toe: u16,
    pub fitint: bool,
    pub aodo: u8,
}

impl GpsUnscaledEph2Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);
        let toe = ((dword & WORD10_TOE_MASK) >> WORD10_TOE_SHIFT) as u16;
        let fitint = (dword & WORD10_FITINT_MASK) > 0;
        let aodo = ((dword & WORD10_AODO_MASK) >> WORD10_AODO_SHIFT) as u8;
        Self { toe, fitint, aodo }
    }
}
