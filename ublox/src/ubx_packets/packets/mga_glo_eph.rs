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

/// UBX-MGA-GLO EPH frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x06, fixed_payload_len = 48)]
struct MgaGloEph {
    /// Message type (0x01 for this type)
    msg_type: u8,

    /// Message version (0x00 for this version)
    version: u8,

    /// Glonass Satellite identifier
    sv_id: u8,

    /// Reserved
    reserved1: u8,

    /// User range accuracy
    ft: u8,

    /// Health flag (from string #2)
    b: u8,

    /// Type of Glonass satellite.
    /// 1 means Glonass-M satellite.
    m: u8,

    /// Carrier frequency number (FDMA/RF).
    /// Range is [-7, +6]. -128 is used when unknown.
    h: i8,

    /// X component in kilometers, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-11)]
    x_km: i32,

    /// Y component in kilometers, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-11)]
    y_km: i32,

    /// Z component in kilometers, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-11)]
    z_km: i32,

    /// Velocity X component in kilometers per second, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    dx_km_s: i32,

    /// Velocity Y component in kilometers per second, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    dy_km_s: i32,

    /// Velocity Z component in kilometers per second, in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    dz_km_s: i32,

    /// Acceleration X component, in kilometers.s⁻², in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    ddx_km_s2: i8,

    /// Acceleration Y component, in kilometers.s⁻², in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    ddy_km_s2: i8,

    /// Acceleration Z component, in kilometers.s⁻², in PZ-90.02 coordinates system.
    #[ubx(map_type = f64, scale = 2e-20)]
    ddz_km_s2: i8,

    /// Index of a time interval within current day (according to UTC-SU), in minutes.
    tb_mins: u8,

    /// Relative frequency deviation
    #[ubx(map_type = f64, scale = 2e-40)]
    gamma: u16,

    /// Ephemeris data age (in days)
    eph_age_days: u8,

    /// L2-L1 time difference (in seconds)
    #[ubx(map_type = f64, scale = 2e-30)]
    delta_tau_s: u8,

    /// SV clock bias (in seconds)
    #[ubx(map_type = f64, scale = 2e-30)]
    tau_s: i32,

    /// Reserved
    reserved2: [u8; 4],
}
