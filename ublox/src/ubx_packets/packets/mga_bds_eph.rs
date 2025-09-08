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

/// UBX-MGA-BDS EPH frame.
#[ubx_packet_recv_send]
#[ubx(class = 0x13, id = 0x03, fixed_payload_len = 88)]
struct MgaBdsEph {
    /// Message type. 0x01 for this type.
    msg_type: u8,

    /// Message version.
    version: u8,

    /// BeiDou Satellite Identifier.
    sv_id: u8,

    /// Reserved
    reserved1: u8,

    /// Autonomous Satellite Health H1 flag
    sat_h1: u8,

    /// Issue of Data, Clock
    iodc: u8,

    /// a2 correction term (in seconds.s⁻²)
    #[ubx(map_type = f64, scale = 2e-66)]
    a2: i16,

    /// a1 correction term (in seconds per second)
    #[ubx(map_type = f64, scale = 2e-50)]
    a1: i32,

    /// a0 correction term (in seconds)
    #[ubx(map_type = f64, scale = 2e-33)]
    a0: i32,

    /// ToC (in seconds)
    #[ubx(map_type = f64, scale = 2e3)]
    toc: u32,

    /// Total group delay (in nanoseconds)
    #[ubx(map_type = f64, scale = 0.1)]
    tgd_ns: i16,

    /// URA index
    ura: u8,

    /// IODE
    iode: u8,

    /// ToE (in seconds)
    #[ubx(map_type = f64, scale = 2e3)]
    toe: u32,

    /// (Square root) of semi-major axis
    #[ubx(map_type = f64, scale = 2e-19)]
    sqrt_a: u32,

    /// Eccentricity
    #[ubx(map_type = f64, scale = 2e-33)]
    e: u32,

    /// Argument of perigee (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega_semicircles: i32,

    /// Mean motion difference from computed value (in semi circles)
    #[ubx(map_type = f64, scale = 2e-43)]
    dn_semicircles: i16,

    /// Rate of change of inclination angle (in semi circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    i_dot_semicircles: i16,

    /// Mean anomaly at reference time (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    m0_semicircles: i32,

    /// Longitude of ascending node of orbital plane (at reference time,
    /// in semi-circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    omega0_semicircles: i32,

    /// Rate of change of right ascension (in semi circles per second)
    #[ubx(map_type = f64, scale = 2e-43)]
    omega_dot_semicircles: i32,

    /// Inclination angle at reference time (in semi circles)
    #[ubx(map_type = f64, scale = 2e-31)]
    i0_semicircles: i32,

    /// Cuc (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cuc_rad: i32,

    /// Cus (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cus_rad: i32,

    /// Crc (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crc_rad: i32,

    /// Crs (in radians)
    #[ubx(map_type = f64, scale = 2e-5)]
    crs_rad: i32,

    /// Cic (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cic_rad: i32,

    /// Cis (in radians)
    #[ubx(map_type = f64, scale = 2e-29)]
    cis_rad: i32,

    /// Reserved
    reserved2: [u8; 4],
}
