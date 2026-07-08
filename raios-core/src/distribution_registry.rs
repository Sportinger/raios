//! Local content-addressed distribution registry selection.
//!
//! Selection proves only local provenance for inert candidate intake. It never
//! grants acquisition, install, load, execute, or persistence authority.

use alloc::vec;

use crate::{
    distribution_provenance::{
        verify_distribution_provenance_signature,
        PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    },
    record::{Field, Value},
    scoped_distribution_provenance_honesty::{
        evaluate_distribution_provenance_honesty, DistributionProvenanceHonestyInput,
    },
    sha256_bytes,
};

pub const DISTRIBUTION_REGISTRY_SELECTION_SCHEMA: &str = "raios.distribution_registry_selection.v0";
pub const DISTRIBUTION_REGISTRY_SELECTION_ID: &str =
    "distribution_registry.selection.current_boot.v0";
pub const DISTRIBUTION_REGISTRY_SOURCE_KIND: &str = "local_signed_registry";
pub const DISTRIBUTION_REGISTRY_TRUST_TIER: &str = "dev_key_not_owner_sealed";

#[derive(Clone, Copy)]
pub struct DistributionRegistryEntryInput<'a> {
    pub entry_id: &'a str,
    pub artifact_bytes: &'a [u8],
    pub claimed_artifact_sha256: [u8; 32],
    pub provenance_signature_der: Option<&'a [u8]>,
    pub publisher_key_sha256: [u8; 32],
    pub classification: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DistributionRegistryEntry<'a> {
    pub entry_id: &'a str,
    pub artifact_bytes: &'a [u8],
    pub artifact_sha256: [u8; 32],
    pub provenance_signature_der: &'a [u8],
    pub publisher_key_sha256: [u8; 32],
    pub classification: &'static str,
}

impl<'a> DistributionRegistryEntry<'a> {
    pub fn new(input: DistributionRegistryEntryInput<'a>) -> Result<Self, RegistrySelectionReason> {
        let artifact_sha256 = sha256_bytes(input.artifact_bytes);
        if artifact_sha256 != input.claimed_artifact_sha256 {
            return Err(RegistrySelectionReason::ArtifactHashMismatch);
        }

        let provenance_signature_der = match input.provenance_signature_der {
            Some(signature) if !signature.is_empty() => signature,
            Some(_) => return Err(RegistrySelectionReason::ProvenanceSignatureEmpty),
            None => return Err(RegistrySelectionReason::ProvenanceSignatureAbsent),
        };

        if input.publisher_key_sha256 != PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256 {
            return Err(RegistrySelectionReason::PublisherKeyMismatch);
        }

        let classification = match input.classification.trim() {
            "public" => "public",
            "local_only" => "local_only",
            "secret" => return Err(RegistrySelectionReason::SecretClassification),
            _ => return Err(RegistrySelectionReason::ClassificationOutOfScope),
        };

        Ok(Self {
            entry_id: input.entry_id,
            artifact_bytes: input.artifact_bytes,
            artifact_sha256,
            provenance_signature_der,
            publisher_key_sha256: input.publisher_key_sha256,
            classification,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegistrySelectionDecision<'a> {
    pub entry_id: &'a str,
    pub status: &'static str,
    pub reason: RegistrySelectionReason,
    pub source_kind: &'static str,
    pub classification: &'static str,
    pub requested_artifact_sha256: [u8; 32],
    pub artifact_sha256: [u8; 32],
    pub artifact_byte_len: usize,
    pub publisher_key_sha256: [u8; 32],
    pub selection_hash_matched: bool,
    pub provenance_signature_present: bool,
    pub provenance_signature_verified: bool,
    pub provenance_honest: bool,
    pub provenance_honesty_status: &'static str,
    pub provenance_honesty_reason: &'static str,
    pub selected_for_candidate_intake: bool,
    pub candidate_intake_kind: &'static str,
    pub requires_m6_reverify_for_load: bool,
    pub load_attempted: bool,
    pub execution_attempted: bool,
    pub durable_write_attempted: bool,
    pub authorizes_acquisition: bool,
    pub authorizes_install: bool,
    pub authorizes_load: bool,
    pub authorizes_execute: bool,
    pub authorizes_persist: bool,
    pub writes_persistent_state: bool,
    pub owner_sealed: bool,
    pub trust_tier: &'static str,
}

impl<'a> RegistrySelectionDecision<'a> {
    pub fn as_record_value(&self) -> Value<'a> {
        Value::Object(vec![
            Field::new("schema", Value::Str(DISTRIBUTION_REGISTRY_SELECTION_SCHEMA)),
            Field::new("id", Value::Str(DISTRIBUTION_REGISTRY_SELECTION_ID)),
            Field::new("scope", Value::Str("current_boot")),
            Field::new("entry_id", Value::Str(self.entry_id)),
            Field::new("status", Value::Str(self.status)),
            Field::new("reason", Value::Str(self.reason.as_str())),
            Field::new("source_kind", Value::Str(self.source_kind)),
            Field::new("classification", Value::Str(self.classification)),
            Field::new(
                "requested_artifact_sha256",
                Value::Sha256(self.requested_artifact_sha256),
            ),
            Field::new("artifact_sha256", Value::Sha256(self.artifact_sha256)),
            Field::new(
                "artifact_byte_len",
                Value::U64(self.artifact_byte_len as u64),
            ),
            Field::new(
                "publisher_key_sha256",
                Value::Sha256(self.publisher_key_sha256),
            ),
            Field::new(
                "selection_hash_matched",
                Value::Bool(self.selection_hash_matched),
            ),
            Field::new(
                "provenance_signature_present",
                Value::Bool(self.provenance_signature_present),
            ),
            Field::new(
                "provenance_signature_verified",
                Value::Bool(self.provenance_signature_verified),
            ),
            Field::new("provenance_honest", Value::Bool(self.provenance_honest)),
            Field::new(
                "provenance_honesty_status",
                Value::Str(self.provenance_honesty_status),
            ),
            Field::new(
                "provenance_honesty_reason",
                Value::Str(self.provenance_honesty_reason),
            ),
            Field::new(
                "selected_for_candidate_intake",
                Value::Bool(self.selected_for_candidate_intake),
            ),
            Field::new(
                "candidate_intake_kind",
                Value::Str(self.candidate_intake_kind),
            ),
            Field::new(
                "requires_m6_reverify_for_load",
                Value::Bool(self.requires_m6_reverify_for_load),
            ),
            Field::new("load_attempted", Value::Bool(self.load_attempted)),
            Field::new("execution_attempted", Value::Bool(self.execution_attempted)),
            Field::new(
                "durable_write_attempted",
                Value::Bool(self.durable_write_attempted),
            ),
            Field::new(
                "authorizes_acquisition",
                Value::Bool(self.authorizes_acquisition),
            ),
            Field::new("authorizes_install", Value::Bool(self.authorizes_install)),
            Field::new("authorizes_load", Value::Bool(self.authorizes_load)),
            Field::new("authorizes_execute", Value::Bool(self.authorizes_execute)),
            Field::new("authorizes_persist", Value::Bool(self.authorizes_persist)),
            Field::new(
                "writes_persistent_state",
                Value::Bool(self.writes_persistent_state),
            ),
            Field::new("owner_sealed", Value::Bool(self.owner_sealed)),
            Field::new("trust_tier", Value::Str(self.trust_tier)),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistrySelectionReason {
    SelectedForInertCandidateIntake,
    ArtifactHashMismatch,
    ProvenanceSignatureAbsent,
    ProvenanceSignatureEmpty,
    PublisherKeyMismatch,
    SecretClassification,
    ClassificationOutOfScope,
    SelectionHashMismatch,
    ProvenanceSignatureUnverified,
    ProvenanceHonestyDenied(&'static str),
}

impl RegistrySelectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            RegistrySelectionReason::SelectedForInertCandidateIntake => {
                "registry_entry_selected_for_inert_candidate_intake"
            }
            RegistrySelectionReason::ArtifactHashMismatch => "artifact_hash_mismatch",
            RegistrySelectionReason::ProvenanceSignatureAbsent => "provenance_signature_absent",
            RegistrySelectionReason::ProvenanceSignatureEmpty => "provenance_signature_empty",
            RegistrySelectionReason::PublisherKeyMismatch => "publisher_key_mismatch",
            RegistrySelectionReason::SecretClassification => "secret_classification_rejected",
            RegistrySelectionReason::ClassificationOutOfScope => "classification_out_of_scope",
            RegistrySelectionReason::SelectionHashMismatch => "selection_hash_mismatch",
            RegistrySelectionReason::ProvenanceSignatureUnverified => {
                "provenance_signature_unverified"
            }
            RegistrySelectionReason::ProvenanceHonestyDenied(reason) => reason,
        }
    }
}

pub fn evaluate_distribution_registry_selection<'a>(
    entry: &DistributionRegistryEntry<'a>,
    requested_artifact_sha256: [u8; 32],
) -> RegistrySelectionDecision<'a> {
    if requested_artifact_sha256 != entry.artifact_sha256 {
        return selection_decision(
            entry,
            requested_artifact_sha256,
            "denied",
            RegistrySelectionReason::SelectionHashMismatch,
            false,
            false,
            false,
            "not_evaluated",
            "selection_hash_mismatch",
            false,
        );
    }

    let provenance_signature_verified = verify_distribution_provenance_signature(
        entry.provenance_signature_der,
        &entry.artifact_sha256,
    );
    let honesty = evaluate_distribution_provenance_honesty(&DistributionProvenanceHonestyInput {
        source_kind: Some(DISTRIBUTION_REGISTRY_SOURCE_KIND),
        artifact_sha256_present: true,
        provenance_signature_present: true,
        provenance_signature_verified,
        intake_as_candidate: true,
        claims_installable: false,
        claims_load_authorized_by_provenance: false,
        requires_m6_reverify_for_load: true,
        claims_owner_sealed: false,
    });

    let selected = honesty.honest;
    let reason = if selected {
        RegistrySelectionReason::SelectedForInertCandidateIntake
    } else if !provenance_signature_verified {
        RegistrySelectionReason::ProvenanceSignatureUnverified
    } else {
        RegistrySelectionReason::ProvenanceHonestyDenied(honesty.reason)
    };
    let status = if selected { "selected" } else { "denied" };

    selection_decision(
        entry,
        requested_artifact_sha256,
        status,
        reason,
        true,
        provenance_signature_verified,
        honesty.honest,
        honesty.status,
        honesty.reason,
        selected,
    )
}

#[allow(clippy::too_many_arguments)]
fn selection_decision<'a>(
    entry: &DistributionRegistryEntry<'a>,
    requested_artifact_sha256: [u8; 32],
    status: &'static str,
    reason: RegistrySelectionReason,
    selection_hash_matched: bool,
    provenance_signature_verified: bool,
    provenance_honest: bool,
    provenance_honesty_status: &'static str,
    provenance_honesty_reason: &'static str,
    selected_for_candidate_intake: bool,
) -> RegistrySelectionDecision<'a> {
    RegistrySelectionDecision {
        entry_id: entry.entry_id,
        status,
        reason,
        source_kind: DISTRIBUTION_REGISTRY_SOURCE_KIND,
        classification: entry.classification,
        requested_artifact_sha256,
        artifact_sha256: entry.artifact_sha256,
        artifact_byte_len: entry.artifact_bytes.len(),
        publisher_key_sha256: entry.publisher_key_sha256,
        selection_hash_matched,
        provenance_signature_present: true,
        provenance_signature_verified,
        provenance_honest,
        provenance_honesty_status,
        provenance_honesty_reason,
        selected_for_candidate_intake,
        candidate_intake_kind: "current_boot_inert_candidate",
        requires_m6_reverify_for_load: true,
        load_attempted: false,
        execution_attempted: false,
        durable_write_attempted: false,
        authorizes_acquisition: false,
        authorizes_install: false,
        authorizes_load: false,
        authorizes_execute: false,
        authorizes_persist: false,
        writes_persistent_state: false,
        owner_sealed: false,
        trust_tier: DISTRIBUTION_REGISTRY_TRUST_TIER,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{distribution_provenance::DISTRIBUTION_PROVENANCE_DOMAIN_TAG, record::write_json};
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use std::{string::String, vec::Vec};

    const PUBLISHER_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02,
    ];
    const WRONG_PRIVATE_SCALAR: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];

    struct Fixture {
        bytes: Vec<u8>,
        hash: [u8; 32],
        signature: Vec<u8>,
    }

    impl Fixture {
        fn input(&self) -> DistributionRegistryEntryInput<'_> {
            DistributionRegistryEntryInput {
                entry_id: "builtin.echo",
                artifact_bytes: &self.bytes,
                claimed_artifact_sha256: self.hash,
                provenance_signature_der: Some(&self.signature),
                publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
                classification: "local_only",
            }
        }
    }

    fn fixture() -> Fixture {
        let bytes = b"\0asm registry selection fixture".to_vec();
        let hash = sha256_bytes(&bytes);
        let signature = sign_hash(&PUBLISHER_PRIVATE_SCALAR, &hash);
        Fixture {
            bytes,
            hash,
            signature,
        }
    }

    fn sign_hash(scalar: &[u8; 32], artifact_sha256: &[u8; 32]) -> Vec<u8> {
        let key = SigningKey::from_slice(scalar).expect("test scalar must be valid");
        let mut message = Vec::new();
        message.extend_from_slice(DISTRIBUTION_PROVENANCE_DOMAIN_TAG);
        message.extend_from_slice(artifact_sha256);
        let signature: Signature = key.sign(&message);
        signature.to_der().as_bytes().to_vec()
    }

    fn decision_for<'a>(entry: &DistributionRegistryEntry<'a>) -> RegistrySelectionDecision<'a> {
        evaluate_distribution_registry_selection(entry, entry.artifact_sha256)
    }

    fn render(value: &Value<'_>) -> String {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        String::from_utf8(out).expect("record JSON is UTF-8")
    }

    fn assert_no_authority(decision: &RegistrySelectionDecision<'_>) {
        assert!(!decision.authorizes_acquisition);
        assert!(!decision.authorizes_install);
        assert!(!decision.authorizes_load);
        assert!(!decision.authorizes_execute);
        assert!(!decision.authorizes_persist);
        assert!(!decision.writes_persistent_state);
        assert!(!decision.load_attempted);
        assert!(!decision.execution_attempted);
        assert!(!decision.durable_write_attempted);
        assert!(!decision.owner_sealed);
    }

    #[test]
    fn valid_registry_entry_selects_inert_candidate_via_record_value() {
        let fixture = fixture();
        let entry = DistributionRegistryEntry::new(fixture.input()).expect("valid entry");
        let decision = decision_for(&entry);

        assert_eq!(entry.artifact_sha256, fixture.hash);
        assert_eq!(
            decision.reason,
            RegistrySelectionReason::SelectedForInertCandidateIntake
        );
        assert!(decision.selected_for_candidate_intake);
        assert!(decision.selection_hash_matched);
        assert!(decision.provenance_signature_verified);
        assert!(decision.provenance_honest);
        assert_no_authority(&decision);

        let rendered = render(&decision.as_record_value());
        assert!(rendered.contains("\"schema\": \"raios.distribution_registry_selection.v0\""));
        assert!(rendered.contains("\"selected_for_candidate_intake\": true"));
        assert!(rendered.contains("\"authorizes_load\": false"));
    }

    #[test]
    fn mismatched_claimed_hash_is_rejected_before_selection() {
        let fixture = fixture();
        let mut input = fixture.input();
        input.claimed_artifact_sha256[0] ^= 0x01;

        assert_eq!(
            DistributionRegistryEntry::new(input),
            Err(RegistrySelectionReason::ArtifactHashMismatch)
        );
    }

    #[test]
    fn wrong_signature_denies_selection_without_granting_authority() {
        let fixture = fixture();
        let wrong_signature = sign_hash(&WRONG_PRIVATE_SCALAR, &fixture.hash);
        let mut input = fixture.input();
        input.provenance_signature_der = Some(&wrong_signature);
        let entry = DistributionRegistryEntry::new(input).expect("structurally valid entry");
        let decision = decision_for(&entry);

        assert_eq!(
            decision.reason,
            RegistrySelectionReason::ProvenanceSignatureUnverified
        );
        assert_eq!(decision.status, "denied");
        assert!(!decision.selected_for_candidate_intake);
        assert!(!decision.provenance_signature_verified);
        assert_no_authority(&decision);
    }

    #[test]
    fn absent_signature_is_rejected_by_constructor() {
        let fixture = fixture();
        let mut input = fixture.input();
        input.provenance_signature_der = None;

        assert_eq!(
            DistributionRegistryEntry::new(input),
            Err(RegistrySelectionReason::ProvenanceSignatureAbsent)
        );
    }

    #[test]
    fn empty_signature_is_rejected_by_constructor() {
        let fixture = fixture();
        let mut input = fixture.input();
        input.provenance_signature_der = Some(&[]);

        assert_eq!(
            DistributionRegistryEntry::new(input),
            Err(RegistrySelectionReason::ProvenanceSignatureEmpty)
        );
    }

    #[test]
    fn wrong_publisher_key_is_rejected_by_constructor() {
        let fixture = fixture();
        let mut input = fixture.input();
        input.publisher_key_sha256[0] ^= 0x01;

        assert_eq!(
            DistributionRegistryEntry::new(input),
            Err(RegistrySelectionReason::PublisherKeyMismatch)
        );
    }

    #[test]
    fn secret_classification_is_rejected_by_constructor() {
        let fixture = fixture();
        let mut input = fixture.input();
        input.classification = "secret";

        assert_eq!(
            DistributionRegistryEntry::new(input),
            Err(RegistrySelectionReason::SecretClassification)
        );
    }

    #[test]
    fn selection_hash_mismatch_never_stages_or_grants_authority() {
        let fixture = fixture();
        let entry = DistributionRegistryEntry::new(fixture.input()).expect("valid entry");
        let mut requested = fixture.hash;
        requested[0] ^= 0x01;
        let decision = evaluate_distribution_registry_selection(&entry, requested);

        assert_eq!(
            decision.reason,
            RegistrySelectionReason::SelectionHashMismatch
        );
        assert_eq!(decision.status, "denied");
        assert!(!decision.selection_hash_matched);
        assert!(!decision.selected_for_candidate_intake);
        assert_no_authority(&decision);
    }

    #[test]
    fn valid_selection_never_authorizes_acquisition_install_load_execute_or_persist() {
        let fixture = fixture();
        let entry = DistributionRegistryEntry::new(fixture.input()).expect("valid entry");
        let decision = decision_for(&entry);

        assert!(decision.selected_for_candidate_intake);
        assert_no_authority(&decision);
    }
}
