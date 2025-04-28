#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// This message is used to retrieve a unique chip identifier
#[ubx_packet_recv]
#[ubx(class = 0x27, id = 0x03, fixed_payload_len = 9)]
struct SecUniqId {
    version: u8,
    reserved1: [u8; 3],
    unique_id: [u8; 5],
}
