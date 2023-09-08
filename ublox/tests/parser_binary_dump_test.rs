#![cfg(feature = "alloc")]

use cpu_time::ProcessTime;
use rand::{thread_rng, Rng};
use std::{env, ffi::OsString, fs, path::Path};
use ublox::{PacketRef, Parser, ParserError};

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
#[test]
#[ignore]
fn test_parse_big_dump() {
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
    let mut rng = thread_rng();
    let mut i = 0;
    while i < biglog.len() {
        let chunk: usize = rng.gen_range(1..MAX_SIZE);
        let chunk = (biglog.len() - i).min(chunk);
        read_sizes.push(chunk);
        i += chunk;
    }

    let mut meta = Meta::default();
    let mut log = biglog.as_slice();
    let mut parser = Parser::default();

    let start = ProcessTime::now();
    for chunk_size in &read_sizes {
        let (buf, rest) = log.split_at(*chunk_size);
        log = rest;
        let mut it = parser.consume(buf);
        while let Some(pack) = it.next() {
            match pack {
                Ok(pack) => match pack {
                    PacketRef::AckAck(_) => meta.ack_ack += 1,
                    PacketRef::NavPosLlh(_) => meta.nav_pos_llh += 1,
                    PacketRef::NavStatus(_) => meta.nav_stat += 1,
                    _ => meta.unknown += 1,
                },
                Err(ParserError::InvalidChecksum { .. }) => meta.wrong_chksum += 1,
                Err(_) => meta.other_errors += 1,
            }
        }
    }
    let cpu_time = start.elapsed();
    println!(
        "parse time of {}: {:?}",
        ubx_big_log_path.display(),
        cpu_time
    );

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
            .map_err(|err| format!("Can not parse integer as usize: {}", err))?;
        match name {
            "wrong_chksum" => wrong_chksum = Some(value),
            "other_errors" => other_errors = Some(value),
            "nav_pos_llh" => nav_pos_llh = Some(value),
            "nav_stat" => nav_stat = Some(value),
            "ack_ack" => ack_ack = Some(value),
            "unknown" => unknown = Some(value),
            _ => return Err(format!("wrong field name: '{}'", name)),
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
