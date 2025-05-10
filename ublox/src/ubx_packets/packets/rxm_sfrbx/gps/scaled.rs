use super::{GpsHowWord, GpsTelemetryWord};

/// Interprated [GpsSubframe]s
#[derive(Debug, Clone)]
pub enum GpsSubframe {
    /// GPS - [GpsSubframe1]
    Subframe1(GpsSubframe1),

    /// GPS - [GpsSubframe2]
    Subframe2(GpsSubframe2),

    /// GPS - [GpsSubframe3]
    Subframe3(GpsSubframe3),

    /// GPS - paginated (use internal page id) [GpsSubframe4]
    Subframe4(GpsSubframe4),

    /// GPS - paginated (use internal page id) [GpsSubframe5]
    Subframe5(GpsSubframe5),
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

    /// 2-bit C/A or P ON L2.  
    /// When asserted, indicates the NAV data stream was commanded OFF on the L2 channel P-code.
    pub ca_or_p_l2: u8,

    /// 4-bit URA index. The lower the better, interprate as follow (error in meters)
    /// - 0:  0 < ura <= 2.4m
    /// - 1:  2.4 < ura <= 3.4m
    /// - 2:  3.4 < ura <= 4.85
    /// - 3:  4.85 < ura <= 6.85
    /// - 4:  6.85 < ura <= 9.65
    /// - 5:  9.65 < ura <= 13.65
    /// - 6:  13.65 < ura <= 24.00
    /// - 7:  24.00 < ura <= 48.00
    /// - 8:  48.00 < ura <= 96.00
    /// - 9:  96.00 < ura <= 192.00
    /// - 10: 192.00 < ura <=  384.00
    /// - 11: 384.00 < ura <=  768.00
    /// - 12: 768.00 < ura <= 1536.00
    /// - 13: 1536.00 < ura <= 3072.00
    /// - 14: 3072.00 < ura <= 6144.00
    /// - 15: 6144.00 < ura
    ///
    /// For each URA index, users may compute a nominal URA value (x)
    ///  - ura < 6: 2**(1+N/2)
    ///  - ura > 6: 2**(N-2)
    pub ura: u8,

    /// 6-bit SV Health. 0 means all good.
    pub health: u8,

    /// 10-bit IODC.  
    /// IODC indicates the issue number of the data set and provides the user
    /// with a convenient means of detecting any change in the correction parameters.
    pub iodc: u16,

    /// TOC in seconds (elapsed within week)
    pub toc: u32,

    /// 8-bit TGD (in seconds)
    pub tgd: f64,

    /// af2 (s.s⁻²)
    pub af2: f64,

    /// af1 (s.s⁻1)
    pub af1: f64,

    /// af0 (s)
    pub af0: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe2 {
    /// IODE: Issue of Data (Ephemeris)
    pub iode: u8,

    /// Mean anomaly at reference time
    pub m0: f64,

    /// Mean motion difference from computed value
    pub delta_n: f64,

    /// Latitude cosine harmonic correction term
    pub cuc: f64,

    /// Orbit radius sine harmonic correction term
    pub crs: f64,

    /// Eccentricity
    pub e: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe3 {
    /// Inclination angle sine harmonic correction term
    pub cis: f64,

    /// Inclination angle at reference time
    pub i0: f64,

    /// Inclination angle cosine harmonic correction term
    pub cic: f64,

    /// Longitude of ascending node of orbit plane at weekly epoch
    pub omega0: f64,

    /// Orbit radius cosine harmonic correction term
    pub crc: f64,

    /// IODE: Issue of Data (Ephemeris)
    pub iode: u8,

    /// Rate of inclination angle
    pub idot: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe4 {
    /// Eccentricity
    pub e: u16,

    pub toa: u8,

    pub delta_i: u16,
    pub p_dot: u16,
    pub sv_health: u8,
    pub sqrt_a: u32,
    pub omega0: u32,

    /// Argument of perigee
    pub omega: u32,

    /// Mean anomaly at Reference Time
    pub m0: u32,

    /// af0 (s)
    pub af0: u16,

    /// af1 (s.s⁻¹)
    pub af1: u16,
}

#[derive(Debug, Default, Clone)]
pub struct GpsSubframe5 {
    /// 2-bit data ID
    pub data_id: u8,

    /// 6-bit SV ID
    pub sv_id: u8,

    /// eccentricity
    pub e: f64,

    /// p_dot
    pub p_dot: f64,

    pub sv_health: u8,

    pub sqrt_a: f64,
    pub omega0: f64,
    pub omega: f64,
    pub m0: f64,
    pub af0: f64,
    pub af1: f64,
}
