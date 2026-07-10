//! Scoped authorization for per-service Wasm host-import grants (M11-2).
//!
//! This evaluator grants nothing by itself. It decides whether one service's
//! declared import list is the exact host-import surface raiOS may later link
//! for that service. Kernel linker wiring is intentionally left to M11-3.

use alloc::vec;

use crate::record::{sha256_of_json, Field, Value};

pub const SCOPED_WASM_IMPORT_GRANT_DECISION_SCHEMA: &str =
    "raios.scoped_wasm_import_grant_authorization_decision.v0";
pub const SCOPED_WASM_IMPORT_GRANT_DECISION_ID: &str =
    "scoped_wasm_import_grant_authorization.current_boot.v0";
pub const SCOPED_WASM_IMPORT_GRANT_DECISION_MARKER: &str = "RAIOS_WASM_IMPORT_GRANT_SCOPE_DECISION";

pub const PERSONAL_SHELL_SERVICE_ID: &str = "svc.user.shell";
pub const PERSONAL_SHELL_UI_IMPORTS: &[(&str, &str)] = &[
    ("ui", "viewport"),
    ("ui", "context_len"),
    ("ui", "context_read"),
    ("ui", "input_len"),
    ("ui", "input_read"),
    ("ui", "frame_submit"),
];

pub const KNOWN_HOST_IMPORTS: &[(&str, &str)] = &[
    ("env", "log"),
    ("env", "counter_get"),
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
    ("ui", "viewport"),
    ("ui", "context_len"),
    ("ui", "context_read"),
    ("ui", "input_len"),
    ("ui", "input_read"),
    ("ui", "frame_submit"),
];
pub const MAX_GRANTED_IMPORTS: usize = 16;

#[derive(Clone, Copy)]
pub struct WasmImportGrantInput<'a> {
    pub service_id: Option<&'a str>,
    pub artifact_sha256_present: bool,
    pub requested_imports: &'a [(&'a str, &'a str)],
    pub policy_allows_beyond_env: bool,
}

impl<'a> WasmImportGrantInput<'a> {
    pub const fn empty() -> Self {
        Self {
            service_id: None,
            artifact_sha256_present: false,
            requested_imports: &[],
            policy_allows_beyond_env: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmImportGrantDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub authorized_import_count: usize,
}

/// A verified evidence record and the concrete artifact/import list it binds.
///
/// Verification happens in the existing descriptor/attestation path. This
/// evaluator checks that all verified records bind the same exact subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedImportEvidence {
    pub evidence_sha256: [u8; 32],
    pub artifact_sha256: [u8; 32],
    pub import_list_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct PersonalShellImportGrantInput<'a> {
    pub service_id: Option<&'a str>,
    pub artifact_sha256: Option<[u8; 32]>,
    pub descriptor_source_signature_evidence: Option<VerifiedImportEvidence>,
    pub artifact_signature_attestation_evidence: Option<VerifiedImportEvidence>,
    pub computed_grant_evidence: Option<VerifiedImportEvidence>,
    pub declared_import_list_sha256: Option<[u8; 32]>,
    pub requested_imports: &'a [(&'a str, &'a str)],
    pub linker_implementations: &'a [(&'a str, &'a str)],
}

impl<'a> PersonalShellImportGrantInput<'a> {
    pub const fn empty() -> Self {
        Self {
            service_id: None,
            artifact_sha256: None,
            descriptor_source_signature_evidence: None,
            artifact_signature_attestation_evidence: None,
            computed_grant_evidence: None,
            declared_import_list_sha256: None,
            requested_imports: &[],
            linker_implementations: &[],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersonalShellImportGrantDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub authorized_import_count: usize,
    pub authorized_imports: &'static [(&'static str, &'static str)],
    pub authorized_service_id: Option<&'static str>,
    pub artifact_sha256: Option<[u8; 32]>,
    pub descriptor_source_signature_evidence_sha256: Option<[u8; 32]>,
    pub artifact_signature_attestation_evidence_sha256: Option<[u8; 32]>,
    pub computed_grant_evidence_sha256: Option<[u8; 32]>,
    pub authorized_import_list_sha256: Option<[u8; 32]>,
}

pub fn evaluate_wasm_import_grant(input: &WasmImportGrantInput<'_>) -> WasmImportGrantDecision {
    match input.service_id {
        Some(service_id) if !service_id.is_empty() => {}
        _ => return denied("missing_service_id"),
    }
    if !input.artifact_sha256_present {
        return denied("missing_artifact_binding");
    }
    if input.requested_imports.is_empty() {
        return denied("missing_import_list");
    }
    if input.requested_imports.len() > MAX_GRANTED_IMPORTS {
        return denied("import_list_exceeds_max");
    }
    for (module, name) in input.requested_imports {
        if !KNOWN_HOST_IMPORTS.contains(&(*module, *name)) {
            return denied("unknown_host_import");
        }
    }
    for (module, _) in input.requested_imports {
        if *module != "env" {
            return if input.policy_allows_beyond_env {
                denied("evidence_bound_import_grant_required")
            } else {
                denied("import_beyond_env_not_owner_authorized")
            };
        }
    }
    // ponytail: O(n^2) is bounded by MAX_GRANTED_IMPORTS=16; use a set only if the cap grows.
    let mut left = 0usize;
    while left < input.requested_imports.len() {
        let mut right = left + 1;
        while right < input.requested_imports.len() {
            if input.requested_imports[left] == input.requested_imports[right] {
                return denied("duplicate_host_import");
            }
            right += 1;
        }
        left += 1;
    }

    WasmImportGrantDecision {
        performed: true,
        status: "import_grant_authorized",
        reason: "authorized_exact_declared_import_surface",
        authorized_import_count: input.requested_imports.len(),
    }
}

/// Authorizes only the owner-approved, evidence-bound personal-shell UI surface.
/// Existing `env` services continue to use `evaluate_wasm_import_grant` unchanged.
pub fn evaluate_personal_shell_import_grant(
    input: &PersonalShellImportGrantInput<'_>,
) -> PersonalShellImportGrantDecision {
    match input.service_id {
        None | Some("") => return personal_shell_denied("missing_service_id"),
        Some(PERSONAL_SHELL_SERVICE_ID) => {}
        Some(_) => return personal_shell_denied("wrong_service_id"),
    }
    let Some(artifact_sha256) = input.artifact_sha256 else {
        return personal_shell_denied("missing_artifact_sha256");
    };

    if input.requested_imports.is_empty() {
        return personal_shell_denied("missing_import_list");
    }
    if input.requested_imports.len() > MAX_GRANTED_IMPORTS {
        return personal_shell_denied("import_list_exceeds_max");
    }
    if has_duplicate(input.requested_imports) {
        return personal_shell_denied("duplicate_host_import");
    }
    if let Some(reason) = exact_list_mismatch_reason(
        input.requested_imports,
        PERSONAL_SHELL_UI_IMPORTS,
        "personal_shell_import_subset",
        "personal_shell_import_superset",
        "personal_shell_import_order_mismatch",
        "personal_shell_import_surface_mismatch",
    ) {
        return personal_shell_denied(reason);
    }

    let import_list_sha256 =
        authorized_import_list_sha256(PERSONAL_SHELL_SERVICE_ID, input.requested_imports);
    match input.declared_import_list_sha256 {
        None => return personal_shell_denied("missing_declared_import_list_sha256"),
        Some(declared) if declared != import_list_sha256 => {
            return personal_shell_denied("declared_import_list_sha256_mismatch")
        }
        Some(_) => {}
    }

    let Some(descriptor_evidence) = input.descriptor_source_signature_evidence else {
        return personal_shell_denied("missing_descriptor_source_signature_evidence");
    };
    if descriptor_evidence.artifact_sha256 != artifact_sha256 {
        return personal_shell_denied("descriptor_evidence_artifact_mismatch");
    }
    if descriptor_evidence.import_list_sha256 != import_list_sha256 {
        return personal_shell_denied("descriptor_evidence_import_list_mismatch");
    }

    let Some(artifact_evidence) = input.artifact_signature_attestation_evidence else {
        return personal_shell_denied("missing_artifact_signature_attestation_evidence");
    };
    if artifact_evidence.artifact_sha256 != artifact_sha256 {
        return personal_shell_denied("artifact_evidence_artifact_mismatch");
    }
    if artifact_evidence.import_list_sha256 != import_list_sha256 {
        return personal_shell_denied("artifact_evidence_import_list_mismatch");
    }

    if let Some(computed_evidence) = input.computed_grant_evidence {
        if computed_evidence.artifact_sha256 != artifact_sha256 {
            return personal_shell_denied("computed_grant_evidence_artifact_mismatch");
        }
        if computed_evidence.import_list_sha256 != import_list_sha256 {
            return personal_shell_denied("computed_grant_evidence_import_list_mismatch");
        }
    }

    if input.linker_implementations.is_empty() {
        return personal_shell_denied("missing_linker_implementations");
    }
    if input.linker_implementations.len() > MAX_GRANTED_IMPORTS {
        return personal_shell_denied("linker_implementation_list_exceeds_max");
    }
    if has_duplicate(input.linker_implementations) {
        return personal_shell_denied("duplicate_linker_implementation");
    }
    if let Some(reason) = exact_list_mismatch_reason(
        input.linker_implementations,
        PERSONAL_SHELL_UI_IMPORTS,
        "linker_implementation_subset",
        "linker_implementation_superset",
        "linker_implementation_order_mismatch",
        "linker_implementation_surface_mismatch",
    ) {
        return personal_shell_denied(reason);
    }

    PersonalShellImportGrantDecision {
        performed: true,
        status: "import_grant_authorized",
        reason: "authorized_evidence_bound_personal_shell_ui_surface",
        authorized_import_count: PERSONAL_SHELL_UI_IMPORTS.len(),
        authorized_imports: PERSONAL_SHELL_UI_IMPORTS,
        authorized_service_id: Some(PERSONAL_SHELL_SERVICE_ID),
        artifact_sha256: Some(artifact_sha256),
        descriptor_source_signature_evidence_sha256: Some(descriptor_evidence.evidence_sha256),
        artifact_signature_attestation_evidence_sha256: Some(artifact_evidence.evidence_sha256),
        computed_grant_evidence_sha256: input
            .computed_grant_evidence
            .map(|evidence| evidence.evidence_sha256),
        authorized_import_list_sha256: Some(import_list_sha256),
    }
}

fn has_duplicate(imports: &[(&str, &str)]) -> bool {
    // ponytail: both import surfaces are hard-capped at 16; a set would add machinery.
    let mut left = 0usize;
    while left < imports.len() {
        let mut right = left + 1;
        while right < imports.len() {
            if imports[left] == imports[right] {
                return true;
            }
            right += 1;
        }
        left += 1;
    }
    false
}

fn exact_list_mismatch_reason(
    actual: &[(&str, &str)],
    expected: &[(&str, &str)],
    subset_reason: &'static str,
    superset_reason: &'static str,
    order_reason: &'static str,
    surface_reason: &'static str,
) -> Option<&'static str> {
    if actual.len() < expected.len() {
        return Some(subset_reason);
    }
    if actual.len() > expected.len() {
        return Some(superset_reason);
    }
    if actual == expected {
        return None;
    }
    if actual.iter().all(|item| expected.contains(item)) {
        Some(order_reason)
    } else {
        Some(surface_reason)
    }
}

fn personal_shell_denied(reason: &'static str) -> PersonalShellImportGrantDecision {
    PersonalShellImportGrantDecision {
        performed: false,
        status: "denied",
        reason,
        authorized_import_count: 0,
        authorized_imports: &[],
        authorized_service_id: None,
        artifact_sha256: None,
        descriptor_source_signature_evidence_sha256: None,
        artifact_signature_attestation_evidence_sha256: None,
        computed_grant_evidence_sha256: None,
        authorized_import_list_sha256: None,
    }
}

fn denied(reason: &'static str) -> WasmImportGrantDecision {
    WasmImportGrantDecision {
        performed: false,
        status: "denied",
        reason,
        authorized_import_count: 0,
    }
}

/// Hashes the declared import list for later equality checks.
///
/// This authorizes nothing by itself. It only gives M11-3 a canonical digest
/// that can bind the declared list and the linked list to the same bytes.
pub fn authorized_import_list_sha256(
    service_id: &str,
    requested_imports: &[(&str, &str)],
) -> [u8; 32] {
    let imports = requested_imports
        .iter()
        .map(|(module, name)| {
            Value::Object(vec![
                Field::new("module", Value::Str(module)),
                Field::new("name", Value::Str(name)),
            ])
        })
        .collect();
    sha256_of_json(&Value::Object(vec![
        Field::new("service_id", Value::Str(service_id)),
        Field::new("imports", Value::Array(imports)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> WasmImportGrantInput<'static> {
        WasmImportGrantInput {
            service_id: Some("svc.demo.echo"),
            artifact_sha256_present: true,
            requested_imports: &[("env", "log")],
            policy_allows_beyond_env: false,
        }
    }

    #[test]
    fn valid_input_authorizes_import_grant() {
        assert_eq!(
            evaluate_wasm_import_grant(&valid_input()),
            WasmImportGrantDecision {
                performed: true,
                status: "import_grant_authorized",
                reason: "authorized_exact_declared_import_surface",
                authorized_import_count: 1,
            }
        );
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingServiceId,
        MissingArtifactBinding,
        MissingImportList,
        ImportListExceedsMax,
        UnknownHostImport,
        DuplicateHostImport,
    }

    fn apply(input: &mut WasmImportGrantInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingServiceId => input.service_id = None,
            Mutation::MissingArtifactBinding => input.artifact_sha256_present = false,
            Mutation::MissingImportList => input.requested_imports = &[],
            Mutation::ImportListExceedsMax => {
                input.requested_imports = &[
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                    ("env", "log"),
                ];
            }
            Mutation::UnknownHostImport => input.requested_imports = &[("net", "tcp_open")],
            Mutation::DuplicateHostImport => {
                input.requested_imports = &[("env", "log"), ("env", "log")];
            }
        }
    }

    #[test]
    fn denial_truth_table_names_first_failed_pin() {
        let cases = [
            (Mutation::MissingServiceId, "missing_service_id"),
            (Mutation::MissingArtifactBinding, "missing_artifact_binding"),
            (Mutation::MissingImportList, "missing_import_list"),
            (Mutation::ImportListExceedsMax, "import_list_exceeds_max"),
            (Mutation::UnknownHostImport, "unknown_host_import"),
            (Mutation::DuplicateHostImport, "duplicate_host_import"),
        ];
        assert_eq!(cases.len(), 6);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_wasm_import_grant(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);
            assert_eq!(decision.authorized_import_count, 0);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn all_legacy_denial_reasons_are_pairwise_unique() {
        let reasons = [
            "missing_service_id",
            "missing_artifact_binding",
            "missing_import_list",
            "import_list_exceeds_max",
            "unknown_host_import",
            "import_beyond_env_not_owner_authorized",
            "evidence_bound_import_grant_required",
            "duplicate_host_import",
        ];
        assert_eq!(reasons.len(), 8);

        let mut idx = 0usize;
        while idx < reasons.len() {
            let mut other = 0usize;
            while other < idx {
                assert_ne!(reasons[idx], reasons[other]);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn empty_service_id_is_missing_service_id() {
        let mut input = valid_input();
        input.service_id = Some("");
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "missing_service_id"
        );
    }

    #[test]
    fn known_host_imports_are_existing_env_plus_exact_personal_ui_surface() {
        assert_eq!(
            KNOWN_HOST_IMPORTS,
            &[
                ("env", "log"),
                ("env", "counter_get"),
                ("env", "input_len"),
                ("env", "input_read"),
                ("env", "output_write"),
                ("ui", "viewport"),
                ("ui", "context_len"),
                ("ui", "context_read"),
                ("ui", "input_len"),
                ("ui", "input_read"),
                ("ui", "frame_submit"),
            ]
        );
        assert_eq!(
            PERSONAL_SHELL_UI_IMPORTS,
            &KNOWN_HOST_IMPORTS[KNOWN_HOST_IMPORTS.len() - 6..]
        );
    }

    #[test]
    fn buffer_data_channel_imports_authorize_only_when_declared() {
        let mut input = valid_input();
        input.requested_imports = &[
            ("env", "input_len"),
            ("env", "input_read"),
            ("env", "output_write"),
        ];
        assert_eq!(
            evaluate_wasm_import_grant(&input),
            WasmImportGrantDecision {
                performed: true,
                status: "import_grant_authorized",
                reason: "authorized_exact_declared_import_surface",
                authorized_import_count: 3,
            }
        );
    }

    #[test]
    fn every_existing_service_keeps_its_current_env_surface() {
        let cases = [
            (
                "svc.demo.echo",
                &[("env", "log"), ("env", "counter_get")][..],
            ),
            (
                "svc.demo.bufecho",
                &[
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                ][..],
            ),
            (
                "svc.demo.certwindow",
                &[
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                ][..],
            ),
            (
                "svc.demo.httphead",
                &[
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                ][..],
            ),
            (
                "svc.demo.certspki",
                &[
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                ][..],
            ),
        ];

        for (service_id, requested_imports) in cases {
            let decision = evaluate_wasm_import_grant(&WasmImportGrantInput {
                service_id: Some(service_id),
                artifact_sha256_present: true,
                requested_imports,
                policy_allows_beyond_env: false,
            });
            assert!(decision.performed, "{}", service_id);
            assert_eq!(decision.authorized_import_count, requested_imports.len());
        }
    }

    #[test]
    fn input_read_import_authorizes_alone() {
        let mut input = valid_input();
        input.requested_imports = &[("env", "input_read")];
        assert_eq!(
            evaluate_wasm_import_grant(&input),
            WasmImportGrantDecision {
                performed: true,
                status: "import_grant_authorized",
                reason: "authorized_exact_declared_import_surface",
                authorized_import_count: 1,
            }
        );
    }

    #[test]
    fn unknown_host_import_is_fail_closed() {
        let mut input = valid_input();
        input.requested_imports = &[("net", "x")];
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "unknown_host_import"
        );
    }

    #[test]
    fn legacy_boolean_never_authorizes_the_personal_ui_surface() {
        let mut input = valid_input();
        input.service_id = Some(PERSONAL_SHELL_SERVICE_ID);
        input.requested_imports = PERSONAL_SHELL_UI_IMPORTS;
        input.policy_allows_beyond_env = false;

        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "import_beyond_env_not_owner_authorized"
        );
        input.policy_allows_beyond_env = true;
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "evidence_bound_import_grant_required"
        );
    }

    #[test]
    fn duplicate_import_is_denied() {
        let mut input = valid_input();
        input.requested_imports = &[("env", "log"), ("env", "log")];
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "duplicate_host_import"
        );
    }

    #[test]
    fn empty_import_list_is_denied() {
        let mut input = valid_input();
        input.requested_imports = &[];
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "missing_import_list"
        );
    }

    #[test]
    fn missing_artifact_binding_is_denied() {
        let mut input = valid_input();
        input.artifact_sha256_present = false;
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "missing_artifact_binding"
        );
    }

    #[test]
    fn authorized_import_list_sha256_is_deterministic_order_sensitive_and_grants_nothing() {
        let first = authorized_import_list_sha256(
            "svc.demo.echo",
            &[("env", "log"), ("env", "counter_get")],
        );
        let same = authorized_import_list_sha256(
            "svc.demo.echo",
            &[("env", "log"), ("env", "counter_get")],
        );
        let reordered = authorized_import_list_sha256(
            "svc.demo.echo",
            &[("env", "counter_get"), ("env", "log")],
        );

        assert_eq!(first, same);
        assert_ne!(first, reordered);
        assert_eq!(
            evaluate_wasm_import_grant(&WasmImportGrantInput::empty()).reason,
            "missing_service_id"
        );
    }

    #[test]
    fn echo_authorized_import_hash_stays_pinned_and_differs_from_bufecho() {
        let echo = authorized_import_list_sha256(
            "svc.demo.echo",
            &[("env", "log"), ("env", "counter_get")],
        );
        let bufecho = authorized_import_list_sha256(
            "svc.demo.bufecho",
            &[
                ("env", "input_len"),
                ("env", "input_read"),
                ("env", "output_write"),
            ],
        );

        assert_eq!(
            crate::sha256_hex(&echo),
            *b"b314db1d2b23903140a060975c281d5a78c43b0f695019c6730fe679f80bcb66"
        );
        assert_ne!(echo, bufecho);
    }

    const PERSONAL_ARTIFACT_SHA256: [u8; 32] = [0x11; 32];

    fn personal_shell_input() -> PersonalShellImportGrantInput<'static> {
        let import_list_sha256 =
            authorized_import_list_sha256(PERSONAL_SHELL_SERVICE_ID, PERSONAL_SHELL_UI_IMPORTS);
        PersonalShellImportGrantInput {
            service_id: Some(PERSONAL_SHELL_SERVICE_ID),
            artifact_sha256: Some(PERSONAL_ARTIFACT_SHA256),
            descriptor_source_signature_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x22; 32],
                artifact_sha256: PERSONAL_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            artifact_signature_attestation_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x33; 32],
                artifact_sha256: PERSONAL_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            computed_grant_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x44; 32],
                artifact_sha256: PERSONAL_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            declared_import_list_sha256: Some(import_list_sha256),
            requested_imports: PERSONAL_SHELL_UI_IMPORTS,
            linker_implementations: PERSONAL_SHELL_UI_IMPORTS,
        }
    }

    #[test]
    fn exact_evidence_bound_personal_shell_surface_is_authorized() {
        let input = personal_shell_input();
        let expected_import_hash =
            authorized_import_list_sha256(PERSONAL_SHELL_SERVICE_ID, PERSONAL_SHELL_UI_IMPORTS);

        assert_eq!(
            evaluate_personal_shell_import_grant(&input),
            PersonalShellImportGrantDecision {
                performed: true,
                status: "import_grant_authorized",
                reason: "authorized_evidence_bound_personal_shell_ui_surface",
                authorized_import_count: 6,
                authorized_imports: PERSONAL_SHELL_UI_IMPORTS,
                authorized_service_id: Some(PERSONAL_SHELL_SERVICE_ID),
                artifact_sha256: Some(PERSONAL_ARTIFACT_SHA256),
                descriptor_source_signature_evidence_sha256: Some([0x22; 32]),
                artifact_signature_attestation_evidence_sha256: Some([0x33; 32]),
                computed_grant_evidence_sha256: Some([0x44; 32]),
                authorized_import_list_sha256: Some(expected_import_hash),
            }
        );
    }

    #[test]
    fn optional_computed_grant_is_bound_when_present_but_not_invented() {
        let mut input = personal_shell_input();
        input.computed_grant_evidence = None;

        let decision = evaluate_personal_shell_import_grant(&input);

        assert!(decision.performed);
        assert_eq!(decision.computed_grant_evidence_sha256, None);
    }

    #[derive(Clone, Copy)]
    enum PersonalMutation {
        MissingService,
        WrongService,
        MissingArtifact,
        MissingImports,
        TooManyImports,
        DuplicateImport,
        ImportSubset,
        ImportSuperset,
        ImportReorder,
        ImportSurface,
        MissingDeclaredHash,
        WrongDeclaredHash,
        MissingDescriptorEvidence,
        DescriptorArtifactMismatch,
        DescriptorImportMismatch,
        MissingArtifactEvidence,
        ArtifactArtifactMismatch,
        ArtifactImportMismatch,
        ComputedArtifactMismatch,
        ComputedImportMismatch,
        MissingLinkerImplementations,
        TooManyLinkerImplementations,
        DuplicateLinkerImplementation,
        LinkerSubset,
        LinkerSuperset,
        LinkerReorder,
        LinkerSurface,
    }

    fn mutate_personal_input(
        input: &mut PersonalShellImportGrantInput<'static>,
        mutation: PersonalMutation,
    ) {
        match mutation {
            PersonalMutation::MissingService => input.service_id = None,
            PersonalMutation::WrongService => input.service_id = Some("svc.demo.echo"),
            PersonalMutation::MissingArtifact => input.artifact_sha256 = None,
            PersonalMutation::MissingImports => input.requested_imports = &[],
            PersonalMutation::TooManyImports => {
                input.requested_imports = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                    ("env", "log"),
                    ("env", "counter_get"),
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                    ("x", "1"),
                    ("x", "2"),
                    ("x", "3"),
                    ("x", "4"),
                    ("x", "5"),
                    ("x", "6"),
                ];
            }
            PersonalMutation::DuplicateImport => {
                input.requested_imports = &[
                    ("ui", "viewport"),
                    ("ui", "viewport"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                ];
            }
            PersonalMutation::ImportSubset => {
                input.requested_imports = &PERSONAL_SHELL_UI_IMPORTS[..5]
            }
            PersonalMutation::ImportSuperset => {
                input.requested_imports = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                    ("env", "log"),
                ];
            }
            PersonalMutation::ImportReorder => {
                input.requested_imports = &[
                    ("ui", "context_len"),
                    ("ui", "viewport"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                ];
            }
            PersonalMutation::ImportSurface => {
                input.requested_imports = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("env", "log"),
                ];
            }
            PersonalMutation::MissingDeclaredHash => input.declared_import_list_sha256 = None,
            PersonalMutation::WrongDeclaredHash => {
                input.declared_import_list_sha256 = Some([0xff; 32])
            }
            PersonalMutation::MissingDescriptorEvidence => {
                input.descriptor_source_signature_evidence = None
            }
            PersonalMutation::DescriptorArtifactMismatch => {
                input
                    .descriptor_source_signature_evidence
                    .as_mut()
                    .unwrap()
                    .artifact_sha256 = [0xff; 32]
            }
            PersonalMutation::DescriptorImportMismatch => {
                input
                    .descriptor_source_signature_evidence
                    .as_mut()
                    .unwrap()
                    .import_list_sha256 = [0xff; 32]
            }
            PersonalMutation::MissingArtifactEvidence => {
                input.artifact_signature_attestation_evidence = None
            }
            PersonalMutation::ArtifactArtifactMismatch => {
                input
                    .artifact_signature_attestation_evidence
                    .as_mut()
                    .unwrap()
                    .artifact_sha256 = [0xff; 32]
            }
            PersonalMutation::ArtifactImportMismatch => {
                input
                    .artifact_signature_attestation_evidence
                    .as_mut()
                    .unwrap()
                    .import_list_sha256 = [0xff; 32]
            }
            PersonalMutation::ComputedArtifactMismatch => {
                input
                    .computed_grant_evidence
                    .as_mut()
                    .unwrap()
                    .artifact_sha256 = [0xff; 32]
            }
            PersonalMutation::ComputedImportMismatch => {
                input
                    .computed_grant_evidence
                    .as_mut()
                    .unwrap()
                    .import_list_sha256 = [0xff; 32]
            }
            PersonalMutation::MissingLinkerImplementations => input.linker_implementations = &[],
            PersonalMutation::TooManyLinkerImplementations => {
                input.linker_implementations = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                    ("env", "log"),
                    ("env", "counter_get"),
                    ("env", "input_len"),
                    ("env", "input_read"),
                    ("env", "output_write"),
                    ("x", "1"),
                    ("x", "2"),
                    ("x", "3"),
                    ("x", "4"),
                    ("x", "5"),
                    ("x", "6"),
                ];
            }
            PersonalMutation::DuplicateLinkerImplementation => {
                input.linker_implementations = &[
                    ("ui", "viewport"),
                    ("ui", "viewport"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                ];
            }
            PersonalMutation::LinkerSubset => {
                input.linker_implementations = &PERSONAL_SHELL_UI_IMPORTS[..5]
            }
            PersonalMutation::LinkerSuperset => {
                input.linker_implementations = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                    ("env", "log"),
                ];
            }
            PersonalMutation::LinkerReorder => {
                input.linker_implementations = &[
                    ("ui", "context_len"),
                    ("ui", "viewport"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("ui", "frame_submit"),
                ];
            }
            PersonalMutation::LinkerSurface => {
                input.linker_implementations = &[
                    ("ui", "viewport"),
                    ("ui", "context_len"),
                    ("ui", "context_read"),
                    ("ui", "input_len"),
                    ("ui", "input_read"),
                    ("env", "log"),
                ];
            }
        }
    }

    #[test]
    fn personal_shell_denials_are_reachable_and_pairwise_unique() {
        let cases = [
            (PersonalMutation::MissingService, "missing_service_id"),
            (PersonalMutation::WrongService, "wrong_service_id"),
            (PersonalMutation::MissingArtifact, "missing_artifact_sha256"),
            (PersonalMutation::MissingImports, "missing_import_list"),
            (PersonalMutation::TooManyImports, "import_list_exceeds_max"),
            (PersonalMutation::DuplicateImport, "duplicate_host_import"),
            (
                PersonalMutation::ImportSubset,
                "personal_shell_import_subset",
            ),
            (
                PersonalMutation::ImportSuperset,
                "personal_shell_import_superset",
            ),
            (
                PersonalMutation::ImportReorder,
                "personal_shell_import_order_mismatch",
            ),
            (
                PersonalMutation::ImportSurface,
                "personal_shell_import_surface_mismatch",
            ),
            (
                PersonalMutation::MissingDeclaredHash,
                "missing_declared_import_list_sha256",
            ),
            (
                PersonalMutation::WrongDeclaredHash,
                "declared_import_list_sha256_mismatch",
            ),
            (
                PersonalMutation::MissingDescriptorEvidence,
                "missing_descriptor_source_signature_evidence",
            ),
            (
                PersonalMutation::DescriptorArtifactMismatch,
                "descriptor_evidence_artifact_mismatch",
            ),
            (
                PersonalMutation::DescriptorImportMismatch,
                "descriptor_evidence_import_list_mismatch",
            ),
            (
                PersonalMutation::MissingArtifactEvidence,
                "missing_artifact_signature_attestation_evidence",
            ),
            (
                PersonalMutation::ArtifactArtifactMismatch,
                "artifact_evidence_artifact_mismatch",
            ),
            (
                PersonalMutation::ArtifactImportMismatch,
                "artifact_evidence_import_list_mismatch",
            ),
            (
                PersonalMutation::ComputedArtifactMismatch,
                "computed_grant_evidence_artifact_mismatch",
            ),
            (
                PersonalMutation::ComputedImportMismatch,
                "computed_grant_evidence_import_list_mismatch",
            ),
            (
                PersonalMutation::MissingLinkerImplementations,
                "missing_linker_implementations",
            ),
            (
                PersonalMutation::TooManyLinkerImplementations,
                "linker_implementation_list_exceeds_max",
            ),
            (
                PersonalMutation::DuplicateLinkerImplementation,
                "duplicate_linker_implementation",
            ),
            (
                PersonalMutation::LinkerSubset,
                "linker_implementation_subset",
            ),
            (
                PersonalMutation::LinkerSuperset,
                "linker_implementation_superset",
            ),
            (
                PersonalMutation::LinkerReorder,
                "linker_implementation_order_mismatch",
            ),
            (
                PersonalMutation::LinkerSurface,
                "linker_implementation_surface_mismatch",
            ),
        ];

        let mut index = 0usize;
        while index < cases.len() {
            let mut input = personal_shell_input();
            mutate_personal_input(&mut input, cases[index].0);
            let decision = evaluate_personal_shell_import_grant(&input);
            assert!(!decision.performed);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[index].1);
            assert_eq!(decision.authorized_import_count, 0);
            assert!(decision.authorized_imports.is_empty());
            assert_eq!(decision.artifact_sha256, None);

            let mut previous = 0usize;
            while previous < index {
                assert_ne!(cases[index].1, cases[previous].1);
                previous += 1;
            }
            index += 1;
        }
    }
}
