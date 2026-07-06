use alloc::{vec, vec::Vec};

use crate::wasm_runtime;
use raios_core::sha256_bytes;
use spin::Mutex;

// Large enough for the current echo Wasm artifact plus growth room, still small
// enough that M6A v0 never treats arbitrary large runtime bytes as acceptable.
pub(crate) const MAX_EXTERNAL_WASM_CANDIDATE_BYTES: usize = 262_144;
pub(crate) const EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL: &str = "serial_console_base64_chunks_v0";

const CURRENT_BOOT_SCOPE: &str = "current_boot";
const RETAINED_INERT_REASON: &str = "retained_current_boot_inert_candidate";
const OVERSIZE_REJECTION_REASON: &str = "rejected_oversize_candidate";
const MALFORMED_EXTERNAL_WASM_CANDIDATE: &[u8] = b"not a wasm module";

#[derive(Clone, Copy)]
pub(crate) struct ExternalWasmCandidateOutcome {
    pub(crate) byte_len: usize,
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) wasm_valid: bool,
    pub(crate) scope: &'static str,
    pub(crate) retained_in_ram: bool,
    pub(crate) rejected: bool,
    pub(crate) reason: &'static str,
    pub(crate) load_attempted: bool,
    pub(crate) execution_attempted: bool,
    pub(crate) authorizes_load: bool,
    pub(crate) authorizes_execution: bool,
    pub(crate) writes_persistent_state: bool,
    pub(crate) external_delivery_channel: &'static str,
}

pub(crate) struct CandidateIntakeProbe {
    pub(crate) echo_external_candidate: ExternalWasmCandidateOutcome,
    pub(crate) malformed_under_bound_candidate: ExternalWasmCandidateOutcome,
    pub(crate) oversize_candidate: ExternalWasmCandidateOutcome,
}

#[derive(Clone)]
pub(crate) struct RetainedExternalWasmCandidate {
    pub(crate) bytes: Vec<u8>,
    pub(crate) sha256: [u8; 32],
    pub(crate) wasm_valid: bool,
}

static RETAINED_CANDIDATE: Mutex<Option<RetainedExternalWasmCandidate>> = Mutex::new(None);

#[used]
static CANDIDATE_INTAKE_PROOF: fn() -> bool = validate_candidate_intake_probe;

pub(crate) fn intake_external_wasm_candidate(bytes: &[u8]) -> ExternalWasmCandidateOutcome {
    let artifact_sha256 = sha256_bytes(bytes);

    if bytes.len() > MAX_EXTERNAL_WASM_CANDIDATE_BYTES {
        return candidate_outcome(
            bytes.len(),
            artifact_sha256,
            false,
            false,
            true,
            OVERSIZE_REJECTION_REASON,
        );
    }

    candidate_outcome(
        bytes.len(),
        artifact_sha256,
        wasm_runtime::validate_module_bytes(bytes),
        true,
        false,
        RETAINED_INERT_REASON,
    )
}

pub(crate) fn run_candidate_intake_probe() -> CandidateIntakeProbe {
    let oversize = vec![0u8; MAX_EXTERNAL_WASM_CANDIDATE_BYTES + 1];

    CandidateIntakeProbe {
        echo_external_candidate: intake_external_wasm_candidate(
            wasm_runtime::ECHO_WASM_ARTIFACT_BYTES,
        ),
        malformed_under_bound_candidate: intake_external_wasm_candidate(
            MALFORMED_EXTERNAL_WASM_CANDIDATE,
        ),
        oversize_candidate: intake_external_wasm_candidate(&oversize),
    }
}

pub(crate) fn retain(bytes: Vec<u8>, sha256: [u8; 32], wasm_valid: bool) {
    *RETAINED_CANDIDATE.lock() = Some(RetainedExternalWasmCandidate {
        bytes,
        sha256,
        wasm_valid,
    });
}

pub(crate) fn retained() -> Option<RetainedExternalWasmCandidate> {
    RETAINED_CANDIDATE.lock().clone()
}

pub(crate) fn clear() {
    *RETAINED_CANDIDATE.lock() = None;
}

fn candidate_outcome(
    byte_len: usize,
    artifact_sha256: [u8; 32],
    wasm_valid: bool,
    retained_in_ram: bool,
    rejected: bool,
    reason: &'static str,
) -> ExternalWasmCandidateOutcome {
    ExternalWasmCandidateOutcome {
        byte_len,
        artifact_sha256,
        wasm_valid,
        scope: CURRENT_BOOT_SCOPE,
        retained_in_ram,
        rejected,
        reason,
        load_attempted: false,
        execution_attempted: false,
        authorizes_load: false,
        authorizes_execution: false,
        writes_persistent_state: false,
        external_delivery_channel: EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL,
    }
}

fn validate_candidate_intake_probe() -> bool {
    let probe = run_candidate_intake_probe();

    retained_echo_candidate_held(&probe.echo_external_candidate)
        && retained_malformed_candidate_held(&probe.malformed_under_bound_candidate)
        && rejected_oversize_candidate_held(&probe.oversize_candidate)
}

fn retained_echo_candidate_held(outcome: &ExternalWasmCandidateOutcome) -> bool {
    common_denials_hold(outcome)
        && outcome.byte_len == wasm_runtime::ECHO_WASM_ARTIFACT_BYTES.len()
        && outcome.artifact_sha256 == wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH
        && outcome.wasm_valid
        && outcome.retained_in_ram
        && !outcome.rejected
}

fn retained_malformed_candidate_held(outcome: &ExternalWasmCandidateOutcome) -> bool {
    common_denials_hold(outcome)
        && outcome.byte_len == MALFORMED_EXTERNAL_WASM_CANDIDATE.len()
        && outcome.artifact_sha256 != wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH
        && !outcome.wasm_valid
        && outcome.retained_in_ram
        && !outcome.rejected
}

fn rejected_oversize_candidate_held(outcome: &ExternalWasmCandidateOutcome) -> bool {
    common_denials_hold(outcome)
        && outcome.byte_len == MAX_EXTERNAL_WASM_CANDIDATE_BYTES + 1
        && !outcome.wasm_valid
        && !outcome.retained_in_ram
        && outcome.rejected
}

fn common_denials_hold(outcome: &ExternalWasmCandidateOutcome) -> bool {
    outcome.scope == CURRENT_BOOT_SCOPE
        && outcome.external_delivery_channel == EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL
        && !outcome.load_attempted
        && !outcome.execution_attempted
        && !outcome.authorizes_load
        && !outcome.authorizes_execution
        && !outcome.writes_persistent_state
}
