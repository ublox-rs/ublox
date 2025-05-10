//////////////////////////////////////////////////////////////
// NB(1): Parity bits are always truncated by the UBX firmware
// See UBX-AID section of the UBX docs
//////////////////////////////////////////////////////////////
mod frame1;
mod frame2;
mod frame3;

pub use frame1::*;
pub use frame2::*;
pub use frame3::*;

const GPS_PARITY_SIZE: u32 = 6;

const GPS_TLM_PREAMBLE_MASK: u32 = 0x8b0000;
const GPS_TLM_MESSAGE_MASK: u32 = 0x00fff8;
const GPS_TLM_MESSAGE_SHIFT: u32 = 3;
const GPS_TLM_INTEGRITY_BIT_MASK: u32 = 0x000004;
const GPS_TLM_RESERVED_BIT_MASK: u32 = 0x000002;

const GPS_HOW_TOW_MASK: u32 = 0xffff80;
const GPS_HOW_TOW_SHIFT: u32 = 5; // remaining payload bits
const GPS_HOW_ALERT_BIT_MASK: u32 = 0x000040;
const GPS_HOW_ANTI_SPOOFING_BIT_MASK: u32 = 0x000020;
const GPS_HOW_FRAME_ID_MASK: u32 = 0x00001C;
const GPS_HOW_FRAME_ID_SHIFT: u32 = 2;

/// [GpsTelemetryWord] marks the beginning of each frame
#[derive(Debug, Default, Clone)]
/// [GpsTelemetryWord]
pub struct GpsTelemetryWord {
    /// TLM Message
    pub tlm_message: u16,

    /// Integrity bit is asserted means the conveying signal is provided
    /// with an enhanced level of integrity assurance.
    pub integrity: bool,

    /// Reserved bit
    pub reserved: bool,
}

impl GpsTelemetryWord {
    pub(crate) fn decode(dword: u32) -> Option<Self> {
        let dword = dword >> GPS_PARITY_SIZE;

        // preamble verification
        if dword & GPS_TLM_PREAMBLE_MASK == 0 {
            // invalid GPS frame
            return None;
        }

        let tlm_message = ((dword & GPS_TLM_MESSAGE_MASK) >> GPS_TLM_MESSAGE_SHIFT) as u16;
        let integrity = (dword & GPS_TLM_INTEGRITY_BIT_MASK) > 0;
        let reserved = (dword & GPS_TLM_RESERVED_BIT_MASK) > 0;

        Some(Self {
            tlm_message,
            integrity,
            reserved,
        })
    }
}

/// [GpsHowWord] marks the beginning of each frame, following [GpsTelemetryWord]
#[derive(Debug, Default, Clone)]
/// [GpsHowWord]
pub struct GpsHowWord {
    /// Elapsed seconds in GPST week
    pub tow: u32,

    /// When alert is asserted, the SV URA may be worse than indicated in subframe 1
    /// and user shall use this SV at their own risk.
    pub alert: bool,

    /// A-S mode is ON in that SV
    pub anti_spoofing: bool,

    /// Following Frame ID (to decoding following data words)
    pub frame_id: u8,
}

impl GpsHowWord {
    pub(crate) fn decode(dword: u32) -> Self {
        // yet another custom shift..
        let dword = dword >> (GPS_PARITY_SIZE + 2);

        let tow = (dword & GPS_HOW_TOW_MASK) >> GPS_HOW_TOW_SHIFT;

        let alert = (dword & GPS_HOW_ALERT_BIT_MASK) > 0;
        let anti_spoofing = (dword & GPS_HOW_ANTI_SPOOFING_BIT_MASK) > 0;

        let frame_id = ((dword & GPS_HOW_FRAME_ID_MASK) >> GPS_HOW_FRAME_ID_SHIFT) as u8;

        Self {
            tow,
            alert,
            frame_id,
            anti_spoofing,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1 {
    pub word3: GpsSubframe1Word3,
    pub word4: GpsSubframe1Word4,
    pub word5: GpsSubframe1Word5,
    pub word6: GpsSubframe1Word6,
    pub word7: GpsSubframe1Word7,
    pub word8: GpsSubframe1Word8,
    pub word9: GpsSubframe1Word9,
    pub word10: GpsSubframe1Word10,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe2 {
    pub word3: GpsSubframe2Word3,
    pub word4: GpsSubframe2Word4,
    pub word5: GpsSubframe2Word5,
    pub word6: GpsSubframe2Word6,
    pub word7: GpsSubframe2Word7,
    pub word8: GpsSubframe2Word8,
    pub word9: GpsSubframe2Word9,
    pub word10: GpsSubframe2Word10,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe3 {
    pub word3: GpsSubframe3Word3,
    pub word4: GpsSubframe3Word4,
    pub word5: GpsSubframe3Word5,
    pub word6: GpsSubframe3Word6,
    pub word7: GpsSubframe3Word7,
    pub word8: GpsSubframe3Word8,
    pub word9: GpsSubframe3Word9,
    pub word10: GpsSubframe3Word10,
}

/// Interprated [GpsSubframe]s
#[derive(Debug, Clone)]
pub enum GpsSubframe {
    /// GPS - Subframe #1
    Subframe1(GpsSubframe1),

    /// GPS - Subframe #2
    Subframe2(GpsSubframe2),

    /// GPS - Subframe #3
    Subframe3(GpsSubframe3),
}

impl Default for GpsSubframe {
    fn default() -> Self {
        Self::Subframe1(Default::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsFrame {
    pub telemetry: GpsTelemetryWord,
    pub how: GpsHowWord,
    pub subframe: GpsSubframe,
}
