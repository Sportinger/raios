//! Scoped authorization for per-service Wasm host-import grants (M11-2).
//!
//! This evaluator grants nothing by itself. It decides whether one service's
//! declared import list is the exact host-import surface raiOS may later link
//! for that service. Kernel linker wiring is intentionally left to M11-3.

use alloc::vec;

use crate::{
    host_import_abi_v1::{
        host_import_abi_ordered_list_sha256, ACQUIRE_CHUNK_ACCEPT, ACQUIRE_FINALIZE,
        CRYPTO_P256_VERIFY, CRYPTO_SHA256, CRYPTO_TLS13_AEAD_OPEN, CRYPTO_TLS13_AEAD_SEAL,
        CRYPTO_TLS13_APPLICATION_KEYS, CRYPTO_TLS13_FINISHED, CRYPTO_TLS13_HANDSHAKE_KEYS,
        CRYPTO_TLS13_SESSION_OPEN, HOST_IMPORT_ABI_V1, NET_TCP_CLOSE, NET_TCP_OPEN, NET_TCP_RECV,
        NET_TCP_SEND, SECRET_LEASE_OPENAI_AUTHORIZATION_SEND, TIME_MONOTONIC_MS,
    },
    record::{sha256_of_json, Field, Value},
};

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
    NET_TCP_OPEN.pair(),
    NET_TCP_SEND.pair(),
    NET_TCP_RECV.pair(),
    NET_TCP_CLOSE.pair(),
    CRYPTO_TLS13_SESSION_OPEN.pair(),
    CRYPTO_SHA256.pair(),
    CRYPTO_P256_VERIFY.pair(),
    CRYPTO_TLS13_HANDSHAKE_KEYS.pair(),
    CRYPTO_TLS13_APPLICATION_KEYS.pair(),
    CRYPTO_TLS13_FINISHED.pair(),
    CRYPTO_TLS13_AEAD_SEAL.pair(),
    CRYPTO_TLS13_AEAD_OPEN.pair(),
    TIME_MONOTONIC_MS.pair(),
    SECRET_LEASE_OPENAI_AUTHORIZATION_SEND.pair(),
    ACQUIRE_CHUNK_ACCEPT.pair(),
    ACQUIRE_FINALIZE.pair(),
    ("ui", "viewport"),
    ("ui", "context_len"),
    ("ui", "context_read"),
    ("ui", "input_len"),
    ("ui", "input_read"),
    ("ui", "frame_submit"),
];

/// `KNOWN_HOST_IMPORTS` as pre-joined `module.name` strings, index-for-index.
/// The kernel honesty report renders its known-import list AND count from this
/// one table; `known_host_imports_dotted_matches_pairs` pins the derivation so
/// the two constants cannot drift apart.
pub const KNOWN_HOST_IMPORTS_DOTTED: &[&str] = &[
    "env.log",
    "env.counter_get",
    "env.input_len",
    "env.input_read",
    "env.output_write",
    "net.tcp_open",
    "net.tcp_send",
    "net.tcp_recv",
    "net.tcp_close",
    "crypto.tls13_session_open",
    "crypto.sha256",
    "crypto.p256_verify",
    "crypto.tls13_handshake_keys",
    "crypto.tls13_application_keys",
    "crypto.tls13_finished",
    "crypto.tls13_aead_seal",
    "crypto.tls13_aead_open",
    "time.monotonic_ms",
    "secret_lease.openai_authorization_send",
    "acquire.chunk_accept",
    "acquire.finalize",
    "ui.viewport",
    "ui.context_len",
    "ui.context_read",
    "ui.input_len",
    "ui.input_read",
    "ui.frame_submit",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedWasmImports<'a> {
    pub artifact_sha256: [u8; 32],
    pub import_list_sha256: [u8; 32],
    pub imports: &'a [(&'a str, &'a str)],
}

#[derive(Clone, Copy)]
pub struct EvidenceBoundWasmImportGrantInput<'a> {
    pub service_id: Option<&'a str>,
    pub artifact_sha256: Option<[u8; 32]>,
    pub host_import_abi: Option<&'a str>,
    pub declared_import_list_sha256: Option<[u8; 32]>,
    pub requested_imports: &'a [(&'a str, &'a str)],
    pub descriptor_source_signature_evidence: Option<VerifiedImportEvidence>,
    pub artifact_signature_attestation_evidence: Option<VerifiedImportEvidence>,
    pub computed_grant_evidence: Option<VerifiedImportEvidence>,
    pub observed_imports: Option<ObservedWasmImports<'a>>,
    pub linker_implementations: &'a [(&'a str, &'a str)],
    pub policy_allows_beyond_env: bool,
}

impl<'a> EvidenceBoundWasmImportGrantInput<'a> {
    pub const fn empty() -> Self {
        Self {
            service_id: None,
            artifact_sha256: None,
            host_import_abi: None,
            declared_import_list_sha256: None,
            requested_imports: &[],
            descriptor_source_signature_evidence: None,
            artifact_signature_attestation_evidence: None,
            computed_grant_evidence: None,
            observed_imports: None,
            linker_implementations: &[],
            policy_allows_beyond_env: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvidenceBoundWasmImportGrantDecision<'a> {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub authorized_import_count: usize,
    pub authorized_imports: &'a [(&'a str, &'a str)],
    pub authorized_service_id: Option<&'a str>,
    pub artifact_sha256: Option<[u8; 32]>,
    pub host_import_abi: Option<&'a str>,
    pub authorized_import_list_sha256: Option<[u8; 32]>,
    pub descriptor_source_signature_evidence_sha256: Option<[u8; 32]>,
    pub artifact_signature_attestation_evidence_sha256: Option<[u8; 32]>,
    pub computed_grant_evidence_sha256: Option<[u8; 32]>,
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
    evaluate_wasm_import_grant_inner(input, false)
}

/// Evaluates an import list read from the Wasm module itself. An observed empty
/// list is a real least-privilege surface; an absent caller declaration remains
/// denied through `evaluate_wasm_import_grant`.
pub fn evaluate_observed_wasm_import_grant(
    input: &WasmImportGrantInput<'_>,
    import_list_observed: bool,
) -> WasmImportGrantDecision {
    evaluate_wasm_import_grant_inner(input, import_list_observed)
}

fn evaluate_wasm_import_grant_inner(
    input: &WasmImportGrantInput<'_>,
    import_list_observed: bool,
) -> WasmImportGrantDecision {
    match input.service_id {
        Some(service_id) if !service_id.is_empty() => {}
        _ => return denied("missing_service_id"),
    }
    if !input.artifact_sha256_present {
        return denied("missing_artifact_binding");
    }
    if input.requested_imports.is_empty() {
        return if import_list_observed {
            WasmImportGrantDecision {
                performed: true,
                status: "import_grant_authorized",
                reason: "authorized_observed_empty_import_surface",
                authorized_import_count: 0,
            }
        } else {
            denied("missing_import_list")
        };
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

/// Evaluates a descriptor, its evidence, the observed module imports and the
/// exact linker surface as one ABI-bound grant.
///
/// This path is not wired into production in NET-1. Every production caller
/// continues to use `policy_allows_beyond_env: false` and no v1 live import is
/// implemented by the kernel linker.
pub fn evaluate_evidence_bound_wasm_import_grant<'a>(
    input: &EvidenceBoundWasmImportGrantInput<'a>,
) -> EvidenceBoundWasmImportGrantDecision<'a> {
    let service_id = match input.service_id {
        Some(service_id) if !service_id.is_empty() => service_id,
        _ => return evidence_bound_denied("missing_service_id"),
    };
    let Some(artifact_sha256) = input.artifact_sha256 else {
        return evidence_bound_denied("missing_artifact_sha256");
    };
    let abi_id = match input.host_import_abi {
        None | Some("") => return evidence_bound_denied("missing_host_import_abi"),
        Some(abi_id) if abi_id == HOST_IMPORT_ABI_V1 => abi_id,
        Some(_) => return evidence_bound_denied("unknown_host_import_abi"),
    };

    if input.requested_imports.is_empty() {
        return evidence_bound_denied("missing_import_list");
    }
    if input.requested_imports.len() > MAX_GRANTED_IMPORTS {
        return evidence_bound_denied("import_list_exceeds_max");
    }
    if has_duplicate(input.requested_imports) {
        return evidence_bound_denied("duplicate_host_import");
    }
    if input
        .requested_imports
        .iter()
        .any(|import| !KNOWN_HOST_IMPORTS.contains(import))
    {
        return evidence_bound_denied("unknown_host_import");
    }

    let import_list_sha256 = host_import_abi_ordered_list_sha256(abi_id, input.requested_imports);
    match input.declared_import_list_sha256 {
        None => return evidence_bound_denied("missing_declared_import_list_sha256"),
        Some(declared) if declared != import_list_sha256 => {
            return evidence_bound_denied("declared_import_list_sha256_mismatch")
        }
        Some(_) => {}
    }

    let Some(descriptor_evidence) = input.descriptor_source_signature_evidence else {
        return evidence_bound_denied("missing_descriptor_source_signature_evidence");
    };
    if descriptor_evidence.artifact_sha256 != artifact_sha256 {
        return evidence_bound_denied("descriptor_evidence_artifact_mismatch");
    }
    if descriptor_evidence.import_list_sha256 != import_list_sha256 {
        return evidence_bound_denied("descriptor_evidence_import_list_mismatch");
    }

    let Some(artifact_evidence) = input.artifact_signature_attestation_evidence else {
        return evidence_bound_denied("missing_artifact_signature_attestation_evidence");
    };
    if artifact_evidence.artifact_sha256 != artifact_sha256 {
        return evidence_bound_denied("artifact_evidence_artifact_mismatch");
    }
    if artifact_evidence.import_list_sha256 != import_list_sha256 {
        return evidence_bound_denied("artifact_evidence_import_list_mismatch");
    }

    let Some(computed_evidence) = input.computed_grant_evidence else {
        return evidence_bound_denied("missing_computed_grant_evidence");
    };
    if computed_evidence.artifact_sha256 != artifact_sha256 {
        return evidence_bound_denied("computed_grant_evidence_artifact_mismatch");
    }
    if computed_evidence.import_list_sha256 != import_list_sha256 {
        return evidence_bound_denied("computed_grant_evidence_import_list_mismatch");
    }

    let Some(observed) = input.observed_imports else {
        return evidence_bound_denied("missing_observed_import_list_evidence");
    };
    if observed.artifact_sha256 != artifact_sha256 {
        return evidence_bound_denied("observed_import_evidence_artifact_mismatch");
    }
    if observed.import_list_sha256 != import_list_sha256 {
        return evidence_bound_denied("observed_import_evidence_hash_mismatch");
    }
    if observed.imports != input.requested_imports {
        return evidence_bound_denied("observed_import_list_mismatch");
    }

    if input
        .requested_imports
        .iter()
        .any(|(module, _)| *module != "env")
        && !input.policy_allows_beyond_env
    {
        return evidence_bound_denied("import_beyond_env_not_owner_authorized");
    }

    if input.linker_implementations.is_empty() {
        return evidence_bound_denied("missing_linker_implementations");
    }
    if input.linker_implementations.len() > MAX_GRANTED_IMPORTS {
        return evidence_bound_denied("linker_implementation_list_exceeds_max");
    }
    if has_duplicate(input.linker_implementations) {
        return evidence_bound_denied("duplicate_linker_implementation");
    }
    if input.linker_implementations != input.requested_imports {
        return evidence_bound_denied("linker_implementation_list_mismatch");
    }

    EvidenceBoundWasmImportGrantDecision {
        performed: true,
        status: "import_grant_authorized",
        reason: "authorized_evidence_bound_host_import_surface",
        authorized_import_count: input.requested_imports.len(),
        authorized_imports: input.requested_imports,
        authorized_service_id: Some(service_id),
        artifact_sha256: Some(artifact_sha256),
        host_import_abi: Some(abi_id),
        authorized_import_list_sha256: Some(import_list_sha256),
        descriptor_source_signature_evidence_sha256: Some(descriptor_evidence.evidence_sha256),
        artifact_signature_attestation_evidence_sha256: Some(artifact_evidence.evidence_sha256),
        computed_grant_evidence_sha256: Some(computed_evidence.evidence_sha256),
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

fn evidence_bound_denied<'a>(reason: &'static str) -> EvidenceBoundWasmImportGrantDecision<'a> {
    EvidenceBoundWasmImportGrantDecision {
        performed: false,
        status: "denied",
        reason,
        authorized_import_count: 0,
        authorized_imports: &[],
        authorized_service_id: None,
        artifact_sha256: None,
        host_import_abi: None,
        authorized_import_list_sha256: None,
        descriptor_source_signature_evidence_sha256: None,
        artifact_signature_attestation_evidence_sha256: None,
        computed_grant_evidence_sha256: None,
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
    use crate::host_import_abi_v1::BEYOND_ENV_HOST_IMPORTS_V1;

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
            Mutation::UnknownHostImport => input.requested_imports = &[("net", "x")],
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
    fn known_host_imports_are_legacy_surfaces_plus_declared_v1_beyond_env_surface() {
        assert_eq!(
            &KNOWN_HOST_IMPORTS[..5],
            &[
                ("env", "log"),
                ("env", "counter_get"),
                ("env", "input_len"),
                ("env", "input_read"),
                ("env", "output_write"),
            ]
        );
        for (index, import) in BEYOND_ENV_HOST_IMPORTS_V1.iter().enumerate() {
            assert_eq!(KNOWN_HOST_IMPORTS[5 + index], import.pair());
        }
        assert_eq!(
            PERSONAL_SHELL_UI_IMPORTS,
            &KNOWN_HOST_IMPORTS[KNOWN_HOST_IMPORTS.len() - 6..]
        );
    }

    #[test]
    fn known_host_imports_dotted_matches_pairs() {
        assert_eq!(KNOWN_HOST_IMPORTS_DOTTED.len(), KNOWN_HOST_IMPORTS.len());
        for (index, (module, name)) in KNOWN_HOST_IMPORTS.iter().enumerate() {
            assert_eq!(
                KNOWN_HOST_IMPORTS_DOTTED[index],
                format!("{module}.{name}"),
                "dotted table drifted from the pair table at index {index}"
            );
        }
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
            (
                "svc.demo.dnsparse",
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
            assert_eq!(
                decision,
                WasmImportGrantDecision {
                    performed: true,
                    status: "import_grant_authorized",
                    reason: "authorized_exact_declared_import_surface",
                    authorized_import_count: requested_imports.len(),
                },
                "{service_id}"
            );
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
    fn legacy_false_policy_never_authorizes_the_personal_ui_surface() {
        let mut input = valid_input();
        input.service_id = Some(PERSONAL_SHELL_SERVICE_ID);
        input.requested_imports = PERSONAL_SHELL_UI_IMPORTS;
        input.policy_allows_beyond_env = false;

        assert_eq!(
            evaluate_wasm_import_grant(&input).reason,
            "import_beyond_env_not_owner_authorized"
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

    const EVIDENCE_BOUND_ARTIFACT_SHA256: [u8; 32] = [0x51; 32];
    const NET_ONLY_IMPORTS: &[(&str, &str)] = &[
        NET_TCP_OPEN.pair(),
        NET_TCP_SEND.pair(),
        NET_TCP_RECV.pair(),
        NET_TCP_CLOSE.pair(),
    ];
    const CRYPTO_ONLY_IMPORTS: &[(&str, &str)] = &[
        CRYPTO_TLS13_SESSION_OPEN.pair(),
        CRYPTO_SHA256.pair(),
        CRYPTO_P256_VERIFY.pair(),
        CRYPTO_TLS13_HANDSHAKE_KEYS.pair(),
        CRYPTO_TLS13_APPLICATION_KEYS.pair(),
        CRYPTO_TLS13_FINISHED.pair(),
        CRYPTO_TLS13_AEAD_SEAL.pair(),
        CRYPTO_TLS13_AEAD_OPEN.pair(),
    ];
    const TIME_ONLY_IMPORTS: &[(&str, &str)] = &[TIME_MONOTONIC_MS.pair()];
    const ACQUIRE_ONLY_IMPORTS: &[(&str, &str)] =
        &[ACQUIRE_CHUNK_ACCEPT.pair(), ACQUIRE_FINALIZE.pair()];
    const SECRET_ONLY_IMPORTS: &[(&str, &str)] = &[SECRET_LEASE_OPENAI_AUTHORIZATION_SEND.pair()];
    const ENV_ONLY_IMPORTS: &[(&str, &str)] = &[("env", "input_len")];
    const MIXED_IMPORTS: &[(&str, &str)] = &[("env", "input_len"), NET_TCP_OPEN.pair()];
    const DUPLICATE_NET_IMPORTS: &[(&str, &str)] = &[NET_TCP_OPEN.pair(), NET_TCP_OPEN.pair()];
    const REORDERED_MIXED_IMPORTS: &[(&str, &str)] = &[NET_TCP_OPEN.pair(), ("env", "input_len")];

    fn evidence_bound_input(
        requested_imports: &'static [(&'static str, &'static str)],
    ) -> EvidenceBoundWasmImportGrantInput<'static> {
        let import_list_sha256 =
            host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, requested_imports);
        let input = EvidenceBoundWasmImportGrantInput {
            service_id: Some("svc.net.acquire.w7"),
            artifact_sha256: Some(EVIDENCE_BOUND_ARTIFACT_SHA256),
            host_import_abi: Some(HOST_IMPORT_ABI_V1),
            declared_import_list_sha256: Some(import_list_sha256),
            requested_imports,
            descriptor_source_signature_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x52; 32],
                artifact_sha256: EVIDENCE_BOUND_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            artifact_signature_attestation_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x53; 32],
                artifact_sha256: EVIDENCE_BOUND_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            computed_grant_evidence: Some(VerifiedImportEvidence {
                evidence_sha256: [0x54; 32],
                artifact_sha256: EVIDENCE_BOUND_ARTIFACT_SHA256,
                import_list_sha256,
            }),
            observed_imports: Some(ObservedWasmImports {
                artifact_sha256: EVIDENCE_BOUND_ARTIFACT_SHA256,
                import_list_sha256,
                imports: requested_imports,
            }),
            linker_implementations: &[],
            policy_allows_beyond_env: false,
        };
        assert!(!input.policy_allows_beyond_env);
        input
    }

    #[test]
    fn each_declared_beyond_env_family_is_policy_denied_alone() {
        let cases = [
            ("net", NET_ONLY_IMPORTS),
            ("crypto", CRYPTO_ONLY_IMPORTS),
            ("time", TIME_ONLY_IMPORTS),
            ("acquire", ACQUIRE_ONLY_IMPORTS),
        ];

        for (family, imports) in cases {
            let input = evidence_bound_input(imports);
            let decision = evaluate_evidence_bound_wasm_import_grant(&input);
            assert!(!decision.performed, "{family}");
            assert_eq!(
                decision.reason, "import_beyond_env_not_owner_authorized",
                "{family}"
            );
            assert!(decision.authorized_imports.is_empty(), "{family}");
        }
    }

    #[test]
    fn declared_future_secret_lease_is_ungrantable_to_w7() {
        let input = evidence_bound_input(SECRET_ONLY_IMPORTS);
        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&input).reason,
            "import_beyond_env_not_owner_authorized"
        );
    }

    #[test]
    fn mixed_env_and_beyond_env_surface_is_policy_denied() {
        let input = evidence_bound_input(MIXED_IMPORTS);
        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&input).reason,
            "import_beyond_env_not_owner_authorized"
        );
    }

    #[test]
    fn evidence_bound_duplicate_import_is_denied_before_policy() {
        let input = evidence_bound_input(DUPLICATE_NET_IMPORTS);
        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&input).reason,
            "duplicate_host_import"
        );
    }

    #[test]
    fn evidence_bound_unknown_abi_and_import_are_distinct_denials() {
        let mut unknown_abi = evidence_bound_input(NET_ONLY_IMPORTS);
        unknown_abi.host_import_abi = Some("raios.host_imports.v2");
        let unknown_import = evidence_bound_input(&[("net", "resolve")]);

        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&unknown_abi).reason,
            "unknown_host_import_abi"
        );
        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&unknown_import).reason,
            "unknown_host_import"
        );
    }

    #[test]
    fn declared_hash_observed_hash_and_observed_list_drift_are_distinct_denials() {
        let mut declared_hash = evidence_bound_input(MIXED_IMPORTS);
        declared_hash.declared_import_list_sha256 = Some([0xa1; 32]);

        let mut observed_hash = evidence_bound_input(MIXED_IMPORTS);
        observed_hash
            .observed_imports
            .as_mut()
            .unwrap()
            .import_list_sha256 = [0xa2; 32];

        let mut observed_list = evidence_bound_input(MIXED_IMPORTS);
        observed_list.observed_imports.as_mut().unwrap().imports = REORDERED_MIXED_IMPORTS;

        let reasons = [
            evaluate_evidence_bound_wasm_import_grant(&declared_hash).reason,
            evaluate_evidence_bound_wasm_import_grant(&observed_hash).reason,
            evaluate_evidence_bound_wasm_import_grant(&observed_list).reason,
        ];
        assert_eq!(
            reasons,
            [
                "declared_import_list_sha256_mismatch",
                "observed_import_evidence_hash_mismatch",
                "observed_import_list_mismatch",
            ]
        );
        assert_ne!(reasons[0], reasons[1]);
        assert_ne!(reasons[0], reasons[2]);
        assert_ne!(reasons[1], reasons[2]);
    }

    #[test]
    fn missing_evidence_reasons_are_reachable_and_pairwise_distinct() {
        let mut service = evidence_bound_input(NET_ONLY_IMPORTS);
        service.service_id = None;
        let mut artifact_subject = evidence_bound_input(NET_ONLY_IMPORTS);
        artifact_subject.artifact_sha256 = None;
        let mut abi = evidence_bound_input(NET_ONLY_IMPORTS);
        abi.host_import_abi = None;
        let mut declared_hash = evidence_bound_input(NET_ONLY_IMPORTS);
        declared_hash.declared_import_list_sha256 = None;
        let mut descriptor = evidence_bound_input(NET_ONLY_IMPORTS);
        descriptor.descriptor_source_signature_evidence = None;
        let mut artifact = evidence_bound_input(NET_ONLY_IMPORTS);
        artifact.artifact_signature_attestation_evidence = None;
        let mut computed = evidence_bound_input(NET_ONLY_IMPORTS);
        computed.computed_grant_evidence = None;
        let mut observed = evidence_bound_input(NET_ONLY_IMPORTS);
        observed.observed_imports = None;

        let reasons = [
            evaluate_evidence_bound_wasm_import_grant(&service).reason,
            evaluate_evidence_bound_wasm_import_grant(&artifact_subject).reason,
            evaluate_evidence_bound_wasm_import_grant(&abi).reason,
            evaluate_evidence_bound_wasm_import_grant(&declared_hash).reason,
            evaluate_evidence_bound_wasm_import_grant(&descriptor).reason,
            evaluate_evidence_bound_wasm_import_grant(&artifact).reason,
            evaluate_evidence_bound_wasm_import_grant(&computed).reason,
            evaluate_evidence_bound_wasm_import_grant(&observed).reason,
        ];
        assert_eq!(
            reasons,
            [
                "missing_service_id",
                "missing_artifact_sha256",
                "missing_host_import_abi",
                "missing_declared_import_list_sha256",
                "missing_descriptor_source_signature_evidence",
                "missing_artifact_signature_attestation_evidence",
                "missing_computed_grant_evidence",
                "missing_observed_import_list_evidence",
            ]
        );
        for (index, reason) in reasons.iter().enumerate() {
            assert!(!reasons[..index].contains(reason));
        }
    }

    #[test]
    fn evidence_subject_drift_reasons_are_reachable_and_pairwise_distinct() {
        let mut descriptor_artifact = evidence_bound_input(NET_ONLY_IMPORTS);
        descriptor_artifact
            .descriptor_source_signature_evidence
            .as_mut()
            .unwrap()
            .artifact_sha256 = [0xb1; 32];
        let mut descriptor_list = evidence_bound_input(NET_ONLY_IMPORTS);
        descriptor_list
            .descriptor_source_signature_evidence
            .as_mut()
            .unwrap()
            .import_list_sha256 = [0xb2; 32];
        let mut artifact_artifact = evidence_bound_input(NET_ONLY_IMPORTS);
        artifact_artifact
            .artifact_signature_attestation_evidence
            .as_mut()
            .unwrap()
            .artifact_sha256 = [0xb3; 32];
        let mut artifact_list = evidence_bound_input(NET_ONLY_IMPORTS);
        artifact_list
            .artifact_signature_attestation_evidence
            .as_mut()
            .unwrap()
            .import_list_sha256 = [0xb4; 32];
        let mut computed_artifact = evidence_bound_input(NET_ONLY_IMPORTS);
        computed_artifact
            .computed_grant_evidence
            .as_mut()
            .unwrap()
            .artifact_sha256 = [0xb5; 32];
        let mut computed_list = evidence_bound_input(NET_ONLY_IMPORTS);
        computed_list
            .computed_grant_evidence
            .as_mut()
            .unwrap()
            .import_list_sha256 = [0xb6; 32];
        let mut observed_artifact = evidence_bound_input(NET_ONLY_IMPORTS);
        observed_artifact
            .observed_imports
            .as_mut()
            .unwrap()
            .artifact_sha256 = [0xb7; 32];

        let reasons = [
            evaluate_evidence_bound_wasm_import_grant(&descriptor_artifact).reason,
            evaluate_evidence_bound_wasm_import_grant(&descriptor_list).reason,
            evaluate_evidence_bound_wasm_import_grant(&artifact_artifact).reason,
            evaluate_evidence_bound_wasm_import_grant(&artifact_list).reason,
            evaluate_evidence_bound_wasm_import_grant(&computed_artifact).reason,
            evaluate_evidence_bound_wasm_import_grant(&computed_list).reason,
            evaluate_evidence_bound_wasm_import_grant(&observed_artifact).reason,
        ];
        assert_eq!(
            reasons,
            [
                "descriptor_evidence_artifact_mismatch",
                "descriptor_evidence_import_list_mismatch",
                "artifact_evidence_artifact_mismatch",
                "artifact_evidence_import_list_mismatch",
                "computed_grant_evidence_artifact_mismatch",
                "computed_grant_evidence_import_list_mismatch",
                "observed_import_evidence_artifact_mismatch",
            ]
        );
        for (index, reason) in reasons.iter().enumerate() {
            assert!(!reasons[..index].contains(reason));
        }
    }

    #[test]
    fn evidence_bound_missing_linker_surface_is_denied_without_authority_flip() {
        let input = evidence_bound_input(ENV_ONLY_IMPORTS);
        assert!(!input.policy_allows_beyond_env);
        assert_eq!(
            evaluate_evidence_bound_wasm_import_grant(&input).reason,
            "missing_linker_implementations"
        );
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
