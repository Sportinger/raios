use crate::{
    artifact_blob_frame::parse_artifact_blob_frame,
    promotion_attestation::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    },
    sha256_bytes,
};

#[derive(Clone, Copy)]
pub struct RepromotionTransactionFields<'a> {
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub attestation_reference_hash: [u8; 32],
    pub promotion_signature_der: &'a [u8],
    pub promotion_authority_key_sha256: [u8; 32],
    pub signature_verified: bool,
    pub grant_binds_capability: bool,
    pub install_authorization_present: bool,
    pub install_envelope_binds_activation: bool,
    pub install_action_signature_message_sha256: [u8; 32],
    pub install_action_signature_der: &'a [u8],
    pub install_authority_key_sha256: [u8; 32],
    pub install_action_signature_verified: bool,
    pub physical_install_approval_consumed: bool,
}

#[derive(Clone, Copy)]
pub struct RepromotionReverifyInput<'a> {
    pub artstor_blob_bytes: &'a [u8],
    pub record_artstor_blob_frame_sha256: [u8; 32],
    pub record_artifact_sha256: [u8; 32],
    pub record_manifest_hash: [u8; 32],
    pub record_vm_report_hash: [u8; 32],
    pub record_grant_hash: [u8; 32],
    pub transaction: Option<RepromotionTransactionFields<'a>>,
    pub recomputed_attestation_reference_hash: [u8; 32],
    pub promotion_authority_is_placeholder: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepromotionDecision {
    pub reverified: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub candidate_payload_sha256: Option<[u8; 32]>,
}

pub fn reverify_persisted_artifact(input: &RepromotionReverifyInput<'_>) -> RepromotionDecision {
    if sha256_bytes(input.artstor_blob_bytes) != input.record_artstor_blob_frame_sha256 {
        return denied("blob_hash_mismatch", None);
    }
    let parsed = match parse_artifact_blob_frame(input.artstor_blob_bytes, 0) {
        Ok(parsed) => parsed,
        Err(_) => return denied("artifact_blob_parse_failed", None),
    };
    if parsed.payload_sha256 != input.record_artifact_sha256 {
        return denied("artifact_sha_mismatch", Some(parsed.payload_sha256));
    }

    let Some(transaction) = input.transaction else {
        return denied(
            "promotion_transaction_not_found",
            Some(parsed.payload_sha256),
        );
    };
    if input.record_manifest_hash != transaction.manifest_hash
        || input.record_vm_report_hash != transaction.vm_report_hash
        || input.record_grant_hash != transaction.computed_grant_hash
        || input.record_artifact_sha256 != transaction.artifact_hash
    {
        return denied(
            "transaction_hash_binding_mismatch",
            Some(parsed.payload_sha256),
        );
    }
    if !transaction.grant_binds_capability {
        return denied(
            "grant_does_not_bind_capability",
            Some(parsed.payload_sha256),
        );
    }
    if input.recomputed_attestation_reference_hash != transaction.attestation_reference_hash {
        return denied(
            "attestation_reference_hash_mismatch",
            Some(parsed.payload_sha256),
        );
    }
    if !verify_promotion_authority_signature(
        transaction.promotion_signature_der,
        &input.recomputed_attestation_reference_hash,
    ) {
        return denied("signature_not_verified", Some(parsed.payload_sha256));
    }
    if transaction.promotion_authority_key_sha256
        != PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
    {
        return denied(
            "promotion_authority_key_mismatch",
            Some(parsed.payload_sha256),
        );
    }
    if !input.promotion_authority_is_placeholder {
        return denied(
            "promotion_authority_not_placeholder",
            Some(parsed.payload_sha256),
        );
    }
    if !transaction.install_authorization_present {
        return denied("install_authorization_missing", Some(parsed.payload_sha256));
    }
    if !transaction.install_envelope_binds_activation {
        return denied(
            "install_envelope_binding_mismatch",
            Some(parsed.payload_sha256),
        );
    }
    if !verify_promotion_authority_signature(
        transaction.install_action_signature_der,
        &transaction.install_action_signature_message_sha256,
    ) || transaction.install_authority_key_sha256
        != PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
        || !transaction.install_action_signature_verified
    {
        return denied(
            "install_action_signature_not_verified",
            Some(parsed.payload_sha256),
        );
    }
    if !transaction.physical_install_approval_consumed {
        return denied(
            "physical_install_approval_missing",
            Some(parsed.payload_sha256),
        );
    }

    RepromotionDecision {
        reverified: true,
        status: "reverified",
        reason: "repromotion_reverified_dev_key_signature",
        candidate_payload_sha256: Some(parsed.payload_sha256),
    }
}

fn denied(reason: &'static str, candidate_payload_sha256: Option<[u8; 32]>) -> RepromotionDecision {
    RepromotionDecision {
        reverified: false,
        status: "repromotion_denied",
        reason,
        candidate_payload_sha256,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        artifact_blob_frame::build_artifact_blob_frame,
        promotion_attestation::PROMOTION_AUTHORITY_IS_PLACEHOLDER, sha256_bytes,
    };
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};

    const PLACEHOLDER_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];

    struct Fixture {
        blob: Vec<u8>,
        signature_der: Vec<u8>,
        attestation_reference_hash: [u8; 32],
        artifact_hash: [u8; 32],
    }

    fn h(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn sign(payload: &[u8]) -> Vec<u8> {
        let key =
            SigningKey::from_slice(&PLACEHOLDER_PRIVATE_SCALAR).expect("test scalar must be valid");
        let signature: Signature = key.sign(payload);
        signature.to_der().as_bytes().to_vec()
    }

    fn fixture() -> Fixture {
        let payload = b"persisted wasm bytes";
        let blob = build_artifact_blob_frame(payload);
        let attestation_reference_hash = h(0x24);
        Fixture {
            blob,
            signature_der: sign(&attestation_reference_hash),
            attestation_reference_hash,
            artifact_hash: sha256_bytes(payload),
        }
    }

    fn valid_input<'a>(
        fixture: &'a Fixture,
        transaction: Option<RepromotionTransactionFields<'a>>,
    ) -> RepromotionReverifyInput<'a> {
        let tx = transaction.unwrap_or(RepromotionTransactionFields {
            computed_grant_hash: h(0x10),
            manifest_hash: h(0x11),
            artifact_hash: fixture.artifact_hash,
            vm_report_hash: h(0x12),
            local_attestation_hash: h(0x13),
            attestation_reference_hash: fixture.attestation_reference_hash,
            promotion_signature_der: &fixture.signature_der,
            promotion_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            signature_verified: false,
            grant_binds_capability: true,
            install_authorization_present: true,
            install_envelope_binds_activation: true,
            install_action_signature_message_sha256: fixture.attestation_reference_hash,
            install_action_signature_der: &fixture.signature_der,
            install_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            install_action_signature_verified: true,
            physical_install_approval_consumed: true,
        });
        RepromotionReverifyInput {
            artstor_blob_bytes: &fixture.blob,
            record_artstor_blob_frame_sha256: sha256_bytes(&fixture.blob),
            record_artifact_sha256: fixture.artifact_hash,
            record_manifest_hash: h(0x11),
            record_vm_report_hash: h(0x12),
            record_grant_hash: h(0x10),
            transaction: Some(tx),
            recomputed_attestation_reference_hash: fixture.attestation_reference_hash,
            promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        }
    }

    #[test]
    fn positive_reverifies_signature_even_when_stored_boolean_is_false() {
        let fixture = fixture();
        let decision = reverify_persisted_artifact(&valid_input(&fixture, None));

        assert_eq!(decision.status, "reverified");
        assert_eq!(decision.reason, "repromotion_reverified_dev_key_signature");
        assert!(decision.reverified);
    }

    #[test]
    fn corrupt_blob_is_denied_before_parse_trust() {
        let fixture = fixture();
        let mut input = valid_input(&fixture, None);
        let mut corrupt = fixture.blob.clone();
        corrupt[48] ^= 0x01;
        input.artstor_blob_bytes = &corrupt;

        assert_eq!(
            reverify_persisted_artifact(&input).reason,
            "blob_hash_mismatch"
        );
    }

    #[test]
    fn tampered_record_binding_is_denied() {
        let fixture = fixture();
        let mut input = valid_input(&fixture, None);
        input.record_manifest_hash = h(0x99);

        assert_eq!(
            reverify_persisted_artifact(&input).reason,
            "transaction_hash_binding_mismatch"
        );
    }

    #[test]
    fn missing_transaction_is_denied() {
        let fixture = fixture();
        let mut input = valid_input(&fixture, None);
        input.transaction = None;

        assert_eq!(
            reverify_persisted_artifact(&input).reason,
            "promotion_transaction_not_found"
        );
    }

    #[test]
    fn bad_signature_is_denied() {
        let fixture = fixture();
        let mut bad_signature = fixture.signature_der.clone();
        let last = bad_signature.len() - 1;
        bad_signature[last] ^= 0x01;
        let tx = RepromotionTransactionFields {
            promotion_signature_der: &bad_signature,
            ..valid_input(&fixture, None).transaction.unwrap()
        };

        assert_eq!(
            reverify_persisted_artifact(&valid_input(&fixture, Some(tx))).reason,
            "signature_not_verified"
        );
    }

    #[test]
    fn attestation_hash_mismatch_is_denied() {
        let fixture = fixture();
        let mut input = valid_input(&fixture, None);
        input.recomputed_attestation_reference_hash = h(0x25);

        assert_eq!(
            reverify_persisted_artifact(&input).reason,
            "attestation_reference_hash_mismatch"
        );
    }

    #[test]
    fn missing_w6_pins_name_their_denials() {
        let fixture = fixture();
        let base = valid_input(&fixture, None).transaction.unwrap();
        let cases = [
            (
                RepromotionTransactionFields {
                    install_authorization_present: false,
                    ..base
                },
                "install_authorization_missing",
            ),
            (
                RepromotionTransactionFields {
                    install_envelope_binds_activation: false,
                    ..base
                },
                "install_envelope_binding_mismatch",
            ),
            (
                RepromotionTransactionFields {
                    install_action_signature_verified: false,
                    ..base
                },
                "install_action_signature_not_verified",
            ),
            (
                RepromotionTransactionFields {
                    physical_install_approval_consumed: false,
                    ..base
                },
                "physical_install_approval_missing",
            ),
        ];
        for (transaction, reason) in cases {
            assert_eq!(
                reverify_persisted_artifact(&valid_input(&fixture, Some(transaction))).reason,
                reason
            );
        }
    }
}
