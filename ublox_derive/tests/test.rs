use ublox_derive::{UbxDefineSubTypes, UbxPacketRecv, UbxPacketSend};

trait UbxPacket {
    const class: u8;
    const id: u8;
    const fixed_length: Option<u16>;
}

#[derive(UbxPacketRecv, UbxDefineSubTypes)]
#[ubx(class = 1, id = 2, fixed_len = 28)]
#[repr(packed)]
/// Geodetic Position Solution
struct NavPosLLHRaw {
    itow: u32,
    #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
    lon: i32,
    lat: i32,
    #[ubx(map_type = f64, scale = 1e-3, alias = height_ellipsoid_meters)]
    height_ellipsoid: i32,
    height_msl: i32,
    /// Horizontal Accuracy Estimate
    horizontal_accuracy: u32,
    vertical_accuracy: u32,
}

#[derive(UbxPacketRecv, UbxDefineSubTypes)]
#[repr(packed)]
#[ubx(class = 1, id = 3, fixed_len = 16)]
struct NavStatusRaw {
    itow: u32,
    #[ubx(enum {
	NoFix = 0,
	DeadReckoningOnly = 1,
	Fix2D = 2,
	Fix3D = 3,
	GpsDeadReckoningCombine = 4,
	TimeOnlyFix = 5,
    })]
    gps_fix: u8,
    #[ubx(bitflags {
	GpsFixOk = bit0,
	DiffSoln = bit1,
	WknSet = bit2,
	TowSet = bit3,
    })]
    flags: u8,
    fix_status: u8,
    flags2: u8,
    time_to_first_fix: u32,
    uptime_ms: u32,
}

#[derive(UbxPacketSend, UbxDefineSubTypes)]
#[repr(packed)]
#[ubx(class = 5, id = 1, fixed_len = 0)]
struct AckAckRaw {}

#[derive(UbxPacketSend, UbxDefineSubTypes)]
#[repr(packed)]
#[ubx(class = 6, id = 4)]
struct CfgRstRaw {
    #[ubx(bitflags {
	Eph = bit0,
	Alm = bit1,
	Health = bit2,
	/// Klobuchard
	Klob = bit3,
	Pos = bit4,
	Clkd = bit5,
	Osc = bit6,
	Utc = bit7,
	Rtc = bit8,
	HotStart = mask0,
    })]
    nav_bbr_mask: u16,
    #[ubx(enum {
	/// Hardware reset (Watchdog) immediately
	HardwareReset = 0,
	ControlledSoftwareReset = 1,
    })]
    reset_mode: u8,
    reserved1: u8,
}
