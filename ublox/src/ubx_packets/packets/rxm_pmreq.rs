use bitflags::bitflags;

use crate::{ubx_checksum, MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta};

use ublox_derive::{ubx_extend_bitflags, ubx_packet_send};

/// Request a power management related task of the receiver
#[ubx_packet_send]
#[ubx(class = 0x02, id = 0x41, fixed_payload_len = 8)]
pub struct RxmPmreq {
    /// Duration of the requested task in ms, set zero to infinite
    /// duration
    duration_ms: u32,

    /// Task flags. See [RxmPmreqFlags]
    #[ubx(map_type = RxmPmreqFlags)]
    flags: u32,
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    pub struct RxmPmreqFlags: u32 {
        /// The receiver goes into backup mode for a time period
        const BACKUP = 0x02;
    }
}
