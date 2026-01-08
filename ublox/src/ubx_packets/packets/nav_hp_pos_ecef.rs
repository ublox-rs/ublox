use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::ubx_packets::types::{PositionECEF, ToECEF};
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv};

/// High Precision Geodetic Position Solution (ECEF)
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x13, fixed_payload_len = 28)]
struct NavHpPosEcef {
    /// Message version (0 for protocol version 27)
    version: u8,

    reserved1: [u8; 3],

    /// GPS Millisecond Time of Week
    itow: u32,

    /// ECEF X coordinate
    #[ubx(map_type = f64, alias = ecef_x_cm)]
    ecef_x: i32,

    /// ECEF Y coordinate
    #[ubx(map_type = f64, alias = ecef_y_cm)]
    ecef_y: i32,

    /// ECEF Z coordinate
    #[ubx(map_type = f64, alias = ecef_z_cm)]
    ecef_z: i32,

    /// High precision component of X
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_x + (ecef_x_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_x_hp_mm)]
    ecef_x_hp: i8,

    /// High precision component of Y
    /// Must be in the range -99..+99
    /// 9. Precise coordinate in cm = ecef_y + (ecef_y_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_y_hp_mm)]
    ecef_y_hp: i8,

    /// High precision component of Z
    /// Must be in the range -99..+99
    /// Precise coordinate in cm = ecef_z + (ecef_z_hp * 1e-2).
    #[ubx(map_type = f64, scale = 1e-1, alias = ecef_z_hp_mm)]
    ecef_z_hp: i8,

    #[ubx(map_type = NavHpPosEcefFlags)]
    flags: u8,

    /// Horizontal accuracy estimate (mm)
    #[ubx(map_type = f64, scale = 1e-1)]
    p_acc: u32,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct NavHpPosEcefFlags: u8 {
        const INVALID_ECEF = 1;

    }
}

fn ecef_from_cm_hp(cm: f64, hp_mm: f64) -> f64 {
    10e-2 * (cm + 0.1 * hp_mm)
}

macro_rules! impl_to_ecef {
    ($type:ty) => {
        impl ToECEF for $type {
            fn to_ecef(&self) -> PositionECEF {
                PositionECEF {
                    x: ecef_from_cm_hp(self.ecef_x_cm(), self.ecef_x_hp_mm()),
                    y: ecef_from_cm_hp(self.ecef_y_cm(), self.ecef_y_hp_mm()),
                    z: ecef_from_cm_hp(self.ecef_z_cm(), self.ecef_z_hp_mm()),
                }
            }
        }
    };
}

impl_to_ecef!(NavHpPosEcefRef<'_>);
impl_to_ecef!(NavHpPosEcefOwned);
