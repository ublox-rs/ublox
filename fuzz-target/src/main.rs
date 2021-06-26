#[macro_use]
extern crate afl;
extern crate ublox;

fn parse(bufsize: usize, chunksize: usize, data: &[u8]) {
    let mut buf = vec![0; bufsize];
    let buf = ublox::FixedLinearBuffer::new(&mut buf[..]);
    let mut parser = ublox::Parser::new(buf);
    for chunk in data.chunks(chunksize) {
        // parser.consume adds the buffer to its internal buffer, and
        // returns an iterator-like object we can use to process the packets
        let mut it = parser.consume(chunk);
        loop {
            match it.next() {
                Some(Ok(packet)) => {}
                Some(Err(_)) => {}
                None => {
                    // We've eaten all the packets we have
                    break;
                }
            }
        }
    }

    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    {
        // Clear out the buffer
        let mut num_acks = 0;
        let mut it = parser.consume(&ack_ack);
        loop {
            match it.next() {
                Some(Ok(ublox::PacketRef::AckAck { .. })) => {
                    num_acks += 1;
                }
                Some(Ok(ublox::PacketRef::Unknown(_))) => {
                    // It's possible that we might get Unknown
                }
                Some(Err(e)) => {
                    // The buffer might contain any of a variety of errors
                }
                _ => {
                    // Parsing other packets or ending the iteration is a failure
                    break;
                }
            }
        }
        // We could conceivably end up with >1 ack, if the underlying buffer ended in
        // a too-large packet, a subset of which was an ack.
        assert!(num_acks >= 1);
    }
    let mut it = parser.consume(&ack_ack);
    loop {
        match it.next() {
            Some(Ok(ublox::PacketRef::AckAck { .. })) => {
                // This is what we expect
                break;
            }
            Some(Err(e)) => {
                // The buffer might contain any of a variety of errors
            }
            _ => {
                // Parsing other packets or ending the iteration is a failure
                panic!();
            }
        }
    }
    assert!(it.next().is_none());
}

fn main() {
    fuzz!(|data: &[u8]| {
        if data.len() > 2 {
            let bufsize = 10; //data[0] as usize;
            let chunksize = data[1] as usize;
            if bufsize >= 10 && chunksize != 0 {
                parse(bufsize, chunksize, &data[2..]);
            }
        }
    });
}
