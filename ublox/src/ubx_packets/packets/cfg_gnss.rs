use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;
#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::fmt;

use crate::{error::ParserError, ubx_checksum, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv_send};

/// Multi-GNSS config
/// Deprecatred in protocol versions above 23, use CfgValSet and CfgValGet for newer protocol version
/// # Example
///
/// ```rust, ignore
/// # use ublox::{UbxPacket, UbxPacketRequest, cfg_gnss::{CfgGnss,CfgGnssBuilder, GnssConfigBlock, GnssId, GpsSigMask, GalileoSigMask, GlonassSigMask, BeidouSigMask, QzssSigMask, SbasSigMask}};
///
/// // Assume you have created a device (see examples folder)
/// let device = Device::new(serialport::new("/dev/ttyUSB0", 115200));
///
/// device.write_all(&UbxPacketRequest::request_for::<CfgGnss>().into_packet_bytes())
/// .expect("Failed to send poll/request for UBX-CFG-GNSS message");
///
/// // Create the packet
/// let mut buffer = Vec::new();
/// let blocks = [
///     GnssConfigBlock::new(true, GnssId::GPS, 8, 16, GpsSigMask::L1CA | GpsSigMask::L2C),
///     GnssConfigBlock::new(true, GnssId::GALILEO, 10, 18, GalileoSigMask::E1 | GalileoSigMask::E5B),
///     GnssConfigBlock::new(true, GnssId::GLONASS, 8, 12, GlonassSigMask::L10F | GlonassSigMask::L20F),
///     GnssConfigBlock::new(true, GnssId::BEIDOU, 8, 12, BeidouSigMask::B1I | BeidouSigMask::B1C),
///     GnssConfigBlock::new(true, GnssId::QZSS, 8, 12, QzssSigMask::L1CA | QzssSigMask::L2C | QzssSigMask::L5),
///     GnssConfigBlock::new(true, GnssId::SBAS, 8, 12, SbasSigMask::L1CA),
/// ];
///
/// CfgGnssBuilder::default()
///     .with_blocks(&blocks)
///     .extend_to(&mut buffer);

/// // Send the packet
/// device.write_all(&buffer)
///     .expect("Failed to send CFG-GNSS packet");
/// ```
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x3e,
    max_payload_len = 1024,
    flags = "default_for_builder"
)]
#[derive(Debug, Default)]
pub struct CfgGnss<'a> {
    /// Message version (0 for this version)
    msg_version: u8,
    /// Number of tracking channels hardware (read only)
    num_trk_ch_hw: u8,
    /// Number of tracking channels to use (<= numTrkChHw) (read/write)
    num_trk_ch_use: u8,
    /// Number of config blocks to follow
    num_config_blocks: u8,

    #[ubx(
       map_type = GnssConfigBlockIter<'a>,
       from = GnssConfigBlockIter::new,
       size_fn = data_len,
       is_valid = GnssConfigBlockIter::is_valid,
       may_fail,
    )]
    blocks: [u8; 0],
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

/// Information message config
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
    Gps(GpsSigMask),
    Galileo(GalileoSigMask),
    Glonass(GlonassSigMask),
    BeiDou(BeidouSigMask),
    Sbas(SbasSigMask),
    Qzss(QzssSigMask),
    Imes(ImesSigMask),
    Unknown(u8),
}

impl From<GpsSigMask> for SigCfgMask {
    fn from(m: GpsSigMask) -> Self {
        SigCfgMask::Gps(m)
    }
}
impl From<GalileoSigMask> for SigCfgMask {
    fn from(m: GalileoSigMask) -> Self {
        SigCfgMask::Galileo(m)
    }
}
impl From<BeidouSigMask> for SigCfgMask {
    fn from(m: BeidouSigMask) -> Self {
        SigCfgMask::BeiDou(m)
    }
}
impl From<GlonassSigMask> for SigCfgMask {
    fn from(m: GlonassSigMask) -> Self {
        SigCfgMask::Glonass(m)
    }
}
impl From<QzssSigMask> for SigCfgMask {
    fn from(m: QzssSigMask) -> Self {
        SigCfgMask::Qzss(m)
    }
}
impl From<SbasSigMask> for SigCfgMask {
    fn from(m: SbasSigMask) -> Self {
        SigCfgMask::Sbas(m)
    }
}
impl From<ImesSigMask> for SigCfgMask {
    fn from(m: ImesSigMask) -> Self {
        SigCfgMask::Imes(m)
    }
}

impl SigCfgMask {
    #[inline]
    pub fn raw_bits(self) -> u8 {
        match self {
            SigCfgMask::Gps(m) => m.bits(),
            SigCfgMask::Galileo(m) => m.bits(),
            SigCfgMask::Glonass(m) => m.bits(),
            SigCfgMask::BeiDou(m) => m.bits(),
            SigCfgMask::Sbas(m) => m.bits(),
            SigCfgMask::Qzss(m) => m.bits(),
            SigCfgMask::Imes(m) => m.bits(),
            SigCfgMask::Unknown(b) => b,
        }
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct GpsSigMask: u8 {
        const L1CA = 0x01;
        const L2C  = 0x10;
        const L5   = 0x20;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct SbasSigMask: u8 {
        const L1CA = 0x01;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct BeidouSigMask: u8 {
        const B1I = 0x01;
        const B1C = 0x10;
        const B2A = 0x80;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct GalileoSigMask: u8 {
        const E1 = 0x01;
        const E5A = 0x10;
        const E5B = 0x20;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct GlonassSigMask: u8 {
        const L10F = 0x01;
        const L20F = 0x10;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct QzssSigMask: u8 {
        const L1CA = 0x01;
        const L1S = 0x04;
        const L2C = 0x10;
        const L5 = 0x20;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ImesSigMask: u8 {
        const L1CA = 0x01;
    }
}

#[derive(Default, Clone)]
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
    #[inline]
    pub fn enabled(&self) -> bool {
        self.flags & 0x01 == 1
    }

    #[inline]
    pub fn new<M: Into<SigCfgMask>>(
        enabled: bool,
        gnss_id: GnssId,
        min: u8,
        max: u8,
        sigs: M,
    ) -> Self {
        let sigs = sigs.into();
        let mask = sigs.raw_bits();
        let flags = (enabled as u32) | ((mask as u32) << 16);
        Self {
            gnss_id,
            res_trk_ch: min,
            max_trk_ch: max,
            flags,
            reserved1: 0,
        }
    }

    #[inline]
    pub fn raw_sig_mask(&self) -> u8 {
        ((self.flags >> 16) & Self::SIG_CFG_MASK) as u8
    }

    pub fn sig_cfg_mask(&self) -> SigCfgMask {
        let m: u8 = self.raw_sig_mask();
        match self.gnss_id {
            GnssId::GPS => SigCfgMask::Gps(GpsSigMask::from_bits_truncate(m)),
            GnssId::GALILEO => SigCfgMask::Galileo(GalileoSigMask::from_bits_truncate(m)),
            GnssId::BEIDOU => SigCfgMask::BeiDou(BeidouSigMask::from_bits_truncate(m)),
            GnssId::GLONASS => SigCfgMask::Glonass(GlonassSigMask::from_bits_truncate(m)),
            GnssId::QZSS => SigCfgMask::Qzss(QzssSigMask::from_bits_truncate(m)),
            GnssId::SBAS => SigCfgMask::Sbas(SbasSigMask::from_bits_truncate(m)),
            GnssId::IMES => SigCfgMask::Unknown(m),
        }
    }

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

impl fmt::Debug for GnssConfigBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GnssConfigBlock")
            .field("gnss_id", &self.gnss_id)
            .field("enabled", &self.enabled())
            .field("res_trk_ch", &self.res_trk_ch)
            .field("max_trk_ch", &self.max_trk_ch)
            .field("sig_cfg_mask", &self.sig_cfg_mask())
            .field(
                "raw_sig_mask",
                &format_args!("0x{:08X}", self.raw_sig_mask()),
            )
            .field("flags_hex", &format_args!("0x{:08X}", self.flags))
            .finish()
    }
}

#[derive(Clone)]
pub enum GnssConfigBlockIter<'a> {
    // If the packet is of type `recv` we use this variant to iterate over the bytes
    Bytes(core::slice::ChunksExact<'a, u8>),
    // If the packet is of type `send` we use this variant to iterate over the slices
    // of GnssConfigBlock for CfgGnssBuilder and build the entire packet
    Slice(core::slice::Iter<'a, GnssConfigBlock>),
}

impl<'a> GnssConfigBlockIter<'a> {
    const BLOCK_SIZE: usize = 8;
    fn from_slice(blocks: &'a [GnssConfigBlock]) -> Self {
        Self::Slice(blocks.iter())
    }

    // For internal use by recv variant
    fn new(bytes: &'a [u8]) -> Self {
        Self::Bytes(bytes.chunks_exact(Self::BLOCK_SIZE))
    }

    fn is_valid(bytes: &'a [u8]) -> bool {
        bytes.len() % Self::BLOCK_SIZE == 0
    }
}

impl Default for GnssConfigBlockIter<'_> {
    fn default() -> Self {
        Self::from_slice(&[])
    }
}

impl fmt::Debug for GnssConfigBlockIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for item in self.clone() {
            list.entry(&item);
        }
        list.finish()
    }
}

impl core::iter::Iterator for GnssConfigBlockIter<'_> {
    type Item = GnssConfigBlock;

    fn next(&mut self) -> Option<Self::Item> {
        const HALF_BLOCK: usize = 4;
        match self {
            Self::Bytes(chunk) => {
                let chunk = chunk.next()?;
                let data = u32::from_le_bytes(chunk[0..HALF_BLOCK].try_into().ok()?);
                let flags =
                    u32::from_le_bytes(chunk[HALF_BLOCK..Self::BLOCK_SIZE].try_into().ok()?);
                let gnss_id = ((data & 0xFF) as u8).try_into().ok()?;
                Some(Self::Item {
                    gnss_id,
                    res_trk_ch: (((data >> 8) & 0xFF) as u8),
                    max_trk_ch: (((data >> 16) & 0xFF) as u8),
                    reserved1: (((data >> 24) & 0xFF) as u8),
                    flags,
                })
            },
            Self::Slice(it) => it.next().cloned(),
        }
    }
}

/// Convenience method to set the blocks for the builder
impl<'a> CfgGnssBuilder<'a> {
    pub fn with_blocks(mut self, blocks: &'a [GnssConfigBlock]) -> Self {
        self.num_config_blocks = blocks.len() as u8;
        self.blocks = GnssConfigBlockIter::from_slice(blocks);
        self
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports, reason = "unused in some feature sets")]
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn serialize_and_parse() {
        let blocks = [
            GnssConfigBlock {
                gnss_id: GnssId::GPS,
                ..Default::default()
            },
            GnssConfigBlock {
                gnss_id: GnssId::GALILEO,
                ..Default::default()
            },
        ];

        let builder = CfgGnssBuilder {
            msg_version: 0,
            num_trk_ch_hw: 1,
            num_trk_ch_use: 2,
            ..Default::default()
        }
        .with_blocks(&blocks);

        let mut out = Vec::new();
        builder.extend_to(&mut out);

        const HEADER_LEN: usize = 6;
        const LEN_LSB: usize = 4;
        const LEN_MSB: usize = 5;
        let len = u16::from_le_bytes([out[LEN_LSB], out[LEN_MSB]]) as usize;
        let payload = &out[HEADER_LEN..HEADER_LEN + len];

        assert!(CfgGnssRef::validate(payload).is_ok());

        let r = CfgGnssRef(payload);
        assert_eq!(r.num_config_blocks(), blocks.len() as u8);

        let parsed: Vec<GnssConfigBlock> = r.blocks().collect();
        assert_eq!(parsed.len(), blocks.len());
        assert_eq!(parsed[0].gnss_id, GnssId::GPS);
        assert_eq!(parsed[1].gnss_id, GnssId::GALILEO);
        assert_eq!(parsed[0].gnss_id as u8, blocks[0].gnss_id as u8);
        assert_eq!(parsed[1].gnss_id as u8, blocks[1].gnss_id as u8);
    }
}
