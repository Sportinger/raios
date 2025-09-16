use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ed25519_dalek::{PublicKey, PUBLIC_KEY_LENGTH};
use hex::decode;
use ota_tools::SignedBlob;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(about = "Verify a signed module manifest against the offline root", version)]
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
    let root_hex = std::fs::read_to_string(&args.root_pub)?.trim().to_string();
    let root_bytes = decode(&root_hex)?;
    if root_bytes.len() != PUBLIC_KEY_LENGTH {
        anyhow::bail!(
            "root key length {} invalid (expected {})",
            root_bytes.len(),
            PUBLIC_KEY_LENGTH
        );
    }
    let mut root_array = [0u8; PUBLIC_KEY_LENGTH];
    root_array.copy_from_slice(&root_bytes);
    let root_public = PublicKey::from_bytes(&root_array)?;
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
