use ublox_device::ublox::{
    cfg_msg::{CfgMsgAllPorts, CfgMsgAllPortsBuilder},
    mon_ver::MonVer,
    nav_pvt, UbxPacket, UbxPacketRequest,
};

/// Use proto23 if enabled, otherwise use proto27 if enabled, otherwise use proto31, otherwise use proto33, otherwise use proto14
#[cfg(feature = "ubx_proto23")]
pub type Proto = ublox_device::ublox::proto23::Proto23;
#[cfg(all(feature = "ubx_proto27", not(feature = "ubx_proto23")))]
pub type Proto = ublox_device::ublox::proto27::Proto27;
#[cfg(all(
    feature = "ubx_proto31",
    not(any(feature = "ubx_proto23", feature = "ubx_proto27"))
))]
pub type Proto = ublox_device::ublox::proto31::Proto31;
#[cfg(all(
    feature = "ubx_proto33",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31",
    ))
))]
pub type Proto = ublox_device::ublox::proto33::Proto33;
#[cfg(all(
    feature = "ubx_proto14",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31",
        feature = "ubx_proto33",
    ))
))]
pub type Proto = ublox_device::ublox::proto14::Proto14;

fn main() {
    let mut cli = ublox_device::cli::CommandBuilder::default().build();
    cli = cli
        .about(clap::crate_description!())
        .name(clap::crate_name!())
        .author(clap::crate_authors!());

    let serialport = ublox_device::cli::Command::serialport(cli.clone())
        .expect("Could not connect to serialport");

    let mut device: ublox_device::Device<Proto> = ublox_device::Device::new(serialport);
    let port_config = ublox_device::cli::Command::ubx_port_configuration_builder(cli);

    device.configure_port(port_config).unwrap();

    // Enable the NavPvt packet
    // By setting 1 in the array below, we enable the NavPvt message for Uart1, Uart2 and USB
    // The other positions are for I2C, SPI, etc. Consult your device manual.
    println!("Enable UBX-NAV-PVT message on all serial ports: USB, UART1 and UART2 ...");
    device
        .write_all(&cfg_msg_enable_nav_pvt_bytes())
        .expect("Could not configure ports for UBX-NAV-PVT");
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Send a packet request for the MonVer packet
    device
        .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-MON-VER message");

    // Start reading data
    println!("Opened uBlox device, waiting for messages...");
    loop {
        device
            .on_data_available(|packet| match packet {
                #[cfg(feature = "ubx_proto14")]
                UbxPacket::Proto14(packet_ref) => handler::handle_packet_proto14(packet_ref),
                #[cfg(feature = "ubx_proto23")]
                UbxPacket::Proto23(packet_ref) => handler::handle_packet_proto23(packet_ref),
                #[cfg(feature = "ubx_proto27")]
                UbxPacket::Proto27(packet_ref) => handler::handle_packet_proto27(packet_ref),
                #[cfg(feature = "ubx_proto31")]
                UbxPacket::Proto31(packet_ref) => handler::handle_packet_proto31(packet_ref),
                #[cfg(feature = "ubx_proto33")]
                UbxPacket::Proto33(packet_ref) => handler::handle_packet_proto33(packet_ref),
            })
            .unwrap();
    }
}

mod handler {
    use chrono::{DateTime, Utc};
    use ublox_device::ublox::{GnssFixType, PositionLLA, Velocity};

    #[cfg(feature = "ubx_proto14")]
    pub fn handle_packet_proto14(packet_ref: ublox_device::ublox::proto14::PacketRef) {
        use ublox_device::ublox::packetref_proto14::PacketRef;
        match &packet_ref {
            PacketRef::MonVer(packet) => {
                println!(
                    "SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
                println!("{packet:?}");
            },
            PacketRef::NavPvt(pvt) => {
                let has_time = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                    || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                if has_posvel {
                    let pos: PositionLLA = pvt.into();
                    let vel: Velocity = pvt.into();
                    let lat = pos.lat;
                    let lon = pos.lon;
                    let alt = pos.alt;
                    let speed = vel.speed;
                    let heading = vel.heading;
                    println!(
                        "NavPvt: Latitude: {lat:.5} Longitude: {lon:.5} Altitude: {alt:.2} m, Speed: {speed:.2} m/s Heading: {heading:.2} degrees"
                    );
                    println!("NavPvt full: {pvt:?}");
                }

                if has_time {
                    let time: DateTime<Utc> = pvt
                        .try_into()
                        .expect("Could not parse NAV-PVT time field to UTC");
                    println!("Time: {time:?}");
                }
            },
            _ => {
                println!("{packet_ref:?}");
            },
        }
    }

    #[cfg(feature = "ubx_proto23")]
    pub fn handle_packet_proto23(packet_ref: ublox_device::ublox::proto23::PacketRef) {
        use ublox_device::ublox::packetref_proto23::PacketRef;
        match &packet_ref {
            PacketRef::MonVer(packet) => {
                println!(
                    "SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
                println!("{packet:?}");
            },
            PacketRef::NavPvt(pvt) => {
                let has_time = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                    || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                if has_posvel {
                    let pos: PositionLLA = pvt.into();
                    let vel: Velocity = pvt.into();
                    let lat = pos.lat;
                    let lon = pos.lon;
                    let alt = pos.alt;
                    let speed = vel.speed;
                    let heading = vel.heading;
                    println!(
                        "NavPvt: Latitude: {lat:.5} Longitude: {lon:.5} Altitude: {alt:.2} m, Speed: {speed:.2} m/s Heading: {heading:.2} degrees"
                    );
                    println!("NavPvt full: {pvt:?}");
                }

                if has_time {
                    let time: DateTime<Utc> = pvt
                        .try_into()
                        .expect("Could not parse NAV-PVT time field to UTC");
                    println!("Time: {time:?}");
                }
            },
            _ => {
                println!("{packet_ref:?}");
            },
        }
    }

    #[cfg(feature = "ubx_proto27")]
    pub fn handle_packet_proto27(packet_ref: ublox_device::ublox::proto27::PacketRef) {
        use ublox_device::ublox::packetref_proto27::PacketRef;
        match &packet_ref {
            PacketRef::MonVer(packet) => {
                println!(
                    "SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
                println!("{packet:?}");
            },
            PacketRef::NavPvt(pvt) => {
                let has_time = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                    || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                if has_posvel {
                    let pos: PositionLLA = pvt.into();
                    let vel: Velocity = pvt.into();
                    let lat = pos.lat;
                    let lon = pos.lon;
                    let alt = pos.alt;
                    let speed = vel.speed;
                    let heading = vel.heading;
                    println!(
                        "NavPvt: Latitude: {lat:.5} Longitude: {lon:.5} Altitude: {alt:.2} m, Speed: {speed:.2} m/s Heading: {heading:.2} degrees"
                    );
                    println!("NavPvt full: {pvt:?}");
                }

                if has_time {
                    let time: DateTime<Utc> = pvt
                        .try_into()
                        .expect("Could not parse NAV-PVT time field to UTC");
                    println!("Time: {time:?}");
                }
            },
            _ => {
                println!("{packet_ref:?}");
            },
        }
    }

    #[cfg(feature = "ubx_proto31")]
    pub fn handle_packet_proto31(packet_ref: ublox_device::ublox::proto31::PacketRef) {
        use ublox_device::ublox::packetref_proto31::PacketRef;
        match &packet_ref {
            PacketRef::MonVer(packet) => {
                println!(
                    "SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
                println!("{packet:?}");
            },
            PacketRef::NavPvt(pvt) => {
                let has_time = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                    || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                if has_posvel {
                    let pos: PositionLLA = pvt.into();
                    let vel: Velocity = pvt.into();
                    let lat = pos.lat;
                    let lon = pos.lon;
                    let alt = pos.alt;
                    let speed = vel.speed;
                    let heading = vel.heading;
                    println!(
                        "NavPvt: Latitude: {lat:.5} Longitude: {lon:.5} Altitude: {alt:.2} m, Speed: {speed:.2} m/s Heading: {heading:.2} degrees"
                    );
                    println!("NavPvt full: {pvt:?}");
                }

                if has_time {
                    let time: DateTime<Utc> = pvt
                        .try_into()
                        .expect("Could not parse NAV-PVT time field to UTC");
                    println!("Time: {time:?}");
                }
            },
            _ => {
                println!("{packet_ref:?}");
            },
        }
    }

    #[cfg(feature = "ubx_proto33")]
    pub fn handle_packet_proto33(packet_ref: ublox_device::ublox::proto33::PacketRef) {
        use ublox_device::ublox::packetref_proto33::PacketRef;
        match &packet_ref {
            PacketRef::MonVer(packet) => {
                println!(
                    "SW version: {} HW version: {}; Extensions: {:?}",
                    packet.software_version(),
                    packet.hardware_version(),
                    packet.extension().collect::<Vec<&str>>()
                );
                println!("{packet:?}");
            },
            PacketRef::NavPvt(pvt) => {
                let has_time = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                    || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                    || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                if has_posvel {
                    let pos: PositionLLA = pvt.into();
                    let vel: Velocity = pvt.into();
                    let lat = pos.lat;
                    let lon = pos.lon;
                    let alt = pos.alt;
                    let speed = vel.speed;
                    let heading = vel.heading;
                    println!(
                        "NavPvt: Latitude: {lat:.5} Longitude: {lon:.5} Altitude: {alt:.2} m, Speed: {speed:.2} m/s Heading: {heading:.2} degrees"
                    );
                    println!("NavPvt full: {pvt:?}");
                }

                if has_time {
                    let time: DateTime<Utc> = pvt
                        .try_into()
                        .expect("Could not parse NAV-PVT time field to UTC");
                    println!("Time: {time:?}");
                }
            },
            _ => {
                println!("{packet_ref:?}");
            },
        }
    }
}

fn cfg_msg_enable_nav_pvt_bytes() -> [u8; 16] {
    #[cfg(feature = "ubx_proto14")]
    use nav_pvt::proto14::NavPvt;

    #[cfg(all(feature = "ubx_proto23", not(feature = "ubx_proto14")))]
    use nav_pvt::proto23::NavPvt;

    #[cfg(all(
        feature = "ubx_proto27",
        not(any(feature = "ubx_proto14", feature = "ubx_proto23"))
    ))]
    use nav_pvt::proto27::NavPvt;

    #[cfg(all(
        feature = "ubx_proto31",
        not(any(
            feature = "ubx_proto14",
            feature = "ubx_proto23",
            feature = "ubx_proto27"
        ))
    ))]
    use nav_pvt::proto31::NavPvt;

    #[cfg(all(
        feature = "ubx_proto33",
        not(any(
            feature = "ubx_proto14",
            feature = "ubx_proto23",
            feature = "ubx_proto27",
            feature = "ubx_proto31",
        ))
    ))]
    use nav_pvt::proto33::NavPvt;
    CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 1, 1, 0, 0]).into_packet_bytes()
}
