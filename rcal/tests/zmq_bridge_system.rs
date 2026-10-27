//! System test for CalBridge: ZMQ inproc forwarding across two transports.
//!
//! Topology (all in-process; three distinct ZMQ inproc addresses):
//!
//!  ┌──────────────────────────────────────────────────────────────────┐
//!  │  sender Cal              CalBridge                  receiver Cal  │
//!  │  (RADIO on sender URI) → Cal_A DISH → Cal_B RADIO → (DISH)       │
//!  └──────────────────────────────────────────────────────────────────┘
//!
//! Cal_A's DISH is pointed at the sender's RADIO via `add_transport_peer`.
//! This breaks the echo loop: when the bridge forwards A→B and the bridge's
//! Cal_B reader tries to echo back to Cal_A's RADIO, nobody is subscribed to
//! Cal_A's RADIO (Cal_A's DISH is pointed elsewhere), so the echo is dropped.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::time::Duration;

use rcal::QName;
use rcal::bridge::{BridgeMsg, CalBridge};
use rcal::cal::{MessageListener, TopicQos, create_cal, get_cal};
use rcal::calconfig;
use rcal::uci::base::UUID;
use rcal::uci::CalMessage;

// ── unique-suffix allocator ───────────────────────────────────────────────────

static NEXT_SUFFIX: AtomicU16 = AtomicU16::new(1);

fn next_suffix() -> u16 {
    NEXT_SUFFIX.fetch_add(1, Ordering::SeqCst)
}

// ── config builder ────────────────────────────────────────────────────────────

fn make_config(suffix: u16) -> Arc<calconfig::CalConfig> {
    let ns = UUID::parse_str("6ef79d81-8a79-4750-9c6a-e5e50a30f81b").unwrap();
    let sys_uuid = UUID::generate_v3(&ns, suffix.to_string().as_bytes());
    // Transport IDs include suffix so each test gets isolated Cal instances in
    // the global CAL_FACTORY (keyed by service_id + transport_id).
    let toml = format!(
        "[system]\n\
         id = \"BridgeTest\"\n\
         uuid = \"{sys_uuid}\"\n\
         default_transport = \"A_{suffix}\"\n\
         \n\
         [[transport]]\n\
         id = \"Sender_{suffix}\"\n\
         type = \"zmq\"\n\
         uri = \"inproc://btest-sender-{suffix}\"\n\
         \n\
         [[transport]]\n\
         id = \"A_{suffix}\"\n\
         type = \"zmq\"\n\
         uri = \"inproc://btest-a-{suffix}\"\n\
         \n\
         [[transport]]\n\
         id = \"B_{suffix}\"\n\
         type = \"zmq\"\n\
         uri = \"inproc://btest-b-{suffix}\"\n\
         \n\
         [[service]]\n\
         id = \"Svc\"\n\
         \n\
         [[service.topic]]\n\
         id = \"data\"\n"
    );
    Arc::new(calconfig::parse_config(&toml).unwrap())
}

// ── counter listener ──────────────────────────────────────────────────────────

struct CountListener {
    count: Arc<AtomicUsize>,
}

impl MessageListener<BridgeMsg> for CountListener {
    fn on_message(&self, _msg: &Arc<BridgeMsg>) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

// ── test ──────────────────────────────────────────────────────────────────────

/// Bridge forwards N messages from a sender Cal to a receiver Cal.
///
/// The sender is an independent [`create_cal`] instance (not shared with the
/// bridge). The bridge's Cal_A reader is pointed at the sender's RADIO via
/// [`CalBridge::add_transport_peer`] so it receives the sender's messages.
/// The receiver shares Cal_B with the bridge (via [`get_cal`]) and counts
/// the forwarded BridgeMsgs with a callback listener.
#[rcal_macros::init_test_logger]
#[tokio::test]
async fn bridge_forwards_messages_between_transports() {
    let suffix = next_suffix();
    let config = make_config(suffix);
    let a_id = format!("A_{suffix}");
    let b_id = format!("B_{suffix}");
    let sender_id = format!("Sender_{suffix}");
    let sender_uri = format!("inproc://btest-sender-{suffix}");

    // Sender: fresh Cal (not in factory), RADIO on sender URI.
    // Uses create_cal so it never collides with the bridge's factory entries.
    let mut sender = create_cal(
        &format!("SenderSvc_{suffix}"),
        Some(sender_id.as_str()),
        Arc::clone(&config),
        logger.clone(),
    )
    .await
    .expect("sender Cal");
    let mut writer = sender
        .create_writer::<BridgeMsg>("data", TopicQos::default())
        .expect("sender writer");

    // Bridge: Cal_A on inproc://btest-a, Cal_B on inproc://btest-b.
    // add_transport_peer wires Cal_A's DISH to the sender's RADIO so the bridge
    // receives from the sender instead of looping back to its own RADIO.
    let mut bridge = CalBridge::new(
        Arc::clone(&config),
        "Svc",
        &[a_id.as_str(), b_id.as_str()],
        logger.clone(),
    )
    .await
    .expect("CalBridge::new");
    bridge.add_transport_peer(&a_id, &sender_uri);
    bridge.activate().expect("activate");

    // Receiver: shares Cal_B with the bridge via the factory.
    let mut receiver = get_cal("Svc", Some(b_id.as_str()), Arc::clone(&config), logger)
        .await
        .expect("receiver Cal");
    let count = Arc::new(AtomicUsize::new(0));
    let mut reader = receiver
        .create_reader::<BridgeMsg>("data", TopicQos::default())
        .expect("receiver reader");
    reader
        .add_listener(Arc::new(CountListener {
            count: Arc::clone(&count),
        }))
        .expect("add listener");

    // Allow all DISH/RADIO connections to establish.
    tokio::time::sleep(Duration::from_millis(150)).await;

    const N: usize = 3;
    for _ in 0..N {
        writer.write(&BridgeMsg::cal_create()).expect("write");
    }

    // Allow forwarding to complete (inproc is fast; 300ms is generous).
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        count.load(Ordering::Relaxed),
        N,
        "bridge must forward exactly {N} messages from sender to receiver"
    );

    bridge.deactivate();
    sender.close().unwrap();
    receiver.close().unwrap();
}
