use chrono::prelude::*;
use serialport::SerialPort;
use std::convert::TryInto;
use std::thread;
use std::time::Duration;
use ublox_device::ublox::mon_ver::MonVer;
use ublox_device::ublox::*;
use ublox_device::ublox::{
    cfg_msg::CfgMsgAllPortsBuilder,
    cfg_prt::{
        CfgPrtUartBuilder, DataBits, InProtoMask, OutProtoMask, Parity, StopBits, UartMode,
        UartPortId,
    },
    esf_raw::EsfRaw,
};

/// Use proto23 if enabled, otherwise use proto27 if enabled, otherwise use proto31, otherwise use proto33
#[cfg(feature = "ubx_proto23")]
pub(crate) type Proto = ublox_device::ublox::proto23::Proto23;
#[cfg(all(feature = "ubx_proto27", not(feature = "ubx_proto23")))]
pub(crate) type Proto = ublox_device::ublox::proto27::Proto27;
#[cfg(all(
    feature = "ubx_proto31",
    not(any(feature = "ubx_proto23", feature = "ubx_proto27"))
))]
pub(crate) type Proto = ublox_device::ublox::proto31::Proto31;
#[cfg(all(
    feature = "ubx_proto33",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31",
    ))
))]
pub(crate) type Proto = ublox_device::ublox::proto33::Proto33;

fn main() {
    let mut cli = ublox_device::cli::CommandBuilder::default().build();
    cli = cli
        .about(clap::crate_description!())
        .name(clap::crate_name!())
        .author(clap::crate_authors!());

    let serialport = ublox_device::cli::Command::serialport(cli.clone())
        .expect("Could not connect to serialport");
    // Clone the port for the sending side
    let serialport_clone = serialport.try_clone().expect("Failed to clone serialport");

    let mut device: ublox_device::Device<Proto> = ublox_device::Device::new(serialport);

    let baud_rate = ublox_device::cli::Command::arg_boud(cli);
    sending_thread(baud_rate, serialport_clone);

    // Start reading data
    println!("Opened uBlox device, waiting for messages...");
    loop {
        device
            .on_data_available(|packet| match packet {
                #[cfg(feature = "ubx_proto14")]
                UbxPacket::Proto14(_) => unreachable!("no ubx_proto14 feature"),
                #[cfg(feature = "ubx_proto23")]
                UbxPacket::Proto23(packet_ref) => {
                    use ublox_device::ublox::packetref_proto23::PacketRef;
                    match &packet_ref {
                        PacketRef::MonVer(packet) => {
                            println!(
                                "SW version: {} HW version: {}; Extensions: {:?}",
                                packet.software_version(),
                                packet.hardware_version(),
                                packet.extension().collect::<Vec<&str>>()
                            );
                        },
                        PacketRef::NavPvt(pvt) => {
                            let has_time = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                                || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                            let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                            if has_posvel {
                                let pos: Position = pvt.into();
                                let vel: Velocity = pvt.into();
                                println!(
                                    "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                                    pos.lat, pos.lon, pos.alt
                                );
                                println!(
                                    "Speed: {:.2} m/s Heading: {:.2} degrees",
                                    vel.speed, vel.heading
                                );
                                println!("Sol: {pvt:?}");
                            }

                            if has_time {
                                let time: DateTime<Utc> = pvt
                                    .try_into()
                                    .expect("Could not parse NAV-PVT time field to UTC");
                                println!("Time: {time:?}");
                            }
                        },
                        PacketRef::EsfRaw(raw) => {
                            println!("Got raw message: {raw:?}");
                        },
                        _ => {
                            println!("{packet_ref:?}");
                        },
                    }
                },
                #[cfg(feature = "ubx_proto27")]
                UbxPacket::Proto27(packet_ref) => {
                    use ublox_device::ublox::packetref_proto27::PacketRef;
                    match &packet_ref {
                        PacketRef::MonVer(packet) => {
                            println!(
                                "SW version: {} HW version: {}; Extensions: {:?}",
                                packet.software_version(),
                                packet.hardware_version(),
                                packet.extension().collect::<Vec<&str>>()
                            );
                        },
                        PacketRef::NavPvt(pvt) => {
                            let has_time = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                                || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                            let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                            if has_posvel {
                                let pos: Position = pvt.into();
                                let vel: Velocity = pvt.into();
                                println!(
                                    "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                                    pos.lat, pos.lon, pos.alt
                                );
                                println!(
                                    "Speed: {:.2} m/s Heading: {:.2} degrees",
                                    vel.speed, vel.heading
                                );
                                println!("Sol: {pvt:?}");
                            }

                            if has_time {
                                let time: DateTime<Utc> = pvt
                                    .try_into()
                                    .expect("Could not parse NAV-PVT time field to UTC");
                                println!("Time: {time:?}");
                            }
                        },
                        PacketRef::EsfRaw(raw) => {
                            println!("Got raw message: {raw:?}");
                        },
                        _ => {
                            println!("{packet_ref:?}");
                        },
                    }
                },
                #[cfg(feature = "ubx_proto31")]
                UbxPacket::Proto31(packet_ref) => {
                    use ublox_device::ublox::packetref_proto31::PacketRef;
                    match &packet_ref {
                        PacketRef::MonVer(packet) => {
                            println!(
                                "SW version: {} HW version: {}; Extensions: {:?}",
                                packet.software_version(),
                                packet.hardware_version(),
                                packet.extension().collect::<Vec<&str>>()
                            );
                        },
                        PacketRef::NavPvt(pvt) => {
                            let has_time = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                                || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                            let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                            if has_posvel {
                                let pos: Position = pvt.into();
                                let vel: Velocity = pvt.into();
                                println!(
                                    "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                                    pos.lat, pos.lon, pos.alt
                                );
                                println!(
                                    "Speed: {:.2} m/s Heading: {:.2} degrees",
                                    vel.speed, vel.heading
                                );
                                println!("Sol: {pvt:?}");
                            }

                            if has_time {
                                let time: DateTime<Utc> = pvt
                                    .try_into()
                                    .expect("Could not parse NAV-PVT time field to UTC");
                                println!("Time: {time:?}");
                            }
                        },
                        PacketRef::EsfRaw(raw) => {
                            println!("Got raw message: {raw:?}");
                        },
                        _ => {
                            println!("{packet_ref:?}");
                        },
                    }
                },
                #[cfg(feature = "ubx_proto33")]
                UbxPacket::Proto33(packet_ref) => {
                    use ublox_device::ublox::packetref_proto33::PacketRef;
                    match &packet_ref {
                        PacketRef::MonVer(packet) => {
                            println!(
                                "SW version: {} HW version: {}; Extensions: {:?}",
                                packet.software_version(),
                                packet.hardware_version(),
                                packet.extension().collect::<Vec<&str>>()
                            );
                        },
                        PacketRef::NavPvt(pvt) => {
                            let has_time = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning
                                || pvt.fix_type() == GnssFixType::TimeOnlyFix;
                            let has_posvel = pvt.fix_type() == GnssFixType::Fix3D
                                || pvt.fix_type() == GnssFixType::GPSPlusDeadReckoning;

                            if has_posvel {
                                let pos: Position = pvt.into();
                                let vel: Velocity = pvt.into();
                                println!(
                                    "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                                    pos.lat, pos.lon, pos.alt
                                );
                                println!(
                                    "Speed: {:.2} m/s Heading: {:.2} degrees",
                                    vel.speed, vel.heading
                                );
                                println!("Sol: {pvt:?}");
                            }

                            if has_time {
                                let time: DateTime<Utc> = pvt
                                    .try_into()
                                    .expect("Could not parse NAV-PVT time field to UTC");
                                println!("Time: {time:?}");
                            }
                        },
                        PacketRef::EsfRaw(raw) => {
                            println!("Got raw message: {raw:?}");
                        },
                        _ => {
                            println!("{packet_ref:?}");
                        },
                    }
                },
            })
            .expect("Failed to consume buffer");
    }
}

fn sending_thread(baud_rate: u32, serialport: Box<dyn SerialPort>) {
    let mut device: ublox_device::Device<Proto> = ublox_device::Device::new(serialport);
    // Send out 4 bytes every second
    thread::spawn(move || {
        println!("Configuration thread: configuring UART1 port ...");
        // - configure the device UART1 to talk UBX with baud rate from CLI input
        device
            .write_all(
                &CfgPrtUartBuilder {
                    portid: UartPortId::Uart1,
                    reserved0: 0,
                    tx_ready: 0,
                    mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
                    baud_rate,
                    in_proto_mask: InProtoMask::UBLOX,
                    out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
                    flags: 0,
                    reserved5: 0,
                }
                .into_packet_bytes(),
            )
            .expect("Could not configure UBX-CFG-PRT-UART");

        println!("Enable UBX-ESF-RAW message on selected ports ...");
        device
            .write_all(
                &CfgMsgAllPortsBuilder::set_rate_for::<EsfRaw>([0, 0, 0, 1, 0, 0])
                    .into_packet_bytes(),
            )
            .expect("Could not configure ports for UBX-ESF-RAW");

        loop {
            println!(
                "Configuration thread: send request for UBX-ESF-RAW and UBX-MON-VER message  ..."
            );
            device
                .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
                .expect("Failed to send poll/request for UBX-MON-VER message");
            device
                .write_all(&UbxPacketRequest::request_for::<EsfRaw>().into_packet_bytes())
                .expect("Failed to send poll/request for UBX-ESF-RAW message");
            thread::sleep(Duration::from_millis(1000));
        }
    });
}
