#[cfg(feature = "serial")]
mod serial {
    use chrono::prelude::*;
    use std::time::Duration;
    use ublox::{Device, Position};

    pub fn main() {
        let mut dev = Device::new("/dev/ttyUSB0").unwrap();

        let pos = Position {
            lon: -97.5,
            lat: 30.2,
            alt: 200.0,
        };
        println!("Setting AID data...");
        match dev.load_aid_data(Some(pos), Some(Utc::now())) {
            Err(e) => {
                println!("Got error setting AID data: {:?}", e);
            }
            _ => {}
        }

        loop {
            dev.poll_for(Duration::from_millis(500)).unwrap();
            println!("{:?}", dev.get_solution());
        }
    }
}

fn main() {
    #[cfg(feature = "serial")]
    serial::main()
}
