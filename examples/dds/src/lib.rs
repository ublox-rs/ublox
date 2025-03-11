use log::info;
use rustdds::QosPolicies;
use rustdds::{with_key::DataWriter, DomainParticipant, Publisher, Timestamp, TopicKind};

pub mod cli;
pub mod idl;

pub fn get_type_name<T: ?Sized>(_: &T) -> String {
    let full_name = std::any::type_name::<T>();
    let name = full_name
        .split("::")
        .last()
        .unwrap_or("unknown_type")
        .to_string();
    name
}

pub fn write_sample<T>(
    writer: &DataWriter<T>,
    sample: &T,
) -> rustdds::dds::result::WriteResult<(), T>
where
    T: rustdds::Keyed + serde::ser::Serialize + Clone + std::fmt::Debug,
{
    let inow = Timestamp::now();
    writer.write(sample.clone(), Some(inow))
}

pub fn create_writer<T: serde::Serialize + rustdds::Keyed + Default>(
    topic_name: &str,
    participant: &DomainParticipant,
    publisher: &Publisher,
    qos: &QosPolicies,
) -> DataWriter<T> {
    let type_name = get_type_name(&T::default());
    info!("Creating topic '{}' with type '{}'", topic_name, type_name);
    let topic = participant
        .create_topic(topic_name.to_string(), type_name, qos, TopicKind::WithKey)
        .unwrap();

    info!("Creating DataWriter for topic '{}'", topic_name);
    publisher.create_datawriter_cdr::<T>(&topic, None).unwrap()
}
