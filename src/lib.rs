use std::result::Result;
//use std::io::{ErrorKind};
use std::str;
use std::io;
use std::convert;
use std::time::{Duration,Instant};
use chrono::prelude::*;
use crc::{crc16, Hasher16};
//use super::UbxPackets::UbxPacket;
mod UbxPackets;
use crate::UbxPackets::*;
pub use crate::UbxPackets::{Position, Velocity};

#[derive(Debug)]
pub enum Error {
    UnexpectedPacket,
    TimedOutWaitingForAck(u8, u8),
    IoError(io::Error),
}

impl convert::From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

#[derive(Debug)]
enum InternalPacket {
    NavPosLLH(NavPosLLH),
    NavStatus(NavStatus),
    AckAck(AckAck),
}

/*#[derive(Debug)]
pub struct Position {
    lon: f32,
    lat: f32,
    alt: f32,
}*/

#[derive(Debug)]
pub enum ResetType {
    Hot,
    Warm,
    Cold,
}

pub struct Device {
    port: Box<dyn serialport::SerialPort>,
    buf: Vec<u8>,

    alp_data: Vec<u8>,
    alp_file_id: u16,

    navpos: Option<NavPosLLH>,
    navvel: Option<NavVelNED>,
    navstatus: Option<NavStatus>,
    solution: Option<NavPosVelTime>,
}

impl Device {
    pub fn new(device: &str) -> Result<Device, Error> {
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
            buf: Vec::new(),
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

    fn init_protocol(&mut self) -> Result<(), Error> {
        // Disable NMEA output in favor of the UBX protocol
        self.send(UbxPacket{
            class: 0x06,
            id: 0x00,
            payload: vec![
                0x01, // portID
                0x00, // reserved0
                0x00, 0x00, // txReady
                0xd0, 0x08, 0x00, 0x00, // mode
                0x80, 0x25, 0x00, 0x00, // baudRate
                0x07, 0x00, // inProtoMask
                0x01, 0x00, // outProtoMask
                0x00, 0x00, // flags
                0x00, 0x00, // reserved5
            ],
        })?;

        // Eat the acknowledge and let the device start
        self.wait_for_ack(0x06, 0x00)?;

        self.enable_packet(0x01, 0x07)?; // Nav pos vel time
        //self.enable_packet(0x01, 0x02)?; // Nav pos
        //self.enable_packet(0x01, 0x03)?; // Nav status
        //self.enable_packet(0x01, 0x12)?; // Nav velocity NED

        // Go get mon-ver
        self.send(UbxPacket{
            class: 0x0A,
            id: 0x04,
            payload: vec![],
        })?;
        self.poll_for(Duration::from_millis(200));

        Ok(())
    }

    fn enable_packet(&mut self, classid: u8, msgid: u8) -> Result<(), Error> {
        self.send(UbxPacket{
            class: 0x06,
            id: 0x01,
            payload: vec![
                classid,
                msgid,
                0, 1, 0, 0, 0, 0,
            ],
        })?;
        self.wait_for_ack(0x06, 0x01)?;
        Ok(())
    }

    fn wait_for_ack(&mut self, classid: u8, msgid: u8) -> Result<(), Error> {
        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(1_000) {
            match self.get_next_message()? {
                Some(InternalPacket::AckAck(packet)) => {
                    if packet.classid != classid || packet.msgid != msgid {
                        panic!("Expecting ack, got ack for wrong packet!");
                    }
                    return Ok(());
                },
                Some(_) => {
                    return Err(Error::UnexpectedPacket);
                },
                None => {
                    // Keep waiting
                }
            }
        }
        return Err(Error::TimedOutWaitingForAck(classid, msgid));
    }

    pub fn poll_for(&mut self, duration: Duration) -> Result<(), Error> {
        let start = Instant::now();
        while start.elapsed() < duration {
            self.poll()?;
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), Error> {
        self.get_next_message()?;
        Ok(())
    }

    pub fn get_position(&mut self) -> Option<Position> {
        match (&self.navstatus, &self.navpos) {
            (Some(status), Some(pos)) => {
                if status.itow != pos.itow {
                    None
                } else if status.flags & 0x1 == 0 {
                    None
                } else {
                    Some(Position{
                        lon: pos.lon as f32 / 10_000_000.0,
                        lat: pos.lat as f32 / 10_000_000.0,
                        alt: pos.height_msl as f32 / 1000.0,
                    })
                }
            },
            _ => { None }
        }
    }

    pub fn get_velocity(&mut self) -> Option<Velocity> {
        match (&self.navstatus, &self.navvel) {
            (Some(status), Some(vel)) => {
                if status.itow != vel.itow {
                    None
                } else if status.flags & 0x1 == 0 {
                    None
                } else {
                    Some(Velocity{
                        speed: vel.ground_speed as f32 / 100.0,
                        heading: vel.heading as f32 / 100_000.0,
                    })
                }
            },
            _ => { None }
        }
    }

    pub fn get_solution(&mut self) -> (Option<Position>, Option<Velocity>, Option<DateTime<Utc>>) {
        match &self.solution {
            Some(sol) => {
                let has_time = sol.fix_type == 0x03 || sol.fix_type == 0x04 || sol.fix_type == 0x05;
                let has_posvel = sol.fix_type == 0x03 || sol.fix_type == 0x04;
                let pos = if has_posvel {
                    Some(Position{
                        lon: sol.lon as f32 / 10_000_000.0,
                        lat: sol.lat as f32 / 10_000_000.0,
                        alt: sol.height_msl as f32 / 1000.0,
                    })
                } else {
                    None
                };

                let vel = if has_posvel {
                    Some(Velocity{
                        speed: sol.ground_speed as f32 / 1_000.0,
                        heading: sol.heading as f32 / 100_000.0,
                    })
                } else {
                    None
                };

                let time = if has_time {
                    //println!("{:?}", sol);
                    let ns = if sol.nanosecond < 0 { 0 } else { sol.nanosecond } as u32;
                    Some(Utc.ymd(sol.year as i32, sol.month.into(), sol.day.into()).and_hms_nano(sol.hour.into(), sol.min.into(), sol.sec.into(), ns))
                } else {
                    None
                };
                (pos, vel, time)
            },
            None => { (None, None, None) }
        }
    }

    pub fn reset(&mut self, temperature: &ResetType) -> Result<(), Error> {
        match temperature {
            ResetType::Hot => {
                self.send(UbxPacket{
                    class: 0x06,
                    id: 0x04,
                    payload: vec![0x00, 0x00, 0x01, 0x00],
                })?;
            },
            ResetType::Warm => {
                self.send(UbxPacket{
                    class: 0x06,
                    id: 0x04,
                    payload: vec![0x01, 0x00, 0x01, 0x00],
                })?;
            },
            ResetType::Cold => {
                self.send(UbxPacket{
                    class: 0x06,
                    id: 0x04,
                    payload: vec![0xFF, 0xFF, 0x01, 0x00],
                })?;
            },
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

    pub fn load_aid_data(&mut self, position: Option<Position>, tm: Option<DateTime<Utc>>) -> Result<(), Error> {
        let mut aid = AidIni::new();
        match position {
            Some(pos) => {
                aid.set_position(pos);
            },
            _ => {}
        };
        match tm {
            Some(tm) => {
                aid.set_time(tm);
            },
            _ => {}
        };

        self.send(UbxPacket{
            class: 0x0B,
            id: 0x01,
            payload: bincode::serialize(&aid).unwrap(),
        })?;
        Ok(())
    }

    pub fn set_alp_offline(&mut self, data: &[u8]) -> Result<(), Error> {
        self.alp_data = vec![0; data.len()];
        self.alp_data.copy_from_slice(data);

        let mut digest = crc16::Digest::new(crc16::X25);
        digest.write(&self.alp_data);
        self.alp_file_id = digest.sum16();

        self.send(UbxPacket{
            class: 0x06,
            id: 0x01,
            payload: vec![0x0B, 0x32, 0x01],
        })?;
        self.wait_for_ack(0x06, 0x01)?;
        Ok(())
    }

    fn get_next_message(&mut self) -> Result<Option<InternalPacket>, Error> {
        let packet = self.recv()?;
        match packet {
            Some(packet) => {
                if packet.class == 0x01 && packet.id == 0x02 {
                    let packet: NavPosLLH = bincode::deserialize(&packet.payload).unwrap();
                    //println!("{:?}", packet);
                    self.navpos = Some(packet);
                    return Ok(None);
                } else if packet.class == 0x01 && packet.id == 0x03 {
                    let packet: NavStatus = bincode::deserialize(&packet.payload).unwrap();
                    //println!("{:?}", packet);
                    self.navstatus = Some(packet);
                    return Ok(None);
                } else if packet.class == 0x01 && packet.id == 0x12 {
                    let packet: NavVelNED = bincode::deserialize(&packet.payload).unwrap();
                    self.navvel = Some(packet);
                    return Ok(None);
                } else if packet.class == 0x01 && packet.id == 0x07 {
                    let packet: NavPosVelTime = bincode::deserialize(&packet.payload).unwrap();
                    self.solution = Some(packet);
                    return Ok(None);
                } else if packet.class == 0x05 && packet.id == 0x01 {
                    // This is an acknowledge packet
                    //println!("Acknowledge: {:?}", packet);
                    let packet: AckAck = bincode::deserialize(&packet.payload).unwrap();
                    return Ok(Some(InternalPacket::AckAck(packet)));
                } else if packet.class == 0x0B && packet.id == 0x32 {
                    if self.alp_data.len() == 0 {
                        // Uh-oh... we must be connecting to a device which was already in alp mode, let's just ignore it
                        return Ok(None);
                    }

                    let packet: AlpSrv = bincode::deserialize(&packet.payload).unwrap();
                    let offset = packet.offset as usize * 2;
                    let mut size = packet.size as usize * 2;
                    println!("Got ALP request for contents offset={} size={}", offset, size);

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
                    self.send(UbxPacket{
                        class: 0x0B,
                        id: 0x32,
                        payload: payload,
                    })?;

                    return Ok(None);
                } else if packet.class == 0x0A && packet.id == 0x04 {
                    let swVersion = str::from_utf8(&packet.payload[0..30]).unwrap();
                    let hwVersion = str::from_utf8(&packet.payload[31..40]).unwrap();
                    println!("Got versions: SW={} HW={}", swVersion, hwVersion);
                    return Ok(None);
                } else {
                    println!("Unrecognized packet: {:?}", packet);
                    return Ok(None);
                }
            },
            None => {
                return Ok(None);
            }
        }
    }

    pub fn send(&mut self, packet: UbxPacket) -> Result<(), Error> {
        let serialized = packet.serialize();
        //println!("About to try sending {} bytes", serialized.len());
        self.port.write_all(&serialized)?;
        //println!("{} bytes successfully written, of {}", bytes_written, serialized.len());
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Option<UbxPacket>, Error> {
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
            if bytes_read > 0 {
                self.buf.push(local_buf[0]);

                while self.buf.len() > 0 && self.buf[0] != 0xb5 {
                    self.buf.remove(0);
                }

                while self.buf.len() > 1 && self.buf[1] != 0x62 {
                    self.buf.remove(0);
                }

                // 6 == header + class + length
                if self.buf.len() >= 6 && self.buf[0] == 0xb5 && self.buf[1] == 0x62 {
                    let payload_length = (self.buf[4] as usize) | ((self.buf[5] as usize) << 8);
                    let packet_length = payload_length + 6 + 2;
                    if self.buf.len() >= packet_length {
                        let cka = self.buf[6 + payload_length];
                        let ckb = self.buf[6 + payload_length + 1];

                        let mut payload = self.buf.clone();
                        let mut payload = payload.split_off(6);
                        payload.truncate(payload_length);
                        let packet = UbxPacket{
                            class: self.buf[2],
                            id: self.buf[3],
                            payload: payload,
                        };
                        self.buf = self.buf.split_off(packet_length);
                        if packet.check_checksum(cka, ckb) {
                            return Ok(Some(packet));
                        } else {
                            //panic!("Got bad checksum for otherwise fine packet!");
                            // @TODO: Throw an error
                            println!("Got bad checksum for packet {:?}", packet);
                            return Ok(None);
                        }
                    }
                }
            } else {
                return Ok(None);
            }
        }
    }
}
