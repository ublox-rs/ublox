use ublox::{
    CfgNav5Builder, CfgNav5DynModel, CfgNav5FixMode, CfgNav5Params, CfgNav5UtcStandard, PacketRef,
    Parser, ParserError, ParserIter,
};

macro_rules! my_vec {
        ($($x:expr),*) => {{
            let v: Vec<Result<(u8, u8), ParserError>> =  vec![$($x),*];
            v
        }}
    }

fn extract_only_ack_ack<T: ublox::UnderlyingBuffer>(
    mut it: ParserIter<T>,
) -> Vec<Result<(u8, u8), ParserError>> {
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
    assert_eq!(0, parser.buffer_len());

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
fn test_parse_cfg_nav5() {
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

    let mut parser = Parser::default();
    let mut found = false;
    let mut it = parser.consume(&bytes);
    while let Some(pack) = it.next() {
        match pack {
            Ok(PacketRef::CfgNav5(pack)) => {
                found = true;

                assert_eq!(CfgNav5Params::DYN, pack.mask());
                assert_eq!(
                    CfgNav5DynModel::AirborneWithLess1gAcceleration,
                    pack.dyn_model()
                );
                assert_eq!(CfgNav5FixMode::Only3D, pack.fix_mode());
                assert!((pack.fixed_alt() - 100.17).abs() < 0.01);
                assert_eq!(pack.fixed_alt_raw(), 10017);
                assert!((pack.fixed_alt_var() - 0.0017).abs() < 0.000_1);
                assert_eq!(17, pack.min_elev_degrees());
                assert!((pack.pdop() - 1.7).abs() < 0.1);
                assert!((pack.tdop() - 1.7).abs() < 0.1);
                assert_eq!(17, pack.pacc());
                assert_eq!(17, pack.tacc());
                assert!((pack.static_hold_thresh() - 2.17) < 0.01);
                assert_eq!(17, pack.dgps_time_out());
                assert_eq!(17, pack.cno_thresh_num_svs());
                assert_eq!(17, pack.cno_thresh());
                assert_eq!(0x1717, pack.static_hold_max_dist());
                assert_eq!(CfgNav5UtcStandard::UtcChina, pack.utc_standard());
            }
            _ => assert!(false),
        }
    }
    assert!(found);
}
