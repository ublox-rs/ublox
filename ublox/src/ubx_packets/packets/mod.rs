use core::convert::TryInto;
use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitflags::bitflags;
use chrono::prelude::*;
use num_traits::cast::{FromPrimitive, ToPrimitive};
use num_traits::float::FloatCore;

use ublox_derive::{
    define_recv_packets, ubx_extend, ubx_extend_bitflags, ubx_packet_recv, ubx_packet_recv_send,
    ubx_packet_send,
};

use crate::error::{MemWriterError, ParserError};

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use super::{
    ubx_checksum, MemWriter, Position, UbxChecksumCalc, UbxPacketCreator, UbxPacketMeta,
    UbxUnknownPacketRef, SYNC_CHAR_1, SYNC_CHAR_2,
};

// CFG- packets definition
pub mod cfg;
pub use cfg::*;

// NAV- packets definition
pub mod nav;
pub use nav::{NavPosLlh, NavPosLlhRef};

// MGA- packets definition
pub mod mga;
pub use mga::*;

// MON- packets definition
pub mod mon;
pub use mon::*;

// RawRXM packet definition
pub mod rxm;
pub use rxm::*;

// INF- packets definition
pub mod inf;
pub use inf::*;

// ESF- packets definition
pub mod esf;
pub use esf::*;

// HNR- packets definition
pub mod hnr;
pub use hnr::*;

// TIM- packets definition
pub mod tim;
pub use tim::*;

/// Used to help serialize the packet's fields flattened within a struct containing the msg_id and class fields, but
/// without using the serde FlatMapSerializer which requires alloc.
#[cfg(feature = "serde")]
pub(crate) trait SerializeUbxPacketFields {
    fn serialize_fields<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeMap;
}

/// GPS fix Type
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GpsFix {
    NoFix = 0,
    DeadReckoningOnly = 1,
    Fix2D = 2,
    Fix3D = 3,
    GPSPlusDeadReckoning = 4,
    TimeOnlyFix = 5,
}

/// Fix Status Information
#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FixStatusInfo(u8);

impl FixStatusInfo {
    pub const fn has_pr_prr_correction(self) -> bool {
        (self.0 & 1) == 1
    }
    pub fn map_matching(self) -> MapMatchingStatus {
        let bits = (self.0 >> 6) & 3;
        match bits {
            0 => MapMatchingStatus::None,
            1 => MapMatchingStatus::Valid,
            2 => MapMatchingStatus::Used,
            3 => MapMatchingStatus::Dr,
            _ => unreachable!(),
        }
    }
    pub const fn from(x: u8) -> Self {
        Self(x)
    }
}

impl fmt::Debug for FixStatusInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixStatusInfo")
            .field("has_pr_prr_correction", &self.has_pr_prr_correction())
            .field("map_matching", &self.map_matching())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MapMatchingStatus {
    None = 0,
    /// valid, i.e. map matching data was received, but was too old
    Valid = 1,
    /// used, map matching data was applied
    Used = 2,
    /// map matching was the reason to enable the dead reckoning
    /// gpsFix type instead of publishing no fix
    Dr = 3,
}

#[ubx_packet_send]
#[ubx(
    class = 0x0B,
    id = 0x01,
    fixed_payload_len = 48,
    flags = "default_for_builder"
)]
struct AidIni {
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

impl AidIniBuilder {
    pub fn set_position(mut self, pos: Position) -> Self {
        self.ecef_x_or_lat = (pos.lat * 10_000_000.0) as i32;
        self.ecef_y_or_lon = (pos.lon * 10_000_000.0) as i32;
        self.ecef_z_or_alt = (pos.alt * 100.0) as i32; // Height is in centimeters, here
        self.flags |= (1 << 0) | (1 << 5);
        self
    }

    pub fn set_time(mut self, tm: DateTime<Utc>) -> Self {
        self.week_or_ym = (match tm.year_ce() {
            (true, yr) => yr - 2000,
            (false, _) => {
                panic!("AID-INI packet only supports years after 2000");
            },
        } * 100
            + tm.month0()) as u16;
        self.tow_or_hms = tm.hour() * 10000 + tm.minute() * 100 + tm.second();
        self.tow_ns = tm.nanosecond() as i32;
        self.flags |= (1 << 1) | (1 << 10);
        self
    }
}

/// ALP client requests AlmanacPlus data from server
#[ubx_packet_recv]
#[ubx(class = 0x0B, id = 0x32, fixed_payload_len = 16)]
struct AlpSrv {
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

/// Messages in this class are sent as a result of a CFG message being
/// received, decoded and processed by thereceiver.
#[ubx_packet_recv]
#[ubx(class = 5, id = 1, fixed_payload_len = 2)]
struct AckAck {
    /// Class ID of the Acknowledged Message
    class: u8,

    /// Message ID of the Acknowledged Message
    msg_id: u8,
}

impl<'a> AckAckRef<'a> {
    pub fn is_ack_for<T: UbxPacketMeta>(&self) -> bool {
        self.class() == T::CLASS && self.msg_id() == T::ID
    }
}

/// Message Not-Acknowledge
#[ubx_packet_recv]
#[ubx(class = 5, id = 0, fixed_payload_len = 2)]
struct AckNak {
    /// Class ID of the Acknowledged Message
    class: u8,

    /// Message ID of the Acknowledged Message
    msg_id: u8,
}

impl<'a> AckNakRef<'a> {
    pub fn is_nak_for<T: UbxPacketMeta>(&self) -> bool {
        self.class() == T::CLASS && self.msg_id() == T::ID
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct ScaleBack<T: FloatCore + FromPrimitive + ToPrimitive>(T);

impl<T: FloatCore + FromPrimitive + ToPrimitive> ScaleBack<T> {
    fn as_i8(self, x: T) -> i8 {
        let x = (x * self.0).round();
        if x < T::from_i8(i8::min_value()).unwrap() {
            i8::min_value()
        } else if x > T::from_i8(i8::max_value()).unwrap() {
            i8::max_value()
        } else {
            x.to_i8().unwrap()
        }
    }

    fn as_i16(self, x: T) -> i16 {
        let x = (x * self.0).round();
        if x < T::from_i16(i16::min_value()).unwrap() {
            i16::min_value()
        } else if x > T::from_i16(i16::max_value()).unwrap() {
            i16::max_value()
        } else {
            x.to_i16().unwrap()
        }
    }

    fn as_i32(self, x: T) -> i32 {
        let x = (x * self.0).round();
        if x < T::from_i32(i32::MIN).unwrap() {
            i32::MIN
        } else if x > T::from_i32(i32::MAX).unwrap() {
            i32::MAX
        } else {
            x.to_i32().unwrap()
        }
    }

    fn as_u32(self, x: T) -> u32 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u32(u32::MAX).unwrap() {
                x.to_u32().unwrap()
            } else {
                u32::MAX
            }
        } else {
            0
        }
    }

    fn as_u16(self, x: T) -> u16 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u16(u16::MAX).unwrap() {
                x.to_u16().unwrap()
            } else {
                u16::MAX
            }
        } else {
            0
        }
    }

    fn as_u8(self, x: T) -> u8 {
        let x = (x * self.0).round();
        if !x.is_sign_negative() {
            if x <= T::from_u8(u8::MAX).unwrap() {
                x.to_u8().unwrap()
            } else {
                u8::MAX
            }
        } else {
            0
        }
    }
}

/// This message is used to retrieve a unique chip identifier
#[ubx_packet_recv]
#[ubx(class = 0x27, id = 0x03, fixed_payload_len = 9)]
struct SecUniqId {
    version: u8,
    reserved1: [u8; 3],
    unique_id: [u8; 5],
}

define_recv_packets!(
    enum PacketRef {
        _ = UbxUnknownPacketRef,
        NavPosLlh,
        // NavStatus,
        // NavDop,
        // NavPvt,
        // NavSolution,
        // NavVelNed,
        // NavHpPosLlh,
        // NavHpPosEcef,
        // NavTimeUTC,
        // NavTimeLs,
        // NavSat,
        // NavEoe,
        // NavOdo,
        // CfgOdo,
        // MgaAck,
        // MgaGpsIono,
        // MgaGpsEph,
        // MgaGloEph,
        // AlpSrv,
        AckAck,
        AckNak,
        // CfgItfm,
        // CfgPrtI2c,
        // CfgPrtSpi,
        // CfgPrtUart,
        // CfgNav5,
        // CfgAnt,
        // CfgTmode2,
        // CfgTmode3,
        // CfgTp5,
        // InfError,
        // InfWarning,
        // InfNotice,
        // InfTest,
        // InfDebug,
        // RxmRawx,
        // TimTp,
        // TimTm2,
        // MonVer,
        // MonGnss,
        // MonHw,
        // RxmRtcm,
        // EsfMeas,
        // EsfIns,
        // HnrAtt,
        // HnrIns,
        // HnrPvt,
        // NavAtt,
        // NavClock,
        // NavVelECEF,
        // MgaGpsEPH,
        // RxmSfrbx,
        // EsfRaw,
        // TimSvin,
        SecUniqId,
    }
);

#[test]
fn test_mon_ver_interpret() {
    let payload: [u8; 160] = [
        82, 79, 77, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41, 0,
        0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 70, 87, 86, 69, 82, 61, 83, 80,
        71, 32, 51, 46, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 82, 79, 84, 86,
        69, 82, 61, 49, 56, 46, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 80,
        83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];
    assert_eq!(Ok(()), <MonVerRef>::validate(&payload));
    let ver = MonVerRef(&payload);
    assert_eq!("ROM CORE 3.01 (107888)", ver.software_version());
    assert_eq!("00080000", ver.hardware_version());
    let mut it = ver.extension();
    assert_eq!("FWVER=SPG 3.01", it.next().unwrap());
    assert_eq!("PROTVER=18.00", it.next().unwrap());
    assert_eq!("GPS;GLO;GAL;BDS", it.next().unwrap());
    assert_eq!("SBAS;IMES;QZSS", it.next().unwrap());
    assert_eq!(None, it.next());
}
