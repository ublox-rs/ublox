use alloc::vec::Vec;

use crate::{
    error::ParserError,
    ubx_packets::{
        match_packet, ubx_checksum, PacketRef, MAX_PAYLOAD_LEN, SYNC_CHAR_1, SYNC_CHAR_2,
    },
};

/// Streaming parser for UBX protocol with buffer
#[derive(Default)]
pub struct Parser {
    buf: Vec<u8>,
}

impl Parser {
    pub fn is_buffer_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn buffer_len(&self) -> usize {
        self.buf.len()
    }

    pub fn consume(&mut self, new_data: &[u8]) -> ParserIter {
        match self
            .buf
            .iter()
            .chain(new_data.iter())
            .position(|x| *x == SYNC_CHAR_1)
        {
            Some(mut off) => {
                if off >= self.buf.len() {
                    off -= self.buf.len();
                    self.buf.clear();
                    self.buf.extend_from_slice(&new_data[off..]);
                    off = 0;
                } else {
                    self.buf.extend_from_slice(new_data);
                }
                ParserIter {
                    buf: &mut self.buf,
                    off,
                }
            }
            None => {
                self.buf.clear();
                ParserIter {
                    buf: &mut self.buf,
                    off: 0,
                }
            }
        }
    }
}

/// Iterator over data stored in `Parser` buffer
pub struct ParserIter<'a> {
    buf: &'a mut Vec<u8>,
    off: usize,
}

impl<'a> Drop for ParserIter<'a> {
    fn drop(&mut self) {
        if self.off <= self.buf.len() {
            self.buf.drain(0..self.off);
        }
    }
}

impl<'a> ParserIter<'a> {
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implmentation after merge of https://github.com/rust-lang/rust/issues/44265
    pub fn next(&mut self) -> Option<Result<PacketRef, ParserError>> {
        while self.off < self.buf.len() {
            let data = &self.buf[self.off..];
            let pos = data.iter().position(|x| *x == SYNC_CHAR_1)?;
            let maybe_pack = &data[pos..];

            if maybe_pack.len() <= 1 {
                return None;
            }
            if maybe_pack[1] != SYNC_CHAR_2 {
                self.off += pos + 2;
                continue;
            }

            if maybe_pack.len() <= 5 {
                return None;
            }

            let pack_len: usize = u16::from_le_bytes([maybe_pack[4], maybe_pack[5]]).into();
            if pack_len > usize::from(MAX_PAYLOAD_LEN) {
                self.off += pos + 2;
                continue;
            }
            if (pack_len + 6 + 2) > maybe_pack.len() {
                return None;
            }
            let (ck_a, ck_b) = ubx_checksum(&maybe_pack[2..(4 + pack_len + 2)]);

            let (expect_ck_a, expect_ck_b) =
                (maybe_pack[6 + pack_len], maybe_pack[6 + pack_len + 1]);
            if (ck_a, ck_b) != (expect_ck_a, expect_ck_b) {
                self.off += pos + 2;
                return Some(Err(ParserError::InvalidChecksum {
                    expect: u16::from_le_bytes([expect_ck_a, expect_ck_b]),
                    got: u16::from_le_bytes([ck_a, ck_b]),
                }));
            }
            let msg_data = &maybe_pack[6..(6 + pack_len)];
            let class_id = maybe_pack[2];
            let msg_id = maybe_pack[3];
            self.off += pos + 6 + pack_len + 2;
            return Some(match_packet(class_id, msg_id, msg_data));
        }
        None
    }
}

#[test]
fn test_max_payload_len() {
    assert!(MAX_PAYLOAD_LEN >= 1240);
}
