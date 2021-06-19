use alloc::vec::Vec;

use crate::{
    error::ParserError,
    ubx_packets::{
        match_packet, ubx_checksum, PacketRef, MAX_PAYLOAD_LEN, SYNC_CHAR_1, SYNC_CHAR_2,
    },
};

pub trait UnderlyingBuffer: core::ops::Index<core::ops::RangeFrom<usize>, Output = [u8]> {
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn extend_from_slice(&mut self, other: &[u8]);
    fn drain(&mut self, count: usize);
    fn find(&self, value: u8) -> Option<usize>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl UnderlyingBuffer for Vec<u8> {
    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn extend_from_slice(&mut self, other: &[u8]) {
        self.extend_from_slice(other);
    }

    fn drain(&mut self, count: usize) {
        self.drain(0..count);
    }

    fn find(&self, value: u8) -> Option<usize> {
        self.iter().position(|elem| *elem == value)
    }
}

/// Streaming parser for UBX protocol with buffer
pub struct Parser<T = Vec<u8>>
where
    T: UnderlyingBuffer,
{
    buf: T,
}

impl std::default::Default for Parser<Vec<u8>> {
    fn default() -> Self {
        Self { buf: vec![] }
    }
}

impl<T: UnderlyingBuffer> Parser<T> {
    pub fn is_buffer_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn buffer_len(&self) -> usize {
        self.buf.len()
    }

    pub fn consume(&mut self, new_data: &[u8]) -> ParserIter<T> {
        let start_idx = match self.buf.find(SYNC_CHAR_1) {
            Some(idx) => Some(idx),
            None => new_data
                .iter()
                .position(|elem| *elem == SYNC_CHAR_1)
                .map(|x| x + self.buf.len()),
        };

        match start_idx {
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
pub struct ParserIter<'a, T: UnderlyingBuffer> {
    buf: &'a mut T,
    off: usize,
}

impl<'a, T: UnderlyingBuffer> Drop for ParserIter<'a, T> {
    fn drop(&mut self) {
        if self.off <= self.buf.len() {
            self.buf.drain(self.off);
        }
    }
}

impl<'a, T: UnderlyingBuffer> ParserIter<'a, T> {
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of https://github.com/rust-lang/rust/issues/44265
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::ubx_packets::*;

    #[test]
    #[cfg(feature = "std")]
    fn parser_accepts_packet_vec_underlying() {
        let bytes = CfgNav5Builder {
            mask: CfgNav5Params::DYN,
            dyn_model: CfgNav5DynModel::AirborneWithLess1gAcceleration,
            fix_mode: CfgNav5FixMode::Only3D,
            fixed_alt: 100.17,
            fixed_alt_var: 0.0017,
            min_elev_degrees: 17,
            pdop: 1.7,
            tdop: 1.7,
            pacc: 17,
            tacc: 17,
            static_hold_thresh: 2.17,
            dgps_time_out: 17,
            cno_thresh_num_svs: 17,
            cno_thresh: 17,
            static_hold_max_dist: 0x1717,
            utc_standard: CfgNav5UtcStandard::UtcChina,
            ..CfgNav5Builder::default()
        }
        .into_packet_bytes();

        let mut parser = Parser::default();
        let mut it = parser.consume(&bytes);
        match it.next() {
            Some(Ok(PacketRef::CfgNav5(_packet))) => {
                // We're good
            }
            _ => {
                assert!(false);
            }
        }
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(feature = "std")]
    fn parser_accepts_multiple_packets() {
        let mut data = vec![];
        data.extend_from_slice(
            &CfgNav5Builder {
                pacc: 21,
                ..CfgNav5Builder::default()
            }
            .into_packet_bytes(),
        );
        data.extend_from_slice(
            &CfgNav5Builder {
                pacc: 18,
                ..CfgNav5Builder::default()
            }
            .into_packet_bytes(),
        );

        let mut parser = Parser::default();
        let mut it = parser.consume(&data);
        match it.next() {
            Some(Ok(PacketRef::CfgNav5(packet))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            }
            _ => {
                assert!(false);
            }
        }
        match it.next() {
            Some(Ok(PacketRef::CfgNav5(packet))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            }
            _ => {
                assert!(false);
            }
        }
        assert!(it.next().is_none());
    }
}

#[test]
fn test_max_payload_len() {
    assert!(MAX_PAYLOAD_LEN >= 1240);
}
