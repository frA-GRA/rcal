use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

pub const BROKER_HOST: &str = "localhost";
pub const BROKER_PORT: u16 = 61613;

// Weak so the Arc refcount drives Drop; no strong ref kept after all tests release.
static BROKER: Mutex<Weak<BrokerGuard>> = Mutex::new(Weak::new());

/// RAII guard that ensures an ActiveMQ broker is running.
/// If no broker is detected, starts one via docker or podman and stops it on drop.
/// Shared via Arc so the container starts exactly once per test binary.
pub struct BrokerGuard {
    runtime: Option<String>,
    container_id: Option<String>,
}

impl BrokerGuard {
    pub fn require() -> Arc<Self> {
        let mut lock = BROKER.lock().unwrap();
        if let Some(arc) = lock.upgrade() {
            return arc;
        }
        let arc = Arc::new(Self::start());
        *lock = Arc::downgrade(&arc);
        arc
    }

    fn start() -> Self {
        if TcpStream::connect((BROKER_HOST, BROKER_PORT)).is_ok() {
            return Self {
                runtime: None,
                container_id: None,
            };
        }

        let runtime = find_runtime().expect(
            "ActiveMQ broker not reachable and neither docker nor podman found. \
             Start a broker (e.g. `docker run -p 61613:61613 apache/activemq-classic`) \
             or install docker/podman before running these tests.",
        );

        let output = Command::new(&runtime)
            .args([
                "run",
                "-d",
                "-p",
                &format!("{BROKER_PORT}:{BROKER_PORT}"),
                "apache/activemq-classic",
            ])
            .output()
            .unwrap_or_else(|e| panic!("Failed to start broker via {runtime}: {e}"));

        assert!(
            output.status.success(),
            "Failed to start broker container: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        wait_for_broker(BROKER_HOST, BROKER_PORT, Duration::from_secs(60))
            .unwrap_or_else(|| panic!("Broker container {container_id} started but port {BROKER_PORT} never became reachable"));

        Self {
            runtime: Some(runtime),
            container_id: Some(container_id),
        }
    }
}

impl Drop for BrokerGuard {
    fn drop(&mut self) {
        if let (Some(runtime), Some(id)) = (&self.runtime, &self.container_id) {
            let _ = Command::new(runtime).args(["stop", id]).output();
            let _ = Command::new(runtime).args(["rm", id]).output();
        }
    }
}

fn find_runtime() -> Option<String> {
    for rt in ["docker", "podman"] {
        if Command::new(rt)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(rt.to_string());
        }
    }
    None
}

fn wait_for_broker(host: &str, port: u16, timeout: Duration) -> Option<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect((host, port)).is_ok() {
            return Some(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    None
}
