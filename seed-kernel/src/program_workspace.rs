use alloc::{string::String, vec::Vec};

use raios_core::{
    ui_program::{Program, ProgramError, ProgramIdentity, MAX_PROGRAM_BYTES},
    ui_program_spec::{self, SpecError},
};
use spin::Mutex;

use crate::module_candidate_channel;

const PROVIDER_PREFIX: &str = "RUIP_BASE64:";
const SERIAL_CHUNK_METHOD: &str = "program.submit_chunk";
const MAX_PROGRAM_BASE64_BYTES: usize = MAX_PROGRAM_BYTES.div_ceil(3) * 4;

static WORKSPACE: Mutex<Workspace> = Mutex::new(Workspace::new());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Source {
    Provider { request_id: u32 },
    Serial { chunk_count: usize },
}

impl Source {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Provider { .. } => "provider",
            Self::Serial { .. } => "serial",
        }
    }

    pub(crate) const fn request_id(self) -> Option<u32> {
        match self {
            Self::Provider { request_id } => Some(request_id),
            Self::Serial { .. } => None,
        }
    }

    pub(crate) const fn chunk_count(self) -> Option<usize> {
        match self {
            Self::Provider { .. } => None,
            Self::Serial { chunk_count } => Some(chunk_count),
        }
    }
}

#[derive(Clone)]
struct Draft {
    program: Program,
    _canonical_bytes: Vec<u8>,
    identity: ProgramIdentity,
    source: Source,
    revision: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) present: bool,
    pub(crate) revision: u64,
    pub(crate) byte_len: usize,
    pub(crate) sha256: Option<[u8; 32]>,
    pub(crate) source: Option<Source>,
    pub(crate) pending_byte_len: usize,
    pub(crate) pending_chunk_count: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct ChunkOutcome {
    pub(crate) accepted: bool,
    pub(crate) rejected: bool,
    pub(crate) reason: &'static str,
    pub(crate) decoded_byte_len: usize,
    pub(crate) pending_byte_len: usize,
    pub(crate) pending_chunk_count: usize,
    pub(crate) discarded_pending_delivery: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct IntakeOutcome {
    pub(crate) accepted: bool,
    pub(crate) rejected: bool,
    pub(crate) reason: &'static str,
    pub(crate) attempted_byte_len: usize,
    pub(crate) snapshot: Snapshot,
}

struct Workspace {
    pending_serial: Vec<u8>,
    pending_chunk_count: usize,
    next_revision: u64,
    retained: Option<Draft>,
}

impl Workspace {
    const fn new() -> Self {
        Self {
            pending_serial: Vec::new(),
            pending_chunk_count: 0,
            next_revision: 1,
            retained: None,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let (present, revision, byte_len, sha256, source) = match self.retained.as_ref() {
            Some(draft) => (
                true,
                draft.revision,
                draft._canonical_bytes.len(),
                Some(draft.identity.sha256),
                Some(draft.source),
            ),
            None => (false, 0, 0, None, None),
        };
        Snapshot {
            present,
            revision,
            byte_len,
            sha256,
            source,
            pending_byte_len: self.pending_serial.len(),
            pending_chunk_count: self.pending_chunk_count,
        }
    }

    fn clear_pending(&mut self) {
        self.pending_serial.clear();
        self.pending_chunk_count = 0;
    }

    fn retain(&mut self, bytes: Vec<u8>, source: Source) -> IntakeOutcome {
        let attempted_byte_len = bytes.len();
        let program = match Program::parse(&bytes) {
            Ok(program) => program,
            Err(error) => {
                return IntakeOutcome {
                    accepted: false,
                    rejected: true,
                    reason: program_error_reason(error),
                    attempted_byte_len,
                    snapshot: self.snapshot(),
                };
            }
        };
        if program.canonical_bytes() != bytes {
            return IntakeOutcome {
                accepted: false,
                rejected: true,
                reason: "program_not_canonical",
                attempted_byte_len,
                snapshot: self.snapshot(),
            };
        }
        self.retain_program(program, attempted_byte_len, source)
    }

    fn retain_program(
        &mut self,
        program: Program,
        attempted_byte_len: usize,
        source: Source,
    ) -> IntakeOutcome {
        let canonical_bytes = program.canonical_bytes();
        let identity = program.identity();
        let revision = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        self.retained = Some(Draft {
            program,
            _canonical_bytes: canonical_bytes,
            identity,
            source,
            revision,
        });
        IntakeOutcome {
            accepted: true,
            rejected: false,
            reason: "retained_current_boot_inert_ui_program",
            attempted_byte_len,
            snapshot: self.snapshot(),
        }
    }
}

pub(crate) fn snapshot() -> Snapshot {
    WORKSPACE.lock().snapshot()
}

/// Returns only the already-validated bounded program data. The workspace keeps
/// its copy; this grants no load, signing, execution, or persistence authority.
pub(crate) fn retained_program() -> Option<Program> {
    WORKSPACE
        .lock()
        .retained
        .as_ref()
        .map(|draft| draft.program.clone())
}

pub(crate) fn submit_serial_chunk(input: &str) -> ChunkOutcome {
    let payload = method_payload(input, SERIAL_CHUNK_METHOD);
    let decoded = match module_candidate_channel::decode_base64_chunk(payload) {
        Ok(decoded) => decoded,
        Err(reason) => {
            let mut workspace = WORKSPACE.lock();
            workspace.clear_pending();
            return ChunkOutcome {
                accepted: false,
                rejected: true,
                reason,
                decoded_byte_len: 0,
                pending_byte_len: 0,
                pending_chunk_count: 0,
                discarded_pending_delivery: true,
            };
        }
    };

    let mut workspace = WORKSPACE.lock();
    let decoded_byte_len = decoded.len();
    if decoded_byte_len > MAX_PROGRAM_BYTES.saturating_sub(workspace.pending_serial.len()) {
        workspace.clear_pending();
        return ChunkOutcome {
            accepted: false,
            rejected: true,
            reason: "rejected_program_delivery_overflow",
            decoded_byte_len,
            pending_byte_len: 0,
            pending_chunk_count: 0,
            discarded_pending_delivery: true,
        };
    }
    workspace.pending_serial.extend_from_slice(&decoded);
    workspace.pending_chunk_count = workspace.pending_chunk_count.saturating_add(1);
    ChunkOutcome {
        accepted: true,
        rejected: false,
        reason: "accepted_program_chunk",
        decoded_byte_len,
        pending_byte_len: workspace.pending_serial.len(),
        pending_chunk_count: workspace.pending_chunk_count,
        discarded_pending_delivery: false,
    }
}

pub(crate) fn finalize_serial() -> IntakeOutcome {
    let mut workspace = WORKSPACE.lock();
    if workspace.pending_serial.is_empty() {
        workspace.clear_pending();
        return IntakeOutcome {
            accepted: false,
            rejected: true,
            reason: "rejected_empty_program_delivery",
            attempted_byte_len: 0,
            snapshot: workspace.snapshot(),
        };
    }
    let bytes = core::mem::take(&mut workspace.pending_serial);
    let chunk_count = workspace.pending_chunk_count;
    workspace.pending_chunk_count = 0;
    workspace.retain(bytes, Source::Serial { chunk_count })
}

pub(crate) fn accept_provider_answer(request_id: u32, answer: String) -> IntakeOutcome {
    if let Some(encoded) = answer.strip_prefix(PROVIDER_PREFIX) {
        if encoded.len() > MAX_PROGRAM_BASE64_BYTES {
            return rejected_provider_answer("provider_program_base64_too_large", encoded.len());
        }
        let bytes = match module_candidate_channel::decode_base64_chunk(encoded) {
            Ok(bytes) => bytes,
            Err(_) => {
                return rejected_provider_answer("provider_program_base64_invalid", encoded.len())
            }
        };
        return WORKSPACE
            .lock()
            .retain(bytes, Source::Provider { request_id });
    }

    if answer == "RAIOS_UI_SPEC_V1"
        || answer.starts_with("RAIOS_UI_SPEC_V1\n")
        || answer.starts_with("RAIOS_UI_SPEC_V1\r\n")
    {
        let attempted_byte_len = answer.len();
        let program = match ui_program_spec::parse(answer.as_bytes()) {
            Ok(program) => program,
            Err(error) => {
                return rejected_provider_answer(spec_error_reason(error), attempted_byte_len)
            }
        };
        return WORKSPACE.lock().retain_program(
            program,
            attempted_byte_len,
            Source::Provider { request_id },
        );
    }

    rejected_provider_answer("provider_program_prefix_missing", answer.len())
}

fn rejected_provider_answer(reason: &'static str, attempted_byte_len: usize) -> IntakeOutcome {
    let workspace = WORKSPACE.lock();
    IntakeOutcome {
        accepted: false,
        rejected: true,
        reason,
        attempted_byte_len,
        snapshot: workspace.snapshot(),
    }
}

fn method_payload<'a>(input: &'a str, method: &str) -> &'a str {
    let input = input.trim();
    input.strip_prefix(method).map(str::trim).unwrap_or(input)
}

const fn program_error_reason(error: ProgramError) -> &'static str {
    match error {
        ProgramError::WrongAbiVersion => "program_wrong_abi_version",
        ProgramError::Malformed => "program_malformed",
        ProgramError::LimitExceeded => "program_limit_exceeded",
        ProgramError::UnknownOpcode => "program_unknown_opcode",
        ProgramError::InvalidReference => "program_invalid_reference",
        ProgramError::DuplicateBinding => "program_duplicate_binding",
        ProgramError::OverlappingButtons => "program_overlapping_buttons",
    }
}

const fn spec_error_reason(error: SpecError) -> &'static str {
    match error {
        SpecError::InputTooLarge => "provider_program_spec_too_large",
        SpecError::InvalidUtf8 => "provider_program_spec_invalid_utf8",
        SpecError::Malformed | SpecError::UnknownCommand | SpecError::InvalidNumber => {
            "provider_program_spec_malformed"
        }
        SpecError::Program(error) => program_error_reason(error),
    }
}
