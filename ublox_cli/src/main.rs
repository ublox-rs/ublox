use std::io;
use std::io::Read;
use ublox::*;

fn main() {
    let reader: Box<dyn Read> = Box::new(io::stdin().lock());
    let mut parser = Parser::default();
    let data: Vec<u8> = reader.bytes().map(|a| a.unwrap()).collect();
    let mut it = parser.consume(&data);
    while let Some(next) = it.next() {
        match next {
            Ok(packet) => {
                println!("{packet:?}");
            }
            Err(err) => println!("{err:?}"),
        }
    }
}
