//! SignBridge native messaging host — a thin app on top of the framework.
//!
//! Speaks Chrome/Firefox native messaging on stdin/stdout:
//! 4-byte little-endian length prefix + UTF-8 JSON payload.

mod confirm;
mod native_messaging;

use anyhow::{Context, Result};
use confirm::NativeConfirm;
use native_messaging::{read_message, write_message};
use signbridge::protocol::Request;
use signbridge::{AppConfig, SignService};
use std::io::{stdin, stdout};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

fn main() {
    if let Err(e) = run() {
        eprintln!("signbridge-host fatal: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("signbridge=info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let config_path = std::env::var_os("SIGNBRIDGE_CONFIG").map(PathBuf::from);
    let config =
        AppConfig::load_or_default(config_path.as_deref()).context("load SignBridge config")?;

    info!(
        version = signbridge::VERSION,
        modules = config.pkcs11_modules.len(),
        "signbridge-host starting (framework app)"
    );

    let service = SignService::new(config, NativeConfirm);
    let mut input = stdin().lock();
    let mut output = stdout().lock();

    loop {
        let raw = match read_message(&mut input) {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                info!("stdin closed; exiting");
                break;
            }
            Err(e) => {
                error!(error = %e, "failed to read native message");
                break;
            }
        };

        let response = match serde_json::from_slice::<Request>(&raw) {
            Ok(req) => service.handle(req),
            Err(e) => signbridge::protocol::Response::err(
                "0",
                "protocol",
                format!("invalid request: {e}"),
            ),
        };

        let encoded = serde_json::to_vec(&response)?;
        write_message(&mut output, &encoded)?;
    }

    Ok(())
}
