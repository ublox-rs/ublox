#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, ubx_packets::packets::ScaleBack, MemWriter, MemWriterError,
    UbxPacketCreator, UbxPacketMeta,
};

use ublox_derive::ubx_packet_recv_send;

#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
struct MgaGpsEph {
    /// Message type (0x01 for this type)
    msg_type: u8,

    /// Message version (0x00 for this version)
    version: u8,

    /// GPS satellite identifier
    sv_id: u8,

    /// Reserved
    reserved1: u8,

    /// Fit interval flag
    fit_interval: u8,

    /// URA index
    ura_index: u8,

    /// Satellite health
    sv_health: u8,

    /// Total group delay (in seconds)
    #[ubx(map_type = f64, scale = 2e-31)]
    tgd_s: i8,

    /// Issue of Ephemeris Data
    iodc: u16,

    /// ToC (in seconds)
    #[ubx(map_type = f64, scale = 2e+4)]
    toc: u16,

    /// Reserved
    reserved2: u8,

    /// af2 correction term (in seconds.s⁻2)
    #[ubx(map_type = f64, scale = 2e-55)]
    af2: i8,

    /// af1 correction term (in seconds per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    af1: i16,

    #[ubx(map_type = f64, scale = 2e-31)]
    af0: i32,

    /// Crs (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crs_rad: i16,

    /// Mean motion difference computed from value (in semi-circles)
    #[ubx(map_type = f64, scale = 2e-43)]
    dn_semicircles: i16,

    /// Mean anomaly at reference time (in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    m0_semicircles: i32,

    /// Cuc (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cuc: i16,

    /// Cus (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cus: i16,

    /// Eccentricity
    #[ubx(map_type = f64, scale = 2e-33)]
    e: u32,

    /// (Square root) of semi-major axis
    #[ubx(map_type = f64, scale = 2e-19)]
    sqrt_a: u32,

    /// ToE (in seconds)
    #[ubx(map_type = f64, scale = 2e+4)]
    toe: u16,

    /// Cic (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cic: i16,

    /// Longitude of ascending node (in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega0_semicircles: i32,

    /// Cis (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cis: i16,

    /// Crc (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crc: i16,

    /// Inclination angle at reference time (in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    i0_semicircles: i32,

    /// Argument of perigee (in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega_semicircles: i32,

    /// Rate of ascension (in semi-circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    omega_dot: i32,

    /// Rate of inclination angle (in semi-circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    idot_semicircles: i16,

    /// Reserved
    reserved3: [u8; 2],
}
