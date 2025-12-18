use bitflags::bitflags;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{
    cfg_val::{CfgKey, CfgVal},
    error::ParserError,
    ubx_checksum, UbxPacketMeta,
};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_send};

#[ubx_packet_send]
#[ubx(
  class = 0x06,
  id = 0x8a,
  max_payload_len = 772, // 4 + (4 + 8) * 64
)]
struct CfgValSet<'a> {
    /// Message version
    version: u8,
    /// The layers from which the configuration items should be retrieved
    #[ubx(map_type = CfgLayerSet)]
    layers: u8,
    reserved1: u16,
    cfg_data: &'a [CfgVal],
}

/// The CfgValGet message is limited to requesting a maximum of 64 key-value pairs.
pub const MAX_CFG_KEYS: u16 = 64;

#[ubx_packet_send]
#[ubx(
  class = 0x06,
  id = 0x8b,
  max_payload_len = 260, // 4 + sizeof(u32) * MAX_CFG_KEYS
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// This message is limited to containing a maximum of 64 key IDs.
/// This message returns a UBX-ACK-NAK
///  - if any key is unknown to the receiver FW
///  - if the layer field speciﬁes an invalid layer to get the value from
///  - if the keys array speciﬁes more than 64 key IDs.
struct CfgValGetRequest<'a> {
    /// Message version
    version: u8,
    /// The layers from which the configuration items should be retrieved
    #[ubx(map_type = CfgLayerGet)]
    layers: u8,
    position: u16,
    cfg_keys: &'a [CfgKey],
}

#[ubx_packet_recv]
#[ubx(
  class = 0x06,
  id = 0x8b,
  max_payload_len = 772, // 4 + (sizeof(u32) + sizeof(largest val)) * MAX_CFG_KEYS
)]
struct CfgValGetResponse {
    /// Message version
    version: u8,
    #[ubx(map_type = CfgLayerGet)]
    layers: u8,
    position: u16,
    #[ubx(
        map_type = CfgValIter,
        from = CfgValIter::new,
        may_fail,
        is_valid = CfgValIter::is_valid,
    )]
    cfg_data: [u8; 0],
}

/// The [CfgLayerGet] enum is used to specify the configuration layer to read from.
/// The configuration system in the ublox device is stacked, so a property
/// may be empty for a particular layer and you will receive a NAK.
#[ubx_extend]
#[ubx(from, into_raw, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum CfgLayerGet {
    /// Read from RAM
    Ram = 0,
    /// Read from BBR (battery backed RAM)
    Bbr = 1,
    /// Read from Flash, if available
    Flash = 2,
    /// Read the current configuration from the active source
    Default = 7,
}

#[derive(Debug, Clone)]
pub struct CfgValIter<'a> {
    data: &'a [u8],
}

impl<'a> CfgValIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn is_valid(bytes: &[u8]) -> bool {
        // we need at least 5 bytes for a key id (4) + val (1)
        bytes.len() >= 5
    }
}

impl core::iter::Iterator for CfgValIter<'_> {
    type Item = CfgVal;

    fn next(&mut self) -> Option<Self::Item> {
        if Self::is_valid(self.data) {
            if let Some(cfg_val) = CfgVal::parse(self.data) {
                self.data = &self.data[cfg_val.len()..];
                return Some(cfg_val);
            }
            // TODO: Is there some logging mechanism?
            // eprintln!("Failure parsing key in (key,value) list, {:?}", self.data);
        }
        None
    }
}

/// The `CfgLayerSet` defines the configuration layer used to set configuration values to.
/// The definition of the Layers for updating the configuration values is different than
/// the definition of the Layers for reading values, see [CfgLayerGet]
#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// A mask describing where configuration is applied.
    pub struct CfgLayerSet: u8 {
        const RAM = 0b001;
        const BBR = 0b010;
        const FLASH = 0b100;
    }
}

impl Default for CfgLayerSet {
    fn default() -> Self {
        Self::RAM | Self::BBR | Self::FLASH
    }
}
