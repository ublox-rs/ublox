use clap::{value_parser, Arg};

pub fn parse_args() -> clap::Command {
    let cli = ublox_device::cli::CommandBuilder::default().build();
    cli.name("uBlox TUI")
        .author(clap::crate_authors!())
        .about("Simple TUI to show PVT and ESF statuses")
        .arg_required_else_help(true)
        .arg(
            Arg::new("debug-mode")
                .value_name("debug-mode")
                .long("debug-mode")
                .action(clap::ArgAction::SetTrue)
                .help("Bypass TUI altogether and run the u-blox connection only. Useful for debugging issues with u-blox connectivity and message parsing."),
        )
        .arg(
            Arg::new("log-file")
                .value_name("log-file")
                .long("log-file")
                .action(clap::ArgAction::SetTrue)
                .help("Log to file besides showing partial logs in the TUI"),
        )
        .arg(
            Arg::new("tui-rate")
                .value_name("tui-rate")
                .long("tui-rate")
                .required(false)
                .default_value("10")
                .value_parser(value_parser!(u64))
                .help("TUI refresh rate in milliseconds"),
        )
}

pub fn tui_rate(command: &clap::Command) -> u64 {
    *command
        .clone()
        .get_matches()
        .get_one("tui-rate")
        .expect("Missing tui-rate cli arg")
}

pub fn tui_debug_mode(command: &clap::Command) -> bool {
    command.clone().get_matches().get_flag("debug-mode")
}

pub fn tui_log_to_file(command: &clap::Command) -> bool {
    command.clone().get_matches().get_flag("log-file")
}
