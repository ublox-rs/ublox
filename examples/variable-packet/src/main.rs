use ublox_device::ublox::*;

fn main() {
    let mut cli = ublox_device::cli::CommandBuilder::default().build();
    cli = cli
        .about(clap::crate_description!())
        .name(clap::crate_name!())
        .author(clap::crate_authors!());

    let serialport = ublox_device::cli::Command::serialport(cli.clone())
        .expect("Could not connect to serialport");

    let mut device = ublox_device::Device::new(serialport);
    let port_config = ublox_device::cli::Command::ubx_port_configuration_builder(cli);

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
        .expect("Failed to send poll/request for UBX-MON-VER message");

    device
        .write_all(&UbxPacketRequest::request_for::<CfgGnss>().into_packet_bytes())
        .expect("Failed to send poll/request for UBX-CFG-GNSS message");

    let mut buffer = Vec::new();

    CfgGnssBuilder {
        msg_version: 0,
        num_trk_ch_hw: 0,
        num_trk_ch_use: 0,
        num_config_blocks: 2,
        blocks: &[
            GnssConfigBlock {
                gnss_id: GnssId::GPS,
                ..Default::default()
            },
            GnssConfigBlock {
                gnss_id: GnssId::GALILEO,
                ..Default
            },
        ],
    }
    .extend_to(&mut buffer);

    // Send the packet
    device
        .write_all(&buffer)
        .expect("Failed to send CFG-GNSS packet");

    // Start reading data
    println!("Opened uBlox device, waiting for messages...");
    loop {
        device
            .on_data_available(|packet| match packet {
                PacketRef::MonVer(packet) => {
                    println!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        packet.software_version(),
                        packet.hardware_version(),
                        packet.extension().collect::<Vec<&str>>()
                    );
                    println!("{:?}", packet);
                },
                PacketRef::CfgGnss(pkg) => {
                    println!("CfgGnss: {:?}", pkg);
                },
                _ => {
                    // println!("{:?}", packet);
                },
            })
            .unwrap();
    }
}
