use super::super::{gps_qzss_bitmask, twos_complement};

const WORD3_CIC_MASK: u32 = 0xffff00;
const WORD3_CIC_SHIFT: u32 = 8; // remaining payload bits
const WORD3_OMEGA0_MASK: u32 = 0x0000ff;
const WORD3_OMEGA0_SHIFT: u32 = 0;

const WORD4_OMEGA0_MASK: u32 = 0xffffff;
const WORD4_OMEGA0_SHIFT: u32 = 0;

const WORD5_CIS_MASK: u32 = 0xffff00;
const WORD5_CIS_SHIFT: u32 = 8;
const WORD5_I0_MASK: u32 = 0x0000ff;
const WORD5_I0_SHIFT: u32 = 0;

const WORD6_I0_MASK: u32 = 0x00ffffff;
const WORD6_I0_SHIFT: u32 = 0;

const WORD7_CRC_MASK: u32 = 0xffff00;
const WORD7_CRC_SHIFT: u32 = 8;
const WORD7_OMEGA_MASK: u32 = 0xff;
const WORD7_OMEGA_SHIFT: u32 = 0;

const WORD8_OMEGA_MASK: u32 = 0xffffff;
const WORD8_OMEGA_SHIFT: u32 = 0;

const WORD9_OMEGADOT_MASK: u32 = 0xffffff;
const WORD9_OMEGADOT_SHIFT: u32 = 0;

const WORD10_IODE_MASK: u32 = 0x3fc000;
const WORD10_IODE_SHIFT: u32 = 14;
const WORD10_IDOT_MASK: u32 = 0x003fff;
const WORD10_IDOT_SHIFT: u32 = 0;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word3 {
    pub cic: i16,

    /// Omega0 (8) MSB, you will have to associate this to Word #4
    pub omega0_msb: u8,
}

impl GpsUnscaledEph3Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let cic = ((dword & WORD3_CIC_MASK) >> WORD3_CIC_SHIFT) as i16;
        let omega0_msb = ((dword & WORD3_OMEGA0_MASK) >> WORD3_OMEGA0_SHIFT) as u8;
        Self { cic, omega0_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word4 {
    /// Omega0 (24) LSB, you will have to associate this to Word #3
    pub omega0_lsb: u32,
}

impl GpsUnscaledEph3Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let omega0_lsb = ((dword & WORD4_OMEGA0_MASK) >> WORD4_OMEGA0_SHIFT) as u32;
        Self { omega0_lsb }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word5 {
    pub cis: i16,

    /// I0 (8) MSB, you will have to associate this to Word #6
    pub i0_msb: u8,
}

impl GpsUnscaledEph3Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let cis = ((dword & WORD5_CIS_MASK) >> WORD5_CIS_SHIFT) as i16;
        let i0_msb = ((dword & WORD5_I0_MASK) >> WORD5_I0_SHIFT) as u8;
        Self { cis, i0_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word6 {
    /// I0 (24) LSB, you will have to associate this to Word #5
    pub i0_lsb: u32,
}

impl GpsUnscaledEph3Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let i0_lsb = ((dword & WORD6_I0_MASK) >> WORD6_I0_SHIFT) as u32;
        Self { i0_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word7 {
    pub crc: i16,

    /// Omega (8) MSB, you will have to associate this to Word #8
    pub omega_msb: u8,
}

impl GpsUnscaledEph3Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let crc = ((dword & WORD7_CRC_MASK) >> WORD7_CRC_SHIFT) as i16;
        let omega_msb = ((dword & WORD7_OMEGA_MASK) >> WORD7_OMEGA_SHIFT) as u8;
        Self { crc, omega_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word8 {
    /// Omega (24) LSB, you will have to associate this to Word #7
    pub omega_lsb: u32,
}

impl GpsUnscaledEph3Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let omega_lsb = ((dword & WORD8_OMEGA_MASK) >> WORD8_OMEGA_SHIFT) as u32;
        Self { omega_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word9 {
    // 24-bit Omega_dot
    pub omega_dot: i32,
}

impl GpsUnscaledEph3Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword);
        let omega_dot = ((dword & WORD9_OMEGADOT_MASK) >> WORD9_OMEGADOT_SHIFT) as u32;
        let omega_dot = twos_complement(omega_dot, 0xffffff, 0x800000);
        Self { omega_dot }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaledEph3Word10 {
    /// 8-bit IODE
    pub iode: u8,

    /// 14-bit IDOT
    pub idot: i32,
}

impl GpsUnscaledEph3Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = gps_qzss_bitmask(dword) >> 2;
        let iode = ((dword & WORD10_IODE_MASK) >> WORD10_IODE_SHIFT) as u8;

        // 14-bit signed 2's
        let idot = ((dword & WORD10_IDOT_MASK) >> WORD10_IDOT_SHIFT) as u32;
        let idot = twos_complement(idot, 0x3fff, 0x2000);

        Self { iode, idot }
    }
}
