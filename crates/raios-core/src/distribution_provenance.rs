use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

pub const DISTRIBUTION_PUBLISHER_IS_PLACEHOLDER: bool = true;
pub const DISTRIBUTION_PROVENANCE_DOMAIN_TAG: &[u8] = b"raios.distribution_provenance.v0";
const DISTRIBUTION_PROVENANCE_MESSAGE_LEN: usize = DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len() + 32;

/*
SECURITY: DEVELOPMENT DISTRIBUTION PUBLISHER KEY (not owner-sealed).

The matching private scalar is the known in-repo TEST fixture scalar 2, encoded
as 32-byte big-endian
0x0000000000000000000000000000000000000000000000000000000000000002. This is
a separate publisher/provenance key from the promotion authority key.

Because the private scalar is public, anyone can forge a signature under this
key. That is acceptable ONLY under trust_tier="dev_key_not_owner_sealed".
Distribution provenance authenticates who published candidate bytes; it grants
NO load, install, acquisition, persistence, promotion, or owner authority.

While DISTRIBUTION_PUBLISHER_IS_PLACEHOLDER is true the system is NOT
owner-sealed. Replacing this key with an owner-held publisher key is a later
owner sealing/key-custody ceremony (ADR 0009 Q3). Provenance remains origin
evidence only and never substitutes for the M6/M7 load-worthiness chain.
*/
pub const PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SEC1: &[u8] = &[
    0x04, 0x7c, 0xf2, 0x7b, 0x18, 0x8d, 0x03, 0x4f, 0x7e, 0x8a, 0x52, 0x38, 0x03, 0x04, 0xb5, 0x1a,
    0xc3, 0xc0, 0x89, 0x69, 0xe2, 0x77, 0xf2, 0x1b, 0x35, 0xa6, 0x0b, 0x48, 0xfc, 0x47, 0x66, 0x99,
    0x78, 0x07, 0x77, 0x55, 0x10, 0xdb, 0x8e, 0xd0, 0x40, 0x29, 0x3d, 0x9a, 0xc6, 0x9f, 0x74, 0x30,
    0xdb, 0xba, 0x7d, 0xad, 0xe6, 0x3c, 0xe9, 0x82, 0x29, 0x9e, 0x04, 0xb7, 0x9d, 0x22, 0x78, 0x73,
    0xd1,
];

pub const PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256: [u8; 32] = [
    0xa9, 0xf3, 0x00, 0xeb, 0x59, 0x60, 0xe8, 0x91, 0x33, 0xaf, 0x73, 0x62, 0x01, 0x1a, 0x1e, 0x26,
    0xf0, 0xe2, 0xea, 0x2e, 0x36, 0xdc, 0x40, 0x2a, 0x04, 0xaf, 0x6c, 0x19, 0x2b, 0x89, 0x1a, 0x8c,
];

fn distribution_provenance_message(
    artifact_sha256: &[u8; 32],
) -> [u8; DISTRIBUTION_PROVENANCE_MESSAGE_LEN] {
    let mut message = [0u8; DISTRIBUTION_PROVENANCE_MESSAGE_LEN];
    message[..DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()]
        .copy_from_slice(DISTRIBUTION_PROVENANCE_DOMAIN_TAG);
    message[DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()..].copy_from_slice(artifact_sha256);
    message
}

pub fn verify_distribution_provenance_signature(
    signature_der: &[u8],
    artifact_sha256: &[u8; 32],
) -> bool {
    let Ok(verifying_key) =
        VerifyingKey::from_sec1_bytes(PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SEC1)
    else {
        return false;
    };
    let Ok(signature) = Signature::from_der(signature_der) else {
        return false;
    };
    verifying_key
        .verify(
            &distribution_provenance_message(artifact_sha256),
            &signature,
        )
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        distribution_provenance_message, verify_distribution_provenance_signature,
        DISTRIBUTION_PROVENANCE_DOMAIN_TAG, PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SEC1,
        PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    };
    use crate::promotion_attestation::verify_promotion_authority_signature;
    use crate::sha256_bytes;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};

    const PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02,
    ];
    const PROMOTION_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];

    fn signing_key(scalar: &[u8; 32]) -> SigningKey {
        SigningKey::from_slice(scalar).expect("test scalar must be valid")
    }

    fn sign_with(scalar: &[u8; 32], payload: &[u8]) -> Signature {
        signing_key(scalar).sign(payload)
    }

    #[test]
    fn domain_tag_is_exactly_32_bytes() {
        assert_eq!(DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len(), 32);
    }

    #[test]
    fn pinned_publisher_key_parses_and_matches_self_pin() {
        let verifying_key = VerifyingKey::from(&signing_key(
            &PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
        ));
        let encoded = verifying_key.to_encoded_point(false);

        assert_eq!(
            encoded.as_bytes(),
            PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SEC1
        );
        assert_eq!(
            sha256_bytes(PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SEC1),
            PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256
        );
    }

    #[test]
    fn roundtrip_scalar2_over_domain_tagged_message_verifies() {
        let artifact_hash = sha256_bytes(b"sample artifact bytes");
        let signature = sign_with(
            &PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
            &distribution_provenance_message(&artifact_hash),
        );

        assert!(verify_distribution_provenance_signature(
            signature.to_der().as_bytes(),
            &artifact_hash
        ));
    }

    #[test]
    fn tampered_artifact_hash_fails() {
        let artifact_hash = sha256_bytes(b"sample artifact bytes");
        let signature = sign_with(
            &PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
            &distribution_provenance_message(&artifact_hash),
        );
        let mut tampered = artifact_hash;
        tampered[0] ^= 0x01;

        assert!(!verify_distribution_provenance_signature(
            signature.to_der().as_bytes(),
            &tampered
        ));
    }

    #[test]
    fn promotion_authority_scalar1_signature_is_rejected() {
        let artifact_hash = sha256_bytes(b"sample artifact bytes");
        let signature = sign_with(
            &PROMOTION_PRIVATE_SCALAR,
            &distribution_provenance_message(&artifact_hash),
        );

        assert!(!verify_distribution_provenance_signature(
            signature.to_der().as_bytes(),
            &artifact_hash
        ));
    }

    #[test]
    fn promotion_domain_bare_payload_signature_is_rejected() {
        let artifact_hash = sha256_bytes(b"sample artifact bytes");
        let bare_signature = sign_with(
            &PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
            &artifact_hash,
        );

        assert!(!verify_distribution_provenance_signature(
            bare_signature.to_der().as_bytes(),
            &artifact_hash
        ));

        let provenance_signature = sign_with(
            &PLACEHOLDER_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
            &distribution_provenance_message(&artifact_hash),
        );

        assert!(!verify_promotion_authority_signature(
            provenance_signature.to_der().as_bytes(),
            &artifact_hash
        ));
    }

    #[test]
    fn malformed_der_fails_without_panic() {
        let artifact_hash = sha256_bytes(b"sample artifact bytes");

        assert!(!verify_distribution_provenance_signature(
            b"not-der",
            &artifact_hash
        ));
    }
}
