use bitflags::bitflags;

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
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv_send};

/// Get/set wheel-tick configuration
/// Only available for ADR products
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x82,
    fixed_payload_len = 32,
    flags = "default_for_builder"
)]
struct CfgEsfWt {
    version: u8,

    #[ubx(map_type = CfgEsfWtFlags1)]
    flags1: u8,

    #[ubx(map_type = CfgEsfWtFlags2)]
    flags2: u8,
    reserved1: u8,

    /// Wheel tick scaling factor
    #[ubx(map_type = f64, scale = 1e-6)]
    wt_factor: u32,

    /// Wheel tick quantization
    #[ubx(map_type = f64, scale = 1e-6)]
    wt_quant_error: u32,

    /// Wheel tick counter maximum value
    wt_count_max: u32,

    /// Wheel tick latency due to e.g. CAN bus
    wt_latency: u16,

    /// Nominal wheel tick data frequency
    wt_frequency: u8,

    #[ubx(map_type = CfgEsfWtFlags3)]
    flags3: u8,

    /// Speed sensor dead band
    speed_dead_band: u16,

    reserved2: [u8; 10],
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags1 : u8 {
        /// Use combined rear wheel-ticks
        const COMBINED_TICKS = 0x01;
        /// Low-speed COG filter enabled flag
        const USE_WHEEL_TICK_SPEED = 0x10;
        /// Direction pin polarity
        const DIR_PIN_POLARITY = 0x20;
        /// Use wheel tick pin for speed measurement
        const USE_WT_PIN = 0x40;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags2 : u8 {
        const AUTO_WT_COUNT_MAX_OFF = 0x01;
        const AUTO_DIR_PIN_POL_OFF = 0x02;
        const AUTO_SOFTWARE_WT_OFF = 0x04;
        const AUTO_USE_WT_SPEED_OFF = 0x08;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct CfgEsfWtFlags3 : u8 {
        /// Count both rising and falling edges of wheel-tick
        const CNT_BOTH_EDGES = 0x01;
    }
}
