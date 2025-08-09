#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    error::ParserError,
    ubx_packets::{
        match_packet, PacketRef, MAX_PAYLOAD_LEN, RTCM_SYNC_CHAR, SYNC_CHAR_1, SYNC_CHAR_2,
    },
};

/// This trait represents an underlying buffer used for the Parser. We provide
/// implementations for `Vec<u8>` and for `FixedLinearBuffer`, if you want to
/// use your own struct as an underlying buffer you can implement this trait.
///
/// Look at the `flb_*` unit tests for ideas of unit tests you can run against
/// your own implementations.
pub trait UnderlyingBuffer:
    core::ops::Index<core::ops::Range<usize>, Output = [u8]> + core::ops::Index<usize, Output = u8>
{
    /// Removes all elements from the buffer.
    fn clear(&mut self);

    /// Returns the number of elements currently stored in the buffer.
    fn len(&self) -> usize;

    /// Returns the maximum capacity of this buffer. This value should be a minimum max
    /// capacity - that is, `extend_from_slice` should succeed if max_capacity bytes are
    /// passed to it.
    ///
    /// Note that, for example, the Vec implementation of this trait returns `usize::MAX`,
    /// which cannot be actually allocated by a Vec. This is okay, because Vec will panic
    /// if an allocation is requested that it can't handle.
    fn max_capacity(&self) -> usize;

    /// Returns the number of bytes not copied over due to buffer size constraints.
    ///
    /// As noted for `max_capacity`, if this function is passed `max_capacity() - len()`
    /// bytes it should either panic or return zero bytes, any other behaviour may cause
    /// unexpected behaviour in the parser.
    fn extend_from_slice(&mut self, other: &[u8]) -> usize;

    /// Removes the first `count` elements from the buffer. Cannot fail.
    fn drain(&mut self, count: usize);

    /// Locates the given u8 value within the buffer, returning the index (if it is found).
    fn find(&self, value: u8) -> Option<usize> {
        (0..self.len()).find(|&i| self[i] == value)
    }

    /// Returns whether the buffer is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl UnderlyingBuffer for Vec<u8> {
    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn max_capacity(&self) -> usize {
        usize::MAX
    }

    fn extend_from_slice(&mut self, other: &[u8]) -> usize {
        self.extend_from_slice(other);
        0
    }

    fn drain(&mut self, count: usize) {
        self.drain(0..count);
    }

    fn find(&self, value: u8) -> Option<usize> {
        self.iter().position(|elem| *elem == value)
    }
}

pub struct FixedLinearBuffer<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> FixedLinearBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buffer: buf,
            len: 0,
        }
    }
}

impl core::ops::Index<core::ops::Range<usize>> for FixedLinearBuffer<'_> {
    type Output = [u8];

    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        if index.end > self.len {
            panic!("Index {} is outside of our length {}", index.end, self.len);
        }
        self.buffer.index(index)
    }
}

impl core::ops::Index<usize> for FixedLinearBuffer<'_> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl UnderlyingBuffer for FixedLinearBuffer<'_> {
    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
    }

    fn max_capacity(&self) -> usize {
        self.buffer.len()
    }

    fn extend_from_slice(&mut self, other: &[u8]) -> usize {
        let to_copy = core::cmp::min(other.len(), self.buffer.len() - self.len);
        let uncopyable = other.len() - to_copy;
        self.buffer[self.len..self.len + to_copy].copy_from_slice(&other[..to_copy]);
        self.len += to_copy;
        uncopyable
    }

    fn drain(&mut self, count: usize) {
        if count >= self.len {
            self.len = 0;
            return;
        }

        let new_size = self.len - count;
        {
            let bufptr = self.buffer.as_mut_ptr();
            unsafe {
                core::ptr::copy(bufptr.add(count), bufptr, new_size);
            }
        }
        self.len = new_size;
    }

    fn find(&self, value: u8) -> Option<usize> {
        (0..self.len()).find(|&i| self[i] == value)
    }
}

/// Streaming parser for UBX protocol with buffer. The default constructor will build
/// a parser containing a Vec, but you can pass your own underlying buffer by passing it
/// to Parser::new().
///
/// If you pass your own buffer, it should be able to store at _least_ 4 bytes. In practice,
/// you won't be able to do anything useful unless it's at least 36 bytes long (the size
/// of a NavPosLlh packet).
pub struct Parser<T>
where
    T: UnderlyingBuffer,
{
    buf: T,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for Parser<Vec<u8>> {
    fn default() -> Self {
        Self { buf: Vec::new() }
    }
}

impl<T: UnderlyingBuffer> Parser<T> {
    pub fn new(underlying: T) -> Self {
        Self { buf: underlying }
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn buffer_len(&self) -> usize {
        self.buf.len()
    }

    pub fn consume_ubx<'a>(&'a mut self, new_data: &'a [u8]) -> UbxParserIter<'a, T> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == SYNC_CHAR_1 {
                buf.drain(i);
                break;
            }
        }

        UbxParserIter { buf }
    }

    pub fn consume_ubx_rtcm<'a>(&'a mut self, new_data: &'a [u8]) -> UbxRtcmParserIter<'a, T> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == SYNC_CHAR_1 || buf[i] == RTCM_SYNC_CHAR {
                buf.drain(i);
                break;
            }
        }

        UbxRtcmParserIter { buf }
    }
}

/// Stores two buffers: A "base" and a "new" buffer. Exposes these as the same buffer,
/// copying data from the "new" buffer to the base buffer as required to maintain that
/// illusion.
struct DualBuffer<'a, T: UnderlyingBuffer> {
    buf: &'a mut T,
    off: usize,

    new_buf: &'a [u8],
    new_buf_offset: usize,
}

impl<T: UnderlyingBuffer> core::ops::Index<usize> for DualBuffer<'_, T> {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        if self.off + index < self.buf.len() {
            &self.buf[index + self.off]
        } else {
            &self.new_buf[self.new_buf_offset + index - (self.buf.len() - self.off)]
        }
    }
}

impl<'a, T: UnderlyingBuffer> DualBuffer<'a, T> {
    fn new(buf: &'a mut T, new_buf: &'a [u8]) -> Self {
        Self {
            buf,
            off: 0,
            new_buf,
            new_buf_offset: 0,
        }
    }

    /// Clears all elements - equivalent to buf.drain(buf.len())
    fn clear(&mut self) {
        self.drain(self.len());
    }

    /// Remove count elements without providing a view into them.
    fn drain(&mut self, count: usize) {
        let underlying_bytes = core::cmp::min(self.buf.len() - self.off, count);
        let new_bytes = count.saturating_sub(underlying_bytes);

        self.off += underlying_bytes;
        self.new_buf_offset += new_bytes;
    }

    /// Return the total number of accessible bytes in this view. Note that you may
    /// not be able to take() this many bytes at once, if the total number of bytes
    /// is more than the underlying store can fit.
    fn len(&self) -> usize {
        self.buf.len() - self.off + self.new_buf.len() - self.new_buf_offset
    }

    // Returns the number of bytes which would be lost (because they can't be copied into
    // the underlying storage) if this DualBuffer were dropped.
    fn potential_lost_bytes(&self) -> usize {
        if self.len() <= self.buf.max_capacity() {
            0
        } else {
            self.len() - self.buf.max_capacity()
        }
    }

    fn can_drain_and_take(&self, drain: usize, take: usize) -> bool {
        let underlying_bytes = core::cmp::min(self.buf.len() - self.off, drain);
        let new_bytes = drain.saturating_sub(underlying_bytes);

        let drained_off = self.off + underlying_bytes;
        let drained_new_off = self.new_buf_offset + new_bytes;

        if take > self.buf.len() - drained_off + self.new_buf.len() - drained_new_off {
            // Draining removed too many bytes, we don't have enough to take
            return false;
        }

        let underlying_bytes = core::cmp::min(self.buf.len() - drained_off, take);
        let new_bytes = take.saturating_sub(underlying_bytes);

        if underlying_bytes == 0 {
            // We would take entirely from the new buffer
            return true;
        }

        if new_bytes == 0 {
            // We would take entirely from the underlying
            return true;
        }

        if new_bytes > self.buf.max_capacity() - (self.buf.len() - drained_off) {
            // We wouldn't be able to fit all the new bytes into underlying
            return false;
        }

        true
    }

    fn peek_raw(&self, range: core::ops::Range<usize>) -> (&[u8], &[u8]) {
        let split = self.buf.len() - self.off;
        let a = if range.start >= split {
            &[]
        } else {
            &self.buf[range.start + self.off..core::cmp::min(self.buf.len(), range.end + self.off)]
        };
        let b = if range.end <= split {
            &[]
        } else {
            &self.new_buf[self.new_buf_offset + range.start.saturating_sub(split)
                ..range.end - split + self.new_buf_offset]
        };
        (a, b)
    }

    /// Provide a view of the next count elements, moving data if necessary.
    /// If the underlying store cannot store enough elements, no data is moved and an
    /// error is returned.
    fn take(&mut self, count: usize) -> Result<&[u8], ParserError> {
        let underlying_bytes = core::cmp::min(self.buf.len() - self.off, count);
        let new_bytes = count.saturating_sub(underlying_bytes);

        if new_bytes > self.new_buf.len() - self.new_buf_offset {
            // We need to pull more bytes from new than it has
            panic!(
                "Cannot pull {} bytes from a buffer with {}-{}",
                new_bytes,
                self.new_buf.len(),
                self.new_buf_offset
            );
        }

        if underlying_bytes == 0 {
            // We can directly return a slice from new
            let offset = self.new_buf_offset;
            self.new_buf_offset += count;
            return Ok(&self.new_buf[offset..offset + count]);
        }

        if new_bytes == 0 {
            // We can directly return from underlying
            let offset = self.off;
            self.off += count;
            return Ok(&self.buf[offset..offset + count]);
        }

        if self.buf.max_capacity() < count {
            // Insufficient space
            return Err(ParserError::OutOfMemory {
                required_size: count,
            });
        }

        if new_bytes < self.buf.max_capacity() - self.buf.len() {
            // Underlying has enough space to extend from new
            let bytes_not_moved = self
                .buf
                .extend_from_slice(&self.new_buf[self.new_buf_offset..]);
            self.new_buf_offset += self.new_buf.len() - self.new_buf_offset - bytes_not_moved;
            let off = self.off;
            self.off += count;
            return Ok(&self.buf[off..off + count]);
        }

        // Last case: We have to move the data in underlying, then extend it
        self.buf.drain(self.off);
        self.off = 0;
        self.buf
            .extend_from_slice(&self.new_buf[self.new_buf_offset..self.new_buf_offset + new_bytes]);
        self.new_buf_offset += new_bytes;
        self.off += count;
        Ok(&self.buf[0..count])
    }
}

impl<T: UnderlyingBuffer> Drop for DualBuffer<'_, T> {
    fn drop(&mut self) {
        self.buf.drain(self.off);
        self.buf
            .extend_from_slice(&self.new_buf[self.new_buf_offset..]);
    }
}

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

#[derive(Debug, PartialEq, Eq)]
enum NextSync {
    Ubx(usize),
    Rtcm(usize),
    None,
}

#[derive(Debug)]
pub enum AnyPacketRef<'a> {
    Ubx(PacketRef<'a>),
    Rtcm(RtcmPacketRef<'a>),
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxParserIter<'a, T: UnderlyingBuffer> {
    buf: DualBuffer<'a, T>,
}

fn extract_packet_ubx<'b, T: UnderlyingBuffer>(
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
    pub fn next(&mut self) -> Option<Result<PacketRef<'_>, ParserError>> {
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

/// Iterator over data stored in `Parser` buffer
pub struct UbxRtcmParserIter<'a, T: UnderlyingBuffer> {
    buf: DualBuffer<'a, T>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RtcmPacketRef<'a> {
    pub data: &'a [u8],
}

fn extract_packet_rtcm<'a, 'b, T: UnderlyingBuffer>(
    buf: &'b mut DualBuffer<'a, T>,
    pack_len: usize,
) -> Option<Result<AnyPacketRef<'b>, ParserError>> {
    if !buf.can_drain_and_take(0, pack_len + 3) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            // TODO: shouldn't we drain pack_len + 3?
            buf.drain(2);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len + 2,
            }));
        }
        return None;
    }

    let maybe_data = buf.take(pack_len + 3);
    match maybe_data {
        Ok(data) => Some(Ok(AnyPacketRef::Rtcm(RtcmPacketRef::<'b> { data }))),
        Err(e) => Some(Err(e)),
    }
}

impl<T: UnderlyingBuffer> UbxRtcmParserIter<'_, T> {
    fn find_sync(&self) -> NextSync {
        for i in 0..self.buf.len() {
            if self.buf[i] == SYNC_CHAR_1 {
                return NextSync::Ubx(i);
            }
            if self.buf[i] == RTCM_SYNC_CHAR {
                return NextSync::Rtcm(i);
            }
        }
        NextSync::None
    }

    #[allow(clippy::should_implement_trait)]
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of https://github.com/rust-lang/rust/issues/44265
    pub fn next(&mut self) -> Option<Result<AnyPacketRef<'_>, ParserError>> {
        while self.buf.len() > 0 {
            match self.find_sync() {
                NextSync::Ubx(pos) => {
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
                    let maybe_packet = extract_packet_ubx(&mut self.buf, pack_len);
                    match maybe_packet {
                        Some(Ok(packet)) => return Some(Ok(AnyPacketRef::Ubx(packet))),
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    }
                },
                NextSync::Rtcm(pos) => {
                    self.buf.drain(pos);

                    // need to read 3 bytes total for sync char (1) + length (2)
                    if self.buf.len() < 3 {
                        return None;
                    }
                    // next 2 bytes contain 6 bits reserved + 10 bits length, big endian
                    let pack_len: usize =
                        (u16::from_be_bytes([self.buf[1], self.buf[2]]) & 0x03ff).into();

                    return extract_packet_rtcm(&mut self.buf, pack_len);
                },
                NextSync::None => {
                    self.buf.clear();
                    return None;
                },
            };
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ubx_packets::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_split_indexing() {
        let mut buf = vec![1, 2, 3, 4];
        let new = [5, 6, 7, 8];
        let dual = DualBuffer::new(&mut buf, &new[..]);
        for i in 0..8 {
            assert_eq!(dual[i], i as u8 + 1);
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    #[should_panic]
    fn dl_take_too_many() {
        let mut buf = vec![1, 2, 3, 4];
        let new = [];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);

            // This should panic
            let _ = dual.take(6);
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_take_range_underlying() {
        let mut buf = vec![1, 2, 3, 4];
        let new = [];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            let x = dual.take(3).unwrap();
            assert_eq!(x, &[1, 2, 3]);
        }
        assert_eq!(buf, &[4]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_take_range_new() {
        let mut buf = vec![];
        let new = [1, 2, 3, 4];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            let x = dual.take(3).unwrap();
            assert_eq!(x, &[1, 2, 3]);
        }
        assert_eq!(buf, &[4]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_take_range_overlapping() {
        let mut buf = vec![1, 2, 3, 4];
        let new = [5, 6, 7, 8];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            let x = dual.take(6).unwrap();
            assert_eq!(x, &[1, 2, 3, 4, 5, 6]);
        }
        assert_eq!(buf, &[7, 8]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_take_multi_ranges() {
        let mut buf = vec![1, 2, 3, 4, 5, 6, 7];
        let new = [8, 9, 10, 11, 12];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            assert_eq!(dual.take(3).unwrap(), &[1, 2, 3]);
            assert_eq!(dual.take(3).unwrap(), &[4, 5, 6]);
            assert_eq!(dual.take(3).unwrap(), &[7, 8, 9]);
            assert_eq!(dual.take(3).unwrap(), &[10, 11, 12]);
        }
        assert_eq!(buf, &[]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn dl_take_multi_ranges2() {
        let mut buf = vec![1, 2, 3, 4, 5, 6, 7];
        let new = [8, 9, 10, 11, 12];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            assert_eq!(dual.take(3).unwrap(), &[1, 2, 3]);
            assert_eq!(dual.take(6).unwrap(), &[4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(buf, &[10, 11, 12]);
    }

    #[test]
    fn dl_move_then_copy() {
        let mut buf = [0; 7];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        let new = [8, 9, 10, 11, 12];
        {
            let mut dual = DualBuffer::new(&mut buf, &new[..]);
            assert_eq!(dual.take(3).unwrap(), &[1, 2, 3]);
            assert_eq!(dual.take(6).unwrap(), &[4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(buf.len(), 3);
    }

    #[test]
    #[should_panic]
    fn dl_take_range_oom() {
        let mut buf = [0; 4];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        let new = [1, 2, 3, 4, 5, 6];

        let mut dual = DualBuffer::new(&mut buf, &new[..]);
        // This should throw
        assert!(
            matches!(dual.take(6), Err(ParserError::OutOfMemory { required_size }) if required_size == 6)
        );
    }

    #[test]
    fn dl_drain_partial_underlying() {
        let mut buf = [0; 4];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3]);
        let new = [4, 5, 6, 7, 8, 9];
        let mut dual = DualBuffer::new(&mut buf, &new[..]);

        dual.drain(2);
        assert_eq!(dual.len(), 7);
        assert_eq!(dual.take(4).unwrap(), &[3, 4, 5, 6]);
        assert_eq!(dual.len(), 3);
    }

    #[test]
    fn dl_drain() {
        let mut buf = [0; 4];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3]);
        let new = [4, 5, 6, 7, 8, 9];
        let mut dual = DualBuffer::new(&mut buf, &new[..]);

        dual.drain(5);
        assert_eq!(dual.take(3).unwrap(), &[6, 7, 8]);
        assert_eq!(dual.len(), 1);
    }

    #[test]
    fn dl_clear() {
        let mut buf = [0; 4];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3]);
        let new = [4, 5, 6, 7, 8, 9];
        let mut dual = DualBuffer::new(&mut buf, &new[..]);

        assert_eq!(dual.len(), 9);
        dual.clear();
        assert_eq!(dual.len(), 0);
    }

    #[test]
    fn dl_peek_raw() {
        let mut buf = [0; 4];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3]);
        let new = [4, 5, 6, 7, 8, 9];
        let dual = DualBuffer::new(&mut buf, &new[..]);

        let (a, b) = dual.peek_raw(2..6);
        assert_eq!(a, &[3]);
        assert_eq!(b, &[4, 5, 6]);
    }

    #[test]
    fn flb_clear() {
        let mut buf = [0; 16];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 7);
        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    #[test]
    #[should_panic]
    fn flb_index_outside_range() {
        let mut buf = [0; 16];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        let _ = buf[5..10];
    }

    #[test]
    fn flb_extend_outside_range() {
        let mut buf = [0; 16];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 16);
    }

    #[test]
    fn flb_drain() {
        let mut buf = [0; 16];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);

        buf.drain(3);
        assert_eq!(buf.len(), 4);
        assert_eq!(&buf[0..buf.len()], &[4, 5, 6, 7]);

        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 11);
        assert_eq!(&buf[0..buf.len()], &[4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn flb_drain_all() {
        let mut buf = [0; 16];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);

        buf.drain(7);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn flb_find() {
        let mut buf = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut buf = FixedLinearBuffer::new(&mut buf);
        assert_eq!(buf.find(5), None);
        buf.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(buf.find(5), None);
        buf.extend_from_slice(&[5, 6, 7, 8]);
        assert_eq!(buf.find(5), Some(4));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn parser_oom_processes_multiple_small_packets() {
        let packet = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        let mut bytes = vec![];
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);

        let mut buffer = [0; 10];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser = Parser::new(buffer);

        let mut it = parser.consume_ubx(&bytes);
        for _ in 0..5 {
            assert!(matches!(it.next(), Some(Ok(PacketRef::AckAck(_)))));
        }
        assert!(it.next().is_none());
    }

    #[test]
    fn parser_handle_garbage_first_byte() {
        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser = Parser::new(buffer);

        let bytes = [0xb5, 0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(it.next(), Some(Ok(PacketRef::AckAck(_)))));
            assert!(it.next().is_none());
        }
    }

    #[test]
    fn parser_oom_clears_buffer() {
        let bytes = CfgNav5Builder {
            mask: CfgNav5Params::DYN,
            dyn_model: NavDynamicModel::AirborneWithLess1gAcceleration,
            fix_mode: NavFixMode::Only3D,
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
            utc_standard: UtcStandardIdentifier::UtcChina,
            ..CfgNav5Builder::default()
        }
        .into_packet_bytes();

        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser = Parser::new(buffer);

        {
            let mut it = parser.consume_ubx(&bytes[0..8]);
            assert!(it.next().is_none());
        }

        {
            let mut it = parser.consume_ubx(&bytes[8..]);
            assert!(
                matches!(it.next(), Some(Err(ParserError::OutOfMemory { required_size })) if required_size == bytes.len() - 6)
            );
            assert!(it.next().is_none());
        }

        // Should now be empty, and we can parse a small packet
        let bytes = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(it.next(), Some(Ok(PacketRef::AckAck(_)))));
            assert!(it.next().is_none());
        }
    }

    #[test]
    fn parser_accepts_packet_array_underlying() {
        let bytes = CfgNav5Builder {
            mask: CfgNav5Params::DYN,
            dyn_model: NavDynamicModel::AirborneWithLess1gAcceleration,
            fix_mode: NavFixMode::Only3D,
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
            utc_standard: UtcStandardIdentifier::UtcChina,
            ..CfgNav5Builder::default()
        }
        .into_packet_bytes();

        let mut buffer = [0; 1024];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser = Parser::new(buffer);
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(it.next(), Some(Ok(PacketRef::CfgNav5(_)))));
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(feature = "std")]
    fn parser_accepts_packet_vec_underlying() {
        let bytes = CfgNav5Builder {
            mask: CfgNav5Params::DYN,
            dyn_model: NavDynamicModel::AirborneWithLess1gAcceleration,
            fix_mode: NavFixMode::Only3D,
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
            utc_standard: UtcStandardIdentifier::UtcChina,
            ..CfgNav5Builder::default()
        }
        .into_packet_bytes();

        let mut parser = Parser::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(it.next(), Some(Ok(PacketRef::CfgNav5(_)))));
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
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(PacketRef::CfgNav5(packet))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(PacketRef::CfgNav5(packet))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            },
            _ => {
                panic!()
            },
        }
        assert!(it.next().is_none());
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_max_payload_len() {
        assert!(MAX_PAYLOAD_LEN >= 1240);
    }
}
