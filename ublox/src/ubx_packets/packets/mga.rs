use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitflags::bitflags;

use super::{FixStatusInfo, GpsFix, SerializeUbxPacketFields};

use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_send};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ubx_checksum, MemWriter, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1,
    SYNC_CHAR_2,
};

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x06, fixed_payload_len = 48)]
pub struct MgaGloEph {
    pub msg_type: u8,
    pub version: u8,
    sv_id: u8,
    reserved1: u8,
    ft: u8,
    b: u8,
    m: u8,
    h: i8,
    x: i32,
    y: i32,
    z: i32,
    dx: i32,
    dy: i32,
    dz: i32,
    ddx: i8,
    ddy: i8,
    ddz: i8,
    tb: u8,
    gamma: u16,
    e: u8,
    delta_tau: u8,
    tau: i32,
    reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 16)]
struct MgaGpsIono {
    /// Message type: 0x06 for this type
    msg_type: u8,
    /// Message version: 0x00 for this version
    version: u8,
    reserved1: [u8; 2],
    /// Ionospheric parameter alpha0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-30
    alpha0: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-27
    alpha1: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha2: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    alpha3: i8,
    /// Ionospheric parameter beta0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-11
    beta0: i8,
    /// Ionospheric parameter beta0 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-14
    beta1: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta2: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    beta3: i8,
    reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
struct MgaGpsEph {
    msg_type: u8,
    version: u8,
    sv_id: u8,
    reserved1: u8,
    fit_interval: u8,
    ura_index: u8,
    sv_health: u8,
    tgd: i8,
    iodc: u16,
    toc: u16,
    reserved2: u8,
    af2: i8,
    af1: i16,
    af0: i32,
    crs: i16,
    delta_n: i16,
    m0: i32,
    cuc: i16,
    cus: i16,
    e: u32,
    sqrt_a: u32,
    toe: u16,
    cic: i16,
    omega0: i32,
    cis: i16,
    crc: i16,
    i0: i32,
    omega: i32,
    omega_dot: i32,
    idot: i16,
    reserved3: [u8; 2],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
struct MgaGpsEPH {
    msg_type: u8,
    version: u8,
    sv_id: u8,
    reserved1: u8,
    fit_interval: u8,
    ura_index: u8,
    sv_health: u8,
    #[ubx(map_type = f64, scale = 2e-31)]
    tgd: i8,
    iodc: u16,
    #[ubx(map_type = f64, scale = 2e+4)]
    toc: u16,
    reserved2: u8,
    #[ubx(map_type = f64, scale = 2e-55)]
    af2: i8,
    #[ubx(map_type = f64, scale = 2e-43)]
    afl: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    af0: i32,
    #[ubx(map_type = f64, scale = 2e-5)]
    crs: i16,
    #[ubx(map_type = f64, scale = 2e-43)]
    delta_n: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    m0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    cuc: i16,
    #[ubx(map_type = f64, scale = 2e-29)]
    cus: i16,
    #[ubx(map_type = f64, scale = 2e-33)]
    e: u32,
    #[ubx(map_type = f64, scale = 2e-19)]
    sqrt_a: u32,
    #[ubx(map_type = f64, scale = 2e+4)]
    toe: u16,
    #[ubx(map_type = f64, scale = 2e-29)]
    cic: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    omega0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    cis: i16,
    #[ubx(map_type = f64, scale = 2e-5)]
    crc: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    i0: i32,
    #[ubx(map_type = f64, scale = 2e-31)]
    omega: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    omega_dot: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    idot: i16,
    reserved3: [u8; 2],
}
