/*
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
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

#[no_mangle]
#[inline(never)]
fn helper(packet: &TestPacket2) -> u32 {
    packet.get_field3()
}

#[test]
#[no_mangle]
#[inline(never)]
fn foo2() {
    let data = [1, 0, 0, 0, 0, 2, 0, 0, 0];
    let packet = TestPacket2::new(data);
    assert_eq!(helper(&packet), 2);
    assert_eq!(packet.get_field2(), 0);
}

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
enum CfgPrtId {
    Usb = 1,
    Spi = 2,
}

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
enum CfgPrtCharLen {
    FiveBit = 0,
    SixBit = 1,
    SevenBit = 2,
    EightBit = 3,
}

#[ubx_packet]
struct TestPacket3 {
    #[ubx_enum(CfgPrtId)]
    port_id: u8,

    rfu0: u8, // TODO: This should be hidden from the user

    #[ubx_bitfield(16)]
    #[ubx_bitrange(0:0)]
    tx_ready_en: bool,

    #[ubx_bitrange(1:1)]
    tx_ready_polarity: bool,

    #[ubx_bitrange(6:2)]
    tx_ready_pin: u8,

    #[ubx_bitrange(15:7)]
    tx_ready_threshold: u16, // TODO: u8 should throw an error

    #[ubx_bitfield(32)]
    #[ubx_bitrange(7:6)]
    #[ubx_enum(CfgPrtCharLen)]
    mode_charlen: u8,

    #[ubx_bitrange(11:9)]
    parity: u8,

    #[ubx_bitrange(13:12)]
    num_stop_bits: u8,

    baudrate: u32,

    #[ubx_bitfield(16)] // TODO: Bitfield without bitrange should error
    #[ubx_bitrange(0:0)]
    in_ubx: bool,

    #[ubx_bitrange(1:1)]
    in_nmea: bool,

    #[ubx_bitrange(2:2)] // TODO: Bitrange without bitfield should error
    in_rtcm: bool,

    #[ubx_bitfield(16)]
    #[ubx_bitrange(0:0)]
    out_ubx: bool,

    #[ubx_bitrange(1:1)]
    out_nmea: bool,

    #[ubx_bitfield(16)]
    #[ubx_bitrange(0:0)]
    extended_tx_timeout: bool,

    rfu5: u16,
}

#[test]
#[no_mangle]
#[inline(never)]
fn bitfields() {
    let mut data = [0; std::mem::size_of::<TestPacket3>()];
    data[0] = 1;
    data[2] = 0x5;
    data[3] = 0x1;
    data[16] = 0x1;
    let mut packet = TestPacket3::new(data);
    assert_eq!(packet.get_port_id(), Some(CfgPrtId::Usb));
    assert_eq!(packet.get_tx_ready_en(), true);
    assert_eq!(packet.get_tx_ready_polarity(), false);
    assert_eq!(packet.get_tx_ready_pin(), 1);
    assert_eq!(packet.get_tx_ready_threshold(), 2);
    assert_eq!(packet.get_extended_tx_timeout(), true);
    assert_eq!(packet.get_mode_charlen(), Some(CfgPrtCharLen::FiveBit));

    packet.set_baudrate(9600);
    assert_eq!(packet.get_baudrate(), 9600);

    packet.set_mode_charlen(CfgPrtCharLen::SixBit);
    packet.set_parity(2);
    assert_eq!(packet.get_mode_charlen(), Some(CfgPrtCharLen::SixBit));
}
*/
