use super::super::GPS_PARITY_SIZE;

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

const WORD6_I0_MASK: u32 = 0xffffff;
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
pub struct GpsUnscaled3Word3 {
    pub cic: i16,

    /// Omega0 (8) MSB, you will have to associate this to Word #4
    pub omega0_msb: u8,
}

impl GpsUnscaled3Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let cic = ((dword & WORD3_CIC_MASK) >> WORD3_CIC_SHIFT) as i16;
        let omega0_msb = ((dword & WORD3_OMEGA0_MASK) >> WORD3_OMEGA0_SHIFT) as u8;
        Self { cic, omega0_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word4 {
    /// Omega0 (24) LSB, you will have to associate this to Word #3
    pub omega0_lsb: u32,
}

impl GpsUnscaled3Word4 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega0_lsb = ((dword & WORD4_OMEGA0_MASK) >> WORD4_OMEGA0_SHIFT) as u32;
        Self { omega0_lsb }
    }
}
#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word5 {
    pub cis: i16,

    /// I0 (8) MSB, you will have to associate this to Word #6
    pub i0_msb: u8,
}

impl GpsUnscaled3Word5 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let cis = ((dword & WORD5_CIS_MASK) >> WORD5_CIS_SHIFT) as i16;
        let i0_msb = ((dword & WORD5_I0_MASK) >> WORD5_I0_SHIFT) as u8;
        Self { cis, i0_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word6 {
    /// I0 (24) LSB, you will have to associate this to Word #5
    pub i0_lsb: u32,
}

impl GpsUnscaled3Word6 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let i0_lsb = ((dword & WORD6_I0_MASK) >> WORD6_I0_SHIFT) as u32;
        Self { i0_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word7 {
    pub crc: i16,

    /// Omega (8) MSB, you will have to associate this to Word #8
    pub omega_msb: u8,
}

impl GpsUnscaled3Word7 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let crc = ((dword & WORD7_CRC_MASK) >> WORD7_CRC_SHIFT) as i16;
        let omega_msb = ((dword & WORD7_OMEGA_MASK) >> WORD7_OMEGA_SHIFT) as u8;
        Self { crc, omega_msb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word8 {
    /// Omega (24) LSB, you will have to associate this to Word #7
    pub omega_lsb: u32,
}

impl GpsUnscaled3Word8 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega_lsb = ((dword & WORD8_OMEGA_MASK) >> WORD8_OMEGA_SHIFT) as u32;
        Self { omega_lsb }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word9 {
    // Omega dot (24 bits)
    pub omega_dot: u32,
}

impl GpsUnscaled3Word9 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let omega_dot = ((dword & WORD9_OMEGADOT_MASK) >> WORD9_OMEGADOT_SHIFT) as u32;
        Self { omega_dot }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled3Word10 {
    pub iode: u8,
    pub idot: i16,
}

impl GpsUnscaled3Word10 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> (GPS_PARITY_SIZE + 2);
        let iode = ((dword & WORD10_IODE_MASK) >> WORD10_IODE_SHIFT) as u8;
        let idot = ((dword & WORD10_IDOT_MASK) >> WORD10_IDOT_SHIFT) as i16;
        Self { iode, idot }
    }
}
