use anyhow::{bail, Result};
use dds::{cli, idl};
use log::{debug, error, info, trace, warn};
use rustdds::{with_key::DataWriter, DomainParticipant};
use std::{io::ErrorKind, time};
use ublox::{
    CfgMsgAllPorts, CfgMsgAllPortsBuilder, MonVer, NavPvt, NavPvtFlags2, PacketRef,
    UbxPacketRequest,
};
use ublox_device::UbxPacketHandler;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();

    let cli = cli::ublox_dds_args()
        .clone()
        .author(clap::crate_authors!())
        .version(clap::crate_version!().to_string())
        .about("Demonstrate how to use DDS to publish uBlox data across devices on same LAN")
        .name("dds_publisher");

    let serialport = ublox_device::cli::Command::serialport(cli.clone());
    let mut device = ublox_device::Device::new(serialport);
    let port_config = ublox_device::cli::Command::ubx_port_configuration_builder(cli.clone());
    device.configure_port(port_config).unwrap();

    // Enable the NavPvt packet
    // By setting 1 in the array below, we enable the NavPvt message for Uart1, Uart2 and USB
    // The other positions are for I2C, SPI, etc. Consult your device manual.
    println!("Enable UBX-NAV-PVT message on all serial ports: USB, UART1 and UART2 ...");
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 1, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-NAV-PVT");
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Send a packet request for the MonVer packet
    device
        .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-MON-VER message");

    let cli_args = cli.get_matches();
    let domain_id = *cli_args
        .get_one("domain_id")
        .expect("Domain ID not provided");
    let topic_key = cli_args
        .get_one::<String>("key")
        .cloned()
        .to_owned()
        .expect("DDS key provided");

    let topic_name = cli_args
        .get_one::<String>("topic")
        .cloned()
        .to_owned()
        .unwrap_or("ubx-nav-pvt".to_string());

    let qos = if cli_args.get_flag("reliable") {
        cli::create_reliable_qos(&cli_args)
    } else {
        info!("Default to BestEffort QoS as no argument was provided on cmdline");
        cli::create_besteffort_qos(&cli_args)
    };

    info!("Creating participant for domain ID: {domain_id}");
    let domain_participant = DomainParticipant::new(domain_id).unwrap();
    info!("Creating Publisher");
    let publisher = domain_participant.create_publisher(&qos).unwrap();

    let mut handler = PkgHandler {
        last_published: time::Instant::now(),
        dds_key: topic_key.clone(),
        nav_pvt_wrt: dds::create_writer::<idl::ubx::nav_pvt::NavPvt>(
            &topic_name,
            &domain_participant,
            &publisher,
            &qos,
        ),
        esf_alg_wrt: dds::create_writer::<idl::ubx::esf_alg::EsfAlg>(
            "ubx-esf-alg",
            &domain_participant,
            &publisher,
            &qos,
        ),
        esf_status_wrt: dds::create_writer::<idl::ubx::esf_status::EsfStatus>(
            "ubx-esf-status",
            &domain_participant,
            &publisher,
            &qos,
        ),
    };

    // Start reading data
    info!("uBlox device opened, waiting for UBX messages ...");
    loop {
        if let Err(e) = device.process(&mut handler) {
            match e.kind() {
                ErrorKind::Interrupted => {
                    let msg = "Receiving signal interrupt. Exiting ...";
                    error!("{msg}");
                    bail!("{msg}");
                },
                ErrorKind::BrokenPipe => {
                    let msg = "BrokenPipe. Exiting ...";
                    error!("{msg}");
                    bail!("{msg}");
                },
                _ => {
                    warn!("Failed to parse UBX packets due to: {e}");
                },
            }
        }
    }
}

pub(crate) struct PkgHandler {
    pub last_published: time::Instant,
    pub dds_key: String,
    pub nav_pvt_wrt: DataWriter<idl::ubx::nav_pvt::NavPvt>,
    pub esf_alg_wrt: DataWriter<idl::ubx::esf_alg::EsfAlg>,
    pub esf_status_wrt: DataWriter<idl::ubx::esf_status::EsfStatus>,
}

impl UbxPacketHandler for PkgHandler {
    fn handle(&mut self, packet: PacketRef) {
        match packet {
            ublox::PacketRef::MonVer(packet) => {
                debug!("{:?}", packet);
                info!(
                    "MonVer: SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
            },
            ublox::PacketRef::NavPvt(pkg) => {
                debug!("{:?}", pkg);
                let mut pvt = to_dds_pvt(&pkg);
                pvt.key = self.dds_key.clone();
                if let Err(e) = dds::write_sample(&self.nav_pvt_wrt, &pvt) {
                    warn!("failed to write PVT message: {e} ");
                } else {
                    info!(
                        "Published new NavPvt message on DDS at: {:.6} [sec] since start",
                        time::Instant::now()
                            .duration_since(self.last_published)
                            .as_secs_f64()
                    );
                }
            },
            ublox::PacketRef::EsfAlg(pkg) => {
                debug!("{:?}", pkg);
                let msg = idl::ubx::esf_alg::EsfAlg {
                    key: self.dds_key.clone(),
                    itow: pkg.itow(),
                    roll: pkg.roll() as f32,
                    pitch: pkg.pitch() as f32,
                    yaw: pkg.yaw() as f32,
                    flags: pkg.flags_raw(),
                    errors: pkg.error_raw(),
                };

                if let Err(e) = dds::write_sample(&self.esf_alg_wrt, &msg) {
                    warn!("failed to write EsfAlg message: {e} ");
                }
            },
            ublox::PacketRef::EsfStatus(pkg) => {
                debug!("{:?}", pkg);
                let fusion_state = idl::ubx::esf_status::EsfFusionStatus {
                    fusion_mode: pkg.fusion_mode_raw(),
                    imu_status: pkg.init_status2().imu_init_status_raw(),
                    ins_status: pkg.init_status1().ins_initialization_status_raw(),
                    wheel_tick_sensor_status: pkg.init_status1().wheel_tick_init_status_raw(),
                    imu_mount_alignment_status: pkg.init_status1().mount_angle_status_raw(),
                };

                let mut esf_status = idl::ubx::esf_status::EsfStatus {
                    key: self.dds_key.clone(),
                    itow: pkg.itow(),
                    fusion_status: fusion_state,
                    ..Default::default()
                };
                for s in pkg.data() {
                    let mut sensor_state = idl::ubx::esf_status::EsfSensorStatus::default();
                    if s.sensor_used() {
                        sensor_state.sensor_type = s.sensor_type_raw();
                        sensor_state.freq = s.freq();
                        sensor_state.faults = s.faults_raw();
                        sensor_state.calibration_status = s.calibration_status_raw();
                        sensor_state.timing_status = s.time_status_raw();
                        esf_status.sensors.push(sensor_state.clone());
                    }
                }

                if let Err(e) = dds::write_sample(&self.esf_status_wrt, &esf_status) {
                    warn!("failed to write EsfStatus message: {e} ");
                }
            },
            _ => {
                trace!("{:?}", packet);
            },
        }
    }
}

pub fn to_dds_pvt(pkg: &ublox::NavPvtRef) -> idl::ubx::nav_pvt::NavPvt {
    let mut pvt = idl::ubx::nav_pvt::NavPvt {
        itow: pkg.itow(),
        ..Default::default()
    };

    pvt.time_confirmation_flags = pkg.flags2_raw();

    if pkg.flags2().contains(NavPvtFlags2::CONFIRMED_AVAI) {
        pvt.day = pkg.day();
        pvt.month = pkg.month();
        pvt.year = pkg.year();
        pvt.hour = pkg.hour();
        pvt.min = pkg.min();
        pvt.sec = pkg.sec();
        pvt.nanosec = pkg.nanosec();

        pvt.utc_time_accuracy = pkg.time_accuracy();
    }

    pvt.gps_fix_type = pkg.fix_type_raw();
    pvt.fix_flags = pkg.flags_raw();

    pvt.lat = pkg.latitude() as f32;
    pvt.lon = pkg.longitude() as f32;
    pvt.height = pkg.height_above_ellipsoid() as f32;
    pvt.msl = pkg.height_msl() as f32;

    pvt.vel_n = pkg.vel_north() as f32;
    pvt.vel_e = pkg.vel_east() as f32;
    pvt.vel_d = pkg.vel_down() as f32;

    pvt.speed_over_ground = pkg.ground_speed_2d() as f32;
    pvt.heading_motion = pkg.heading_motion() as f32;
    pvt.heading_vehicle = pkg.heading_vehicle() as f32;
    pvt.magnetic_declination = pkg.magnetic_declination() as f32;

    pvt.pdop = pkg.pdop() as f32;

    pvt.satellites_used = pkg.num_satellites();

    pvt.llh_validity = !pkg.flags3().invalid_llh();
    pvt.horizontal_accuracy = pkg.horizontal_accuracy() as f32;
    pvt.vertical_accuracy = pkg.vertical_accuracy() as f32;
    pvt.velocity_accuracy = pkg.speed_accuracy() as f32;
    pvt.heading_accuracy = pkg.heading_accuracy() as f32;
    pvt.magnetic_declination_accuracy = pkg.magnetic_declination_accuracy() as f32;

    pvt
}
