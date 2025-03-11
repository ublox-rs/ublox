use clap::{Arg, ArgMatches, Command, ValueEnum};

use rustdds::policy::{Deadline, Durability, History, Lifespan, Liveliness, Reliability};
use rustdds::{Duration, QosPolicies, QosPolicyBuilder};

const HISTORY_DEPTH_ARG_ID: &str = "history-depth";
const DEFAULT_HISTORY_DEPTH: i32 = 10;

const DURABILITY_ARG_ID: &str = "durability";

const DEADLINE_ARG_ID: &str = "deadline";
const DEFAULT_DEADLINE_DURATION_SEC: f64 = 1.0;

const LIFESPAN_ARG_ID: &str = "lifespan";
const DEFAULT_LIFESPAN_DURATION_SEC: f64 = 2.0;

const RELIABLE_MAX_BLOCKING_DURATION_ID: &str = "reliability-max-blocking";

const LIVELINESS_ARG_ID: &str = "liveliness";
const LIVELINESS_LEASE_DURATION_ID: &str = "liveliness-duration";
const DEFAULT_LEASE_DURATION_SEC: f64 = 1.0;

pub fn create_reliable_qos(arg_matches: &ArgMatches) -> QosPolicies {
    let depth = *arg_matches
        .get_one(HISTORY_DEPTH_ARG_ID)
        .unwrap_or(&DEFAULT_HISTORY_DEPTH);

    let cli_durability = *arg_matches
        .get_one(DURABILITY_ARG_ID)
        .unwrap_or(&CliDurability::Volatile);

    let cli_liveliness = *arg_matches
        .get_one(LIVELINESS_ARG_ID)
        .unwrap_or(&CliLiveliness::Automatic);

    let deadline = *arg_matches
        .get_one(DEADLINE_ARG_ID)
        .unwrap_or(&DEFAULT_DEADLINE_DURATION_SEC);

    let dds_reliability = match arg_matches.get_one(RELIABLE_MAX_BLOCKING_DURATION_ID) {
        None => Reliability::Reliable {
            max_blocking_time: Duration::ZERO,
        },
        Some(v) => Reliability::Reliable {
            max_blocking_time: Duration::from_frac_seconds(*v),
        },
    };

    let dds_lifespan = match arg_matches.get_one(LIFESPAN_ARG_ID) {
        None => Lifespan {
            duration: Duration::INFINITE,
        },
        Some(v) => Lifespan {
            duration: Duration::from_frac_seconds(*v),
        },
    };

    let liveliness_lease_duration = match arg_matches.get_one(LIVELINESS_LEASE_DURATION_ID) {
        None => Duration::from_frac_seconds(DEFAULT_LEASE_DURATION_SEC),
        Some(v) => Duration::from_frac_seconds(*v),
    };

    let dds_liveliness = match cli_liveliness {
        CliLiveliness::Automatic => Liveliness::Automatic {
            lease_duration: liveliness_lease_duration,
        },
        CliLiveliness::ManualByParticipant => Liveliness::ManualByParticipant {
            lease_duration: liveliness_lease_duration,
        },
        CliLiveliness::ManualByTopic => Liveliness::ManualByTopic {
            lease_duration: liveliness_lease_duration,
        },
    };

    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .reliability(dds_reliability)
            .liveliness(dds_liveliness)
            .history(History::KeepLast { depth })
            .durability(Durability::from(cli_durability))
            .deadline(Deadline(Duration::from_frac_seconds(deadline)))
            .lifespan(dds_lifespan)
            .build()
    };
    service_qos
}

pub fn create_besteffort_qos(arg_matches: &ArgMatches) -> QosPolicies {
    let depth = *arg_matches
        .get_one(HISTORY_DEPTH_ARG_ID)
        .unwrap_or(&DEFAULT_HISTORY_DEPTH);

    let cli_durability = *arg_matches
        .get_one(DURABILITY_ARG_ID)
        .unwrap_or(&CliDurability::Volatile);

    let cli_liveliness = *arg_matches
        .get_one(LIVELINESS_ARG_ID)
        .unwrap_or(&CliLiveliness::ManualByTopic);

    let deadline = *arg_matches
        .get_one(DEADLINE_ARG_ID)
        .unwrap_or(&DEFAULT_DEADLINE_DURATION_SEC);

    let dds_lifespan = match arg_matches.get_one(LIFESPAN_ARG_ID) {
        None => Lifespan {
            duration: Duration::from_frac_seconds(DEFAULT_LIFESPAN_DURATION_SEC),
        },
        Some(&f64::INFINITY) => Lifespan {
            duration: Duration::INFINITE,
        },
        Some(v) => Lifespan {
            duration: Duration::from_frac_seconds(*v),
        },
    };

    let liveliness_lease_duration = match arg_matches.get_one(LIVELINESS_LEASE_DURATION_ID) {
        None => Duration::from_frac_seconds(DEFAULT_LEASE_DURATION_SEC),
        Some(v) => Duration::from_frac_seconds(*v),
    };

    let dds_liveliness = match cli_liveliness {
        CliLiveliness::Automatic => Liveliness::Automatic {
            lease_duration: liveliness_lease_duration,
        },
        CliLiveliness::ManualByParticipant => Liveliness::ManualByParticipant {
            lease_duration: liveliness_lease_duration,
        },
        CliLiveliness::ManualByTopic => Liveliness::ManualByTopic {
            lease_duration: liveliness_lease_duration,
        },
    };

    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .reliability(Reliability::BestEffort)
            .liveliness(dds_liveliness)
            .history(History::KeepLast { depth })
            .durability(Durability::from(cli_durability))
            .deadline(Deadline(Duration::from_frac_seconds(deadline)))
            .lifespan(dds_lifespan)
            .build()
    };
    service_qos
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliDurability {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

impl From<CliDurability> for Durability {
    fn from(other: CliDurability) -> Durability {
        match other {
            CliDurability::Volatile => Durability::Volatile,
            CliDurability::TransientLocal => Durability::TransientLocal,
            CliDurability::Transient => Durability::Transient,
            CliDurability::Persistent => Durability::Persistent,
        }
    }
}

impl From<Durability> for CliDurability {
    fn from(other: Durability) -> CliDurability {
        match other {
            Durability::Volatile => CliDurability::Volatile,
            Durability::TransientLocal => CliDurability::TransientLocal,
            Durability::Transient => CliDurability::Transient,
            Durability::Persistent => CliDurability::Persistent,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliLiveliness {
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

pub fn dds_args() -> Command {
    Command::new(clap::crate_name!())
        .arg(
            Arg::new("domain_id")
                .display_order(100)
                .short('d')
                .long("domain")
                .value_name("value")
                .value_parser(clap::value_parser!(u16))
                .default_value("0")
                .required(false)
                .help("Sets the DDS domain id number"),
        )
        .arg(
            Arg::new("topic")
                .display_order(101)
                .short('t')
                .long("topic")
                .value_name("name")
                .help("Sets the DDS topic name for the NavPvt message")
                .required(false),
        )
        .arg(
            Arg::new("key")
                .display_order(102)
                .short('k')
                .long("key")
                .value_name("key-name")
                .default_value("empty")
                .value_parser(clap::value_parser!(String))
                .required(false)
                .help("Sets the DDS key for all published topics"),
        )
        .arg(
            Arg::new("best-effort")
                .display_order(103)
                .help("Sets DDS QoS reliability BEST_EFFORT")
                .short('b')
                .long("best-effort")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("reliable"),
        )
        .arg(
            Arg::new("reliable")
                .display_order(104)
                .help("Sets DDS QoS reliability to RELIABLE")
                .short('r')
                .long("reliable")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("best-effort"),
        )
        .arg(
            Arg::new(HISTORY_DEPTH_ARG_ID)
                .value_name("num samples")
                .long(HISTORY_DEPTH_ARG_ID)
                .display_order(105)
                .help("Sets DDS QoS history depth")
                .value_parser(clap::value_parser!(i32))
                .default_value("1")
                .required(false),
        )
        .arg(
            Arg::new(DURABILITY_ARG_ID)
                .display_order(106)
                .short('D')
                .long(DURABILITY_ARG_ID)
                .value_name("type")
                .help("Sets DDS QoS durability")
                .default_value("volatile")
                .value_parser(clap::value_parser!(CliDurability)),
        )
        .arg(
            Arg::new(DEADLINE_ARG_ID)
                .display_order(107)
                .help("Sets DDS QoS 'deadline' policy duration in seconds")
                .short('f')
                .long(DEADLINE_ARG_ID)
                .value_parser(clap::value_parser!(f64))
                .value_name("duration"),
        )
        .arg(
            Arg::new(LIFESPAN_ARG_ID)
                .display_order(108)
                .help("Sets DDS QoS 'lifespan' policy duration in seconds")
                .short('l')
                .long(LIFESPAN_ARG_ID)
                .value_name("lifespan")
                .default_value(DEFAULT_LIFESPAN_DURATION_SEC.to_string())
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new(RELIABLE_MAX_BLOCKING_DURATION_ID)
                .display_order(109)
                .help("Sets DDS QoS 'reliability' policy max blocking time duration in seconds. Only applies to Reliable QoS and not used when BestEffort QoS is selected.")
                .long(RELIABLE_MAX_BLOCKING_DURATION_ID)
                .default_value("0.0")
                .short('B')
                .value_name("seconds")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new(LIVELINESS_ARG_ID)
                .display_order(110)
                .short('L')
                .long(LIVELINESS_ARG_ID)
                .value_name("type")
                .help("Sets DDS QoS liveliness policy")
                .default_value("automatic")
                .value_parser(clap::value_parser!(CliLiveliness)),
        )
        .arg(
            Arg::new(LIVELINESS_LEASE_DURATION_ID)
                .display_order(111)
                .help("Sets DDS QoS 'liveliness' policy lease duration in seconds")
                .long(LIVELINESS_LEASE_DURATION_ID)
                .default_value(DEFAULT_LEASE_DURATION_SEC.to_string())
                .value_name("seconds")
                .value_parser(clap::value_parser!(f64))
        )
}

pub fn ublox_dds_args() -> Command {
    let dds_command = dds_args();
    let mut ublox_command = ublox_device::cli::CommandBuilder::default().build();

    // Combine dds and ublox args
    for d_arg in dds_command.get_arguments() {
        ublox_command = ublox_command.arg(d_arg)
    }

    ublox_command.clone()
}
