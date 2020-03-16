use cpu_time::ProcessTime;
use rand::{thread_rng, Rng};
use std::{env, fs, path::Path};
use ublox::{PacketRef, Parser, ParserError, ParserIter};

macro_rules! my_vec {
        ($($x:expr),*) => {{
            let v: Vec<Result<(u8, u8), ParserError>> =  vec![$($x),*];
            v
        }}
    }

fn extract_only_ack_ack(mut it: ParserIter) -> Vec<Result<(u8, u8), ParserError>> {
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

static FULL_ACK_ACK_PACK: [u8; 10] = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x6, 0x1, 0xf, 0x38];

#[test]
fn test_parse_empty_buffer() {
    let mut parser = Parser::default();
    assert!(parser.is_buffer_empty());
    assert_eq!(my_vec![], extract_only_ack_ack(parser.consume(&[])));
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_byte_by_byte() {
    let mut parser = Parser::default();
    for b in FULL_ACK_ACK_PACK.iter().take(FULL_ACK_ACK_PACK.len() - 1) {
        assert_eq!(my_vec![], extract_only_ack_ack(parser.consume(&[*b])));
        assert!(!parser.is_buffer_empty());
    }
    let last_byte = FULL_ACK_ACK_PACK[FULL_ACK_ACK_PACK.len() - 1];
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&[last_byte])),
    );
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_in_one_go() {
    let mut parser = Parser::default();
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&FULL_ACK_ACK_PACK)),
    );
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_bad_checksum() {
    let mut parser = Parser::default();
    let mut bad_pack = FULL_ACK_ACK_PACK.clone();
    bad_pack[bad_pack.len() - 3] = 5;
    assert_eq!(
        my_vec![Err(ParserError::InvalidChecksum {
            expect: 0x380f,
            got: 0x3c13
        })],
        extract_only_ack_ack(parser.consume(&bad_pack)),
    );
    assert_eq!(bad_pack.len() - 2, parser.buffer_len());

    let mut two_packs = FULL_ACK_ACK_PACK.to_vec();
    two_packs.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&two_packs)),
    );
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_parted_two_packets() {
    let mut parser = Parser::default();
    assert_eq!(
        my_vec![],
        extract_only_ack_ack(parser.consume(&FULL_ACK_ACK_PACK[0..5])),
    );
    assert_eq!(5, parser.buffer_len());
    let mut rest_and_next = (&FULL_ACK_ACK_PACK[5..]).to_vec();
    rest_and_next.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&rest_and_next)),
    );
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_two_in_one_go() {
    let mut parser = Parser::default();
    let mut two_packs = FULL_ACK_ACK_PACK.to_vec();
    two_packs.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&two_packs))
    );
    assert!(parser.is_buffer_empty());
}

#[test]
fn test_parse_ack_ack_garbage_before() {
    let mut parser = Parser::default();
    let mut garbage_before = vec![0x00, 0x06, 0x01, 0x0f, 0x38];
    garbage_before.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&garbage_before)),
        "garbage before1"
    );
    assert!(parser.is_buffer_empty());

    let mut garbage_before = vec![0xb5, 0xb5, 0x62, 0x62, 0x38];
    garbage_before.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_only_ack_ack(parser.consume(&garbage_before)),
        "garbage before2"
    );
    assert!(parser.is_buffer_empty());
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
                    PacketRef::NavPosLlh(_) => nav_pos_llh += 1,
                    PacketRef::NavStatus(_) => nav_stat += 1,
                    _ => unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => wrong_chksum += 1,
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
