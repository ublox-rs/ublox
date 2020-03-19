use ublox::{CfgMsgSinglePortBuilder, NavPosLlh, NavStatus};

#[test]
fn test_cfg_msg_simple() {
    assert_eq!(
        [0xb5, 0x62, 0x06, 0x01, 0x03, 0x00, 0x01, 0x02, 0x01, 0x0E, 0x47],
        CfgMsgSinglePortBuilder::set_rate_for::<NavPosLlh>(1).into_packet_bytes()
    );

    assert_eq!(
        [0xb5, 0x62, 0x06, 0x01, 0x03, 0x00, 0x01, 0x03, 0x01, 0x0F, 0x49],
        CfgMsgSinglePortBuilder::set_rate_for::<NavStatus>(1).into_packet_bytes()
    );
}
