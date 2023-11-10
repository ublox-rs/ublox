use chrono::prelude::*;
use clap::{value_parser, Arg, Command};
use serialport::{
    DataBits as SerialDataBits, FlowControl as SerialFlowControl, Parity as SerialParity,
    StopBits as SerialStopBits,
};
use std::convert::TryInto;
use std::time::Duration;
use ublox::*;

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
                .default_value("9600")
                .value_parser(value_parser!(u32))
                .help("Baud rate of the port to open"),
        )
        .arg(
            Arg::new("stop-bits")
                .long("stop-bits")
                .help("Number of stop bits to use for open port")
                .required(false)
                .value_parser(["1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("data-bits")
                .long("data-bits")
                .help("Number of data bits to use for open port")
                .required(false)
                .value_parser(["7", "8"])
                .default_value("8"),
        )
        .arg(
            Arg::new("parity")
                .long("parity")
                .help("Parity to use for open port")
                .required(false)
                .value_parser(["even", "odd"]),
        )
        .subcommand(
            Command::new("configure")
                .about("Configure settings for specific UART/USB port")
                .arg(
                    Arg::new("port")
                        .long("select")
                        .required(true)
                        .default_value("usb")
                        .value_parser(value_parser!(String))
                        .long_help(
                            "Apply specific configuration to the selected port. Supported: usb, uart1, uart2.
Configuration includes: protocol in/out, data-bits, stop-bits, parity, baud-rate",
                        ),
                    )
                .arg(
                    Arg::new("cfg-baud")
                        .value_name("baud")
                        .long("baud")
                        .required(false)
                        .default_value("9600")
                        .value_parser(value_parser!(u32))
                        .help("Baud rate to set"),
                )
                .arg(
                    Arg::new("stop-bits")
                        .long("stop-bits")
                        .help("Number of stop bits to set")
                        .required(false)
                        .value_parser(["1", "2"])
                        .default_value("1"),
                )
                .arg(
                    Arg::new("data-bits")
                        .long("data-bits")
                        .help("Number of data bits to set")
                        .required(false)
                        .value_parser(["7", "8"])
                        .default_value("8"),
                )
                .arg(
                    Arg::new("parity")
                        .long("parity")
                        .help("Parity to set")
                        .required(false)
                        .value_parser(["even", "odd"]),
                )
                .arg(
                    Arg::new("in-ublox")
                        .long("in-ublox")
                        .default_value("true")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving UBX proprietary protocol on port"),
                )
                .arg(
                    Arg::new("in-nmea")
                        .long("in-nmea")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving NMEA protocol on port"),
                )
                .arg(
                    Arg::new("in-rtcm")
                        .long("in-rtcm")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving RTCM protocol on port"),
                )
                .arg(
                    Arg::new("in-rtcm3")
                        .long("in-rtcm3")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help(
                            "Toggle receiving RTCM3 protocol on port.
        Not supported on uBlox protocol versions below 20",
                        ),
                )
                .arg(
                    Arg::new("out-ublox")
                        .long("out-ublox")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle sending UBX proprietary protocol on port"),
                )
                .arg(
                    Arg::new("out-nmea")
                        .long("out-nmea")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle sending NMEA protocol on port"),
                )
                .arg(
                    Arg::new("out-rtcm3")
                        .long("out-rtcm3")
                        .action(clap::ArgAction::SetTrue)
                        .help(
                            "Toggle seding RTCM3 protocol on port.
        Not supported on uBlox protocol versions below 20",
                        ),
                ),
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
        Some("7") => SerialDataBits::Seven,
        Some("8") => SerialDataBits::Eight,
        _ => {
            println!("Number of DataBits supported by uBlox is either 7 or 8");
            std::process::exit(1);
        },
    };

    let parity = match matches.get_one::<String>("parity").map(|s| s.as_str()) {
        Some("odd") => SerialParity::Even,
        Some("even") => SerialParity::Odd,
        _ => SerialParity::None,
    };

    let builder = serialport::new(port, baud)
        .stop_bits(stop_bits)
        .data_bits(data_bits)
        .timeout(Duration::from_millis(10))
        .parity(parity)
        .flow_control(SerialFlowControl::None);

    println!("{:?}", &builder);
    let port = builder.open().unwrap_or_else(|e| {
        eprintln!("Failed to open \"{}\". Error: {}", port, e);
        ::std::process::exit(1);
    });

    let mut device = Device::new(port);

    // Parse cli for configuring specific uBlox UART port
    if let Some(("configure", sub_matches)) = matches.subcommand() {
        let (port_id, port_name) = match sub_matches.get_one::<String>("port").map(|s| s.as_str()) {
            Some(x) if x == "usb" => (Some(UartPortId::Usb), x),
            Some(x) if x == "uart1" => (Some(UartPortId::Uart1), x),
            Some(x) if x == "uart2" => (Some(UartPortId::Uart2), x),
            _ => (None, ""),
        };

        let baud = sub_matches.get_one::<u32>("baud").cloned().unwrap_or(9600);

        let stop_bits = match sub_matches
            .get_one::<String>("stop-bits")
            .map(|s| s.as_str())
        {
            Some("2") => SerialStopBits::Two,
            _ => SerialStopBits::One,
        };

        let data_bits = match sub_matches
            .get_one::<String>("data-bits")
            .map(|s| s.as_str())
        {
            Some("7") => SerialDataBits::Seven,
            Some("8") => SerialDataBits::Eight,
            _ => {
                println!("Number of DataBits supported by uBlox is either 7 or 8");
                std::process::exit(1);
            },
        };

        let parity = match sub_matches.get_one::<String>("parity").map(|s| s.as_str()) {
            Some("odd") => SerialParity::Even,
            Some("even") => SerialParity::Odd,
            _ => SerialParity::None,
        };
        let inproto = match (
            sub_matches.get_flag("in-ublox"),
            sub_matches.get_flag("in-nmea"),
            sub_matches.get_flag("in-rtcm"),
            sub_matches.get_flag("in-rtcm3"),
        ) {
            (true, false, false, false) => InProtoMask::UBLOX,
            (false, true, false, false) => InProtoMask::NMEA,
            (false, false, true, false) => InProtoMask::RTCM,
            (false, false, false, true) => InProtoMask::RTCM3,
            (true, true, false, false) => InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA),
            (true, false, true, false) => InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM),
            (true, false, false, true) => {
                InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM3)
            },
            (false, true, true, false) => InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM),
            (false, true, false, true) => InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM3),
            (true, true, true, false) => InProtoMask::union(
                InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA),
                InProtoMask::RTCM,
            ),
            (true, true, false, true) => InProtoMask::union(
                InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA),
                InProtoMask::RTCM3,
            ),
            (_, _, true, true) => {
                eprintln!("Cannot use RTCM and RTCM3 simultaneously. Choose one or the other");
                std::process::exit(1)
            },
            (false, false, false, false) => InProtoMask::UBLOX,
        };

        let outproto = match (
            sub_matches.get_flag("out-ublox"),
            sub_matches.get_flag("out-nmea"),
            sub_matches.get_flag("out-rtcm3"),
        ) {
            (true, false, false) => OutProtoMask::UBLOX,
            (false, true, false) => OutProtoMask::NMEA,
            (false, false, true) => OutProtoMask::RTCM3,
            (true, true, false) => OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
            (true, false, true) => OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::RTCM3),
            (false, true, true) => OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::RTCM3),
            (true, true, true) => OutProtoMask::union(
                OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
                OutProtoMask::RTCM3,
            ),
            (false, false, false) => OutProtoMask::UBLOX,
        };

        if let Some(port_id) = port_id {
            println!("Configuring '{}' port ...", port_name.to_uppercase());
            device
                .write_all(
                    &CfgPrtUartBuilder {
                        portid: port_id,
                        reserved0: 0,
                        tx_ready: 0,
                        mode: UartMode::new(
                            ublox_databits(data_bits),
                            ublox_parity(parity),
                            ublox_stopbits(stop_bits),
                        ),
                        baud_rate: baud,
                        in_proto_mask: inproto,
                        out_proto_mask: outproto,
                        flags: 0,
                        reserved5: 0,
                    }
                    .into_packet_bytes(),
                )
                .expect("Could not configure UBX-CFG-PRT-UART");
            device
                .wait_for_ack::<CfgPrtUart>()
                .expect("Could not acknowledge UBX-CFG-PRT-UART msg");
        }
    }

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
                PacketRef::NavPvt(sol) => {
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

fn ublox_stopbits(s: SerialStopBits) -> StopBits {
    // Seriaport crate doesn't support the other StopBits option of uBlox
    match s {
        SerialStopBits::One => StopBits::One,
        SerialStopBits::Two => StopBits::Two,
    }
}

fn ublox_databits(d: SerialDataBits) -> DataBits {
    match d {
        SerialDataBits::Seven => DataBits::Seven,
        SerialDataBits::Eight => DataBits::Eight,
        _ => {
            println!("uBlox only supports Seven or Eight data bits");
            DataBits::Eight
        },
    }
}

fn ublox_parity(v: SerialParity) -> Parity {
    match v {
        SerialParity::Even => Parity::Even,
        SerialParity::Odd => Parity::Odd,
        SerialParity::None => Parity::None,
    }
}

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
            const MAX_PAYLOAD_LEN: usize = 1240;
            let mut local_buf = [0; MAX_PAYLOAD_LEN];
            let nbytes = self.read_port(&mut local_buf)?;
            if nbytes == 0 {
                break;
            }

            // parser.consume adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it = self.parser.consume(&local_buf[..nbytes]);
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
