use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError, ubx_checksum, ubx_packets::UbxChecksumCalc, MemWriter, MemWriterError,
    UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv_send};

/// Reset Receiver / Clear Backup Data Structures
#[ubx_packet_recv_send]
#[ubx(class = 6, id = 0x13, fixed_payload_len = 4)]
struct CfgAnt {
    /// Antenna flag mask. See [AntFlags] for details.
    #[ubx(map_type = AntFlags)]
    flags: u16,
    /// Antenna pin configuration. See 32.10.1.1 in receiver spec for details.
    pins: u16,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    #[derive(Default, Debug)]
    pub struct AntFlags: u16 {
        /// Enable supply voltage control signal
        const SVCS = 0x01;
        /// Enable short circuit detection
        const SCD = 0x02;
        /// Enable open circuit detection
        const OCD = 0x04;
        /// Power down on short circuit detection
        const PDWN_ON_SCD = 0x08;
        /// Enable automatic recovery from short circuit state
        const RECOVERY = 0x10;
    }
}
