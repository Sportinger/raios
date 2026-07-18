use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use ota_tools::{load_certificate, load_keymaterial, SignedBlob};

#[derive(Parser, Debug)]
#[command(about = "Sign an OTA or module blob with the online signer", version)]
struct Args {
    /// Path to the online signer key JSON (with secret)
    #[arg(long)]
    key: PathBuf,
    /// Path to the signer certificate signed by the offline root
    #[arg(long)]
    cert: PathBuf,
    /// Blob to sign
    #[arg(long)]
    input: PathBuf,
    /// Output signed metadata JSON
    #[arg(long)]
    output: PathBuf,
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
    let signed = SignedBlob::sign(&args.input, &key, &cert, None)?;
    let json = serde_json::to_string_pretty(&signed)?;
    std::fs::write(&args.output, json)?;
    println!("wrote signature manifest to {}", args.output.display());
    Ok(())
}
