use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct NotEnoughMem;

impl fmt::Display for NotEnoughMem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Not enough memory error")
    }
}

impl std::error::Error for NotEnoughMem {}

/// Error that possible during packets parsing
#[derive(Debug, PartialEq)]
pub enum ParserError {
    InvalidChecksum,
    InvalidField(&'static str),
    InvalidPacketLen(&'static str),
}

//impl std::error::Error for ParserError {}
