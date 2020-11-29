use alloc::vec::Vec;

use crate::{
    circular_buffer::CircularBuffer,
    error::{MemWriterError, ParserError},
    linear_buffer::LinearBuffer,
    ubx_packets::{
        match_packet, ubx_checksum, PacketRef, UbxChecksumCalc, MAX_PAYLOAD_LEN, SYNC_CHAR_1,
        SYNC_CHAR_2,
    },
};

pub struct BufParser<'a> {
    buf: CircularBuffer<'a>,
}

impl<'a> BufParser<'a> {
    pub fn new(buf: &mut [u8]) -> BufParser {
        BufParser {
            buf: CircularBuffer::new(buf),
        }
    }

    pub fn consume_from_slice<T: LinearBuffer>(
        &'a mut self,
        new_data: &'a [u8],
        packet_store: &'a mut T,
    ) -> BufParserIter<'a, T> {
        self.consume(new_data.iter(), packet_store)
    }

    pub fn consume<T: LinearBuffer, ITER: core::iter::Iterator<Item = &'a u8>>(
        &'a mut self,
        new_data: ITER,
        packet_store: &'a mut T,
    ) -> BufParserIter<'a, T> {
        for x in new_data {
            self.buf.push(*x);
        }

        packet_store.clear();
        BufParserIter {
            buf: &mut self.buf,
            temp_storage: packet_store,
        }
    }
}

pub struct BufParserIter<'a, T: LinearBuffer> {
    buf: &'a mut CircularBuffer<'a>,
    temp_storage: &'a mut T,
}

impl<'a, T: LinearBuffer> BufParserIter<'a, T> {
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of https://github.com/rust-lang/rust/issues/44265
    pub fn next(&mut self) -> Option<Result<PacketRef, MemWriterError<ParserError>>> {
        while self.buf.len() > 0 {
            if self.buf.len() <= 5 {
                return None;
            }

            if (self.buf.at(0), self.buf.at(1)) != (SYNC_CHAR_1, SYNC_CHAR_2) {
                self.buf.skip(1);
                continue;
            }

            let pack_len: usize = u16::from_le_bytes([self.buf.at(4), self.buf.at(5)]).into();
            if pack_len > usize::from(MAX_PAYLOAD_LEN) {
                self.buf.skip(2);
                continue;
            }
            if (pack_len + 6 + 2) > self.buf.len() {
                return None;
            }
            let (ck_a, ck_b) = {
                let mut ck_calc = UbxChecksumCalc::new();
                for i in 2..(4 + pack_len + 2) {
                    ck_calc.update(&[self.buf.at(i)]);
                }
                ck_calc.result()
            };

            let (expect_ck_a, expect_ck_b) =
                (self.buf.at(6 + pack_len), self.buf.at(6 + pack_len + 1));
            if (ck_a, ck_b) != (expect_ck_a, expect_ck_b) {
                self.buf.skip(2);
                return Some(Err(MemWriterError::Custom(ParserError::InvalidChecksum {
                    expect: u16::from_le_bytes([expect_ck_a, expect_ck_b]),
                    got: u16::from_le_bytes([ck_a, ck_b]),
                })));
            }

            // Fill the underlying storage with the packet
            // If we run out of memory in the scratch buffer, skip the packet and tell the user
            match self
                .temp_storage
                .set::<(), _>(self.buf.iter().take(6 + pack_len + 2))
            {
                Ok(_) => {}
                Err(MemWriterError::NotEnoughMem) => {
                    self.buf.skip(2);
                    return Some(Err(MemWriterError::NotEnoughMem));
                }
                Err(MemWriterError::Custom(_)) => {
                    panic!("LinearBuffer::set() should never return a Custom error");
                }
            };
            self.buf.skip(6 + pack_len + 2);

            let packet = self.temp_storage.get_ref(6 + pack_len + 2);
            let msg_data = &packet[6..(6 + pack_len)];
            let class_id = packet[2];
            let msg_id = packet[3];
            match match_packet(class_id, msg_id, msg_data) {
                Ok(x) => {
                    return Some(Ok(x));
                }
                Err(e) => {
                    return Some(Err(MemWriterError::Custom(e)));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::linear_buffer::ArrayBuffer;
    use crate::ubx_packets::*;

    #[test]
    fn parser_accepts_packet_array_underlying() {
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

        let mut buf = [0; 1024];
        let mut parser = BufParser::new(&mut buf);
        let mut underlying = [0; 128];
        let mut underlying = ArrayBuffer::new(&mut underlying);
        let mut it = parser.consume_from_slice(&bytes, &mut underlying);
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

        let mut buf = [0; 1024];
        let mut parser = BufParser::new(&mut buf);
        let mut underlying = Vec::new();
        let mut it = parser.consume_from_slice(&bytes, &mut underlying);
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
}

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
