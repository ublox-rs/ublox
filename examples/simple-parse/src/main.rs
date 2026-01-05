use clap::Parser;
use std::io::Read;
use ublox::UbxPacket;

/// Use proto23 if enabled, otherwise use proto27 if enabled, otherwise use proto31, otherwise use proto33, otherwise use proto14
#[cfg(feature = "ubx_proto23")]
type Proto = ublox::proto23::Proto23;
#[cfg(all(feature = "ubx_proto27", not(feature = "ubx_proto23")))]
type Proto = ublox::proto27::Proto27;
#[cfg(all(
    feature = "ubx_proto31",
    not(any(feature = "ubx_proto23", feature = "ubx_proto27"))
))]
type Proto = ublox::proto31::Proto31;
#[cfg(all(
    feature = "ubx_proto33",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31",
    ))
))]
type Proto = ublox::proto33::Proto33;
#[cfg(all(
    feature = "ubx_proto14",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31"
    ))
))]
type Proto = ublox::proto14::Proto14;

#[derive(Parser)]
struct Args {
    /// Serial port device
    #[arg(short, long)]
    port: String,

    /// Baud rate
    #[arg(short, long, default_value_t = 9600)]
    baud_rate: u32,

    /// Stop bits (1 or 2)
    #[arg(short, long, default_value_t = 1)]
    stop_bits: u8,

    /// Data bits (7 or 8)
    #[arg(short, long, default_value_t = 8)]
    data_bits: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let arg_port = args.port;
    let arg_baud_rate = args.baud_rate;

    let arg_stop_bits = match args.stop_bits {
        1 => serialport::StopBits::One,
        2 => serialport::StopBits::Two,
        _ => return Err("Invalid stop bits. Must be 1 or 2".into()),
    };

    let arg_data_bits = match args.data_bits {
        7 => serialport::DataBits::Seven,
        8 => serialport::DataBits::Eight,
        _ => return Err("Invalid data bits. Must be 7 or 8".into()),
    };

    let mut port = serialport::new(&arg_port, arg_baud_rate)
        .stop_bits(arg_stop_bits)
        .data_bits(arg_data_bits)
        .open()?;

    println!(
        "Serial port '{arg_port}' opened successfully at {arg_baud_rate} baud, {arg_data_bits} data bits, {arg_stop_bits} stop bits!",
    );
    let mut parser = ublox::Parser::<Vec<u8>, Proto>::default();
    let mut buffer = [0u8; 1024];

    loop {
        match port.read(&mut buffer) {
            Ok(bytes_read) => {
                let mut it = parser.consume_ubx(&buffer[..bytes_read]);
                loop {
                    match it.next() {
                        #[cfg(feature = "ubx_proto14")]
                        Some(Ok(UbxPacket::Proto14(p))) => {
                            handle_packet_proto14(p);
                        },
                        #[cfg(feature = "ubx_proto23")]
                        Some(Ok(UbxPacket::Proto23(p))) => {
                            handle_packet_proto23(p);
                        },
                        #[cfg(feature = "ubx_proto27")]
                        Some(Ok(UbxPacket::Proto27(p))) => {
                            handle_packet_proto27(p);
                        },
                        #[cfg(feature = "ubx_proto31")]
                        Some(Ok(UbxPacket::Proto31(p))) => {
                            handle_packet_proto31(p);
                        },
                        #[cfg(feature = "ubx_proto33")]
                        Some(Ok(UbxPacket::Proto33(p))) => {
                            handle_packet_proto33(p);
                        },
                        Some(Err(e)) => {
                            println!("Received malformed packet: {e:?}");
                        },
                        None => {
                            // The internal buffer is now empty
                            break;
                        },
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading from serial port: {e}");
                break;
            },
        }
    }

    Ok(())
}

#[cfg(feature = "ubx_proto14")]
fn handle_packet_proto14(p: ublox::proto14::PacketRef) {
    println!("Received UBX packet: {p:?}");
    if let ublox::proto14::PacketRef::NavPvt(nav_pvt) = p {
        println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
    }
}

#[cfg(feature = "ubx_proto23")]
fn handle_packet_proto23(p: ublox::proto23::PacketRef) {
    println!("Received UBX packet: {p:?}");
    match p {
        ublox::proto23::PacketRef::NavPvt(nav_pvt) => {
            println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
        },
        ublox::proto23::PacketRef::EsfMeas(esf_meas) => {
            for data in esf_meas.data() {
                println!("ESF MEAS DATA: {data:?}");
            }
        },
        _ => (),
    }
}

#[cfg(feature = "ubx_proto27")]
fn handle_packet_proto27(p: ublox::proto27::PacketRef) {
    println!("Received UBX packet: {p:?}");
    match p {
        ublox::proto27::PacketRef::NavPvt(nav_pvt) => {
            println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
        },
        ublox::proto27::PacketRef::EsfMeas(esf_meas) => {
            for data in esf_meas.data() {
                println!("ESF MEAS DATA: {data:?}");
            }
        },
        _ => (),
    }
}

#[cfg(feature = "ubx_proto31")]
fn handle_packet_proto31(p: ublox::proto31::PacketRef) {
    println!("Received UBX packet: {p:?}");
    match p {
        ublox::proto31::PacketRef::NavPvt(nav_pvt) => {
            println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
        },
        ublox::proto31::PacketRef::EsfMeas(esf_meas) => {
            for data in esf_meas.data() {
                println!("ESF MEAS DATA: {data:?}");
            }
        },
        _ => (),
    }
}

#[cfg(feature = "ubx_proto33")]
fn handle_packet_proto33(p: ublox::proto33::PacketRef) {
    println!("Received UBX packet: {p:?}");
    match p {
        ublox::proto33::PacketRef::NavPvt(nav_pvt) => {
            println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
        },
        ublox::proto33::PacketRef::EsfMeas(esf_meas) => {
            for data in esf_meas.data() {
                println!("ESF MEAS DATA: {data:?}");
            }
        },
        _ => (),
    }
}
