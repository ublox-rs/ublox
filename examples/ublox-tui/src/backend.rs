use std::sync::mpsc::Sender;
use std::thread;
use tracing::{debug, error, info, trace};
use ublox_device::ublox::{
    cfg_msg::{CfgMsgAllPorts, CfgMsgAllPortsBuilder},
    esf_alg::{EsfAlg, EsfAlgError},
    esf_meas::EsfMeas,
    esf_status::EsfStatus,
    mon_ver::MonVer,
    nav_pvt::{NavPvt, NavPvtFlags2},
    *,
};

use crate::app::{
    EsfAlgImuAlignmentWidgetState, EsfAlgStatusWidgetState, EsfMeasurementWidgetState,
    EsfSensorWidget, EsfSensorsWidgetState, MonVersionWidgetState, NavPvtWidgetState, UbxStatus,
};

pub struct UbxDevice {
    device: ublox_device::Device,
}

impl UbxDevice {
    pub fn from(device: ublox_device::Device) -> Self {
        Self { device }
    }

    pub fn configure(&mut self) {
        // Enable the NavPvt packet
        // By setting 1 in the array below, we enable the NavPvt message for Uart1, Uart2 and USB
        // The other positions are for I2C, SPI, etc. Consult your device manual.
        info!("Enable UBX-NAV-PVT message on all serial ports: USB, UART1 and UART2 ...");
        self.device
            .write_all(
                &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 1, 1, 0, 0])
                    .into_packet_bytes(),
            )
            .expect("Could not configure ports for UBX-NAV-PVT");

        self.device
            .wait_for_ack::<CfgMsgAllPorts>()
            .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

        // Send a packet request for the MonVer packet
        self.device
            .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-MON-VER message");

        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfAlg>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-ESF-ALG message");

        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfStatus>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-ESF-STATUS message");
        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfMeas>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-ESF-MEAS message");
    }

    pub fn run(mut self, sender: Sender<UbxStatus>) {
        info!("Opened uBlox device, waiting for messages...");
        thread::spawn(move || loop {
            let res = self.device.on_data_available(|packet| match packet {
                PacketRef::MonVer(pkg) => {
                    trace!("{:?}", pkg);
                    info!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        pkg.software_version(),
                        pkg.hardware_version(),
                        pkg.extension().collect::<Vec<&str>>()
                    );
                    let mut state = MonVersionWidgetState::default();

                    state
                        .software_version
                        .copy_from_slice(pkg.software_version_raw());
                    state
                        .hardware_version
                        .copy_from_slice(pkg.hardware_version_raw());

                    for s in pkg.extension() {
                        state.extensions.push_str(s);
                    }

                    sender.send(UbxStatus::MonVer(Box::new(state))).unwrap();
                },
                PacketRef::NavPvt(pkg) => {
                    let mut state = NavPvtWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };

                    state.flags2 = pkg.flags2();

                    if pkg.flags2().contains(NavPvtFlags2::CONFIRMED_AVAI) {
                        state.day = pkg.day();
                        state.month = pkg.month();
                        state.year = pkg.year();
                        state.hour = pkg.hour();
                        state.min = pkg.min();
                        state.sec = pkg.sec();
                        state.nanosecond = pkg.nanosec();

                        state.utc_time_accuracy = pkg.time_accuracy();
                    }

                    state.position_fix_type = pkg.fix_type();
                    state.fix_flags = pkg.flags();

                    state.lat = pkg.latitude();
                    state.lon = pkg.longitude();
                    state.height = pkg.height_above_ellipsoid();
                    state.msl = pkg.height_msl();

                    state.vel_ned = (pkg.vel_north(), pkg.vel_east(), pkg.vel_down());

                    state.speed_over_ground = pkg.ground_speed_2d();
                    state.heading_motion = pkg.heading_motion();
                    state.heading_vehicle = pkg.heading_vehicle();

                    state.magnetic_declination = pkg.magnetic_declination();

                    state.pdop = pkg.pdop();

                    state.satellites_used = pkg.num_satellites();

                    state.invalid_llh = pkg.flags3().invalid_llh();
                    state.position_accuracy = (pkg.horizontal_accuracy(), pkg.vertical_accuracy());
                    state.velocity_accuracy = pkg.speed_accuracy();
                    state.heading_accuracy = pkg.heading_accuracy();
                    state.magnetic_declination_accuracy = pkg.magnetic_declination_accuracy();

                    sender.send(UbxStatus::Pvt(Box::new(state))).unwrap();
                    debug!("{:?}", pkg);
                },
                PacketRef::EsfAlg(pkg) => {
                    let mut state = EsfAlgImuAlignmentWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };
                    state.roll = pkg.roll();
                    state.pitch = pkg.pitch();
                    state.yaw = pkg.yaw();

                    state.auto_alignment = pkg.flags().auto_imu_mount_alg_on();
                    state.alignment_status = pkg.flags().status();

                    if pkg.error().contains(EsfAlgError::ANGLE_ERROR) {
                        state.angle_singularity = true;
                    }

                    sender.send(UbxStatus::EsfAlgImu(state)).unwrap();
                    // debug!("{:?}", pkg);
                },

                PacketRef::EsfStatus(pkg) => {
                    let mut alg_state = EsfAlgStatusWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };
                    alg_state.fusion_mode = pkg.fusion_mode();

                    alg_state.imu_status = pkg.init_status2().imu_init_status();
                    alg_state.ins_status = pkg.init_status1().ins_initialization_status();
                    alg_state.ins_status = pkg.init_status1().ins_initialization_status();
                    alg_state.wheel_tick_sensor_status =
                        pkg.init_status1().wheel_tick_init_status();

                    let mut sensors = EsfSensorsWidgetState::default();
                    let mut sensor_state = EsfSensorWidget::default();
                    for s in pkg.data() {
                        if s.sensor_used() {
                            sensor_state.sensor_type = s.sensor_type();
                            sensor_state.freq = s.freq();
                            sensor_state.faults = s.faults();
                            sensor_state.calib_status = s.calibration_status();
                            sensor_state.time_status = s.time_status();
                            sensors.sensors.push(sensor_state.clone());
                        }
                    }

                    sender.send(UbxStatus::EsfAlgStatus(alg_state)).unwrap();
                    sender.send(UbxStatus::EsfAlgSensors(sensors)).unwrap();
                    // debug!("{:?}", pkg);
                },

                PacketRef::EsfMeas(pkg) => {
                    let mut esf_meas = EsfMeasurementWidgetState {
                        time_tag: (pkg.itow() as f64) / 1000.0,
                        ..Default::default()
                    };
                    for s in pkg.data() {
                        esf_meas.measurements.push(s)
                    }

                    sender.send(UbxStatus::EsfMeas(esf_meas)).unwrap();
                    // debug!("{:?}", pkg);
                },
                _ => {
                    trace!("{:?}", packet);
                },
            });
            if let Err(e) = res {
                error!("Stopping UBX messages parsing thread. Failed to parse incoming UBX packet: {e}");
            }
        });
    }
}
