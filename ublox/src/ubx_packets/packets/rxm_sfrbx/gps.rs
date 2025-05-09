// Preamble is masked out by UBX
const GPS_TLM_PREAMBLE_MASK: u32 = 0xff000000;
const GPS_TLM_PREAMBLE_SHIFT: u32 = 24;
const GPS_TLM_MESSAGE_MASK: u32 = 0x00fffc00;
const GPS_TLM_MESSAGE_SHIFT: u32 = 8;
const GPS_TLM_INTEGRITY_BIT_MASK: u32 = 0x00000200;
const GPS_TLM_RESERVED_BIT_MASK: u32 = 0x00000100;
const GPS_TLM_PARITY_MASK: u32 = 0x000000ff;

const GPS_HOW_TOW_MASK: u32 = 0xffff8000;
const GPS_HOW_TOW_SHIFT: u32 = 13;
const GPS_HOW_ALERT_BIT_MASK: u32 = 0x00004000;
const GPS_HOW_ANTI_SPOOFING_BIT_MASK: u32 = 0x00002000;
const GPS_HOW_FRAME_ID_MASK: u32 = 0x00001c00;
const GPS_HOW_FRAME_ID_SHIFT: u32 = 10;

/// [GpsTelemetryWord] marks the beginning of each frame
#[derive(Debug, Clone)]
/// [GpsTelemetryWord]
pub struct GpsTelemetryWord {
    /// Preamble data bits. Should be constant. Ublox seems to mask some of them ?
    pub preamble: u8,

    /// TLM Message
    pub tlm_message: u16,

    /// Integrity bit is asserted means the conveying signal is provided
    /// with an enhanced level of integrity assurance.
    pub integrity: bool,

    /// Reserved bit
    pub reserved: bool,

    /// Parity bits
    pub parity: u8,
}

impl GpsTelemetryWord {
    pub fn decode(dword: u32) -> Option<Self> {
        let preamble = ((dword & GPS_TLM_PREAMBLE_MASK) >> GPS_TLM_PREAMBLE_SHIFT) as u8;
        let tlm_message = ((dword & GPS_TLM_MESSAGE_MASK) >> GPS_TLM_MESSAGE_SHIFT) as u16;
        let integrity = (dword & GPS_TLM_INTEGRITY_BIT_MASK) > 0;
        let reserved = (dword & GPS_TLM_INTEGRITY_BIT_MASK) > 0;
        let parity = (dword & GPS_TLM_PARITY_MASK) as u8;

        Some(Self {
            preamble,
            tlm_message,
            integrity,
            reserved,
            parity,
        })
    }
}

/// [GpsHowWord] marks the beginning of each frame, following [GpsTelemetryWord]
#[derive(Debug, Clone)]
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
    pub fn decode(dword: u32) -> Option<Self> {
        let tow = (dword & GPS_HOW_TOW_MASK) >> GPS_HOW_TOW_SHIFT;

        let alert = (dword & GPS_HOW_ALERT_BIT_MASK) > 0;
        let anti_spoofing = (dword & GPS_HOW_ANTI_SPOOFING_BIT_MASK) > 0;
        let frame_id = ((dword & GPS_HOW_FRAME_ID_MASK) >> GPS_HOW_FRAME_ID_SHIFT) as u8;

        Some(Self {
            tow,
            alert,
            frame_id,
            anti_spoofing,
        })
    }
}

#[derive(Debug, Clone)]
pub enum GpsDataWord {
    /// [GpsDataWord::Telemetry] is a marks the beginning of each frame.
    Telemetry(GpsTelemetryWord),

    /// [GpsDataWord::How] marks the beginning of each frame, following [GpsDataWord::Telemetry].
    How(GpsHowWord),
}
