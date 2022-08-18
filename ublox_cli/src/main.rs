use chrono::prelude::*;
use clap::{App, Arg};
use std::convert::TryInto;
use std::io;
use std::io::Read;
use std::time::Duration;
use ublox::*;

fn main() {
    let reader: Box<dyn Read> = Box::new(io::stdin().lock());
    let mut parser = Parser::default();
    let data: Vec<u8> = reader.bytes().map(|a| a.unwrap()).collect();
    let mut it = parser.consume(&data);
    while let next = it.next() {
        match next {
            Some(Ok(packet)) => {
                println!("{:?}", packet);
            }
            Some(Err(err)) => {}
            None => {
                break;
            }
        }
    }
}
