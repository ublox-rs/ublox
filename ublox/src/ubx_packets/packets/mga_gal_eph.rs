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

/// UBX-MGA-GAL EPH frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x02, fixed_payload_len = 76)]
struct MgaGalEph {
    /// Message type. 0x01 for this type.
    msg_type: u8,

    /// Message version.
    version: u8,

    /// Galileo Satellite Identifier.
    sv_id: u8,

    /// Reserved
    reserved1: u8,

    /// Ephemeris and clock correction issue of data
    iodnav: u16,

    /// Mean motion difference from computed value (in semi circles)
    #[ubx(map_type = f64, scale = 2e-43)]
    dn_semicircles: i16,

    /// Mean anomaly at reference time (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    m0_semicircles: i32,

    /// Eccentricity
    #[ubx(map_type = f64, scale = 2e-33)]
    e: u32,

    /// (Square root) of semi-major axis
    #[ubx(map_type = f64, scale = 2e-19)]
    sqrt_a: u32,

    /// Longitude of ascending node of orbital plane (at reference time,
    /// in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega0_semicircles: i32,

    /// Inclination angle at reference time (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    i0_semicircles: i32,

    /// Argument of perigee (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega_semicircles: i32,

    /// Rate of change of right ascension (in semi circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    omega_dot_semicircles: i32,

    /// Rate of change of inclination angle (in semi circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    i_dot_semicircles: i16,

    /// Cuc (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cuc_rad: i16,

    /// Cus (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cus_rad: i16,

    /// Crc (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crc_rad: i16,

    /// Crs (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crs_rad: i16,

    /// Cic (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cic_rad: i16,

    /// Cis (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cis_rad: i16,

    /// ToE (in seconds)
    #[ubx(map_type = f64, scale = 60.0)]
    toe: u16,

    /// af0 correction term (in seconds)
    #[ubx(map_type = f64, scale = 2e-34)]
    af0: i32,

    /// af1 correction term (in seconds per second)
    #[ubx(map_type = f64, scale = 2e-46)]
    af1: i32,

    /// af2 correction term (in seconds.s⁻²)
    #[ubx(map_type = f64, scale = 2e-59)]
    af2: i8,

    /// Signal-in-space accuracy index for E1-E5b dual frequency
    sisa_e1_e5b: u8,

    /// ToC (in seconds)
    #[ubx(map_type = f64, scale = 60.0)]
    toc: u16,

    /// E1-B broadcast group delay (in seconds)
    #[ubx(map_type = f64, scale = 2e-32)]
    bgd_e1_e5b_s: i16,

    /// Reserved
    reserved2: [u8; 2],

    /// E1-B signal health
    e1b_health: u8,

    /// E1-B data validity
    e1b_validity: u8,

    /// E5b signal health
    e5b_health: u8,

    /// E5b data validity
    e5b_validity: u8,

    /// Reserved3
    reserved3: [u8; 4],
}
