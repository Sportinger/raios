//! dev-promotion-signer — reliable host signer for the raiOS DEV promotion-authority key.
//!
//! Produces a P-256 ECDSA signature (RFC6979-deterministic, DER-encoded, lowercase hex) over a
//! 32-byte `attestation_reference_hash`, using the KNOWN in-repo DEV private scalar 1. This is
//! byte-identical to what `raios_core::promotion_attestation::verify_promotion_authority_signature`
//! verifies against (same p256 crate, same curve, same deterministic signing), so it drives a REAL
//! `module.attestation_diagnostic` signature-verify + a real durable promotion transaction on the
//! two-boot (M7D-2) proof.
//!
//! This is the DEV key, deliberately public and NOT owner-sealed: `PROMOTION_AUTHORITY_IS_PLACEHOLDER`
//! stays `true`; the owner's real key K replaces it at the sealing ceremony. The existing Windows
//! PowerShell 5.1 / .NET Framework scalar-1 signer is documented-unreliable (its output frequently
//! fails to verify), which is why the real-promotion flow shells out to this deterministic Rust signer.
//!
//! Usage:
//!   dev-promotion-signer <64-hex-attestation-reference-hash>
//!   echo <64-hex-...> | dev-promotion-signer
//! Output: the lowercase DER-hex signature on stdout (single line).

use std::io::Read;

use p256::ecdsa::{signature::Signer, Signature, SigningKey};

/// The known DEV private scalar = 1 (31 zero bytes followed by 0x01). Public because the DEV key is
/// public; mirrors `PLACEHOLDER_PROMOTION_AUTHORITY_PRIVATE_SCALAR` in raios-core's attestation tests.
const DEV_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

/// Sign the 32-byte attestation_reference_hash with the DEV key; return lowercase DER-hex.
fn sign_attestation_reference_hash(hash32: &[u8; 32]) -> String {
    let signing_key = SigningKey::from_slice(&DEV_PRIVATE_SCALAR).expect("dev scalar 1 is a valid key");
    // `sign` hashes the message with SHA-256 internally (RFC6979-deterministic); the verifier does
    // the same over the identical 32-byte payload, so this is byte-identical to the kernel's verify.
    let signature: Signature = signing_key.sign(hash32);
    hex::encode(signature.to_der().as_bytes())
}

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
        .map_err(|e| format!("attestation_reference_hash is not valid hex: {e}"))?;
    <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| format!("attestation_reference_hash must be exactly 32 bytes, got {}", bytes.len()))
}

fn main() {
    match read_hash_input() {
        Ok(hash) => println!("{}", sign_attestation_reference_hash(&hash)),
        Err(e) => {
            eprintln!("dev-promotion-signer: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{sign_attestation_reference_hash, DEV_PRIVATE_SCALAR};
    use raios_core::promotion_attestation::verify_promotion_authority_signature;

    #[test]
    fn signer_output_verifies_against_the_pinned_dev_public_key() {
        let hash: [u8; 32] = [7u8; 32];
        let der = hex::decode(sign_attestation_reference_hash(&hash)).expect("output is hex");
        assert!(verify_promotion_authority_signature(&der, &hash));
    }

    #[test]
    fn signature_does_not_verify_over_a_different_hash() {
        let hash: [u8; 32] = [7u8; 32];
        let der = hex::decode(sign_attestation_reference_hash(&hash)).unwrap();
        let mut other = hash;
        other[0] ^= 0x01;
        assert!(!verify_promotion_authority_signature(&der, &other));
    }

    #[test]
    fn deterministic_output_is_stable() {
        let hash: [u8; 32] = [0x42u8; 32];
        assert_eq!(
            sign_attestation_reference_hash(&hash),
            sign_attestation_reference_hash(&hash),
            "RFC6979 signing must be deterministic"
        );
    }

    #[test]
    fn dev_scalar_is_one() {
        assert_eq!(DEV_PRIVATE_SCALAR[31], 1);
        assert!(DEV_PRIVATE_SCALAR[..31].iter().all(|b| *b == 0));
    }
}
