use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::LevelFilter;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::cli;

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        std::env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref LOG_ENV: String = "TUI_LOGLEVEL".to_string();
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "ublox", env!("CARGO_PKG_NAME"))
}

pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}

pub fn initialize(cli: &clap::Command) -> Result<PathBuf> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(LOG_ENV.clone()))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );

    let log_file = if cli::tui_log_to_file(cli) {
        let directory = get_data_dir();
        info!("Log to file : {:?}", directory);
        std::fs::create_dir_all(directory.clone())?;
        let log_path = directory.join(LOG_FILE.clone());
        let log_file = std::fs::File::create(log_path)?;

        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_writer(log_file)
            .with_target(false)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(ErrorLayer::default())
            .with(tui_logger::tracing_subscriber_layer())
            .init();
        info!("Full log available in: {}", directory.to_string_lossy());
        directory
    } else {
        tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .with(tui_logger::tracing_subscriber_layer())
            .init();
        PathBuf::new()
    };

    let level = std::env::var("RUST_LOG")
        .unwrap_or("info".to_string())
        .to_ascii_lowercase();
    let level = match level.as_str() {
        "off" => LevelFilter::Off,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "info" => LevelFilter::Info,
        _ => LevelFilter::Info,
    };

    tui_logger::init_logger(level)?;
    tui_logger::set_default_level(level);
    Ok(log_file)
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}
