use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ed25519_dalek::{PublicKey, PUBLIC_KEY_LENGTH};
use hex::decode;
use ota_tools::SignedBlob;

#[derive(Parser, Debug)]
#[command(about = "Verify a signed OTA/blob against the offline root")] 
struct Args {
    /// Original blob content
    #[arg(long)]
    input: PathBuf,
    /// Signed manifest JSON produced by ota-sign
    #[arg(long)]
    signature: PathBuf,
    /// Offline root public key hex file
    #[arg(long)]
    root_pub: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let manifest: SignedBlob = serde_json::from_str(&std::fs::read_to_string(&args.signature)?)?;
    let root_hex = std::fs::read_to_string(&args.root_pub)?.trim().to_string();
    let root_bytes = decode(root_hex)?;
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
    println!("verification OK");
    Ok(())
}
