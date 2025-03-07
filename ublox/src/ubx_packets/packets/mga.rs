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
    pub sv_id: u8,
    pub reserved1: u8,
    pub ft: u8,
    pub b: u8,
    pub m: u8,
    pub h: i8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub dx: i32,
    pub dy: i32,
    pub dz: i32,
    pub ddx: i8,
    pub ddy: i8,
    pub ddz: i8,
    pub tb: u8,
    pub gamma: u16,
    pub e: u8,
    pub delta_tau: u8,
    pub tau: i32,
    pub reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 16)]
pub struct MgaGpsIono {
    /// Message type: 0x06 for this type
    pub msg_type: u8,
    /// Message version: 0x00 for this version
    pub version: u8,
    pub reserved1: [u8; 2],
    /// Ionospheric parameter alpha0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-30
    pub alpha0: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-27
    pub alpha1: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    pub alpha2: i8,
    /// Ionospheric parameter alpha1 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-24
    pub alpha3: i8,
    /// Ionospheric parameter beta0 [s]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-11
    pub beta0: i8,
    /// Ionospheric parameter beta0 [s/semi-circle]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-14
    pub beta1: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^2]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    pub beta2: i8,
    /// Ionospheric parameter beta0 [s/semi-circle^3]
    #[ubx(map_type = f64, scale = 1.0)] // 2^-16
    pub beta3: i8,
    pub reserved2: [u8; 4],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
pub struct MgaGpsEph {
    pub msg_type: u8,
    pub version: u8,
    pub sv_id: u8,
    pub reserved1: u8,
    pub fit_interval: u8,
    pub ura_index: u8,
    pub sv_health: u8,
    pub tgd: i8,
    pub iodc: u16,
    pub toc: u16,
    pub reserved2: u8,
    pub af2: i8,
    pub af1: i16,
    pub af0: i32,
    pub crs: i16,
    pub delta_n: i16,
    pub m0: i32,
    pub cuc: i16,
    pub cus: i16,
    pub e: u32,
    pub sqrt_a: u32,
    pub toe: u16,
    pub cic: i16,
    pub omega0: i32,
    pub cis: i16,
    pub crc: i16,
    pub i0: i32,
    pub omega: i32,
    pub omega_dot: i32,
    pub idot: i16,
    pub reserved3: [u8; 2],
}

#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x00, fixed_payload_len = 68)]
pub struct MgaGpsEPH {
    pub msg_type: u8,
    pub version: u8,
    pub sv_id: u8,
    pub reserved1: u8,
    pub fit_interval: u8,
    pub ura_index: u8,
    pub sv_health: u8,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub tgd: i8,
    pub iodc: u16,
    #[ubx(map_type = f64, scale = 2e+4)]
    pub toc: u16,
    pub reserved2: u8,
    #[ubx(map_type = f64, scale = 2e-55)]
    pub af2: i8,
    #[ubx(map_type = f64, scale = 2e-43)]
    pub afl: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub af0: i32,
    #[ubx(map_type = f64, scale = 2e-5)]
    pub crs: i16,
    #[ubx(map_type = f64, scale = 2e-43)]
    pub delta_n: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub m0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    pub cuc: i16,
    #[ubx(map_type = f64, scale = 2e-29)]
    pub cus: i16,
    #[ubx(map_type = f64, scale = 2e-33)]
    pub e: u32,
    #[ubx(map_type = f64, scale = 2e-19)]
    pub sqrt_a: u32,
    #[ubx(map_type = f64, scale = 2e+4)]
    pub toe: u16,
    #[ubx(map_type = f64, scale = 2e-29)]
    pub cic: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub omega0: i32,
    #[ubx(map_type = f64, scale = 2e-29)]
    pub cis: i16,
    #[ubx(map_type = f64, scale = 2e-5)]
    pub crc: i16,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub i0: i32,
    #[ubx(map_type = f64, scale = 2e-31)]
    pub omega: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    pub omega_dot: i32,
    #[ubx(map_type = f64, scale = 2e-43)]
    pub idot: i16,
    pub reserved3: [u8; 2],
}

#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MsgAckInfoCode {
    Accepted = 0,
    RejectedNoTime = 1,
    RejectedBadVersion = 2,
    RejectedBadSize = 3,
    RejectedDBStoreFailed = 4,
    RejectedNotReady = 5,
    RejectedUnknownType = 6,
}

/// GNSS Assistance ACK UBX-MGA-ACK
#[ubx_packet_recv]
#[ubx(class = 0x13, id = 0x60, fixed_payload_len = 8)]
pub struct MgaAck {
    /// Type of acknowledgment: 0 -> not used, 1 -> accepted
    pub ack_type: u8,

    /// Version 0
    pub version: u8,

    /// Provides greater information on what the receiver chose to do with the message contents.
    /// See [MsgAckInfoCode].
    #[ubx(map_type = MsgAckInfoCode)]
    pub info_code: u8,

    /// UBX message ID of the acknowledged message
    pub msg_id: u8,

    /// The first 4 bytes of the acknowledged message's payload
    pub msg_payload_start: [u8; 4],
}
