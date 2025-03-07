#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::cfg_val::CfgVal;

use bitflags::bitflags;

use super::SerializeUbxPacketFields;

use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv_send, ubx_packet_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    nav::NavBbrMask, ubx_checksum, MemWriter, ScaleBack, UbxChecksumCalc, UbxPacketCreator,
    UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};

/// Configure odometer
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x1E,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
pub struct CfgOdo {
    pub version: u8,
    pub reserved: [u8; 3],
    /// Odometer COG filter flags. See [OdoCogFilterFlags] for details.
    #[ubx(map_type = OdoCogFilterFlags)]
    pub flags: u8,
    #[ubx(map_type = OdoProfile, may_fail)]
    pub odo_cfg: u8,
    pub reserved2: [u8; 6],
    pub cog_max_speed: u8,
    pub cog_max_pos_acc: u8,
    pub reserved3: [u8; 2],
    pub vel_lp_gain: u8,
    pub cog_lp_gain: u8,
    pub reserved4: [u8; 2],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct OdoCogFilterFlags: u8 {
        /// Odometer enabled flag
        const USE_ODO = 0x01;
        /// Low-speed COG filter enabled flag
        const USE_COG = 0x02;
        /// Output low-pass filtered velocity flag
        const OUT_LP_VEL = 0x04;
        /// Output low-pass filtered heading (COG) flag
        const OUT_LP_COG = 0x08;
    }
}

/// Odometer configuration profile
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OdoProfile {
    #[default]
    Running = 0,
    Cycling = 1,
    Swimming = 2,
    Car = 3,
    Custom = 4,
}

/// Reset Type
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ResetMode {
    /// Hardware reset (Watchdog) immediately
    HardwareResetImmediately = 0,
    ControlledSoftwareReset = 0x1,
    ControlledSoftwareResetGpsOnly = 0x02,
    /// Hardware reset (Watchdog) after shutdown (>=FW6.0)
    HardwareResetAfterShutdown = 0x04,
    ControlledGpsStop = 0x08,
    ControlledGpsStart = 0x09,
}

impl ResetMode {
    const fn into_raw(self) -> u8 {
        self as u8
    }
}

#[ubx_packet_send]
#[ubx(
  class = 0x06,
  id = 0x8a,
  max_payload_len = 772, // 4 + (4 + 8) * 64
)]
pub struct CfgValSet<'a> {
    /// Message version
    pub version: u8,
    /// The layers from which the configuration items should be retrieved
    #[ubx(map_type = CfgLayer)]
    pub layers: u8,
    pub reserved1: u16,
    pub cfg_data: &'a [CfgVal],
}

#[derive(Debug, Clone)]
pub struct CfgValIter<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) offset: usize,
}

impl<'a> CfgValIter<'a> {
    pub fn new(data: &'a mut [u8], values: &[CfgVal]) -> Self {
        let mut offset = 0;

        for value in values {
            offset += value.write_to(&mut data[offset..]);
        }

        Self {
            data: &data[..offset],
            offset: 0,
        }
    }
}

impl<'a> core::iter::Iterator for CfgValIter<'a> {
    type Item = CfgVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.data.len() {
            let cfg_val = CfgVal::parse(&self.data[self.offset..]);

            self.offset += cfg_val.len();

            Some(cfg_val)
        } else {
            None
        }
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing where configuration is applied.
    pub struct CfgLayer: u8 {
        const RAM = 0b001;
        const BBR = 0b010;
        const FLASH = 0b100;
    }
}

impl Default for CfgLayer {
    fn default() -> Self {
        Self::RAM | Self::BBR | Self::FLASH
    }
}

impl UartMode {
    pub const fn new(data_bits: DataBits, parity: Parity, stop_bits: StopBits) -> Self {
        Self {
            data_bits,
            parity,
            stop_bits,
        }
    }

    const fn into_raw(self) -> u32 {
        self.data_bits.into_raw() | self.parity.into_raw() | self.stop_bits.into_raw()
    }
}

impl From<u32> for UartMode {
    fn from(mode: u32) -> Self {
        let data_bits = DataBits::from(mode);
        let parity = Parity::from(mode);
        let stop_bits = StopBits::from(mode);

        Self {
            data_bits,
            parity,
            stop_bits,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataBits {
    Seven,
    Eight,
}

impl DataBits {
    const POSITION: u32 = 6;
    const MASK: u32 = 0b11;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::Seven => 0b10,
            Self::Eight => 0b11,
        }) << Self::POSITION
    }
}

impl From<u32> for DataBits {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b00 => unimplemented!("five data bits"),
            0b01 => unimplemented!("six data bits"),
            0b10 => Self::Seven,
            0b11 => Self::Eight,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parity {
    Even,
    Odd,
    None,
}

impl Parity {
    const POSITION: u32 = 9;
    const MASK: u32 = 0b111;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::Even => 0b000,
            Self::Odd => 0b001,
            Self::None => 0b100,
        }) << Self::POSITION
    }
}

impl From<u32> for Parity {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b000 => Self::Even,
            0b001 => Self::Odd,
            0b100 | 0b101 => Self::None,
            0b010 | 0b011 | 0b110 | 0b111 => unimplemented!("reserved"),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StopBits {
    One,
    OneHalf,
    Two,
    Half,
}

impl StopBits {
    const POSITION: u32 = 12;
    const MASK: u32 = 0b11;

    const fn into_raw(self) -> u32 {
        (match self {
            Self::One => 0b00,
            Self::OneHalf => 0b01,
            Self::Two => 0b10,
            Self::Half => 0b11,
        }) << Self::POSITION
    }
}

impl From<u32> for StopBits {
    fn from(mode: u32) -> Self {
        match (mode >> Self::POSITION) & Self::MASK {
            0b00 => Self::One,
            0b01 => Self::OneHalf,
            0b10 => Self::Two,
            0b11 => Self::Half,
            _ => unreachable!(),
        }
    }
}

/// Port Configuration for SPI Port
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x00,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
pub struct CfgPrtSpi {
    #[ubx(map_type = SpiPortId, may_fail)]
    pub portid: u8,
    pub reserved0: u8,
    /// TX ready PIN configuration
    pub tx_ready: u16,
    /// SPI Mode Flags
    pub mode: u32,
    pub reserved3: u32,
    #[ubx(map_type = InProtoMask)]
    pub in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    pub out_proto_mask: u16,
    pub flags: u16,
    pub reserved5: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing which input protocols are active
    /// Each bit of this mask is used for a protocol.
    /// Through that, multiple protocols can be defined on a single port
    /// Used in `CfgPrtSpi` and `CfgPrtI2c`
    #[derive(Default, Debug)]
    pub struct InProtoMask: u16 {
        const UBLOX = 1;
        const NMEA = 2;
        const RTCM = 4;
        /// The bitfield inRtcm3 is not supported in protocol
        /// versions less than 20
        const RTCM3 = 0x20;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing which output protocols are active.
    /// Each bit of this mask is used for a protocol.
    /// Through that, multiple protocols can be defined on a single port
    /// Used in `CfgPrtSpi` and `CfgPrtI2c`
    #[derive(Default, Debug)]
    pub struct OutProtoMask: u16 {
        const UBLOX = 1;
        const NMEA = 2;
        /// The bitfield outRtcm3 is not supported in protocol
        /// versions less than 20
        const RTCM3 = 0x20;
    }
}

/// Port Identifier Number (= 4 for SPI port)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum SpiPortId {
    #[default]
    Spi = 4,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UartMode {
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

/// Port Configuration for I2C
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x00,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
pub struct CfgPrtI2c {
    #[ubx(map_type = I2cPortId, may_fail)]
    pub portid: u8,
    pub reserved1: u8,
    /// TX ready PIN configuration
    pub tx_ready: u16,
    /// I2C Mode Flags
    pub mode: u32,
    pub reserved2: u32,
    #[ubx(map_type = InProtoMask)]
    pub in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    pub out_proto_mask: u16,
    pub flags: u16,
    pub reserved3: u16,
}

/// Port Identifier Number (= 0 for I2C ports)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum I2cPortId {
    #[default]
    I2c = 0,
}

/// Port Configuration for UART
#[ubx_packet_recv_send]
#[ubx(class = 0x06, id = 0x00, fixed_payload_len = 20)]
pub struct CfgPrtUart {
    #[ubx(map_type = UartPortId, may_fail)]
    pub portid: u8,
    pub reserved0: u8,
    pub tx_ready: u16,
    #[ubx(map_type = UartMode)]
    pub mode: u32,
    pub baud_rate: u32,
    #[ubx(map_type = InProtoMask)]
    pub in_proto_mask: u16,
    #[ubx(map_type = OutProtoMask)]
    pub out_proto_mask: u16,
    pub flags: u16,
    pub reserved5: u16,
}

/// Port Identifier Number (= 1 or 2 for UART ports)
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum UartPortId {
    Uart1 = 1,
    Uart2 = 2,
    Usb = 3,
}

/// Navigation/Measurement Rate Settings
#[ubx_packet_send]
#[ubx(class = 6, id = 8, fixed_payload_len = 6)]
pub struct CfgRate {
    /// Measurement Rate, GPS measurements are taken every `measure_rate_ms` milliseconds
    pub measure_rate_ms: u16,

    /// Navigation Rate, in number of measurement cycles.
    /// On u-blox 5 and u-blox 6, this parametercannot be changed, and is always equals 1.
    pub nav_rate: u16,

    /// Alignment to reference time
    #[ubx(map_type = AlignmentToReferenceTime)]
    pub time_ref: u16,
}

/// Configure Jamming interference monitoring
#[ubx_packet_recv_send]
#[ubx(class = 0x06, id = 0x39, fixed_payload_len = 8)]
struct CfgItfm {
    /// Interference config Word
    #[ubx(map_type = CfgItfmConfig)]
    config: u32,
    /// Extra settings
    #[ubx(map_type = CfgItfmConfig2)]
    config2: u32,
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmConfig {
    /// enable interference detection
    enable: bool,
    /// Broadband jamming detection threshold (dB)
    bb_threshold: CfgItfmBbThreshold,
    /// CW jamming detection threshold (dB)
    cw_threshold: CfgItfmCwThreshold,
    /// Reserved algorithm settings
    /// should be set to 0x16B156 default value
    /// for correct settings
    algorithm_bits: CfgItfmAlgoBits,
}

impl CfgItfmConfig {
    pub fn new(enable: bool, bb_threshold: u32, cw_threshold: u32) -> Self {
        Self {
            enable,
            bb_threshold: bb_threshold.into(),
            cw_threshold: cw_threshold.into(),
            algorithm_bits: CfgItfmAlgoBits::default(),
        }
    }

    const fn into_raw(self) -> u32 {
        (self.enable as u32) << 31
            | self.cw_threshold.into_raw()
            | self.bb_threshold.into_raw()
            | self.algorithm_bits.into_raw()
    }
}

impl From<u32> for CfgItfmConfig {
    fn from(cfg: u32) -> Self {
        let enable = (cfg & 0x80000000) > 0;
        let bb_threshold = CfgItfmBbThreshold::from(cfg);
        let cw_threshold = CfgItfmCwThreshold::from(cfg);
        let algorithm_bits = CfgItfmAlgoBits::from(cfg);
        Self {
            enable,
            bb_threshold,
            cw_threshold,
            algorithm_bits,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmBbThreshold(u32);

impl CfgItfmBbThreshold {
    const POSITION: u32 = 0;
    const LENGTH: u32 = 4;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmBbThreshold {
    fn default() -> Self {
        Self(3) // from UBX specifications
    }
}

impl From<u32> for CfgItfmBbThreshold {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmCwThreshold(u32);

impl CfgItfmCwThreshold {
    const POSITION: u32 = 4;
    const LENGTH: u32 = 5;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmCwThreshold {
    fn default() -> Self {
        Self(15) // from UBX specifications
    }
}

impl From<u32> for CfgItfmCwThreshold {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmAlgoBits(u32);

impl CfgItfmAlgoBits {
    const POSITION: u32 = 9;
    const LENGTH: u32 = 22;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmAlgoBits {
    fn default() -> Self {
        Self(0x16B156) // from UBX specifications
    }
}

impl From<u32> for CfgItfmAlgoBits {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmConfig2 {
    /// General settings, should be set to
    /// 0x31E default value, for correct setting
    general: CfgItfmGeneralBits,
    /// antenna settings
    antenna: CfgItfmAntennaSettings,
    /// Set to true to scan auxillary bands on ublox-M8,
    /// ignored otherwise
    scan_aux_bands: bool,
}

impl CfgItfmConfig2 {
    pub fn new(antenna: CfgItfmAntennaSettings, scan_aux_bands: bool) -> Self {
        Self {
            general: CfgItfmGeneralBits::default(),
            antenna,
            scan_aux_bands,
        }
    }

    const fn into_raw(self) -> u32 {
        ((self.scan_aux_bands as u32) << 14)
            | self.general.into_raw()
            | self.antenna.into_raw() as u32
    }
}

impl From<u32> for CfgItfmConfig2 {
    fn from(cfg: u32) -> Self {
        let scan_aux_bands = (cfg & 0x4000) > 0;
        let general = CfgItfmGeneralBits::from(cfg);
        let antenna = CfgItfmAntennaSettings::from(cfg);
        Self {
            scan_aux_bands,
            general,
            antenna,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgItfmGeneralBits(u32);

impl CfgItfmGeneralBits {
    const POSITION: u32 = 0;
    const LENGTH: u32 = 12;
    const MASK: u32 = (1 << Self::LENGTH) - 1;
    const fn into_raw(self) -> u32 {
        (self.0 & Self::MASK) << Self::POSITION
    }
}

impl Default for CfgItfmGeneralBits {
    fn default() -> Self {
        Self(0x31E) // from UBX specifications
    }
}

impl From<u32> for CfgItfmGeneralBits {
    fn from(thres: u32) -> Self {
        Self(thres)
    }
}

/// ITFM Antenna settings helps the interference
/// monitoring module
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum CfgItfmAntennaSettings {
    /// Type of Antenna is not known
    #[default]
    Unknown = 0,
    /// Active antenna
    Active = 1,
    /// Passive antenna
    Passive = 2,
}

impl From<u32> for CfgItfmAntennaSettings {
    fn from(cfg: u32) -> Self {
        let cfg = (cfg & 0x3000) >> 12;
        match cfg {
            1 => CfgItfmAntennaSettings::Active,
            2 => CfgItfmAntennaSettings::Passive,
            _ => CfgItfmAntennaSettings::Unknown,
        }
    }
}

/// Information message conifg
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x2,
    fixed_payload_len = 10,
    flags = "default_for_builder"
)]
pub struct CfgInf {
    pub protocol_id: u8,
    pub reserved: [u8; 3],
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_0: u8,
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_1: u8,
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_2: u8,
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_3: u8,
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_4: u8,
    #[ubx(map_type = CfgInfMask)]
    pub inf_msg_mask_5: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgInfMask` parameters bitmask
    #[derive(Default, Debug, Clone, Copy)]
    pub struct CfgInfMask: u8 {
        const ERROR = 0x1;
        const WARNING = 0x2;
        const NOTICE = 0x4;
        const TEST  = 0x08;
        const DEBUG = 0x10;
    }
}

/// Reset Receiver / Clear Backup Data Structures
#[ubx_packet_send]
#[ubx(class = 6, id = 4, fixed_payload_len = 4)]
pub struct CfgRst {
    /// Battery backed RAM sections to clear
    #[ubx(map_type = NavBbrMask)]
    pub nav_bbr_mask: u16,

    /// Reset Type
    #[ubx(map_type = ResetMode)]
    pub reset_mode: u8,
    pub reserved1: u8,
}

/// Set Message Rate the current port
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 3)]
pub struct CfgMsgSinglePort {
    pub msg_class: u8,
    pub msg_id: u8,

    /// Send rate on current Target
    pub rate: u8,
}

impl CfgMsgSinglePortBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rate: u8) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rate,
        }
    }
}

/// Set Message rate configuration
/// Send rate is relative to the event a message is registered on.
/// For example, if the rate of a navigation message is set to 2,
/// the message is sent every second navigation solution
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 8)]
pub struct CfgMsgAllPorts {
    pub msg_class: u8,
    pub msg_id: u8,

    /// Send rate on I/O Port (6 Ports)
    pub rates: [u8; 6],
}

impl CfgMsgAllPortsBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rates: [u8; 6]) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rates,
        }
    }
}

/// Navigation Engine Settings
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x24,
    fixed_payload_len = 36,
    flags = "default_for_builder"
)]
struct CfgNav5 {
    /// Only the masked parameters will be applied
    #[ubx(map_type = CfgNav5Params)]
    mask: u16,
    #[ubx(map_type = CfgNav5DynModel, may_fail)]
    dyn_model: u8,
    #[ubx(map_type = CfgNav5FixMode, may_fail)]
    fix_mode: u8,

    /// Fixed altitude (mean sea level) for 2D fixmode (m)
    #[ubx(map_type = f64, scale = 0.01)]
    fixed_alt: i32,

    /// Fixed altitude variance for 2D mode (m^2)
    #[ubx(map_type = f64, scale = 0.0001)]
    fixed_alt_var: u32,

    /// Minimum Elevation for a GNSS satellite to be used in NAV (deg)
    min_elev_degrees: i8,

    /// Reserved
    dr_limit: u8,

    /// Position DOP Mask to use
    #[ubx(map_type = f32, scale = 0.1)]
    pdop: u16,

    /// Time DOP Mask to use
    #[ubx(map_type = f32, scale = 0.1)]
    tdop: u16,

    /// Position Accuracy Mask (m)
    pacc: u16,

    /// Time Accuracy Mask
    /// according to manual unit is "m", but this looks like typo
    tacc: u16,

    /// Static hold threshold
    #[ubx(map_type = f32, scale = 0.01)]
    static_hold_thresh: u8,

    /// DGNSS timeout (seconds)
    dgps_time_out: u8,

    /// Number of satellites required to have
    /// C/N0 above `cno_thresh` for a fix to be attempted
    cno_thresh_num_svs: u8,

    /// C/N0 threshold for deciding whether toattempt a fix (dBHz)
    cno_thresh: u8,
    reserved1: [u8; 2],

    /// Static hold distance threshold (beforequitting static hold)
    static_hold_max_dist: u16,

    /// UTC standard to be used
    #[ubx(map_type = CfgNav5UtcStandard, may_fail)]
    utc_standard: u8,
    reserved2: [u8; 5],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNav5` parameters bitmask
    #[derive(Default, Debug, PartialEq, Eq)]
    pub struct CfgNav5Params: u16 {
        /// Apply dynamic model settings
        const DYN = 1;
        /// Apply minimum elevation settings
        const MIN_EL = 2;
        /// Apply fix mode settings
       const POS_FIX_MODE = 4;
        /// Reserved
        const DR_LIM = 8;
        /// position mask settings
       const POS_MASK_APPLY = 0x10;
        /// Apply time mask settings
        const TIME_MASK = 0x20;
        /// Apply static hold settings
        const STATIC_HOLD_MASK = 0x40;
        /// Apply DGPS settings
        const DGPS_MASK = 0x80;
        /// Apply CNO threshold settings (cnoThresh, cnoThreshNumSVs)
        const CNO_THRESHOLD = 0x100;
        /// Apply UTC settings (not supported in protocol versions less than 16)
        const UTC = 0x400;
    }
}

/// Dynamic platform model
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5DynModel {
    Portable = 0,
    Stationary = 2,
    Pedestrian = 3,
    Automotive = 4,
    Sea = 5,
    AirborneWithLess1gAcceleration = 6,
    AirborneWithLess2gAcceleration = 7,
    #[default]
    AirborneWith4gAcceleration = 8,
    /// not supported in protocol versions less than 18
    WristWornWatch = 9,
    /// supported in protocol versions 19.2
    Bike = 10,
}

/// Position Fixing Mode
#[derive(Default)] // default needs to be derived before ubx_extend
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5FixMode {
    Only2D = 1,
    Only3D = 2,
    #[default]
    Auto2D3D = 3,
}

/// UTC standard to be used
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgNav5UtcStandard {
    /// receiver selects based on GNSS configuration (see GNSS timebases)
    #[default]
    Automatic = 0,
    /// UTC as operated by the U.S. NavalObservatory (USNO);
    /// derived from GPStime
    Usno = 3,
    /// UTC as operated by the former Soviet Union; derived from GLONASS time
    UtcSu = 6,
    /// UTC as operated by the National TimeService Center, China;
    /// derived from BeiDou time
    UtcChina = 7,
}

/// Navigation Engine Expert Settings
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x23,
    fixed_payload_len = 40,
    flags = "default_for_builder"
)]
struct CfgNavX5 {
    /// Only version 2 supported
    version: u16,

    /// Only the masked parameters will be applied
    #[ubx(map_type = CfgNavX5Params1)]
    mask1: u16,

    #[ubx(map_type = CfgNavX5Params2)]
    mask2: u32,

    /// Reserved
    reserved1: [u8; 2],

    /// Minimum number of satellites for navigation
    min_svs: u8,

    ///Maximum number of satellites for navigation
    max_svs: u8,

    /// Minimum satellite signal level for navigation
    min_cno_dbhz: u8,

    /// Reserved
    reserved2: u8,

    /// initial fix must be 3D
    ini_fix_3d: u8,

    /// Reserved
    reserved3: [u8; 2],

    /// issue acknowledgements for assistance message input
    ack_aiding: u8,

    /// GPS week rollover number
    wkn_rollover: u16,

    /// Permanently attenuated signal compensation
    sig_atten_comp_mode: u8,

    /// Reserved
    reserved4: u8,
    reserved5: [u8; 2],
    reserved6: [u8; 2],

    /// Use Precise Point Positioning (only available with the PPP product variant)
    use_ppp: u8,

    /// AssistNow Autonomous configuration
    aop_cfg: u8,

    /// Reserved
    reserved7: [u8; 2],

    /// Maximum acceptable (modeled) AssistNow Autonomous orbit error
    aop_orb_max_err: u16,

    /// Reserved
    reserved8: [u8; 4],
    reserved9: [u8; 3],

    /// Enable/disable ADR/UDR sensor fusion
    use_adr: u8,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNavX51` parameters bitmask
    #[derive(Default, Debug)]
    pub struct CfgNavX5Params1: u16 {
        /// apply min/max SVs settings
        const MIN_MAX = 0x4;
        /// apply minimum C/N0 setting
        const MIN_CNO = 0x8;
        /// apply initial 3D fix settings
        const INITIAL_3D_FIX = 0x40;
        /// apply GPS weeknumber rollover settings
        const WKN_ROLL = 0x200;
        /// apply assistance acknowledgement settings
        const AID_ACK = 0x400;
        /// apply usePPP flag
        const USE_PPP = 0x2000;
        /// apply aopCfg (useAOP flag) and aopOrbMaxErr settings (AssistNow Autonomous)
        const AOP_CFG = 0x4000;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// `CfgNavX5Params2` parameters bitmask
    #[derive(Default, Debug)]
    pub struct CfgNavX5Params2: u32 {
        ///  apply ADR/UDR sensor fusion on/off setting
        const USE_ADR = 0x40;
        ///  apply signal attenuation compensation feature settings
        const USE_SIG_ATTEN_COMP = 0x80;
    }
}

/// Time MODE3 Config Frame (32.10.37.1)
/// only available on `timing` receivers
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x71,
    fixed_payload_len = 40,
    flags = "default_for_builder"
)]
struct CfgTmode3 {
    version: u8,
    reserved1: u8,
    /// Receiver mode, see [CfgTmode3RcvrMode] enum
    #[ubx(map_type = CfgTmode3RcvrMode)]
    rcvr_mode: u8,
    #[ubx(map_type = CfgTmode3Flags)]
    flags: u8,
    /// WGS84 ECEF.x coordinate in [m] or latitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_x_or_lat: i32,
    /// WGS84 ECEF.y coordinate in [m] or longitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_y_or_lon: i32,
    /// WGS84 ECEF.z coordinate or altitude, both in [m],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    ecef_z_or_alt: i32,
    /// High precision WGS84 ECEF.x coordinate in [tenths of mm],
    /// or high precision latitude, in nano degrees,
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_x_or_lat_hp: i8,
    /// High precision WGS84 ECEF.y coordinate in [tenths of mm]
    /// or high precision longitude, in nano degrees,
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_y_or_lon_hp: i8,
    /// High precision WGS84 ECEF.z coordinate or altitude,
    /// both if tenths of [mm],
    /// depending on `flags` field.
    #[ubx(map_type = f32, scale = 1.0)]
    ecef_z_or_alt_hp: i8,
    reserved2: u8,
    /// Fixed position 3D accuracy [0.1 mm]
    #[ubx(map_type = f64, scale = 1e-4)]
    fixed_pos_acc: u32,
    /// Survey in minimum duration [s]
    sv_in_min_duration: u32,
    /// Survey in position accuracy limit [0.1 mm]
    #[ubx(map_type = f64, scale = 1e-4)]
    sv_in_accur_limit: u32,
    reserved3: [u8; 8],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode3RcvrMode: u8 {
        const DISABLED = 0x01;
        const SURVEY_IN = 0x02;
        /// True ARP position is required in `FixedMode`
        const FIXED_MODE = 0x04;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTp5Flags: u32 {
        // Enables time pulse
        const ACTIVE = 0x01;
        /// Synchronize time pulse to GNSS as
        /// soon as GNSS time is valid.
        /// Uses local lock otherwise.
        const LOCK_GNSS_FREQ = 0x02;
        /// use `freq_period_lock` and `pulse_len_ratio_lock`
        /// fields as soon as GPS time is valid. Uses
        /// `freq_period` and `pulse_len_ratio` when GPS time is invalid.
        const LOCKED_OTHER_SET = 0x04;
        /// `freq_period` and `pulse_len_ratio` fields
        /// are interprated as frequency when this bit is set
        const IS_FREQ = 0x08;
        /// Interprate pulse lengths instead of duty cycle
        const IS_LENGTH = 0x10;
        /// Align pulse to top of second
        /// Period time must be integer fraction of `1sec`
        /// `LOCK_GNSS_FREQ` is expected, to unlock this feature
        const ALIGN_TO_TOW = 0x20;
        /// Pulse polarity,
        /// 0: falling edge @ top of second,
        /// 1: rising edge @ top of second,
        const POLARITY = 0x40;
        /// UTC time grid
        const UTC_TIME_GRID = 0x80;
        /// GPS time grid
        const GPS_TIME_GRID = 0x100;
        /// GLO time grid
        const GLO_TIME_GRID = 0x200;
        /// BDS time grid
        const BDS_TIME_GRID = 0x400;
        /// GAL time grid
        /// not supported in protocol < 18
        const GAL_TIME_GRID = 0x800;
        /// Switches to FreqPeriodLock and PulseLenRatio
        /// as soon as Sync Manager has an accurate time,
        /// never switches back
        const SYNC_MODE_0 = 0x1000;
        /// Switches to FreqPeriodLock and PulseLenRatioLock
        /// as soon as Sync Manager has an accurante time,
        /// and switch back to FreqPeriodLock and PulseLenRatio
        /// when time gets inaccurate
        const SYNC_MODE_1 = 0x2000;
    }
}

/// Alignment to reference time
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum AlignmentToReferenceTime {
    Utc = 0,
    Gps = 1,
    Glo = 2,
    Bds = 3,
    Gal = 4,
}

impl AlignmentToReferenceTime {
    const fn into_raw(self) -> u16 {
        self as u16
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode3Flags: u8 {
        /// Set if position is given in Lat/Lon/Alt,
        /// ECEF coordinates being used otherwise
        const LLA = 0x01;
    }
}

/// Reset Receiver / Clear Backup Data Structures
#[ubx_packet_recv_send]
#[ubx(class = 6, id = 0x13, fixed_payload_len = 4)]
struct CfgAnt {
    /// Antenna flag mask. See [AntFlags] for details.
    #[ubx(map_type = AntFlags)]
    flags: u16,
    /// Antenna pin configuration. See 32.10.1.1 in receiver spec for details.
    pins: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct AntFlags: u16 {
        /// Enable supply voltage control signal
        const SVCS = 0x01;
        /// Enable short circuit detection
        const SCD = 0x02;
        /// Enable open circuit detection
        const OCD = 0x04;
        /// Power down on short circuit detection
        const PDWN_ON_SCD = 0x08;
        /// Enable automatic recovery from short circuit state
        const RECOVERY = 0x10;
    }
}

/// TP5: "Time Pulse" Config frame (32.10.38.4)
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x31,
    fixed_payload_len = 32,
    flags = "default_for_builder"
)]
struct CfgTp5 {
    #[ubx(map_type = CfgTp5TimePulseMode, may_fail)]
    tp_idx: u8,
    version: u8,
    reserved1: [u8; 2],
    /// Antenna cable delay [ns]
    #[ubx(map_type = f32, scale = 1.0)]
    ant_cable_delay: i16,
    /// RF group delay [ns]
    #[ubx(map_type = f32, scale = 1.0)]
    rf_group_delay: i16,
    /// Frequency in Hz or Period in us,
    /// depending on `flags::IS_FREQ` bit
    #[ubx(map_type = f64, scale = 1.0)]
    freq_period: u32,
    /// Frequency in Hz or Period in us,
    /// when locked to GPS time.
    /// Only used when `flags::LOCKED_OTHER_SET` is set
    #[ubx(map_type = f64, scale = 1.0)]
    freq_period_lock: u32,
    /// Pulse length or duty cycle, [us] or [*2^-32],
    /// depending on `flags::LS_LENGTH` bit
    #[ubx(map_type = f64, scale = 1.0)]
    pulse_len_ratio: u32,
    /// Pulse Length in us or duty cycle (*2^-32),
    /// when locked to GPS time.
    /// Only used when `flags::LOCKED_OTHER_SET` is set
    #[ubx(map_type = f64, scale = 1.0)]
    pulse_len_ratio_lock: u32,
    /// User configurable time pulse delay in [ns]
    #[ubx(map_type = f64, scale = 1.0)]
    user_delay: i32,
    /// Configuration flags, see [CfgTp5Flags]
    #[ubx(map_type = CfgTp5Flags)]
    flags: u32,
}

/// Time pulse selection, used in CfgTp5 frame
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum CfgTp5TimePulseMode {
    #[default]
    TimePulse = 0,
    TimePulse2 = 1,
}

/// Time MODE2 Config Frame (32.10.36.1)
/// only available on `timing` receivers
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x3d,
    fixed_payload_len = 28,
    flags = "default_for_builder"
)]
pub struct CfgTmode2 {
    /// Time transfer modes, see [CfgTmode2TimeXferModes] for details
    #[ubx(map_type = CfgTmode2TimeXferModes, may_fail)]
    pub time_transfer_mode: u8,
    pub reserved1: u8,
    #[ubx(map_type = CfgTmode2Flags)]
    pub flags: u16,
    /// WGS84 ECEF.x coordinate in [m] or latitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_x_or_lat: i32,
    /// WGS84 ECEF.y coordinate in [m] or longitude in [deg° *1E-5],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_y_or_lon: i32,
    /// WGS84 ECEF.z coordinate or altitude, both in [m],
    /// depending on `flags` field
    #[ubx(map_type = f64, scale = 1e-2)]
    pub ecef_z_or_alt: i32,
    /// Fixed position 3D accuracy in [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    pub fixed_pos_acc: u32,
    /// Survey in minimum duration in [s]
    pub survey_in_min_duration: u32,
    /// Survey in position accuracy limit in [m]
    #[ubx(map_type = f64, scale = 1e-3)]
    pub survery_in_accur_limit: u32,
}

/// Time transfer modes (32.10.36)
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CfgTmode2TimeXferModes {
    #[default]
    Disabled = 0,
    SurveyIn = 1,
    /// True position information required
    /// when using `fixed mode`
    FixedMode = 2,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgTmode2Flags :u16 {
        /// Position given in LAT/LON/ALT
        /// default being WGS84 ECEF
        const LLA = 0x01;
        /// In case LLA was set, Altitude value is not valid
        const ALT_INVALID = 0x02;
    }
}
