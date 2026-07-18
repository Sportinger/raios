//! Scoped authorization for provider-minimal context export (M9C-2a).
//!
//! This evaluator grants nothing by itself. It is the raios-core-only,
//! host-tested gate that decides whether the exact `provider_minimal` context
//! packet may leave the machine and whether the export-audit binding is
//! complete. Kernel wiring is intentionally left to a later slice.

pub const SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA: &str =
    "raios.scoped_provider_export_authorization_decision.v0";
pub const SCOPED_PROVIDER_EXPORT_DECISION_ID: &str =
    "scoped_provider_export_authorization.current_boot.provider_minimal.v0";
pub const SCOPED_PROVIDER_EXPORT_DECISION_MARKER: &str = "RAIOS_PROVIDER_EXPORT_SCOPE_DECISION";

pub const EXPECTED_METHOD: &str = "provider.context_export";
pub const EXPECTED_PROFILE: &str = "provider_minimal";
pub const POSITIVE_TRUST_STATES: &[&str] = &[
    "pinned_cert_verified",
    "pinned_spki_verified",
    "webpki_verified",
];

#[derive(Clone, Copy)]
pub struct ProviderExportGateInput<'a> {
    pub method: Option<&'a str>,
    pub profile: Option<&'a str>,
    pub trust_state: Option<&'a str>,
    pub tls_certificate_verification_bypassed: bool,
    pub packet_all_records_public: bool,
    pub packet_record_count: Option<u64>,
    pub budget_tokens: Option<u64>,
    pub packet_estimated_tokens: Option<u64>,
    pub audit_packet_hash: Option<[u8; 32]>,
    pub audit_destination: Option<&'a str>,
    pub audit_trust_snapshot_present: bool,
    pub audit_binding_hash: Option<[u8; 32]>,
}

impl<'a> ProviderExportGateInput<'a> {
    pub const fn empty() -> Self {
        Self {
            method: None,
            profile: None,
            trust_state: None,
            tls_certificate_verification_bypassed: false,
            packet_all_records_public: false,
            packet_record_count: None,
            budget_tokens: None,
            packet_estimated_tokens: None,
            audit_packet_hash: None,
            audit_destination: None,
            audit_trust_snapshot_present: false,
            audit_binding_hash: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderExportGateDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
}

pub fn evaluate_provider_export_gate(
    input: &ProviderExportGateInput<'_>,
) -> ProviderExportGateDecision {
    if let Err(decision) = require_str(
        input.method,
        EXPECTED_METHOD,
        "missing_method",
        "method_not_provider_context_export",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.profile,
        EXPECTED_PROFILE,
        "missing_profile",
        "profile_not_provider_minimal",
    ) {
        return decision;
    }

    let trust_state = match input.trust_state {
        Some(value) => value,
        None => return denied("missing_trust_state"),
    };
    if input.tls_certificate_verification_bypassed {
        return denied("trust_dev_bypass_not_positive");
    }
    if !POSITIVE_TRUST_STATES.contains(&trust_state) {
        return denied("trust_state_not_positive");
    }

    if !input.packet_all_records_public {
        return denied("packet_contains_non_public_record");
    }
    if input.packet_record_count.is_none() {
        return denied("missing_packet_record_count");
    }

    let budget_tokens = match input.budget_tokens {
        Some(value) => value,
        None => return denied("missing_budget_tokens"),
    };
    let packet_estimated_tokens = match input.packet_estimated_tokens {
        Some(value) => value,
        None => return denied("missing_packet_estimated_tokens"),
    };
    if packet_estimated_tokens > budget_tokens {
        return denied("packet_exceeds_token_budget");
    }

    if input.audit_packet_hash.is_none() {
        return denied("missing_audit_packet_hash");
    }
    match input.audit_destination {
        Some(value) if !value.is_empty() => {}
        _ => return denied("missing_audit_destination"),
    }
    if !input.audit_trust_snapshot_present {
        return denied("missing_audit_trust_snapshot");
    }
    if input.audit_binding_hash.is_none() {
        return denied("missing_audit_binding_hash");
    }

    ProviderExportGateDecision {
        performed: true,
        status: "export_authorized",
        reason: "authorized_provider_export_public_only_audited",
    }
}

fn require_str(
    actual: Option<&str>,
    expected: &str,
    missing: &'static str,
    mismatch: &'static str,
) -> Result<(), ProviderExportGateDecision> {
    match actual {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err(denied(mismatch)),
        None => Err(denied(missing)),
    }
}

fn denied(reason: &'static str) -> ProviderExportGateDecision {
    ProviderExportGateDecision {
        performed: false,
        status: "denied",
        reason,
    }
}

/// Returns the per-boot denial-audit dedupe key shape: exact gate id plus reason.
///
/// The pair is allocation-free and collision-free; callers own any storage or
/// hashing needed for their dedupe table.
pub fn export_denial_dedupe_key<'a>(gate: &'a str, reason: &'a str) -> (&'a str, &'a str) {
    (gate, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn valid_input() -> ProviderExportGateInput<'static> {
        ProviderExportGateInput {
            method: Some(EXPECTED_METHOD),
            profile: Some(EXPECTED_PROFILE),
            trust_state: Some("pinned_spki_verified"),
            tls_certificate_verification_bypassed: false,
            packet_all_records_public: true,
            packet_record_count: Some(3),
            budget_tokens: Some(3000),
            packet_estimated_tokens: Some(1200),
            audit_packet_hash: Some(h(1)),
            audit_destination: Some("provider.openai.responses"),
            audit_trust_snapshot_present: true,
            audit_binding_hash: Some(h(2)),
        }
    }

    #[test]
    fn authorizes_valid_provider_minimal_export() {
        assert_eq!(
            evaluate_provider_export_gate(&valid_input()),
            ProviderExportGateDecision {
                performed: true,
                status: "export_authorized",
                reason: "authorized_provider_export_public_only_audited"
            }
        );
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingMethod,
        WrongMethod,
        MissingProfile,
        WrongProfile,
        MissingTrustState,
        TlsBypass,
        TrustStateNotPositive,
        NonPublicRecord,
        MissingRecordCount,
        MissingBudget,
        MissingEstimatedTokens,
        ExceedsBudget,
        MissingAuditPacketHash,
        MissingAuditDestination,
        MissingTrustSnapshot,
        MissingAuditBindingHash,
    }

    fn apply(input: &mut ProviderExportGateInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingMethod => input.method = None,
            Mutation::WrongMethod => input.method = Some("memory.context"),
            Mutation::MissingProfile => input.profile = None,
            Mutation::WrongProfile => input.profile = Some("diagnostic"),
            Mutation::MissingTrustState => input.trust_state = None,
            Mutation::TlsBypass => input.tls_certificate_verification_bypassed = true,
            Mutation::TrustStateNotPositive => input.trust_state = Some("pin_config_missing"),
            Mutation::NonPublicRecord => input.packet_all_records_public = false,
            Mutation::MissingRecordCount => input.packet_record_count = None,
            Mutation::MissingBudget => input.budget_tokens = None,
            Mutation::MissingEstimatedTokens => input.packet_estimated_tokens = None,
            Mutation::ExceedsBudget => input.packet_estimated_tokens = Some(3001),
            Mutation::MissingAuditPacketHash => input.audit_packet_hash = None,
            Mutation::MissingAuditDestination => input.audit_destination = None,
            Mutation::MissingTrustSnapshot => input.audit_trust_snapshot_present = false,
            Mutation::MissingAuditBindingHash => input.audit_binding_hash = None,
        }
    }

    #[test]
    fn denial_truth_table_names_first_failed_pin() {
        let cases = [
            (Mutation::MissingMethod, "missing_method"),
            (Mutation::WrongMethod, "method_not_provider_context_export"),
            (Mutation::MissingProfile, "missing_profile"),
            (Mutation::WrongProfile, "profile_not_provider_minimal"),
            (Mutation::MissingTrustState, "missing_trust_state"),
            (Mutation::TlsBypass, "trust_dev_bypass_not_positive"),
            (Mutation::TrustStateNotPositive, "trust_state_not_positive"),
            (
                Mutation::NonPublicRecord,
                "packet_contains_non_public_record",
            ),
            (Mutation::MissingRecordCount, "missing_packet_record_count"),
            (Mutation::MissingBudget, "missing_budget_tokens"),
            (
                Mutation::MissingEstimatedTokens,
                "missing_packet_estimated_tokens",
            ),
            (Mutation::ExceedsBudget, "packet_exceeds_token_budget"),
            (
                Mutation::MissingAuditPacketHash,
                "missing_audit_packet_hash",
            ),
            (
                Mutation::MissingAuditDestination,
                "missing_audit_destination",
            ),
            (
                Mutation::MissingTrustSnapshot,
                "missing_audit_trust_snapshot",
            ),
            (
                Mutation::MissingAuditBindingHash,
                "missing_audit_binding_hash",
            ),
        ];
        assert_eq!(cases.len(), 16);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_provider_export_gate(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn empty_audit_destination_is_missing_destination() {
        let mut input = valid_input();
        input.audit_destination = Some("");
        assert_eq!(
            evaluate_provider_export_gate(&input).reason,
            "missing_audit_destination"
        );
    }

    #[test]
    fn security_critical_denials_are_fail_closed() {
        let cases = [
            (Mutation::TlsBypass, "trust_dev_bypass_not_positive"),
            (Mutation::TrustStateNotPositive, "trust_state_not_positive"),
            (
                Mutation::NonPublicRecord,
                "packet_contains_non_public_record",
            ),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            assert_eq!(evaluate_provider_export_gate(&input).reason, cases[idx].1);
            idx += 1;
        }
    }

    #[test]
    fn estimated_tokens_equal_to_budget_is_authorized() {
        // Boundary: estimated == budget must be allowed (spec uses strict `>`).
        // Guards against a future `>` -> `>=` regression that would falsely deny
        // legal at-budget exports. A `<` regression is already caught by ExceedsBudget.
        let mut input = valid_input();
        input.budget_tokens = Some(3000);
        input.packet_estimated_tokens = Some(3000);
        assert_eq!(
            evaluate_provider_export_gate(&input),
            ProviderExportGateDecision {
                performed: true,
                status: "export_authorized",
                reason: "authorized_provider_export_public_only_audited"
            }
        );
    }

    #[test]
    fn export_denial_dedupe_key_is_gate_reason_pair() {
        assert_eq!(
            export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, "missing_method"),
            export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, "missing_method")
        );
        assert_ne!(
            export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, "missing_method"),
            export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, "missing_profile")
        );
        assert_ne!(
            export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, "missing_method"),
            export_denial_dedupe_key("other_gate", "missing_method")
        );
    }
}
