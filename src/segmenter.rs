use crate::error::{Error, Result};
use crate::ubx_packets::Packet;
use std::collections::VecDeque;

fn segment_stream(data: &mut VecDeque<u8>) -> Result<Option<Packet>> {
    loop {
        match (data.get(0), data.get(1)) {
            (Some(0xb5), Some(0x62)) => {
                break;
            }
            (_, None) => {
                return Ok(None);
            }
            (Some(_), Some(_)) => {
                data.pop_front();
            }
            (None, Some(_)) => {
                panic!("This should never happen!");
            }
        }
    }
    if data.len() < 6 {
        return Ok(None);
    }

    // Check length
    let length = (data[4] as u16 + data[5] as u16 * 256) as usize;
    if data.len() < length + 8 {
        return Ok(None);
    }

    // Check checksum
    let expected_checksum = data[length + 6] as u16 + data[length + 7] as u16 * 256;

    let data: Vec<u8> = data.drain(..length + 8).collect();
    let mut cka = 0;
    let mut ckb = 0;
    for c in &data[2..length + 6] {
        cka = ((cka as usize + *c as usize) & 0xFF) as u8;
        ckb = ((cka as usize + ckb as usize) & 0xFF) as u8;
    }
    let checksum = cka as u16 + ckb as u16 * 256;
    if checksum != expected_checksum {
        return Err(Error::InvalidChecksum);
    }

    // Parse
    let classid = data[2];
    let msgid = data[3];
    let data = &data[6..length + 6];
    Ok(Some(Packet::deserialize(classid, msgid, data)?))
}

pub struct Segmenter {
    buffer: VecDeque<u8>,
}

impl Segmenter {
    pub fn new() -> Self {
        Segmenter {
            buffer: VecDeque::new(),
        }
    }

    pub fn consume(&mut self, data: &[u8]) -> Result<Option<Packet>> {
        for c in data {
            self.buffer.push_back(*c);
        }
        segment_stream(&mut self.buffer)
    }

    pub fn consume_all(&mut self, data: &[u8]) -> Result<Vec<Packet>> {
        for c in data {
            self.buffer.push_back(*c);
        }
        let mut packets = vec![];
        loop {
            match segment_stream(&mut self.buffer)? {
                Some(packet) => {
                    packets.push(packet);
                }
                None => {
                    return Ok(packets);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ubx_packets::AckAck;

    #[test]
    fn segmentation_works() {
        let mut buf: VecDeque<u8> =
            vec![0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 0x02, 0x03, 0x0d, 0x32]
                .into_iter()
                .collect();
        let res = segment_stream(&mut buf);
        assert_eq!(
            res.unwrap().unwrap(),
            Packet::AckAck(AckAck {
                classid: 0x02,
                msgid: 0x03,
            })
        );
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn segmentation_skips_to_sync_leaves_extras() {
        let mut buf: VecDeque<u8> = vec![
            0x64, 0x12, 0x06, 0xb5, 0x01, 0x62, 0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 0x02, 0x03,
            0x0d, 0x32, 0x1, 0x2, 0x3,
        ]
        .into_iter()
        .collect();
        let res = segment_stream(&mut buf);
        assert_eq!(
            res.unwrap().unwrap(),
            Packet::AckAck(AckAck {
                classid: 0x02,
                msgid: 0x03,
            })
        );
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn consume_all_consumes_all() {
        let buf = vec![
            0x64, 0x12, 0x06, 0xb5, 0x01, 0x62, 0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 0x02, 0x03,
            0x0d, 0x32, 0x1, 0x2, 0x3,
        ];
        let mut buf2 = vec![];
        buf2.extend_from_slice(&buf[..]);
        buf2.extend_from_slice(&buf[..]);
        let buf = buf2;

        let mut p = Segmenter::new();

        let res = p.consume_all(&buf).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            Packet::AckAck(AckAck {
                classid: 0x02,
                msgid: 0x03,
            })
        );
        assert_eq!(
            res[1],
            Packet::AckAck(AckAck {
                classid: 0x02,
                msgid: 0x03,
            })
        );
    }
}
