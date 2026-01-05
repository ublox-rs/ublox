//! MON-PATCH: Installed Patches
//!
//! Reports information about installed firmware patches.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Size of each patch entry in bytes
const PATCH_ENTRY_SIZE: usize = 16;

/// Installed Patches
///
/// Reports information about installed firmware patches including
/// their activation status, location, and patch data.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x27, max_payload_len = 516)] // 4 + 32*16
struct MonPatch {
    /// Message version (0x0001 for this version)
    version: u16,

    /// Number of patch entries
    n_entries: u16,

    /// Patch entries (repeated n_entries times, 16 bytes each)
    #[ubx(map_type = MonPatchEntryIter, may_fail,
          from = MonPatchEntryIter::new,
          is_valid = MonPatchEntryIter::is_valid)]
    patches: [u8; 0],
}

/// Information about a single installed patch
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonPatchEntry {
    /// Patch status information
    pub patch_info: MonPatchInfo,
    /// Comparator number used by this patch
    pub comparator_number: u32,
    /// Target address of the patch
    pub patch_address: u32,
    /// Patch data
    pub patch_data: u32,
}

/// Patch status information bitfield
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonPatchInfo(u32);

impl MonPatchInfo {
    /// Returns true if the patch is currently active
    pub fn activated(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// Returns the storage location of the patch
    /// 0 = eFuse/OTP, 1 = ROM, 2 = BBR, 3 = file system
    pub fn location(&self) -> u8 {
        ((self.0 >> 1) & 0x03) as u8
    }

    /// Returns the raw patch info value
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl From<u32> for MonPatchInfo {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Iterator for MON-PATCH entry blocks
#[derive(Debug, Clone)]
pub struct MonPatchEntryIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MonPatchEntryIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    #[allow(dead_code, reason = "Used by ubx_packet_recv macro for validation")]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % PATCH_ENTRY_SIZE == 0
    }
}

impl core::iter::Iterator for MonPatchEntryIter<'_> {
    type Item = MonPatchEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + PATCH_ENTRY_SIZE)?;

        let entry = MonPatchEntry {
            patch_info: MonPatchInfo(u32::from_le_bytes(chunk[0..4].try_into().ok()?)),
            comparator_number: u32::from_le_bytes(chunk[4..8].try_into().ok()?),
            patch_address: u32::from_le_bytes(chunk[8..12].try_into().ok()?),
            patch_data: u32::from_le_bytes(chunk[12..16].try_into().ok()?),
        };

        self.offset += PATCH_ENTRY_SIZE;
        Some(entry)
    }
}
