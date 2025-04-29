use core::f64;
use std::{path::PathBuf, vec};

use ublox_device::ublox::{
    EsfAlgStatus, EsfMeasData, EsfSensorFaults, EsfSensorStatusCalibration, EsfSensorStatusTime,
    EsfSensorType, EsfStatusFusionMode, EsfStatusImuInit, EsfStatusInsInit, EsfStatusMountAngle,
    EsfStatusWheelTickInit, GnssFixType, NavPvtFlags, NavPvtFlags2,
};

use crate::{signal::Signal, ui::LogWidget};

#[allow(dead_code)]
pub struct App<'a> {
    pub title: &'a str,
    pub log_file: PathBuf,
    pub pvt_state: NavPvtWidgetState,
    pub mon_ver_state: MonVersionWidgetState,
    pub esf_sensors_state: EsfSensorsWidgetState,
    pub esf_alg_state: EsfAlgStatusWidgetState,
    pub esf_alg_imu_alignment_state: EsfAlgImuAlignmentWidgetState,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub log_widget: LogWidget,
    pub signals: Signals,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, log_file: PathBuf) -> Self {
        let signals = Signals {
            speed: Signal::new(200, 1.0),
            speed_tick: Signal::new(200, 1.0),
            acc_x: Signal::new(200, 1.0),
            acc_y: Signal::new(200, 1.0),
            acc_z: Signal::new(200, 1.0),
            gyro_x: Signal::new(200, 1.0),
            gyro_y: Signal::new(200, 1.0),
            gyro_z: Signal::new(200, 1.0),
            gyro_temp: Signal::new(200, 1.0),
            wt_fl: Signal::new(200, 1.0),
            wt_fr: Signal::new(200, 1.0),
            wt_rl: Signal::new(200, 1.0),
            wt_rr: Signal::new(200, 1.0),
        };

        App {
            title,
            log_file,
            pvt_state: NavPvtWidgetState::default(),
            mon_ver_state: MonVersionWidgetState::default(),
            esf_sensors_state: EsfSensorsWidgetState::default(),
            esf_alg_state: EsfAlgStatusWidgetState::default(),
            esf_alg_imu_alignment_state: EsfAlgImuAlignmentWidgetState::default(),
            should_quit: false,
            log_widget: LogWidget,
            tabs: TabsState::new(vec![
                "PVT",
                "ESF Status",
                "ESF Charts",
                "Version Info",
                "World Map",
            ]),
            signals,
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            },
            'Q' => {
                self.should_quit = true;
            },
            _ => {},
        }
    }
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub const fn new(titles: Vec<&'a str>) -> Self {
        Self { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub enum UbxStatus {
    Pvt(Box<NavPvtWidgetState>),
    MonVer(Box<MonVersionWidgetState>),
    EsfAlgImu(EsfAlgImuAlignmentWidgetState),
    EsfAlgSensors(EsfSensorsWidgetState),
    EsfAlgStatus(EsfAlgStatusWidgetState),
    EsfMeas(EsfMeasurementWidgetState),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct NavPvtWidgetState {
    pub time_tag: f64,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub time_accuracy: u32,
    pub nanosecond: i32,
    pub utc_time_accuracy: u32,
    pub lat: f64,
    pub lon: f64,
    pub height: f64,
    pub msl: f64,
    pub vel_ned: (f64, f64, f64),
    pub speed_over_ground: f64,
    pub heading_motion: f64,
    pub heading_vehicle: f64,
    pub magnetic_declination: f64,

    pub pdop: f64,
    pub satellites_used: u8,

    pub position_fix_type: GnssFixType,
    pub fix_flags: NavPvtFlags,
    pub invalid_llh: bool,
    pub position_accuracy: (f64, f64),
    pub velocity_accuracy: f64,
    pub heading_accuracy: f64,
    pub magnetic_declination_accuracy: f64,
    pub flags2: NavPvtFlags2,
}

impl Default for NavPvtWidgetState {
    fn default() -> Self {
        Self {
            time_tag: f64::NAN,
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            min: 0,
            sec: 0,
            valid: 0,
            time_accuracy: 0,
            nanosecond: 0,
            lat: f64::NAN,
            lon: f64::NAN,
            height: f64::NAN,
            msl: f64::NAN,
            vel_ned: (f64::NAN, f64::NAN, f64::NAN),
            speed_over_ground: f64::NAN,
            heading_motion: f64::NAN,
            heading_vehicle: f64::NAN,
            magnetic_declination: f64::NAN,
            pdop: f64::NAN,
            satellites_used: 0,
            utc_time_accuracy: 0,
            invalid_llh: true,
            position_accuracy: (f64::NAN, f64::NAN),
            velocity_accuracy: f64::NAN,
            heading_accuracy: f64::NAN,
            magnetic_declination_accuracy: f64::NAN,
            position_fix_type: GnssFixType::NoFix,
            fix_flags: NavPvtFlags::empty(),
            flags2: NavPvtFlags2::empty(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EsfSensorsWidgetState {
    pub sensors: Vec<EsfSensorWidget>,
}

#[derive(Debug, Clone)]
pub struct EsfSensorWidget {
    pub sensor_type: EsfSensorType,
    pub calib_status: EsfSensorStatusCalibration,
    pub time_status: EsfSensorStatusTime,
    pub freq: u16,
    pub faults: EsfSensorFaults,
}

impl Default for EsfSensorWidget {
    fn default() -> Self {
        Self {
            sensor_type: EsfSensorType::Invalid,
            calib_status: EsfSensorStatusCalibration::NotCalibrated,
            time_status: EsfSensorStatusTime::NoData,
            freq: 0,
            faults: EsfSensorFaults::default(),
        }
    }
}

pub struct EsfAlgStatusWidgetState {
    pub time_tag: f64,
    pub fusion_mode: EsfStatusFusionMode,
    pub imu_status: EsfStatusImuInit,
    pub wheel_tick_sensor_status: EsfStatusWheelTickInit,
    pub ins_status: EsfStatusInsInit,
    pub imu_mount_alignment_status: EsfStatusMountAngle,
}

impl Default for EsfAlgStatusWidgetState {
    fn default() -> Self {
        Self {
            time_tag: f64::NAN,
            fusion_mode: EsfStatusFusionMode::Disabled,
            imu_status: EsfStatusImuInit::Off,
            wheel_tick_sensor_status: EsfStatusWheelTickInit::Off,
            ins_status: EsfStatusInsInit::Off,
            imu_mount_alignment_status: EsfStatusMountAngle::Off,
        }
    }
}

#[derive(Debug)]
pub struct EsfAlgImuAlignmentWidgetState {
    pub time_tag: f64,
    pub auto_alignment: bool,
    pub alignment_status: EsfAlgStatus,
    pub angle_singularity: bool,
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

impl Default for EsfAlgImuAlignmentWidgetState {
    fn default() -> Self {
        Self {
            time_tag: f64::NAN,
            auto_alignment: false,
            alignment_status: EsfAlgStatus::UserDefinedAngles,
            angle_singularity: false,
            roll: f64::NAN,
            pitch: f64::NAN,
            yaw: f64::NAN,
        }
    }
}

#[derive(Debug, Default)]
pub struct EsfMeasurementWidgetState {
    pub time_tag: f64,
    pub measurements: Vec<EsfMeasData>,
}

#[derive(Debug, Default)]
pub struct MonVersionWidgetState {
    pub software_version: [u8; 30],
    pub hardware_version: [u8; 10],
    pub extensions: String,
}

#[derive(Debug, Default)]
pub struct Signals {
    pub speed: Signal,
    pub speed_tick: Signal,
    pub acc_x: Signal,
    pub acc_y: Signal,
    pub acc_z: Signal,
    pub gyro_x: Signal,
    pub gyro_y: Signal,
    pub gyro_z: Signal,
    pub gyro_temp: Signal,
    pub wt_fl: Signal,
    pub wt_fr: Signal,
    pub wt_rl: Signal,
    pub wt_rr: Signal,
}
