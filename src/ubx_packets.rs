use crate::error::Result;
use bincode;
use chrono::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::vec::Vec;
use std::str;
//use syn::{parse_macro_input, parse_quote, DeriveInput, Data, TokenStream};
use ublox_derive::ubx_packet;

// These are needed for ubx_packet
use std::convert::TryInto;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug)]
pub struct Position {
    pub lon: f32,
    pub lat: f32,
    pub alt: f32,
}

#[derive(Debug)]
pub struct Velocity {
    pub speed: f32,   // m/s over ground
    pub heading: f32, // degrees
}

#[derive(Debug)]
pub struct UbxPacket {
    pub class: u8,
    pub id: u8,
    pub payload: Vec<u8>,
}

impl UbxPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(0xB5);
        v.push(0x62);
        v.push(self.class);
        v.push(self.id);

        let length = self.payload.len() as u16;
        v.push((length & 0xFF) as u8);
        v.push(((length >> 8) & 0xFF) as u8);

        for b in self.payload.iter() {
            v.push(*b);
        }

        // Calculate the checksum
        let mut cka = 0;
        let mut ckb = 0;
        for i in 0..self.payload.len() + 4 {
            cka = ((cka as usize + v[i + 2] as usize) & 0xFF) as u8;
            ckb = ((cka as usize + ckb as usize) & 0xFF) as u8;
        }
        v.push(cka);
        v.push(ckb);
        v
    }

    fn compute_checksum(&self) -> (u8, u8) {
        let s = self.serialize();
        let cka = s[s.len() - 2];
        let ckb = s[s.len() - 1];
        return (cka, ckb);
    }

    pub fn check_checksum(&self, test_cka: u8, test_ckb: u8) -> bool {
        let (cka, ckb) = self.compute_checksum();
        cka == test_cka && ckb == test_ckb
    }
}

/*#[proc_macro_attribute]
fn ubx_packet(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    println!("{:?}", fields);
                }
                Fields::Unnamed(ref fields) => {
                    //
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    }
}*/

/*#[ubx_packet]
struct MyPacket {
    tow: u32,
    lon: i32,
    lat: i32,
    height: i32,
    height_msl: i32,
    h_acc: u32,
    v_acc: u32,
}*/

pub trait UbxMeta {
    fn get_classid() -> u8;
    fn get_msgid() -> u8;

    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! ubx_meta {
    ($struct:ident, $classid:literal, $msgid:literal) => {
        impl $struct {
            fn get_classid() -> u8 {
                $classid
            }

            fn get_msgid() -> u8 {
                $msgid
            }

            pub fn to_bytes(&self) -> Vec<u8> {
                let upacket: UbxPacket = self.into();
                upacket.serialize()
            }
        }

        impl From<&$struct> for UbxPacket {
            fn from(packet: &$struct) -> UbxPacket {
                UbxPacket {
                    class: $classid,
                    id: $msgid,
                    payload: bincode::serialize(packet).unwrap(),
                }
            }
        }

        impl From<$struct> for UbxPacket {
            fn from(packet: $struct) -> UbxPacket {
                UbxPacket {
                    class: $classid,
                    id: $msgid,
                    payload: bincode::serialize(&packet).unwrap(),
                }
            }
        }
    };
}

#[ubx_packet]
pub struct NavPosLLH {
    itow: u32,
    lon: i32,
    lat: i32,
    height: i32,
    height_msl: i32,
    horizontal_accuracy: u32,
    vertical_accuracy: u32,
}

/*#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NavPosLLH {
    pub itow: u32,
    pub lon: i32,
    pub lat: i32,
    pub height: i32,
    pub height_msl: i32,
    pub horizontal_accuracy: u32,
    pub vertical_accuracy: u32,
}*/

//ubx_meta!(NavPosLLH, 0x01, 0x02);

impl From<&NavPosLLH> for Position {
    fn from(packet: &NavPosLLH) -> Self {
        Position {
            lon: packet.get_lon() as f32 / 10_000_000.0,
            lat: packet.get_lat() as f32 / 10_000_000.0,
            alt: packet.get_height_msl() as f32 / 1000.0,
        }
    }
}

trait FooTrait {
    fn foo() -> u32;
}

impl<T: FooTrait> From<&T> for Velocity {
    fn from(packet: &T) -> Self {
        Velocity {
            speed: 0.0,
            heading: 0.0,
        }
    }
}

#[ubx_packet]
pub struct NavVelNED {
    pub itow: u32,
    pub vel_north: i32, // cm/s
    pub vel_east: i32,
    pub vel_down: i32,
    pub speed: u32,
    pub ground_speed: u32,
    pub heading: i32, // 1e-5 degrees
}

/*#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NavVelNED {
    pub itow: u32,
    pub vel_north: i32, // cm/s
    pub vel_east: i32,
    pub vel_down: i32,
    pub speed: u32,
    pub ground_speed: u32,
    pub heading: i32, // 1e-5 degrees
}

ubx_meta!(NavVelNED, 0x01, 0x12);*/

impl From<&NavVelNED> for Velocity {
    fn from(packet: &NavVelNED) -> Self {
        Velocity {
            speed: packet.get_ground_speed() as f32 / 1_000.0,
            heading: packet.get_heading() as f32 / 100_000.0,
        }
    }
}

/*pub struct NavPosVelTime {
    itow: u32,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,

    #[ubx_bitfield(8)]
    #[ubx_range(0:0)]
    valid: bool,

    // etc.
}*/

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NavPosVelTime {
    pub itow: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub time_accuracy: u32,
    pub nanosecond: i32,
    pub fix_type: u8,
    pub flags: u8,
    pub reserved1: u8,
    pub num_satellites: u8,
    pub lon: i32,
    pub lat: i32,
    pub height: i32,
    pub height_msl: i32,
    pub horiz_accuracy: u32,
    pub vert_accuracy: u32,
    pub vel_north: i32, // mm/s
    pub vel_east: i32,
    pub vel_down: i32,
    pub ground_speed: i32, // mm/s
    pub heading: i32,      // 1e-5 deg
    pub speed_accuracy: u32,
    pub heading_accuracy: u32,
    pub pos_dop: u16,
    pub reserved2: u16,
    pub reserved3: u32,
}

ubx_meta!(NavPosVelTime, 0x01, 0x07);

impl From<&NavPosVelTime> for Position {
    fn from(packet: &NavPosVelTime) -> Self {
        Position {
            lon: packet.lon as f32 / 10_000_000.0,
            lat: packet.lat as f32 / 10_000_000.0,
            alt: packet.height_msl as f32 / 1000.0,
        }
    }
}

impl From<&NavPosVelTime> for Velocity {
    fn from(packet: &NavPosVelTime) -> Self {
        Velocity {
            speed: packet.ground_speed as f32 / 1_000.0,
            heading: packet.heading as f32 / 100_000.0,
        }
    }
}

impl From<&NavPosVelTime> for DateTime<Utc> {
    fn from(sol: &NavPosVelTime) -> Self {
        let ns = if sol.nanosecond < 0 { 0 } else { sol.nanosecond } as u32;
        Utc.ymd(sol.year as i32, sol.month.into(), sol.day.into())
            .and_hms_nano(
                sol.hour.into(),
                sol.min.into(),
                sol.sec.into(),
                ns,
            )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NavStatus {
    pub itow: u32,
    pub gps_fix: u8,
    pub flags: u8,
    pub fix_status: u8,
    pub flags2: u8,
    pub time_to_first_fix: u32,
    pub uptime_ms: u32,
}

ubx_meta!(NavStatus, 0x01, 0x03);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AidIni {
    ecef_x_or_lat: i32,
    ecef_y_or_lon: i32,
    ecef_z_or_alt: i32,
    pos_accuracy: u32,
    time_cfg: u16,
    week_or_ym: u16,
    tow_or_hms: u32,
    tow_ns: i32,
    tm_accuracy_ms: u32,
    tm_accuracy_ns: u32,
    clk_drift_or_freq: i32,
    clk_drift_or_freq_accuracy: u32,
    flags: u32,
}

ubx_meta!(AidIni, 0x0B, 0x01);

impl AidIni {
    pub fn new() -> AidIni {
        AidIni {
            ecef_x_or_lat: 0,
            ecef_y_or_lon: 0,
            ecef_z_or_alt: 0,
            pos_accuracy: 0,
            time_cfg: 0,
            week_or_ym: 0,
            tow_or_hms: 0,
            tow_ns: 0,
            tm_accuracy_ms: 0,
            tm_accuracy_ns: 0,
            clk_drift_or_freq: 0,
            clk_drift_or_freq_accuracy: 0,
            flags: 0,
        }
    }

    pub fn set_position(&mut self, pos: Position) {
        self.ecef_x_or_lat = (pos.lat * 10_000_000.0) as i32;
        self.ecef_y_or_lon = (pos.lon * 10_000_000.0) as i32;
        self.ecef_z_or_alt = (pos.alt * 100.0) as i32; // Height is in centimeters, here
        self.flags |= (1 << 0) | (1 << 5);
    }

    pub fn set_time(&mut self, tm: DateTime<Utc>) {
        self.week_or_ym = (match tm.year_ce() {
            (true, yr) => yr - 2000,
            (false, _) => {
                panic!("Jesus must have been born for this method to work");
            }
        } * 100
            + tm.month0()) as u16;
        self.tow_or_hms = tm.hour() * 10000 + tm.minute() * 100 + tm.second();
        self.tow_ns = tm.nanosecond() as i32;
        self.flags |= (1 << 1) | (1 << 10);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AlpSrv {
    pub id_size: u8,
    pub data_type: u8,
    pub offset: u16,
    pub size: u16,
    pub file_id: u16,
    pub data_size: u16,
    pub id1: u8,
    pub id2: u8,
    pub id3: u32,
}

ubx_meta!(AlpSrv, 0x0B, 0x32);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CfgPrtUart {
    pub portid: u8,
    pub reserved0: u8,
    pub tx_ready: u16,
    pub mode: u32,
    pub baud_rate: u32,
    pub in_proto_mask: u16,
    pub out_proto_mask: u16,
    pub flags: u16,
    pub reserved5: u16,
}

ubx_meta!(CfgPrtUart, 0x06, 0x00);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CfgPrtSpi {
    pub portid: u8,
    pub reserved0: u8,
    pub tx_ready: u16,
    pub mode: u32,
    pub reserved3: u32,
    pub in_proto_mask: u16,
    pub out_proto_mask: u16,
    pub flags: u16,
    pub reserved5: u16,
}

ubx_meta!(CfgPrtSpi, 0x06, 0x00);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AckAck {
    pub classid: u8,
    pub msgid: u8,
}

ubx_meta!(AckAck, 0x05, 0x01);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CfgRst {
    pub nav_bbr_mask: u16,
    pub reset_mode: u8,
    pub reserved1: u8,
}

ubx_meta!(CfgRst, 0x06, 0x04);

impl CfgRst {
    pub const HOT: CfgRst = CfgRst {
        nav_bbr_mask: 0,
        reset_mode: 1,
        reserved1: 0,
    };

    pub const WARM: CfgRst = CfgRst {
        nav_bbr_mask: 0x01,
        reset_mode: 1,
        reserved1: 0,
    };

    pub const COLD: CfgRst = CfgRst {
        nav_bbr_mask: 0xFFFF,
        reset_mode: 1,
        reserved1: 0,
    };
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CfgMsg {
    pub classid: u8,
    pub msgid: u8,
    pub rates: [u8; 6],
}

ubx_meta!(CfgMsg, 0x06, 0x01);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MonVer {
    pub sw_version: String,
    pub hw_version: String,
}

impl UbxMeta for MonVer {
    fn get_classid() -> u8 { 0x0a }
    fn get_msgid() -> u8 { 0x04 }

    fn to_bytes(&self) -> Vec<u8> {
        unimplemented!("Sending MonVer packets is unimplemented");
    }
}

#[derive(Debug, PartialEq)]
pub enum Packet {
    NavPosLLH(NavPosLLH),
    NavStatus(NavStatus),
    NavPosVelTime(NavPosVelTime),
    NavVelNED(NavVelNED),
    AckAck(AckAck),
    CfgPrtUart(CfgPrtUart),
    CfgPrtSpi(CfgPrtSpi),
    CfgMsg(CfgMsg),
    CfgRst(CfgRst),
    MonVer(MonVer),
    AidIni(AidIni),
    AlpSrv(AlpSrv),
}

macro_rules! parse_packet_branch {
    ($struct: path, $payload: ident) => {{
        let packet = bincode::deserialize($payload)?;
        Ok($struct(packet))
    }};
}

impl Packet {
    pub fn deserialize(classid: u8, msgid: u8, payload: &[u8]) -> Result<Packet> {
        match (classid, msgid) {
            //(0x01, 0x02) => parse_packet_branch!(Packet::NavPosLLH, payload),
            (0x01, 0x02) => {
                Ok(Packet::NavPosLLH(NavPosLLH::new(payload.try_into().unwrap())))
            },
            (0x01, 0x03) => parse_packet_branch!(Packet::NavStatus, payload),
            (0x01, 0x07) => parse_packet_branch!(Packet::NavPosVelTime, payload),
            //(0x01, 0x12) => parse_packet_branch!(Packet::NavVelNED, payload),
            (0x01, 0x12) => {
                Ok(Packet::NavVelNED(NavVelNED::new(payload.try_into().unwrap())))
            }
            (0x05, 0x01) => parse_packet_branch!(Packet::AckAck, payload),
            (0x06, 0x00) => {
                // Depending on the port ID, we parse different packets
                match payload[0] {
                    1 => parse_packet_branch!(Packet::CfgPrtUart, payload),
                    4 => parse_packet_branch!(Packet::CfgPrtSpi, payload),
                    _ => {
                        panic!("Unrecognized port ID {}! (is it USB?)", payload[0]);
                    }
                }
            }
            (0x06, 0x01) => parse_packet_branch!(Packet::CfgMsg, payload),
            (0x06, 0x04) => parse_packet_branch!(Packet::CfgRst, payload),
            (0x0A, 0x04) => {
                let sw_version = str::from_utf8(&payload[0..30]).unwrap();
                let hw_version = str::from_utf8(&payload[31..40]).unwrap();
                return Ok(Packet::MonVer(MonVer{
                    sw_version: sw_version.to_string(),
                    hw_version: hw_version.to_string(),
                }));
            },
            (0x0B, 0x01) => parse_packet_branch!(Packet::AidIni, payload),
            (0x0B, 0x32) => parse_packet_branch!(Packet::AlpSrv, payload),
            (c, m) => {
                panic!("Unimplemented packet classid={} msgid={}", c, m);
            }
        }
    }
}
