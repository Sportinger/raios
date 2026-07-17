use alloc::{string::String, vec::Vec};

use raios_core::{
    parse_sha256_ref,
    project_workspace::{
        build_agent_answer, build_local_import, Classification, ProjectId, ProjectRevision,
        SourceFile, MAX_FILES, MAX_FILE_BYTES, MAX_MEDIA_TYPE_BYTES, MAX_PATH_BYTES,
        MAX_TOTAL_SOURCE_BYTES,
    },
    sha256_bytes,
};
use spin::Mutex;

use crate::{module_candidate_channel::decode_base64_chunk, project_store};

static IMPORT: Mutex<Option<ImportSession>> = Mutex::new(None);

#[derive(Clone, Debug)]
pub(crate) struct OperationResult {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) project_id: Option<[u8; 16]>,
    pub(crate) file_count: usize,
    pub(crate) total_byte_len: usize,
    pub(crate) active_path: Option<String>,
    pub(crate) active_byte_len: usize,
    pub(crate) expected_byte_len: usize,
    pub(crate) revision: Option<ProjectRevision>,
    pub(crate) storage_write_attempted: bool,
    pub(crate) writes_persistent_state: bool,
}

impl OperationResult {
    fn denied(reason: &'static str) -> Self {
        Self {
            accepted: false,
            reason,
            project_id: None,
            file_count: 0,
            total_byte_len: 0,
            active_path: None,
            active_byte_len: 0,
            expected_byte_len: 0,
            revision: None,
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }

    fn from_session(session: &ImportSession, reason: &'static str) -> Self {
        Self {
            accepted: true,
            reason,
            project_id: Some(session.project_id),
            file_count: session.files.len(),
            total_byte_len: session.total_byte_len(),
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
            revision: None,
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }
}

struct ImportSession {
    project_id: [u8; 16],
    files: Vec<OwnedFile>,
    active: Option<PendingFile>,
}

impl ImportSession {
    fn total_byte_len(&self) -> usize {
        self.files.iter().map(|file| file.bytes.len()).sum()
    }
}

struct OwnedFile {
    path: String,
    classification: Classification,
    media_type: String,
    bytes: Vec<u8>,
}

struct PendingFile {
    path: String,
    classification: Classification,
    media_type: String,
    expected_byte_len: usize,
    expected_sha256: [u8; 32],
    bytes: Vec<u8>,
}

pub(crate) fn import_begin(input: &str) -> OperationResult {
    let mut guard = IMPORT.lock();
    *guard = None;
    let Some(value) = one_arg(input, "project.import_begin") else {
        return OperationResult::denied("project_import_begin_malformed");
    };
    let Some(project_id) = parse_hex_16(value) else {
        return OperationResult::denied("project_id_invalid");
    };
    let session = ImportSession {
        project_id,
        files: Vec::new(),
        active: None,
    };
    let result = OperationResult::from_session(&session, "project_import_started");
    *guard = Some(session);
    result
}

pub(crate) fn import_file_begin(input: &str) -> OperationResult {
    let mut guard = IMPORT.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_import_not_active")?;
        if session.active.is_some() {
            return Err("project_import_file_already_active");
        }
        if session.files.len() >= MAX_FILES {
            return Err("project_file_quota_exceeded");
        }
        let args = args::<5>(input, "project.import_file_begin")?;
        if args[0].len() > encoded_len_ceiling(MAX_PATH_BYTES) {
            return Err("project_path_too_long");
        }
        let path_bytes = decode_base64_chunk(args[0]).map_err(|_| "project_path_base64_invalid")?;
        let path = String::from(
            core::str::from_utf8(&path_bytes).map_err(|_| "project_path_utf8_invalid")?,
        );
        validate_path(&path)?;
        if session.files.iter().any(|file| file.path == path) {
            return Err("project_path_duplicate");
        }
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
        if session
            .total_byte_len()
            .checked_add(expected_byte_len)
            .filter(|total| *total <= MAX_TOTAL_SOURCE_BYTES)
            .is_none()
        {
            return Err("project_total_quota_exceeded");
        }
        let expected_sha256 = parse_sha256_ref(args[4]).ok_or("project_sha256_invalid")?;
        session.active = Some(PendingFile {
            path,
            classification,
            media_type: args[2].into(),
            expected_byte_len,
            expected_sha256,
            bytes: Vec::new(),
        });
        Ok(OperationResult::from_session(
            session,
            "project_import_file_started",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard(&mut guard, reason),
    }
}

pub(crate) fn import_chunk(input: &str) -> OperationResult {
    let mut guard = IMPORT.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_import_not_active")?;
        let encoded = one_arg(input, "project.import_chunk").ok_or("project_chunk_malformed")?;
        let active = session
            .active
            .as_mut()
            .ok_or("project_import_file_not_active")?;
        let remaining = active.expected_byte_len.saturating_sub(active.bytes.len());
        if encoded.len() > encoded_len_ceiling(remaining) {
            return Err("project_chunk_overflow");
        }
        let decoded = decode_base64_chunk(encoded).map_err(|_| "project_chunk_base64_invalid")?;
        if decoded.is_empty() || decoded.len() > remaining {
            return Err("project_chunk_overflow");
        }
        active.bytes.extend_from_slice(&decoded);
        Ok(OperationResult::from_session(
            session,
            "project_import_chunk_accepted",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard(&mut guard, reason),
    }
}

pub(crate) fn import_file_finalize() -> OperationResult {
    let mut guard = IMPORT.lock();
    let result = (|| {
        let session = guard.as_mut().ok_or("project_import_not_active")?;
        let active = session
            .active
            .take()
            .ok_or("project_import_file_not_active")?;
        if active.bytes.len() != active.expected_byte_len {
            return Err("project_file_length_mismatch");
        }
        if sha256_bytes(&active.bytes) != active.expected_sha256 {
            return Err("project_file_hash_mismatch");
        }
        session.files.push(OwnedFile {
            path: active.path,
            classification: active.classification,
            media_type: active.media_type,
            bytes: active.bytes,
        });
        Ok(OperationResult::from_session(
            session,
            "project_import_file_finalized",
        ))
    })();
    match result {
        Ok(result) => result,
        Err(reason) => discard(&mut guard, reason),
    }
}

pub(crate) fn import_commit() -> OperationResult {
    let mut guard = IMPORT.lock();
    let result = (|| {
        let session = guard.as_ref().ok_or(("project_import_not_active", false))?;
        if session.active.is_some() || session.files.is_empty() {
            return Err(("project_import_incomplete", false));
        }
        let files: Vec<_> = session
            .files
            .iter()
            .map(|file| SourceFile {
                path: &file.path,
                classification: file.classification,
                media_type: &file.media_type,
                bytes: &file.bytes,
            })
            .collect();
        let project_id = ProjectId::new(session.project_id);
        let parent = project_store::inspect(project_id)
            .map_err(|error| (error.reason(), false))?
            .map(|revision| revision.revision_sha256);
        let built = build_local_import(project_id, parent, &files)
            .map_err(|error| (error.reason(), false))?;
        project_store::commit(&built).map_err(|error| (error.reason(), true))?;
        Ok((
            session.project_id,
            session.files.len(),
            session.total_byte_len(),
            built,
        ))
    })();
    *guard = None;
    match result {
        Ok((project_id, file_count, total_byte_len, built)) => OperationResult {
            accepted: true,
            reason: "project_revision_committed",
            project_id: Some(project_id),
            file_count,
            total_byte_len,
            active_path: None,
            active_byte_len: 0,
            expected_byte_len: 0,
            revision: Some(built.revision),
            storage_write_attempted: true,
            writes_persistent_state: true,
        },
        Err((reason, storage_write_attempted)) => OperationResult {
            storage_write_attempted,
            ..OperationResult::denied(reason)
        },
    }
}

pub(crate) fn commit_agent_answer(
    project_id: ProjectId,
    parent_revision_sha256: Option<[u8; 32]>,
    files: &[SourceFile<'_>],
) -> Result<ProjectRevision, &'static str> {
    let current_parent = project_store::inspect(project_id)
        .map_err(|error| error.reason())?
        .map(|revision| revision.revision_sha256);
    if current_parent != parent_revision_sha256 {
        return Err("agent_answer_parent_revision_mismatch");
    }
    let built = build_agent_answer(project_id, parent_revision_sha256, files)
        .map_err(|error| error.reason())?;
    project_store::commit(&built).map_err(|error| error.reason())?;
    Ok(built.revision)
}

pub(crate) fn revision_readback_exact(revision: &ProjectRevision) -> Result<(), &'static str> {
    let loaded = project_store::load_revision(revision.project_id, revision.revision_sha256)
        .map_err(|error| error.reason())?
        .ok_or("agent_answer_parent_revision_not_readable")?;
    if loaded.revision != *revision {
        return Err("agent_answer_parent_revision_mismatch");
    }
    Ok(())
}

pub(crate) fn inspect(input: &str) -> Result<Option<ProjectRevision>, &'static str> {
    let value = one_arg(input, "project.inspect").ok_or("project_inspect_malformed")?;
    let project_id = parse_hex_16(value).ok_or("project_id_invalid")?;
    project_store::inspect(ProjectId::new(project_id)).map_err(|error| error.reason())
}

fn discard(guard: &mut Option<ImportSession>, reason: &'static str) -> OperationResult {
    *guard = None;
    OperationResult::denied(reason)
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
        return Err("project_command_malformed");
    }
    let mut out = [""; N];
    for value in &mut out {
        *value = values.next().ok_or("project_command_malformed")?;
    }
    if values.next().is_some() {
        return Err("project_command_malformed");
    }
    Ok(out)
}

fn parse_hex_16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (idx, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[idx] = (hex(pair[0])? << 4) | hex(pair[1])?;
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

fn validate_path(path: &str) -> Result<(), &'static str> {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
    {
        return Err("project_path_invalid");
    }
    Ok(())
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
