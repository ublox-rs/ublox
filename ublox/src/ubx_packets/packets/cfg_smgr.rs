use bitflags::bitflags;

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use crate::{
    error::ParserError,
    ubx_checksum,
    ubx_packets::{packets::ScaleBack, UbxChecksumCalc},
    MemWriter, MemWriterError, UbxPacketCreator, UbxPacketMeta, SYNC_CHAR_1, SYNC_CHAR_2,
};
use ublox_derive::{ubx_extend_bitflags, ubx_packet_recv_send};

/// Synchronization management configuration frame
#[ubx_packet_recv_send]
#[ubx(
    class = 0x06,
    id = 0x62,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
#[derive(Debug)]
struct CfgSmgr {
    version: u8,
    /// Minimum # of GNSS fixes before we
    /// commit to use it as a source
    min_gnss_fix: u8,
    /// Maximum frequency rate change, in ppb/sec,
    /// when disciplining. Must be < 30 ppb/s.
    #[ubx(map_type = f32, scale = 1.0)]
    max_freq_change_rate: u16,
    /// Maximum phase correction rate, in ns/s
    /// in coherent time pulse mode.
    /// Must be < 100 ns/s
    max_phase_corr_rate: u16,
    reserved1: u16,
    /// Limit possible deviation in ppb,
    /// before UBX-TIM-TOS indicates that frequency
    /// is out of tolerance
    #[ubx(map_type = f32, scale = 1.0)]
    freq_tolerance: u16,
    /// Limit possible deviation, in ns,
    /// before UBX-TIM-TOS indicates that pulse
    /// is out of tolerance
    #[ubx(map_type = f32, scale = 1.0)]
    time_tolerance: u16,
    /// Message configuration, see [CfgSmgrMsgFlags]
    #[ubx(map_type = CfgSmgrMsgFlags)]
    msg: u16,
    /// Maximum slew rate, in s/s
    #[ubx(map_type = f32, scale = 1.0E-6)]
    max_slew_rate: u16,
    /// Configuration flags, see [CfgSmgrFlags]
    #[ubx(map_type = CfgSmgrFlags)]
    flags: u32,
}

/// Synchronization Manager message flags
#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Sync manager message flags
    #[derive(Default, Debug)]
    pub struct CfgSmgrMsgFlags: u16 {
        /// Report internal oscillator offset estimate from oscillator model
        const MEAS_INTERNAL1 = 0x01;
        /// Report internal oscillator offset relative to GNSS
        const MEAS_GNSS = 0x02;
        /// Report internal oscillator offset relative to EXTINT0 source
        const MEAS_EXTINT0 = 0x04;
        /// Report internal oscillator offset relative to EXTINT1 source
        const MEAS_EXTINT1 = 0x08;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, into_raw, rest_reserved)]
bitflags! {
    /// Synchronization Manager config flags
    #[derive(Default, Debug)]
    pub struct CfgSmgrFlags: u32 {
        /// Disable internal Osc. disciplining
        const DISABLE_INTERNAL = 0x01;
        /// Disable external Osc. disciplining
        const DISABLE_EXTERNAL = 0x02;
        /// Reference selection preference,
        /// `Best Phase accuracy` when set,
        /// `Best frequency accuracy` when unset
        const BEST_PHASE_ACCURACY_PREFERENCE = 0x04;
        /// Enables GNSS as sync source
        const ENABLE_GNSS = 0x08;
        /// Enables ExtInt0 as sync source
        const ENABLE_EXTINT0 = 0x10;
        /// Enables ExtInt1 as sync source
        const ENABLE_EXTINT1 = 0x20;
        /// Enable host measurements of the internal
        /// oscillator as sync source.
        /// TimSmeasData0 frame should be used
        /// to send measurements data
        const ENABLE_HOST_MEAS_INT = 0x40;
        /// Enable host measurements of the external
        /// oscillator as sync source.
        /// TimSmeasData1 frame should be used
        /// to send measurements data
        const ENABLE_HOST_MEAS_EXT = 0x80;
        /// Uses any fix when asserted,
        /// otherwise, only `over determined` navigation
        /// solutions are used
        const USE_ANY_FIX = 0x100;
        /// MaxSlewRate field is discarded when asserted,
        /// otherwise MaxSlewRate field is used for
        /// maximum time correction, in corrective time pulse mode
        const DISABLE_MAX_SLEW_RATE = 0x200;
        /// Issues UBX-TIME-TOS warning when frequency uncertainty
        /// exceeds `freq_tolerance`
        const ISSUE_FREQ_WARNING = 0x400;
        /// Issues UBX-TIME-TOS warning when time uncertainty
        /// exceeds `time_tolerance`
        const ISSUE_TIME_WARNING = 0x800;
        /// Coherence Pulses. Time phase offsets will be corrected
        /// gradually by varying the GNSS oscillator rate within
        /// freq. tolerance limits.
        const TP_COHERENT_PULSES = 0x1000;
        /// Non coherence Pulses. Time phase offsets will be corrected
        /// as quickly as allowed by specified `max_slew_rate`
        const TP_NON_COHERENCE_PULSES = 0x2000;
        /// Post init. coherent pulses.
        /// Starts off in non coherent mode, then automatically switches
        /// to coherent pulse mode, when PLL is locked
        const TP_POST_INIT_COHERENT_PULSES = 0x4000;
        /// Disable automatic storage of oscillator offset
        const DISABLE_OFFSET_STORAGE = 0x8000;
    }
}
