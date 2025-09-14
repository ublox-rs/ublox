#![cfg(feature = "alloc")]

use ublox::{
    ack::{AckAckOwned, AckAckRef},
    cfg_nav5::{CfgNav5Builder, CfgNav5Params, CfgNav5Ref, NavDynamicModel, NavFixMode},
    Parser, ParserError, UbxPacket, UbxParserIter, UtcStandardIdentifier,
};

macro_rules! my_vec {
        ($($x:expr),*) => {{
            let v: Vec<Result<(u8, u8), ParserError>> =  vec![$($x),*];
            v
        }}
    }

#[cfg(feature = "ubx_proto14")]
fn extract_only_ack_ack_proto14<T: ublox::UnderlyingBuffer>(
    mut it: UbxParserIter<T, ublox::proto17::Proto17>,
) -> Vec<Result<(u8, u8), ParserError>> {
    let mut ret = vec![];
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto17(ublox::proto17::PacketRef::AckAck(pack))) => {
                ret.push(Ok((pack.class(), pack.msg_id())));
            },
            Err(err) => ret.push(Err(err)),
            _ => {}, // Ignore other packet types instead of panic
        }
    }
    ret
}

#[cfg(feature = "ubx_proto23")]
fn extract_only_ack_ack_proto23<T: ublox::UnderlyingBuffer>(
    mut it: UbxParserIter<T, ublox::proto23::Proto23>,
) -> Vec<Result<(u8, u8), ParserError>> {
    let mut ret = vec![];
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto23(ublox::proto23::PacketRef::AckAck(pack))) => {
                ret.push(Ok((pack.class(), pack.msg_id())));
            },
            Err(err) => ret.push(Err(err)),
            _ => {},
        }
    }
    ret
}

#[cfg(feature = "ubx_proto27")]
fn extract_only_ack_ack_proto27<T: ublox::UnderlyingBuffer>(
    mut it: UbxParserIter<T, ublox::proto27::Proto27>,
) -> Vec<Result<(u8, u8), ParserError>> {
    let mut ret = vec![];
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto27(ublox::proto27::PacketRef::AckAck(pack))) => {
                ret.push(Ok((pack.class(), pack.msg_id())));
            },
            Err(err) => ret.push(Err(err)),
            _ => {},
        }
    }
    ret
}

#[cfg(feature = "ubx_proto31")]
fn extract_only_ack_ack_proto31<T: ublox::UnderlyingBuffer>(
    mut it: UbxParserIter<T, ublox::proto31::Proto31>,
) -> Vec<Result<(u8, u8), ParserError>> {
    let mut ret = vec![];
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto31(ublox::proto31::PacketRef::AckAck(pack))) => {
                ret.push(Ok((pack.class(), pack.msg_id())));
            },
            Err(err) => ret.push(Err(err)),
            _ => {},
        }
    }
    ret
}

static FULL_ACK_ACK_PACK: [u8; 10] = [0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 0x06, 0x01, 0x0f, 0x38];

fn test_util_empty_buffer_asserts<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    assert!(parser.is_buffer_empty());
    assert_eq!(my_vec![], extract_fn(parser.consume_ubx(&[])));
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_empty_buffer_proto14() {
    test_util_empty_buffer_asserts(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_empty_buffer_proto23() {
    test_util_empty_buffer_asserts(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}
#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_empty_buffer_proto27() {
    test_util_empty_buffer_asserts(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}
#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_empty_buffer_proto31() {
    test_util_empty_buffer_asserts(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_byte_by_byte_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    for b in FULL_ACK_ACK_PACK.iter().take(FULL_ACK_ACK_PACK.len() - 1) {
        assert_eq!(my_vec![], extract_fn(parser.consume_ubx(&[*b])));
        assert!(!parser.is_buffer_empty());
    }
    let last_byte = FULL_ACK_ACK_PACK[FULL_ACK_ACK_PACK.len() - 1];
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_fn(parser.consume_ubx(&[last_byte])),
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_byte_by_byte_proto14() {
    test_util_byte_by_byte_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_byte_by_byte_proto23() {
    test_util_byte_by_byte_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}
#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_byte_by_byte_proto27() {
    test_util_byte_by_byte_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_byte_by_byte_proto31() {
    test_util_byte_by_byte_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_in_one_go_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_fn(parser.consume_ubx(&FULL_ACK_ACK_PACK)),
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_in_one_go_proto14() {
    test_util_in_one_go_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}
#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_in_one_go_proto23() {
    test_util_in_one_go_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}
#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_in_one_go_proto27() {
    test_util_in_one_go_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}
#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_in_one_go_proto31() {
    test_util_in_one_go_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_bad_checksum_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    let mut bad_pack = FULL_ACK_ACK_PACK;
    bad_pack[bad_pack.len() - 3] = 5;
    assert_eq!(
        my_vec![Err(ParserError::InvalidChecksum {
            expect: 0x380f,
            got: 0x3c13
        })],
        extract_fn(parser.consume_ubx(&bad_pack)),
    );
    assert_eq!(0, parser.buffer_len());

    let mut two_packs = FULL_ACK_ACK_PACK.to_vec();
    two_packs.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_fn(parser.consume_ubx(&two_packs)),
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_bad_checksum_proto14() {
    test_util_bad_checksum_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}
#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_bad_checksum_proto23() {
    test_util_bad_checksum_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}
#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_bad_checksum_proto27() {
    test_util_bad_checksum_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}
#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_bad_checksum_proto31() {
    test_util_bad_checksum_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_parted_two_packets_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    assert_eq!(
        my_vec![],
        extract_fn(parser.consume_ubx(&FULL_ACK_ACK_PACK[0..5])),
    );
    assert_eq!(5, parser.buffer_len());
    let mut rest_and_next = (FULL_ACK_ACK_PACK[5..]).to_vec();
    rest_and_next.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_fn(parser.consume_ubx(&rest_and_next)),
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_parted_two_packets_proto14() {
    test_util_parted_two_packets_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}
#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_parted_two_packets_proto23() {
    test_util_parted_two_packets_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_parted_two_packets_proto27() {
    test_util_parted_two_packets_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_parted_two_packets_proto31() {
    test_util_parted_two_packets_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_two_in_one_go_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    let mut two_packs = FULL_ACK_ACK_PACK.to_vec();
    two_packs.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1)), Ok((6, 1))],
        extract_fn(parser.consume_ubx(&two_packs))
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_two_in_one_go_proto14() {
    test_util_two_in_one_go_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}
#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_two_in_one_go_proto23() {
    test_util_two_in_one_go_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_two_in_one_go_proto27() {
    test_util_two_in_one_go_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_two_in_one_go_proto31() {
    test_util_two_in_one_go_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_garbage_before_assert<T, P, F>(mut parser: Parser<T, P>, extract_fn: F)
where
    T: ublox::UnderlyingBuffer + Default,
    P: ublox::UbxProtocol,
    F: Fn(UbxParserIter<T, P>) -> Vec<Result<(u8, u8), ParserError>>,
{
    let mut garbage_before = vec![0x00, 0x06, 0x01, 0x0f, 0x38];
    garbage_before.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_fn(parser.consume_ubx(&garbage_before)),
        "garbage before1"
    );
    assert!(parser.is_buffer_empty());

    let mut garbage_before = vec![0xb5, 0xb5, 0x62, 0x62, 0x38];
    garbage_before.extend_from_slice(&FULL_ACK_ACK_PACK);
    assert_eq!(
        my_vec![Ok((6, 1))],
        extract_fn(parser.consume_ubx(&garbage_before)),
        "garbage before2"
    );
    assert!(parser.is_buffer_empty());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_ack_ack_garbage_before_proto14() {
    test_util_garbage_before_assert(
        Parser::<_, ublox::proto17::Proto17>::default(),
        extract_only_ack_ack_proto14,
    );
}
#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_ack_ack_garbage_before_proto23() {
    test_util_garbage_before_assert(
        Parser::<_, ublox::proto23::Proto23>::default(),
        extract_only_ack_ack_proto23,
    );
}
#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_ack_ack_garbage_before_proto27() {
    test_util_garbage_before_assert(
        Parser::<_, ublox::proto27::Proto27>::default(),
        extract_only_ack_ack_proto27,
    );
}
#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_ack_ack_garbage_before_proto31() {
    test_util_garbage_before_assert(
        Parser::<_, ublox::proto31::Proto31>::default(),
        extract_only_ack_ack_proto31,
    );
}

fn test_util_cfg_nav5_bytes() -> [u8; 44] {
    CfgNav5Builder {
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
    .into_packet_bytes()
}

fn test_util_assert_expected_cfg_nav5(pack: CfgNav5Ref) {
    assert_eq!(CfgNav5Params::DYN, pack.mask());
    assert_eq!(
        NavDynamicModel::AirborneWithLess1gAcceleration,
        pack.dyn_model()
    );
    assert_eq!(NavFixMode::Only3D, pack.fix_mode());
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
    assert_eq!(UtcStandardIdentifier::UtcChina, pack.utc_standard());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_parse_cfg_nav5_proto14() {
    use ublox::proto17::{PacketRef, Proto17};
    let bytes = test_util_cfg_nav5_bytes();

    let mut parser = Parser::<_, Proto17>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&bytes);
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto17(PacketRef::CfgNav5(pack))) => {
                found = true;
                test_util_assert_expected_cfg_nav5(pack);
            },
            _ => panic!(),
        }
    }
    assert!(found);
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_parse_cfg_nav5_proto23() {
    use ublox::proto23::{PacketRef, Proto23};
    let bytes = test_util_cfg_nav5_bytes();

    let mut parser = Parser::<_, Proto23>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&bytes);
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto23(PacketRef::CfgNav5(pack))) => {
                found = true;
                test_util_assert_expected_cfg_nav5(pack);
            },
            _ => panic!(),
        }
    }
    assert!(found);
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_parse_cfg_nav5_proto27() {
    use ublox::proto27::{PacketRef, Proto27};
    let bytes = test_util_cfg_nav5_bytes();

    let mut parser = Parser::<_, Proto27>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&bytes);
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto27(PacketRef::CfgNav5(pack))) => {
                found = true;
                test_util_assert_expected_cfg_nav5(pack);
            },
            _ => panic!(),
        }
    }
    assert!(found);
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_parse_cfg_nav5_proto31() {
    use ublox::proto31::{PacketRef, Proto31};
    let bytes = test_util_cfg_nav5_bytes();

    let mut parser = Parser::<_, Proto31>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&bytes);
    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto31(PacketRef::CfgNav5(pack))) => {
                found = true;
                test_util_assert_expected_cfg_nav5(pack);
            },
            _ => panic!(),
        }
    }
    assert!(found);
}

#[cfg(feature = "serde")]
const RET_ESF_MEAS_SERIALIZE: [u8; 24] = [
    181, 98, 16, 2, 16, 0, 243, 121, 129, 1, 24, 8, 0, 0, 77, 100, 0, 11, 211, 148, 129, 1, 213,
    198,
];

#[cfg(feature = "serde")]
fn test_util_esf_meas_assert_expected_json(pack: UbxPacket) {
    // Match full packet including class & msg id
    let expected_packet_json = serde_json::json! {
        {
          "class": 16,
          "msg_id": 2,
          "itow": 25262579,
          "flags": 2072,
          "id": 0,
          "data": [
            {
              "data_type": "Speed",
              "data_field": 25677
            }
          ],
          "calib_tag": 25269459
        }
    };
    let expected_esf_meas_json = serde_json::json! {
        {
          "itow": 25262579,
          "flags": 2072,
          "id": 0,
          "data": [
            {
              "data_type": "Speed",
              "data_field": 25677
            }
          ],
          "calib_tag": 25269459
        }
    };
    match pack {
        UbxPacket::Proto17(_) => unreachable!("Does not support ESF MEAS"),
        UbxPacket::Proto23(packet_ref) => {
            let actual = serde_json::to_value(&packet_ref).unwrap();
            assert_eq!(expected_packet_json, actual);
            if let ublox::proto23::PacketRef::EsfMeas(esf_meas_ref) = &packet_ref {
                let actual = serde_json::to_value(esf_meas_ref).unwrap();
                assert_eq!(expected_esf_meas_json, actual);
            } else {
                panic!();
            }
        },
        UbxPacket::Proto27(packet_ref) => {
            let actual = serde_json::to_value(&packet_ref).unwrap();
            assert_eq!(expected_packet_json, actual);
            if let ublox::proto27::PacketRef::EsfMeas(esf_meas_ref) = &packet_ref {
                let actual = serde_json::to_value(esf_meas_ref).unwrap();
                assert_eq!(expected_esf_meas_json, actual);
            } else {
                panic!();
            }
        },
        UbxPacket::Proto31(packet_ref) => {
            let actual = serde_json::to_value(&packet_ref).unwrap();
            assert_eq!(expected_packet_json, actual);
            if let ublox::proto31::PacketRef::EsfMeas(esf_meas_ref) = &packet_ref {
                let actual = serde_json::to_value(esf_meas_ref).unwrap();
                assert_eq!(expected_esf_meas_json, actual);
            } else {
                panic!();
            }
        },
    }
}

#[test]
#[cfg(feature = "ubx_proto23")]
#[cfg(feature = "serde")]
fn test_esf_meas_serialize_proto23() {
    use ublox::proto23::Proto23;
    let ret = RET_ESF_MEAS_SERIALIZE;

    let mut parser = Parser::<_, Proto23>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&ret);

    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto23(pack)) => {
                test_util_esf_meas_assert_expected_json(ublox::UbxPacket::Proto23(pack));
                found = true;
            },
            _ => panic!(),
        }
    }
    assert!(found);
}
#[test]
#[cfg(feature = "ubx_proto27")]
#[cfg(feature = "serde")]
fn test_esf_meas_serialize_proto27() {
    use ublox::proto27::Proto27;
    let ret = RET_ESF_MEAS_SERIALIZE;

    let mut parser = Parser::<_, Proto27>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&ret);

    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto27(pack)) => {
                test_util_esf_meas_assert_expected_json(ublox::UbxPacket::Proto27(pack));
                found = true;
            },
            _ => panic!(),
        }
    }
    assert!(found);
}
#[test]
#[cfg(feature = "ubx_proto31")]
#[cfg(feature = "serde")]
fn test_esf_meas_serialize_proto31() {
    use ublox::proto31::Proto31;
    let ret = RET_ESF_MEAS_SERIALIZE;

    let mut parser = Parser::<_, Proto31>::default();
    let mut found = false;
    let mut it = parser.consume_ubx(&ret);

    while let Some(pack) = it.next() {
        match pack {
            Ok(UbxPacket::Proto31(pack)) => {
                test_util_esf_meas_assert_expected_json(ublox::UbxPacket::Proto31(pack));
                found = true;
            },
            _ => panic!(),
        }
    }
    assert!(found);
}

const ZERO_SIZED_ACK_ACK_BYTES: [u8; 8] = [0xb5, 0x62, 0x05, 0x01, 0x00, 0x00, 0x06, 0x17];

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_zero_sized_ackack_proto14() {
    use ublox::proto17::{PacketRef, Proto17};
    let mut parser = Parser::<_, Proto17>::default();
    let mut it = parser.consume_ubx(&ZERO_SIZED_ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto17(PacketRef::Unknown(_)))) => {
            // This is expected
        },
        _ => panic!(),
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_zero_sized_ackack_proto23() {
    use ublox::proto23::{PacketRef, Proto23};
    let mut parser = Parser::<_, Proto23>::default();
    let mut it = parser.consume_ubx(&ZERO_SIZED_ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto23(PacketRef::Unknown(_)))) => {
            // This is expected
        },
        _ => panic!(),
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_zero_sized_ackack_proto27() {
    use ublox::proto27::{PacketRef, Proto27};
    let mut parser = Parser::<_, Proto27>::default();
    let mut it = parser.consume_ubx(&ZERO_SIZED_ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto27(PacketRef::Unknown(_)))) => {
            // This is expected
        },
        _ => panic!(),
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_zero_sized_ackack_proto31() {
    use ublox::proto31::{PacketRef, Proto31};
    let mut parser = Parser::<_, Proto31>::default();
    let mut it = parser.consume_ubx(&ZERO_SIZED_ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto31(PacketRef::Unknown(_)))) => {
            // This is expected
        },
        _ => panic!(),
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_double_start_at_end_proto14() {
    use ublox::{
        proto17::{PacketRef, Proto17},
        FixedLinearBuffer,
    };
    #[rustfmt::skip]
    let bytes = [
        0xb5, 0x62, // Extraneous start header
        0xb5, 0x62, 0x05, 0x01, 0x00, 0x00, 0x06, 0x17, // Zero-sized packet
    ];

    let mut buf = [0; 10];
    let mut parser = ublox::Parser::<_, Proto17>::new(FixedLinearBuffer::new(&mut buf));

    for byte in bytes.iter() {
        parser.consume_ubx(&[*byte]);
    }

    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    {
        let mut it = parser.consume_ubx(&ack_ack);
        match it.next() {
            Some(Err(_)) => {
                // First, a buffer-too-small error
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto17(PacketRef::Unknown(_)))) => {
                // Then an unknown packet
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto17(PacketRef::AckAck(_)))) => {
                // Then the ackack we passed
            },
            _ => panic!(),
        }
        assert!(it.next().is_none());
    }
    let mut it = parser.consume_ubx(&ack_ack);
    match it.next() {
        Some(Ok(UbxPacket::Proto17(PacketRef::AckAck { .. }))) => {
            // This is what we expect
        },
        _ => {
            // Parsing other packets or ending the iteration is a failure
            panic!();
        },
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_double_start_at_end_proto23() {
    use ublox::{
        proto23::{PacketRef, Proto23},
        FixedLinearBuffer,
    };
    #[rustfmt::skip]
    let bytes = [
        0xb5, 0x62, // Extraneous start header
        0xb5, 0x62, 0x05, 0x01, 0x00, 0x00, 0x06, 0x17, // Zero-sized packet
    ];

    let mut buf = [0; 10];
    let mut parser = ublox::Parser::<_, Proto23>::new(FixedLinearBuffer::new(&mut buf));

    for byte in bytes.iter() {
        parser.consume_ubx(&[*byte]);
    }

    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    {
        let mut it = parser.consume_ubx(&ack_ack);
        match it.next() {
            Some(Err(_)) => {
                // First, a buffer-too-small error
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto23(PacketRef::Unknown(_)))) => {
                // Then an unknown packet
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto23(PacketRef::AckAck(_)))) => {
                // Then the ackack we passed
            },
            _ => panic!(),
        }
        assert!(it.next().is_none());
    }
    let mut it = parser.consume_ubx(&ack_ack);
    match it.next() {
        Some(Ok(UbxPacket::Proto23(PacketRef::AckAck { .. }))) => {
            // This is what we expect
        },
        _ => {
            // Parsing other packets or ending the iteration is a failure
            panic!();
        },
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_double_start_at_end_proto27() {
    use ublox::{
        proto27::{PacketRef, Proto27},
        FixedLinearBuffer,
    };
    #[rustfmt::skip]
    let bytes = [
        0xb5, 0x62, // Extraneous start header
        0xb5, 0x62, 0x05, 0x01, 0x00, 0x00, 0x06, 0x17, // Zero-sized packet
    ];

    let mut buf = [0; 10];
    let mut parser = ublox::Parser::<_, Proto27>::new(FixedLinearBuffer::new(&mut buf));

    for byte in bytes.iter() {
        parser.consume_ubx(&[*byte]);
    }

    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    {
        let mut it = parser.consume_ubx(&ack_ack);
        match it.next() {
            Some(Err(_)) => {
                // First, a buffer-too-small error
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto27(PacketRef::Unknown(_)))) => {
                // Then an unknown packet
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto27(PacketRef::AckAck(_)))) => {
                // Then the ackack we passed
            },
            _ => panic!(),
        }
        assert!(it.next().is_none());
    }
    let mut it = parser.consume_ubx(&ack_ack);
    match it.next() {
        Some(Ok(UbxPacket::Proto27(PacketRef::AckAck { .. }))) => {
            // This is what we expect
        },
        _ => {
            // Parsing other packets or ending the iteration is a failure
            panic!();
        },
    }
    assert!(it.next().is_none());
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_double_start_at_end_proto31() {
    use ublox::{
        proto31::{PacketRef, Proto31},
        FixedLinearBuffer,
    };
    #[rustfmt::skip]
    let bytes = [
        0xb5, 0x62, // Extraneous start header
        0xb5, 0x62, 0x05, 0x01, 0x00, 0x00, 0x06, 0x17, // Zero-sized packet
    ];

    let mut buf = [0; 10];
    let mut parser = ublox::Parser::<_, Proto31>::new(FixedLinearBuffer::new(&mut buf));

    for byte in bytes.iter() {
        parser.consume_ubx(&[*byte]);
    }

    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    {
        let mut it = parser.consume_ubx(&ack_ack);
        match it.next() {
            Some(Err(_)) => {
                // First, a buffer-too-small error
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto31(PacketRef::Unknown(_)))) => {
                // Then an unknown packet
            },
            _ => panic!(),
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto31(PacketRef::AckAck(_)))) => {
                // Then the ackack we passed
            },
            _ => panic!(),
        }
        assert!(it.next().is_none());
    }
    let mut it = parser.consume_ubx(&ack_ack);
    match it.next() {
        Some(Ok(UbxPacket::Proto31(PacketRef::AckAck { .. }))) => {
            // This is what we expect
        },
        _ => {
            // Parsing other packets or ending the iteration is a failure
            panic!();
        },
    }
    assert!(it.next().is_none());
}

const ACK_ACK_BYTES: [u8; 10] = [0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 0x04, 0x05, 0x11, 0x38];

#[cfg(feature = "ubx_proto14")]
#[test]
fn test_ack_ack_to_owned_can_be_moved_proto14() {
    use ublox::proto17::{PacketRef, Proto17};
    // 4 is the class id of the acknowledged packet from the payload of UbxAckAck
    let expect_ack_payload_class_id = 4;

    let mut parser = ublox::Parser::<_, Proto17>::default();
    let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto17(PacketRef::AckAck(ack_packet)))) => {
            assert_eq!(ack_packet.class(), expect_ack_payload_class_id);
            let borrowed: AckAckRef = ack_packet;
            let owned: AckAckOwned = borrowed.to_owned();

            assert_eq!(borrowed.class(), owned.class());
            assert_eq!(borrowed.msg_id(), owned.msg_id());

            let thread = std::thread::spawn(move || {
                // This won't compile if the owned packet is not moveable.
                std::dbg!(owned);
            });
            thread.join().unwrap();
        },
        _ => panic!(),
    };
}

#[cfg(feature = "ubx_proto23")]
#[test]
fn test_ack_ack_to_owned_can_be_moved_proto23() {
    use ublox::proto23::{PacketRef, Proto23};
    // 4 is the class id of the acknowledged packet from the payload of UbxAckAck
    let expect_ack_payload_class_id = 4;

    let mut parser = ublox::Parser::<_, Proto23>::default();
    let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto23(PacketRef::AckAck(ack_packet)))) => {
            assert_eq!(ack_packet.class(), expect_ack_payload_class_id);
            let borrowed: AckAckRef = ack_packet;
            let owned: AckAckOwned = borrowed.to_owned();

            assert_eq!(borrowed.class(), owned.class());
            assert_eq!(borrowed.msg_id(), owned.msg_id());

            let thread = std::thread::spawn(move || {
                // This won't compile if the owned packet is not moveable.
                std::dbg!(owned);
            });
            thread.join().unwrap();
        },
        _ => panic!(),
    };
}

#[cfg(feature = "ubx_proto27")]
#[test]
fn test_ack_ack_to_owned_can_be_moved_proto27() {
    use ublox::proto27::{PacketRef, Proto27};
    // 4 is the class id of the acknowledged packet from the payload of UbxAckAck
    let expect_ack_payload_class_id = 4;

    let mut parser = ublox::Parser::<_, Proto27>::default();
    let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto27(PacketRef::AckAck(ack_packet)))) => {
            assert_eq!(ack_packet.class(), expect_ack_payload_class_id);
            let borrowed: AckAckRef = ack_packet;
            let owned: AckAckOwned = borrowed.to_owned();

            assert_eq!(borrowed.class(), owned.class());
            assert_eq!(borrowed.msg_id(), owned.msg_id());

            let thread = std::thread::spawn(move || {
                // This won't compile if the owned packet is not moveable.
                std::dbg!(owned);
            });
            thread.join().unwrap();
        },
        _ => panic!(),
    };
}

#[cfg(feature = "ubx_proto31")]
#[test]
fn test_ack_ack_to_owned_can_be_moved_proto31() {
    use ublox::proto31::{PacketRef, Proto31};
    // 4 is the class id of the acknowledged packet from the payload of UbxAckAck
    let expect_ack_payload_class_id = 4;

    let mut parser = ublox::Parser::<_, Proto31>::default();
    let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
    match it.next() {
        Some(Ok(UbxPacket::Proto31(PacketRef::AckAck(ack_packet)))) => {
            assert_eq!(ack_packet.class(), expect_ack_payload_class_id);
            let borrowed: AckAckRef = ack_packet;
            let owned: AckAckOwned = borrowed.to_owned();

            assert_eq!(borrowed.class(), owned.class());
            assert_eq!(borrowed.msg_id(), owned.msg_id());

            let thread = std::thread::spawn(move || {
                // This won't compile if the owned packet is not moveable.
                std::dbg!(owned);
            });
            thread.join().unwrap();
        },
        _ => panic!(),
    };
}
