use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ota_tools::{ensure_dir, store_certificate, KeyMaterial, SignerCertificate};

#[derive(Parser, Debug)]
#[command(about = "Generate deterministic seed OS signing keys and certificate", version)]
struct Args {
    /// Output directory for generated keys and certificates
    #[arg(long, default_value = "keys/dev")] // relative to repo root
    output: PathBuf,
    /// Identifier for the offline root key
    #[arg(long, default_value = "seed-root")]
    root_id: String,
    /// Identifier for the online signing key
    #[arg(long, default_value = "seed-online")]
    online_id: String,
    /// Context string mixed into deterministic key derivation
    #[arg(long, default_value = "seed-os/keys/v1")]
    context: String,
    /// Issued-at timestamp in Unix milliseconds
    #[arg(long, default_value_t = 1_700_000_000_000u64)]
    issued_at_ms: u64,
    /// Lifetime for the signer certificate in milliseconds
    #[arg(long, default_value_t = 7 * 24 * 60 * 60 * 1000u64)]
    ttl_ms: u64,
    /// Skip overwriting existing key material
    #[arg(long)]
    no_overwrite: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output_dir = args.output;
    ensure_dir(&output_dir)?;

    let root_key_path = output_dir.join("root-key.json");
    let root_pub_path = output_dir.join("root.pub");
    let online_key_path = output_dir.join("online-key.json");
    let online_pub_path = output_dir.join("online.pub");
    let cert_path = output_dir.join("online.cert.json");

    if args.no_overwrite {
        for p in [&root_key_path, &online_key_path, &cert_path] {
            if p.exists() {
                eprintln!("Skipping existing artifact {}", p.display());
                return Ok(());
            }
        }
    }

    let root_material = KeyMaterial::from_seed(&args.root_id, &args.context)?;
    root_material.save(&root_key_path)?;
    std::fs::write(&root_pub_path, format!("{}\n", root_material.public_key))?;

    let online_material = KeyMaterial::from_seed(&args.online_id, &args.context)?;
    online_material.save(&online_key_path)?;
    std::fs::write(&online_pub_path, format!("{}\n", online_material.public_key))?;

    let cert = SignerCertificate::new(
        &online_material,
        &root_material,
        args.issued_at_ms,
        args.ttl_ms,
    )?;
    store_certificate(&cert, &cert_path)?;

    println!("root key: {}", root_key_path.display());
    println!("root pub: {}", root_pub_path.display());
    println!("online key: {}", online_key_path.display());
    println!("online pub: {}", online_pub_path.display());
    println!("certificate: {}", cert_path.display());

    Ok(())
}
