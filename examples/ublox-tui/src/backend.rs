use std::sync::mpsc::Sender;
use std::thread;
use tracing::{debug, error, info, trace};
use ublox_device::ublox::{
    cfg_msg::{CfgMsgAllPorts, CfgMsgAllPortsBuilder},
    esf_alg::{EsfAlg, EsfAlgError, EsfAlgRef},
    esf_meas::{EsfMeas, EsfMeasRef},
    esf_status::{EsfStatus, EsfStatusRef},
    mon_ver::{MonVer, MonVerRef},
    nav_pvt::{
        common::NavPvtFlags2, proto23::NavPvtRef as NavPvt23, proto27::NavPvtRef as NavPvt27,
        proto31::NavPvtRef as NavPvt31, proto33::NavPvtRef as NavPvt33,
    },
    *,
};

use crate::app::{
    EsfAlgImuAlignmentWidgetState, EsfAlgStatusWidgetState, EsfMeasurementWidgetState,
    EsfSensorWidget, EsfSensorsWidgetState, MonVersionWidgetState, NavPvtWidgetState, UbxStatus,
};

pub struct UbxDevice<P: UbxProtocol> {
    device: ublox_device::Device<P>,
}

impl<P: UbxProtocol + 'static> UbxDevice<P> {
    pub fn from(device: ublox_device::Device<P>) -> Self {
        Self { device }
    }

    pub fn configure(&mut self) {
        // Enable the NavPvt packet
        // By setting 1 in the array below, we enable the NavPvt message for Uart1, Uart2 and USB
        // The other positions are for I2C, SPI, etc. Consult your device manual.
        info!("Enable UBX-NAV-PVT message on all serial ports: USB, UART1 and UART2 ...");
        self.device
            .write_all(
                &CfgMsgAllPortsBuilder::set_rate_for::<nav_pvt::proto31::NavPvt>([
                    0, 1, 1, 1, 0, 0,
                ])
                .into_packet_bytes(),
            )
            .expect("Could not configure ports for UBX-NAV-PVT");

        self.device
            .wait_for_ack::<CfgMsgAllPorts>()
            .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

        // Send a packet request for the MonVer packet
        self.device
            .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
            .expect("Failed to send poll/request for UBX-MON-VER message");

        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfAlg>().into_packet_bytes())
            .expect("Failed to send poll/request for UBX-ESF-ALG message");

        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfStatus>().into_packet_bytes())
            .expect("Failed to send poll/request for UBX-ESF-STATUS message");
        self.device
            .write_all(&UbxPacketRequest::request_for::<EsfMeas>().into_packet_bytes())
            .expect("Failed to send poll/request for UBX-ESF-MEAS message");
    }

    pub fn run(mut self, sender: Sender<UbxStatus>) {
        info!("Opened uBlox device, waiting for messages...");
        let sender = SenderWrapper { tx: sender };
        thread::spawn(move || loop {
            let res = self.device.on_data_available(|packet| match &packet {
                UbxPacket::Proto23(packet_ref) => {
                    use packetref_proto23::PacketRef;
                    match packet_ref {
                        PacketRef::MonVer(pkg) => {
                            sender.handle_monver(pkg);
                        },
                        PacketRef::NavPvt(pkg) => {
                            sender.handle_navpvt_23(pkg);
                        },
                        PacketRef::EsfAlg(pkg) => {
                            sender.handle_esfalg(pkg);
                        },

                        PacketRef::EsfStatus(pkg) => {
                            sender.handle_esf_status(pkg);
                        },

                        PacketRef::EsfMeas(pkg) => {
                            sender.handle_esf_meas(pkg);
                        },
                        _ => {
                            trace!("{packet:?}");
                        },
                    }
                },
                UbxPacket::Proto27(packet_ref) => {
                    use packetref_proto27::PacketRef;
                    match packet_ref {
                        PacketRef::MonVer(pkg) => {
                            sender.handle_monver(pkg);
                        },
                        PacketRef::NavPvt(pkg) => {
                            sender.handle_navpvt_27(pkg);
                        },
                        PacketRef::EsfAlg(pkg) => {
                            sender.handle_esfalg(pkg);
                        },

                        PacketRef::EsfStatus(pkg) => {
                            sender.handle_esf_status(pkg);
                        },

                        PacketRef::EsfMeas(pkg) => {
                            sender.handle_esf_meas(pkg);
                        },
                        _ => {
                            trace!("{packet:?}");
                        },
                    }
                },
                UbxPacket::Proto31(packet_ref) => {
                    use packetref_proto31::PacketRef;
                    match packet_ref {
                        PacketRef::MonVer(pkg) => {
                            sender.handle_monver(pkg);
                        },
                        PacketRef::NavPvt(pkg) => {
                            sender.handle_navpvt_31(pkg);
                        },
                        PacketRef::EsfAlg(pkg) => {
                            sender.handle_esfalg(pkg);
                        },

                        PacketRef::EsfStatus(pkg) => {
                            sender.handle_esf_status(pkg);
                        },

                        PacketRef::EsfMeas(pkg) => {
                            sender.handle_esf_meas(pkg);
                        },
                        _ => {
                            trace!("{packet:?}");
                        },
                    }
                },
                UbxPacket::Proto33(packet_ref) => {
                    use packetref_proto33::PacketRef;
                    match packet_ref {
                        PacketRef::MonVer(pkg) => {
                            sender.handle_monver(pkg);
                        },
                        PacketRef::NavPvt(pkg) => {
                            sender.handle_navpvt_33(pkg);
                        },
                        PacketRef::EsfAlg(pkg) => {
                            sender.handle_esfalg(pkg);
                        },

                        PacketRef::EsfStatus(pkg) => {
                            sender.handle_esf_status(pkg);
                        },

                        PacketRef::EsfMeas(pkg) => {
                            sender.handle_esf_meas(pkg);
                        },
                        _ => {
                            trace!("{packet:?}");
                        },
                    }
                },
                #[cfg(feature = "ubx_proto14")]
                UbxPacket::Proto14(_) => unimplemented!(),
            });
            if let Err(e) = res {
                error!(
                        "Stopping UBX messages parsing thread. Failed to parse incoming UBX packet: {e}"
                    );
            }
        });
    }
}

struct SenderWrapper {
    tx: Sender<UbxStatus>,
}

impl SenderWrapper {
    fn send(&self, msg: UbxStatus) {
        self.tx.send(msg).expect("failed sending ubx status")
    }

    fn handle_monver(&self, pkg: &MonVerRef) {
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

        self.send(UbxStatus::MonVer(Box::new(state)));
    }

    fn handle_navpvt_23(&self, pkg: &NavPvt23) {
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

        self.send(UbxStatus::Pvt(Box::new(state)));
        debug!("{pkg:?}");
    }

    fn handle_navpvt_31(&self, pkg: &NavPvt31) {
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

        self.send(UbxStatus::Pvt(Box::new(state)));
        debug!("{pkg:?}");
    }
    fn handle_navpvt_27(&self, pkg: &NavPvt27) {
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

        self.send(UbxStatus::Pvt(Box::new(state)));
        debug!("{pkg:?}");
    }

    fn handle_navpvt_33(&self, pkg: &NavPvt33) {
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

        self.send(UbxStatus::Pvt(Box::new(state)));
        debug!("{pkg:?}");
    }

    fn handle_esfalg(&self, pkg: &EsfAlgRef) {
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

        self.send(UbxStatus::EsfAlgImu(state));
        debug!("{pkg:?}");
    }

    fn handle_esf_status(&self, pkg: &EsfStatusRef) {
        let mut alg_state = EsfAlgStatusWidgetState {
            time_tag: (pkg.itow() / 1000) as f64,
            ..Default::default()
        };
        alg_state.fusion_mode = pkg.fusion_mode();

        alg_state.imu_status = pkg.init_status2().imu_init_status();
        alg_state.ins_status = pkg.init_status1().ins_initialization_status();
        alg_state.ins_status = pkg.init_status1().ins_initialization_status();
        alg_state.wheel_tick_sensor_status = pkg.init_status1().wheel_tick_init_status();

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

        self.send(UbxStatus::EsfAlgStatus(alg_state));
        self.send(UbxStatus::EsfAlgSensors(sensors));
    }

    fn handle_esf_meas(&self, pkg: &EsfMeasRef) {
        let mut esf_meas = EsfMeasurementWidgetState {
            time_tag: (pkg.itow() as f64) / 1000.0,
            ..Default::default()
        };
        for s in pkg.data() {
            esf_meas.measurements.push(s)
        }

        self.send(UbxStatus::EsfMeas(esf_meas));
    }
}
