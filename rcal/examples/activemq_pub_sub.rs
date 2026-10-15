//! ActiveMQ pub/sub example.
//!
//! Demonstrates a writer and a reader on the same STOMP broker, using XML
//! serialization and Reliable QoS.
//!
//! Prerequisites: ActiveMQ Classic or Artemis running on localhost:61613.
//!
//! Run with:
//!   cargo run --features activemq --example activemq_pub_sub

use std::sync::Arc;
use std::time::Duration;

use rcal::QName;
use rcal::cal::{Reliability, TopicQos, get_cal};
use rcal::uci::CalMessage;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SensorReading {
    sensor_id: u32,
    value: f64,
}

impl CalMessage for SensorReading {
    fn message_type_name() -> QName {
        QName::new(Some("example"), "SensorReading")
    }
    fn cal_create() -> Self {
        Self {
            sensor_id: 0,
            value: 0.0,
        }
    }
}

fn make_config() -> Arc<rcal::calconfig::CalConfig> {
    use rcal::calconfig;
    use rcal::uci::base::UUID;
    const NS: &str = "6ef79d81-8a79-4750-9c6a-e5e50a30f81b";
    let ns = UUID::parse_str(NS).unwrap();
    let sys_uuid = UUID::generate_v3(&ns, b"localhost:61613");
    let toml = format!(
        "[system]\nid = \"ExampleSystem\"\nuuid = \"{sys_uuid}\"\ndefault_transport = \"Amq\"\n\
         \n[[transport]]\nid = \"Amq\"\ntype = \"activemq\"\nuri = \"stomp://localhost:61613\"\n"
    );
    Arc::new(calconfig::parse_config(&toml).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use slog::Drain;
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog::o!());
    let config = make_config();

    let reliable_qos = TopicQos {
        reliability: Reliability::Reliable,
        ..Default::default()
    };

    let mut publisher = get_cal(
        "Publisher",
        None::<&str>,
        Arc::clone(&config),
        logger.clone(),
    )
    .await?;
    let mut subscriber = get_cal(
        "Subscriber",
        None::<&str>,
        Arc::clone(&config),
        logger.clone(),
    )
    .await?;

    let mut writer = publisher
        .create_writer::<SensorReading>("sensors", reliable_qos.clone())
        .map_err(|e| e.to_string())?;
    let mut reader = subscriber
        .create_reader::<SensorReading>("sensors", reliable_qos)
        .map_err(|e| e.to_string())?;

    // Allow subscriptions to register before publishing.
    tokio::time::sleep(Duration::from_millis(500)).await;

    for i in 0..5u32 {
        let reading = SensorReading {
            sensor_id: i,
            value: (i as f64) * 1.5,
        };
        println!("Publishing: {reading:?}");
        writer.write(&reading).map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    for _ in 0..5 {
        if let Some(msg) = reader
            .read(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?
        {
            println!("Received: {msg:?}");
        }
    }

    publisher.close().map_err(|e| e.to_string())?;
    subscriber.close().map_err(|e| e.to_string())?;
    Ok(())
}
