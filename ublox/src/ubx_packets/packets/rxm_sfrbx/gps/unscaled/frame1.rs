use super::super::{twos_complement, GPS_PARITY_SIZE};

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

const WORD4_L2P_DATA_MASK: u32 = 0x800000;
const WORD4_RESERVED_MASK: u32 = 0x7fffff;
const WORD4_RESERVED_SHIFT: u32 = 0;

const WORD5_RESERVED_MASK: u32 = 0xffffff;

const WORD6_RESERVED_MASK: u32 = 0xffffff;

const WORD7_RESERVED_MASK: u32 = 0xffff00;
const WORD7_RESERVED_SHIFT: u32 = 8;
const WORD7_TGD_MASK: u32 = 0x0000ff;
const WORD7_TGD_SHIFT: u32 = 0;

const WORD8_IODC_MASK: u32 = 0xff0000;
const WORD8_IODC_SHIFT: u32 = 16;
const WORD8_TOC_MASK: u32 = 0x00ffff;
const WORD8_TOC_SHIFT: u32 = 0;

const WORD9_AF2_MASK: u32 = 0xff0000;
const WORD9_AF2_SHIFT: u32 = 16;
const WORD9_AF1_MASK: u32 = 0x00ffff;
const WORD9_AF1_SHIFT: u32 = 0;

const WORD10_AF0_MASK: u32 = 0x3fffff;
const WORD10_AF0_SHIFT: u32 = 0;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word3 {
    /// 10-bit week counter
    pub week: u16,

    /// 2 bits C/A or P ON L2
    pub ca_or_p_l2: u8,

    /// 4-bit URA index
    pub ura: u8,

    /// 6-bit SV Health
    pub health: u8,

    /// 2-bit (MSB) IODC, you will have to associate this to Word # 8
    pub iodc_msb: u8,
}

impl GpsUnscaledEph1Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;

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
pub struct GpsUnscaledEph1Word4 {
    pub l2_p_data_flag: bool,
    pub reserved: u32,
}

impl GpsUnscaledEph1Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let l2_p_data_flag = (dword & WORD4_L2P_DATA_MASK) > 0;
        let reserved = ((dword & WORD4_RESERVED_MASK) >> WORD4_RESERVED_SHIFT) as u32;
        Self {
            l2_p_data_flag,
            reserved,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word5 {
    /// 24-bit reserved
    pub reserved: u32,
}

impl GpsUnscaledEph1Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let reserved = dword & WORD5_RESERVED_MASK;
        Self { reserved }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word6 {
    /// 24-bit reserved
    pub reserved: u32,
}

impl GpsUnscaledEph1Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let reserved = dword & WORD6_RESERVED_MASK;
        Self { reserved }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word7 {
    /// 16-bit reserved
    pub reserved: u16,

    /// TGD
    pub tgd: i8,
}

impl GpsUnscaledEph1Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let reserved = ((dword & WORD7_RESERVED_MASK) >> WORD7_RESERVED_SHIFT) as u16;
        let tgd = ((dword & WORD7_TGD_MASK) >> WORD7_TGD_SHIFT) as i8;
        Self { reserved, tgd }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word8 {
    /// 8-bit IODC LSB to associate with Word # 3
    pub iodc_lsb: u8,

    /// 16 bit ToC
    pub toc: u16,
}

impl GpsUnscaledEph1Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let iodc_lsb = ((dword & WORD8_IODC_MASK) >> WORD8_IODC_SHIFT) as u8;
        let toc = ((dword & WORD8_TOC_MASK) >> WORD8_TOC_SHIFT) as u16;
        Self { iodc_lsb, toc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word9 {
    /// 8 bit af2
    pub af2: i8,

    /// 16 bit af1
    pub af1: i16,
}

impl GpsUnscaledEph1Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let af2 = ((dword & WORD9_AF2_MASK) >> WORD9_AF2_SHIFT) as i8;
        let af1 = ((dword & WORD9_AF1_MASK) >> WORD9_AF1_SHIFT) as i16;
        Self { af2, af1 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph1Word10 {
    /// 22-bit af0
    pub af0: i32,
}

impl GpsUnscaledEph1Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);
        let af0 = ((dword & WORD10_AF0_MASK) >> WORD10_AF0_SHIFT) as u32;
        let af0 = twos_complement(af0, 0x3fffff, 22);
        Self { af0 }
    }
}
