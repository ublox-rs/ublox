// Preamble is masked out by UBX
// const GPS_PREAMBLE_MSB_MASK: u32 = 0xff000000;
// const GPS_PREAMBLE_OK_MASK: u32 = 0x8b000000;

const GPS_HOW_TOW_MASK: u32 = 0xffff1000;
const GPS_HOW_ALERT_BIT_MASK: u32 = 0x00002000;
const GPS_HOW_ANTI_SPOOFING_BIT_MASK: u32 = 0x00004000;
const GPS_HOW_FRAME_ID_MASK: u32 = 0x00008300;

/// [GpsTelemetryWord] marks the beginning of each frame
#[derive(Debug, Clone)]
/// [GpsTelemetryWord]
pub struct GpsTelemetryWord {
    // masked out by UBX
    // pub preamble: u8,
    // pub tlm_message: u16,
    // pub integrity: u8,
    // pub reserved: u8,
    // pub parity: u8,
}

impl GpsTelemetryWord {
    pub fn decode(dword: u32) -> Option<Self> {
        // let preamble = dword & GPS_PREAMBLE_MSB_MASK;

        // if preamble != GPS_PREAMBLE_OK_MASK {
        //     return None;
        // }

        Some(Self {
            // preamble: (preamble >> 24) as u8,
            // tlm_message: 0,
            // integrity: 0,
            // reserved: 0,
            // parity: 0,
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
        let tow = dword & GPS_HOW_TOW_MASK;
        let alert = (dword & GPS_HOW_ALERT_BIT_MASK);
        let anti_spoofing = dword & GPS_HOW_ANTI_SPOOFING_BIT_MASK;
        let frame_id = ((dword & GPS_HOW_FRAME_ID_MASK) >> 18) as u8;

        Some(Self {
            tow,
            alert: alert > 0,
            anti_spoofing: anti_spoofing > 0,
            frame_id,
            // preamble: (preamble >> 24) as u8,
            // tlm_message: 0,
            // integrity: 0,
            // reserved: 0,
            // parity: 0,
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
