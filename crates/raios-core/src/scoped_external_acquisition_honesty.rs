//! Scoped honesty check for external artifact acquisition labels (M12-1).
//!
//! This evaluator grants nothing. It only certifies that received artifact bytes
//! are labeled as candidate intake, provenance-only, and still subject to the
//! local M6/M7 load-worthiness chain.

pub const SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_SCHEMA: &str =
    "raios.scoped_external_acquisition_honesty_decision.v0";
pub const SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_ID: &str =
    "scoped_external_acquisition_honesty.current_boot.v0";
pub const SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_MARKER: &str =
    "RAIOS_EXTERNAL_ACQUISITION_HONESTY_SCOPE_DECISION";

#[derive(Clone, Copy)]
pub struct ExternalAcquisitionHonestyInput<'a> {
    pub source_kind: Option<&'a str>,
    pub artifact_sha256_present: bool,
    pub intake_as_candidate: bool,
    pub claims_installable: bool,
    pub claims_load_authorized_by_distribution: bool,
    pub requires_m6_reverify_for_load: bool,
    pub claims_owner_sealed: bool,
}

impl<'a> ExternalAcquisitionHonestyInput<'a> {
    pub const fn empty() -> Self {
        Self {
            source_kind: None,
            artifact_sha256_present: false,
            intake_as_candidate: false,
            claims_installable: false,
            claims_load_authorized_by_distribution: false,
            requires_m6_reverify_for_load: false,
            claims_owner_sealed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalAcquisitionHonestyDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub honest: bool,
    pub authorizes_acquisition: bool,
    pub authorizes_install: bool,
    pub authorizes_load: bool,
}

pub fn evaluate_external_acquisition_honesty(
    input: &ExternalAcquisitionHonestyInput<'_>,
) -> ExternalAcquisitionHonestyDecision {
    match input.source_kind {
        Some(value) if !value.is_empty() => {}
        _ => return denied("missing_source_kind"),
    }

    if !input.artifact_sha256_present {
        return denied("missing_artifact_content_hash");
    }
    if !input.intake_as_candidate {
        return denied("intake_not_labeled_candidate");
    }
    if input.claims_installable {
        return denied("install_overclaimed");
    }
    if input.claims_load_authorized_by_distribution {
        return denied("distribution_signature_load_authority_overclaimed");
    }
    if !input.requires_m6_reverify_for_load {
        return denied("load_worthiness_not_gated_on_m6_reverify");
    }
    if input.claims_owner_sealed {
        return denied("owner_sealed_overclaimed");
    }

    ExternalAcquisitionHonestyDecision {
        performed: true,
        status: "acquisition_labels_honest",
        reason: "honest_candidate_intake_provenance_only_grants_nothing",
        honest: true,
        authorizes_acquisition: false,
        authorizes_install: false,
        authorizes_load: false,
    }
}

fn denied(reason: &'static str) -> ExternalAcquisitionHonestyDecision {
    ExternalAcquisitionHonestyDecision {
        performed: false,
        status: "denied",
        reason,
        honest: false,
        authorizes_acquisition: false,
        authorizes_install: false,
        authorizes_load: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ExternalAcquisitionHonestyInput<'static> {
        ExternalAcquisitionHonestyInput {
            source_kind: Some("local_signed_registry"),
            artifact_sha256_present: true,
            intake_as_candidate: true,
            claims_installable: false,
            claims_load_authorized_by_distribution: false,
            requires_m6_reverify_for_load: true,
            claims_owner_sealed: false,
        }
    }

    #[test]
    fn accepts_honest_candidate_intake_provenance_only_labels() {
        assert_eq!(
            evaluate_external_acquisition_honesty(&valid_input()),
            ExternalAcquisitionHonestyDecision {
                performed: true,
                status: "acquisition_labels_honest",
                reason: "honest_candidate_intake_provenance_only_grants_nothing",
                honest: true,
                authorizes_acquisition: false,
                authorizes_install: false,
                authorizes_load: false,
            }
        );
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingSourceKind,
        MissingArtifactHash,
        IntakeNotCandidate,
        ClaimsInstallable,
        ClaimsLoadAuthorizedByDistribution,
        MissingM6ReverifyForLoad,
        ClaimsOwnerSealed,
    }

    fn apply(input: &mut ExternalAcquisitionHonestyInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingSourceKind => input.source_kind = None,
            Mutation::MissingArtifactHash => input.artifact_sha256_present = false,
            Mutation::IntakeNotCandidate => input.intake_as_candidate = false,
            Mutation::ClaimsInstallable => input.claims_installable = true,
            Mutation::ClaimsLoadAuthorizedByDistribution => {
                input.claims_load_authorized_by_distribution = true
            }
            Mutation::MissingM6ReverifyForLoad => input.requires_m6_reverify_for_load = false,
            Mutation::ClaimsOwnerSealed => input.claims_owner_sealed = true,
        }
    }

    #[test]
    fn denial_truth_table_names_first_failed_pin() {
        let cases = [
            (Mutation::MissingSourceKind, "missing_source_kind"),
            (
                Mutation::MissingArtifactHash,
                "missing_artifact_content_hash",
            ),
            (Mutation::IntakeNotCandidate, "intake_not_labeled_candidate"),
            (Mutation::ClaimsInstallable, "install_overclaimed"),
            (
                Mutation::ClaimsLoadAuthorizedByDistribution,
                "distribution_signature_load_authority_overclaimed",
            ),
            (
                Mutation::MissingM6ReverifyForLoad,
                "load_worthiness_not_gated_on_m6_reverify",
            ),
            (Mutation::ClaimsOwnerSealed, "owner_sealed_overclaimed"),
        ];
        assert_eq!(cases.len(), 7);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_external_acquisition_honesty(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);
            assert!(!decision.honest);
            assert!(!decision.authorizes_acquisition);
            assert!(!decision.authorizes_install);
            assert!(!decision.authorizes_load);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn safety_critical_denials_are_explicit() {
        let cases = [
            (Mutation::ClaimsInstallable, "install_overclaimed"),
            (
                Mutation::ClaimsLoadAuthorizedByDistribution,
                "distribution_signature_load_authority_overclaimed",
            ),
            (Mutation::ClaimsOwnerSealed, "owner_sealed_overclaimed"),
            (
                Mutation::MissingM6ReverifyForLoad,
                "load_worthiness_not_gated_on_m6_reverify",
            ),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            assert_eq!(
                evaluate_external_acquisition_honesty(&input).reason,
                cases[idx].1
            );
            idx += 1;
        }
    }

    #[test]
    fn empty_source_kind_is_missing_source_kind() {
        let mut input = valid_input();
        input.source_kind = Some("");
        assert_eq!(
            evaluate_external_acquisition_honesty(&input).reason,
            "missing_source_kind"
        );
    }

    #[test]
    fn success_never_authorizes_acquisition_install_or_load() {
        let decision = evaluate_external_acquisition_honesty(&valid_input());
        assert!(decision.honest);
        assert!(!decision.authorizes_acquisition);
        assert!(!decision.authorizes_install);
        assert!(!decision.authorizes_load);
    }
}
