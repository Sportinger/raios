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
const FEEDBACK_REQUEST_DOMAIN: &[u8] = b"raios.fixture_feedback_request.v1";
const ANSWER_HEADER: &str = "RAIOS_SOURCE_FILES_V1";
const MAX_ANSWER_BYTES: usize = MAX_TOTAL_SOURCE_BYTES * 3;
const FIXTURE_REQUEST_ID: u32 = 0xb2_01_a;
const FIXTURE_REVISION_REQUEST_ID: u32 = 0xb2_02_a;
const FIXTURE_REQUEST: &str = "Build the fixed B2.1a Rust/TOML source fixture.";
const FIXTURE_ANSWER: &str = concat!(
    "RAIOS_SOURCE_FILES_V1\n",
    "file Q2FyZ28udG9tbA== W3BhY2thZ2VdCm5hbWUgPSAiYjIxYS1maXh0dXJlIgp2ZXJzaW9uID0gIjAuMS4wIgplZGl0aW9uID0gIjIwMjEiCg==\n",
    "file c3JjL21haW4ucnM= Zm4gbWFpbigpIHsKICAgIGxldCBtZXNzYWdlID0gImhlbGxvIGZyb20gcmFpT1MiOwogICAgbGV0IF8gPSBtZXNzYWdlOwp9Cg==\n",
    "end",
);
const FIXTURE_REVISION_ANSWER: &str = concat!(
    "RAIOS_SOURCE_FILES_V1\n",
    "file Q2FyZ28ubG9jaw== IyBUaGlzIGZpbGUgaXMgYXV0b21hdGljYWxseSBAZ2VuZXJhdGVkIGJ5IENhcmdvLgp2ZXJzaW9uID0gMwo=\n",
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
    RevisionNeeded,
    VerifiedSource,
    Rejected,
}

impl LoopPhase {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::RequestingSource => "requesting_source",
            Self::SourceReady => "source_ready",
            Self::RevisionNeeded => "revision_needed",
            Self::VerifiedSource => "verified_source",
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
    parent_revision: Option<ProjectRevision>,
    answer_sha256: [u8; 32],
    answer_byte_len: usize,
    answer_origin: &'static str,
    provider_trust_positive: bool,
    test_infrastructure: bool,
    parent_revision_readback_verified: bool,
}

struct BuildLoopState {
    phase: LoopPhase,
    current_request: Option<RequestIdentity>,
    pending_request: Option<RequestIdentity>,
    latest: Option<AcceptedAnswer>,
    verifier_result: Option<VerifierResult>,
    feedback_packet: Option<FeedbackPacket>,
    last_reason: Option<&'static str>,
}

impl BuildLoopState {
    const fn new() -> Self {
        Self {
            phase: LoopPhase::Idle,
            current_request: None,
            pending_request: None,
            latest: None,
            verifier_result: None,
            feedback_packet: None,
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
            parent_revision: self
                .latest
                .as_ref()
                .and_then(|latest| latest.parent_revision.clone()),
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
            verifier_result: self.verifier_result,
            feedback_packet: self.feedback_packet,
            parent_revision_readback_verified: self
                .latest
                .as_ref()
                .is_some_and(|latest| latest.parent_revision_readback_verified),
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
    pub(crate) parent_revision: Option<ProjectRevision>,
    pub(crate) answer_sha256: Option<[u8; 32]>,
    pub(crate) answer_byte_len: Option<usize>,
    pub(crate) answer_origin: Option<&'static str>,
    pub(crate) provider_trust_positive: bool,
    pub(crate) test_infrastructure: bool,
    pub(crate) verifier_result: Option<VerifierResult>,
    pub(crate) feedback_packet: Option<FeedbackPacket>,
    pub(crate) parent_revision_readback_verified: bool,
    pub(crate) last_reason: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VerifierResult {
    pub(crate) check_id: &'static str,
    pub(crate) revision_sha256: [u8; 32],
    pub(crate) tree_sha256: [u8; 32],
    pub(crate) reason: &'static str,
    pub(crate) passed: bool,
}

impl VerifierResult {
    pub(crate) const fn outcome(self) -> &'static str {
        if self.passed {
            "passed"
        } else {
            "failed"
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FeedbackPacket {
    pub(crate) check_id: &'static str,
    pub(crate) revision_sha256: [u8; 32],
    pub(crate) tree_sha256: [u8; 32],
    pub(crate) reason: &'static str,
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
    pub(crate) parent_revision_readback_verified: bool,
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

pub(crate) fn record_verifier_result(
    revision: &ProjectRevision,
    result: crate::project_build::SourceVerifierResult,
) -> Result<VerifierResult, &'static str> {
    if result.project_id != revision.project_id.bytes()
        || result.revision_sha256 != revision.revision_sha256
        || result.tree_sha256 != revision.tree_sha256
        || result.passed != (result.reason == "source_preflight_ok")
    {
        return Err("source_preflight_revision_binding_mismatch");
    }
    let mut state = STATE.lock();
    if !state
        .latest
        .as_ref()
        .is_some_and(|latest| latest.revision == *revision)
    {
        return Err("source_preflight_revision_not_tracked");
    }
    let recorded = VerifierResult {
        check_id: result.check_id,
        revision_sha256: result.revision_sha256,
        tree_sha256: result.tree_sha256,
        reason: result.reason,
        passed: result.passed,
    };
    state.verifier_result = Some(recorded);
    state.phase = if recorded.passed {
        LoopPhase::VerifiedSource
    } else {
        LoopPhase::RevisionNeeded
    };
    state.last_reason = Some(recorded.reason);
    Ok(recorded)
}

pub(crate) fn build_feedback_packet() -> Result<FeedbackPacket, &'static str> {
    let mut state = STATE.lock();
    let result = state
        .verifier_result
        .filter(|result| !result.passed)
        .ok_or("source_preflight_failure_missing")?;
    if !state.latest.as_ref().is_some_and(|latest| {
        latest.revision.revision_sha256 == result.revision_sha256
            && latest.revision.tree_sha256 == result.tree_sha256
    }) {
        return Err("source_preflight_revision_not_current");
    }
    let packet = FeedbackPacket {
        check_id: result.check_id,
        revision_sha256: result.revision_sha256,
        tree_sha256: result.tree_sha256,
        reason: result.reason,
    };
    state.feedback_packet = Some(packet);
    Ok(packet)
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
    accept_answer(
        FIXTURE_REQUEST_ID,
        FIXTURE_ANSWER,
        TEST_FIXTURE_PROVENANCE,
        None,
    )
}

pub(crate) fn accept_revision_answer_fixture() -> AnswerOutcome {
    let parent_revision = {
        let state = STATE.lock();
        let Some(latest) = state.latest.as_ref() else {
            return denied(
                "agent_revision_parent_not_tracked",
                false,
                sha256_bytes(FIXTURE_REVISION_ANSWER.as_bytes()),
                FIXTURE_REVISION_ANSWER.len(),
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        };
        latest.revision.clone()
    };
    accept_revision_answer(&parent_revision, FIXTURE_REVISION_ANSWER)
}

pub(crate) fn accept_revision_answer(
    parent_revision: &ProjectRevision,
    fixture: &str,
) -> AnswerOutcome {
    let answer_sha256 = sha256_bytes(fixture.as_bytes());
    let answer_byte_len = fixture.len();
    let request = {
        let mut state = STATE.lock();
        if fixture != FIXTURE_REVISION_ANSWER {
            return denied(
                "agent_revision_fixture_invalid",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        }
        if state.pending_request.is_some() {
            return denied(
                "agent_build_request_already_pending",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        }
        if !state
            .latest
            .as_ref()
            .is_some_and(|latest| latest.revision == *parent_revision)
        {
            return denied(
                "agent_revision_parent_mismatch",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        }
        let Some(result) = state.verifier_result else {
            return denied(
                "agent_revision_verifier_result_missing",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        };
        if result.passed
            || result.revision_sha256 != parent_revision.revision_sha256
            || result.tree_sha256 != parent_revision.tree_sha256
        {
            return denied(
                "agent_revision_verifier_result_mismatch",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        }
        let Some(packet) = state.feedback_packet else {
            return denied(
                "agent_revision_feedback_packet_missing",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        };
        if packet.check_id != result.check_id
            || packet.revision_sha256 != result.revision_sha256
            || packet.tree_sha256 != result.tree_sha256
            || packet.reason != result.reason
        {
            return denied(
                "agent_revision_feedback_packet_mismatch",
                false,
                answer_sha256,
                answer_byte_len,
                TEST_FIXTURE_PROVENANCE,
                state.snapshot(),
            );
        }
        let request = RequestIdentity {
            request_id: FIXTURE_REVISION_REQUEST_ID,
            project_id: parent_revision.project_id,
            request_sha256: feedback_request_sha256(packet),
        };
        state.current_request = Some(request);
        state.pending_request = Some(request);
        state.phase = LoopPhase::RequestingSource;
        state.last_reason = None;
        request
    };
    accept_answer(
        request.request_id,
        fixture,
        TEST_FIXTURE_PROVENANCE,
        Some(parent_revision),
    )
}

fn accept_answer(
    request_id: u32,
    answer: &str,
    provenance: AnswerProvenance,
    expected_parent: Option<&ProjectRevision>,
) -> AnswerOutcome {
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
    let revision = match project_workspace::commit_agent_answer(
        request.project_id,
        expected_parent.map(|revision| revision.revision_sha256),
        &sources,
    ) {
        Ok(revision) => revision,
        Err(reason) => {
            return reject_consumed(reason, true, answer_sha256, answer_byte_len, provenance)
        }
    };
    let parent_revision_readback_verified = expected_parent
        .map(project_workspace::revision_readback_exact)
        .transpose()
        .is_ok()
        && expected_parent.is_some();

    let mut state = STATE.lock();
    state.latest = Some(AcceptedAnswer {
        request,
        revision,
        parent_revision: expected_parent.cloned(),
        answer_sha256,
        answer_byte_len,
        answer_origin: provenance.answer_origin,
        provider_trust_positive: provenance.provider_trust_positive,
        test_infrastructure: provenance.test_infrastructure,
        parent_revision_readback_verified,
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
        parent_revision_readback_verified,
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
        parent_revision_readback_verified: false,
        snapshot,
    }
}

fn feedback_request_sha256(packet: FeedbackPacket) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(FEEDBACK_REQUEST_DOMAIN);
    hasher.update((packet.check_id.len() as u64).to_le_bytes());
    hasher.update(packet.check_id.as_bytes());
    hasher.update(packet.revision_sha256);
    hasher.update(packet.tree_sha256);
    hasher.update((packet.reason.len() as u64).to_le_bytes());
    hasher.update(packet.reason.as_bytes());
    hasher.finalize().into()
}
