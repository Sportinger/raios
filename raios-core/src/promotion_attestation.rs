use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

pub const PROMOTION_AUTHORITY_IS_PLACEHOLDER: bool = true;

/*
SECURITY: DEVELOPMENT PROMOTION AUTHORITY KEY (not owner-sealed).

The matching private scalar is the known in-repo TEST fixture scalar 1, encoded
as 32-byte big-endian
0x0000000000000000000000000000000000000000000000000000000000000001, and is used
only by #[cfg(test)] signing code below. Because the private scalar is public,
anyone can forge a signature under this key — that is acceptable ONLY under the
development trust tier.

Ratified owner decision (ADR 0007, "M6B-2 Enforcement Precondition"): for
development this dev key is deliberately given full function — a signature that
verifies MAY drive a capability grant (M6B-2) and, in later slices, load/run —
but EVERY such authorization MUST carry trust_tier="dev_key_not_owner_sealed".
While PROMOTION_AUTHORITY_IS_PLACEHOLDER is true the system is NOT owner-sealed:
no owner-authority may be claimed. Replacing this key with the owner-generated
promotion key K (and flipping this flag) is the later sealing ceremony that
turns dev_key_not_owner_sealed into owner_sealed. Any slice that consumes a
grant for real load/promotion MUST emit the dev tier honestly until then.
*/
pub const PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1: &[u8] = &[
    0x04, 0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6, 0xe5, 0x63, 0xa4, 0x40,
    0xf2, 0x77, 0x03, 0x7d, 0x81, 0x2d, 0xeb, 0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8, 0x98, 0xc2,
    0x96, 0x4f, 0xe3, 0x42, 0xe2, 0xfe, 0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb, 0x4a, 0x7c, 0x0f, 0x9e,
    0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31, 0x5e, 0xce, 0xcb, 0xb6, 0x40, 0x68, 0x37, 0xbf, 0x51,
    0xf5,
];

pub const PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256: [u8; 32] = [
    0x69, 0x8b, 0xea, 0x63, 0xdc, 0x44, 0xa3, 0x44, 0x66, 0x3f, 0xf1, 0x42, 0x9a, 0xea, 0x10, 0x84,
    0x2d, 0xf2, 0x7b, 0x6b, 0x99, 0x1e, 0xf2, 0x58, 0x66, 0xb2, 0xc6, 0xc0, 0x2c, 0xdc, 0xc5, 0xbe,
];

pub fn verify_promotion_authority_signature(
    signature_der: &[u8],
    canonical_payload: &[u8],
) -> bool {
    let Ok(verifying_key) =
        VerifyingKey::from_sec1_bytes(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1)
    else {
        return false;
    };
    let Ok(signature) = Signature::from_der(signature_der) else {
        return false;
    };
    verifying_key.verify(canonical_payload, &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1,
        PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    };
    use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
    use sha2::{Digest, Sha256};

    const PLACEHOLDER_PROMOTION_AUTHORITY_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];
    const OTHER_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02,
    ];

    fn signing_key(scalar: &[u8; 32]) -> SigningKey {
        SigningKey::from_slice(scalar).expect("test scalar must be valid")
    }

    fn sign_with(scalar: &[u8; 32], payload: &[u8]) -> Signature {
        signing_key(scalar).sign(payload)
    }

    #[test]
    fn deterministic_roundtrip_succeeds_for_pinned_placeholder_key() {
        let payload = Sha256::digest(b"sample canonical local-attestation reference").to_vec();
        let signature = sign_with(&PLACEHOLDER_PROMOTION_AUTHORITY_PRIVATE_SCALAR, &payload);

        assert!(verify_promotion_authority_signature(
            signature.to_der().as_bytes(),
            &payload
        ));
    }

    #[test]
    fn tampered_payload_fails() {
        let payload = Sha256::digest(b"sample canonical local-attestation reference").to_vec();
        let signature = sign_with(&PLACEHOLDER_PROMOTION_AUTHORITY_PRIVATE_SCALAR, &payload);
        let mut tampered = payload.clone();
        tampered[0] ^= 0x01;

        assert!(!verify_promotion_authority_signature(
            signature.to_der().as_bytes(),
            &tampered
        ));
    }

    #[test]
    fn different_key_signature_fails() {
        let payload = Sha256::digest(b"sample canonical local-attestation reference").to_vec();
        let signature = sign_with(&OTHER_PRIVATE_SCALAR, &payload);

        assert!(!verify_promotion_authority_signature(
            signature.to_der().as_bytes(),
            &payload
        ));
    }

    #[test]
    fn malformed_der_fails_without_panic() {
        let payload = Sha256::digest(b"sample canonical local-attestation reference").to_vec();

        assert!(!verify_promotion_authority_signature(b"not-der", &payload));
    }

    #[test]
    fn pinned_public_key_parses_and_matches_self_pin() {
        let _key = VerifyingKey::from_sec1_bytes(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1)
            .expect("placeholder SEC1 public key must parse");
        let digest = Sha256::digest(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1);

        assert_eq!(
            digest.as_slice(),
            PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
        );
    }
}
