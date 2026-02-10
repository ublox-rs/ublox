#![cfg(feature = "alloc")]

use cpu_time::ProcessTime;
use rand::RngExt;
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use ublox::{Parser, ParserError, UbxPacket};

fn read_big_log() -> (Vec<u8>, Meta, Meta, PathBuf, Vec<usize>) {
    let ubx_big_log_path = env::var("UBX_BIG_LOG_PATH").unwrap();
    let ubx_big_log_path = Path::new(&ubx_big_log_path);

    let meta_ext: OsString = if let Some(ext) = ubx_big_log_path.extension() {
        let mut ext: OsString = ext.into();
        ext.push(".meta");
        ext
    } else {
        "meta".into()
    };
    let ubx_big_log_path_meta = ubx_big_log_path.with_extension(meta_ext);
    let meta_data = fs::read_to_string(ubx_big_log_path_meta).unwrap();
    let expect = parse_meta_data(&meta_data).unwrap();

    let biglog = fs::read(ubx_big_log_path).unwrap();
    const MAX_SIZE: usize = 100;
    let mut read_sizes = Vec::with_capacity(biglog.len() / MAX_SIZE / 2);
    let mut rng = rand::rng();
    let mut i = 0;
    while i < biglog.len() {
        let chunk: usize = rng.random_range(1..MAX_SIZE);
        let chunk = (biglog.len() - i).min(chunk);
        read_sizes.push(chunk);
        i += chunk;
    }

    let meta = Meta::default();
    (
        biglog,
        meta,
        expect,
        ubx_big_log_path.to_owned(),
        read_sizes,
    )
}

/// To run test against file with path X,
/// use such command (if use shell compatible with /bin/sh).
/// ```sh
/// UBX_BIG_LOG_PATH=X time cargo test --release test_parse_big_dump -- --ignored --nocapture
/// ```
/// Binary dump should be at path X and at path X.meta, should be file with meta
/// information about what packets you expect find in dump, example:
/// ```sh
/// $ cat /var/tmp/gps.bin.meta
///wrong_chksum=0
///other_errors=0
///nav_pos_llh=38291
///nav_stat=38291
///unknown=120723
///ack_ack=1
/// ```
#[cfg(feature = "ubx_proto14")]
#[test]
#[ignore]
fn test_parse_big_dump_proto14() {
    use ublox::proto14::{PacketRef, Proto14};
    let (log, mut meta, expect, log_path, read_sizes) = read_big_log();
    let mut log_slice = log.as_slice();
    let mut parser = Parser::<_, Proto14>::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log_slice.split_at(*chunk_size);
        log_slice = rest;
        let mut it = parser.consume_ubx(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    UbxPacket::Proto14(PacketRef::AckAck(_)) => meta.ack_ack += 1,
                    UbxPacket::Proto14(PacketRef::NavPosLlh(_)) => meta.nav_pos_llh += 1,
                    UbxPacket::Proto14(PacketRef::NavStatus(_)) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!("parse time of {}: {cpu_time:?}", log_path.display());

    assert_eq!(expect, meta);
}

/// To run test against file with path X,
/// use such command (if use shell compatible with /bin/sh).
/// ```sh
/// UBX_BIG_LOG_PATH=X time cargo test --release test_parse_big_dump -- --ignored --nocapture
/// ```
/// Binary dump should be at path X and at path X.meta, should be file with meta
/// information about what packets you expect find in dump, example:
/// ```sh
/// $ cat /var/tmp/gps.bin.meta
///wrong_chksum=0
///other_errors=0
///nav_pos_llh=38291
///nav_stat=38291
///unknown=120723
///ack_ack=1
/// ```
#[cfg(feature = "ubx_proto23")]
#[test]
#[ignore]
fn test_parse_big_dump_proto23() {
    use ublox::proto23::{PacketRef, Proto23};
    let (log, mut meta, expect, log_path, read_sizes) = read_big_log();
    let mut log_slice = log.as_slice();
    let mut parser = Parser::<_, Proto23>::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log_slice.split_at(*chunk_size);
        log_slice = rest;
        let mut it = parser.consume_ubx(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    UbxPacket::Proto23(PacketRef::AckAck(_)) => meta.ack_ack += 1,
                    UbxPacket::Proto23(PacketRef::NavPosLlh(_)) => meta.nav_pos_llh += 1,
                    UbxPacket::Proto23(PacketRef::NavStatus(_)) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!("parse time of {}: {cpu_time:?}", log_path.display());

    assert_eq!(expect, meta);
}

#[cfg(feature = "ubx_proto27")]
#[test]
#[ignore]
fn test_parse_big_dump_proto27() {
    use ublox::proto27::{PacketRef, Proto27};
    let (log, mut meta, expect, log_path, read_sizes) = read_big_log();
    let mut log_slice = log.as_slice();
    let mut parser = Parser::<_, Proto27>::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log_slice.split_at(*chunk_size);
        log_slice = rest;
        let mut it = parser.consume_ubx(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    UbxPacket::Proto27(PacketRef::AckAck(_)) => meta.ack_ack += 1,
                    UbxPacket::Proto27(PacketRef::NavPosLlh(_)) => meta.nav_pos_llh += 1,
                    UbxPacket::Proto27(PacketRef::NavStatus(_)) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!("parse time of {}: {cpu_time:?}", log_path.display());

    assert_eq!(expect, meta);
}

#[cfg(feature = "ubx_proto31")]
#[test]
#[ignore]
fn test_parse_big_dump_proto31() {
    use ublox::proto31::{PacketRef, Proto31};
    let (log, mut meta, expect, log_path, read_sizes) = read_big_log();
    let mut log_slice = log.as_slice();
    let mut parser = Parser::<_, Proto31>::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log_slice.split_at(*chunk_size);
        log_slice = rest;
        let mut it = parser.consume_ubx(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    UbxPacket::Proto31(PacketRef::AckAck(_)) => meta.ack_ack += 1,
                    UbxPacket::Proto31(PacketRef::NavPosLlh(_)) => meta.nav_pos_llh += 1,
                    UbxPacket::Proto31(PacketRef::NavStatus(_)) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!("parse time of {}: {cpu_time:?}", log_path.display());

    assert_eq!(expect, meta);
}

#[cfg(feature = "ubx_proto33")]
#[test]
#[ignore]
fn test_parse_big_dump_proto33() {
    use ublox::proto33::{PacketRef, Proto33};
    let (log, mut meta, expect, log_path, read_sizes) = read_big_log();
    let mut log_slice = log.as_slice();
    let mut parser = Parser::<_, Proto33>::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log_slice.split_at(*chunk_size);
        log_slice = rest;
        let mut it = parser.consume_ubx(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    UbxPacket::Proto33(PacketRef::AckAck(_)) => meta.ack_ack += 1,
                    UbxPacket::Proto33(PacketRef::NavPosLlh(_)) => meta.nav_pos_llh += 1,
                    UbxPacket::Proto33(PacketRef::NavStatus(_)) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!("parse time of {}: {cpu_time:?}", log_path.display());

    assert_eq!(expect, meta);
}

#[derive(Default, PartialEq, Debug)]
struct Meta {
    wrong_chksum: usize,
    other_errors: usize,
    nav_pos_llh: usize,
    nav_stat: usize,
    ack_ack: usize,
    unknown: usize,
}

fn parse_meta_data(text: &str) -> Result<Meta, String> {
    let mut wrong_chksum = None;
    let mut other_errors = None;
    let mut nav_pos_llh = None;
    let mut nav_stat = None;
    let mut ack_ack = None;
    let mut unknown = None;

    for line in text.lines() {
        let mut it = line.split('=');
        let name = it
            .next()
            .ok_or_else(|| "missed variable name".to_string())?
            .trim();
        let value = it
            .next()
            .ok_or_else(|| "missed variable value".to_string())?
            .trim();
        let value: usize = value
            .parse()
            .map_err(|err| format!("Can not parse integer as usize: {err}"))?;
        match name {
            "wrong_chksum" => wrong_chksum = Some(value),
            "other_errors" => other_errors = Some(value),
            "nav_pos_llh" => nav_pos_llh = Some(value),
            "nav_stat" => nav_stat = Some(value),
            "ack_ack" => ack_ack = Some(value),
            "unknown" => unknown = Some(value),
            _ => return Err(format!("wrong field name: '{name}'")),
        }
    }
    let missed = || "missed field".to_string();
    Ok(Meta {
        wrong_chksum: wrong_chksum.ok_or_else(missed)?,
        other_errors: other_errors.ok_or_else(missed)?,
        nav_pos_llh: nav_pos_llh.ok_or_else(missed)?,
        nav_stat: nav_stat.ok_or_else(missed)?,
        ack_ack: ack_ack.ok_or_else(missed)?,
        unknown: unknown.ok_or_else(missed)?,
    })
}
