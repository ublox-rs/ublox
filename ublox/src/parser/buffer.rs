#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::ParserError;
use core::cmp::min;

/// This trait represents an underlying buffer used for the Parser. We provide
/// implementations for `Vec<u8>`, `[u8; N]`([FixedBuffer]), and for `&mut [u8]` ([FixedLinearBuffer]), if you want to
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

/// Holds a mutable reference to a fixed byte array
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
            panic!(
                "index out of bounds: the len is {len} but the index is {idx}",
                len = self.len,
                idx = index.end
            );
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
        let available_space = self.buffer.len() - self.len;
        let to_copy = min(other.len(), available_space);

        self.buffer[self.len..self.len + to_copy].copy_from_slice(&other[..to_copy]);
        self.len += to_copy;

        other.len() - to_copy // Remainder that didn't fit in the buffer
    }

    fn drain(&mut self, count: usize) {
        if count >= self.len {
            self.len = 0;
            return;
        }

        let remaining = self.len - count;
        // Move the remaining elements to the start of the buffer
        self.buffer.copy_within(count..self.len, 0);
        self.len = remaining;
    }
}

/// An owned, fixed-size linear buffer with a capacity known at compile time.
///
/// This struct owns its data in a `[u8; N]` array.
/// It implements the `UnderlyingBuffer` trait, making it
/// a drop-in replacement for `FixedLinearBuffer` where owned data is preferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedBuffer<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuffer<N> {
    /// Creates a new, empty `FixedBuffer`.
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> Default for FixedBuffer<N> {
    /// Creates a new, empty `FixedBuffer`.
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> core::ops::Index<core::ops::Range<usize>> for FixedBuffer<N> {
    type Output = [u8];

    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        if index.end > self.len {
            panic!(
                "index out of bounds: the len is {len} but the index is {idx}",
                len = self.len,
                idx = index.end
            );
        }
        &self.buffer[index]
    }
}

impl<const N: usize> core::ops::Index<usize> for FixedBuffer<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!(
                "index out of bounds: the len is {len} but the index is {idx}",
                len = self.len,
                idx = index
            );
        }
        &self.buffer[index]
    }
}

impl<const N: usize> UnderlyingBuffer for FixedBuffer<N> {
    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
    }

    fn max_capacity(&self) -> usize {
        N
    }

    fn extend_from_slice(&mut self, other: &[u8]) -> usize {
        let available_space = N - self.len;
        let to_copy = min(other.len(), available_space);

        self.buffer[self.len..self.len + to_copy].copy_from_slice(&other[..to_copy]);
        self.len += to_copy;

        other.len() - to_copy // Remainder that didn't fit in the buffer
    }

    fn drain(&mut self, count: usize) {
        if count >= self.len {
            self.len = 0;
            return;
        }

        let remaining = self.len - count;
        // Move the remaining elements to the start of the buffer
        self.buffer.copy_within(count..self.len, 0);
        self.len = remaining;
    }
}

/// Stores two buffers: A "base" and a "new" buffer. Exposes these as the same buffer,
/// copying data from the "new" buffer to the base buffer as required to maintain that
/// illusion.
pub(crate) struct DualBuffer<'a, T: UnderlyingBuffer> {
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
    pub(crate) const fn new(buf: &'a mut T, new_buf: &'a [u8]) -> Self {
        Self {
            buf,
            off: 0,
            new_buf,
            new_buf_offset: 0,
        }
    }

    /// Clears all elements - equivalent to buf.drain(buf.len())
    pub(crate) fn clear(&mut self) {
        self.drain(self.len());
    }

    /// Remove count elements without providing a view into them.
    pub(crate) fn drain(&mut self, count: usize) {
        let underlying_bytes = core::cmp::min(self.buf.len() - self.off, count);
        let new_bytes = count.saturating_sub(underlying_bytes);

        self.off += underlying_bytes;
        self.new_buf_offset += new_bytes;
    }

    /// Return the total number of accessible bytes in this view. Note that you may
    /// not be able to take() this many bytes at once, if the total number of bytes
    /// is more than the underlying store can fit.
    pub(crate) fn len(&self) -> usize {
        self.buf.len() - self.off + self.new_buf.len() - self.new_buf_offset
    }

    // Returns the number of bytes which would be lost (because they can't be copied into
    // the underlying storage) if this DualBuffer were dropped.
    pub(crate) fn potential_lost_bytes(&self) -> usize {
        if self.len() <= self.buf.max_capacity() {
            0
        } else {
            self.len() - self.buf.max_capacity()
        }
    }

    pub(crate) fn can_drain_and_take(&self, drain: usize, take: usize) -> bool {
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

    pub(crate) fn peek_raw(&self, range: core::ops::Range<usize>) -> (&[u8], &[u8]) {
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
    pub(crate) fn take(&mut self, count: usize) -> Result<&[u8], ParserError> {
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

#[cfg(test)]
mod test {
    use super::*;

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
        assert!(buf.is_empty());
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
    fn dl_move_then_copy_fixed_lin_buf() {
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
    fn dl_move_then_copy_fixed_buf() {
        let mut buf = FixedBuffer::<7>::new();
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
    fn dl_take_range_oom_lin_buf() {
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
    #[should_panic]
    fn dl_take_range_oom_fixed_buf() {
        let new = [1, 2, 3, 4, 5, 6];
        let mut buf = FixedBuffer::<4>::new();

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

    #[test]
    fn fixed_buf_clear() {
        let mut buf = FixedBuffer::<16>::new();
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 7);
        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    #[test]
    #[should_panic]
    fn fixed_buf_index_outside_range() {
        let mut buf = FixedBuffer::<16>::new();
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        let _ = buf[5..10];
    }

    #[test]
    fn fixed_buf_extend_outside_range() {
        let mut buf = FixedBuffer::<16>::new();
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 16);
    }

    #[test]
    fn fixed_buf_drain() {
        let mut buf = FixedBuffer::<16>::new();
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);

        buf.drain(3);
        assert_eq!(buf.len(), 4);
        assert_eq!(&buf[0..buf.len()], &[4, 5, 6, 7]);

        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(buf.len(), 11);
        assert_eq!(&buf[0..buf.len()], &[4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn fixed_buf_drain_all() {
        let mut buf = FixedBuffer::<16>::new();
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);

        buf.drain(7);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn fixed_buf_find() {
        let mut buf = FixedBuffer::<16>::new();
        assert_eq!(buf.find(5), None);
        buf.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(buf.find(5), None);
        buf.extend_from_slice(&[5, 6, 7, 8]);
        assert_eq!(buf.find(5), Some(4));
    }
}
