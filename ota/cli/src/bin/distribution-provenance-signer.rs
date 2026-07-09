//! distribution-provenance-signer - host signer for the raiOS DEV publisher key.
//!
//! Produces a P-256 ECDSA signature (RFC6979-deterministic, DER-encoded,
//! lowercase hex) over `DISTRIBUTION_PROVENANCE_DOMAIN_TAG ++ artifact_sha256`,
//! using the known public DEV private scalar 2. This is provenance only, never
//! load or install authority.
//!
//! Usage:
//!   distribution-provenance-signer <64-hex-artifact-sha256>
//!   echo <64-hex-...> | distribution-provenance-signer
//! Output: the lowercase DER-hex signature on stdout (single line).

use std::io::Read;

use ota_tools::sign_distribution_provenance_hex;

fn read_hash_input() -> Result<[u8; 32], String> {
    let hex_str = match std::env::args().nth(1) {
        Some(arg) => arg,
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("failed to read stdin: {e}"))?;
            buf
        }
    };
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| format!("artifact_sha256 is not valid hex: {e}"))?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
        format!(
            "artifact_sha256 must be exactly 32 bytes, got {}",
            bytes.len()
        )
    })
}

fn main() {
    match read_hash_input() {
        Ok(hash) => println!("{}", sign_distribution_provenance_hex(&hash)),
        Err(e) => {
            eprintln!("distribution-provenance-signer: {e}");
            std::process::exit(1);
        }
    }
}
