use crate::{
    error::ParserError,
    parser::{Parser, ParserIter},
    ubx_packets::*,
};
use chrono::prelude::*;
use crc::{crc16, Hasher16};
use std::io;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum ResetType {
    /// The fastest, clears only the SV data.
    Hot,

    /// Clears the ephemeris.
    Warm,

    /// Clears everything. This takes the longest.
    Cold,
}

#[derive(Debug)]
pub enum Error {
    InvalidChecksum,
    UnexpectedPacket,
    TimedOutWaitingForAck(u8, u8),
    IoError(io::Error),
    ParserError(ParserError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<ParserError> for Error {
    fn from(e: ParserError) -> Self {
        Self::ParserError(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct Device {
    port: Box<dyn serialport::SerialPort>,
    ubx_parser: Parser,
    //buf: Vec<u8>,
    alp_data: Vec<u8>,
    alp_file_id: u16,
    /*
    TODO: should be something like Position, Velocity, DateTime here,
    why data copy?
    navpos: Option<NavPosLlh>,
    navvel: Option<NavVelNed>,
    navstatus: Option<NavStatus>,
    solution: Option<NavPosVelTime>,    */
}

impl Device {
    /// Returns a new u-blox GPS device connected on the given serial port.
    /// Opens the device with a 9600 baud 8 bit serial port, and configures the
    /// device to talk the UBX protocol over port 1.
    ///
    /// Note that port 1 may not be the port you're currently talking over! If
    /// not and you have trouble, please open an issue.
    ///
    /// This function will take approximately 200ms to execute.
    ///
    /// # Errors
    ///
    /// The function can error if there is an issue setting the protocol,
    /// usually if a packet sent is not acknowledged.
    ///
    /// # Panics
    ///
    /// This function will panic if it cannot open the serial port.
    pub fn new(device: &str) -> Result<Device> {
        let s = serialport::SerialPortSettings {
            baud_rate: 9600,
            data_bits: serialport::DataBits::Eight,
            flow_control: serialport::FlowControl::None,
            parity: serialport::Parity::None,
            stop_bits: serialport::StopBits::One,
            timeout: Duration::from_millis(1),
        };
        let port = serialport::open_with_settings(device, &s).unwrap();
        let mut dev = Device {
            port: port,
            ubx_parser: Parser::default(),
            alp_data: Vec::new(),
            alp_file_id: 0,
            /*
            navpos: None,
            navvel: None,
            navstatus: None,
            solution: None,*/
        };

        dev.init_protocol()?;
        Ok(dev)
    }

    fn init_protocol(&mut self) -> Result<()> {
        // Disable NMEA output in favor of the UBX protocol
        self.port.write_all(
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
        )?;

        // Eat the acknowledge and let the device start
        self.wait_for_ack::<CfgPrtUart>()?;
        self.enable_packet::<NavPosVelTime>()?; // Nav pos vel time

        // Go get mon-ver
        self.port
            .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())?;
        self.poll_for(Duration::from_millis(200))?;
        Ok(())
    }

    fn enable_packet<T: UbxPacketMeta>(&mut self) -> Result<()> {
        self.port.write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<T>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )?;
        self.wait_for_ack::<CfgMsgAllPorts>()?;
        Ok(())
    }

    fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> Result<()> {
        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(1_000) {
            let mut it = self.recv()?;
            match it {
                Some(mut it) => {
                    while let Some(pack) = it.next() {
                        match pack {
                            Ok(PacketRef::AckAck(ack_ack)) => {
                                if ack_ack.class() != T::CLASS || ack_ack.msg_id() != T::ID {
                                    eprintln!("Expecting ack, got ack for wrong packet!");
                                    return Err(Error::UnexpectedPacket);
                                }
                                return Ok(());
                            }
                            Ok(_) => return Err(Error::UnexpectedPacket),
                            Err(err) => return Err(err.into()),
                        }
                    }
                }
                None => {
                    // Keep waiting
                }
            }
        }
        return Err(Error::TimedOutWaitingForAck(T::CLASS, T::ID));
    }

    /// Runs the processing loop for the given amount of time. You must run the
    /// processing loop in order to receive updates from the device.
    pub fn poll_for(&mut self, duration: Duration) -> Result<()> {
        let start = Instant::now();
        while start.elapsed() < duration {
            self.poll()?;
        }
        Ok(())
    }

    /// Processes all messages not yet processed. You must periodically call
    /// poll (or poll_for) to process messages in order to receive position
    /// updates.
    pub fn poll(&mut self) -> Result<()> {
        self.get_next_message()?;
        Ok(())
    }
    /*
        /// DO NOT USE. Use get_solution instead.
        pub fn get_position(&mut self) -> Option<Position> {
            match (&self.navstatus, &self.navpos) {
                (Some(status), Some(pos)) => {
                    if status.itow != pos.get_itow() {
                        None
                    } else if status.flags & 0x1 == 0 {
                        None
                    } else {
                        Some(pos.into())
                    }
                }
                _ => None,
            }
        }

        /// DO NOT USE. Use get_solution instead.
        pub fn get_velocity(&mut self) -> Option<Velocity> {
            match (&self.navstatus, &self.navvel) {
                (Some(status), Some(vel)) => {
                    if status.itow != vel.get_itow() {
                        None
                    } else if status.flags & 0x1 == 0 {
                        None
                    } else {
                        Some(vel.into())
                    }
                }
                _ => None,
            }
        }

        /// Returns the most recent solution, as a tuple of position/velocity/time
        /// options. Note that a position may have any combination of these,
        /// including none of them - if no solution has been returned from the
        /// device, all fields will be None.
        pub fn get_solution(&mut self) -> (Option<Position>, Option<Velocity>, Option<DateTime<Utc>>) {
            match &self.solution {
                Some(sol) => {
                    let has_time = sol.fix_type == 0x03 || sol.fix_type == 0x04 || sol.fix_type == 0x05;
                    let has_posvel = sol.fix_type == 0x03 || sol.fix_type == 0x04;
                    let pos = if has_posvel { Some(sol.into()) } else { None };
                    let vel = if has_posvel { Some(sol.into()) } else { None };
                    let time = if has_time { Some(sol.into()) } else { None };
                    (pos, vel, time)
                }
                None => (None, None, None),
            }
        }
    */
    /// Performs a reset of the device, and waits for the device to fully reset.
    pub fn reset(&mut self, temperature: ResetType) -> Result<()> {
        let reset_mask = match temperature {
            ResetType::Hot => NavBbrPredefinedMask::HOT_START,
            ResetType::Warm => NavBbrPredefinedMask::WARM_START,
            ResetType::Cold => NavBbrPredefinedMask::COLD_START,
        };

        self.port.write_all(
            &CfgRstBuilder {
                nav_bbr_mask: reset_mask.into(),
                reset_mode: ResetMode::ControlledSoftwareReset,
                reserved1: 0,
            }
            .into_packet_bytes(),
        )?;

        // Clear our device state
        //        self.navpos = None;
        //        self.navstatus = None;

        // Wait a bit for it to reset
        // (we can't wait for the ack, because we get a bad checksum)
        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(500) {
            //self.poll();
            // Eat any messages
            self.recv()?;
        }

        self.init_protocol()?;
        Ok(())
    }
    /*
       /// If the position and time are known, you can pass them to the GPS device
       /// on startup using this method.
       ///
       /// # Errors
       ///
       /// Will throw an error if there is an error sending the packet.
       ///
       /// # Panics
       ///
       /// Panics if there is an issue serializing the message.
       pub fn load_aid_data(
           &mut self,
           position: Option<Position>,
           tm: Option<DateTime<Utc>>,
       ) -> Result<()> {
           let mut aid = AidIni::new();
           match position {
               Some(pos) => {
                   aid.set_position(pos);
               }
               _ => {}
           };
           match tm {
               Some(tm) => {
                   aid.set_time(tm);
               }
               _ => {}
           };

           self.send(UbxPacket {
               class: 0x0B,
               id: 0x01,
               payload: bincode::serialize(&aid).unwrap(),
           })?;
           Ok(())
       }

       /// DO NOT USE. Experimental!
       pub fn set_alp_offline(&mut self, data: &[u8]) -> Result<()> {
           self.alp_data = vec![0; data.len()];
           self.alp_data.copy_from_slice(data);

           let mut digest = crc16::Digest::new(crc16::X25);
           digest.write(&self.alp_data);
           self.alp_file_id = digest.sum16();

           self.send(UbxPacket {
               class: 0x06,
               id: 0x01,
               payload: vec![0x0B, 0x32, 0x01],
           })?;
           self.wait_for_ack(0x06, 0x01)?;
           Ok(())
       }

    */

    fn get_next_message(&mut self) -> Result<Option<PacketRef>> {
        let mut it = match self.recv()? {
            Some(it) => it,
            None => return Ok(None),
        };
        while let Some(pack) = it.next() {
            let pack = pack?;

            match pack {
                PacketRef::MonVer(packet) => {
                    println!(
                        "Got versions: SW={} HW={}",
                        packet.software_version(),
                        packet.hardware_version()
                    );
                    return Ok(None);
                }
                PacketRef::NavPosVelTime(packet) => {
                    //                    self.solution = Some(packet);
                    return Ok(None);
                }
                PacketRef::NavVelNed(packet) => {
                    //                  self.navvel = Some(packet);
                    return Ok(None);
                }
                PacketRef::NavStatus(packet) => {
                    //                self.navstatus = Some(packet);
                    return Ok(None);
                }
                PacketRef::NavPosLlh(packet) => {
                    //              self.navpos = Some(packet);
                    return Ok(None);
                }
                PacketRef::AlpSrv(packet) => {
                    /*
                    if alp_data.len() == 0 {
                        // Uh-oh... we must be connecting to a device which was already in alp mode, let's just ignore it
                        return Ok(None);
                    }

                    let offset = packet.offset() as usize * 2;
                    let mut size = packet.size() as usize * 2;
                    println!(
                        "Got ALP request for contents offset={} size={}",
                        offset, size
                    );
                    TODO: why we need clone?
                    let mut reply = packet.clone();
                    reply.file_id = self.alp_file_id;

                    if offset > self.alp_data.len() {
                        size = 0;
                    } else if offset + size > self.alp_data.len() {
                        size = self.alp_data.len() - reply.offset as usize;
                    }
                    reply.data_size = size as u16;

                    //println!("Have {} bytes of data, ultimately requesting range {}..{}", self.alp_data.len(), offset, offset+size);

                    TODO: have no idea why `AlpSrv` not used here
                    let contents = &self.alp_data[offset..offset + size];
                    let mut payload = bincode::serialize(&reply).unwrap();
                    for b in contents.iter() {
                        payload.push(*b);
                    }
                    //println!("Payload size: {}", payload.len());
                    self.send(UbxPacket {
                        class: 0x0B,
                        id: 0x32,
                        payload: payload,
                    })?;*/

                    return Ok(None);
                }
                _ => {
                    println!("Received packet");
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    fn recv(&mut self) -> Result<Option<ParserIter>> {
        // Read bytes until we see the header 0xB5 0x62
        loop {
            let mut local_buf = [0; 1];
            let bytes_read = match self.port.read(&mut local_buf) {
                Ok(b) => b,
                Err(e) => {
                    if e.kind() == io::ErrorKind::TimedOut {
                        return Ok(None);
                    } else {
                        return Err(Error::IoError(e));
                    }
                }
            };

            if bytes_read == 0 {
                return Ok(None);
            }

            let it = self.ubx_parser.consume(&local_buf[..bytes_read]);

            return Ok(Some(it));
        }
    }
}
