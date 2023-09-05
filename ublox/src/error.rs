use core::fmt;

#[derive(Debug)]
pub enum MemWriterError<E> {
    NotEnoughMem,
    Custom(E),
}

impl<E: core::fmt::Display> fmt::Display for MemWriterError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemWriterError::NotEnoughMem => f.write_str("Not enough memory error"),
            MemWriterError::Custom(e) => write!(f, "MemWriterError: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<E> std::error::Error for MemWriterError<E> where E: std::error::Error {}

/// Error that possible during packets parsing
#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    InvalidChecksum {
        expect: u16,
        got: u16,
    },
    InvalidField {
        packet: &'static str,
        field: &'static str,
    },
    InvalidPacketLen {
        packet: &'static str,
        expect: usize,
        got: usize,
    },
    /// Returned when the parser buffer is not big enough to store the packet
    OutOfMemory {
        required_size: usize,
    },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::InvalidChecksum { expect, got } => write!(
                f,
                "Not valid packet's checksum, expect {:x}, got {:x}",
                expect, got
            ),
            ParserError::InvalidField { packet, field } => {
                write!(f, "Invalid field {} of packet {}", field, packet)
            },
            ParserError::InvalidPacketLen {
                packet,
                expect,
                got,
            } => write!(
                f,
                "Invalid packet({}) length, expect {}, got {}",
                packet, expect, got
            ),
            ParserError::OutOfMemory { required_size } => write!(
                f,
                "Insufficient parser buffer size, required {} bytes",
                required_size
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParserError {}

#[derive(Debug, Clone, Copy)]
pub enum DateTimeError {
    InvalidDate,
    InvalidTime,
    InvalidNanoseconds,
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateTimeError::InvalidDate => f.write_str("invalid date"),
            DateTimeError::InvalidTime => f.write_str("invalid time"),
            DateTimeError::InvalidNanoseconds => f.write_str("invalid nanoseconds"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DateTimeError {}
