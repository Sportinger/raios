use super::invocation::{runtime_now_ms, BeyondEnvState};
use super::*;
use raios_core::{
    beyond_env_invocation::BoundaryState,
    host_import_abi_v1::{
        ACQUIRE_HOST_IMPORTS_V1, HOST_IMPORT_ERROR_CAPABILITY_DENIED,
        HOST_IMPORT_ERROR_INVALID_ARGUMENT, HOST_IMPORT_ERROR_INVALID_STATE,
        HOST_IMPORT_ERROR_KILLED, HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
    },
};

const ACQUIRE_SERVICE_ID: &str = "test.fixture.acquire_shims.signed";
const ACQUIRE_MAX_CHUNK_BYTES: usize = 65_536;
pub(super) const ACQUIRE_FIXTURE_CASE_COUNT: usize = 21;

// Labeled NET-6 test module only. Its Linker is the only Linker that receives
// acquire.* closures while beyond-env policy remains false everywhere.
pub(super) const ACQUIRE_SHIM_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0c, 0x02, 0x60, 0x03, 0x7f, 0x7f, 0x7f,
    0x01, 0x7f, 0x60, 0x00, 0x01, 0x7f, 0x02, 0x2b, 0x02, 0x07, 0x61, 0x63, 0x71, 0x75, 0x69, 0x72,
    0x65, 0x0c, 0x63, 0x68, 0x75, 0x6e, 0x6b, 0x5f, 0x61, 0x63, 0x63, 0x65, 0x70, 0x74, 0x00, 0x00,
    0x07, 0x61, 0x63, 0x71, 0x75, 0x69, 0x72, 0x65, 0x08, 0x66, 0x69, 0x6e, 0x61, 0x6c, 0x69, 0x7a,
    0x65, 0x00, 0x01, 0x03, 0x05, 0x04, 0x00, 0x01, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x2b, 0x05, 0x05, 0x63, 0x68, 0x75, 0x6e, 0x6b, 0x00, 0x02, 0x08, 0x66, 0x69, 0x6e, 0x61, 0x6c,
    0x69, 0x7a, 0x65, 0x00, 0x03, 0x04, 0x74, 0x72, 0x61, 0x70, 0x00, 0x04, 0x04, 0x6c, 0x6f, 0x6f,
    0x70, 0x00, 0x05, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x1f, 0x04, 0x0a,
    0x00, 0x20, 0x00, 0x20, 0x01, 0x20, 0x02, 0x10, 0x00, 0x0b, 0x04, 0x00, 0x10, 0x01, 0x0b, 0x03,
    0x00, 0x00, 0x0b, 0x09, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AcquireFixtureMode {
    Normal,
    ShortBody,
    LongBody,
    SourceEvidenceMismatch,
    CatalogMismatch,
    ReceiverMismatch,
    PostureDenied,
    ForeignOwner,
    StaleSession,
}

#[derive(Clone, Copy)]
struct ChunkExpectation {
    len: usize,
    sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AcquireDenial {
    KillGenerationChanged,
    InvocationAuthorityInvalid,
    SessionOwnerMismatch,
    SessionNotCurrent,
    PostureDenied,
    SourceTlsEvidenceMismatch,
    CatalogMismatch,
    ReceiverIdentityMismatch,
    GuestMemoryFault,
    ChunkIndexMismatch,
    ChunkLengthMismatch,
    ChunkHashMismatch,
    ChunkOutOfOrder,
    DuplicateChunk,
    ExtraChunk,
    FinalizePriorChunkDenied,
    FinalizeMissingChunks,
    FinalizeShortBody,
    FinalizeLongBody,
    SharedVerifierDenied,
}

impl AcquireDenial {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::KillGenerationChanged => "acquire_kill_generation_changed",
            Self::InvocationAuthorityInvalid => "acquire_invocation_authority_invalid",
            Self::SessionOwnerMismatch => "acquire_session_owner_mismatch",
            Self::SessionNotCurrent => "acquire_session_not_current",
            Self::PostureDenied => "acquire_posture_denied",
            Self::SourceTlsEvidenceMismatch => "acquire_source_tls_evidence_mismatch",
            Self::CatalogMismatch => "acquire_catalog_mismatch",
            Self::ReceiverIdentityMismatch => "acquire_receiver_identity_mismatch",
            Self::GuestMemoryFault => "acquire_guest_memory_fault",
            Self::ChunkIndexMismatch => "acquire_chunk_index_mismatch",
            Self::ChunkLengthMismatch => "acquire_chunk_length_mismatch",
            Self::ChunkHashMismatch => "acquire_chunk_hash_mismatch",
            Self::ChunkOutOfOrder => "acquire_chunk_out_of_order",
            Self::DuplicateChunk => "acquire_duplicate_chunk",
            Self::ExtraChunk => "acquire_extra_chunk",
            Self::FinalizePriorChunkDenied => "acquire_finalize_prior_chunk_denied",
            Self::FinalizeMissingChunks => "acquire_finalize_missing_chunks",
            Self::FinalizeShortBody => "acquire_finalize_short_body",
            Self::FinalizeLongBody => "acquire_finalize_long_body",
            Self::SharedVerifierDenied => "acquire_shared_m12_verifier_denied",
        }
    }

    const fn abi(self) -> i32 {
        match self {
            Self::KillGenerationChanged => HOST_IMPORT_ERROR_KILLED,
            Self::GuestMemoryFault
            | Self::ChunkIndexMismatch
            | Self::ChunkLengthMismatch
            | Self::ChunkHashMismatch => HOST_IMPORT_ERROR_INVALID_ARGUMENT,
            Self::ExtraChunk | Self::FinalizeLongBody => HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
            Self::SourceTlsEvidenceMismatch
            | Self::CatalogMismatch
            | Self::ReceiverIdentityMismatch
            | Self::PostureDenied => HOST_IMPORT_ERROR_CAPABILITY_DENIED,
            _ => HOST_IMPORT_ERROR_INVALID_STATE,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AcquireFixtureCase {
    pub(crate) name: &'static str,
    pub(crate) denial: &'static str,
    pub(crate) denied: bool,
    pub(crate) prior_candidate_unchanged: bool,
    pub(crate) pending_dropped: bool,
}

impl AcquireFixtureCase {
    pub(super) const fn failed(name: &'static str) -> Self {
        Self {
            name,
            denial: "fixture_setup_failed",
            denied: false,
            prior_candidate_unchanged: false,
            pending_dropped: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AcquireFixtureProbeSnapshot {
    pub(crate) positive_complete: bool,
    pub(crate) candidate_sha256: Option<[u8; 32]>,
    pub(crate) receipt_sha256: Option<[u8; 32]>,
    pub(crate) serial_candidate_sha256: Option<[u8; 32]>,
    pub(crate) serial_receipt_sha256: Option<[u8; 32]>,
    pub(crate) candidate_hash_converged: bool,
    pub(crate) receipt_hash_converged: bool,
    pub(crate) cases: [AcquireFixtureCase; ACQUIRE_FIXTURE_CASE_COUNT],
    pub(crate) failure_count: usize,
    pub(crate) typed_denials_pairwise_distinct: bool,
    pub(crate) all_prior_candidates_preserved: bool,
    pub(crate) all_incomplete_acquisitions_dropped: bool,
    pub(crate) direct_candidate_intake_calls: u64,
}

pub(super) struct AcquisitionInvocationState {
    owner_invocation_id: u64,
    session_current: bool,
    posture_allows_execution: bool,
    source_tls_evidence_valid: bool,
    catalog_valid: bool,
    receiver_identity_valid: bool,
    pending: Option<crate::agent_protocol::agent_protocol_registry::PendingDistributionAcquisition>,
    expected: [ChunkExpectation; 3],
    next_index: usize,
    accepted_bytes: usize,
    chunk_failure_observed: bool,
    finalized_candidate_sha256: Option<[u8; 32]>,
    receipt_sha256: Option<[u8; 32]>,
    last_denial: Option<AcquireDenial>,
}

impl AcquisitionInvocationState {
    pub(super) fn fixture(owner_invocation_id: u64, mode: AcquireFixtureMode) -> Self {
        let bytes = ECHO_WASM_ARTIFACT_BYTES;
        let first_end = bytes.len() / 3;
        let second_end = (bytes.len() * 2) / 3;
        let mut expected = [
            ChunkExpectation {
                len: first_end,
                sha256: sha256_bytes(&bytes[..first_end]),
            },
            ChunkExpectation {
                len: second_end - first_end,
                sha256: sha256_bytes(&bytes[first_end..second_end]),
            },
            ChunkExpectation {
                len: bytes.len() - second_end,
                sha256: sha256_bytes(&bytes[second_end..]),
            },
        ];
        if mode == AcquireFixtureMode::ShortBody {
            expected[2].len -= 1;
            expected[2].sha256 = sha256_bytes(&bytes[second_end..bytes.len() - 1]);
        } else if mode == AcquireFixtureMode::LongBody {
            let mut last = Vec::from(&bytes[second_end..]);
            last.push(0);
            expected[2].len += 1;
            expected[2].sha256 = sha256_bytes(&last);
        }
        Self {
            owner_invocation_id: if mode == AcquireFixtureMode::ForeignOwner {
                owner_invocation_id.wrapping_add(1)
            } else {
                owner_invocation_id
            },
            session_current: mode != AcquireFixtureMode::StaleSession,
            posture_allows_execution: mode != AcquireFixtureMode::PostureDenied,
            source_tls_evidence_valid: mode != AcquireFixtureMode::SourceEvidenceMismatch,
            catalog_valid: mode != AcquireFixtureMode::CatalogMismatch,
            receiver_identity_valid: mode != AcquireFixtureMode::ReceiverMismatch,
            pending: Some(
                crate::agent_protocol::agent_protocol_registry::PendingDistributionAcquisition::preauthorized_fixture(
                    crate::agent_protocol::distribution_registry::BUILTIN_ECHO_REGISTRY_ENTRY_ID,
                    ECHO_WASM_ARTIFACT_BYTES_HASH,
                    bytes.len(),
                    expected.len(),
                    crate::agent_protocol::distribution_registry::BUILTIN_ECHO_PROVENANCE_SIGNATURE_DER,
                ),
            ),
            expected,
            next_index: 0,
            accepted_bytes: 0,
            chunk_failure_observed: false,
            finalized_candidate_sha256: None,
            receipt_sha256: None,
            last_denial: None,
        }
    }

    fn record(&mut self, denial: Option<AcquireDenial>) {
        if denial.is_some() {
            self.last_denial = denial;
        }
    }

    pub(super) fn pending_present(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn candidate_sha256(&self) -> Option<[u8; 32]> {
        self.finalized_candidate_sha256
    }

    pub(super) fn receipt_sha256(&self) -> Option<[u8; 32]> {
        self.receipt_sha256
    }

    pub(super) fn last_denial(&self) -> &'static str {
        self.last_denial.map_or("none", AcquireDenial::as_str)
    }

    pub(super) fn expected_chunk_len(&self, index: usize) -> Option<usize> {
        self.expected.get(index).map(|chunk| chunk.len)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AcquireShimGrantProbe {
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) import_list_sha256: [u8; 32],
    pub(crate) denial_reason: &'static str,
    pub(crate) denied_before_instantiation: bool,
    pub(crate) requested_import_count: u64,
}

pub(crate) fn acquire_shim_grant_probe() -> AcquireShimGrantProbe {
    const IMPORTS: &[(&str, &str)] = &[("acquire", "chunk_accept"), ("acquire", "finalize")];
    let artifact_sha256 = sha256_bytes(ACQUIRE_SHIM_WASM_MODULE);
    let import_list_sha256 = host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, IMPORTS);
    let evidence = |evidence_sha256| VerifiedImportEvidence {
        evidence_sha256,
        artifact_sha256,
        import_list_sha256,
    };
    let decision = evaluate_evidence_bound_wasm_import_grant(&EvidenceBoundWasmImportGrantInput {
        service_id: Some(ACQUIRE_SERVICE_ID),
        artifact_sha256: Some(artifact_sha256),
        host_import_abi: Some(HOST_IMPORT_ABI_V1),
        declared_import_list_sha256: Some(import_list_sha256),
        requested_imports: IMPORTS,
        descriptor_source_signature_evidence: Some(evidence([0x61; 32])),
        artifact_signature_attestation_evidence: Some(evidence([0x62; 32])),
        computed_grant_evidence: Some(evidence([0x63; 32])),
        observed_imports: Some(ObservedWasmImports {
            artifact_sha256,
            import_list_sha256,
            imports: IMPORTS,
        }),
        linker_implementations: IMPORTS,
        policy_allows_beyond_env: false,
    });
    AcquireShimGrantProbe {
        artifact_sha256,
        import_list_sha256,
        denial_reason: decision.reason,
        denied_before_instantiation: !decision.performed,
        requested_import_count: ACQUIRE_HOST_IMPORTS_V1.len() as u64,
    }
}

pub(super) fn host_acquire_chunk_accept(
    mut caller: Caller<'_, BeyondEnvState>,
    index: i32,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    if let Err(denial) = check_acquire_call_boundary(caller.data_mut()) {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            denial,
        ));
    }
    if index < 0 || ptr < 0 || len < 0 {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::ChunkIndexMismatch,
        ));
    }
    let index = index as usize;
    let ptr = ptr as usize;
    let len = len as usize;
    if len > ACQUIRE_MAX_CHUNK_BYTES {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::ChunkLengthMismatch,
        ));
    }
    let (next_index, expected) = {
        let state = caller
            .data()
            .acquire
            .as_ref()
            .expect("acquire fixture missing");
        if state.next_index >= state.expected.len() {
            return Ok(record_denial(
                caller.data_mut(),
                "acquire.chunk_accept",
                AcquireDenial::ExtraChunk,
            ));
        }
        if index >= state.expected.len() {
            return Ok(record_denial(
                caller.data_mut(),
                "acquire.chunk_accept",
                AcquireDenial::ChunkIndexMismatch,
            ));
        }
        if index < state.next_index {
            return Ok(record_denial(
                caller.data_mut(),
                "acquire.chunk_accept",
                AcquireDenial::DuplicateChunk,
            ));
        }
        if index > state.next_index {
            return Ok(record_denial(
                caller.data_mut(),
                "acquire.chunk_accept",
                AcquireDenial::ChunkOutOfOrder,
            ));
        }
        (state.next_index, state.expected[state.next_index])
    };
    if len != expected.len {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::ChunkLengthMismatch,
        ));
    }
    caller
        .consume_fuel(50 + len as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("acquire.chunk_accept memory export missing"))?;
    let mut bytes = alloc::vec![0u8; len];
    if ptr
        .checked_add(len)
        .is_none_or(|end| end > memory.data(&caller).len())
        || memory.read(&caller, ptr, &mut bytes).is_err()
    {
        record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::GuestMemoryFault,
        );
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    }
    if sha256_bytes(&bytes) != expected.sha256 {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::ChunkHashMismatch,
        ));
    }
    let result = {
        let state = caller
            .data_mut()
            .acquire
            .as_mut()
            .expect("acquire fixture missing");
        crate::agent_protocol::agent_protocol_registry::accept_acquisition_chunk(
            state.pending.as_mut().expect("pending acquisition missing"),
            index,
            bytes,
            expected.sha256,
        )
    };
    if result.is_err() {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.chunk_accept",
            AcquireDenial::SharedVerifierDenied,
        ));
    }
    let state = caller
        .data_mut()
        .acquire
        .as_mut()
        .expect("acquire fixture missing");
    state.next_index = next_index + 1;
    state.accepted_bytes += len;
    state.record(None);
    Ok(0)
}

pub(super) fn host_acquire_finalize(mut caller: Caller<'_, BeyondEnvState>) -> Result<i32, Trap> {
    if let Err(denial) = check_acquire_call_boundary(caller.data_mut()) {
        return Ok(record_denial(caller.data_mut(), "acquire.finalize", denial));
    }
    caller
        .consume_fuel(100)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let precheck = {
        let state = caller
            .data()
            .acquire
            .as_ref()
            .expect("acquire fixture missing");
        if state.chunk_failure_observed {
            Some(AcquireDenial::FinalizePriorChunkDenied)
        } else if !state.catalog_valid {
            Some(AcquireDenial::CatalogMismatch)
        } else if !state.receiver_identity_valid {
            Some(AcquireDenial::ReceiverIdentityMismatch)
        } else if state.next_index != state.expected.len() {
            Some(AcquireDenial::FinalizeMissingChunks)
        } else {
            let total_length = state
                .pending
                .as_ref()
                .expect("pending acquisition missing")
                .total_length();
            if state.accepted_bytes < total_length {
                Some(AcquireDenial::FinalizeShortBody)
            } else if state.accepted_bytes > total_length {
                Some(AcquireDenial::FinalizeLongBody)
            } else {
                None
            }
        }
    };
    if let Some(denial) = precheck {
        return Ok(record_denial(caller.data_mut(), "acquire.finalize", denial));
    }
    let finalized = {
        let state = caller
            .data()
            .acquire
            .as_ref()
            .expect("acquire fixture missing");
        crate::agent_protocol::agent_protocol_registry::finalize_acquisition(
            state.pending.as_ref().expect("pending acquisition missing"),
        )
    };
    let candidate_sha256 = finalized
        .staged_candidate
        .filter(|candidate| {
            candidate.retained_in_ram && candidate.wasm_valid && !candidate.rejected
        })
        .map(|candidate| candidate.artifact_sha256);
    if finalized.status != "selected"
        || candidate_sha256.is_none()
        || finalized.receipt_sha256.is_none()
    {
        return Ok(record_denial(
            caller.data_mut(),
            "acquire.finalize",
            AcquireDenial::SharedVerifierDenied,
        ));
    }
    let state = caller
        .data_mut()
        .acquire
        .as_mut()
        .expect("acquire fixture missing");
    state.finalized_candidate_sha256 = candidate_sha256;
    state.receipt_sha256 = finalized.receipt_sha256;
    state.pending = None;
    state.session_current = false;
    state.record(None);
    Ok(0)
}

pub(super) fn finish_acquire_resources(state: &mut BeyondEnvState) -> bool {
    state.acquire.as_mut().is_none_or(|acquire| {
        acquire.pending = None;
        !acquire.pending_present()
    })
}

fn check_acquire_call_boundary(state: &mut BeyondEnvState) -> Result<(), AcquireDenial> {
    let authority = state.lifecycle.authority();
    if crate::input::secure_attention_kill_generation() != authority.captured_kill_generation {
        return Err(AcquireDenial::KillGenerationChanged);
    }
    if authority.service_id != ACQUIRE_SERVICE_ID
        || authority.invocation_id == 0
        || authority.service_generation == 0
        || authority.instance_generation == 0
    {
        return Err(AcquireDenial::InvocationAuthorityInvalid);
    }
    let acquisition = state
        .acquire
        .as_ref()
        .ok_or(AcquireDenial::SessionNotCurrent)?;
    if acquisition.owner_invocation_id != authority.invocation_id {
        return Err(AcquireDenial::SessionOwnerMismatch);
    }
    if !acquisition.session_current {
        return Err(AcquireDenial::SessionNotCurrent);
    }
    if !acquisition.posture_allows_execution {
        return Err(AcquireDenial::PostureDenied);
    }
    state
        .lifecycle
        .check_boundary(BoundaryState {
            now_ms: runtime_now_ms(),
            kill_generation: authority.captured_kill_generation,
            service_generation: authority.service_generation,
            instance_generation: authority.instance_generation,
            posture_allows_execution: true,
        })
        .map_err(|_| AcquireDenial::InvocationAuthorityInvalid)?;
    if !state
        .acquire
        .as_ref()
        .is_some_and(|acquire| acquire.source_tls_evidence_valid)
    {
        return Err(AcquireDenial::SourceTlsEvidenceMismatch);
    }
    Ok(())
}

fn record_denial(state: &mut BeyondEnvState, import: &'static str, denial: AcquireDenial) -> i32 {
    if let Some(acquire) = state.acquire.as_mut() {
        if import == "acquire.chunk_accept" {
            acquire.chunk_failure_observed = true;
        }
        acquire.record(Some(denial));
    }
    denial.abi()
}
