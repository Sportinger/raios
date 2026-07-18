use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ota_tools::{load_public_key_hex, SignedBlob};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(
    about = "Verify a signed module manifest against the offline root",
    version
)]
struct Args {
    /// Module binary path to verify against the manifest
    #[arg(long)]
    input: PathBuf,
    /// Signed manifest JSON produced by mod-sign
    #[arg(long)]
    signature: PathBuf,
    /// Offline root public key hex file
    #[arg(long)]
    root_pub: PathBuf,
    /// Expected module identifier (optional)
    #[arg(long)]
    expected_module_id: Option<String>,
    /// Expected module version (optional)
    #[arg(long)]
    expected_module_version: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let manifest: SignedBlob = serde_json::from_str(&std::fs::read_to_string(&args.signature)?)?;
    let root_public = load_public_key_hex(&args.root_pub)?;
    manifest.verify(&args.input, &root_public)?;

    if let Some(expected_id) = args.expected_module_id {
        let actual = extract_metadata_field(&manifest.metadata, "module_id")?;
        if actual.as_deref() != Some(expected_id.as_str()) {
            anyhow::bail!(
                "module_id mismatch: manifest={:?} expected={}",
                actual,
                expected_id
            );
        }
    }

    if let Some(expected_version) = args.expected_module_version {
        let actual = extract_metadata_field(&manifest.metadata, "module_version")?;
        if actual.as_deref() != Some(expected_version.as_str()) {
            anyhow::bail!(
                "module_version mismatch: manifest={:?} expected={}",
                actual,
                expected_version
            );
        }
    }

    println!("verification OK");
    Ok(())
}

fn extract_metadata_field(metadata: &Option<Value>, field: &str) -> Result<Option<String>> {
    if let Some(Value::Object(map)) = metadata {
        if let Some(Value::String(value)) = map.get(field) {
            return Ok(Some(value.clone()));
        }
        return Ok(None);
    }
    Ok(None)
}
