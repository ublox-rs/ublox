use anyhow::{Context, Result};
use clap::{value_parser, Arg};
use serialport::{FlowControl as SerialFlowControl, SerialPort};
use std::time::Duration;

pub struct CommandBuilder {
    command: clap::Command,
}

pub struct Command;

#[derive(Debug)]
pub struct UbxPortConfiguration {
    pub port_name: String,
    pub port_id: Option<ublox::UartPortId>,
    pub baud_rate: u32,
    pub stop_bits: ublox::StopBits,
    pub data_bits: ublox::DataBits,
    pub parity: ublox::Parity,
    pub in_proto_mask: ublox::InProtoMask,
    pub out_proto_mask: ublox::OutProtoMask,
}

impl Default for CommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBuilder {
    pub fn new() -> Self {
        let command = clap::Command::new("uBlox CLI device arguments")
        .about("Demonstrates usage of the Rust uBlox API")
        .arg_required_else_help(true)
        .arg(
            Arg::new("port")
                .value_name("port")
                .short('p')
                .long("port")
                .required(true)
                .help("Serial port to open to connect to uBlox device"),
        )
        .arg(
            Arg::new("baud")
                .value_name("baud")
                .short('s')
                .long("baud")
                .required(false)
                .default_value("9600")
                .value_parser(value_parser!(u32))
                .help("Baud rate for the selected port"),
        )
        .arg(
            Arg::new("stop-bits")
                .long("stop-bits")
                .help("Number of stop bits for the selected port")
                .required(false)
                .value_parser(["1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("data-bits")
                .long("data-bits")
                .help("Number of data bits for the selected port")
                .required(false)
                .value_parser(["7", "8"])
                .default_value("8"),
        )
        .arg(
            Arg::new("parity")
                .long("parity")
                .help("Parity to use for selected port")
                .required(false)
                .value_parser(["even", "odd"]),
        )
        .subcommand(
            clap::Command::new("configure")
                .about("Select configuration settings for specific UART/USB port to send to uBlox as a configuration message")
                .arg(
                    Arg::new("port")
                        .long("select")
                        .required(true)
                        .default_value("usb")
                        .value_parser(value_parser!(String))
                        .long_help(
                            "Apply specific configuration to the selected port. Supported: usb, uart1, uart2. Configuration includes: protocol in/out, data-bits, stop-bits, parity, baud-rate",
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
                            "Toggle receiving RTCM3 protocol on port. Not supported on uBlox protocol versions below 20",
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
                            "Toggle sending RTCM3 protocol on port. Not supported on uBlox protocol versions below 20",
                        ),
                ),
        );
        Self { command }
    }

    pub fn build(&self) -> clap::Command {
        self.command.clone()
    }
}

impl Command {
    pub fn arg_boud(command: clap::Command) -> u32 {
        let args = command.get_matches();
        args.get_one::<u32>("baud").cloned().unwrap_or(9600)
    }

    pub fn serialport(command: clap::Command) -> Result<Box<dyn SerialPort>> {
        let cli = command.get_matches();

        let port = cli
            .get_one::<String>("port")
            .expect("Expected required 'port' cli argument");

        let baud = cli.get_one::<u32>("baud").cloned().unwrap_or(9600);
        let stop_bits = match cli.get_one::<String>("stop-bits").map(|s| s.as_str()) {
            Some("2") => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        };
        let data_bits = match cli.get_one::<String>("data-bits").map(|s| s.as_str()) {
            Some("7") => serialport::DataBits::Seven,
            Some("8") => serialport::DataBits::Eight,
            _ => {
                println!("Number of DataBits supported by uBlox is either 7 or 8");
                std::process::exit(1);
            },
        };

        let parity = match cli.get_one::<String>("parity").map(|s| s.as_str()) {
            Some("odd") => serialport::Parity::Even,
            Some("even") => serialport::Parity::Odd,
            _ => serialport::Parity::None,
        };

        let builder = serialport::new(port, baud)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .timeout(Duration::from_millis(10))
            .parity(parity)
            .flow_control(SerialFlowControl::None);

        println!("{:?}", &builder);
        builder
            .open()
            .with_context(|| format!("Failed to open port: {}", port))
    }

    pub fn ubx_port_configuration_builder(command: clap::Command) -> Option<UbxPortConfiguration> {
        use ublox::InProtoMask;
        use ublox::OutProtoMask;

        let cli = command.get_matches();

        // Parse cli for configuring specific uBlox UART port
        if let Some(("configure", sub_matches)) = cli.subcommand() {
            let (port_id, port_name) =
                match sub_matches.get_one::<String>("port").map(|s| s.as_str()) {
                    Some(x) if x == "usb" => (Some(ublox::UartPortId::Usb), x),
                    Some(x) if x == "uart1" => (Some(ublox::UartPortId::Uart1), x),
                    Some(x) if x == "uart2" => (Some(ublox::UartPortId::Uart2), x),
                    _ => (None, ""),
                };

            let baud_rate = sub_matches.get_one::<u32>("baud").cloned().unwrap_or(9600);

            let stop_bits = match sub_matches
                .get_one::<String>("stop-bits")
                .map(|s| s.as_str())
            {
                Some("2") => serialport::StopBits::Two,
                _ => serialport::StopBits::One,
            };

            let data_bits = match sub_matches
                .get_one::<String>("data-bits")
                .map(|s| s.as_str())
            {
                Some("7") => serialport::DataBits::Seven,
                Some("8") => serialport::DataBits::Eight,
                _ => {
                    println!("Number of DataBits supported by uBlox is either 7 or 8");
                    std::process::exit(1);
                },
            };

            let parity = match sub_matches.get_one::<String>("parity").map(|s| s.as_str()) {
                Some("odd") => serialport::Parity::Even,
                Some("even") => serialport::Parity::Odd,
                _ => serialport::Parity::None,
            };
            let in_proto_mask = match (
                sub_matches.get_flag("in-ublox"),
                sub_matches.get_flag("in-nmea"),
                sub_matches.get_flag("in-rtcm"),
                sub_matches.get_flag("in-rtcm3"),
            ) {
                (true, false, false, false) => InProtoMask::UBLOX,
                (false, true, false, false) => InProtoMask::NMEA,
                (false, false, true, false) => InProtoMask::RTCM,
                (false, false, false, true) => InProtoMask::RTCM3,
                (true, true, false, false) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA)
                },
                (true, false, true, false) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM)
                },
                (true, false, false, true) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM3)
                },
                (false, true, true, false) => {
                    InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM)
                },
                (false, true, false, true) => {
                    InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM3)
                },
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

            let out_proto_mask = match (
                sub_matches.get_flag("out-ublox"),
                sub_matches.get_flag("out-nmea"),
                sub_matches.get_flag("out-rtcm3"),
            ) {
                (true, false, false) => OutProtoMask::UBLOX,
                (false, true, false) => OutProtoMask::NMEA,
                (false, false, true) => OutProtoMask::RTCM3,
                (true, true, false) => OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
                (true, false, true) => {
                    OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::RTCM3)
                },
                (false, true, true) => OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::RTCM3),
                (true, true, true) => OutProtoMask::union(
                    OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
                    OutProtoMask::RTCM3,
                ),
                (false, false, false) => OutProtoMask::UBLOX,
            };

            Some(UbxPortConfiguration {
                port_name: port_name.to_string(),
                port_id,
                baud_rate,
                data_bits: ublox_databits(data_bits),
                stop_bits: ublox_stopbits(stop_bits),
                in_proto_mask,
                out_proto_mask,
                parity: ublox_parity(parity),
            })
        } else {
            None
        }
    }
}

fn ublox_stopbits(s: serialport::StopBits) -> ublox::StopBits {
    // Seriaport crate doesn't support the other StopBits option of uBlox
    match s {
        serialport::StopBits::One => ublox::StopBits::One,
        serialport::StopBits::Two => ublox::StopBits::Two,
    }
}

fn ublox_databits(d: serialport::DataBits) -> ublox::DataBits {
    match d {
        serialport::DataBits::Seven => ublox::DataBits::Seven,
        serialport::DataBits::Eight => ublox::DataBits::Eight,
        _ => {
            println!("uBlox only supports Seven or Eight data bits");
            ublox::DataBits::Eight
        },
    }
}

fn ublox_parity(v: serialport::Parity) -> ublox::Parity {
    match v {
        serialport::Parity::Even => ublox::Parity::Even,
        serialport::Parity::Odd => ublox::Parity::Odd,
        serialport::Parity::None => ublox::Parity::None,
    }
}
