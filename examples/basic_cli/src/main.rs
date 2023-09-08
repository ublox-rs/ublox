use chrono::prelude::*;
use clap::{value_parser, Arg, Command};
use serialport::{
    DataBits as SerialDataBits, FlowControl as SerialFlowControl, Parity as SerialParity,
    StopBits as SerialStopBits,
};
use std::convert::TryInto;
use std::time::Duration;
use ublox::*;

struct Device {
    port: Box<dyn serialport::SerialPort>,
    parser: Parser<Vec<u8>>,
}

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device {
        let parser = Parser::default();
        Device { port, parser }
    }

    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.port.write_all(data)
    }

    pub fn update<T: FnMut(PacketRef)>(&mut self, mut cb: T) -> std::io::Result<()> {
        loop {
            let mut local_buf = [0; 100];
            let nbytes = self.read_port(&mut local_buf)?;
            if nbytes == 0 {
                break;
            }

            // parser.consume adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it = self.parser.consume_ubx(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        cb(packet);
                    },
                    Some(Err(_)) => {
                        // Received a malformed packet, ignore it
                    },
                    None => {
                        // We've eaten all the packets we have
                        break;
                    },
                }
            }
        }
        Ok(())
    }

    pub fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
        let mut found_packet = false;
        while !found_packet {
            self.update(|packet| {
                if let PacketRef::AckAck(ack) = packet {
                    if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                        found_packet = true;
                    }
                }
            })?;
        }
        Ok(())
    }

    /// Reads the serial port, converting timeouts into "no data received"
    fn read_port(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        match self.port.read(output) {
            Ok(b) => Ok(b),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(0)
                } else {
                    Err(e)
                }
            },
        }
    }
}

fn main() {
    let matches = Command::new("uBlox CLI example program")
        .author(clap::crate_authors!())
        .about("Demonstrates usage of the Rust uBlox API")
        .arg_required_else_help(true)
        .arg(
            Arg::new("port")
                .value_name("port")
                .short('p')
                .long("port")
                .required(true)
                .help("Serial port to open"),
        )
        .arg(
            Arg::new("baud")
                .value_name("baud")
                .short('s')
                .long("baud")
                .required(false)
                .value_parser(value_parser!(u32))
                .help("Baud rate of the port"),
        )
        .arg(
            Arg::new("stop-bits")
                .long("stop-bits")
                .help("Number of stop bits to use")
                .required(false)
                .value_parser(["1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("data-bits")
                .long("data-bits")
                .help("Number of data bits to use")
                .required(false)
                .value_parser(["5", "6", "7", "8"])
                .default_value("8"),
        )
        .get_matches();

    let port = matches
        .get_one::<String>("port")
        .expect("Expected required 'port' cli argumnet");
    let baud = matches.get_one::<u32>("baud").cloned().unwrap_or(9600);
    let stop_bits = match matches.get_one::<String>("stop-bits").map(|s| s.as_str()) {
        Some("2") => SerialStopBits::Two,
        _ => SerialStopBits::One,
    };
    let data_bits = match matches.get_one::<String>("data-bits").map(|s| s.as_str()) {
        Some("5") => SerialDataBits::Five,
        Some("6") => SerialDataBits::Six,
        Some("7") => SerialDataBits::Seven,
        _ => SerialDataBits::Eight,
    };

    let builder = serialport::new(port, baud)
        .stop_bits(stop_bits)
        .data_bits(data_bits)
        .timeout(Duration::from_millis(1))
        .parity(SerialParity::None)
        .flow_control(SerialFlowControl::None);

    println!("{:?}", &builder);
    let port = builder.open().unwrap_or_else(|e| {
        eprintln!("Failed to open \"{}\". Error: {}", port, e);
        ::std::process::exit(1);
    });
    let mut device = Device::new(port);

    // Configure the device to talk UBX
    println!("Configuring UART1 port ...");
    device
        .write_all(
            &CfgPrtUartBuilder {
                portid: UartPortId::Uart1,
                reserved0: 0,
                tx_ready: 0,
                mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
                baud_rate: baud,
                in_proto_mask: InProtoMask::UBLOX,
                out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
                flags: 0,
                reserved5: 0,
            }
            .into_packet_bytes(),
        )
        .expect("Could not configure UBX-CFG-PRT-UART");
    device
        .wait_for_ack::<CfgPrtUart>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Enable the NavPosVelTime packet
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPosVelTime>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
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
            .update(|packet| match packet {
                PacketRef::MonVer(packet) => {
                    println!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        packet.software_version(),
                        packet.hardware_version(),
                        packet.extension().collect::<Vec<&str>>()
                    );
                    println!("{:?}", packet);
                },
                PacketRef::NavPosVelTime(sol) => {
                    let has_time = sol.fix_type() == GpsFix::Fix3D
                        || sol.fix_type() == GpsFix::GPSPlusDeadReckoning
                        || sol.fix_type() == GpsFix::TimeOnlyFix;
                    let has_posvel = sol.fix_type() == GpsFix::Fix3D
                        || sol.fix_type() == GpsFix::GPSPlusDeadReckoning;

                    if has_posvel {
                        let pos: Position = (&sol).into();
                        let vel: Velocity = (&sol).into();
                        println!(
                            "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                            pos.lat, pos.lon, pos.alt
                        );
                        println!(
                            "Speed: {:.2} m/s Heading: {:.2} degrees",
                            vel.speed, vel.heading
                        );
                        println!("Sol: {:?}", sol);
                    }

                    if has_time {
                        let time: DateTime<Utc> = (&sol)
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
