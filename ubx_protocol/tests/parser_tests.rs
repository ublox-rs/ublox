use cpu_time::ProcessTime;
use rand::{thread_rng, Rng};
use std::{env, fs, path::Path};
use ubx_protocol::{PacketRef, Parser, ParserError, ParserIter};

#[test]
fn test_ack_ack_simple() {
    type ParseResult = Result<(u8, u8), ParserError>;
    fn extract_ack_ack(mut it: ParserIter) -> Vec<ParseResult> {
        let mut ret = vec![];
        while let Some(pack) = it.next() {
            match pack {
                Ok(PacketRef::AckAck(pack)) => {
                    ret.push(Ok((pack.class(), pack.msg_id())));
                }
                Err(err) => ret.push(Err(err)),
                _ => assert!(false),
            }
        }
        ret
    }
    macro_rules! my_vec {
        ($($x:expr),*) => {{
            let v: Vec<ParseResult> =  vec![$($x),*];
            v
        }}
    }

    let mut parser = Parser::default();
    assert!(parser.is_buffer_empty());
    assert_eq!(
        my_vec![],
        extract_ack_ack(parser.consume(&[])),
        "empty buffer parsing"
    );
    assert!(parser.is_buffer_empty());

    let full_pack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x6, 0x1, 0xf, 0x38];
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_ack_ack(parser.consume(&full_pack)),
        "full packet parsing"
    );
    assert!(parser.is_buffer_empty());

    let mut bad_pack = full_pack.clone();
    bad_pack[bad_pack.len() - 3] = 5;
    assert_eq!(
        my_vec![Err(ParserError::InvalidChecksum)],
        extract_ack_ack(parser.consume(&bad_pack)),
        "invalid checksum"
    );
    assert_eq!(bad_pack.len() - 2, parser.buffer_len());

    let mut two_packs = full_pack.to_vec();
    two_packs.extend_from_slice(&full_pack);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_ack_ack(parser.consume(&two_packs)),
        "two packets"
    );
    assert!(parser.is_buffer_empty());

    assert_eq!(
        my_vec![],
        extract_ack_ack(parser.consume(&full_pack[0..5])),
        "part of packet"
    );
    assert_eq!(5, parser.buffer_len());
    let mut rest_and_next = (&full_pack[5..]).to_vec();
    rest_and_next.extend_from_slice(&full_pack);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_ack_ack(parser.consume(&two_packs)),
        "two packets"
    );
    assert!(parser.is_buffer_empty());

    let mut garbage_before = vec![0x00, 0x06, 0x01, 0x0f, 0x38];
    garbage_before.extend_from_slice(&full_pack);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_ack_ack(parser.consume(&garbage_before)),
        "garbage before1"
    );

    let mut garbage_before = vec![0xb5, 0xb5, 0x62, 0x62, 0x38];
    garbage_before.extend_from_slice(&full_pack);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_ack_ack(parser.consume(&garbage_before)),
        "garbage before1"
    );
}

#[test]
#[ignore]
fn test_parse_big_file() {
    let ubx_big_log_path = env::var("UBX_BIG_LOG_PATH").unwrap();
    let ubx_big_log_path = Path::new(&ubx_big_log_path);

    let biglog = fs::read(ubx_big_log_path).unwrap();
    const MAX_SIZE: usize = 100;
    let mut read_sizes = Vec::with_capacity(biglog.len() / MAX_SIZE / 2);
    let mut rng = thread_rng();
    let mut i = 0;
    while i < biglog.len() {
        let chunk: usize = rng.gen_range(1, MAX_SIZE);
        let chunk = (biglog.len() - i).min(chunk);
        read_sizes.push(chunk);
        i += chunk;
    }

    let mut wrong_chksum = 0usize;
    let mut other_errors = 0usize;
    let mut nav_pos_llh = 0usize;
    let mut nav_stat = 0usize;
    let mut ack_ack = 0usize;
    let mut unknown = 0usize;

    let mut log = biglog.as_slice();
    let mut parser = Parser::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log.split_at(*chunk_size);
        log = rest;
        let mut it = parser.consume(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    PacketRef::AckAck(_) => ack_ack += 1,
                    PacketRef::NavPosLLH(_) => nav_pos_llh += 1,
                    PacketRef::NavStatus(_) => nav_stat += 1,
                    _ => unknown += 1,
                },
                Err(ParserError::InvalidChecksum) => wrong_chksum += 1,
                Err(_) => other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!(
        "parse time of {}: {:?}",
        ubx_big_log_path.display(),
        cpu_time
    );
    assert_eq!(0, wrong_chksum);
    assert_eq!(0, other_errors);
    assert_eq!(38291, nav_pos_llh);
    assert_eq!(38291, nav_stat);
    assert_eq!(120723, unknown);
    assert_eq!(1, ack_ack);
}
