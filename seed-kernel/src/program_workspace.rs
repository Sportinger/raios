use alloc::{string::String, vec::Vec};

use raios_core::{
    sha256_bytes,
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
    original_request: Option<String>,
    provider_source_spec: Option<String>,
    provider_source_spec_sha256: Option<[u8; 32]>,
    parent_sha256: Option<[u8; 32]>,
    root_sha256: [u8; 32],
    lineage_depth: u64,
}

struct PendingProviderRequest {
    request_id: u32,
    original_request: String,
    parent_sha256: Option<[u8; 32]>,
    root_sha256: Option<[u8; 32]>,
    lineage_depth: u64,
}

pub(crate) struct RevisionContext {
    pub(crate) original_request: String,
    pub(crate) provider_source_spec: String,
    pub(crate) parent_sha256: [u8; 32],
    pub(crate) root_sha256: [u8; 32],
    pub(crate) lineage_depth: u64,
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
    pub(crate) pending_provider_request_id: Option<u32>,
    pub(crate) original_request_present: bool,
    pub(crate) original_request_byte_len: usize,
    pub(crate) provider_source_spec_present: bool,
    pub(crate) provider_source_spec_byte_len: usize,
    pub(crate) provider_source_spec_sha256: Option<[u8; 32]>,
    pub(crate) parent_sha256: Option<[u8; 32]>,
    pub(crate) root_sha256: Option<[u8; 32]>,
    pub(crate) lineage_depth: u64,
    pub(crate) last_rejection_reason: Option<&'static str>,
    pub(crate) last_rejection_attempted_byte_len: usize,
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
    pending_provider: Option<PendingProviderRequest>,
    last_rejection_reason: Option<&'static str>,
    last_rejection_attempted_byte_len: usize,
}

impl Workspace {
    const fn new() -> Self {
        Self {
            pending_serial: Vec::new(),
            pending_chunk_count: 0,
            next_revision: 1,
            retained: None,
            pending_provider: None,
            last_rejection_reason: None,
            last_rejection_attempted_byte_len: 0,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let (
            present,
            revision,
            byte_len,
            sha256,
            source,
            original_request_present,
            original_request_byte_len,
            provider_source_spec_present,
            provider_source_spec_byte_len,
            provider_source_spec_sha256,
            parent_sha256,
            root_sha256,
            lineage_depth,
        ) = match self.retained.as_ref() {
            Some(draft) => (
                true,
                draft.revision,
                draft._canonical_bytes.len(),
                Some(draft.identity.sha256),
                Some(draft.source),
                draft.original_request.is_some(),
                draft.original_request.as_ref().map_or(0, String::len),
                draft.provider_source_spec.is_some(),
                draft.provider_source_spec.as_ref().map_or(0, String::len),
                draft.provider_source_spec_sha256,
                draft.parent_sha256,
                Some(draft.root_sha256),
                draft.lineage_depth,
            ),
            None => (
                false, 0, 0, None, None, false, 0, false, 0, None, None, None, 0,
            ),
        };
        Snapshot {
            present,
            revision,
            byte_len,
            sha256,
            source,
            pending_byte_len: self.pending_serial.len(),
            pending_chunk_count: self.pending_chunk_count,
            pending_provider_request_id: self
                .pending_provider
                .as_ref()
                .map(|pending| pending.request_id),
            original_request_present,
            original_request_byte_len,
            provider_source_spec_present,
            provider_source_spec_byte_len,
            provider_source_spec_sha256,
            parent_sha256,
            root_sha256,
            lineage_depth,
            last_rejection_reason: self.last_rejection_reason,
            last_rejection_attempted_byte_len: self.last_rejection_attempted_byte_len,
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
                self.note_rejection(program_error_reason(error), attempted_byte_len);
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
            self.note_rejection("program_not_canonical", attempted_byte_len);
            return IntakeOutcome {
                accepted: false,
                rejected: true,
                reason: "program_not_canonical",
                attempted_byte_len,
                snapshot: self.snapshot(),
            };
        }
        self.retain_program(program, attempted_byte_len, source, None, None)
    }

    fn retain_program(
        &mut self,
        program: Program,
        attempted_byte_len: usize,
        source: Source,
        provider: Option<PendingProviderRequest>,
        provider_source_spec: Option<String>,
    ) -> IntakeOutcome {
        let canonical_bytes = program.canonical_bytes();
        let identity = program.identity();
        let revision = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        let parent_sha256 = provider.as_ref().and_then(|pending| pending.parent_sha256);
        let root_sha256 = provider
            .as_ref()
            .and_then(|pending| pending.root_sha256)
            .unwrap_or(identity.sha256);
        let lineage_depth = provider.as_ref().map_or(0, |pending| pending.lineage_depth);
        let original_request = provider.map(|pending| pending.original_request);
        let provider_source_spec_sha256 = provider_source_spec
            .as_ref()
            .map(|spec| sha256_bytes(spec.as_bytes()));
        self.retained = Some(Draft {
            program,
            _canonical_bytes: canonical_bytes,
            identity,
            source,
            revision,
            original_request,
            provider_source_spec,
            provider_source_spec_sha256,
            parent_sha256,
            root_sha256,
            lineage_depth,
        });
        self.last_rejection_reason = None;
        self.last_rejection_attempted_byte_len = 0;
        IntakeOutcome {
            accepted: true,
            rejected: false,
            reason: "retained_current_boot_inert_ui_program",
            attempted_byte_len,
            snapshot: self.snapshot(),
        }
    }

    fn note_rejection(&mut self, reason: &'static str, attempted_byte_len: usize) {
        self.last_rejection_reason = Some(reason);
        self.last_rejection_attempted_byte_len = attempted_byte_len;
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

pub(crate) fn note_provider_build_request(request_id: u32, request: &str) {
    WORKSPACE.lock().pending_provider = Some(PendingProviderRequest {
        request_id,
        original_request: String::from(request),
        parent_sha256: None,
        root_sha256: None,
        lineage_depth: 0,
    });
}

pub(crate) fn revision_context() -> Result<RevisionContext, &'static str> {
    let workspace = WORKSPACE.lock();
    let draft = workspace
        .retained
        .as_ref()
        .ok_or("program_revision_requires_retained_program")?;
    if matches!(draft.source, Source::Serial { .. }) {
        return Err("program_revision_serial_source_unsupported");
    }
    let original_request = draft
        .original_request
        .as_ref()
        .ok_or("program_revision_original_request_missing")?;
    let provider_source_spec = draft
        .provider_source_spec
        .as_ref()
        .ok_or("program_revision_provider_source_spec_unavailable")?;
    Ok(RevisionContext {
        original_request: original_request.clone(),
        provider_source_spec: provider_source_spec.clone(),
        parent_sha256: draft.identity.sha256,
        root_sha256: draft.root_sha256,
        lineage_depth: draft.lineage_depth.saturating_add(1),
    })
}

pub(crate) fn note_provider_revision_request(request_id: u32, context: RevisionContext) {
    WORKSPACE.lock().pending_provider = Some(PendingProviderRequest {
        request_id,
        original_request: context.original_request,
        parent_sha256: Some(context.parent_sha256),
        root_sha256: Some(context.root_sha256),
        lineage_depth: context.lineage_depth,
    });
}

pub(crate) fn note_provider_error(request_id: u32) {
    let mut workspace = WORKSPACE.lock();
    if workspace
        .pending_provider
        .as_ref()
        .is_some_and(|pending| pending.request_id == request_id)
    {
        workspace.pending_provider = None;
        workspace.note_rejection("provider_program_request_failed", 0);
    }
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
    let provider = {
        let mut workspace = WORKSPACE.lock();
        match workspace.pending_provider.take() {
            Some(pending) if pending.request_id == request_id => Some(pending),
            Some(pending) => {
                workspace.pending_provider = Some(pending);
                None
            }
            None => None,
        }
    };
    let Some(provider) = provider else {
        return rejected_provider_answer("provider_program_request_not_tracked", answer.len());
    };
    if let Some(parent_sha256) = provider.parent_sha256 {
        let parent_still_current = WORKSPACE
            .lock()
            .retained
            .as_ref()
            .is_some_and(|draft| draft.identity.sha256 == parent_sha256);
        if !parent_still_current {
            return rejected_provider_answer("program_revision_parent_changed", answer.len());
        }
    }
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
        let attempted_byte_len = bytes.len();
        let program = match Program::parse(&bytes) {
            Ok(program) if program.canonical_bytes() == bytes => program,
            Ok(_) => return rejected_provider_answer("program_not_canonical", attempted_byte_len),
            Err(error) => {
                return rejected_provider_answer(program_error_reason(error), attempted_byte_len)
            }
        };
        return WORKSPACE.lock().retain_program(
            program,
            attempted_byte_len,
            Source::Provider { request_id },
            Some(provider),
            None,
        );
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
            Some(provider),
            Some(answer),
        );
    }

    rejected_provider_answer("provider_program_prefix_missing", answer.len())
}

fn rejected_provider_answer(reason: &'static str, attempted_byte_len: usize) -> IntakeOutcome {
    let mut workspace = WORKSPACE.lock();
    workspace.note_rejection(reason, attempted_byte_len);
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
