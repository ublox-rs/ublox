//use serde::{Serialize};
//use ublox::UbxSerialize;
use ublox::{Device, Position};
use std::time::{Duration,Instant};
use chrono::prelude::*;
use std::fs::File;
use std::io::Read;

//#[derive(Serialize)]
struct AckAck {
    classid: u8,
    msgid: u8,
}

fn characterize_reset(dev: &mut Device, reset: &ublox::ResetType, use_pos_time: bool) -> Duration {
    /*let mut pos = None;
    while match pos { Some(_) => false, _ => true } {
        dev.poll_for(Duration::from_millis(100));
        pos = dev.get_position();
    }
    let pos = pos.unwrap();
    println!("Got position: {:?}", pos);*/
    let pos = Position{lon: -97.5, lat: 30.2, alt: 200.0};

    match dev.reset(reset) {
        Err(e) => {
            println!("Got error resetting: {:?}", e);
        },
        _ => {}
    }

    if use_pos_time {
        println!("Setting AID data...");
        match dev.load_aid_data(Some(pos), Some(Utc::now())) {
            Err(e) => {
                println!("Got error loading AID data: {:?}", e);
            },
            _ => {}
        }
    }

    /*println!("Setting ALP offline data...");
    let mut file = File::open("current_14d.alp").unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    match dev.set_alp_offline(&data) {
        Err(e) => {
            println!("Error setting ALP offline data: {:?}", e);
        },
        _ => {}
    }*/

    println!("Polling for fix acquisition...");
    let start_tm = Instant::now();
    loop {
        dev.poll_for(Duration::from_millis(100)).unwrap();
        match dev.get_position() {
            Some(pos) => {
                //println!("{:?}", pos);
                break;
            },
            None => {
            }
        }
    }
    start_tm.elapsed()
}

fn main() {
    println!("Hello, world!");
    let mut dev = Device::new().unwrap();
    //let now = Instant::now();
    //println!("{:?}", dev.get_position());
    //println!("{:?} elapsed getting position", now.elapsed());

    //println!("{:?}", characterize_reset(&mut dev, &ublox::ResetType::Cold, true));
    let pos = Position{lon: -97.5, lat: 30.2, alt: 200.0};
    println!("Setting AID data...");
    match dev.load_aid_data(Some(pos), Some(Utc::now())) {
        Err(e) => {
            println!("Got error loading AID data: {:?}", e);
        },
        _ => {}
    }

    loop {
        dev.poll_for(Duration::from_millis(500)).unwrap();
        println!("{:?}", dev.get_solution());
    }

    /*let reset_types = [ublox::ResetType::Hot, ublox::ResetType::Warm, ublox::ResetType::Cold];
    let aids = [true, false];

    for rst in reset_types.iter() {
        for aid in aids.iter() {
            let rst = &(*rst);
            println!("{:?}/{}: {:?}", rst, aid, characterize_reset(&mut dev, rst, *aid));
        }
    }*/

    //println!("warm/aid: {:?}", characterize_reset(&mut dev, ublox::ResetType::Warm, true));

    /*let now = Instant::now();
    while now.elapsed() < Duration::from_millis(2_000) {
        dev.poll();
    }
    let pos = dev.get_position().unwrap();
    println!("{:?}", dev.get_position());

    dev.reset(ublox::ResetType::Warm);

    //dev.load_aid_data(Some(pos), Some(Utc::now()));

    let start_tm = Instant::now();
    loop {
        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(100) {
            dev.poll();
        }
        match dev.get_position() {
            Some(pos) => {
                println!("{:?}", pos);
                break;
            },
            None => {
            }
        }
        //println!("{:?}", dev.get_position());
    }
    println!("Loaded in {:?}", start_tm.elapsed());*/

    /*dev.send(UbxPacket{
        class: 0x01,
        id: 0x02,
        payload: vec![],
    });

    dev.send(UbxPacket{
        class: 0x01,
        id: 0x03,
        payload: vec![],
    });

    let now = Instant::now();
    while now.elapsed() < Duration::from_millis(3_000) {
        dev.update();
    }

    // Do a warm reset
    println!("Resetting...");
    dev.send(UbxPacket{
        class: 0x06,
        id: 0x04,
        payload: vec![0x01, 0x00, 0x01, 0x00],
    });

    let now = Instant::now();
    while now.elapsed() < Duration::from_millis(3_000) {
        dev.update();
    }

    loop {
        println!("Getting new position");
        dev.send(UbxPacket{
            class: 0x01,
            id: 0x02,
            payload: vec![],
        });

        dev.send(UbxPacket{
            class: 0x01,
            id: 0x03,
            payload: vec![],
        });

        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(1_000) {
            dev.update();
        }
    }*/
}
