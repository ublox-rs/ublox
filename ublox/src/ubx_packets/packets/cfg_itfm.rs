#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, ubx_packets::UbxChecksumCalc, MemWriter, MemWriterError,
    UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::{ubx_extend, ubx_packet_recv_send};

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
        ((self.enable as u32) << 31)
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
    /// Set to true to scan auxiliary bands on ublox-M8,
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
