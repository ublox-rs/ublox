use clap::{value_parser, Arg};
use dds::{
    cli,
    idl::ubx::{esf_alg::EsfAlg, esf_status::EsfStatus, nav_pvt::NavPvt},
};
use futures::StreamExt;
use log::{debug, error, info};
use rustdds::{
    with_key::Sample, DomainParticipant, QosPolicies, Subscriber, TopicDescription, TopicKind,
};
use serde::Deserialize;
use std::thread::{self, JoinHandle};

fn main() {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();

    let cli = cli::dds_args();
    let cli = cli
        .arg(
            Arg::new("topic-type")
                .value_name("topic-type")
                .long("topic-type")
                .short('T')
                .default_value("NavPvt")
                .required(false)
                .value_parser(value_parser!(String))
                .help("Select the topic type to subscribe to: Possible values: HnrPvt, HnrAtt, HnrIns, EsfImuStatus, EsfStatus, NavPvt."),
        )
        .version(clap::crate_version!().to_string())
        .about("Dummy data publisher for testing".to_string())
        .get_matches();

    let domain_id = *cli.get_one("domain_id").unwrap();
    info!("Creating participant for domain ID: {domain_id}");
    let domain_participant = DomainParticipant::new(domain_id).unwrap();

    let qos = if cli.get_flag("reliable") {
        cli::create_reliable_qos(&cli)
    } else {
        info!("Default to BestEffort QoS as no argument was provided on cmdline");
        cli::create_besteffort_qos(&cli)
    };

    info!("Creating Subscriber");
    let subscriber = domain_participant.create_subscriber(&qos).unwrap();

    let topic_name = cli
        .get_one::<String>("topic")
        .cloned()
        .to_owned()
        .unwrap_or("ubx-nav-pvt".to_string());
    let key = cli.get_one::<String>("key").unwrap();

    let topic_type = cli.get_one::<String>("topic-type").cloned();
    match topic_type.as_deref() {
        Some("NavPvt") => {
            let h = ublox_topic_thread::<NavPvt>(
                &topic_name,
                key,
                &domain_participant,
                &subscriber,
                &qos,
            );
            h.join().unwrap();
        },

        Some("EsfAlg") => {
            let h = ublox_topic_thread::<EsfAlg>(
                &"ubx-esf-alg".to_string(),
                key,
                &domain_participant,
                &subscriber,
                &qos,
            );
            h.join().unwrap();
        },
        Some("EsfStatus") => {
            let h = ublox_topic_thread::<EsfStatus>(
                &"ubx-esf-status".to_string(),
                key,
                &domain_participant,
                &subscriber,
                &qos,
            );
            h.join().unwrap();
        },
        Some(name) => {
            error!("Unknown topic type {name}");
        },
        None => {
            error!("No topic name provided, nothing to subscribe to. Stopping");
        },
    }
}

fn ublox_topic_thread<T>(
    topic_name: &String,
    _key: &str,
    participant: &DomainParticipant,
    subscriber: &Subscriber,
    qos: &QosPolicies,
) -> JoinHandle<()>
where
    T: rustdds::Keyed
        + Default
        + for<'de> serde::de::Deserialize<'de>
        + std::fmt::Debug
        + std::fmt::Display
        + 'static
        + std::marker::Send,
    for<'de> <T as rustdds::Keyed>::K: Deserialize<'de>,
    <T as rustdds::Keyed>::K: std::fmt::Debug + 'static,
{
    let type_name = dds::get_type_name(&T::default());
    info!("Creating topic '{topic_name}' with type '{type_name}'");
    let topic = participant
        .create_topic(
            topic_name.clone().to_string(),
            type_name.clone(),
            qos,
            TopicKind::WithKey,
        )
        .unwrap();

    let subscriber = subscriber.clone();
    let t = topic_name.clone();

    thread::spawn(move || loop {
        info!("Creating a DataReader for topic: '{t}'");
        let reader = subscriber.create_datareader_cdr::<T>(&topic, None).unwrap();
        smol::block_on(async {
            let mut datareader_stream = reader.async_sample_stream();
            let mut datareader_event_stream = datareader_stream.async_event_stream();

            loop {
                futures::select! {
                  r=datareader_stream.select_next_some()=>{
                    match r{
                      Ok(v)=>{
                        debug!("Sample: {v:?}");
                        debug!("SampleInfo: {:?}", v.sample_info());
                        match v.value() {
                        Sample::Value(sample) => info!(
                          "topic {}: {}",
                          topic.name(),
                          sample,
                        ),
                        Sample::Dispose(key) => {
                            info!("Disposed key {key:?}");
                        },
                      }
                    }
                      Err(e)=> {
                        error!("Got error when reading DDS sample: {e:?}");
                        break;
                      }
                    }
                  }
                  e=datareader_event_stream.select_next_some()=>{
                    info!("DataReader event: {e:?}");
                  }
                }
            }
        })
    })
}
