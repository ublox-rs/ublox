use std::convert;
use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidChecksum,
    UnexpectedPacket,
    TimedOutWaitingForAck(u8, u8),
    IoError(io::Error),
    BincodeError(bincode::Error),
}

impl convert::From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl convert::From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Error::BincodeError(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
