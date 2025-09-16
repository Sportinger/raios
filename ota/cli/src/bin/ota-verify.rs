use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ota_tools::{load_public_key_hex, SignedBlob};

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
    let root_public = load_public_key_hex(&args.root_pub)?;
    manifest.verify(&args.input, &root_public)?;
    println!("verification OK");
    Ok(())
}
