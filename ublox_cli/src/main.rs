use chrono::prelude::*;
use clap::{App, Arg};
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
            let mut it = self.parser.consume(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        cb(packet);
                    }
                    Some(Err(_)) => {
                        // Received a malformed packet, ignore it
                    }
                    None => {
                        // We've eaten all the packets we have
                        break;
                    }
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
            }
        }
    }
}

fn main() {
    let matches = App::new("ublox CLI example program")
        .about("Demonstrates usage of the Rust ublox API")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .required(true)
                .help("Serial port to open"),
        )
        .arg(
            Arg::with_name("baud")
                .short("s")
                .long("baud")
                .takes_value(true)
                .help("Baud rate of the port"),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap();
    let baud: u32 = matches
        .value_of("baud")
        .unwrap_or("9600")
        .parse()
        .expect("Could not parse baudrate as an integer");

    let s = serialport::SerialPortSettings {
        baud_rate: baud,
        data_bits: serialport::DataBits::Eight,
        flow_control: serialport::FlowControl::None,
        parity: serialport::Parity::None,
        stop_bits: serialport::StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let port = serialport::open_with_settings(port, &s).unwrap();
    let mut device = Device::new(port);

    // Configure the device to talk UBX
    device
        .write_all(
            &CfgPrtUartBuilder {
                portid: UartPortId::Uart1,
                reserved0: 0,
                tx_ready: 0,
                mode: 0x8d0,
                baud_rate: 9600,
                in_proto_mask: 0x07,
                out_proto_mask: 0x01,
                flags: 0,
                reserved5: 0,
            }
            .into_packet_bytes(),
        )
        .unwrap();
    device.wait_for_ack::<CfgPrtUart>().unwrap();

    // Enable the NavPosVelTime packet
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPosVelTime>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
        )
        .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();

    // Send a packet request for the MonVer packet
    device
        .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
        .unwrap();

    // Start reading data
    println!("Opened u-blox device, waiting for solutions...");
    loop {
        device
            .update(|packet| match packet {
                PacketRef::MonVer(packet) => {
                    println!(
                        "SW version: {} HW version: {}",
                        packet.software_version(),
                        packet.hardware_version()
                    );
                }
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
                    }

                    if has_time {
                        let time: DateTime<Utc> = (&sol).try_into().unwrap();
                        println!("Time: {:?}", time);
                    }
                }
                _ => {}
            })
            .unwrap();
    }
}
