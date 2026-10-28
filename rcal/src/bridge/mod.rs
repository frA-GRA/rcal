//! CAL bridge: repeats messages across multiple CAL instances (issue #86).
//!
//! Connects two or more CALs and forwards every message received on one
//! to all others, without updating timestamps (dumb repeater / hub).
//!
//! # Filtered mode
//! Only topics listed in the service's `[[service.topic]]` config section
//! are forwarded. Topics with `direction = "Out"` are skipped (write-only
//! on this service).
//!
//! # Binary
//! The `cal_bridge` binary reads a `[bridge]` section from `CALConfig.toml`:
//! ```toml
//! [bridge]
//! service  = "CalBridge"
//! transports = ["TransportA", "TransportB"]
//! ```
//!
//! # Service all-listeners API
//! Use [`register_all_listeners`] (or the [`setup_all_listeners!`] macro) to
//! register a single callback for every configured topic on a service without
//! calling `create_reader` per type:
//! ```ignore
//! bridge::register_all_listeners(&mut service, &config, "MySvc",
//!     TopicQos::default(),
//!     |topic, msg| println!("{topic}: {:?}", msg))?;
//! ```

use std::any::Any;
use std::sync::{Arc, Mutex};

use slog::Logger;

use crate::QName;
use crate::cal::{AbstractCal, AbstractWriter, Cal, MessageListener, TopicQos, get_cal};
use crate::calconfig::{CalConfig, TopicDirection};
use crate::uci::{CalMessage, CalResult};

// ════════════════════════════════════════════════════════════════════════════
// BridgeMsg
// ════════════════════════════════════════════════════════════════════════════

/// Type-erased [`CalMessage`] used internally by the bridge.
///
/// Deserializes any topic's wire bytes into a JSON value tree, allowing the
/// bridge to forward messages without knowing their concrete types at compile
/// time.
///
/// # Note
/// ponytail: serde_json::Value as intermediate may lose XML-specific metadata
/// (attributes, namespace prefixes). Acceptable for same-serialization-format
/// bridges.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct BridgeMsg(serde_json::Value);

impl CalMessage for BridgeMsg {
    fn message_type_name() -> QName {
        QName::new(None, "any")
    }

    fn cal_create() -> Self {
        BridgeMsg(serde_json::Value::Object(Default::default()))
    }
}

// ════════════════════════════════════════════════════════════════════════════
// TypedForwarder  (private)
// ════════════════════════════════════════════════════════════════════════════

struct TypedForwarder {
    targets: Vec<Arc<Mutex<Box<dyn AbstractWriter<BridgeMsg>>>>>,
}

impl MessageListener<BridgeMsg> for TypedForwarder {
    fn on_message(&self, message: &Arc<BridgeMsg>) {
        for writer in &self.targets {
            // Timestamps NOT updated — forwarded as received (bridge contract: dumb repeater).
            if let Ok(mut w) = writer.lock() {
                let _ = w.write(message);
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// CalBridge
// ════════════════════════════════════════════════════════════════════════════

/// A bridge that repeats messages across multiple CAL instances.
///
/// Create with [`CalBridge::new`], then call [`CalBridge::activate`] to begin
/// forwarding. Call [`CalBridge::deactivate`] (or drop the value) to stop.
pub struct CalBridge {
    cals: Vec<Cal>,
    transport_ids: Vec<String>,
    config: Arc<CalConfig>,
    service_id: String,
    logger: Logger,
    /// Keeps readers and writers alive so their background tasks continue running.
    _keep_alive: Vec<Box<dyn Any + Send>>,
}

impl CalBridge {
    /// Creates a `CalBridge` connecting the named transports.
    ///
    /// `service_id`: a `[[service]]` entry in the config whose `[[service.topic]]`
    /// list defines which topics are bridged.
    /// `transport_ids`: two or more transport IDs from `[[transport]]` entries.
    pub async fn new(
        config: Arc<CalConfig>,
        service_id: impl Into<String>,
        transport_ids: &[&str],
        logger: Logger,
    ) -> CalResult<Self> {
        let service_id = service_id.into();
        let mut cals = Vec::with_capacity(transport_ids.len());
        for &transport_id in transport_ids {
            let cal = get_cal(
                service_id.clone(),
                Some(transport_id.to_string()),
                Arc::clone(&config),
                logger.clone(),
            )
            .await?;
            cals.push(cal);
        }
        Ok(Self {
            cals,
            transport_ids: transport_ids.iter().map(|s| s.to_string()).collect(),
            config,
            service_id,
            logger,
            _keep_alive: Vec::new(),
        })
    }

    /// Registers a remote RADIO URI for the named transport (ZMQ only).
    ///
    /// Must be called before [`activate`]. Causes the bridge's reader DISH on
    /// that transport to connect to the given URI instead of its own RADIO,
    /// which is necessary for multi-process topologies and also prevents the
    /// self-echo loop that arises in single-process use.
    pub fn add_transport_peer(&mut self, transport_id: &str, uri: impl Into<String>) {
        if let Some(idx) = self.transport_ids.iter().position(|id| id == transport_id) {
            self.cals[idx].add_receive_peer(uri.into());
        }
    }

    /// Begins forwarding messages between all connected CALs.
    ///
    /// For each topic in the service config (direction ≠ `Out`), wires a
    /// reader on every source CAL to writers on all other CALs.
    pub fn activate(&mut self) -> CalResult<()> {
        let topics: Vec<String> = self
            .config
            .get_service(&self.service_id)
            .map(|svc| {
                svc.topic
                    .iter()
                    .filter(|t| t.direction != TopicDirection::Out)
                    .map(|t| t.id.clone())
                    .collect()
            })
            .unwrap_or_default();

        let n = self.cals.len();

        if topics.is_empty() {
            slog::warn!(self.logger, "CalBridge: no topics configured, nothing to forward";
                "service_id" => &self.service_id);
            return Ok(());
        }
        if n < 2 {
            slog::warn!(self.logger, "CalBridge: fewer than 2 CALs, no forwarding possible";
                "cal_count" => n);
            return Ok(());
        }

        // Phase A: create one writer per (target_cal j, topic t).
        // writers[j][t] → the writer for topic `topics[t]` on `cals[j]`.
        let mut writers: Vec<Vec<Arc<Mutex<Box<dyn AbstractWriter<BridgeMsg>>>>>> =
            Vec::with_capacity(n);
        for j in 0..n {
            let mut row = Vec::with_capacity(topics.len());
            for topic in &topics {
                let w = self.cals[j].create_writer::<BridgeMsg>(topic, TopicQos::default())?;
                row.push(Arc::new(Mutex::new(w)));
            }
            writers.push(row);
        }

        // Phase B: for each source CAL i and topic t, create a reader that fans
        // messages out to all writers[j != i][t].
        for i in 0..n {
            for (t_idx, topic) in topics.iter().enumerate() {
                let targets: Vec<_> = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| Arc::clone(&writers[j][t_idx]))
                    .collect();
                let forwarder = Arc::new(TypedForwarder { targets });
                let mut reader =
                    self.cals[i].create_reader::<BridgeMsg>(topic, TopicQos::default())?;
                reader.add_listener(forwarder)?;
                self._keep_alive
                    .push(Box::new(reader) as Box<dyn Any + Send>);
            }
        }

        // Keep writers alive — dropping a writer closes the underlying topic connection.
        self._keep_alive.extend(
            writers
                .into_iter()
                .flatten()
                .map(|w| Box::new(w) as Box<dyn Any + Send>),
        );

        slog::info!(self.logger, "CalBridge activated";
            "cal_count" => n, "topic_count" => topics.len());
        Ok(())
    }

    /// Stops forwarding and releases all readers and writers.
    pub fn deactivate(&mut self) {
        self._keep_alive.clear();
        slog::info!(self.logger, "CalBridge deactivated");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// register_all_listeners
// ════════════════════════════════════════════════════════════════════════════

/// Registers a single callback for every topic in `service_id`'s config.
///
/// Iterates all topics in the `[[service]]` entry with the given `service_id`
/// (skipping `Out`-only topics) and calls `service.create_reader::<BridgeMsg>`
/// for each. The callback receives `(topic, &Arc<BridgeMsg>)`.
///
/// This is the service-side counterpart to [`CalBridge`]: while the bridge
/// forwards messages between transports, `register_all_listeners` lets a
/// service receive all its configured topics with a single handler.
pub fn register_all_listeners<A: AbstractCal>(
    service: &mut crate::service::AbstractServiceImpl<A>,
    config: &CalConfig,
    service_id: &str,
    qos: TopicQos,
    callback: impl Fn(&str, &Arc<BridgeMsg>) + Send + Sync + Clone + 'static,
) -> CalResult<()> {
    let topics: Vec<String> = config
        .get_service(service_id)
        .map(|svc| {
            svc.topic
                .iter()
                .filter(|t| t.direction != TopicDirection::Out)
                .map(|t| t.id.clone())
                .collect()
        })
        .unwrap_or_default();

    for topic in topics {
        let cb = callback.clone();
        // AbstractServiceImpl::create_reader callback is (msg, topic); we swap to (topic, msg).
        service.create_reader::<BridgeMsg, _>(&topic, qos.clone(), move |msg, t| cb(t, msg))?;
    }
    Ok(())
}

/// Convenience macro wrapping [`register_all_listeners`].
///
/// # Example
/// ```ignore
/// rcal::setup_all_listeners!(my_service, &config, "MySvc", TopicQos::default(),
///     |topic, msg| println!("{topic}"));
/// ```
#[macro_export]
macro_rules! setup_all_listeners {
    ($service:expr, $config:expr, $service_id:expr, $qos:expr, $callback:expr) => {
        $crate::bridge::register_all_listeners(&mut $service, $config, $service_id, $qos, $callback)
    };
}

// ════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ── Mock writer ───────────────────────────────────────────────────────────

    struct MockWriter {
        call_count: Arc<AtomicUsize>,
    }

    impl AbstractWriter<BridgeMsg> for MockWriter {
        fn topic(&self) -> &str {
            "test"
        }

        fn write(&mut self, _message: &BridgeMsg) -> CalResult<()> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn close(self: Box<Self>) -> CalResult<()> {
            Ok(())
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn forwarder_calls_every_writer_once_per_message() {
        let count_a = Arc::new(AtomicUsize::new(0));
        let count_b = Arc::new(AtomicUsize::new(0));

        let wa: Box<dyn AbstractWriter<BridgeMsg>> = Box::new(MockWriter {
            call_count: Arc::clone(&count_a),
        });
        let wb: Box<dyn AbstractWriter<BridgeMsg>> = Box::new(MockWriter {
            call_count: Arc::clone(&count_b),
        });

        let forwarder = TypedForwarder {
            targets: vec![Arc::new(Mutex::new(wa)), Arc::new(Mutex::new(wb))],
        };

        let msg = Arc::new(BridgeMsg::cal_create());
        forwarder.on_message(&msg);
        forwarder.on_message(&msg);

        assert_eq!(count_a.load(Ordering::Relaxed), 2, "writer A called twice");
        assert_eq!(count_b.load(Ordering::Relaxed), 2, "writer B called twice");
    }

    #[test]
    fn forwarder_with_no_targets_does_not_panic() {
        let forwarder = TypedForwarder { targets: vec![] };
        let msg = Arc::new(BridgeMsg::cal_create());
        forwarder.on_message(&msg);
    }

    #[test]
    fn bridge_msg_cal_create_is_empty_object() {
        let msg = BridgeMsg::cal_create();
        assert!(msg.0.is_object());
        assert!(msg.0.as_object().unwrap().is_empty());
    }
}
