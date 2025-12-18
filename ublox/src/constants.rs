pub const UBX_SYNC_CHAR_1: u8 = 0xb5;
pub const UBX_SYNC_CHAR_2: u8 = 0x62;
pub(crate) const UBX_SYNC_SIZE: usize = 2;
pub(crate) const UBX_PAYLOAD_SIZE_LEN: usize = 2;
pub(crate) const UBX_CLASS_LEN: usize = 1;
pub(crate) const UBX_ID_LEN: usize = 1;
pub(crate) const UBX_HEADER_LEN: usize =
    UBX_SYNC_SIZE + UBX_PAYLOAD_SIZE_LEN + UBX_CLASS_LEN + UBX_ID_LEN;
#[allow(dead_code, reason = "Used in tests")]
pub(crate) const UBX_CHECKSUM_LEN: usize = 2;

pub(crate) const UBX_CLASS_OFFSET: usize = 2; // After SYNC_CHAR_1, SYNC_CHAR_2
pub(crate) const UBX_MSG_ID_OFFSET: usize = 3; // After CLASS
pub(crate) const UBX_LENGTH_OFFSET: usize = 4; // After MSG_ID

// pub(crate) const UBX_CHECKSUM_OFFSET: usize = UBX_HEADER_LEN + UBX_PAYLOAD_SIZE_LEN;

pub const NMEA_SYNC_CHAR: u8 = 0x24; // '$'
pub const NMEA_END_CHAR_1: u8 = 0x0d; // '\r' (<CR>)
pub const NMEA_END_CHAR_2: u8 = 0x0a; // '\n' (<LF>)
pub(crate) const NMEA_MIN_BUFFER_SIZE: usize = 8; // sync (1) + talker (2) + msg type (3) + end chars (2)
pub(crate) const NMEA_MAX_SENTENCE_LENGTH: usize = 82; // Maximum NMEA sentence length

pub const RTCM_SYNC_CHAR: u8 = 0xd3;
pub(crate) const RTCM_HEADER_SIZE: usize = 3; // sync char (1) + length field (2)
pub(crate) const RTCM_LENGTH_MASK: u16 = 0x03ff; // 10 bits for length (6 bits reserved)
