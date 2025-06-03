use num_traits::cast::{FromPrimitive, ToPrimitive};
use num_traits::float::FloatCore;
use ublox_derive::ubx_extend;

#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

pub mod packetref_proto17;
pub mod packetref_proto23;
pub mod packetref_proto27;
pub mod packetref_proto31;

#[cfg(feature = "ubx_proto14")]
pub use packetref_proto17::PacketRef;
#[cfg(feature = "ubx_proto23")]
pub use packetref_proto23::PacketRef;
#[cfg(feature = "ubx_proto27")]
pub use packetref_proto27::PacketRef;
#[cfg(feature = "ubx_proto31")]
pub use packetref_proto31::PacketRef;

#[cfg(feature = "ubx_proto14")]
pub(crate) use packetref_proto17::{match_packet, MAX_PAYLOAD_LEN};
#[cfg(feature = "ubx_proto23")]
pub(crate) use packetref_proto23::{match_packet, MAX_PAYLOAD_LEN};
#[cfg(feature = "ubx_proto27")]
pub(crate) use packetref_proto27::{match_packet, MAX_PAYLOAD_LEN};
#[cfg(feature = "ubx_proto31")]
pub(crate) use packetref_proto31::{match_packet, MAX_PAYLOAD_LEN};

pub mod aid_ini;
pub use aid_ini::*;

pub mod ack;
pub use ack::*;

pub mod cfg_ant;
pub use cfg_ant::*;

pub mod cfg_esf_alg;
pub use cfg_esf_alg::*;

pub mod cfg_esf_wt;
pub use cfg_esf_wt::*;

pub mod cfg_inf;
pub use cfg_inf::*;

pub mod cfg_itfm;
pub use cfg_itfm::*;

pub mod cfg_msg;
pub use cfg_msg::*;

pub mod cfg_nav5;
pub use cfg_nav5::*;

pub mod cfg_navx5;
pub use cfg_navx5::*;

pub mod cfg_odo;
pub use cfg_odo::*;

pub mod cfg_prt;
pub use cfg_prt::*;

pub mod cfg_rate;
pub use cfg_rate::*;

pub mod cfg_rst;
pub use cfg_rst::*;

pub mod cfg_smgr;
pub use cfg_smgr::*;

pub mod cfg_tmode2;
pub use cfg_tmode2::*;

pub mod cfg_tmode3;
pub use cfg_tmode3::*;

pub mod cfg_tp5;
pub use cfg_tp5::*;

pub mod cfg_val;
pub use cfg_val::*;

pub mod esf_alg;
pub use esf_alg::*;

pub mod esf_ins;
pub use esf_ins::*;

pub mod esf_meas;
#[cfg(not(feature = "ubx_proto14"))]
pub use esf_meas::*;

pub mod esf_raw;
pub use esf_raw::*;

pub mod esf_status;
#[cfg(not(feature = "ubx_proto14"))]
pub use esf_status::*;

pub mod hnr_att;
pub use hnr_att::*;

pub mod hnr_ins;
pub use hnr_ins::*;

pub mod hnr_pvt;
pub use hnr_pvt::*;

pub(crate) mod inf;

pub mod inf_debug;
pub use inf_debug::*;

pub mod inf_error;
pub use inf_error::*;

pub mod inf_notice;
pub use inf_notice::*;

pub mod inf_test;
pub use inf_test::*;

pub mod inf_warning;
pub use inf_warning::*;

pub mod mga_ack;
pub use mga_ack::*;

pub mod mga_glo_eph;
pub use mga_glo_eph::*;

pub mod mga_gps_eph;
pub use mga_gps_eph::*;

pub mod mga_gps_iono;
pub use mga_gps_iono::*;

pub mod mga_gal_eph;
pub use mga_gal_eph::*;

pub mod mga_bds_eph;
pub use mga_bds_eph::*;

pub mod mon_gnss;
pub use mon_gnss::*;

pub mod mon_hw;
pub use mon_hw::*;

pub mod mon_ver;
pub use mon_ver::*;

pub mod nav_att;
pub use nav_att::*;

pub mod nav_clock;
pub use nav_clock::*;

pub mod nav_dop;
pub use nav_dop::*;

pub mod nav_other;
pub use nav_other::*;

pub mod nav_pvt;
pub mod nav_pvt_proto14;
#[cfg(not(feature = "ubx_proto14"))]
pub use nav_pvt::*;
#[cfg(feature = "ubx_proto14")]
pub use nav_pvt_proto14::*;

pub mod nav_rel_pos_ned_p14_23;
pub mod nav_rel_pos_ned_p27_31;

#[cfg(any(feature = "ubx_proto14", feature = "ubx_proto23"))]
pub use nav_rel_pos_ned_p14_23::*;
#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
pub use nav_rel_pos_ned_p27_31::*;

pub mod nav_pos_llh;
pub use nav_pos_llh::*;

pub mod nav_sat;
pub use nav_sat::*;

pub mod nav_sig;
#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
pub use nav_sig::*;

pub mod nav_sol;
pub use nav_sol::*;

pub mod nav_status;
pub use nav_status::*;

pub mod nav_time_ls;
pub use nav_time_ls::*;

pub mod nav_time_utc;
pub use nav_time_utc::*;

pub mod nav_vel_ned;
pub use nav_vel_ned::*;

pub mod nav_hp_pos_llh;
pub use nav_hp_pos_llh::*;

pub mod nav_hp_pos_ecef;
pub use nav_hp_pos_ecef::*;

pub mod rxm_rawx;
pub use rxm_rawx::*;

pub mod rxm_rtcm;
pub use rxm_rtcm::*;

pub mod rxm_sfrbx;
pub use rxm_sfrbx::*;

pub mod sec_uniq_id;
pub use sec_uniq_id::*;

pub mod tim_svin;
pub use tim_svin::*;

pub mod tim_tm2;
pub use tim_tm2::*;

pub mod tim_tp;
pub use tim_tp::*;

pub mod tim_tos;
pub use tim_tos::*;

pub mod proto17_packets;
#[cfg(feature = "ubx_proto14")]
pub use proto17_packets::*;

/// Used to help serialize the packet's fields flattened within a struct containing the msg_id and class fields, but
/// without using the serde FlatMapSerializer which requires alloc.
#[cfg(feature = "serde")]
pub(crate) trait SerializeUbxPacketFields {
    fn serialize_fields<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeMap;
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct ScaleBack<T: FloatCore + FromPrimitive + ToPrimitive>(T);

impl<T: FloatCore + FromPrimitive + ToPrimitive> ScaleBack<T> {
    fn as_i8(self, x: T) -> i8 {
        let x = (x * self.0).round();
        if x < T::from_i8(i8::MIN).unwrap() {
            i8::MIN
        } else if x > T::from_i8(i8::MAX).unwrap() {
            i8::MAX
        } else {
            x.to_i8().unwrap()
        }
    }

    fn as_i16(self, x: T) -> i16 {
        let x = (x * self.0).round();
        if x < T::from_i16(i16::MIN).unwrap() {
            i16::MIN
        } else if x > T::from_i16(i16::MAX).unwrap() {
            i16::MAX
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

/// UTC standard to be used
#[derive(Default)]
#[ubx_extend]
#[ubx(from_unchecked, into_raw, rest_error)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UtcStandardIdentifier {
    /// receiver selects based on GNSS configuration (see GNSS timebases)
    #[default]
    Automatic = 0,
    /// UTC as operated by the U.S. NavalObservatory (USNO);
    /// derived from GPStime
    Usno = 3,
    /// UTC as operated by the former Soviet Union; derived from GLONASS time
    UtcSu = 6,
    /// UTC as operated by the National TimeService Center, China;
    /// derived from BeiDou time
    UtcChina = 7,
}

/// GNSS fix Type
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GnssFixType {
    NoFix = 0,
    DeadReckoningOnly = 1,
    Fix2D = 2,
    Fix3D = 3,
    GPSPlusDeadReckoning = 4,
    TimeOnlyFix = 5,
}
