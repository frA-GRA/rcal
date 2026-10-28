//! System test: CalBridge forwards messages from a ZMQ process to a DDS receiver.
//!
//! Topology:
//!
//!  zmq_peer_sender (separate process)
//!      │  TCP ZMQ on port_sender
//!      │
//!  CalBridge (in-process)
//!      Cal_ZMQ: add_transport_peer → port_sender (DISH receives from sender RADIO)
//!      Cal_DDS: DDS on domain_id
//!      │
//!  In-process DDS reader
//!      counts forwarded BridgeMsgs, asserts receipt of all 3 messages
//!
//! Using different transports on each bridge side means the echo loop is
//! naturally broken: messages forwarded ZMQ→DDS cannot return ZMQ→DDS→ZMQ
//! because the bridge's ZMQ DISH is pointed at the external sender, not its own
//! RADIO, so the DDS→ZMQ echo write lands on a RADIO with no subscribers.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

use rcal::bridge::{BridgeMsg, CalBridge};
use rcal::cal::{MessageListener, TopicQos, get_cal};
use rcal::calconfig;
use rcal::uci::base::UUID;

// ── port / domain allocators ──────────────────────────────────────────────────

// Start well above the zmq_asb_system range (3000+) to avoid port collisions.
static NEXT_PORT: AtomicU16 = AtomicU16::new(4500);
static NEXT_DOMAIN: AtomicU32 = AtomicU32::new(60);

fn next_port() -> u16 {
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

fn next_domain() -> u32 {
    NEXT_DOMAIN.fetch_add(1, Ordering::SeqCst)
}

// ── config builder ────────────────────────────────────────────────────────────

fn make_config(zmq_port: u16, dds_domain: u32) -> Arc<calconfig::CalConfig> {
    let ns = UUID::parse_str("6ef79d81-8a79-4750-9c6a-e5e50a30f81b").unwrap();
    let sys_uuid = UUID::generate_v3(&ns, format!("{zmq_port}-{dds_domain}").as_bytes());
    let toml = format!(
        "[system]\n\
         id = \"BridgeTest\"\n\
         uuid = \"{sys_uuid}\"\n\
         default_transport = \"ZMQ\"\n\
         \n\
         [[transport]]\n\
         id = \"ZMQ\"\n\
         type = \"zmq\"\n\
         uri = \"tcp://127.0.0.1:{zmq_port}\"\n\
         \n\
         [[transport]]\n\
         id = \"DDS\"\n\
         type = \"dds\"\n\
         uri = \"{dds_domain}\"\n\
         \n\
         [[service]]\n\
         id = \"BridgeSvc\"\n\
         \n\
         [[service.topic]]\n\
         id = \"data\"\n"
    );
    Arc::new(calconfig::parse_config(&toml).unwrap())
}

// ── listener ──────────────────────────────────────────────────────────────────

struct CountListener {
    count: Arc<AtomicUsize>,
}

impl MessageListener<BridgeMsg> for CountListener {
    fn on_message(&self, _msg: &Arc<BridgeMsg>) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

// ── test ──────────────────────────────────────────────────────────────────────

/// Bridge forwards IntMsgs published by a ZMQ sender process to a DDS receiver.
///
/// The sender (`zmq_peer_sender`) runs as a separate OS process on a TCP port.
/// The bridge's ZMQ Cal is wired to that port via `add_transport_peer` so its
/// DISH receives from the sender's RADIO rather than its own. The DDS Cal uses
/// an in-process participant; the test's DDS reader shares the same Cal via the
/// factory and counts arriving [`BridgeMsg`]s.
#[rcal_macros::init_test_logger]
#[tokio::test(flavor = "multi_thread")]
async fn bridge_forwards_zmq_sender_to_dds_receiver() {
    let port_sender = next_port();
    let port_bridge_zmq = next_port();
    let dds_domain = next_domain();

    let sender_bin = env!("CARGO_BIN_EXE_zmq_peer_sender");
    // Spawn before setting up the bridge so the sender's 1-second startup
    // delay overlaps with bridge construction.
    let mut child = std::process::Command::new(sender_bin)
        .args([port_sender.to_string(), "3".to_string()])
        .spawn()
        .expect("failed to spawn zmq_peer_sender");

    let config = make_config(port_bridge_zmq, dds_domain);

    // Bridge: ZMQ receives from the sender process, DDS forwards to receiver.
    let mut bridge = CalBridge::new(
        Arc::clone(&config),
        "BridgeSvc",
        &["ZMQ", "DDS"],
        logger.clone(),
    )
    .await
    .expect("CalBridge::new");
    bridge.add_transport_peer("ZMQ", format!("tcp://127.0.0.1:{port_sender}"));
    bridge.activate().expect("activate");

    // Receiver: shares the DDS Cal with the bridge via the global factory.
    let mut receiver = get_cal("BridgeSvc", Some("DDS"), Arc::clone(&config), logger)
        .await
        .expect("receiver DDS Cal");
    let count = Arc::new(AtomicUsize::new(0));
    let mut reader = receiver
        .create_reader::<BridgeMsg>("data", TopicQos::default())
        .expect("DDS reader");
    reader
        .add_listener(Arc::new(CountListener {
            count: Arc::clone(&count),
        }))
        .expect("add listener");

    // Wait for sender startup (1 s), message delivery, ZMQ→DDS forwarding.
    tokio::time::sleep(Duration::from_millis(2500)).await;

    assert_eq!(
        count.load(Ordering::Relaxed),
        3,
        "bridge must forward all 3 ZMQ messages to the DDS receiver"
    );

    child.wait().unwrap();
    bridge.deactivate();
    receiver.close().unwrap();
}
