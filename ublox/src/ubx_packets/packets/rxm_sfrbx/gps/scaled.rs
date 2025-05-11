use super::{GpsHowWord, GpsTelemetryWord};

/// Interprated [GpsSubframe]s
#[derive(Debug, Clone)]
pub enum GpsSubframe {
    /// GPS Ephemeris Frame #1
    Eph1(GpsEphFrame1),

    /// GPS Ephemeris Frame #2
    Eph2(GpsEphFrame2),

    /// GPS Ephemeris Frame #3
    Eph3(GpsEphFrame3),
    // Almanac frames page 1-24 (not supported yet)
    // /// GPS - Frame #5 Pages 1-24 (included)
    // Subframe5Page1Thru24(GpsSubframe5Page1Thru24),

    // Almanac frames page 25 (not supported yet)
    // Subframe5Page25(GpsSubframe5Page25),
}

impl Default for GpsSubframe {
    fn default() -> Self {
        Self::Eph1(Default::default())
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

/// Frame #1 of all Ephemeris GPS frames.
#[derive(Debug, Default, Clone)]
pub struct GpsEphFrame1 {
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

    /// 24-bit reserved word #5
    pub reserved_word5: u32,

    /// 24-bit reserved word #6
    pub reserved_word6: u32,

    ///16-bit reserved word #7
    pub reserved_word7: u16,
}

/// Frame #2 of all Ephemeris GPS frames.
#[derive(Debug, Default, Clone)]
pub struct GpsEphFrame2 {
    /// Time of issue of ephemeris (in seconds of week)
    pub toe: u32,

    /// IODE: Issue of Data (Ephemeris)
    pub iode: u8,

    /// Mean anomaly at reference time
    pub m0: f64,

    /// Mean motion difference from computed value
    pub delta_n: f64,

    /// Latitude cosine harmonic correction term
    pub cuc: f64,

    /// Latitude sine harmonic correction term
    pub cus: f64,

    /// Orbit radius sine harmonic correction term
    pub crs: f64,

    /// Eccentricity
    pub e: f64,

    /// Sqrt(a)
    pub sqrt_a: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpsEphFrame3 {
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

    /// Omega:
    pub omega: f64,

    /// Omega_dot
    pub omega_dot: f64,
}

// #[derive(Debug, Default, Clone)]
// pub struct GpsSubframe4 {
//     /// Eccentricity
//     pub e: u16,

//     pub toa: u8,

//     pub delta_i: u16,
//     pub p_dot: u16,
//     pub sv_health: u8,
//     pub sqrt_a: u32,
//     pub omega0: u32,

//     /// Argument of perigee
//     pub omega: u32,

//     /// Mean anomaly at Reference Time
//     pub m0: u32,

//     /// af0 (s)
//     pub af0: u16,

//     /// af1 (s.s⁻¹)
//     pub af1: u16,
// }

// /// Interpretation that applies for Pages 1-24 (included) of GPS Frame #5 (Almanac).
// #[derive(Debug, Default, Clone)]
// pub struct GpsSubframe5Page1Thru24 {
//     /// 2-bit data ID
//     pub data_id: u8,

//     /// 6-bit SV ID
//     pub sv_id: u8,

//     /// eccentricity
//     pub e: f64,

//     /// Time of Issue of Almanac (seconds within week)
//     pub toa: u32,

//     /// delta_i
//     pub delta_i: f64,

//     /// Omega_dot
//     pub omega_dot: f64,

//     /// 8-bit SV health
//     pub sv_health: u8,

//     /// sqrt(a)
//     pub sqrt_a: f64,

//     /// omega_0
//     pub omega_0: f64,

//     /// omega
//     pub omega: f64,

//     /// m0
//     pub m0: f64,
// }

// /// Interpretation that applies for Page 25 of GPS Frame #5.
// #[derive(Debug, Default, Clone)]
// pub struct GpsSubframe5Page25 {
//     /// 2-bit data ID
//     pub data_id: u8,

//     /// 6-bit Page ID
//     pub page_id: u8,

//     /// Almanac week counter
//     pub wna: u8,

//     /// Time of Issue of Almanac (seconds)
//     pub toa: u32,

//     /// 6-bit SV (#1) health flags
//     pub sv1_health: u8,
//     /// 6-bit SV (#2) health flags
//     pub sv2_health: u8,
//     /// 6-bit SV (#3) health flags
//     pub sv3_health: u8,
//     /// 6-bit SV (#4) health flags
//     pub sv4_health: u8,
//     /// 6-bit SV (#5) health flags
//     pub sv5_health: u8,
//     /// 6-bit SV (#6) health flags
//     pub sv6_health: u8,
//     /// 6-bit SV (#7) health flags
//     pub sv7_health: u8,
//     /// 6-bit SV (#8) health flags
//     pub sv8_health: u8,
//     /// 6-bit SV (#9) health flags
//     pub sv9_health: u8,
//     /// 6-bit SV (#10) health flags
//     pub sv10_health: u8,
//     /// 6-bit SV (#11) health flags
//     pub sv11_health: u8,
//     /// 6-bit SV (#12) health flags
//     pub sv12_health: u8,
//     /// 6-bit SV (#13) health flags
//     pub sv13_health: u8,
//     /// 6-bit SV (#14) health flags
//     pub sv14_health: u8,
//     /// 6-bit SV (#15) health flags
//     pub sv15_health: u8,
//     /// 6-bit SV (#16) health flags
//     pub sv16_health: u8,
//     /// 6-bit SV (#17) health flags
//     pub sv17_health: u8,
//     /// 6-bit SV (#18) health flags
//     pub sv18_health: u8,
//     /// 6-bit SV (#19) health flags
//     pub sv19_health: u8,
//     /// 6-bit SV (#20) health flags
//     pub sv20_health: u8,
//     /// 6-bit SV (#21) health flags
//     pub sv21_health: u8,
//     /// 6-bit SV (#22) health flags
//     pub sv22_health: u8,
//     /// 6-bit SV (#23) health flags
//     pub sv23_health: u8,
//     /// 6-bit SV (#24) health flags
//     pub sv24_health: u8,
// }
