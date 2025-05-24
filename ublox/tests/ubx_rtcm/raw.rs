/**
 * Tests the UBX + RTCM joint iterator & extraction, without actual interpretation
 */
use ublox::{AckAckOwned, AckAckRef, AnyPacketRef, PacketRef as UbxPacketRef, Parser};

use rtcm_rs::{
    msg::{Msg1001Sat, Msg1001T},
    util::DataVec,
    Message as RtcmMessage, MessageBuilder as RtcmMessageBuilder,
};

// Verifies that UBX frames are still decoded correctly
#[test]
fn single_ubx() {
    let ack_ack = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];
    let expect_ack_payload_class_id = 4;

    let mut parser = Parser::default();
    let mut it = parser.consume_ubx_rtcm(&ack_ack);

    match it.next() {
        Some(Ok(AnyPacketRef::Ubx(UbxPacketRef::AckAck(ack_packet)))) => {
            assert_eq!(ack_packet.class(), expect_ack_payload_class_id);
            let borrowed: AckAckRef = ack_packet;
            let owned: AckAckOwned = borrowed.to_owned();

            assert_eq!(borrowed.class(), owned.class());
            assert_eq!(borrowed.msg_id(), owned.msg_id());
        },
        _ => panic!(),
    };
}

/// Verifies RTCM frames are extracted
#[test]
fn single_rtcm() {
    let mut builder = RtcmMessageBuilder::new();

    let buffered = builder
        .build_message(&RtcmMessage::Msg1001(Msg1001T {
            reference_station_id: 100,
            gps_epoch_time_ms: 10,
            synchronous_gnss_msg_flag: 1,
            divergence_free_smoothing_flag: 2,
            smoothing_interval_index: 3,
            satellites: {
                let mut satellites = DataVec::new();
                satellites.push(Msg1001Sat {
                    gps_satellite_id: 20,
                    gps_l1_code_ind: 21,
                    l1_pseudorange_m: Some(1.0),
                    l1_phase_pseudorange_diff_m: Some(2.0),
                    l1_lock_time_index: 3,
                });
                satellites
            },
        }))
        .unwrap_or_else(|e| {
            panic!("Failed to forge RTCM message: {}", e);
        });

    let mut parser = Parser::default();
    let mut it = parser.consume_ubx_rtcm(&buffered);

    match it.next() {
        Some(Ok(AnyPacketRef::Rtcm(rtcm))) => {},
        Some(Ok(AnyPacketRef::Ubx(_))) => {
            panic!("RTCM packet interpreted as UBX!");
        },
        _ => panic!("Decoding did not work"),
    };
}
