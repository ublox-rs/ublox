use std::fmt;

#[derive(Debug)]
pub enum MemWriterError<E>
where
    E: std::error::Error,
{
    NotEnoughMem,
    Custom(E),
}

impl<E> fmt::Display for MemWriterError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemWriterError::NotEnoughMem => f.write_str("Not enough memory error"),
            MemWriterError::Custom(e) => write!(f, "MemWriterError: {}", e),
        }
    }
}

impl<E> std::error::Error for MemWriterError<E> where E: std::error::Error {}

/// Error that possible during packets parsing
#[derive(Debug, PartialEq)]
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
            }
            ParserError::InvalidPacketLen {
                packet,
                expect,
                got,
            } => write!(
                f,
                "Invalid packet({}) length, expect {}, got {}",
                packet, expect, got
            ),
        }
    }
}

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

impl std::error::Error for DateTimeError {}
