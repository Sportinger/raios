//! Sign raiOS Wasm descriptor files with the public dev scalar-1 key.
//!
//! This signs the raw descriptor file bytes. The build verifier hashes the same
//! payload internally through p256's `Verifier::verify`; do not pre-hash here.

use std::{env, fs, process};

use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};

const DEV_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

fn sign_descriptor(path: &str) -> Result<(String, String), String> {
    let file_bytes = fs::read(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let key = SigningKey::from_slice(&DEV_PRIVATE_SCALAR)
        .map_err(|e| format!("dev scalar 1 is invalid: {e}"))?;
    let sig: Signature = key.sign(&file_bytes);
    let pubkey = VerifyingKey::from(&key);
    Ok((
        hex::encode(sig.to_der().as_bytes()),
        hex::encode(pubkey.to_encoded_point(false).as_bytes()),
    ))
}

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: wasm-descriptor-signer <path-to-.desc>");
        process::exit(2);
    };
    if env::args().nth(2).is_some() {
        eprintln!("usage: wasm-descriptor-signer <path-to-.desc>");
        process::exit(2);
    }

    match sign_descriptor(&path) {
        Ok((sig, pubkey)) => {
            println!("signature_der_hex={sig}");
            println!("public_key_sec1_hex={pubkey}");
        }
        Err(e) => {
            eprintln!("wasm-descriptor-signer: {e}");
            process::exit(1);
        }
    }
}
