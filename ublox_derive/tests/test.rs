use std::convert::TryInto;
use ublox_derive::ubx_packet;

#[ubx_packet]
struct TestPacket {
    field1: u32,
    field2: u32,
    field3: u8,
    field4: u16,
    field5: i32,
}

#[test]
fn foo() {
    assert_eq!(std::mem::size_of::<TestPacket>(), 15);
}

#[ubx_packet]
struct TestPacket2 {
    field1: u32,
    field2: u8,
    field3: u32,
}

fn helper(packet: &TestPacket2) -> u32 {
    packet.get_field3()
}

#[test]
fn foo2() {
    let data = [1, 0, 0, 0, 0, 2, 0, 0, 0];
    let packet = TestPacket2::new(data);
    assert_eq!(helper(&packet), 2);
    assert_eq!(packet.get_field2(), 0);
}
