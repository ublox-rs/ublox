mod error;
mod parser;

pub use error::{NotEnoughMem, ParserError};
pub use parser::{Parser, ParserIter};

/// Information about concrete UBX protocol's packet
pub trait UbxPacket {
    const CLASS: u8;
    const ID: u8;
    const FIXED_PAYLOAD_LENGTH: Option<u16>;
}

const SYNC_CHAR_1: u8 = 0xb5;
const SYNC_CHAR_2: u8 = 0x62;

/// The checksum is calculated over the packet, starting and including the CLASS field,
/// up until, but excluding, the Checksum Field
/// So slice should starts with class id
/// Return ck_a and ck_b
fn ubx_checksum(data: &[u8]) -> (u8, u8) {
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
    /// make sure that we have at least `len` bytes for writing
    fn reserve_allocate(&mut self, len: usize) -> Result<(), NotEnoughMem>;
    fn write(&mut self, buf: &[u8]) -> Result<(), NotEnoughMem>;
}

impl MemWriter for Vec<u8> {
    fn reserve_allocate(&mut self, len: usize) -> Result<(), NotEnoughMem> {
        self.reserve(len);
        Ok(())
    }
    fn write(&mut self, buf: &[u8]) -> Result<(), NotEnoughMem> {
        let ret = <dyn std::io::Write>::write(self, buf).map_err(|_| NotEnoughMem)?;
        if ret == buf.len() {
            Ok(())
        } else {
            Err(NotEnoughMem)
        }
    }
}

pub trait UbxPacketCreator {
    /// Create packet and store bytes sequence to somewhere using `out`
    fn create_packet(self, out: &mut dyn MemWriter) -> Result<(), NotEnoughMem>;
}

/// Packet not supported yet by this crate
#[derive(Debug)]
pub struct UnknownPacketRef<'a> {
    pub payload: &'a [u8],
    pub class: u8,
    pub msg_id: u8,
}

include!(concat!(env!("OUT_DIR"), "/packets.rs"));
