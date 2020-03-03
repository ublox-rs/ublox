use std::{env, path::Path};

fn main() {
    env_logger::init();

    let out_dir = env::var("OUT_DIR").unwrap();
    let in_src = Path::new("src").join("packets.rs.in");
    let out_src = Path::new(&out_dir).join("packets.rs");

    ublox_derive::expand_ubx_packets_code_in_file(&in_src, &out_src);
    
    println!("cargo:rerun-if-changed={}", in_src.display());
}
