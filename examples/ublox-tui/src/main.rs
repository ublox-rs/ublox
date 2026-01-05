use anyhow::Result;
use std::{error::Error, sync::mpsc::channel};

mod app;
mod backend;
mod cli;
mod logging;
mod signal;
mod tui;
mod ui;

/// Use proto23 if enabled, otherwise use proto27 if enabled, otherwise use proto31, otherwise use proto33
#[cfg(feature = "ubx_proto23")]
pub(crate) type Proto = ublox_device::ublox::proto23::Proto23;
#[cfg(all(feature = "ubx_proto27", not(feature = "ubx_proto23")))]
pub(crate) type Proto = ublox_device::ublox::proto27::Proto27;
#[cfg(all(
    feature = "ubx_proto31",
    not(any(feature = "ubx_proto23", feature = "ubx_proto27"))
))]
pub(crate) type Proto = ublox_device::ublox::proto31::Proto31;
#[cfg(all(
    feature = "ubx_proto33",
    not(any(
        feature = "ubx_proto23",
        feature = "ubx_proto27",
        feature = "ubx_proto31",
    ))
))]
pub(crate) type Proto = ublox_device::ublox::proto33::Proto33;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::parse_args();

    if cli::tui_debug_mode(&cli) {
        device_debug_mode(&cli)?;
    } else {
        let log_file = logging::initialize(&cli)?;
        crate::tui::run(&cli, log_file)?;
    }
    Ok(())
}

fn device_debug_mode(cli: &clap::Command) -> Result<()> {
    use log::error;
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_env("TUI_LOGLEVEL")
        .init();

    let (ubx_msg_tx, ubx_msg_rs) = channel();

    let serialport = ublox_device::cli::Command::serialport(cli.clone())?;
    let device: ublox_device::Device<Proto> = ublox_device::Device::new(serialport);

    let mut backend_device = backend::UbxDevice::from(device);
    backend_device.configure();
    backend_device.run(ubx_msg_tx);

    loop {
        match ubx_msg_rs.recv() {
            Ok(_) => {
                // We don't do anything with the received messages as data as this is intended for the TUI Widgets;
            },
            Err(e) => error!("Error: {e}"),
        }
    }
}
