use crate::{
    ubx_checksum, ubx_packets::UbxChecksumCalc, MemWriter, MemWriterError, UbxPacketCreator,
    UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::ubx_packet_send;

/// Set Message Rate the current port
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 3)]
struct CfgMsgSinglePort {
    msg_class: u8,
    msg_id: u8,

    /// Send rate on current Target
    rate: u8,
}

impl CfgMsgSinglePortBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rate: u8) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rate,
        }
    }
}

/// Set Message rate configuration
/// Send rate is relative to the event a message is registered on.
/// For example, if the rate of a navigation message is set to 2,
/// the message is sent every second navigation solution
#[ubx_packet_send]
#[ubx(class = 6, id = 1, fixed_payload_len = 8)]
struct CfgMsgAllPorts {
    msg_class: u8,
    msg_id: u8,

    /// Send rate on I/O Port (6 Ports)
    rates: [u8; 6],
}

impl CfgMsgAllPortsBuilder {
    #[inline]
    pub fn set_rate_for<T: UbxPacketMeta>(rates: [u8; 6]) -> Self {
        Self {
            msg_class: T::CLASS,
            msg_id: T::ID,
            rates,
        }
    }
}
