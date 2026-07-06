use alloc::vec::Vec;
use spin::Mutex;

use crate::module_candidate_intake::{
    self, ExternalWasmCandidateOutcome, EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL,
    MAX_EXTERNAL_WASM_CANDIDATE_BYTES,
};
use raios_core::sha256_bytes;

const CURRENT_BOOT_SCOPE: &str = "current_boot";
const EMPTY_FINALIZE_REJECTION_REASON: &str = "rejected_empty_candidate_delivery";
const ACCEPTED_CHUNK_REASON: &str = "accepted_candidate_chunk";
const MALFORMED_CHUNK_REASON: &str = "rejected_malformed_base64_chunk";
const EMPTY_CHUNK_REASON: &str = "rejected_empty_base64_chunk";
const INVALID_LENGTH_REASON: &str = "rejected_invalid_base64_length";
const INVALID_PADDING_REASON: &str = "rejected_invalid_base64_padding";
const OVERFLOW_REASON: &str = "rejected_candidate_delivery_overflow";
const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";

static PENDING_DELIVERY: Mutex<PendingDelivery> = Mutex::new(PendingDelivery::new());

#[used]
static MODULE_CANDIDATE_CHANNEL_PROOF: fn() -> bool = validate_module_candidate_channel;

pub(crate) struct ChunkOutcome {
    pub(crate) chunk_index: usize,
    pub(crate) decoded_byte_len: usize,
    pub(crate) pending_byte_len: usize,
    pub(crate) pending_chunk_count: usize,
    pub(crate) accepted: bool,
    pub(crate) rejected: bool,
    pub(crate) discarded_pending_delivery: bool,
    pub(crate) reason: &'static str,
    pub(crate) load_attempted: bool,
    pub(crate) execution_attempted: bool,
    pub(crate) authorizes_load: bool,
    pub(crate) authorizes_execution: bool,
    pub(crate) writes_persistent_state: bool,
    pub(crate) external_delivery_channel: &'static str,
}

pub(crate) struct FinalizeOutcome {
    pub(crate) delivered_byte_len: usize,
    pub(crate) delivered_chunk_count: usize,
    pub(crate) candidate: ExternalWasmCandidateOutcome,
}

struct PendingDelivery {
    bytes: Vec<u8>,
    chunk_count: usize,
}

impl PendingDelivery {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            chunk_count: 0,
        }
    }

    fn clear(&mut self) {
        self.bytes.clear();
        self.chunk_count = 0;
    }
}

pub(crate) fn submit_candidate_chunk(arg: &str) -> ChunkOutcome {
    let payload = candidate_chunk_payload(arg);
    let decoded = match decode_base64_chunk(payload) {
        Ok(decoded) => decoded,
        Err(reason) => {
            let mut pending = PENDING_DELIVERY.lock();
            let chunk_index = pending.chunk_count.saturating_add(1);
            pending.clear();
            return rejected_chunk(chunk_index, 0, reason);
        }
    };

    let mut pending = PENDING_DELIVERY.lock();
    append_decoded_chunk(&mut pending, decoded)
}

pub(crate) fn submit_candidate_finalize() -> FinalizeOutcome {
    let mut pending = PENDING_DELIVERY.lock();
    if pending.bytes.is_empty() {
        let chunk_count = pending.chunk_count;
        pending.clear();
        module_candidate_intake::clear();
        return FinalizeOutcome {
            delivered_byte_len: 0,
            delivered_chunk_count: chunk_count,
            candidate: rejected_empty_candidate(),
        };
    }

    let bytes = core::mem::take(&mut pending.bytes);
    let chunk_count = pending.chunk_count;
    pending.chunk_count = 0;
    drop(pending);

    let candidate = module_candidate_intake::intake_external_wasm_candidate(&bytes);
    module_candidate_intake::clear();
    if candidate.retained_in_ram && candidate.wasm_valid && !candidate.rejected {
        module_candidate_intake::retain(bytes, candidate.artifact_sha256, candidate.wasm_valid);
    }
    FinalizeOutcome {
        delivered_byte_len: candidate.byte_len,
        delivered_chunk_count: chunk_count,
        candidate,
    }
}

fn append_decoded_chunk(pending: &mut PendingDelivery, decoded: Vec<u8>) -> ChunkOutcome {
    let chunk_index = pending.chunk_count.saturating_add(1);
    let decoded_byte_len = decoded.len();
    if decoded_byte_len > MAX_EXTERNAL_WASM_CANDIDATE_BYTES.saturating_sub(pending.bytes.len()) {
        pending.clear();
        return rejected_chunk(chunk_index, decoded_byte_len, OVERFLOW_REASON);
    }

    pending.bytes.extend_from_slice(&decoded);
    pending.chunk_count = chunk_index;
    ChunkOutcome {
        chunk_index,
        decoded_byte_len,
        pending_byte_len: pending.bytes.len(),
        pending_chunk_count: pending.chunk_count,
        accepted: true,
        rejected: false,
        discarded_pending_delivery: false,
        reason: ACCEPTED_CHUNK_REASON,
        load_attempted: false,
        execution_attempted: false,
        authorizes_load: false,
        authorizes_execution: false,
        writes_persistent_state: false,
        external_delivery_channel: EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL,
    }
}

fn rejected_chunk(
    chunk_index: usize,
    decoded_byte_len: usize,
    reason: &'static str,
) -> ChunkOutcome {
    ChunkOutcome {
        chunk_index,
        decoded_byte_len,
        pending_byte_len: 0,
        pending_chunk_count: 0,
        accepted: false,
        rejected: true,
        discarded_pending_delivery: true,
        reason,
        load_attempted: false,
        execution_attempted: false,
        authorizes_load: false,
        authorizes_execution: false,
        writes_persistent_state: false,
        external_delivery_channel: EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL,
    }
}

fn rejected_empty_candidate() -> ExternalWasmCandidateOutcome {
    ExternalWasmCandidateOutcome {
        byte_len: 0,
        artifact_sha256: sha256_bytes(&[]),
        wasm_valid: false,
        scope: CURRENT_BOOT_SCOPE,
        retained_in_ram: false,
        rejected: true,
        reason: EMPTY_FINALIZE_REJECTION_REASON,
        load_attempted: false,
        execution_attempted: false,
        authorizes_load: false,
        authorizes_execution: false,
        writes_persistent_state: false,
        external_delivery_channel: EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL,
    }
}

fn candidate_chunk_payload(arg: &str) -> &str {
    let trimmed = arg.trim();
    if let Some(rest) = trimmed.strip_prefix("module.submit_candidate_chunk") {
        rest.trim()
    } else {
        trimmed
    }
}

fn decode_base64_chunk(input: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Err(EMPTY_CHUNK_REASON);
    }
    if bytes.len() % 4 != 0 {
        return Err(INVALID_LENGTH_REASON);
    }

    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let last_quad = idx + 4 == bytes.len();
        let a = base64_value(bytes[idx])?;
        let b = base64_value(bytes[idx + 1])?;
        let c_pad = bytes[idx + 2] == b'=';
        let d_pad = bytes[idx + 3] == b'=';
        if (c_pad || d_pad) && !last_quad {
            return Err(INVALID_PADDING_REASON);
        }
        if c_pad && !d_pad {
            return Err(INVALID_PADDING_REASON);
        }
        let c = if c_pad {
            0
        } else {
            base64_value(bytes[idx + 2])?
        };
        let d = if d_pad {
            0
        } else {
            base64_value(bytes[idx + 3])?
        };

        out.push((a << 2) | (b >> 4));
        if !c_pad {
            out.push(((b & 0x0f) << 4) | (c >> 2));
        }
        if !d_pad {
            out.push(((c & 0x03) << 6) | d);
        }
        idx += 4;
    }
    Ok(out)
}

fn base64_value(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        b'=' => Err(INVALID_PADDING_REASON),
        _ => Err(MALFORMED_CHUNK_REASON),
    }
}

fn validate_module_candidate_channel() -> bool {
    {
        let mut pending = PENDING_DELIVERY.lock();
        pending.clear();
    }

    let first = submit_candidate_chunk("AGFz");
    let second = submit_candidate_chunk("bQEA");
    let third = submit_candidate_chunk("AAA=");
    let finalized = submit_candidate_finalize();
    let happy_path = first.accepted
        && second.accepted
        && third.accepted
        && finalized.delivered_byte_len == EMPTY_WASM_MODULE.len()
        && finalized.delivered_chunk_count == 3
        && finalized.candidate.artifact_sha256 == sha256_bytes(EMPTY_WASM_MODULE)
        && finalized.candidate.wasm_valid
        && finalized.candidate.retained_in_ram
        && !finalized.candidate.rejected
        && common_candidate_denials_hold(&finalized.candidate);

    let malformed = submit_candidate_chunk("!!!!");

    let mut local_pending = PendingDelivery::new();
    local_pending
        .bytes
        .resize(MAX_EXTERNAL_WASM_CANDIDATE_BYTES, 0);
    local_pending.chunk_count = 1;
    let overflow = append_decoded_chunk(&mut local_pending, Vec::from([0u8]));

    {
        let mut pending = PENDING_DELIVERY.lock();
        pending.clear();
    }

    happy_path
        && malformed.rejected
        && malformed.discarded_pending_delivery
        && malformed.pending_byte_len == 0
        && overflow.rejected
        && overflow.discarded_pending_delivery
        && local_pending.bytes.is_empty()
        && local_pending.chunk_count == 0
}

fn common_candidate_denials_hold(outcome: &ExternalWasmCandidateOutcome) -> bool {
    !outcome.load_attempted
        && !outcome.execution_attempted
        && !outcome.authorizes_load
        && !outcome.authorizes_execution
        && !outcome.writes_persistent_state
}
