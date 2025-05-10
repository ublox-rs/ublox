use super::{GpsHowWord, GpsTelemetryWord};

/// Interprated [GpsSubframe]s
#[derive(Debug, Clone)]
pub enum GpsSubframe {
    /// GPS - [GpsSubframe1]
    Subframe1(GpsSubframe1),

    /// GPS - [Gpsubframe2]
    Subframe2(GpsSubframe2),

    /// GPS - [Gpsubframe3]
    Subframe3(GpsSubframe3),
}

impl Default for GpsSubframe {
    fn default() -> Self {
        Self::Subframe1(Default::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpsFrame {
    /// [GpsTelemetryWord]
    pub telemetry: GpsTelemetryWord,

    /// [GpsHowWord]
    pub how: GpsHowWord,

    /// [GpsSubframe]
    pub subframe: GpsSubframe,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe1 {
    /// 10-bit week counter
    pub week: u16,

    /// TOE in seconds (elapsed within week)
    pub toe: u32,

    /// 2 bits C/A or P ON L2
    pub ca_or_p_l2: u8,

    /// 4-bit URA index
    pub ura: u8,

    /// 6-bit SV Health
    pub health: u8,

    pub delta_n: f64,
    pub m0: f64,
    pub cuc: f64,
    pub e: f64,
    pub cus: f64,
    pub sqrt_a: f64,

    pub fitint: bool,
    pub aodo: u8,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe2 {
    pub iode: u8,
    pub crs: f64,
    pub delta_n: f64,
    pub m0: f64,
    pub cuc: f64,
    pub e: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe3 {
    pub cis: f64,
    pub i0: f64,
    pub cic: f64,
    pub omega0: f64,
    pub crc: f64,
    pub iode: f64,
    pub idot: f64,
}
