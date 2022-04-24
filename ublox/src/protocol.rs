use crate::match_packet;
use crate::{PacketRef, ParserError};

#[derive(Debug)]
pub enum BufferHeadContents<'a> {
    /// The first N bytes are garbage
    Garbage(usize),

    /// We think there's a header, but there aren't enough bytes to be sure
    IncompleteHeader,

    /// The buffer requires N more bytes
    Incomplete(usize),

    /// We found a N-byte packet but the checksum didn't match
    InvalidChecksum(usize),

    /// The first N bytes are a packet, parse it and advance the buffer
    Packet(PacketRef<'a>),

    InvalidPacket(ParserError),
}

pub fn parse_buffer(buf: &[u8]) -> BufferHeadContents {
    // Scan the buffer for the start [b5 62]
    let mut start_offset = None;
    for i in 0..buf.len() - 1 {
        if buf[i] == 0xb5 && buf[i + 1] == 0x62 {
            start_offset = Some(i);
            break;
        }
    }
    if start_offset == None {
        if buf.len() > 0 && buf[buf.len() - 1] == 0xb5 {
            if buf.len() == 1 {
                return BufferHeadContents::IncompleteHeader;
            }
            return BufferHeadContents::Garbage(buf.len() - 1);
        }
        return BufferHeadContents::Garbage(buf.len());
    }
    let start_offset = start_offset.unwrap();
    if start_offset > 0 {
        return BufferHeadContents::Garbage(start_offset);
    }
    if buf.len() < 6 {
        return BufferHeadContents::IncompleteHeader;
    }

    let class = buf[2];
    let id = buf[3];
    let length = buf[5] as usize * 256 + buf[4] as usize;
    /*if !crate::is_valid_packet_length(class, id, length) {
        return BufferHeadContents::Garbage(2);
    }*/
    if buf.len() < 6 + length + 2 {
        return BufferHeadContents::Incomplete(6 + length + 2 - buf.len());
    }
    let expected_checksum = crate::ubx_checksum(&buf[2..6 + length]);
    let actual_checksum = (buf[6 + length], buf[6 + length + 1]);
    if expected_checksum != actual_checksum {
        return BufferHeadContents::InvalidChecksum(6 + length + 2);
    }
    match match_packet(class, id, &buf[6..6 + length + 2]) {
        Ok(p) => BufferHeadContents::Packet(p),
        Err(e) => BufferHeadContents::InvalidPacket(e),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check(head1: BufferHeadContents, head2: BufferHeadContents) {
        let ok = match (head1, head2) {
            (BufferHeadContents::Garbage(n), BufferHeadContents::Garbage(n2)) => n == n2,
            (BufferHeadContents::IncompleteHeader, BufferHeadContents::IncompleteHeader) => true,
            (BufferHeadContents::Incomplete(n), BufferHeadContents::Incomplete(n2)) => n == n2,
            (BufferHeadContents::Packet(_), BufferHeadContents::Packet(_)) => true,
            (BufferHeadContents::InvalidChecksum(n), BufferHeadContents::InvalidChecksum(n2)) => {
                n == n2
            }
            _ => false,
        };
        assert!(ok);
    }

    #[test]
    fn parse_single_packet() {
        check(parse_buffer(&[0xb5]), BufferHeadContents::IncompleteHeader);
        check(
            parse_buffer(&[0xb5, 0x62]),
            BufferHeadContents::IncompleteHeader,
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5]),
            BufferHeadContents::IncompleteHeader,
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1]),
            BufferHeadContents::IncompleteHeader,
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2]),
            BufferHeadContents::IncompleteHeader,
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0]),
            BufferHeadContents::Incomplete(4),
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4]),
            BufferHeadContents::Incomplete(3),
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5]),
            BufferHeadContents::Incomplete(2),
        );
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11]),
            BufferHeadContents::Incomplete(1),
        );
        match parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38]) {
            BufferHeadContents::Packet(_) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn discard_initial_garbage() {
        check(
            parse_buffer(&[
                0xb5, 0x13, 0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38,
            ]),
            BufferHeadContents::Garbage(2),
        );
    }

    #[test]
    fn invalid_checksum() {
        check(
            parse_buffer(&[0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x39]),
            BufferHeadContents::InvalidChecksum(10),
        );
    }

    #[test]
    fn incomplete_header() {
        check(
            parse_buffer(&[0x13, 0x14, 0xb5, 0x62]),
            BufferHeadContents::Garbage(2),
        );
        check(
            parse_buffer(&[0x13, 0x14, 0x15, 0xb5]),
            BufferHeadContents::Garbage(3),
        );
    }
}
