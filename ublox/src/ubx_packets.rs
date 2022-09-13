mod packets;
mod types;

use crate::error::MemWriterError;
pub use packets::*;
pub use types::*;

/// Information about concrete UBX protocol's packet
pub trait UbxPacketMeta {
    const CLASS: u8;
    const ID: u8;
    const FIXED_PAYLOAD_LEN: Option<u16>;
    const MAX_PAYLOAD_LEN: u16;
}

pub(crate) const SYNC_CHAR_1: u8 = 0xb5;
pub(crate) const SYNC_CHAR_2: u8 = 0x62;

/// The checksum is calculated over the packet, starting and including
/// the CLASS field, up until, but excluding, the checksum field.
/// So slice should starts with class id.
/// Return ck_a and ck_b
pub(crate) fn ubx_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a = 0_u8;
    let mut ck_b = 0_u8;
    for byte in data {
        ck_a = ck_a.overflowing_add(*byte).0;
        ck_b = ck_b.overflowing_add(ck_a).0;
    }
    (ck_a, ck_b)
}

/// For ubx checksum on the fly
#[derive(Default)]
struct UbxChecksumCalc {
    ck_a: u8,
    ck_b: u8,
}

impl UbxChecksumCalc {
    fn update(&mut self, chunk: &[u8]) {
        for byte in chunk {
            self.ck_a = self.ck_a.overflowing_add(*byte).0;
            self.ck_b = self.ck_b.overflowing_add(self.ck_a).0;
        }
    }
    fn result(self) -> (u8, u8) {
        (self.ck_a, self.ck_b)
    }
}

/// Abstraction for buffer creation/reallocation
/// to storing packet
pub trait MemWriter {
    type Error;
    /// make sure that we have at least `len` bytes for writing
    fn reserve_allocate(&mut self, len: usize) -> Result<(), MemWriterError<Self::Error>>;
    fn write(&mut self, buf: &[u8]) -> Result<(), MemWriterError<Self::Error>>;
}

#[cfg(feature = "std")]
impl MemWriter for Vec<u8> {
    type Error = std::io::Error;

    fn reserve_allocate(&mut self, len: usize) -> Result<(), MemWriterError<Self::Error>> {
        self.reserve(len);
        Ok(())
    }
    fn write(&mut self, buf: &[u8]) -> Result<(), MemWriterError<Self::Error>> {
        let ret = <dyn std::io::Write>::write(self, buf).map_err(MemWriterError::Custom)?;
        if ret == buf.len() {
            Ok(())
        } else {
            Err(MemWriterError::NotEnoughMem)
        }
    }
}

pub trait UbxPacketCreator {
    /// Create packet and store bytes sequence to somewhere using `out`
    fn create_packet<T: MemWriter>(self, out: &mut T) -> Result<(), MemWriterError<T::Error>>;
}

/// Packet not supported yet by this crate
#[derive(Debug, serde::Serialize)]
pub struct UbxUnknownPacketRef<'a> {
    pub payload: &'a [u8],
    pub class: u8,
    pub msg_id: u8,
}

/// Request specific packet
pub struct UbxPacketRequest {
    req_class: u8,
    req_id: u8,
}

impl UbxPacketRequest {
    pub const PACKET_LEN: usize = 8;

    #[inline]
    pub fn request_for<T: UbxPacketMeta>() -> Self {
        Self {
            req_class: T::CLASS,
            req_id: T::ID,
        }
    }
    #[inline]
    pub fn request_for_unknown(req_class: u8, req_id: u8) -> Self {
        Self { req_class, req_id }
    }

    #[inline]
    pub fn into_packet_bytes(self) -> [u8; Self::PACKET_LEN] {
        let mut ret = [
            SYNC_CHAR_1,
            SYNC_CHAR_2,
            self.req_class,
            self.req_id,
            0,
            0,
            0,
            0,
        ];
        let (ck_a, ck_b) = ubx_checksum(&ret[2..6]);
        ret[6] = ck_a;
        ret[7] = ck_b;
        ret
    }
}
