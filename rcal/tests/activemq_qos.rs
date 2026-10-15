//! QoS integration tests for the ActiveMQ ASB.
//!
//! Requires a running ActiveMQ Classic or Artemis broker on localhost:61613.
//! Run with:
//!   cargo test --features activemq --test activemq_qos

use std::time::Duration;

use rcal::QName;
use rcal::cal::{Cal, Expiration, MessageBuffer, TimeBasedFilter, TopicQos, get_cal};
use rcal::uci::CalMessage;

const BROKER_HOST: &str = "localhost";
const BROKER_PORT: u16 = 61613;

fn make_config() -> std::sync::Arc<rcal::calconfig::CalConfig> {
    use rcal::calconfig;
    use rcal::uci::base::UUID;
    const NS: &str = "6ef79d81-8a79-4750-9c6a-e5e50a30f81b";
    let ns = UUID::parse_str(NS).unwrap();
    let sys_uuid = UUID::generate_v3(&ns, format!("{BROKER_HOST}:{BROKER_PORT}").as_bytes());
    let toml = format!(
        "[system]\nid = \"TestSystem\"\nuuid = \"{sys_uuid}\"\ndefault_transport = \"TestAmq\"\n\
         \n[[transport]]\nid = \"TestAmq\"\ntype = \"activemq\"\nuri = \"stomp://{BROKER_HOST}:{BROKER_PORT}\"\n"
    );
    std::sync::Arc::new(calconfig::parse_config(&toml).unwrap())
}

async fn make_bus(service: &str, logger: slog::Logger) -> Cal {
    get_cal(service, None::<&str>, make_config(), logger)
        .await
        .unwrap()
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct IntMsg {
    value: i32,
}

impl CalMessage for IntMsg {
    fn message_type_name() -> QName {
        QName::new(Some("test"), "IntMsg")
    }
    fn cal_create() -> Self {
        Self { value: 0 }
    }
}

/// TimeBasedFilter drops messages within min_separation (CAL-005431).
#[rcal_macros::init_test_logger]
#[tokio::test]
async fn test_time_based_filter() {
    let mut bus = make_bus("TbfTest", logger).await;

    let wqos = TopicQos::default();
    let rqos = TopicQos {
        time_based_filter: Some(TimeBasedFilter {
            min_separation: Duration::from_millis(500),
        }),
        ..Default::default()
    };

    let mut writer = bus.create_writer::<IntMsg>("tbf", wqos).unwrap();
    let mut reader = bus.create_reader::<IntMsg>("tbf", rqos).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send two rapid messages — second should be dropped by filter.
    writer.write(&IntMsg { value: 1 }).unwrap();
    writer.write(&IntMsg { value: 2 }).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let m1 = reader.read(Some(Duration::from_secs(2))).unwrap().unwrap();
    assert_eq!(m1.value, 1, "first message should pass filter");
    assert!(
        reader.read_no_wait().unwrap().is_none(),
        "second message should be dropped by TimeBasedFilter"
    );

    bus.close().unwrap();
}

/// Expiration drops messages older than max_age (CAL-005437).
#[rcal_macros::init_test_logger]
#[tokio::test]
async fn test_expiration() {
    let mut bus = make_bus("ExpiryTest", logger).await;

    let wqos = TopicQos::default();
    let rqos = TopicQos {
        expiration: Some(Expiration {
            max_age: Duration::from_millis(100),
        }),
        ..Default::default()
    };

    let mut writer = bus.create_writer::<IntMsg>("expiry", wqos).unwrap();
    let mut reader = bus.create_reader::<IntMsg>("expiry", rqos).unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    writer.write(&IntMsg { value: 7 }).unwrap();
    // Wait longer than max_age before reading.
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(
        reader.read_no_wait().unwrap().is_none(),
        "expired message should be discarded"
    );

    bus.close().unwrap();
}

/// Writer buffer drops oldest when full (CAL-005445).
#[rcal_macros::init_test_logger]
#[tokio::test]
async fn test_writer_buffer_drop_oldest() {
    let mut bus = make_bus("WBufTest", logger).await;

    let wqos = TopicQos {
        writer_buffer: Some(MessageBuffer { max_messages: 2 }),
        ..Default::default()
    };
    // No reader — we just verify the buffer doesn't panic or deadlock.
    let mut writer = bus.create_writer::<IntMsg>("wbuf", wqos).unwrap();

    for v in 0..5 {
        writer.write(&IntMsg { value: v }).unwrap();
    }

    bus.close().unwrap();
}

/// Reader buffer drops oldest when full (CAL-015746).
#[rcal_macros::init_test_logger]
#[tokio::test]
async fn test_reader_buffer_drop_oldest() {
    let mut bus = make_bus("RBufTest", logger).await;

    let wqos = TopicQos::default();
    let rqos = TopicQos {
        reader_buffer: Some(MessageBuffer { max_messages: 2 }),
        ..Default::default()
    };

    let mut writer = bus.create_writer::<IntMsg>("rbuf", wqos).unwrap();
    let mut reader = bus.create_reader::<IntMsg>("rbuf", rqos).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    for v in [10, 20, 30] {
        writer.write(&IntMsg { value: v }).unwrap();
    }
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Only 2 slots; oldest (10) was dropped.
    let m1 = reader.read(Some(Duration::from_secs(2))).unwrap().unwrap();
    let m2 = reader.read(Some(Duration::from_secs(2))).unwrap().unwrap();
    assert_eq!([m1.value, m2.value], [20, 30]);
    assert!(reader.read_no_wait().unwrap().is_none());

    bus.close().unwrap();
}
