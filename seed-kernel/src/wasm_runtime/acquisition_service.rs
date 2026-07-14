use super::*;

include!(concat!(env!("OUT_DIR"), "/net_acquire_w7_wasm_artifact.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/net_acquire_w7_current_boot_load_descriptor.rs"
));

#[path = "../../../raios-w7-acquire-logic/src/lib.rs"]
pub(crate) mod logic;

pub(crate) const NET_ACQUIRE_W7_SERVICE_ID: &str = "svc.net.acquire.w7";
pub(crate) const NET_8_W7_SOURCE_POLICY_ID: &str = "local.qemu.w7";
pub(crate) const NET_8_W7_SNI: &str = "w7.test.raios";
const NET_8_W7_PIN_HEX: Option<&str> = option_env!("RAIOS_NET_8_W7_SPKI_SHA256");
pub(crate) const NET_8_APPROVED_W7_ARTIFACT_SHA256: [u8; 32] = [
    0x32, 0xa0, 0x18, 0xb0, 0xc7, 0x30, 0xa4, 0xf8, 0x52, 0x10, 0xca, 0x82, 0x04, 0x83, 0xca, 0x68,
    0xf8, 0xa4, 0xd0, 0x71, 0x50, 0x21, 0xa1, 0xdd, 0xa9, 0x79, 0x51, 0xfe, 0x30, 0x5e, 0x9e, 0x54,
];
pub(crate) const NET_8_APPROVED_W7_IMPORT_LIST_SHA256: [u8; 32] = [
    0xeb, 0x39, 0x0e, 0xc5, 0xc2, 0xdf, 0xde, 0x5a, 0xc6, 0x32, 0xb1, 0x27, 0x51, 0x5c, 0x51, 0x01,
    0xc8, 0x12, 0xed, 0x6c, 0xa2, 0x09, 0x19, 0x18, 0x46, 0xbc, 0x76, 0x24, 0x09, 0xbf, 0x43, 0x45,
];
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
    pub(crate) policy_allows_beyond_env: bool,
    pub(crate) production_linker_armed: bool,
    pub(crate) live_pin_configured: bool,
    pub(crate) artifact_mismatch_denied: bool,
    pub(crate) import_list_mismatch_denied: bool,
    pub(crate) source_policy_mismatch_denied: bool,
    pub(crate) different_service_denied_before_instantiation: bool,
    pub(crate) vector_report: logic::VectorReport,
    pub(crate) guest_trap_cleanup: bool,
    pub(crate) out_of_fuel_cleanup: bool,
}

#[derive(Clone, Copy)]
pub(super) struct PreparedW7Invocation {
    pub(super) source: W7SourcePolicy,
    pub(super) request: StagedW7Request,
    pub(super) policy_allows_beyond_env: bool,
}

#[derive(Clone, Copy)]
pub(super) struct W7SourcePolicy {
    pub(super) id: &'static str,
    pub(super) address: [u8; 4],
    pub(super) port: u16,
    pub(super) sni: &'static str,
    pub(super) spki_pin_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub(super) struct StagedW7Request {
    pub(super) bytes: [u8; 271],
    pub(super) total_len: usize,
    pub(super) whole_sha256: [u8; 32],
    pub(super) chunk_count: usize,
    pub(super) chunk_sha256: [[u8; 32]; 4],
}

#[derive(Clone, Copy)]
pub(crate) struct W7AcquisitionSnapshot {
    pub(crate) request_status: &'static str,
    pub(crate) active: bool,
    pub(crate) outcome: &'static str,
    pub(crate) invocation_id: u64,
    pub(crate) run_count: u64,
    pub(crate) success_count: u64,
    pub(crate) killed_count: u64,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) teardown_complete: bool,
    pub(crate) no_resume_after_kill: bool,
    pub(crate) crypto_session_zeroized: bool,
    pub(crate) transport_lease_held: bool,
    pub(crate) pending_acquisition_present: bool,
    pub(crate) source_tls_evidence: bool,
    pub(crate) tx_bytes: u64,
    pub(crate) rx_bytes: u64,
    pub(crate) candidate_sha256: Option<[u8; 32]>,
    pub(crate) receipt_sha256: Option<[u8; 32]>,
    pub(crate) w7_blocks_provider_reason: &'static str,
    pub(crate) prior_candidate_preserved: bool,
}

impl W7AcquisitionSnapshot {
    const fn empty() -> Self {
        Self {
            request_status: "idle",
            active: false,
            outcome: "not_run",
            invocation_id: 0,
            run_count: 0,
            success_count: 0,
            killed_count: 0,
            suspension_count: 0,
            resume_count: 0,
            teardown_count: 0,
            teardown_complete: false,
            no_resume_after_kill: false,
            crypto_session_zeroized: true,
            transport_lease_held: false,
            pending_acquisition_present: false,
            source_tls_evidence: false,
            tx_bytes: 0,
            rx_bytes: 0,
            candidate_sha256: None,
            receipt_sha256: None,
            w7_blocks_provider_reason: "not_observed",
            prior_candidate_preserved: true,
        }
    }
}

static W7_ACQUISITION: Mutex<W7AcquisitionSnapshot> = Mutex::new(W7AcquisitionSnapshot::empty());

pub(crate) fn w7_acquisition_snapshot() -> W7AcquisitionSnapshot {
    *W7_ACQUISITION.lock()
}

pub(super) fn note_w7_request(status: &'static str) {
    W7_ACQUISITION.lock().request_status = status;
}

pub(super) fn note_w7_started(invocation_id: u64) {
    let mut snapshot = W7_ACQUISITION.lock();
    snapshot.request_status = "running";
    snapshot.active = true;
    snapshot.outcome = "pending";
    snapshot.invocation_id = invocation_id;
    snapshot.run_count = snapshot.run_count.saturating_add(1);
    snapshot.teardown_complete = false;
    snapshot.crypto_session_zeroized = false;
    snapshot.transport_lease_held = false;
    snapshot.pending_acquisition_present = true;
    snapshot.source_tls_evidence = false;
    snapshot.candidate_sha256 = None;
    snapshot.receipt_sha256 = None;
    snapshot.w7_blocks_provider_reason = "not_observed";
}

pub(super) fn note_w7_lease_held() {
    W7_ACQUISITION.lock().transport_lease_held = true;
}

pub(crate) fn observe_w7_blocks_provider() -> &'static str {
    let active_with_lease = {
        let snapshot = W7_ACQUISITION.lock();
        snapshot.active && snapshot.transport_lease_held
    };
    let reason = if active_with_lease {
        super::net_shims::probe_w7_blocks_provider()
    } else {
        "w7_lease_not_active"
    };
    W7_ACQUISITION.lock().w7_blocks_provider_reason = reason;
    reason
}

pub(super) struct W7FinishEvidence {
    pub(super) outcome: &'static str,
    pub(super) suspension_count: u32,
    pub(super) resume_count: u32,
    pub(super) teardown_count: u32,
    pub(super) source_tls_evidence: bool,
    pub(super) crypto_session_zeroized: bool,
    pub(super) tx_bytes: u64,
    pub(super) rx_bytes: u64,
    pub(super) candidate_sha256: Option<[u8; 32]>,
    pub(super) receipt_sha256: Option<[u8; 32]>,
    pub(super) prior_candidate_preserved: bool,
}

pub(super) fn note_w7_finished(evidence: W7FinishEvidence) {
    let mut snapshot = W7_ACQUISITION.lock();
    snapshot.request_status = "complete";
    snapshot.active = false;
    snapshot.outcome = evidence.outcome;
    snapshot.success_count = snapshot.success_count.saturating_add(u64::from(
        evidence.outcome == "finished" && evidence.candidate_sha256.is_some(),
    ));
    snapshot.killed_count = snapshot
        .killed_count
        .saturating_add(u64::from(evidence.outcome == "killed"));
    snapshot.suspension_count = evidence.suspension_count;
    snapshot.resume_count = evidence.resume_count;
    snapshot.teardown_count = evidence.teardown_count;
    snapshot.teardown_complete = true;
    snapshot.no_resume_after_kill = evidence.outcome == "killed";
    snapshot.crypto_session_zeroized = evidence.crypto_session_zeroized;
    snapshot.transport_lease_held = false;
    snapshot.pending_acquisition_present = false;
    snapshot.source_tls_evidence = evidence.source_tls_evidence;
    snapshot.tx_bytes = evidence.tx_bytes;
    snapshot.rx_bytes = evidence.rx_bytes;
    snapshot.candidate_sha256 = evidence.candidate_sha256;
    snapshot.receipt_sha256 = evidence.receipt_sha256;
    snapshot.prior_candidate_preserved = evidence.prior_candidate_preserved;
}

fn net_8_w7_policy_allows(
    artifact_sha256: [u8; 32],
    import_list_sha256: [u8; 32],
    source_policy_id: &str,
) -> bool {
    let policy_allows_beyond_env = artifact_sha256 == NET_8_APPROVED_W7_ARTIFACT_SHA256
        && import_list_sha256 == NET_8_APPROVED_W7_IMPORT_LIST_SHA256
        && source_policy_id == "local.qemu.w7";
    policy_allows_beyond_env
}

pub(super) fn prepare_w7_invocation() -> Option<PreparedW7Invocation> {
    let artifact_sha256 = NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH;
    let import_list_sha256 =
        host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, NET_ACQUIRE_W7_AUTHORIZED_IMPORTS);
    let source = W7SourcePolicy {
        id: NET_8_W7_SOURCE_POLICY_ID,
        // 10.0.2.100, not the slirp gateway 10.0.2.2: QEMU rejects a guestfwd rule
        // targeting the gateway address, so the live W7 fixture is bridged at .100.
        address: [10, 0, 2, 100],
        port: 8443,
        sni: NET_8_W7_SNI,
        spki_pin_sha256: parse_sha256_hex(NET_8_W7_PIN_HEX?)?,
    };
    let source_policy_id = source.id;
    let policy_allows_beyond_env =
        net_8_w7_policy_allows(artifact_sha256, import_list_sha256, source_policy_id);
    let evidence = |evidence_sha256| VerifiedImportEvidence {
        evidence_sha256,
        artifact_sha256,
        import_list_sha256,
    };
    let decision = evaluate_evidence_bound_wasm_import_grant(&EvidenceBoundWasmImportGrantInput {
        service_id: Some(NET_ACQUIRE_W7_SERVICE_ID),
        artifact_sha256: Some(artifact_sha256),
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
            artifact_sha256,
            import_list_sha256,
            imports: NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
        }),
        linker_implementations: NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
        policy_allows_beyond_env,
    });
    if !decision.performed
        || sha256_bytes(NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES) != artifact_sha256
        || ECHO_WASM_ARTIFACT_BYTES.len() != 4205
        || sha256_bytes(ECHO_WASM_ARTIFACT_BYTES) != ECHO_WASM_ARTIFACT_BYTES_HASH
    {
        return None;
    }
    Some(PreparedW7Invocation {
        source,
        request: encode_w7_request(),
        policy_allows_beyond_env,
    })
}

fn encode_w7_request() -> StagedW7Request {
    let mut request = [0u8; 271];
    request[..8].copy_from_slice(b"RW7REQ01");
    request[8] = NET_8_W7_SNI.len() as u8;
    request[9..11].copy_from_slice(&(82u16).to_be_bytes());
    request[11..15].copy_from_slice(&(ECHO_WASM_ARTIFACT_BYTES.len() as u32).to_be_bytes());
    request[15] = 1;
    request[16..48].copy_from_slice(&ECHO_WASM_ARTIFACT_BYTES_HASH);
    request[48..80].copy_from_slice(&ECHO_WASM_ARTIFACT_BYTES_HASH);
    // Layout after the 176-byte header: sni (13), then path = "/raios/cas/sha256/"
    // (18) + 64 lowercase hex = 82 bytes, filling exactly [189..271]. The old offsets
    // wrote the 18-byte prefix into a 22-byte slot (length-mismatch panic) and ran the
    // hex loop to index 274 in a 271-byte array (out of bounds) — the guest never saw a
    // request and returned before tcp_open.
    request[176..189].copy_from_slice(NET_8_W7_SNI.as_bytes());
    request[189..207].copy_from_slice(b"/raios/cas/sha256/");
    for (index, byte) in ECHO_WASM_ARTIFACT_BYTES_HASH.iter().enumerate() {
        request[207 + index * 2] = HEX[(byte >> 4) as usize];
        request[208 + index * 2] = HEX[(byte & 0x0f) as usize];
    }
    StagedW7Request {
        bytes: request,
        total_len: ECHO_WASM_ARTIFACT_BYTES.len(),
        whole_sha256: ECHO_WASM_ARTIFACT_BYTES_HASH,
        chunk_count: 1,
        chunk_sha256: [ECHO_WASM_ARTIFACT_BYTES_HASH, [0; 32], [0; 32], [0; 32]],
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

fn parse_sha256_hex(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut output = [0u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = hex_nibble(pair[0])?.checked_mul(16)? + hex_nibble(pair[1])?;
    }
    Some(output)
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
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
    let policy_allows_beyond_env = net_8_w7_policy_allows(
        NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
        import_list_sha256,
        NET_8_W7_SOURCE_POLICY_ID,
    );
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
        policy_allows_beyond_env,
    });
    let different_service =
        evaluate_evidence_bound_wasm_import_grant(&EvidenceBoundWasmImportGrantInput {
            service_id: Some("svc.net.acquire.other"),
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
        policy_allows_beyond_env,
        production_linker_armed: artifact_valid && observed_imports_exact && decision.performed,
        live_pin_configured: NET_8_W7_PIN_HEX.and_then(parse_sha256_hex).is_some(),
        artifact_mismatch_denied: !net_8_w7_policy_allows(
            [0; 32],
            import_list_sha256,
            NET_8_W7_SOURCE_POLICY_ID,
        ),
        import_list_mismatch_denied: !net_8_w7_policy_allows(
            NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
            [0; 32],
            NET_8_W7_SOURCE_POLICY_ID,
        ),
        source_policy_mismatch_denied: !net_8_w7_policy_allows(
            NET_ACQUIRE_W7_WASM_ARTIFACT_BYTES_HASH,
            import_list_sha256,
            "local.qemu.other",
        ),
        different_service_denied_before_instantiation: !different_service.performed,
        vector_report: logic::run_fixture_vectors(),
        guest_trap_cleanup: terminal_cleanup("guest_trap", "guest_trap"),
        out_of_fuel_cleanup: terminal_cleanup("wasm_out_of_fuel", "out_of_fuel"),
    }
}
