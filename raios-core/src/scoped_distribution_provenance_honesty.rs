//! Scoped honesty check for distribution provenance labels.
//!
//! This evaluator verifies that publisher provenance is reported as provenance
//! only. It delegates candidate/load/install label honesty to M12-1.

use crate::scoped_external_acquisition_honesty::{
    evaluate_external_acquisition_honesty, ExternalAcquisitionHonestyInput,
};

pub const SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_SCHEMA: &str =
    "raios.scoped_distribution_provenance_honesty_decision.v0";
pub const SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_ID: &str =
    "scoped_distribution_provenance_honesty.current_boot.v0";
pub const SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_MARKER: &str =
    "RAIOS_DISTRIBUTION_PROVENANCE_HONESTY_SCOPE_DECISION";

#[derive(Clone, Copy)]
pub struct DistributionProvenanceHonestyInput<'a> {
    pub source_kind: Option<&'a str>,
    pub artifact_sha256_present: bool,
    pub provenance_signature_present: bool,
    pub provenance_signature_verified: bool,
    pub intake_as_candidate: bool,
    pub claims_installable: bool,
    pub claims_load_authorized_by_provenance: bool,
    pub requires_m6_reverify_for_load: bool,
    pub claims_owner_sealed: bool,
}

impl<'a> DistributionProvenanceHonestyInput<'a> {
    pub const fn empty() -> Self {
        Self {
            source_kind: None,
            artifact_sha256_present: false,
            provenance_signature_present: false,
            provenance_signature_verified: false,
            intake_as_candidate: false,
            claims_installable: false,
            claims_load_authorized_by_provenance: false,
            requires_m6_reverify_for_load: false,
            claims_owner_sealed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DistributionProvenanceHonestyDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub honest: bool,
    pub provenance_verified: bool,
    pub authorizes_acquisition: bool,
    pub authorizes_install: bool,
    pub authorizes_load: bool,
}

pub fn evaluate_distribution_provenance_honesty(
    input: &DistributionProvenanceHonestyInput<'_>,
) -> DistributionProvenanceHonestyDecision {
    if !input.provenance_signature_present {
        return denied("provenance_signature_absent", false);
    }
    if !input.provenance_signature_verified {
        return denied("provenance_signature_unverified", false);
    }

    let acquisition = ExternalAcquisitionHonestyInput {
        source_kind: input.source_kind,
        artifact_sha256_present: input.artifact_sha256_present,
        intake_as_candidate: input.intake_as_candidate,
        claims_installable: input.claims_installable,
        claims_load_authorized_by_distribution: input.claims_load_authorized_by_provenance,
        requires_m6_reverify_for_load: input.requires_m6_reverify_for_load,
        claims_owner_sealed: input.claims_owner_sealed,
    };
    let acquisition_decision = evaluate_external_acquisition_honesty(&acquisition);
    if acquisition_decision.status != "acquisition_labels_honest" {
        return denied(acquisition_decision.reason, true);
    }

    DistributionProvenanceHonestyDecision {
        performed: true,
        status: "distribution_candidate_provenance_verified_load_still_denied",
        reason: "provenance_authenticated_candidate_intake_only_grants_nothing",
        honest: true,
        provenance_verified: true,
        authorizes_acquisition: false,
        authorizes_install: false,
        authorizes_load: false,
    }
}

fn denied(
    reason: &'static str,
    provenance_verified: bool,
) -> DistributionProvenanceHonestyDecision {
    DistributionProvenanceHonestyDecision {
        performed: false,
        status: "denied",
        reason,
        honest: false,
        provenance_verified,
        authorizes_acquisition: false,
        authorizes_install: false,
        authorizes_load: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> DistributionProvenanceHonestyInput<'static> {
        DistributionProvenanceHonestyInput {
            source_kind: Some("local_signed_registry"),
            artifact_sha256_present: true,
            provenance_signature_present: true,
            provenance_signature_verified: true,
            intake_as_candidate: true,
            claims_installable: false,
            claims_load_authorized_by_provenance: false,
            requires_m6_reverify_for_load: true,
            claims_owner_sealed: false,
        }
    }

    #[test]
    fn accepts_verified_provenance_with_honest_candidate_labels() {
        assert_eq!(
            evaluate_distribution_provenance_honesty(&valid_input()),
            DistributionProvenanceHonestyDecision {
                performed: true,
                status: "distribution_candidate_provenance_verified_load_still_denied",
                reason: "provenance_authenticated_candidate_intake_only_grants_nothing",
                honest: true,
                provenance_verified: true,
                authorizes_acquisition: false,
                authorizes_install: false,
                authorizes_load: false,
            }
        );
    }

    #[test]
    fn absent_and_unverified_provenance_are_named_denials() {
        let mut absent = valid_input();
        absent.provenance_signature_present = false;
        let absent_decision = evaluate_distribution_provenance_honesty(&absent);
        assert_eq!(absent_decision.reason, "provenance_signature_absent");
        assert!(!absent_decision.provenance_verified);
        assert!(!absent_decision.authorizes_load);

        let mut unverified = valid_input();
        unverified.provenance_signature_verified = false;
        let unverified_decision = evaluate_distribution_provenance_honesty(&unverified);
        assert_eq!(
            unverified_decision.reason,
            "provenance_signature_unverified"
        );
        assert!(!unverified_decision.provenance_verified);
        assert!(!unverified_decision.authorizes_load);
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingSourceKind,
        MissingArtifactHash,
        IntakeNotCandidate,
        ClaimsInstallable,
        ClaimsLoadAuthorizedByProvenance,
        MissingM6ReverifyForLoad,
        ClaimsOwnerSealed,
    }

    fn apply(input: &mut DistributionProvenanceHonestyInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingSourceKind => input.source_kind = None,
            Mutation::MissingArtifactHash => input.artifact_sha256_present = false,
            Mutation::IntakeNotCandidate => input.intake_as_candidate = false,
            Mutation::ClaimsInstallable => input.claims_installable = true,
            Mutation::ClaimsLoadAuthorizedByProvenance => {
                input.claims_load_authorized_by_provenance = true
            }
            Mutation::MissingM6ReverifyForLoad => input.requires_m6_reverify_for_load = false,
            Mutation::ClaimsOwnerSealed => input.claims_owner_sealed = true,
        }
    }

    #[test]
    fn m12_label_overclaims_propagate_exact_reasons_and_grant_nothing() {
        let cases = [
            (Mutation::MissingSourceKind, "missing_source_kind"),
            (
                Mutation::MissingArtifactHash,
                "missing_artifact_content_hash",
            ),
            (Mutation::IntakeNotCandidate, "intake_not_labeled_candidate"),
            (Mutation::ClaimsInstallable, "install_overclaimed"),
            (
                Mutation::ClaimsLoadAuthorizedByProvenance,
                "distribution_signature_load_authority_overclaimed",
            ),
            (
                Mutation::MissingM6ReverifyForLoad,
                "load_worthiness_not_gated_on_m6_reverify",
            ),
            (Mutation::ClaimsOwnerSealed, "owner_sealed_overclaimed"),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_distribution_provenance_honesty(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);
            assert!(!decision.honest);
            assert!(decision.provenance_verified);
            assert!(!decision.authorizes_acquisition);
            assert!(!decision.authorizes_install);
            assert!(!decision.authorizes_load);
            idx += 1;
        }
    }

    #[test]
    fn success_and_every_path_never_authorizes_load() {
        let success = evaluate_distribution_provenance_honesty(&valid_input());
        assert!(success.honest);
        assert!(!success.authorizes_load);

        let mut absent = valid_input();
        absent.provenance_signature_present = false;
        assert!(!evaluate_distribution_provenance_honesty(&absent).authorizes_load);

        let mut unverified = valid_input();
        unverified.provenance_signature_verified = false;
        assert!(!evaluate_distribution_provenance_honesty(&unverified).authorizes_load);

        let mutations = [
            Mutation::MissingSourceKind,
            Mutation::MissingArtifactHash,
            Mutation::IntakeNotCandidate,
            Mutation::ClaimsInstallable,
            Mutation::ClaimsLoadAuthorizedByProvenance,
            Mutation::MissingM6ReverifyForLoad,
            Mutation::ClaimsOwnerSealed,
        ];

        let mut idx = 0usize;
        while idx < mutations.len() {
            let mut input = valid_input();
            apply(&mut input, mutations[idx]);
            assert!(!evaluate_distribution_provenance_honesty(&input).authorizes_load);
            idx += 1;
        }
    }
}
