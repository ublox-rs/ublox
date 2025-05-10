use super::GPS_PARITY_SIZE;

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
const WORD10_FITINT_MASK: u32 = 0x000080;
const WORD10_AODO_MASK: u32 = 0x00007C;
const WORD10_AODO_SHIFT: u32 = 2;

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word3 {
    pub iode: u8,
    pub crs: u16,
}

impl GpsSubframe2Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let iode = ((dword & WORD3_IODE_MASK) >> WORD3_IODE_SHIFT) as u8;
        let crs = ((dword & WORD3_CRS_MASK) >> WORD3_CRS_SHIFT) as u16;

        Self { iode, crs }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word4 {
    pub delta_n: u16,

    /// M0 (8) msb, you need to associate this to Subframe #2 Word #5
    pub m0_msb: u8,
}

impl GpsSubframe2Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let delta_n = ((dword & WORD4_DELTAN_MASK) >> WORD4_DELTAN_SHIFT) as u16;
        let m0_msb = ((dword & WORD4_M0_MSB_MASK) >> WORD4_M0_MSB_SHIFT) as u8;

        Self { delta_n, m0_msb }
    }
}
#[derive(Debug, Clone)]
pub struct GpsSubframe2Word5 {
    /// M0 (24) lsb, you need to associate this to Subframe #2 Word #4
    pub m0_lsb: u32,
}

impl GpsSubframe2Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);
        let m0_lsb = ((dword & WORD5_M0_LSB_MASK) >> WORD5_M0_LSB_SHIFT) as u32;

        Self { m0_lsb }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word6 {
    pub cuc: u16,

    /// MSB(8) eccentricity, you need to associate this to Subframe #2 Word #7
    pub e_msb: u8,
}

impl GpsSubframe2Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let cuc = ((dword & WORD6_CUC_MASK) >> WORD6_CUC_SHIFT) as u16;
        let e_msb = ((dword & WORD6_E_MSB_MASK) >> WORD6_E_MSB_SHIFT) as u8;

        Self { cuc, e_msb }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word7 {
    /// LSB(24) eccentricity, you need to associate this to Subframe #2 Word #6
    pub e_lsb: u32,
}

impl GpsSubframe2Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let e_lsb = ((dword & WORD7_E_LSB_MASK) >> WORD7_E_LSB_SHIFT) as u32;

        Self { e_lsb }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word8 {
    pub cus: u16,

    /// MSB(8) A⁻¹: you need to associate this to Subframe #2 Word #9
    pub sqrt_a_msb: u8,
}

impl GpsSubframe2Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let cus = ((dword & WORD8_CUS_MASK) >> WORD8_CUS_SHIFT) as u16;
        let sqrt_a_msb = ((dword & WORD8_SQRTA_MSB_MASK) >> WORD8_SQRTA_MSB_SHIFT) as u8;

        Self { cus, sqrt_a_msb }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word9 {
    /// LSB(24) A⁻¹: you need to associate this to Subframe #2 Word #8
    pub sqrt_a_lsb: u32,
}

impl GpsSubframe2Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let sqrt_a_lsb = ((dword & WORD9_SQRTA_LSB_MASK) >> WORD9_SQRTA_LSB_SHIFT) as u32;

        Self { sqrt_a_lsb }
    }
}

#[derive(Debug, Clone)]
pub struct GpsSubframe2Word10 {
    /// Time of issue of Ephemeris (u16)
    pub toe: u16,

    pub fitint: bool,
    pub aodo: u8,
}

impl GpsSubframe2Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let toe = ((dword & WORD10_TOE_MASK) >> WORD10_TOE_SHIFT) as u16;

        let fitint = (dword & WORD10_FITINT_MASK) > 0;
        let aodo = ((dword & WORD10_AODO_MASK) >> WORD10_AODO_SHIFT) as u8;

        Self { toe, fitint, aodo }
    }
}
