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

use p256::ecdsa::{signature::Signer, Signature, SigningKey};

/// The known DEV distribution publisher scalar = 2. Public because the DEV key is public.
const DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
];

/// MUST byte-match `raios_core::distribution_provenance::DISTRIBUTION_PROVENANCE_DOMAIN_TAG`.
const DISTRIBUTION_PROVENANCE_DOMAIN_TAG: &[u8] = b"raios.distribution_provenance.v0";

fn sign_distribution_provenance(artifact_sha256: &[u8; 32]) -> String {
    let signing_key = SigningKey::from_slice(&DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR)
        .expect("dev scalar 2 is a valid key");
    let mut message = [0u8; DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len() + 32];
    message[..DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()]
        .copy_from_slice(DISTRIBUTION_PROVENANCE_DOMAIN_TAG);
    message[DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()..].copy_from_slice(artifact_sha256);
    let signature: Signature = signing_key.sign(&message);
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
        Ok(hash) => println!("{}", sign_distribution_provenance(&hash)),
        Err(e) => {
            eprintln!("distribution-provenance-signer: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        sign_distribution_provenance, DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
        DISTRIBUTION_PROVENANCE_DOMAIN_TAG,
    };
    use raios_core::distribution_provenance::verify_distribution_provenance_signature;
    use raios_core::promotion_attestation::verify_promotion_authority_signature;

    #[test]
    fn signer_domain_tag_matches_core() {
        assert_eq!(
            DISTRIBUTION_PROVENANCE_DOMAIN_TAG,
            raios_core::distribution_provenance::DISTRIBUTION_PROVENANCE_DOMAIN_TAG
        );
    }

    #[test]
    fn signer_output_verifies_as_provenance_not_promotion() {
        let hash: [u8; 32] = [7u8; 32];
        let der = hex::decode(sign_distribution_provenance(&hash)).expect("output is hex");

        assert!(verify_distribution_provenance_signature(&der, &hash));
        assert!(!verify_promotion_authority_signature(&der, &hash));
    }

    #[test]
    fn deterministic_output_is_stable() {
        let hash: [u8; 32] = [0x42u8; 32];
        assert_eq!(
            sign_distribution_provenance(&hash),
            sign_distribution_provenance(&hash),
            "RFC6979 signing must be deterministic"
        );
    }

    #[test]
    fn dev_distribution_publisher_scalar_is_two() {
        assert_eq!(DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR[31], 2);
        assert!(DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR[..31]
            .iter()
            .all(|b| *b == 0));
    }
}
