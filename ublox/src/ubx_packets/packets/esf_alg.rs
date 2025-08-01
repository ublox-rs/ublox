use bitflags::bitflags;
use core::fmt;

#[allow(unused_imports, reason = "It's only unused in some feature sets")]
use crate::FieldIter;
#[cfg(feature = "serde")]
use {super::SerializeUbxPacketFields, crate::serde::ser::SerializeMap};

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

#[ubx_packet_recv]
#[ubx(class = 0x10, id = 0x14, fixed_payload_len = 16)]
struct EsfAlg {
    itow: u32,
    /// Message version: 0x01 for M8L
    version: u8,

    #[ubx(map_type = EsfAlgFlags, from = EsfAlgFlags)]
    flags: u8,

    #[ubx(map_type = EsfAlgError)]
    error: u8,

    reserved1: u8,

    /// IMU mount yaw angle [0, 360]
    #[ubx(map_type = f64, scale = 1e-2, alias = yaw)]
    yaw: u32,

    /// IMU mount pitch angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = pitch)]
    pitch: i16,

    /// IMU mount roll angle [-90, 90]
    #[ubx(map_type = f64, scale = 1e-2, alias = roll)]
    roll: i16,
}

/// UBX-ESF-ALG flags
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EsfAlgFlags(u8);

impl EsfAlgFlags {
    pub fn flags_raw(&self) -> u8 {
        self.0
    }
    pub fn auto_imu_mount_alg_on(self) -> bool {
        (self.0) & 0x1 != 0
    }

    pub fn status(self) -> EsfAlgStatus {
        let bits = (self.0 >> 1) & 0x07;
        match bits {
            0 => EsfAlgStatus::UserDefinedAngles,
            1 => EsfAlgStatus::RollPitchAlignmentOngoing,
            2 => EsfAlgStatus::RollPitchYawAlignmentOngoing,
            3 => EsfAlgStatus::CoarseAlignment,
            4 => EsfAlgStatus::FineAlignment,
            _ => EsfAlgStatus::Invalid,
        }
    }
}

impl fmt::Debug for EsfAlgFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("flags")
            .field("autoMntAlgOn", &self.auto_imu_mount_alg_on())
            .field("status", &self.status())
            .finish()
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct EsfAlgError: u8 {
        const TILT_ALG_ERROR = 0x01;
        const YAW_ALG_ERROR = 0x02;
        const ANGLE_ERROR = 0x04;
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EsfAlgStatus {
    UserDefinedAngles = 0,
    RollPitchAlignmentOngoing = 1,
    RollPitchYawAlignmentOngoing = 2,
    CoarseAlignment = 3,
    FineAlignment = 4,
    /// Only three bits are reserved for this field
    Invalid = 5,
}
