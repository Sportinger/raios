use super::*;

include!(concat!(env!("OUT_DIR"), "/net_acquire_w7_wasm_artifact.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/net_acquire_w7_current_boot_load_descriptor.rs"
));

#[path = "../../../raios-w7-acquire-logic/src/lib.rs"]
pub(crate) mod logic;

pub(crate) const NET_ACQUIRE_W7_SERVICE_ID: &str = "svc.net.acquire.w7";
pub(crate) const NET_ACQUIRE_W7_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("net", "tcp_open"),
    ("net", "tcp_send"),
    ("net", "tcp_recv"),
    ("net", "tcp_close"),
    ("crypto", "tls13_session_open"),
    ("crypto", "sha256"),
    ("crypto", "p256_verify"),
    ("crypto", "tls13_handshake_keys"),
    ("crypto", "tls13_application_keys"),
    ("crypto", "tls13_finished"),
    ("crypto", "tls13_aead_seal"),
    ("crypto", "tls13_aead_open"),
    ("acquire", "chunk_accept"),
    ("acquire", "finalize"),
];

pub(crate) struct AcquisitionServiceProbe {
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) descriptor_sha256: [u8; 32],
    pub(crate) load_descriptor_sha256: [u8; 32],
    pub(crate) import_list_sha256: [u8; 32],
    pub(crate) artifact_valid: bool,
    pub(crate) signatures_build_verified: bool,
    pub(crate) observed_imports_exact: bool,
    pub(crate) observed_import_count: u64,
    pub(crate) denial_reason: &'static str,
    pub(crate) denied_before_instantiation: bool,
    pub(crate) vector_report: logic::VectorReport,
    pub(crate) guest_trap_cleanup: bool,
    pub(crate) out_of_fuel_cleanup: bool,
}

pub(crate) fn acquisition_service_probe() -> AcquisitionServiceProbe {
    let engine = Engine::default();
    let module = Module::new(&engine, NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES);
    // The compiler emits the Wasm import section in first-use order, not in the
    // descriptor's ABI order, so this must be SET equality, not positional:
    // section order is codegen with no security meaning, and the real enforcement
    // (subset check + per-instance linker) is already order-independent. We still
    // reject any extra, missing, or duplicated import — exact membership holds.
    let (observed_imports_exact, observed_import_count) = match module.as_ref() {
        Ok(module) => {
            let mut count = 0usize;
            let mut all_authorized = true;
            let mut no_duplicates = true;
            let mut seen: [bool; 16] = [false; 16];
            for import in module.imports() {
                count += 1;
                match NET_ACQUIRE_W7_AUTHORIZED_IMPORTS
                    .iter()
                    .position(|pair| pair == &(import.module(), import.name()))
                {
                    Some(pos) => {
                        if seen[pos] {
                            no_duplicates = false;
                        }
                        seen[pos] = true;
                    }
                    None => all_authorized = false,
                }
            }
            let exact = all_authorized
                && no_duplicates
                && count == NET_ACQUIRE_W7_AUTHORIZED_IMPORTS.len()
                && seen.iter().all(|hit| *hit);
            (exact, count as u64)
        }
        Err(_) => (false, 0),
    };
    let artifact_valid = module.is_ok()
        && sha256_bytes(NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES)
            == NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(NET_ACQUIRE_W7_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == NET_ACQUIRE_W7_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(NET_ACQUIRE_W7_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == NET_ACQUIRE_W7_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && sha256_bytes(NET_ACQUIRE_W7_LOAD_DESCRIPTOR_SOURCE.as_bytes())
            == NET_ACQUIRE_W7_LOAD_DESCRIPTOR_HASH
        && sha256_bytes(NET_ACQUIRE_W7_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == NET_ACQUIRE_W7_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH;
    let import_list_sha256 =
        host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, NET_ACQUIRE_W7_AUTHORIZED_IMPORTS);
    let evidence = |evidence_sha256| VerifiedImportEvidence {
        evidence_sha256,
        artifact_sha256: NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
        import_list_sha256,
    };
    let decision = evaluate_evidence_bound_wasm_import_grant(&EvidenceBoundWasmImportGrantInput {
        service_id: Some(NET_ACQUIRE_W7_SERVICE_ID),
        artifact_sha256: Some(NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH),
        host_import_abi: Some(HOST_IMPORT_ABI_V1),
        declared_import_list_sha256: Some(import_list_sha256),
        requested_imports: NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
        descriptor_source_signature_evidence: Some(evidence(
            NET_ACQUIRE_W7_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH,
        )),
        artifact_signature_attestation_evidence: Some(evidence(
            NET_ACQUIRE_W7_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        )),
        computed_grant_evidence: Some(evidence(NET_ACQUIRE_W7_LOAD_DESCRIPTOR_HASH)),
        observed_imports: Some(ObservedWasmImports {
            artifact_sha256: NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
            import_list_sha256,
            imports: NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
        }),
        linker_implementations: NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
        policy_allows_beyond_env: false,
    });
    let lifecycle = super::probes::run_beyond_env_lifecycle_suite();
    let terminal_cleanup = |name, outcome| {
        lifecycle.cases.iter().any(|case| {
            case.name == name
                && case.outcome == outcome
                && case.teardown_complete
                && case.teardown_count == 1
        })
    };
    AcquisitionServiceProbe {
        artifact_sha256: NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
        descriptor_sha256: NET_ACQUIRE_W7_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        load_descriptor_sha256: NET_ACQUIRE_W7_LOAD_DESCRIPTOR_HASH,
        import_list_sha256,
        artifact_valid,
        signatures_build_verified: artifact_valid
            && NET_ACQUIRE_W7_LOAD_DESCRIPTOR_HOST_IMPORT_ABI == HOST_IMPORT_ABI_V1
            && NET_ACQUIRE_W7_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORT_COUNT == 16
            && NET_ACQUIRE_W7_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORTS
                == "env.input_len,env.input_read,net.tcp_open,net.tcp_send,net.tcp_recv,net.tcp_close,crypto.tls13_session_open,crypto.sha256,crypto.p256_verify,crypto.tls13_handshake_keys,crypto.tls13_application_keys,crypto.tls13_finished,crypto.tls13_aead_seal,crypto.tls13_aead_open,acquire.chunk_accept,acquire.finalize",
        observed_imports_exact,
        observed_import_count,
        denial_reason: decision.reason,
        denied_before_instantiation: artifact_valid
            && observed_imports_exact
            && !decision.performed,
        vector_report: logic::run_fixture_vectors(),
        guest_trap_cleanup: terminal_cleanup("guest_trap", "guest_trap"),
        out_of_fuel_cleanup: terminal_cleanup("wasm_out_of_fuel", "out_of_fuel"),
    }
}
