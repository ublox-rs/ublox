use crate::error::MemWriterError;
use alloc::vec::Vec;

/// LinearBuffer is an object which allows a user to write data into it and retrieve a slice referencing it.
/// LinearBuffers may be growable, like a Vec, or may have a fixed size, like a fixed-size array, or may be a Vec with a maximum permitted size.
pub trait LinearBuffer {
    fn set<E, ITER: core::iter::Iterator<Item = u8>>(
        &mut self,
        iter: ITER,
    ) -> Result<(), MemWriterError<E>>;
    fn clear(&mut self);
    fn get_ref(&self, size: usize) -> &[u8];
}

impl LinearBuffer for Vec<u8> {
    fn set<E, ITER: core::iter::Iterator<Item = u8>>(
        &mut self,
        iter: ITER,
    ) -> Result<(), MemWriterError<E>> {
        for x in iter {
            self.push(x);
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.drain(..);
    }

    fn get_ref(&self, _size: usize) -> &[u8] {
        self
    }
}

pub struct ArrayBuffer<'a> {
    underlying: &'a mut [u8],
}

impl<'a> ArrayBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> ArrayBuffer<'a> {
        ArrayBuffer { underlying: buf }
    }
}

impl<'a> LinearBuffer for ArrayBuffer<'a> {
    fn set<E, ITER: core::iter::Iterator<Item = u8>>(
        &mut self,
        iter: ITER,
    ) -> Result<(), MemWriterError<E>> {
        for (i, x) in iter.enumerate() {
            if i >= self.underlying.len() {
                return Err(MemWriterError::NotEnoughMem);
            }
            self.underlying[i] = x;
        }
        Ok(())
    }

    fn clear(&mut self) {
        // No-op
    }

    fn get_ref(&self, size: usize) -> &[u8] {
        &self.underlying[0..size]
    }
}
