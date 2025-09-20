use clap::Parser;
use std::io::Read;
use ublox::UbxPacket;

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
    let mut parser = ublox::Parser::default();
    let mut buffer = [0u8; 1024];

    loop {
        match port.read(&mut buffer) {
            Ok(bytes_read) => {
                let mut it = parser.consume_ubx(&buffer[..bytes_read]);
                loop {
                    match it.next() {
                        Some(Ok(UbxPacket::Proto23(p))) => {
                            println!("Received UBX packet: {p:?}");
                            match p {
                                ublox::proto23::PacketRef::NavPvt(nav_pvt) => {
                                    println!("Speed: {} [m/s]", nav_pvt.ground_speed_2d())
                                },
                                _ => (), // Ignore packets we don't care about
                            };
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
