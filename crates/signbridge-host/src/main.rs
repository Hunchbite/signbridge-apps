//! SignBridge native messaging host — thin app on the language-agnostic protocol.
//!
//! Same JSON as `signbridge-rpc --framed` (Chrome/Firefox native messaging).

mod confirm;

use anyhow::{Context, Result};
use confirm::NativeConfirm;
use signbridge::ipc::{serve_stdio, Framing};
use signbridge::{AppConfig, SignService};
use std::io::{stdin, stdout};
use std::path::PathBuf;
use tracing::info;
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
        "signbridge-host starting"
    );

    let service = SignService::new(config, NativeConfirm);
    serve_stdio(
        &service,
        Framing::NativeMessage,
        stdin().lock(),
        stdout().lock(),
    )?;
    Ok(())
}
