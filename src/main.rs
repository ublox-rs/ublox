use chrono::prelude::*;
use std::time::{Duration, Instant};
use ublox::{Device, Position};

fn characterize_reset(dev: &mut Device, reset: &ublox::ResetType, use_pos_time: bool) -> Duration {
    let pos = Position {
        lon: -97.5,
        lat: 30.2,
        alt: 200.0,
    };

    match dev.reset(reset) {
        Err(e) => {
            println!("Got error resetting: {:?}", e);
        }
        _ => {}
    }

    if use_pos_time {
        println!("Setting AID data...");
        match dev.load_aid_data(Some(pos), Some(Utc::now())) {
            Err(e) => {
                println!("Got error loading AID data: {:?}", e);
            }
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
            Some(_pos) => {
                break;
            }
            None => {}
        }
    }
    start_tm.elapsed()
}

fn main() {
    let mut dev = Device::new().unwrap();
    let pos = Position {
        lon: -97.5,
        lat: 30.2,
        alt: 200.0,
    };
    println!("Setting AID data...");
    match dev.load_aid_data(Some(pos), Some(Utc::now())) {
        Err(e) => {
            println!("Got error loading AID data: {:?}", e);
        }
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
}
