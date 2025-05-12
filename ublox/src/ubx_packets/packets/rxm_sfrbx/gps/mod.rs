//////////////////////////////////////////////////////////////
// NB(1): Parity bits are always truncated by the UBX firmware
// See UBX-AID section of the UBX docs
//////////////////////////////////////////////////////////////
pub(crate) mod unscaled;
pub(crate) use unscaled::*;

pub mod scaled;
pub use scaled::*;

const GPS_PARITY_SIZE: u32 = 6;

const GPS_TLM_PREAMBLE_MASK: u32 = 0x8b0000;
const GPS_TLM_MESSAGE_MASK: u32 = 0x00fff8;
const GPS_TLM_MESSAGE_SHIFT: u32 = 3;
const GPS_TLM_INTEGRITY_BIT_MASK: u32 = 0x000004;
const GPS_TLM_RESERVED_BIT_MASK: u32 = 0x000002;

const GPS_HOW_TOW_MASK: u32 = 0x3fffe0;
const GPS_HOW_TOW_SHIFT: u32 = 5; // remaining payload bits
const GPS_HOW_ALERT_BIT_MASK: u32 = 0x000010;
const GPS_HOW_ANTI_SPOOFING_BIT_MASK: u32 = 0x000008;
const GPS_HOW_FRAME_ID_MASK: u32 = 0x000007;
const GPS_HOW_FRAME_ID_SHIFT: u32 = 0;

// const GPS_HOW_TOW_MASK: u32 = 0x0001c0;
// const GPS_HOW_TOW_SHIFT: u32 = 5; // remaining payload bits
// const GPS_HOW_ALERT_BIT_MASK: u32 = 0x000040;
// const GPS_HOW_ANTI_SPOOFING_BIT_MASK: u32 = 0x000020;
// const GPS_HOW_FRAME_ID_MASK: u32 = 0x00001c;
// const GPS_HOW_FRAME_ID_SHIFT: u32 = 2;

pub(crate) fn twos_complement(value: u32, bits_mask: u32, sign_bit_mask: u32) -> i32 {
    let value = value & bits_mask;

    let signed = (value & sign_bit_mask) > 0;

    if signed {
        (value | !bits_mask) as i32
    } else {
        value as i32
    }
}

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
        // stripped parity bits..
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
