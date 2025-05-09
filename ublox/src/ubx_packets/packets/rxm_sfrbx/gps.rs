// Preamble is masked out by UBX
const GPS_PREAMBLE_MASK: u32 = 0xff000000;
const GPS_PREAMBLE_SHIFT: u32 = 24;
const GPS_TLM_MESSAGE_MASK: u32 = 0x00fffc00;
const GPS_TLM_MESSAGE_SHIFT: u32 = 8;

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
    pub preamble: u8,
    pub tlm_message: u16,
    // pub integrity: u8,
    // pub reserved: u8,
    // pub parity: u8,
}

impl GpsTelemetryWord {
    pub fn decode(dword: u32) -> Option<Self> {
        let preamble = ((dword & GPS_PREAMBLE_MASK) >> GPS_PREAMBLE_SHIFT) as u8;
        let tlm_message = ((dword & GPS_TLM_MESSAGE_MASK) >> GPS_TLM_MESSAGE_SHIFT) as u16;

        Some(Self {
            preamble,
            tlm_message,
        })
    }
}

/// [GpsHowWord] marks the beginning of each frame, following [GpsTelemetryWord]
#[derive(Debug, Clone)]
/// [GpsHowWord]
pub struct GpsHowWord {
    pub tow: u32,
    pub alert: bool,
    pub anti_spoofing: bool,
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
