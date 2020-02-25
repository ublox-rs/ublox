extern crate proc_macro;

macro_rules! debug {
    ($($arg:tt)+) => (println!($($arg)+);)
}

macro_rules! trace {
    ($($arg:tt)+) => {};
}

mod input;
mod output;
mod types;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UbxPacketRecv, attributes(ubx))]
pub fn ubx_packet_recv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    debug!("ubx_packet_recv: input.name {}", input.ident);
    let pack_descr = match input::parse_packet_description(input) {
        Ok(x) => x,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    TokenStream::from(output::generate_recv_code_for_packet(&pack_descr))
}

#[proc_macro_derive(UbxPacketSend, attributes(ubx))]
pub fn ubx_packet_send(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    debug!("ubx_packet_send: input.name {}", input.ident);
    let pack_descr = match input::parse_packet_description(input) {
        Ok(x) => x,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    TokenStream::from(output::generate_send_code_for_packet(&pack_descr))
}

#[proc_macro_derive(UbxDefineSubTypes, attributes(ubx))]
pub fn ubx_packet_subtypes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    debug!("ubx_packet_subtypes: input.name {}", input.ident);
    let pack_descr = match input::parse_packet_description(input) {
        Ok(x) => x,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    TokenStream::from(output::generate_types_for_packet(&pack_descr))
}
