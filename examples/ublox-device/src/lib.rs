use cli::UbxPortConfiguration;
use std::time::Duration;

pub mod cli;
pub use ublox;
use ublox::{
    cfg_prt::{CfgPrtUart, CfgPrtUartBuilder, UartMode},
    Parser, UbxPacket, UbxPacketMeta, UbxProtocol,
};

pub trait UbxPacketHandler {
    fn handle(&mut self, _packet: UbxPacket) {}
}

/// Implement handler for simple callbacks / closures
impl<F: FnMut(UbxPacket)> UbxPacketHandler for F {
    fn handle(&mut self, package: UbxPacket) {
        self(package)
    }
}

pub struct Device<P: UbxProtocol> {
    port: Box<dyn serialport::SerialPort>,
    parser: Parser<Vec<u8>, P>,
}

impl<P: UbxProtocol> Device<P> {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device<P> {
        let parser = Parser::<_, P>::new(vec![]);
        Device { port, parser }
    }

    pub fn configure_port(
        &mut self,
        port_config: Option<UbxPortConfiguration>,
    ) -> anyhow::Result<()> {
        if let Some(config) = port_config {
            println!("Configuring '{}' port ...", config.port_name.to_uppercase());
            self.write_all(
                &CfgPrtUartBuilder {
                    portid: config.port_id.unwrap(),
                    reserved0: 0,
                    tx_ready: 0,
                    mode: UartMode::new(config.data_bits, config.parity, config.stop_bits),
                    baud_rate: config.baud_rate,
                    in_proto_mask: config.in_proto_mask,
                    out_proto_mask: config.out_proto_mask,
                    flags: 0,
                    reserved5: 0,
                }
                .into_packet_bytes(),
            )?;

            self.wait_for_ack::<CfgPrtUart>()?;
        }
        Ok(())
    }

    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.port.write_all(data)
    }

    pub fn on_data_available<F: FnMut(ublox::UbxPacket)>(
        &mut self,
        mut callback: F,
    ) -> std::io::Result<()> {
        self.process(&mut callback)
    }

    pub fn process(&mut self, handler: &mut impl UbxPacketHandler) -> std::io::Result<()> {
        loop {
            const MAX_PAYLOAD_LEN: usize = 1240;
            let mut local_buf = [0; MAX_PAYLOAD_LEN];
            let nbytes = self.read_port(&mut local_buf)?;
            if nbytes == 0 {
                break;
            }

            // parser.consume_ubx adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it: ublox::UbxParserIter<'_, Vec<u8>, P> =
                self.parser.consume_ubx(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        handler.handle(packet);
                    },
                    Some(Err(e)) => {
                        eprintln!("Malformed packet, ignore it; cause {e}");
                    },
                    None => {
                        // debug!("Parsed all data in buffer ...");
                        break;
                    },
                }
            }
        }
        Ok(())
    }

    pub fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
        let mut found_packet = false;
        let start = std::time::SystemTime::now();
        let timeout = Duration::from_secs(3);
        while !found_packet {
            self.on_data_available(|packet| match packet {
                #[cfg(feature = "ubx_proto23")]
                UbxPacket::Proto23(packet_ref) => {
                    if let ublox::proto23::PacketRef::AckAck(ack) = packet_ref {
                        if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                            found_packet = true;
                        }
                    }
                },
                #[cfg(feature = "ubx_proto27")]
                UbxPacket::Proto27(packet_ref) => {
                    if let ublox::proto27::PacketRef::AckAck(ack) = packet_ref {
                        if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                            found_packet = true;
                        }
                    }
                },
                #[cfg(feature = "ubx_proto31")]
                UbxPacket::Proto31(packet_ref) => {
                    if let ublox::proto31::PacketRef::AckAck(ack) = packet_ref {
                        if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                            found_packet = true;
                        }
                    }
                },
                #[cfg(feature = "ubx_proto14")]
                UbxPacket::Proto17(_) => unreachable!("No ubx_proto14 support"),
            })?;

            if start.elapsed().unwrap().as_millis() > timeout.as_millis() {
                eprintln!("Did not receive ACK message for request");
                break;
            }
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
