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

pub const KNOWN_HOST_IMPORTS: &[(&str, &str)] = &[
    ("env", "log"),
    ("env", "counter_get"),
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
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
    if !input.policy_allows_beyond_env {
        for (module, _) in input.requested_imports {
            if *module != "env" {
                return denied("import_beyond_env_not_owner_authorized");
            }
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
    fn all_denial_reasons_are_pairwise_unique_including_forward_guard() {
        let reasons = [
            "missing_service_id",
            "missing_artifact_binding",
            "missing_import_list",
            "import_list_exceeds_max",
            "unknown_host_import",
            "import_beyond_env_not_owner_authorized",
            "duplicate_host_import",
        ];
        assert_eq!(reasons.len(), 7);

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
    fn known_host_imports_are_only_current_env_functions() {
        assert_eq!(
            KNOWN_HOST_IMPORTS,
            &[
                ("env", "log"),
                ("env", "counter_get"),
                ("env", "input_len"),
                ("env", "input_read"),
                ("env", "output_write")
            ]
        );
        assert!(KNOWN_HOST_IMPORTS
            .iter()
            .all(|(module, _)| *module == "env"));
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
    fn beyond_env_guard_is_unreachable_until_a_non_env_known_import_exists() {
        let mut input = valid_input();
        input.requested_imports = &[("sys", "x")];
        input.policy_allows_beyond_env = false;

        // Honest current invariant: no non-env host import exists, so the
        // unknown-import pin fires before the forward owner-authorization guard.
        assert!(KNOWN_HOST_IMPORTS
            .iter()
            .all(|(module, _)| *module == "env"));
        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "unknown_host_import"
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
}
