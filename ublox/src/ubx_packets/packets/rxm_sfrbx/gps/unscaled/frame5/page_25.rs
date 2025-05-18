/// Interpretation that prevails for Frame #5 page 25 (last)
use super::super::super::GPS_PARITY_SIZE;

const WORD3_DID_MASK: u32 = 0xc00000;
const WORD3_DID_SHIFT: u32 = 22;
const WORD3_SID_MASK: u32 = 0x3f0000;
const WORD3_SID_SHIFT: u32 = 16;
const WORD3_TOA_MASK: u32 = 0x00ff00;
const WORD3_TOA_SHIFT: u32 = 8;
const WORD3_WNA_MASK: u32 = 0x0000ff;
const WORD3_WNA_SHIFT: u32 = 0;

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page25Word3 {
    /// 2-bit data id
    pub data_id: u8,
    /// 6-bit page ID
    pub page_id: u8,
    /// ToA
    pub toa: u8,
    /// Week
    pub wna: u8
}

impl GpsUnscaled5Page25Word3 {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let data_id = ((dword & WORD3_DID_MASK) >> WORD3_DID_SHIFT) as u8;
        let page_id = ((dword & WORD3_SID_MASK) >> WORD3_SID_SHIFT) as u8;
        let toa = ((dword & WORD3_TOA_MASK) >> WORD3_TOA_SHIFT) as u8;
        let wna = ((dword & WORD3_WNA_MASK) >> WORD3_WNA_SHIFT) as u8;
        Self { data_id, page_id, toa, wna }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsUnscaled5Page25HealthWord {
    pub sv_1msb_health: u8,
    pub sv_2_health: u8,
    pub sv_3_health: u8,
    pub sv_4lsb_health: u8,
}

impl GpsUnscaled5Page25HealthWord {
    pub(crate) fn decode(dword: u32) -> Self {
        let dword = dword >> GPS_PARITY_SIZE;
        let sv_1msb_health = ((dword & 0xfc0000) >> 14) as u8;
        let sv_2_health = ((dword & 0x03f000) >> 8) as u8;
        let sv_3_health = ((dword & 0x000fc0) >> 6) as u8;
        let sv_4lsb_health = (dword & 0x00003f) as u8;

        Self {
            sv_1msb_health,
            sv_2_health,
            sv_3_health,
            sv_4lsb_health,
        }
    }
}
