use std::{
    error::Error,
    io,
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
    time::{Duration, Instant},
};

use log::error;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use anyhow::Result;
use tracing::{debug, info, instrument};
use ublox_device::ublox::{self, SensorData};

use crate::{
    app::{App, UbxStatus},
    backend, cli, ui,
};

pub fn run(cli: &clap::Command, log_file: PathBuf) -> Result<(), Box<dyn Error>> {
    ratatui::init();
    let tick_rate: u64 = cli::tui_rate(cli);
    let tick_rate = Duration::from_millis(tick_rate);

    // trace_dbg!(level: tracing::Level::INFO,"test");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (ubx_msg_tx, ubx_msg_rs) = channel();

    let serialport = match ublox_device::cli::Command::serialport(cli.clone()) {
        Err(e) => {
            ratatui::restore();
            return Err(e.into());
        },
        Ok(s) => s,
    };
    let device = ublox_device::Device::new(serialport);
    let mut backend_device = backend::UbxDevice::from(device);
    backend_device.configure();
    backend_device.run(ubx_msg_tx);

    let app = App::new("uBlox TUI", log_file);
    let app_result = run_app(&mut terminal, app, tick_rate, ubx_msg_rs);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        error!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    receiver: Receiver<UbxStatus>,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        update_states(&mut app, &receiver);
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                        KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                        KeyCode::Char(c) => app.on_key(c),
                        _ => {},
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
        if app.should_quit {
            info!("Q/q pressed. Exiting application.");
            println!("See the log file logs");
            return Ok(());
        }
    }
}

fn update_states(app: &mut App, receiver: &Receiver<UbxStatus>) {
    match receiver.recv_timeout(std::time::Duration::from_millis(5)) {
        Ok(UbxStatus::Pvt(v)) => {
            app.pvt_state = *v;
        },
        Ok(UbxStatus::MonVer(v)) => {
            app.mon_ver_state = *v;
        },
        Ok(UbxStatus::EsfAlgImu(v)) => {
            app.esf_alg_imu_alignment_state = v;
        },
        Ok(UbxStatus::EsfAlgStatus(v)) => {
            app.esf_alg_state = v;
        },
        Ok(UbxStatus::EsfAlgSensors(v)) => {
            app.esf_sensors_state = v;
        },
        Ok(UbxStatus::EsfMeas(v)) => {
            for meas in v.measurements.iter() {
                let value = match meas.value() {
                    SensorData::Tick(v) => v as f64,
                    SensorData::Value(v) => v as f64,
                };
                let value = (v.time_tag, value);
                match meas.data_type {
                    ublox::EsfSensorType::AccX => app.signals.acc_x.append(value),
                    ublox::EsfSensorType::AccY => app.signals.acc_y.append(value),
                    ublox::EsfSensorType::AccZ => app.signals.acc_z.append(value),
                    ublox::EsfSensorType::GyroX => app.signals.gyro_x.append(value),
                    ublox::EsfSensorType::GyroY => app.signals.gyro_y.append(value),
                    ublox::EsfSensorType::GyroZ => app.signals.gyro_z.append(value),
                    ublox::EsfSensorType::GyroTemp => app.signals.gyro_temp.append(value),
                    ublox::EsfSensorType::FrontLeftWheelTicks => app.signals.wt_fl.append(value),
                    ublox::EsfSensorType::FrontRightWheelTicks => app.signals.wt_fr.append(value),
                    ublox::EsfSensorType::RearLeftWheelTicks => app.signals.wt_rl.append(value),
                    ublox::EsfSensorType::RearRightWheelTicks => app.signals.wt_rr.append(value),
                    ublox::EsfSensorType::Speed => app.signals.speed.append(value),
                    ublox::EsfSensorType::SpeedTick => app.signals.speed_tick.append(value),
                    _ => {
                        unimplemented!("Not implemented for {:?}", meas.data_type);
                    },
                }
                // app.signals.wt_rl_data.push(value);
            }
        },
        _ => {}, // Err(e) => println!("Not value from channel"),
    }
}

/// Handle events and insert them into the events vector keeping only the last 10 events
#[instrument(skip(events))]
fn handle_events(events: &mut Vec<Event>) -> Result<()> {
    // Render the UI at least once every 100ms
    if event::poll(Duration::from_millis(100))? {
        let event = event::read()?;
        debug!(?event);
        events.insert(0, event);
    }
    events.truncate(10);
    Ok(())
}
