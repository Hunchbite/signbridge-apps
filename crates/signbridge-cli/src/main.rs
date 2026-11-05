//! `signbridge` — reference CLI built on the SignBridge framework.
//!
//! Demonstrates how an application embeds the library without the browser host.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use clap::{Parser, Subcommand};
use serde_json::json;
use sha2::{Digest, Sha256};
use signbridge::protocol::{HashAlg, Request, ResponseBody, SignHashParams};
use signbridge::{AppConfig, AutoConfirm, SignService};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "signbridge", version, about = "SignBridge framework CLI")]
struct Cli {
    /// Optional JSON config (allowlist + PKCS#11 modules).
    #[arg(long, env = "SIGNBRIDGE_CONFIG")]
    config: Option<PathBuf>,

    /// PIN for the token (or set SIGNBRIDGE_PIN).
    #[arg(long, env = "SIGNBRIDGE_PIN")]
    pin: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print framework name + version (same as protocol `ping`).
    Ping,
    /// List certificates / keys from configured PKCS#11 modules.
    List,
    /// Sign a SHA-256 digest (hex) or the hash of --message / --file.
    Sign {
        /// Certificate id from `signbridge list`.
        #[arg(long)]
        certificate_id: String,
        /// Hex digest (64 chars for SHA-256). Mutually exclusive with --message/--file.
        #[arg(long)]
        digest_hex: Option<String>,
        /// UTF-8 message to hash then sign.
        #[arg(long)]
        message: Option<String>,
        /// File to hash then sign.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Document title shown to confirm hooks.
        #[arg(long)]
        title: Option<String>,
        /// Emit raw JSON protocol response.
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("signbridge=info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let mut config = AppConfig::load_or_default(cli.config.as_deref())?;
    config.skip_origin_check = true;
    if let Some(pin) = cli.pin.clone() {
        config.test_pin = Some(pin.clone());
    }

    let pin = cli
        .pin
        .or_else(|| std::env::var("SIGNBRIDGE_PIN").ok())
        .or_else(|| config.test_pin.clone());

    let service = SignService::new(config, AutoConfirm { pin });

    match cli.command {
        Command::Ping => {
            let resp = service.handle(Request::Ping { id: "cli".into() });
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::List => {
            let resp = service.handle(Request::ListCertificates {
                id: "cli".into(),
                origin: "cli://signbridge".into(),
            });
            if !resp.ok {
                bail!("{}", resp.error.map(|e| e.message).unwrap_or_default());
            }
            match resp.result {
                Some(ResponseBody::Certificates(c)) => {
                    if c.certificates.is_empty() {
                        println!("(no certificates — install OpenSC / SoftHSM, or plug in a token)");
                    }
                    for cert in c.certificates {
                        println!("{}", cert.id);
                        println!("  subject: {}", cert.subject);
                        println!("  token:   {}", cert.token_label.unwrap_or_default());
                        println!("  algo:    {}", cert.key_algo.unwrap_or_default());
                        println!();
                    }
                }
                _ => bail!("unexpected response"),
            }
        }
        Command::Sign {
            certificate_id,
            digest_hex,
            message,
            file,
            title,
            json,
        } => {
            let digest_hex = resolve_digest(digest_hex, message, file)?;
            let resp = service.handle(Request::SignHash {
                id: "cli".into(),
                origin: "cli://signbridge".into(),
                params: SignHashParams {
                    certificate_id,
                    hash_alg: HashAlg::Sha256,
                    digest_hex,
                    document_title: title,
                    document_id: None,
                },
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                if !resp.ok {
                    std::process::exit(1);
                }
                return Ok(());
            }
            if !resp.ok {
                bail!("{}", resp.error.map(|e| e.message).unwrap_or_default());
            }
            match resp.result {
                Some(ResponseBody::Sign(s)) => {
                    println!("{}", s.signature_b64);
                    eprintln!(
                        "{}",
                        json!({
                            "certificateId": s.certificate_id,
                            "hashAlg": "SHA-256",
                            "signatureBytes": B64.decode(&s.signature_b64).map(|b| b.len()).unwrap_or(0),
                        })
                    );
                }
                _ => bail!("unexpected response"),
            }
        }
    }
    Ok(())
}

fn resolve_digest(
    digest_hex: Option<String>,
    message: Option<String>,
    file: Option<PathBuf>,
) -> Result<String> {
    match (digest_hex, message, file) {
        (Some(h), None, None) => Ok(h),
        (None, Some(m), None) => Ok(hex::encode(Sha256::digest(m.as_bytes()))),
        (None, None, Some(path)) => {
            let bytes = std::fs::read(&path)
                .with_context(|| format!("read {}", path.display()))?;
            Ok(hex::encode(Sha256::digest(&bytes)))
        }
        _ => bail!("provide exactly one of --digest-hex, --message, or --file"),
    }
}
