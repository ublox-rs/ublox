use chrono::prelude::*;
use std::convert::TryInto;
use ublox::*;

fn main() {
    let mut cli = ublox_device::cli::CommandBuilder::default().build();
    cli = cli
        .about("Demonstrate basic usage of uBlox package")
        .name("basic_cli")
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
        .expect("Unable to write request/poll for UBX-MON-VER message");

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
                PacketRef::NavPvt(pvt) => {
                    let has_time = pvt.fix_type() == GpsFix::Fix3D
                        || pvt.fix_type() == GpsFix::GPSPlusDeadReckoning
                        || pvt.fix_type() == GpsFix::TimeOnlyFix;
                    let has_posvel = pvt.fix_type() == GpsFix::Fix3D
                        || pvt.fix_type() == GpsFix::GPSPlusDeadReckoning;

                    if has_posvel {
                        let pos: Position = (&pvt).into();
                        let vel: Velocity = (&pvt).into();
                        println!(
                            "NavPvt: Latitude: {:.5} Longitude: {:.5} Altitude: {:.2} m, Speed: {:.2} m/s Heading: {:.2} degrees",
                            pos.lat, pos.lon, pos.alt
                            ,vel.speed, vel.heading
                        );
                        println!("NavPvt full: {:?}", pvt);
                    }

                    if has_time {
                        let time: DateTime<Utc> = (&pvt)
                            .try_into()
                            .expect("Could not parse NAV-PVT time field to UTC");
                        println!("Time: {:?}", time);
                    }
                },
                _ => {
                    println!("{:?}", packet);
                },
            })
            .unwrap();
    }
}
