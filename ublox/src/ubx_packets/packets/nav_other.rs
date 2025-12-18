#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta,
};
use ublox_derive::{ubx_packet_recv, ubx_packet_send};

/// Odometer solution
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x09, fixed_payload_len = 20)]
struct NavOdo {
    version: u8,
    reserved: [u8; 3],
    itow: u32,
    distance: u32,
    total_distance: u32,
    distance_std: u32,
}

/// End of Epoch Marker
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x61, fixed_payload_len = 4)]
struct NavEoe {
    /// GPS time of week for navigation epoch
    itow: u32,
}

/// Reset odometer
#[ubx_packet_send]
#[ubx(class = 0x01, id = 0x10, fixed_payload_len = 0)]
struct NavResetOdo {}

#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x11, fixed_payload_len = 20)]
struct NavVelECEF {
    itow: u32,
    ecef_vx: i32,
    ecef_vy: i32,
    ecef_vz: i32,
    s_acc: u32,
}
