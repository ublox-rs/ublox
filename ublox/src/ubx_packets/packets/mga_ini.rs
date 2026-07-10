use chrono::prelude::*;
use chrono::Datelike;

use crate::{
    ubx_checksum, MemWriter, MemWriterError, PositionLLA, UbxPacketCreator, UbxPacketMeta,
};
use ublox_derive::ubx_packet_send;

#[ubx_packet_send]
#[ubx(
    class = 0x13,
    id = 0x40,
    fixed_payload_len = 20,
    flags = "default_for_builder"
)]
struct MgaIniPos {
    msg_type: u8,
    version: u8,
    reserved0: [u8; 2],
    lat: i32,
    lon: i32,
    alt: i32,
    pos_acc: u32,
}

impl MgaIniPosBuilder {
    /// Initializes the builder with the correct type and version, and sets the position.
    pub fn set_position(mut self, pos: PositionLLA) -> Self {
        self.msg_type = 0x01; // UBX_MGA_INI_POS_TYPE_LLH
        self.version = 0x00; // UBX_MGA_INI_POS_VERSION_LLH
        self.lat = (pos.lat * 10_000_000.0) as i32;
        self.lon = (pos.lon * 10_000_000.0) as i32;
        self.alt = (pos.alt * 100.0) as i32; // Height in centimeters
        self
    }

    /// Sets the position accuracy (standard deviation) in centimeters.
    pub fn set_accuracy(mut self, accuracy_cm: u32) -> Self {
        self.pos_acc = accuracy_cm;
        self
    }
}

#[ubx_packet_send]
#[ubx(
    class = 0x13,
    id = 0x40,
    fixed_payload_len = 24,
    flags = "default_for_builder"
)]
struct MgaIniTimeUtc {
    msg_type: u8,
    version: u8,
    source: u8,
    leap_secs: i8,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    trusted_source: u8,
    ns: u32,
    t_acc_s: u16,
    reserved0: [u8; 2],
    t_acc_ns: u32,
}

impl MgaIniTimeUtcBuilder {
    /// Initializes the builder with the correct type, version, and sets the UTC time.
    pub fn set_time(mut self, tm: DateTime<Utc>) -> Self {
        self.msg_type = 0x10; // UBX_MGA_INI_TIME_TYPE_UTC
        self.version = 0x00; // UBX_MGA_INI_TIME_VERSION_UTC

        // Default to unknown leap seconds as defined in the spec
        self.leap_secs = -128; // 0x80 (i8 equivalent of uint8 128 is -128)

        // Source 0x00 = none (on receipt of message)
        self.source = 0x00;

        self.year = tm.year() as u16;
        self.month = tm.month() as u8;
        self.day = tm.day() as u8;
        self.hour = tm.hour() as u8;
        self.minute = tm.minute() as u8;
        self.second = tm.second() as u8;
        self.ns = tm.nanosecond();

        self
    }

    /// Overrides the default time source and trusted status.
    pub fn set_source(mut self, source: u8, trusted: bool) -> Self {
        self.source = source;
        self.trusted_source = if trusted { 1 } else { 0 };
        self
    }

    /// Sets the time accuracy in seconds and nanoseconds.
    pub fn set_accuracy(mut self, acc_s: u16, acc_ns: u32) -> Self {
        self.t_acc_s = acc_s;
        self.t_acc_ns = acc_ns;
        self
    }

    /// Sets known leap seconds (if available).
    pub fn set_leap_seconds(mut self, leap_secs: i8) -> Self {
        self.leap_secs = leap_secs;
        self
    }
}
