use super::GPS_PARITY_SIZE;

const WORD3_WEEK_MASK: u32 = 0xffC000;
const WORD3_WEEK_SHIFT: u32 = 14; // remaining payload bits
const WORD3_CA_P_L2_MASK: u32 = 0x003000;
const WORD3_CA_P_L2_SHIFT: u32 = 12;
const WORD3_URA_MASK: u32 = 0x000f00;
const WORD3_URA_SHIFT: u32 = 8;
const WORD3_HEALTH_MASK: u32 = 0x0000fc;
const WORD3_HEALTH_SHIFT: u32 = 2;
const WORD3_IODC_MASK: u32 = 0x000003;
const WORD3_IODC_SHIFT: u32 = 0;

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

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word3 {
    // 10-bit week counter
    pub week: u16,

    // 2 bits C/A or P ON L2
    pub ca_or_p_l2: u8,

    // 4-bit URA index
    pub ura: u8,

    // 6-bit SV Health
    pub health: u8,

    // 2-bit (MSB) IODC, you will have to associate this to Word #4
    pub iodc_msb: u8,
}

impl GpsSubframe1Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let week = ((dword & WORD3_WEEK_MASK) >> WORD3_WEEK_SHIFT) as u16;
        let ca_or_p_l2 = ((dword & WORD3_CA_P_L2_MASK) >> WORD3_CA_P_L2_SHIFT) as u8;
        let ura = ((dword & WORD3_URA_MASK) >> WORD3_URA_SHIFT) as u8;
        let health = ((dword & WORD3_HEALTH_MASK) >> WORD3_HEALTH_SHIFT) as u8;
        let iodc_msb = ((dword & WORD3_IODC_MASK) >> WORD3_IODC_SHIFT) as u8;

        Self {
            week,
            ca_or_p_l2,
            ura,
            health,
            iodc_msb,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word4 {
    pub delta_n: u16,

    /// M0 (8) msb, you need to associate this to Subframe #2 Word #5
    pub m0_msb: u8,
}

impl GpsSubframe1Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let delta_n = ((dword & WORD4_DELTAN_MASK) >> WORD4_DELTAN_SHIFT) as u16;
        let m0_msb = ((dword & WORD4_M0_MSB_MASK) >> WORD4_M0_MSB_SHIFT) as u8;

        Self { delta_n, m0_msb }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word5 {
    /// M0 (24) lsb, you need to associate this to Subframe #2 Word #4
    pub m0_lsb: u32,
}

impl GpsSubframe1Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);
        let m0_lsb = ((dword & WORD5_M0_LSB_MASK) >> WORD5_M0_LSB_SHIFT) as u32;

        Self { m0_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word6 {
    pub cuc: u16,

    /// MSB(8) eccentricity, you need to associate this to Subframe #2 Word #7
    pub e_msb: u8,
}

impl GpsSubframe1Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let cuc = ((dword & WORD6_CUC_MASK) >> WORD6_CUC_SHIFT) as u16;
        let e_msb = ((dword & WORD6_E_MSB_MASK) >> WORD6_E_MSB_SHIFT) as u8;

        Self { cuc, e_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word7 {
    /// LSB(24) eccentricity, you need to associate this to Subframe #2 Word #6
    pub e_lsb: u32,
}

impl GpsSubframe1Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let e_lsb = ((dword & WORD7_E_LSB_MASK) >> WORD7_E_LSB_SHIFT) as u32;

        Self { e_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word8 {
    pub cus: u16,

    /// MSB(8) A⁻¹: you need to associate this to Subframe #2 Word #9
    pub sqrt_a_msb: u8,
}

impl GpsSubframe1Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let cus = ((dword & WORD8_CUS_MASK) >> WORD8_CUS_SHIFT) as u16;
        let sqrt_a_msb = ((dword & WORD8_SQRTA_MSB_MASK) >> WORD8_SQRTA_MSB_SHIFT) as u8;

        Self { cus, sqrt_a_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word9 {
    /// LSB(24) A⁻¹: you need to associate this to Subframe #2 Word #8
    pub sqrt_a_lsb: u32,
}

impl GpsSubframe1Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let sqrt_a_lsb = ((dword & WORD9_SQRTA_LSB_MASK) >> WORD9_SQRTA_LSB_SHIFT) as u32;

        Self { sqrt_a_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1Word10 {
    /// Time of issue of Ephemeris (u16)
    pub toe: u16,

    pub fitint: bool,
    pub aodo: u8,
}

impl GpsSubframe1Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let toe = ((dword & WORD10_TOE_MASK) >> WORD10_TOE_SHIFT) as u16;

        let fitint = (dword & WORD10_FITINT_MASK) > 0;
        let aodo = ((dword & WORD10_AODO_MASK) >> WORD10_AODO_SHIFT) as u8;

        Self { toe, fitint, aodo }
    }
}
