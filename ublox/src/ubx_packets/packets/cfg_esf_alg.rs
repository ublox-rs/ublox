use core::fmt;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError,
    ubx_checksum,
    ubx_packets::{packets::ScaleBack, UbxChecksumCalc},
    MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::ubx_packet_recv_send;

/// Get/set IMU-mount misalignment configuration
/// Only available for ADR products
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x56,
    fixed_payload_len = 12,
    flags = "default_for_builder"
)]
struct CfgEsfAlg {
    #[ubx(map_type = CfgEsfAlgFlags)]
    flags: u32,

    /// IMU mount yaw angle [0, 360]
    #[ubx(map_type = f64, scale = 1e-2, alias = yaw)]
    yaw: u32,

    /// IMU mount pitch angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = pitch)]
    pitch: i16,

    /// IMU mount roll angle [-180, 180]
    #[ubx(map_type = f64, scale = 1e-2, alias = roll)]
    roll: i16,
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CfgEsfAlgFlags {
    /// Not writable, only readable
    version: u8,
    /// Auto alignment status
    auto_alignment: bool,
}

impl From<u32> for CfgEsfAlgFlags {
    fn from(cfg: u32) -> Self {
        let version = cfg.to_le_bytes()[0];
        let auto_alignment = ((cfg >> 8) & 0x01) == 1;
        Self {
            version,
            auto_alignment,
        }
    }
}

impl CfgEsfAlgFlags {
    pub fn auto_imu_mount_alg_on(self) -> bool {
        self.auto_alignment
    }

    pub fn set_auto_imu_mount_alg(&mut self, enable: bool) {
        self.auto_alignment = enable
    }

    pub fn version(self) -> u8 {
        self.version
    }

    const fn into_raw(self) -> u32 {
        (self.auto_alignment as u32) << 8
    }
}

impl fmt::Debug for CfgEsfAlgFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("autoMntAlgOn", &self.auto_imu_mount_alg_on())
            .field("version", &self.version())
            .finish()
    }
}
