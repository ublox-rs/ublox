use ubx_protocol::{CfgMsg3Builder, NavPosLLH, NavStatus, UbxPacket};

#[test]
fn test_cfg_msg_simple() {
    assert_eq!(
        [0xb5, 0x62, 0x06, 0x01, 0x03, 0x00, 0x01, 0x02, 0x01, 0x0E, 0x47],
        CfgMsg3Builder {
            msg_class: NavPosLLH::CLASS,
            msg_id: NavPosLLH::ID,
            rate: 1,
        }
        .to_packet_bytes()
    );

    assert_eq!(
        [0xb5, 0x62, 0x06, 0x01, 0x03, 0x00, 0x01, 0x03, 0x01, 0x0F, 0x49],
        CfgMsg3Builder {
            msg_class: NavStatus::CLASS,
            msg_id: NavStatus::ID,
            rate: 1,
        }
        .to_packet_bytes()
    );
}
