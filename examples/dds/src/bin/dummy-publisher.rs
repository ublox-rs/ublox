use anyhow::Result;
use dds::{cli, idl};
use log::{info, warn};
use rustdds::DomainParticipant;
use std::time::Duration;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();

    let cli = cli::dds_args();
    let cli = cli
        .version(clap::crate_version!().to_string())
        .about("Dummy data publisher for testing".to_string())
        .get_matches();

    let domain_id = *cli.get_one("domain_id").expect("Domain ID not provided");
    let topic_key = cli
        .get_one::<String>("key")
        .cloned()
        .expect("DDS key provided")
        .to_owned();

    info!("Creating participant for domain ID: {domain_id}");
    let domain_participant = DomainParticipant::new(domain_id).unwrap();

    let qos = if cli.get_flag("reliable") {
        cli::create_reliable_qos(&cli)
    } else {
        info!("Default to BestEffort QoS as no argument was provided on cmdline");
        cli::create_besteffort_qos(&cli)
    };

    info!("Creating Publisher");
    let publisher = domain_participant.create_publisher(&qos).unwrap();

    let writer_pvt = dds::create_writer::<idl::ubx::nav_pvt::NavPvt>(
        "ubx-nav-pvt",
        &domain_participant,
        &publisher,
        &qos,
    );

    let writer_esf_imu_alg = dds::create_writer::<idl::ubx::esf_alg::EsfAlg>(
        "ubx-esf-alg",
        &domain_participant,
        &publisher,
        &qos,
    );

    let writer_esf_status = dds::create_writer::<idl::ubx::esf_status::EsfStatus>(
        "ubx-esf-status",
        &domain_participant,
        &publisher,
        &qos,
    );

    // Start reading data
    info!("uBlox device opened, waiting for UBX messages ...");

    loop {
        let msg = idl::ubx::nav_pvt::NavPvt {
            key: topic_key.clone(),
            itow: rand::random(),
            ..Default::default()
        };
        info!("Publishing: {msg:?}  ");
        if let Err(e) = dds::write_sample(&writer_pvt, &msg) {
            warn!("failed to write PVT message: {e} ");
        }

        let msg = idl::ubx::esf_status::EsfStatus {
            key: topic_key.clone(),
            itow: rand::random(),
            fusion_status: idl::ubx::esf_status::EsfFusionStatus {
                ..Default::default()
            },
            sensors: Vec::new(),
        };
        info!("Publishing: {msg:?}  ");
        if let Err(e) = dds::write_sample(&writer_esf_status, &msg) {
            warn!("failed to write EsfStatus message: {e} ");
        }

        let msg = idl::ubx::esf_alg::EsfAlg {
            key: topic_key.clone(),
            itow: rand::random(),
            pitch: rand::random(),
            yaw: rand::random(),
            ..Default::default()
        };

        info!("Publishing: {msg:?}  ");
        if let Err(e) = dds::write_sample(&writer_esf_imu_alg, &msg) {
            warn!("failed to write EsfAlg message: {e} ");
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}
