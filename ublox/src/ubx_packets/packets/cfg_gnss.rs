#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{error::ParserError, ubx_checksum, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2};
use ublox_derive::ubx_packet_recv_send;

/// Information message config
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GnssId {
    #[default]
    GPS = 0,
    SBAS = 1,
    GALILEO = 2,
    BEIDOU = 3,
    IMES = 4,
    QZSS = 5,
    GLONASS = 6,
}

impl TryFrom<u8> for GnssId {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GnssId::GPS),
            1 => Ok(GnssId::SBAS),
            2 => Ok(GnssId::GALILEO),
            3 => Ok(GnssId::BEIDOU),
            4 => Ok(GnssId::IMES),
            5 => Ok(GnssId::QZSS),
            6 => Ok(GnssId::GLONASS),
            _ => Err("Invalid GnssId value: value must be in range [0, 6]"),
        }
    }
}

/// Signal configuration mask
/// Bits 23-16 of flags in CFG-GNSS
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SigCfgMask {
    GPSL1CA,
    GPSL2C,
    GPSL5,
    SBASL1CA,
    GALILEOE1,
    GALILEOE5A,
    GALILEOE5B,
    BEIDOUB1I,
    BEIDOUB1C,
    BEIDOUB2A,
    QZSSL1CA,
    QZSSL1S,
    QZSSL2C,
    QZSSL5,
    GLONASSL10F,
    GLONASSL20F,
    Unknown,
}
/// Multi-GNSS config
/// Deprecatred in protocol versions above 23
/// Use CfgValSet and CfgValGet for newer protocol version
#[ubx_packet_recv_send]
#[ubx(class = 0x06, id = 0x3e, max_payload_len = 1024)]
#[derive(Debug, Default)]
struct CfgGnss<'a> {
    /// Message version (0 for this version)
    msg_version: u8,
    /// Number of tracking channels hardware (read only)
    num_trk_ch_hw: u8,
    /// Number of tracking channels to use (<= numTrkChHw) (read/write)
    num_trk_ch_use: u8,
    /// Number of config blocks to follow
    num_config_blocks: u8,

    // TODO: This should be used when the packet is of `recv` type
    // ---
    #[ubx(
       map_type = GnssConfigBlockIter<'a>,
       from = GnssConfigBlockIter::new,
       size_fn = data_len,
       is_valid = GnssConfigBlockIter::is_valid,
       may_fail,
    )]
    blocks: [u8; 0],
    // ----
    //
    // TODO: This should be used when the packet is of type `send` so it can be constructed
    // ---
    //blocks:&'a [GnssConfigBlock],
}

impl CfgGnssRef<'_> {
    const BLOCK_SIZE: usize = 8;
    fn data_len(&self) -> usize {
        self.num_config_blocks() as usize * Self::BLOCK_SIZE
    }
}

impl CfgGnssOwned {
    const BLOCK_SIZE: usize = 8;
    fn data_len(&self) -> usize {
        self.num_config_blocks() as usize * Self::BLOCK_SIZE
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GnssConfigBlock {
    /// GNSS identifier (see [GnssId])
    pub gnss_id: GnssId,
    /// Minimum number of tracking channels reserved for this GNSS (read only)
    pub res_trk_ch: u8,
    /// Maximum number of tracking channels supported by this GNSS (read only)
    pub max_trk_ch: u8,

    pub reserved1: u8,
    pub flags: u32,
}

impl GnssConfigBlock {
    const SIG_CFG_MASK: u32 = 0x00FF;
    pub fn enabled(&self) -> bool {
        self.flags & 0x01 == 1
    }

    pub fn sig_cfg_mask(&self) -> SigCfgMask {
        let sig_cfg: u8 = u32::to_le_bytes((self.flags >> 16) & Self::SIG_CFG_MASK)[0];
        match self.gnss_id {
            GnssId::GPS => match sig_cfg {
                0x01 => SigCfgMask::GPSL1CA,
                0x10 => SigCfgMask::GPSL2C,
                0x20 => SigCfgMask::GPSL5,
                _ => SigCfgMask::Unknown,
            },
            GnssId::SBAS => match sig_cfg {
                0x01 => SigCfgMask::SBASL1CA,
                _ => SigCfgMask::Unknown,
            },
            GnssId::BEIDOU => match sig_cfg {
                0x01 => SigCfgMask::BEIDOUB1I,
                0x10 => SigCfgMask::BEIDOUB1C,
                0x80 => SigCfgMask::BEIDOUB2A,
                _ => SigCfgMask::Unknown,
            },
            GnssId::GALILEO => match sig_cfg {
                0x01 => SigCfgMask::GALILEOE1,
                0x10 => SigCfgMask::GALILEOE5A,
                0x20 => SigCfgMask::GALILEOE5B,
                _ => SigCfgMask::Unknown,
            },
            GnssId::GLONASS => match sig_cfg {
                0x01 => SigCfgMask::GLONASSL10F,
                0x10 => SigCfgMask::GLONASSL20F,
                _ => SigCfgMask::Unknown,
            },
            GnssId::QZSS => match sig_cfg {
                0x01 => SigCfgMask::QZSSL1CA,
                0x04 => SigCfgMask::QZSSL1S,
                0x10 => SigCfgMask::QZSSL2C,
                0x20 => SigCfgMask::QZSSL5,
                _ => SigCfgMask::Unknown,
            },
            GnssId::IMES => SigCfgMask::Unknown,
        }
    }

    /// TODO: check that this is correct
    pub fn extend_to<T>(&self, buf: &mut T) -> usize
    where
        T: core::iter::Extend<u8>,
    {
        let flags_bytes = self.flags.to_le_bytes();
        let bytes = [
            self.gnss_id as u8,
            self.res_trk_ch,
            self.max_trk_ch,
            self.reserved1,
            flags_bytes[0],
            flags_bytes[1],
            flags_bytes[2],
            flags_bytes[3],
        ];
        buf.extend(bytes);
        bytes.len()
    }
}

#[derive(Debug, Clone)]
pub struct GnssConfigBlockIter<'a>(core::slice::ChunksExact<'a, u8>);

impl<'a> GnssConfigBlockIter<'a> {
    const BLOCK_SIZE: usize = 8;
    fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl core::iter::Iterator for GnssConfigBlockIter<'_> {
    type Item = GnssConfigBlock;

    fn next(&mut self) -> Option<Self::Item> {
        const HALF_BLOCK: usize = 4;
        let chunk = self.0.next()?;
        let data = u32::from_le_bytes(chunk[0..HALF_BLOCK].try_into().unwrap());
        let flags = u32::from_le_bytes(chunk[HALF_BLOCK..Self::BLOCK_SIZE].try_into().unwrap());
        Some(Self::Item {
            gnss_id: (((data >> 24) & 0xFF) as u8).try_into().unwrap(),
            res_trk_ch: (((data >> 16) & 0xFF) as u8),
            max_trk_ch: (((data >> 8) & 0xFF) as u8),
            reserved1: 0,
            flags,
        })
    }
}
