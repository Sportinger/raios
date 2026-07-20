use alloc::{string::String, vec::Vec};

use raios_core::{
    parse_sha256_ref,
    project_overlay::{DiffEntry, OverlayError, ProjectOverlay},
    project_workspace::{
        Classification, ProjectId, ProjectRevision, SourceFile, MAX_FILE_BYTES,
        MAX_MEDIA_TYPE_BYTES, MAX_PATH_BYTES,
    },
    sha256_bytes,
};
use spin::Mutex;

use crate::{module_candidate_channel::decode_base64_chunk, project_store};

static EDITOR: Mutex<Option<EditSession>> = Mutex::new(None);

pub(crate) struct EditResult {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) project_id: Option<[u8; 16]>,
    pub(crate) base_revision_sha256: Option<[u8; 32]>,
    pub(crate) file_count: usize,
    pub(crate) total_byte_len: usize,
    pub(crate) active_path: Option<String>,
    pub(crate) active_byte_len: usize,
    pub(crate) expected_byte_len: usize,
    pub(crate) diff: Vec<DiffEntry>,
    pub(crate) proposed_revision: Option<ProjectRevision>,
    pub(crate) committed_revision: Option<ProjectRevision>,
    pub(crate) storage_write_attempted: bool,
    pub(crate) writes_persistent_state: bool,
}

impl EditResult {
    fn denied(reason: &'static str) -> Self {
        Self {
            accepted: false,
            reason,
            project_id: None,
            base_revision_sha256: None,
            file_count: 0,
            total_byte_len: 0,
            active_path: None,
            active_byte_len: 0,
            expected_byte_len: 0,
            diff: Vec::new(),
            proposed_revision: None,
            committed_revision: None,
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }

    fn from_session(session: &EditSession, accepted: bool, reason: &'static str) -> Self {
        let diff = session.overlay.diff();
        let proposed_revision = session
            .overlay
            .build_revision()
            .ok()
            .map(|built| built.revision);
        let revision = proposed_revision.as_ref().unwrap_or(&session.base);
        Self {
            accepted,
            reason,
            project_id: Some(session.project_id),
            base_revision_sha256: Some(session.base.revision_sha256),
            file_count: revision.entries.len(),
            total_byte_len: revision.entries.iter().map(|entry| entry.byte_len).sum(),
            active_path: session.active.as_ref().map(|file| file.path.clone()),
            active_byte_len: session
                .active
                .as_ref()
                .map(|file| file.bytes.len())
                .unwrap_or(0),
            expected_byte_len: session
                .active
                .as_ref()
                .map(|file| file.expected_byte_len)
                .unwrap_or(0),
            diff,
            proposed_revision,
            committed_revision: None,
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }
}

struct EditSession {
    project_id: [u8; 16],
    base: ProjectRevision,
    overlay: ProjectOverlay,
    active: Option<PendingFile>,
}

struct PendingFile {
    path: String,
    classification: Classification,
    media_type: String,
    expected_byte_len: usize,
    expected_sha256: [u8; 32],
    bytes: Vec<u8>,
}

pub(crate) fn edit_begin(input: &str) -> EditResult {
    let mut guard = EDITOR.lock();
    *guard = None;
    let Some(value) = one_arg(input, "project.edit_begin") else {
        return EditResult::denied("project_edit_begin_malformed");
    };
    let Some(project_id) = parse_hex_16(value) else {
        return EditResult::denied("project_id_invalid");
    };
    let loaded = match project_store::load(ProjectId::new(project_id)) {
        Ok(Some(loaded)) => loaded,
        Ok(None) => return EditResult::denied("project_not_found"),
        Err(error) => return EditResult::denied(error.reason()),
    };
    let sources: Vec<_> = loaded
        .files
        .iter()
        .map(|file| SourceFile {
            path: &file.entry.path,
            classification: file.entry.classification,
            media_type: &file.entry.media_type,
            bytes: &file.bytes,
        })
        .collect();
    let overlay = match ProjectOverlay::new(
        ProjectId::new(project_id),
        loaded.revision.revision_sha256,
        &loaded.revision,
        &sources,
    ) {
        Ok(overlay) => overlay,
        Err(error) => return EditResult::denied(overlay_reason(error)),
    };
    let session = EditSession {
        project_id,
        base: loaded.revision,
        overlay,
        active: None,
    };
    let result = EditResult::from_session(&session, true, "project_edit_started");
    *guard = Some(session);
    result
}

pub(crate) fn edit_file_begin(input: &str) -> EditResult {
    let mut guard = EDITOR.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_edit_not_active")?;
        if session.active.is_some() {
            return Err("project_edit_file_already_active");
        }
        let args = args::<5>(input, "project.edit_file_begin")?;
        let path = decode_path(args[0])?;
        let classification = match args[1] {
            "public" => Classification::Public,
            "local_only" => Classification::LocalOnly,
            _ => return Err("project_classification_invalid"),
        };
        validate_media_type(args[2])?;
        let expected_byte_len = parse_usize(args[3]).ok_or("project_byte_len_invalid")?;
        if expected_byte_len > MAX_FILE_BYTES {
            return Err("project_file_quota_exceeded");
        }
        let expected_sha256 = parse_sha256_ref(args[4]).ok_or("project_sha256_invalid")?;
        session.active = Some(PendingFile {
            path,
            classification,
            media_type: String::from(args[2]),
            expected_byte_len,
            expected_sha256,
            bytes: Vec::new(),
        });
        Ok(EditResult::from_session(
            session,
            true,
            "project_edit_file_started",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard_error(&mut guard, reason),
    }
}

pub(crate) fn edit_chunk(input: &str) -> EditResult {
    let mut guard = EDITOR.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_edit_not_active")?;
        let encoded = one_arg(input, "project.edit_chunk").ok_or("project_edit_chunk_malformed")?;
        let active = session
            .active
            .as_mut()
            .ok_or("project_edit_file_not_active")?;
        let remaining = active.expected_byte_len.saturating_sub(active.bytes.len());
        if encoded.len() > encoded_len_ceiling(remaining) {
            return Err("project_edit_chunk_overflow");
        }
        let decoded = decode_base64_chunk(encoded).map_err(|_| "project_edit_chunk_invalid")?;
        if decoded.is_empty() || decoded.len() > remaining {
            return Err("project_edit_chunk_overflow");
        }
        active.bytes.extend_from_slice(&decoded);
        Ok(EditResult::from_session(
            session,
            true,
            "project_edit_chunk_accepted",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard_error(&mut guard, reason),
    }
}

pub(crate) fn edit_file_finalize() -> EditResult {
    let mut guard = EDITOR.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_edit_not_active")?;
        let active = session
            .active
            .take()
            .ok_or("project_edit_file_not_active")?;
        if active.bytes.len() != active.expected_byte_len {
            return Err("project_edit_file_length_mismatch");
        }
        if sha256_bytes(&active.bytes) != active.expected_sha256 {
            return Err("project_edit_file_hash_mismatch");
        }
        session
            .overlay
            .replace_or_add(
                ProjectId::new(session.project_id),
                session.base.revision_sha256,
                SourceFile {
                    path: &active.path,
                    classification: active.classification,
                    media_type: &active.media_type,
                    bytes: &active.bytes,
                },
            )
            .map_err(overlay_reason)?;
        Ok(EditResult::from_session(
            session,
            true,
            "project_edit_file_finalized",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard_error(&mut guard, reason),
    }
}

pub(crate) fn edit_delete(input: &str) -> EditResult {
    let mut guard = EDITOR.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_edit_not_active")?;
        if session.active.is_some() {
            return Err("project_edit_file_already_active");
        }
        let value = one_arg(input, "project.edit_delete").ok_or("project_edit_delete_malformed")?;
        let path = decode_path(value)?;
        session
            .overlay
            .delete(
                ProjectId::new(session.project_id),
                session.base.revision_sha256,
                &path,
            )
            .map_err(overlay_reason)?;
        Ok(EditResult::from_session(
            session,
            true,
            "project_edit_delete_accepted",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard_error(&mut guard, reason),
    }
}

pub(crate) fn edit_diff() -> EditResult {
    let guard = EDITOR.lock();
    let Some(session) = guard.as_ref() else {
        return EditResult::denied("project_edit_not_active");
    };
    if session.active.is_some() {
        return EditResult::from_session(session, false, "project_edit_file_incomplete");
    }
    EditResult::from_session(session, true, "project_edit_diff_verified")
}

pub(crate) fn edit_commit() -> EditResult {
    let mut guard = EDITOR.lock();
    let result = (|| {
        let session = guard
            .as_ref()
            .ok_or(("project_edit_not_active", false, false))?;
        if session.active.is_some() {
            return Err(("project_edit_file_incomplete", false, false));
        }
        let built = session
            .overlay
            .build_revision()
            .map_err(|error| (overlay_reason(error), false, false))?;
        let latest = project_store::load(ProjectId::new(session.project_id))
            .map_err(|error| (error.reason(), false, false))?
            .ok_or(("project_not_found", false, false))?;
        if latest.revision.revision_sha256 != session.base.revision_sha256 {
            return Err(("project_edit_base_stale", false, false));
        }
        project_store::commit(&built).map_err(|error| (error.reason(), true, false))?;
        let reloaded = project_store::load(ProjectId::new(session.project_id))
            .map_err(|error| (error.reason(), true, true))?
            .ok_or(("project_edit_commit_reload_missing", true, true))?;
        if reloaded.revision != built.revision {
            return Err(("project_edit_commit_reload_mismatch", true, true));
        }
        Ok((
            session.project_id,
            session.base.revision_sha256,
            built.revision,
        ))
    })();
    let denied = result
        .as_ref()
        .err()
        .and_then(|(reason, attempted, writes)| {
            guard.as_ref().map(|session| {
                let mut result = EditResult::from_session(session, false, reason);
                result.storage_write_attempted = *attempted;
                result.writes_persistent_state = *writes;
                result
            })
        });
    *guard = None;
    match result {
        Ok((project_id, base_revision_sha256, revision)) => EditResult {
            accepted: true,
            reason: "project_edit_revision_committed",
            project_id: Some(project_id),
            base_revision_sha256: Some(base_revision_sha256),
            file_count: revision.entries.len(),
            total_byte_len: revision.entries.iter().map(|entry| entry.byte_len).sum(),
            active_path: None,
            active_byte_len: 0,
            expected_byte_len: 0,
            diff: Vec::new(),
            proposed_revision: None,
            committed_revision: Some(revision),
            storage_write_attempted: true,
            writes_persistent_state: true,
        },
        Err((reason, attempted, writes)) => denied.unwrap_or_else(|| EditResult {
            storage_write_attempted: attempted,
            writes_persistent_state: writes,
            ..EditResult::denied(reason)
        }),
    }
}

pub(crate) fn edit_discard() -> EditResult {
    let mut guard = EDITOR.lock();
    let result = guard
        .as_ref()
        .map(|session| EditResult::from_session(session, true, "project_edit_discarded"))
        .unwrap_or_else(|| EditResult::denied("project_edit_not_active"));
    *guard = None;
    result
}

fn discard_error(guard: &mut Option<EditSession>, reason: &'static str) -> EditResult {
    *guard = None;
    EditResult::denied(reason)
}

fn overlay_reason(error: OverlayError) -> &'static str {
    match error {
        OverlayError::WrongProject => "project_edit_wrong_project",
        OverlayError::WrongBaseRevision => "project_edit_base_stale",
        OverlayError::BaseFilesMismatch => "project_edit_base_files_mismatch",
        OverlayError::DuplicateOperation => "project_edit_path_already_touched",
        OverlayError::NoOp => "project_edit_diff_empty",
        OverlayError::InvalidDelete => "project_edit_delete_missing",
        OverlayError::Workspace(error) => error.reason(),
    }
}

fn one_arg<'a>(input: &'a str, method: &str) -> Option<&'a str> {
    let mut values = input.trim().split_ascii_whitespace();
    if !values.next()?.eq_ignore_ascii_case(method) {
        return None;
    }
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

fn args<'a, const N: usize>(input: &'a str, method: &str) -> Result<[&'a str; N], &'static str> {
    let mut values = input.trim().split_ascii_whitespace();
    if !values
        .next()
        .is_some_and(|value| value.eq_ignore_ascii_case(method))
    {
        return Err("project_edit_command_malformed");
    }
    let mut out = [""; N];
    for value in &mut out {
        *value = values.next().ok_or("project_edit_command_malformed")?;
    }
    if values.next().is_some() {
        return Err("project_edit_command_malformed");
    }
    Ok(out)
}

fn decode_path(value: &str) -> Result<String, &'static str> {
    if value.len() > encoded_len_ceiling(MAX_PATH_BYTES) {
        return Err("project_path_invalid");
    }
    let decoded = decode_base64_chunk(value).map_err(|_| "project_path_invalid")?;
    if decoded.is_empty() || decoded.len() > MAX_PATH_BYTES {
        return Err("project_path_invalid");
    }
    Ok(String::from(
        core::str::from_utf8(&decoded).map_err(|_| "project_path_invalid")?,
    ))
}

fn validate_media_type(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > MAX_MEDIA_TYPE_BYTES
        || !value.contains('/')
        || value.bytes().any(|byte| !(0x21..=0x7e).contains(&byte))
    {
        return Err("project_media_type_invalid");
    }
    Ok(())
}

fn parse_hex_16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[index] = (hex(pair[0])? << 4) | hex(pair[1])?;
    }
    Some(out)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn parse_usize(value: &str) -> Option<usize> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.bytes().try_fold(0usize, |parsed, byte| {
        parsed.checked_mul(10)?.checked_add((byte - b'0') as usize)
    })
}

fn encoded_len_ceiling(decoded_len: usize) -> usize {
    decoded_len.saturating_add(2) / 3 * 4
}
