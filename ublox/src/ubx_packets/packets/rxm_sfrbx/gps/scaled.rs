use super::{RxmSfrbxGpsQzssHow, RxmSfrbxGpsQzssTelemetry};

/// GPS / QZSS Interpreted subframes
#[derive(Debug, Clone)]
pub enum RxmSfrbxGpsQzssSubframe {
    /// GPS Ephemeris Frame #1
    Eph1(RxmSfrbxGpsQzssFrame1),

    /// GPS Ephemeris Frame #2
    Eph2(RxmSfrbxGpsQzssFrame2),

    /// GPS Ephemeris Frame #3
    Eph3(RxmSfrbxGpsQzssFrame3),
}

impl Default for RxmSfrbxGpsQzssSubframe {
    fn default() -> Self {
        Self::Eph1(Default::default())
    }
}

/// GPS / QZSS interpreted frame.
#[derive(Debug, Default, Clone)]
pub struct RxmSfrbxGpsQzssFrame {
    /// [RxmSfrbxGpsQzssTelemetry] describes following frame.
    pub telemetry: RxmSfrbxGpsQzssTelemetry,

    /// [RxmSfrbxGpsQzssHow] describes following frame.
    pub how: RxmSfrbxGpsQzssHow,

    /// [RxmSfrbxGpsQzssSubframe] depends on associated How.
    pub subframe: RxmSfrbxGpsQzssSubframe,
}

/// GPS / QZSS Frame #1 interpretation
#[derive(Debug, Default, Clone)]
pub struct RxmSfrbxGpsQzssFrame1 {
    /// 10-bit week counter (no rollover compensation).
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

    /// Time of clock (s)
    pub toc_s: u32,

    /// 8-bit TGD (in seconds)
    pub tgd_s: f64,

    /// af2 (s.s⁻²)
    pub af2_s_s2: f64,

    /// af1 (s.s⁻1)
    pub af1_s_s: f64,

    /// af0 (s)
    pub af0_s: f64,

    /// 32-bit reserved word #4
    pub reserved_word4: u32,

    pub l2_p_data_flag: bool,

    /// 24-bit reserved word #5
    pub reserved_word5: u32,

    /// 24-bit reserved word #6
    pub reserved_word6: u32,

    ///16-bit reserved word #7
    pub reserved_word7: u16,
}

/// GPS / QZSS Frame #2 interpretation
#[derive(Debug, Default, Clone)]
pub struct RxmSfrbxGpsQzssFrame2 {
    /// Time of issue of ephemeris (s)
    pub toe_s: u32,

    /// IODE: Issue of Data (Ephemeris)
    pub iode: u8,

    /// Mean anomaly at reference time
    pub m0_rad: f64,

    /// Mean motion difference from computed value
    pub dn_rad: f64,

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

    /// Fit interval flag
    pub fit_int_flag: bool,

    /// 5-bit AODO
    pub aodo: u8,
}

/// GPS / QZSS Frame #3 interpretation
#[derive(Debug, Default, Clone)]
pub struct RxmSfrbxGpsQzssFrame3 {
    /// Inclination angle cosine harmonic correction term
    pub cic: f64,

    /// Inclination angle sine harmonic correction term
    pub cis: f64,

    /// Orbit radius cosine harmonic correction term
    pub crc: f64,

    /// Inclination angle at reference time
    pub i0_rad: f64,

    /// IODE: Issue of Data (Ephemeris)
    pub iode: u8,

    /// Rate of inclination angle
    pub idot_rad_s: f64,

    /// Longitude of ascending node of orbit plane at weekly epoch
    pub omega0_rad: f64,

    /// Omega:
    pub omega_rad: f64,

    /// Omega_dot
    pub omega_dot_rad_s: f64,
}
