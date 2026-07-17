use alloc::{string::String, vec::Vec};

use raios_core::{
    project_workspace::{
        agent_project_id, agent_source_media_type, Classification, ProjectId, ProjectRevision,
        SourceFile, MAX_TOTAL_SOURCE_BYTES,
    },
    sha256_bytes,
};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{module_candidate_channel::decode_base64_chunk, project_workspace};

const PROJECT_ID_DOMAIN: &[u8] = b"raios.agent_project_id.v1";
const ANSWER_HEADER: &str = "RAIOS_SOURCE_FILES_V1";
const MAX_ANSWER_BYTES: usize = MAX_TOTAL_SOURCE_BYTES * 3;
const FIXTURE_REQUEST_ID: u32 = 0xb2_01_a;
const FIXTURE_REQUEST: &str = "Build the fixed B2.1a Rust/TOML source fixture.";
const FIXTURE_ANSWER: &str = concat!(
    "RAIOS_SOURCE_FILES_V1\n",
    "file Q2FyZ28udG9tbA== W3BhY2thZ2VdCm5hbWUgPSAiYjIxYS1maXh0dXJlIgp2ZXJzaW9uID0gIjAuMS4wIgplZGl0aW9uID0gIjIwMjEiCg==\n",
    "file c3JjL21haW4ucnM= Zm4gbWFpbigpIHsKICAgIGxldCBtZXNzYWdlID0gImhlbGxvIGZyb20gcmFpT1MiOwogICAgbGV0IF8gPSBtZXNzYWdlOwp9Cg==\n",
    "end",
);

static STATE: Mutex<BuildLoopState> = Mutex::new(BuildLoopState::new());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopPhase {
    Idle,
    RequestingSource,
    SourceReady,
    Rejected,
}

impl LoopPhase {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::RequestingSource => "requesting_source",
            Self::SourceReady => "source_ready",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Copy)]
struct RequestIdentity {
    request_id: u32,
    project_id: ProjectId,
    request_sha256: [u8; 32],
}

#[derive(Clone)]
struct AcceptedAnswer {
    request: RequestIdentity,
    revision: ProjectRevision,
    answer_sha256: [u8; 32],
    answer_byte_len: usize,
    answer_origin: &'static str,
    provider_trust_positive: bool,
    test_infrastructure: bool,
}

struct BuildLoopState {
    phase: LoopPhase,
    current_request: Option<RequestIdentity>,
    pending_request: Option<RequestIdentity>,
    latest: Option<AcceptedAnswer>,
    last_reason: Option<&'static str>,
}

impl BuildLoopState {
    const fn new() -> Self {
        Self {
            phase: LoopPhase::Idle,
            current_request: None,
            pending_request: None,
            latest: None,
            last_reason: None,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let request = self
            .current_request
            .or_else(|| self.latest.as_ref().map(|latest| latest.request));
        Snapshot {
            phase: self.phase,
            pending_request_id: self.pending_request.map(|pending| pending.request_id),
            latest_request_id: self.latest.as_ref().map(|latest| latest.request.request_id),
            project_id: request.map(|request| request.project_id.bytes()),
            request_sha256: request.map(|request| request.request_sha256),
            latest_revision: self.latest.as_ref().map(|latest| latest.revision.clone()),
            answer_sha256: self.latest.as_ref().map(|latest| latest.answer_sha256),
            answer_byte_len: self.latest.as_ref().map(|latest| latest.answer_byte_len),
            answer_origin: self.latest.as_ref().map(|latest| latest.answer_origin),
            provider_trust_positive: self
                .latest
                .as_ref()
                .is_some_and(|latest| latest.provider_trust_positive),
            test_infrastructure: self
                .latest
                .as_ref()
                .is_some_and(|latest| latest.test_infrastructure),
            last_reason: self.last_reason,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Snapshot {
    pub(crate) phase: LoopPhase,
    pub(crate) pending_request_id: Option<u32>,
    pub(crate) latest_request_id: Option<u32>,
    pub(crate) project_id: Option<[u8; 16]>,
    pub(crate) request_sha256: Option<[u8; 32]>,
    pub(crate) latest_revision: Option<ProjectRevision>,
    pub(crate) answer_sha256: Option<[u8; 32]>,
    pub(crate) answer_byte_len: Option<usize>,
    pub(crate) answer_origin: Option<&'static str>,
    pub(crate) provider_trust_positive: bool,
    pub(crate) test_infrastructure: bool,
    pub(crate) last_reason: Option<&'static str>,
}

pub(crate) struct AnswerOutcome {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) answer_sha256: [u8; 32],
    pub(crate) answer_byte_len: usize,
    pub(crate) answer_origin: &'static str,
    pub(crate) provider_trust_positive: bool,
    pub(crate) test_infrastructure: bool,
    pub(crate) storage_write_attempted: bool,
    pub(crate) writes_persistent_state: bool,
    pub(crate) snapshot: Snapshot,
}

#[derive(Clone, Copy)]
struct AnswerProvenance {
    answer_origin: &'static str,
    provider_trust_positive: bool,
    test_infrastructure: bool,
}

const TEST_FIXTURE_PROVENANCE: AnswerProvenance = AnswerProvenance {
    answer_origin: "test_fixture",
    provider_trust_positive: false,
    test_infrastructure: true,
};

pub(crate) struct AnswerFile {
    pub(crate) path: String,
    pub(crate) content: Vec<u8>,
    pub(crate) media_type: &'static str,
}

pub(crate) fn derive_project_id(owner_request: &str) -> ProjectId {
    let mut hasher = Sha256::new();
    hasher.update(PROJECT_ID_DOMAIN);
    hasher.update((owner_request.len() as u64).to_le_bytes());
    hasher.update(owner_request.as_bytes());
    agent_project_id(hasher.finalize().into())
}

pub(crate) fn note_provider_build_request(
    request_id: u32,
    owner_request: &str,
) -> Result<(), &'static str> {
    let project_id = derive_project_id(owner_request);
    let request_sha256 = sha256_bytes(owner_request.as_bytes());
    if request_id == 0 || project_id.bytes() == [0; 16] || request_sha256 == [0; 32] {
        return Err("agent_build_request_identity_invalid");
    }
    let mut state = STATE.lock();
    if state.pending_request.is_some() {
        return Err("agent_build_request_already_pending");
    }
    let request = RequestIdentity {
        request_id,
        project_id,
        request_sha256,
    };
    state.phase = LoopPhase::RequestingSource;
    state.current_request = Some(request);
    state.pending_request = Some(request);
    state.last_reason = None;
    Ok(())
}

pub(crate) fn note_provider_error(request_id: u32) -> bool {
    let mut state = STATE.lock();
    if !state
        .pending_request
        .is_some_and(|pending| pending.request_id == request_id)
    {
        return false;
    }
    state.pending_request = None;
    state.phase = LoopPhase::Rejected;
    state.last_reason = Some("provider_project_request_failed");
    true
}

pub(crate) fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

pub(crate) fn parse_answer(answer: &str) -> Result<Vec<AnswerFile>, &'static str> {
    if answer.len() > MAX_ANSWER_BYTES {
        return Err("agent_answer_too_large");
    }
    let mut lines = answer.lines();
    if lines.next() != Some(ANSWER_HEADER) {
        return Err("agent_answer_header_invalid");
    }

    let mut files = Vec::new();
    for line in lines.by_ref() {
        if line == "end" {
            if files.is_empty() {
                return Err("agent_answer_no_files");
            }
            if lines.next().is_some() {
                return Err("agent_answer_trailing_content");
            }
            return Ok(files);
        }
        let encoded = line
            .strip_prefix("file ")
            .ok_or("agent_answer_line_invalid")?;
        let (encoded_path, encoded_content) =
            encoded.split_once(' ').ok_or("agent_answer_line_invalid")?;
        if encoded_path.is_empty() || encoded_content.is_empty() || encoded_content.contains(' ') {
            return Err("agent_answer_line_invalid");
        }
        let path = String::from_utf8(
            decode_base64_chunk(encoded_path).map_err(|_| "agent_answer_path_base64_invalid")?,
        )
        .map_err(|_| "agent_answer_path_utf8_invalid")?;
        let media_type = media_type_for_path(&path)?;
        let content = decode_base64_chunk(encoded_content)
            .map_err(|_| "agent_answer_content_base64_invalid")?;
        core::str::from_utf8(&content).map_err(|_| "agent_answer_content_utf8_invalid")?;
        files.push(AnswerFile {
            path,
            content,
            media_type,
        });
    }
    Err("agent_answer_end_missing")
}

pub(crate) fn media_type_for_path(path: &str) -> Result<&'static str, &'static str> {
    agent_source_media_type(path).map_err(|error| error.reason())
}

pub(crate) fn accept_test_fixture() -> AnswerOutcome {
    let should_start = {
        let state = STATE.lock();
        state.phase == LoopPhase::Idle && state.current_request.is_none()
    };
    if should_start {
        if let Err(reason) = note_provider_build_request(FIXTURE_REQUEST_ID, FIXTURE_REQUEST) {
            return denied(
                reason,
                false,
                sha256_bytes(FIXTURE_ANSWER.as_bytes()),
                FIXTURE_ANSWER.len(),
                TEST_FIXTURE_PROVENANCE,
                snapshot(),
            );
        }
    }
    accept_answer(FIXTURE_REQUEST_ID, FIXTURE_ANSWER, TEST_FIXTURE_PROVENANCE)
}

fn accept_answer(request_id: u32, answer: &str, provenance: AnswerProvenance) -> AnswerOutcome {
    let answer_sha256 = sha256_bytes(answer.as_bytes());
    let answer_byte_len = answer.len();
    let request = {
        let mut state = STATE.lock();
        match state.pending_request {
            Some(pending) if pending.request_id == request_id => {
                state.pending_request = None;
                pending
            }
            Some(_) => {
                state.last_reason = Some("agent_answer_request_mismatch");
                return denied(
                    "agent_answer_request_mismatch",
                    false,
                    answer_sha256,
                    answer_byte_len,
                    provenance,
                    state.snapshot(),
                );
            }
            None => {
                state.last_reason = Some("agent_answer_request_not_tracked");
                return denied(
                    "agent_answer_request_not_tracked",
                    false,
                    answer_sha256,
                    answer_byte_len,
                    provenance,
                    state.snapshot(),
                );
            }
        }
    };

    let files = match parse_answer(answer) {
        Ok(files) => files,
        Err(reason) => {
            return reject_consumed(reason, false, answer_sha256, answer_byte_len, provenance)
        }
    };
    let sources: Vec<_> = files
        .iter()
        .map(|file| SourceFile {
            path: &file.path,
            classification: Classification::LocalOnly,
            media_type: file.media_type,
            bytes: &file.content,
        })
        .collect();
    let revision = match project_workspace::commit_agent_answer(request.project_id, &sources) {
        Ok(revision) => revision,
        Err(reason) => {
            return reject_consumed(reason, true, answer_sha256, answer_byte_len, provenance)
        }
    };

    let mut state = STATE.lock();
    state.latest = Some(AcceptedAnswer {
        request,
        revision,
        answer_sha256,
        answer_byte_len,
        answer_origin: provenance.answer_origin,
        provider_trust_positive: provenance.provider_trust_positive,
        test_infrastructure: provenance.test_infrastructure,
    });
    state.phase = LoopPhase::SourceReady;
    state.last_reason = None;
    AnswerOutcome {
        accepted: true,
        reason: "agent_answer_revision_committed",
        answer_sha256,
        answer_byte_len,
        answer_origin: provenance.answer_origin,
        provider_trust_positive: provenance.provider_trust_positive,
        test_infrastructure: provenance.test_infrastructure,
        storage_write_attempted: true,
        writes_persistent_state: true,
        snapshot: state.snapshot(),
    }
}

fn reject_consumed(
    reason: &'static str,
    storage_write_attempted: bool,
    answer_sha256: [u8; 32],
    answer_byte_len: usize,
    provenance: AnswerProvenance,
) -> AnswerOutcome {
    let mut state = STATE.lock();
    state.phase = LoopPhase::Rejected;
    state.last_reason = Some(reason);
    denied(
        reason,
        storage_write_attempted,
        answer_sha256,
        answer_byte_len,
        provenance,
        state.snapshot(),
    )
}

fn denied(
    reason: &'static str,
    storage_write_attempted: bool,
    answer_sha256: [u8; 32],
    answer_byte_len: usize,
    provenance: AnswerProvenance,
    snapshot: Snapshot,
) -> AnswerOutcome {
    AnswerOutcome {
        accepted: false,
        reason,
        answer_sha256,
        answer_byte_len,
        answer_origin: provenance.answer_origin,
        provider_trust_positive: provenance.provider_trust_positive,
        test_infrastructure: provenance.test_infrastructure,
        storage_write_attempted,
        writes_persistent_state: false,
        snapshot,
    }
}
