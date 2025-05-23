//! UBX (only) parser

use crate::{
    error::ParserError,
    parser::{DualBuffer, UnderlyingBuffer},
    ubx_packets::{match_packet, PacketRef, MAX_PAYLOAD_LEN, SYNC_CHAR_1, SYNC_CHAR_2},
};

/// For ubx checksum on the fly
#[derive(Default)]
struct UbxChecksumCalc {
    ck_a: u8,
    ck_b: u8,
}

impl UbxChecksumCalc {
    fn new() -> Self {
        Self { ck_a: 0, ck_b: 0 }
    }

    fn update(&mut self, bytes: &[u8]) {
        let mut a = self.ck_a;
        let mut b = self.ck_b;
        for byte in bytes.iter() {
            a = a.overflowing_add(*byte).0;
            b = b.overflowing_add(a).0;
        }
        self.ck_a = a;
        self.ck_b = b;
    }

    fn result(self) -> (u8, u8) {
        (self.ck_a, self.ck_b)
    }
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxParserIter<'a, T: UnderlyingBuffer> {
    pub(crate) buf: DualBuffer<'a, T>,
}

pub fn extract_packet_ubx<'b, T: UnderlyingBuffer>(
    buf: &'b mut DualBuffer<'_, T>,
    pack_len: usize,
) -> Option<Result<PacketRef<'b>, ParserError>> {
    if !buf.can_drain_and_take(6, pack_len + 2) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            buf.drain(2);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len + 2,
            }));
        }
        return None;
    }
    let mut checksummer = UbxChecksumCalc::new();
    let (a, b) = buf.peek_raw(2..(4 + pack_len + 2));
    checksummer.update(a);
    checksummer.update(b);
    let (ck_a, ck_b) = checksummer.result();

    let (expect_ck_a, expect_ck_b) = (buf[6 + pack_len], buf[6 + pack_len + 1]);
    if (ck_a, ck_b) != (expect_ck_a, expect_ck_b) {
        buf.drain(2);
        return Some(Err(ParserError::InvalidChecksum {
            expect: u16::from_le_bytes([expect_ck_a, expect_ck_b]),
            got: u16::from_le_bytes([ck_a, ck_b]),
        }));
    }
    let class_id = buf[2];
    let msg_id = buf[3];
    buf.drain(6);
    let msg_data = match buf.take(pack_len + 2) {
        Ok(x) => x,
        Err(e) => {
            return Some(Err(e));
        },
    };
    return Some(match_packet(
        class_id,
        msg_id,
        &msg_data[..msg_data.len() - 2], // Exclude the checksum
    ));
}

impl<T: UnderlyingBuffer> UbxParserIter<'_, T> {
    fn find_sync(&self) -> Option<usize> {
        (0..self.buf.len()).find(|&i| self.buf[i] == SYNC_CHAR_1)
    }

    #[allow(clippy::should_implement_trait)]
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of `<https://github.com/rust-lang/rust/issues/44265>`
    pub fn next(&mut self) -> Option<Result<PacketRef, ParserError>> {
        while self.buf.len() > 0 {
            let pos = match self.find_sync() {
                Some(x) => x,
                None => {
                    self.buf.clear();
                    return None;
                },
            };
            self.buf.drain(pos);

            if self.buf.len() < 2 {
                return None;
            }
            if self.buf[1] != SYNC_CHAR_2 {
                self.buf.drain(1);
                continue;
            }

            if self.buf.len() < 6 {
                return None;
            }

            let pack_len: usize = u16::from_le_bytes([self.buf[4], self.buf[5]]).into();
            if pack_len > usize::from(MAX_PAYLOAD_LEN) {
                self.buf.drain(2);
                continue;
            }
            return extract_packet_ubx(&mut self.buf, pack_len);
        }
        None
    }
}
