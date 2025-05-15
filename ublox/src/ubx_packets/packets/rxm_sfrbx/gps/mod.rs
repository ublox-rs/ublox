//! GPS / QZSS frames

//////////////////////////////////////////////////////////////
// NB(1): Parity bits are always truncated by the UBX firmware
// See UBX-AID section of the UBX docs
//////////////////////////////////////////////////////////////
pub(crate) mod unscaled;
pub(crate) use unscaled::*;

pub mod scaled;
pub use scaled::*;

const GPS_BITMASK: u32 = 0x3fffffff;
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

/// Grab GPS (and QZSS) frame bits
pub(crate) fn gps_qzss_bitmask(dword: u32) -> u32 {
    // 2-MSB Padding (30->32bits) and stripped 6 LSB parity
    (dword & GPS_BITMASK) >> GPS_PARITY_SIZE
}

/// Two's complement parsing & interpretation.
/// ## Input
/// - raw bytes as [u32]
/// - bits_mask: masking u32
/// - sign_bit_mask: sign bit
pub(crate) fn twos_complement(value: u32, bits_mask: u32, sign_bit_mask: u32) -> i32 {
    let value = value & bits_mask;

    let signed = (value & sign_bit_mask) > 0;

    if signed {
        (value | !bits_mask) as i32
    } else {
        value as i32
    }
}

/// [RxmSfrbxGpsQzssTelemetry] marks the beginning of each frame
#[derive(Debug, Default, Clone)]
pub struct RxmSfrbxGpsQzssTelemetry {
    /// TLM Message
    pub tlm_message: u16,

    /// Integrity bit is asserted means the conveying signal is provided
    /// with an enhanced level of integrity assurance.
    pub integrity: bool,

    /// Reserved bit
    pub reserved: bool,
}

impl RxmSfrbxGpsQzssTelemetry {
    pub(crate) fn decode(dword: u32) -> Option<Self> {
        let dword = gps_qzss_bitmask(dword);

        // preamble verification
        if dword & GPS_TLM_PREAMBLE_MASK == GPS_TLM_PREAMBLE_MASK {
            let tlm_message = ((dword & GPS_TLM_MESSAGE_MASK) >> GPS_TLM_MESSAGE_SHIFT) as u16;
            let integrity = (dword & GPS_TLM_INTEGRITY_BIT_MASK) > 0;
            let reserved = (dword & GPS_TLM_RESERVED_BIT_MASK) > 0;

            Some(Self {
                tlm_message,
                integrity,
                reserved,
            })
        } else {
            // Invalid frame
            None
        }
    }
}

/// [RxmSfrbxGpsQzssHow] marks the beginning of each frame, following [RxmSfrbxGpsTelemetry]
#[derive(Debug, Default, Clone)]
/// [GpsHowWord]
pub struct RxmSfrbxGpsQzssHow {
    /// TOW
    pub tow: u32,

    /// Following Frame ID (to decoding following data words)
    pub frame_id: u8,

    /// When alert is asserted, the SV URA may be worse than indicated in subframe 1
    /// and user shall use this SV at their own risk.
    pub alert: bool,

    /// A-S mode is ON in that SV
    pub anti_spoofing: bool,
}

impl RxmSfrbxGpsQzssHow {
    pub(crate) fn decode(dword: u32) -> Self {
        // strip two more bits here
        let dword = gps_qzss_bitmask(dword) >> 2;

        let frame_id = (dword & 0x7) as u8;
        let anti_spoofing = (dword & 0x08) > 0;
        let alert = (dword & 0x10) > 0;
        let tow = ((dword >> 5) & 0x1ffff) * 6;

        Self {
            tow,
            alert,
            frame_id,
            anti_spoofing,
        }
    }
}

#[cfg(test)]
mod test {
    use super::twos_complement;

    #[test]
    fn test_twos_complement() {
        let value = 0x3fff;
        let parsed = twos_complement(value, 0x3fff, 0x2000);
        assert_eq!(parsed, 0xffffffffu32 as i32);
    }
}
