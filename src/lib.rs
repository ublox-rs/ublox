//! # ublox
//!
//! `ublox` is a library to talk to u-blox GPS devices using the UBX protocol.
//! At time of writing this library is developed for a device which behaves like
//! a NEO-6M device.
use chrono::prelude::*;
use crc::{crc16, Hasher16};
use std::io;
use std::time::{Duration, Instant};
use crate::error::{Error, Result};

pub use crate::ubx_packets::*;
pub use crate::segmenter::Segmenter;

mod error;
mod ubx_packets;
mod segmenter;

#[derive(Debug)]
pub enum ResetType {
    /// The fastest, clears only the SV data.
    Hot,

    /// Clears the ephemeris.
    Warm,

    /// Clears everything. This takes the longest.
    Cold,
}

pub struct Device {
    port: Box<dyn serialport::SerialPort>,
    segmenter: Segmenter,
    //buf: Vec<u8>,

    alp_data: Vec<u8>,
    alp_file_id: u16,

    navpos: Option<NavPosLLH>,
    navvel: Option<NavVelNED>,
    navstatus: Option<NavStatus>,
    solution: Option<NavPosVelTime>,
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
        let s = serialport::SerialPortSettings{
            baud_rate: 9600,
            data_bits: serialport::DataBits::Eight,
            flow_control: serialport::FlowControl::None,
            parity: serialport::Parity::None,
            stop_bits: serialport::StopBits::One,
            timeout: Duration::from_millis(1),
        };
        let port = serialport::open_with_settings(device, &s).unwrap();
        let mut dev = Device{
            port: port,
            segmenter: Segmenter::new(),
            alp_data: Vec::new(),
            alp_file_id: 0,
            navpos: None,
            navvel: None,
            navstatus: None,
            solution: None,
        };

        dev.init_protocol()?;
        Ok(dev)
    }

    fn init_protocol(&mut self) -> Result<()> {
        // Disable NMEA output in favor of the UBX protocol
        self.send(
            CfgPrtUart {
                portid: 1,
                reserved0: 0,
                tx_ready: 0,
                mode: 0x8d0,
                baud_rate: 9600,
                in_proto_mask: 0x07,
                out_proto_mask: 0x01,
                flags: 0,
                reserved5: 0,
            }
            .into(),
        )?;

        // Eat the acknowledge and let the device start
        self.wait_for_ack(0x06, 0x00)?;

        self.enable_packet(0x01, 0x07)?; // Nav pos vel time

        // Go get mon-ver
        self.send(UbxPacket {
            class: 0x0A,
            id: 0x04,
            payload: vec![],
        })?;
        self.poll_for(Duration::from_millis(200))?;

        Ok(())
    }

    fn enable_packet(&mut self, classid: u8, msgid: u8) -> Result<()> {
        self.send(
            CfgMsg {
                classid: classid,
                msgid: msgid,
                rates: [0, 1, 0, 0, 0, 0],
            }
            .into(),
        )?;
        self.wait_for_ack(0x06, 0x01)?;
        Ok(())
    }

    fn wait_for_ack(&mut self, classid: u8, msgid: u8) -> Result<()> {
        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(1_000) {
            match self.get_next_message()? {
                Some(Packet::AckAck(packet)) => {
                    if packet.classid != classid || packet.msgid != msgid {
                        panic!("Expecting ack, got ack for wrong packet!");
                    }
                    return Ok(());
                }
                Some(_) => {
                    return Err(Error::UnexpectedPacket);
                }
                None => {
                    // Keep waiting
                }
            }
        }
        return Err(Error::TimedOutWaitingForAck(classid, msgid));
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

    /// Performs a reset of the device, and waits for the device to fully reset.
    pub fn reset(&mut self, temperature: &ResetType) -> Result<()> {
        match temperature {
            ResetType::Hot => {
                self.send(CfgRst::HOT.into())?;
            }
            ResetType::Warm => {
                self.send(CfgRst::WARM.into())?;
            }
            ResetType::Cold => {
                self.send(CfgRst::COLD.into())?;
            }
        }

        // Clear our device state
        self.navpos = None;
        self.navstatus = None;

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
        tm: Option<DateTime<Utc>>
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

    fn get_next_message(&mut self) -> Result<Option<Packet>> {
        let packet = self.recv()?;
        match packet {
            Some(Packet::AckAck(packet)) => {
                return Ok(Some(Packet::AckAck(packet)));
            }
            Some(Packet::MonVer(packet)) => {
                println!("Got versions: SW={} HW={}", packet.sw_version, packet.hw_version);
                return Ok(None);
            }
            Some(Packet::NavPosVelTime(packet)) => {
                self.solution = Some(packet);
                return Ok(None);
            }
            Some(Packet::NavVelNED(packet)) => {
                self.navvel = Some(packet);
                return Ok(None);
            }
            Some(Packet::NavStatus(packet)) => {
                self.navstatus = Some(packet);
                return Ok(None);
            }
            Some(Packet::NavPosLLH(packet)) => {
                self.navpos = Some(packet);
                return Ok(None);
            }
            Some(Packet::AlpSrv(packet)) => {
                if self.alp_data.len() == 0 {
                    // Uh-oh... we must be connecting to a device which was already in alp mode, let's just ignore it
                    return Ok(None);
                }

                let offset = packet.offset as usize * 2;
                let mut size = packet.size as usize * 2;
                println!(
                    "Got ALP request for contents offset={} size={}",
                    offset, size
                );

                let mut reply = packet.clone();
                reply.file_id = self.alp_file_id;

                if offset > self.alp_data.len() {
                    size = 0;
                } else if offset + size > self.alp_data.len() {
                    size = self.alp_data.len() - reply.offset as usize;
                }
                reply.data_size = size as u16;

                //println!("Have {} bytes of data, ultimately requesting range {}..{}", self.alp_data.len(), offset, offset+size);
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
                })?;

                return Ok(None);
            }
            Some(packet) => {
                println!("Received packet {:?}", packet);
                return Ok(None);
            }
            None => {
                // Got nothing, do nothing
                return Ok(None);
            }
        }
    }

    fn send(&mut self, packet: UbxPacket) -> Result<()> {
        CfgMsg{classid: 5, msgid: 4, rates: [0, 0, 0, 0, 0, 0]}.to_bytes();
        let serialized = packet.serialize();
        self.port.write_all(&serialized)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<Packet>> {
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

            match self.segmenter.consume(&local_buf[..bytes_read])? {
                Some(packet) => {
                    return Ok(Some(packet));
                }
                None => {
                    // Do nothing
                }
            }
        }
    }
}
