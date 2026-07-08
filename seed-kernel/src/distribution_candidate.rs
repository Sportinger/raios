use raios_core::{
    distribution_provenance::{
        verify_distribution_provenance_signature,
        PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    },
    scoped_distribution_provenance_honesty::{
        evaluate_distribution_provenance_honesty, DistributionProvenanceHonestyInput,
    },
};

pub(crate) const DISTRIBUTION_SOURCE_KIND: &str = "local_signed_registry";
const TRUST_TIER: &str = "dev_key_not_owner_sealed";

#[derive(Clone, Copy)]
pub(crate) struct DistributionCandidateOutcome {
    pub(crate) source_kind: &'static str,
    pub(crate) retained_present: bool,
    pub(crate) retained_wasm_valid: bool,
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) provenance_signature_present: bool,
    pub(crate) provenance_verified: bool,
    pub(crate) publisher_key_sha256: [u8; 32],
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) honest: bool,
    pub(crate) live_load_projection_present: bool,
    pub(crate) live_load_projection_can_load_now: bool,
    pub(crate) load_attempted: bool,
    pub(crate) execution_attempted: bool,
    pub(crate) authorizes_load: bool,
    pub(crate) authorizes_execution: bool,
    pub(crate) writes_persistent_state: bool,
    pub(crate) load_authorized: bool,
    pub(crate) install_authorized: bool,
    pub(crate) owner_sealed: bool,
    pub(crate) requires_m6_reverify_for_load: bool,
    pub(crate) trust_tier: &'static str,
}

pub(crate) fn verify_retained_candidate_provenance(
    signature_der: &[u8],
) -> DistributionCandidateOutcome {
    let projection = crate::granted_candidate_service::live_load_projection();
    let retained = crate::module_candidate_intake::retained();
    let signature_present = !signature_der.is_empty();

    let Some(candidate) = retained else {
        return outcome(
            false,
            false,
            [0; 32],
            signature_present,
            false,
            "denied",
            "no_retained_candidate",
            false,
            projection.present,
            projection.can_load_now,
        );
    };

    if !candidate.wasm_valid {
        return outcome(
            true,
            false,
            candidate.sha256,
            signature_present,
            false,
            "denied",
            "retained_candidate_not_wasm_valid",
            false,
            projection.present,
            projection.can_load_now,
        );
    }

    let provenance_verified =
        verify_distribution_provenance_signature(signature_der, &candidate.sha256);
    let decision = evaluate_distribution_provenance_honesty(&DistributionProvenanceHonestyInput {
        source_kind: Some(DISTRIBUTION_SOURCE_KIND),
        artifact_sha256_present: true,
        provenance_signature_present: signature_present,
        provenance_signature_verified: provenance_verified,
        intake_as_candidate: true,
        claims_installable: false,
        claims_load_authorized_by_provenance: false,
        requires_m6_reverify_for_load: true,
        claims_owner_sealed: false,
    });

    outcome(
        true,
        true,
        candidate.sha256,
        signature_present,
        decision.provenance_verified,
        decision.status,
        decision.reason,
        decision.honest,
        projection.present,
        projection.can_load_now,
    )
}

fn outcome(
    retained_present: bool,
    retained_wasm_valid: bool,
    artifact_sha256: [u8; 32],
    provenance_signature_present: bool,
    provenance_verified: bool,
    status: &'static str,
    reason: &'static str,
    honest: bool,
    live_load_projection_present: bool,
    live_load_projection_can_load_now: bool,
) -> DistributionCandidateOutcome {
    DistributionCandidateOutcome {
        source_kind: DISTRIBUTION_SOURCE_KIND,
        retained_present,
        retained_wasm_valid,
        artifact_sha256,
        provenance_signature_present,
        provenance_verified,
        publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
        status,
        reason,
        honest,
        live_load_projection_present,
        live_load_projection_can_load_now,
        load_attempted: false,
        execution_attempted: false,
        authorizes_load: false,
        authorizes_execution: false,
        writes_persistent_state: false,
        load_authorized: false,
        install_authorized: false,
        owner_sealed: false,
        requires_m6_reverify_for_load: true,
        trust_tier: TRUST_TIER,
    }
}
