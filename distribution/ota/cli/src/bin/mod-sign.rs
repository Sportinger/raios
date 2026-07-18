use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use ota_tools::{load_certificate, load_keymaterial, SignedBlob};
use serde_json::{Map, Value};

#[derive(Parser, Debug)]
#[command(about = "Sign a Wasm module bundle with module metadata", version)]
struct Args {
    /// Path to the online signer key JSON (with secret)
    #[arg(long)]
    key: PathBuf,
    /// Path to the signer certificate signed by the offline root
    #[arg(long)]
    cert: PathBuf,
    /// Module binary to sign (e.g. Wasm)
    #[arg(long)]
    input: PathBuf,
    /// Output manifest JSON
    #[arg(long)]
    output: PathBuf,
    /// Logical module identifier included in metadata
    #[arg(long)]
    module_id: Option<String>,
    /// Module semantic version included in metadata
    #[arg(long)]
    module_version: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let key = load_keymaterial(&args.key)?;
    let cert = load_certificate(&args.cert)?;
    if cert.subject != key.key_id {
        return Err(anyhow!(
            "certificate subject {} != key id {}",
            cert.subject,
            key.key_id
        ));
    }

    let mut metadata = Map::new();
    if let Some(id) = args.module_id {
        metadata.insert("module_id".to_string(), Value::String(id));
    }
    if let Some(version) = args.module_version {
        metadata.insert("module_version".to_string(), Value::String(version));
    }
    let metadata_value = if metadata.is_empty() {
        None
    } else {
        Some(Value::Object(metadata))
    };

    let signed = SignedBlob::sign(&args.input, &key, &cert, metadata_value)?;
    let json = serde_json::to_string_pretty(&signed)?;
    std::fs::write(&args.output, json)?;
    println!(
        "wrote module signature manifest to {}",
        args.output.display()
    );
    Ok(())
}
