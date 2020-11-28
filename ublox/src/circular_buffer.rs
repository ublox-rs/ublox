struct CircularBuffer<'a> {
    buf: &'a mut [u8],
    head: usize,
    tail: usize,
}

impl<'a> CircularBuffer<'a> {
    fn new(buf: &mut [u8]) -> CircularBuffer {
        CircularBuffer {
            buf: buf,
            head: 0,
            tail: 0,
        }
    }

    fn len(&self) -> usize {
        let len = self.buf.len() + self.tail - self.head;
        if len >= self.buf.len() {
            len - self.buf.len()
        } else {
            len
        }
    }

    /// Returns true if the push was successful, false otherwise
    fn push(&mut self, data: u8) -> bool {
        if (self.tail + 1) % self.buf.len() == self.head {
            return false;
        }
        self.buf[self.tail] = data;
        self.tail += 1;
        true
    }

    /// Returns None if there was no element available to pop
    fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            return None;
        }
        let x = self.buf[self.head];
        self.head += 1;
        Some(x)
    }

    /// Returns the element at the given index, panicing if the index is invalid
    fn at(&self, idx: usize) -> u8 {
        assert!(idx < self.len());
        let idx = self.head + idx;
        let idx = if idx >= self.len() {
            idx - self.len()
        } else {
            idx
        };
        self.buf[idx]
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    /// Returns the number of elements we could consume
    fn extend_from_slice(&mut self, new_data: &[u8]) -> usize {
        for (i, x) in new_data.iter().enumerate() {
            if !self.push(*x) {
                return i;
            }
        }
        return new_data.len();
    }

    fn iter(&'a self) -> CircularBufferIter<'_, 'a> {
        CircularBufferIter {
            buf: self,
            offset: 0,
        }
    }
}

struct CircularBufferIter<'a, 'b> {
    buf: &'a CircularBuffer<'b>,
    offset: usize,
}

impl<'a, 'b> core::iter::Iterator for CircularBufferIter<'a, 'b> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.buf.len() {
            let x = self.buf.at(self.offset);
            self.offset += 1;
            Some(x)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cb_push_works() {
        let mut buf = [0; 5];
        let mut buf = CircularBuffer::new(&mut buf);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.push(13), true);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.push(15), true);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop(), Some(13));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pop(), Some(15));
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.pop(), None);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn cb_fills() {
        let mut buf = [0; 5];
        let mut buf = CircularBuffer::new(&mut buf);
        assert_eq!(buf.push(13), true);
        assert_eq!(buf.push(14), true);
        assert_eq!(buf.push(15), true);
        assert_eq!(buf.push(16), true);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.push(17), false);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.pop(), Some(13));
        assert_eq!(buf.pop(), Some(14));
        assert_eq!(buf.pop(), Some(15));
        assert_eq!(buf.pop(), Some(16));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn cbiter_iters() {
        let mut buf = [0; 5];
        let mut buf = CircularBuffer::new(&mut buf);
        let expected: [u8; 4] = [13, 14, 15, 16];
        buf.push(13);
        buf.push(14);
        buf.push(15);
        buf.push(16);
        for (a, b) in buf.iter().zip(expected.iter()) {
            assert_eq!(a, *b);
        }
    }
}
