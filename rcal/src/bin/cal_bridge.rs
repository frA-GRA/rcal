//! CAL bridge binary: repeats messages across two or more CAL transports.
//!
//! Reads `[bridge]` from `CALConfig.toml` (or `RCAL_CALCONFIG_PATH`):
//! ```toml
//! [bridge]
//! service    = "CalBridge"          # [[service]] id whose topics are bridged
//! transports = ["TransA", "TransB"] # two or more [[transport]] ids
//! ```
//!
//! Run with:
//! ```sh
//! cal_bridge [CONFIG_PATH]
//! ```
//! Ctrl-C stops the bridge and releases all resources.

use std::sync::Arc;

use rcal::asb::get_asb_config_location;
use rcal::bridge::CalBridge;
use rcal::calconfig::parse_config_from_file;
use rcal::logging::build_logger;
use rcal::uci::{CalError, CalErrorKind, CalResult};

#[derive(serde::Deserialize)]
struct BridgeConfig {
    service: String,
    transports: Vec<String>,
}

#[tokio::main]
async fn main() -> CalResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = if let Some(path) = args.get(1) {
        path.clone()
    } else {
        get_asb_config_location(None)?
    };

    let config = Arc::new(parse_config_from_file(&config_path)?);
    let logger = build_logger(&config.system.logging);

    slog::info!(logger, "cal_bridge starting"; "config" => &config_path);

    let bridge_cfg: BridgeConfig = config.get_extension("bridge").map_err(|e| {
        CalError::new(
            CalErrorKind::InitializationFailure,
            format!("missing [bridge] section in config: {e}"),
        )
    })?;

    if bridge_cfg.transports.len() < 2 {
        return Err(CalError::new(
            CalErrorKind::InitializationFailure,
            "bridge.transports must list at least 2 transport IDs",
        ));
    }

    let transport_ids: Vec<&str> = bridge_cfg.transports.iter().map(String::as_str).collect();

    slog::info!(logger, "bridge config";
        "service" => &bridge_cfg.service,
        "transports" => format!("{:?}", &bridge_cfg.transports));

    let mut bridge = CalBridge::new(
        Arc::clone(&config),
        bridge_cfg.service.clone(),
        &transport_ids,
        logger.clone(),
    )
    .await?;

    bridge.activate()?;

    tokio::signal::ctrl_c()
        .await
        .map_err(|e| CalError::new(CalErrorKind::InitializationFailure, e.to_string()))?;

    bridge.deactivate();
    slog::info!(logger, "cal_bridge stopped");
    Ok(())
}
