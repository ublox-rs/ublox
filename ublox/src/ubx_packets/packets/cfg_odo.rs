use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta,
};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv_send};

/// Configure odometer
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x1E,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
struct CfgOdo {
    version: u8,
    reserved: [u8; 3],
    /// Odometer COG filter flags. See [OdoCogFilterFlags] for details.
    #[ubx(map_type = OdoCogFilterFlags)]
    flags: u8,
    #[ubx(map_type = OdoProfile, may_fail)]
    odo_cfg: u8,
    reserved2: [u8; 6],
    cog_max_speed: u8,
    cog_max_pos_acc: u8,
    reserved3: [u8; 2],
    vel_lp_gain: u8,
    cog_lp_gain: u8,
    reserved4: [u8; 2],
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
