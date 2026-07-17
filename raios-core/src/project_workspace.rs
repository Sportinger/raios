use alloc::{string::String, vec::Vec};

use crate::{sha256_bytes, structured_store::RecordKey};

pub const MAX_FILES: usize = 32;
pub const MAX_PATH_BYTES: usize = 240;
pub const MAX_MEDIA_TYPE_BYTES: usize = 64;
pub const MAX_FILE_BYTES: usize = 32 * 1024;
pub const MAX_TOTAL_SOURCE_BYTES: usize = 48 * 1024;
pub const LOCAL_IMPORT_ACTION: &str = "owner_local_import";
pub const AGENT_OVERLAY_ACTION: &str = "agent_overlay_commit";
pub const AGENT_ANSWER_ACTION: &str = "agent_answer";
pub const LOCAL_IMPORT_TIME_BASIS: &str = "none_deterministic";

pub const WORKSPACE_BLOB_NAMESPACE: [u8; 16] = *b"project.blob.v1!";
pub const WORKSPACE_REVISION_NAMESPACE: [u8; 16] = *b"project.rev.v1!!";

const BLOB_MAGIC: &[u8; 8] = b"RAIOSBL1";
const TREE_MAGIC: &[u8; 8] = b"RAIOSTR1";
const REVISION_MAGIC: &[u8; 8] = b"RAIOSRV1";
const REVISION_HASH_MAGIC: &[u8; 9] = b"RAIOSREV1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectId([u8; 16]);

impl ProjectId {
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

pub const fn agent_project_id(system_derived_sha256: [u8; 32]) -> ProjectId {
    let mut bytes = [0u8; 16];
    let mut index = 0;
    while index < bytes.len() {
        bytes[index] = system_derived_sha256[index];
        index += 1;
    }
    ProjectId::new(bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Classification {
    Public,
    LocalOnly,
}

impl Classification {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly => "local_only",
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Public => 1,
            Self::LocalOnly => 2,
        }
    }

    const fn from_code(code: u8) -> Result<Self, WorkspaceError> {
        match code {
            1 => Ok(Self::Public),
            2 => Ok(Self::LocalOnly),
            _ => Err(WorkspaceError::Malformed),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionAction {
    OwnerLocalImport,
    AgentOverlayCommit,
    AgentAnswer,
}

impl RevisionAction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OwnerLocalImport => LOCAL_IMPORT_ACTION,
            Self::AgentOverlayCommit => AGENT_OVERLAY_ACTION,
            Self::AgentAnswer => AGENT_ANSWER_ACTION,
        }
    }

    fn from_label(label: &str) -> Result<Self, WorkspaceError> {
        match label {
            LOCAL_IMPORT_ACTION => Ok(Self::OwnerLocalImport),
            AGENT_OVERLAY_ACTION => Ok(Self::AgentOverlayCommit),
            AGENT_ANSWER_ACTION => Ok(Self::AgentAnswer),
            _ => Err(WorkspaceError::Malformed),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SourceFile<'a> {
    pub path: &'a str,
    pub classification: Classification,
    pub media_type: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeEntry {
    pub path: String,
    pub classification: Classification,
    pub media_type: String,
    pub byte_len: usize,
    pub blob_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRevision {
    pub project_id: ProjectId,
    pub parent_revision_sha256: Option<[u8; 32]>,
    pub action: RevisionAction,
    pub tree_sha256: [u8; 32],
    pub revision_sha256: [u8; 32],
    pub entries: Vec<TreeEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobRecord {
    pub sha256: [u8; 32],
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedBlobRecord {
    pub sha256: [u8; 32],
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltRevision {
    pub revision: ProjectRevision,
    pub blobs: Vec<BlobRecord>,
    pub manifest_payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceError {
    NoFiles,
    TooManyFiles,
    InvalidPath,
    UnsupportedPathType,
    InvalidUtf8,
    DuplicatePath,
    InvalidMediaType,
    FileTooLarge,
    TotalTooLarge,
    Malformed,
    HashMismatch,
    LimitExceeded,
}

impl WorkspaceError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::NoFiles => "project_no_files",
            Self::TooManyFiles => "project_file_quota_exceeded",
            Self::InvalidPath => "project_path_invalid",
            Self::UnsupportedPathType => "project_source_path_type_unsupported",
            Self::InvalidUtf8 => "project_source_utf8_invalid",
            Self::DuplicatePath => "project_path_collision",
            Self::InvalidMediaType => "project_media_type_invalid",
            Self::FileTooLarge => "project_file_quota_exceeded",
            Self::TotalTooLarge => "project_total_quota_exceeded",
            Self::Malformed => "project_record_malformed",
            Self::HashMismatch => "project_record_hash_mismatch",
            Self::LimitExceeded => "project_record_limit_exceeded",
        }
    }
}

pub fn blob_record_key(sha256: [u8; 32]) -> RecordKey {
    RecordKey {
        namespace: WORKSPACE_BLOB_NAMESPACE,
        record_id: hash_prefix(sha256),
    }
}

pub fn revision_record_key(sha256: [u8; 32]) -> RecordKey {
    RecordKey {
        namespace: WORKSPACE_REVISION_NAMESPACE,
        record_id: hash_prefix(sha256),
    }
}

pub fn is_blob_record_key(key: RecordKey) -> bool {
    key.namespace == WORKSPACE_BLOB_NAMESPACE
}

pub fn is_revision_record_key(key: RecordKey) -> bool {
    key.namespace == WORKSPACE_REVISION_NAMESPACE
}

pub fn build_local_import(
    project_id: ProjectId,
    parent_revision_sha256: Option<[u8; 32]>,
    files: &[SourceFile<'_>],
) -> Result<BuiltRevision, WorkspaceError> {
    build_revision(
        project_id,
        parent_revision_sha256,
        RevisionAction::OwnerLocalImport,
        files,
    )
}

pub fn build_agent_edit(
    project_id: ProjectId,
    parent_revision_sha256: [u8; 32],
    files: &[SourceFile<'_>],
) -> Result<BuiltRevision, WorkspaceError> {
    build_revision(
        project_id,
        Some(parent_revision_sha256),
        RevisionAction::AgentOverlayCommit,
        files,
    )
}

pub fn build_agent_answer(
    project_id: ProjectId,
    parent_revision_sha256: Option<[u8; 32]>,
    files: &[SourceFile<'_>],
) -> Result<BuiltRevision, WorkspaceError> {
    build_revision(
        project_id,
        parent_revision_sha256,
        RevisionAction::AgentAnswer,
        files,
    )
}

pub fn agent_source_media_type(path: &str) -> Result<&'static str, WorkspaceError> {
    validate_path(path)?;
    if path.ends_with(".rs") {
        Ok("text/rust")
    } else if path.ends_with(".toml") {
        Ok("text/toml")
    } else {
        Err(WorkspaceError::UnsupportedPathType)
    }
}

fn build_revision(
    project_id: ProjectId,
    parent_revision_sha256: Option<[u8; 32]>,
    action: RevisionAction,
    files: &[SourceFile<'_>],
) -> Result<BuiltRevision, WorkspaceError> {
    if files.is_empty() {
        return Err(WorkspaceError::NoFiles);
    }
    if files.len() > MAX_FILES {
        return Err(WorkspaceError::TooManyFiles);
    }

    let mut entries = Vec::with_capacity(files.len());
    let mut blobs = Vec::new();
    let mut total = 0usize;
    for file in files {
        if file.bytes.len() > MAX_FILE_BYTES {
            return Err(WorkspaceError::FileTooLarge);
        }
        let (classification, media_type) = if action == RevisionAction::AgentAnswer {
            core::str::from_utf8(file.bytes).map_err(|_| WorkspaceError::InvalidUtf8)?;
            (
                Classification::LocalOnly,
                agent_source_media_type(file.path)?,
            )
        } else {
            validate_path(file.path)?;
            validate_media_type(file.media_type)?;
            (file.classification, file.media_type)
        };
        total = total
            .checked_add(file.bytes.len())
            .ok_or(WorkspaceError::TotalTooLarge)?;
        if total > MAX_TOTAL_SOURCE_BYTES {
            return Err(WorkspaceError::TotalTooLarge);
        }
        let blob_sha256 = sha256_bytes(file.bytes);
        entries.push(TreeEntry {
            path: String::from(file.path),
            classification,
            media_type: String::from(media_type),
            byte_len: file.bytes.len(),
            blob_sha256,
        });
        if !blobs
            .iter()
            .any(|blob: &BlobRecord| blob.sha256 == blob_sha256)
        {
            blobs.push(BlobRecord {
                sha256: blob_sha256,
                payload: encode_blob_record(file.bytes)?,
            });
        }
    }

    entries.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    if has_path_collision(&entries) {
        return Err(WorkspaceError::DuplicatePath);
    }
    let tree_sha256 = tree_hash(&entries)?;
    let revision_sha256 = revision_hash(
        project_id,
        parent_revision_sha256,
        tree_sha256,
        action,
        LOCAL_IMPORT_TIME_BASIS,
    )?;
    let revision = ProjectRevision {
        project_id,
        parent_revision_sha256,
        action,
        tree_sha256,
        revision_sha256,
        entries,
    };
    let manifest_payload = encode_revision_manifest(&revision)?;
    Ok(BuiltRevision {
        revision,
        blobs,
        manifest_payload,
    })
}

pub fn encode_blob_record(bytes: &[u8]) -> Result<Vec<u8>, WorkspaceError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(WorkspaceError::FileTooLarge);
    }
    let len = u32::try_from(bytes.len()).map_err(|_| WorkspaceError::LimitExceeded)?;
    let mut out = Vec::with_capacity(8 + 32 + 4 + bytes.len());
    out.extend_from_slice(BLOB_MAGIC);
    out.extend_from_slice(&sha256_bytes(bytes));
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(out)
}

pub fn decode_blob_record(payload: &[u8]) -> Result<DecodedBlobRecord, WorkspaceError> {
    if payload.len() < 44 || &payload[..8] != BLOB_MAGIC {
        return Err(WorkspaceError::Malformed);
    }
    let mut recorded = [0u8; 32];
    recorded.copy_from_slice(&payload[8..40]);
    let len = u32::from_le_bytes(
        payload[40..44]
            .try_into()
            .map_err(|_| WorkspaceError::Malformed)?,
    ) as usize;
    if len > MAX_FILE_BYTES || payload.len() != 44 + len {
        return Err(WorkspaceError::Malformed);
    }
    let bytes = payload[44..].to_vec();
    if sha256_bytes(&bytes) != recorded {
        return Err(WorkspaceError::HashMismatch);
    }
    Ok(DecodedBlobRecord {
        sha256: recorded,
        bytes,
    })
}

pub fn encode_revision_manifest(revision: &ProjectRevision) -> Result<Vec<u8>, WorkspaceError> {
    validate_entries(&revision.entries)?;
    if tree_hash(&revision.entries)? != revision.tree_sha256
        || revision_hash(
            revision.project_id,
            revision.parent_revision_sha256,
            revision.tree_sha256,
            revision.action,
            LOCAL_IMPORT_TIME_BASIS,
        )? != revision.revision_sha256
    {
        return Err(WorkspaceError::HashMismatch);
    }
    let mut out = Vec::new();
    out.extend_from_slice(REVISION_MAGIC);
    out.extend_from_slice(&revision.project_id.bytes());
    put_optional_hash(&mut out, revision.parent_revision_sha256);
    out.extend_from_slice(&revision.tree_sha256);
    out.extend_from_slice(&revision.revision_sha256);
    put_string(&mut out, revision.action.label())?;
    put_string(&mut out, LOCAL_IMPORT_TIME_BASIS)?;
    put_u16(&mut out, revision.entries.len())?;
    encode_entries(&mut out, &revision.entries)?;
    if out.len() > crate::structured_store::MAX_RECORD_PAYLOAD_LEN {
        return Err(WorkspaceError::LimitExceeded);
    }
    Ok(out)
}

pub fn decode_revision_manifest(payload: &[u8]) -> Result<ProjectRevision, WorkspaceError> {
    if payload.len() > crate::structured_store::MAX_RECORD_PAYLOAD_LEN {
        return Err(WorkspaceError::LimitExceeded);
    }
    let mut cursor = Cursor::new(payload);
    if cursor.take(8)? != REVISION_MAGIC {
        return Err(WorkspaceError::Malformed);
    }
    let mut project = [0u8; 16];
    project.copy_from_slice(cursor.take(16)?);
    let parent_revision_sha256 = cursor.optional_hash()?;
    let tree_sha256 = cursor.hash()?;
    let revision_sha256 = cursor.hash()?;
    let action = RevisionAction::from_label(cursor.string(MAX_PATH_BYTES)?)?;
    if cursor.string(MAX_PATH_BYTES)? != LOCAL_IMPORT_TIME_BASIS {
        return Err(WorkspaceError::Malformed);
    }
    let count = cursor.u16()? as usize;
    if count == 0 || count > MAX_FILES {
        return Err(WorkspaceError::LimitExceeded);
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let path = String::from(cursor.string(MAX_PATH_BYTES)?);
        let classification = Classification::from_code(cursor.u8()?)?;
        let media_type = String::from(cursor.string(MAX_MEDIA_TYPE_BYTES)?);
        let byte_len = cursor.u32()? as usize;
        let blob_sha256 = cursor.hash()?;
        entries.push(TreeEntry {
            path,
            classification,
            media_type,
            byte_len,
            blob_sha256,
        });
    }
    if !cursor.finished() {
        return Err(WorkspaceError::Malformed);
    }
    validate_entries(&entries)?;
    let project_id = ProjectId::new(project);
    if tree_hash(&entries)? != tree_sha256
        || revision_hash(
            project_id,
            parent_revision_sha256,
            tree_sha256,
            action,
            LOCAL_IMPORT_TIME_BASIS,
        )? != revision_sha256
    {
        return Err(WorkspaceError::HashMismatch);
    }
    Ok(ProjectRevision {
        project_id,
        parent_revision_sha256,
        action,
        tree_sha256,
        revision_sha256,
        entries,
    })
}

fn validate_entries(entries: &[TreeEntry]) -> Result<(), WorkspaceError> {
    if entries.is_empty() || entries.len() > MAX_FILES {
        return Err(WorkspaceError::LimitExceeded);
    }
    let mut total = 0usize;
    let mut previous: Option<&str> = None;
    for entry in entries {
        validate_path(&entry.path)?;
        validate_media_type(&entry.media_type)?;
        if entry.byte_len > MAX_FILE_BYTES {
            return Err(WorkspaceError::FileTooLarge);
        }
        total = total
            .checked_add(entry.byte_len)
            .ok_or(WorkspaceError::TotalTooLarge)?;
        if total > MAX_TOTAL_SOURCE_BYTES {
            return Err(WorkspaceError::TotalTooLarge);
        }
        if let Some(prior) = previous {
            if prior.as_bytes() >= entry.path.as_bytes() {
                return Err(WorkspaceError::DuplicatePath);
            }
        }
        previous = Some(&entry.path);
    }
    if has_path_collision(entries) {
        return Err(WorkspaceError::DuplicatePath);
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), WorkspaceError> {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
    {
        return Err(WorkspaceError::InvalidPath);
    }
    for segment in path.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.ends_with('.')
            || segment.ends_with(' ')
            || is_device_name(segment)
        {
            return Err(WorkspaceError::InvalidPath);
        }
    }
    Ok(())
}

fn validate_media_type(value: &str) -> Result<(), WorkspaceError> {
    if value.is_empty()
        || value.len() > MAX_MEDIA_TYPE_BYTES
        || !value.contains('/')
        || value.bytes().any(|byte| !(0x21..=0x7e).contains(&byte))
    {
        return Err(WorkspaceError::InvalidMediaType);
    }
    Ok(())
}

fn is_device_name(segment: &str) -> bool {
    let stem = segment.split('.').next().unwrap_or(segment);
    let mut upper = [0u8; 4];
    if stem.len() > upper.len() || !stem.is_ascii() {
        return false;
    }
    for (index, byte) in stem.bytes().enumerate() {
        upper[index] = byte.to_ascii_uppercase();
    }
    matches!(&upper[..stem.len()], b"CON" | b"PRN" | b"AUX" | b"NUL")
        || (stem.len() == 4
            && matches!(&upper[..3], b"COM" | b"LPT")
            && matches!(upper[3], b'1'..=b'9'))
}

fn paths_alias(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

fn has_path_collision(entries: &[TreeEntry]) -> bool {
    for left in 0..entries.len() {
        for right in left + 1..entries.len() {
            if paths_alias(&entries[left].path, &entries[right].path) {
                return true;
            }
        }
    }
    false
}

fn tree_hash(entries: &[TreeEntry]) -> Result<[u8; 32], WorkspaceError> {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(TREE_MAGIC);
    put_u16(&mut canonical, entries.len())?;
    encode_entries(&mut canonical, entries)?;
    Ok(sha256_bytes(&canonical))
}

fn revision_hash(
    project_id: ProjectId,
    parent: Option<[u8; 32]>,
    tree: [u8; 32],
    action: RevisionAction,
    time_basis: &str,
) -> Result<[u8; 32], WorkspaceError> {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(REVISION_HASH_MAGIC);
    canonical.extend_from_slice(&project_id.bytes());
    put_optional_hash(&mut canonical, parent);
    canonical.extend_from_slice(&tree);
    put_string(&mut canonical, action.label())?;
    put_string(&mut canonical, time_basis)?;
    Ok(sha256_bytes(&canonical))
}

fn encode_entries(out: &mut Vec<u8>, entries: &[TreeEntry]) -> Result<(), WorkspaceError> {
    for entry in entries {
        put_string(out, &entry.path)?;
        out.push(entry.classification.code());
        put_string(out, &entry.media_type)?;
        let len = u32::try_from(entry.byte_len).map_err(|_| WorkspaceError::LimitExceeded)?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&entry.blob_sha256);
    }
    Ok(())
}

fn put_optional_hash(out: &mut Vec<u8>, value: Option<[u8; 32]>) {
    match value {
        Some(hash) => {
            out.push(1);
            out.extend_from_slice(&hash);
        }
        None => {
            out.push(0);
            out.extend_from_slice(&[0u8; 32]);
        }
    }
}

fn put_u16(out: &mut Vec<u8>, value: usize) -> Result<(), WorkspaceError> {
    let value = u16::try_from(value).map_err(|_| WorkspaceError::LimitExceeded)?;
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), WorkspaceError> {
    put_u16(out, value.len())?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

const fn hash_prefix(hash: [u8; 32]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut index = 0;
    while index < out.len() {
        out[index] = hash[index];
        index += 1;
    }
    out
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], WorkspaceError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(WorkspaceError::Malformed)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(WorkspaceError::Malformed)?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, WorkspaceError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, WorkspaceError> {
        Ok(u16::from_le_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| WorkspaceError::Malformed)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, WorkspaceError> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| WorkspaceError::Malformed)?,
        ))
    }

    fn hash(&mut self) -> Result<[u8; 32], WorkspaceError> {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.take(32)?);
        Ok(out)
    }

    fn optional_hash(&mut self) -> Result<Option<[u8; 32]>, WorkspaceError> {
        let present = self.u8()?;
        let hash = self.hash()?;
        match present {
            0 if hash == [0u8; 32] => Ok(None),
            1 => Ok(Some(hash)),
            _ => Err(WorkspaceError::Malformed),
        }
    }

    fn string(&mut self, max_len: usize) -> Result<&'a str, WorkspaceError> {
        let len = self.u16()? as usize;
        if len > max_len {
            return Err(WorkspaceError::LimitExceeded);
        }
        core::str::from_utf8(self.take(len)?).map_err(|_| WorkspaceError::Malformed)
    }

    fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT: ProjectId = ProjectId::new([7u8; 16]);

    fn files<'a>() -> [SourceFile<'a>; 2] {
        [
            SourceFile {
                path: "src/main.rs",
                classification: Classification::LocalOnly,
                media_type: "text/rust",
                bytes: b"fn main() {}\n",
            },
            SourceFile {
                path: "Cargo.toml",
                classification: Classification::Public,
                media_type: "text/toml",
                bytes: b"[package]\nname='demo'\n",
            },
        ]
    }

    fn agent_files<'a>() -> [SourceFile<'a>; 2] {
        [
            SourceFile {
                path: "src/main.rs",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: b"fn main() {}\n",
            },
            SourceFile {
                path: "Cargo.toml",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: b"[package]\nname='agent-demo'\n",
            },
        ]
    }

    #[test]
    fn local_import_is_sorted_deterministic_and_round_trips() {
        let built = build_local_import(PROJECT, None, &files()).unwrap();
        assert_eq!(built.revision.action, RevisionAction::OwnerLocalImport);
        assert_eq!(built.revision.entries[0].path, "Cargo.toml");
        assert_eq!(built.revision.entries[1].path, "src/main.rs");
        assert_eq!(
            decode_revision_manifest(&built.manifest_payload).unwrap(),
            built.revision
        );
        let rebuilt = build_local_import(PROJECT, None, &files()).unwrap();
        assert_eq!(
            rebuilt.revision.revision_sha256,
            built.revision.revision_sha256
        );
        for blob in built.blobs {
            let decoded = decode_blob_record(&blob.payload).unwrap();
            assert_eq!(decoded.sha256, blob.sha256);
        }
    }

    #[test]
    fn agent_answer_is_two_file_local_only_and_hash_stable() {
        let mut system_hash = [0u8; 32];
        for (index, byte) in system_hash.iter_mut().enumerate() {
            *byte = index as u8;
        }
        let project_id = agent_project_id(system_hash);
        let mut expected_project_id = [0u8; 16];
        expected_project_id.copy_from_slice(&system_hash[..16]);
        assert_eq!(project_id.bytes(), expected_project_id);

        let sources = agent_files();
        let built = build_agent_answer(project_id, None, &sources).unwrap();
        assert_eq!(built.revision.action, RevisionAction::AgentAnswer);
        assert_eq!(built.revision.entries[0].path, "Cargo.toml");
        assert_eq!(built.revision.entries[0].media_type, "text/toml");
        assert_eq!(built.revision.entries[1].path, "src/main.rs");
        assert_eq!(built.revision.entries[1].media_type, "text/rust");
        assert!(built
            .revision
            .entries
            .iter()
            .all(|entry| entry.classification == Classification::LocalOnly));
        for entry in &built.revision.entries {
            let source = sources.iter().find(|file| file.path == entry.path).unwrap();
            assert_eq!(entry.byte_len, source.bytes.len());
            assert_eq!(entry.blob_sha256, sha256_bytes(source.bytes));
        }
        assert_eq!(
            tree_hash(&built.revision.entries).unwrap(),
            built.revision.tree_sha256
        );
        assert_eq!(
            revision_hash(
                project_id,
                None,
                built.revision.tree_sha256,
                RevisionAction::AgentAnswer,
                LOCAL_IMPORT_TIME_BASIS,
            )
            .unwrap(),
            built.revision.revision_sha256
        );
        assert_eq!(
            decode_revision_manifest(&built.manifest_payload).unwrap(),
            built.revision
        );
        for blob in &built.blobs {
            let decoded = decode_blob_record(&blob.payload).unwrap();
            assert_eq!(decoded.sha256, blob.sha256);
            assert_eq!(decoded.sha256, sha256_bytes(&decoded.bytes));
        }

        let rebuilt = build_agent_answer(project_id, None, &sources).unwrap();
        assert_eq!(rebuilt, built);
    }

    #[test]
    fn agent_answer_rejects_paths_aliases_duplicates_utf8_and_quotas() {
        let mut invalid_path = agent_files();
        invalid_path[0].path = "../main.rs";
        assert_eq!(
            build_agent_answer(PROJECT, None, &invalid_path),
            Err(WorkspaceError::InvalidPath)
        );

        let mut unknown_path_type = agent_files();
        unknown_path_type[0].path = "README.md";
        assert_eq!(
            build_agent_answer(PROJECT, None, &unknown_path_type),
            Err(WorkspaceError::UnsupportedPathType)
        );

        let invalid_utf8 = [SourceFile {
            path: "src/main.rs",
            classification: Classification::Public,
            media_type: "provider/ignored",
            bytes: &[0xff],
        }];
        assert_eq!(
            build_agent_answer(PROJECT, None, &invalid_utf8),
            Err(WorkspaceError::InvalidUtf8)
        );

        let aliases = [
            SourceFile {
                path: "src/Main.rs",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: b"fn first() {}\n",
            },
            SourceFile {
                path: "src/main.rs",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: b"fn second() {}\n",
            },
        ];
        assert_eq!(
            build_agent_answer(PROJECT, None, &aliases),
            Err(WorkspaceError::DuplicatePath)
        );

        let duplicates = [agent_files()[0], agent_files()[0]];
        assert_eq!(
            build_agent_answer(PROJECT, None, &duplicates),
            Err(WorkspaceError::DuplicatePath)
        );

        let oversized = vec![b'a'; MAX_FILE_BYTES + 1];
        let oversized_file = [SourceFile {
            path: "src/main.rs",
            classification: Classification::Public,
            media_type: "provider/ignored",
            bytes: &oversized,
        }];
        assert_eq!(
            build_agent_answer(PROJECT, None, &oversized_file),
            Err(WorkspaceError::FileTooLarge)
        );

        let half_plus_one = vec![b'a'; MAX_TOTAL_SOURCE_BYTES / 2 + 1];
        let total_too_large = [
            SourceFile {
                path: "src/a.rs",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: &half_plus_one,
            },
            SourceFile {
                path: "src/b.rs",
                classification: Classification::Public,
                media_type: "provider/ignored",
                bytes: &half_plus_one,
            },
        ];
        assert_eq!(
            build_agent_answer(PROJECT, None, &total_too_large),
            Err(WorkspaceError::TotalTooLarge)
        );

        let too_many = vec![agent_files()[0]; MAX_FILES + 1];
        assert_eq!(
            build_agent_answer(PROJECT, None, &too_many),
            Err(WorkspaceError::TooManyFiles)
        );
    }

    #[test]
    fn agent_answer_child_binds_current_parent() {
        let first = build_agent_answer(PROJECT, None, &agent_files()).unwrap();
        let child = build_agent_answer(
            PROJECT,
            Some(first.revision.revision_sha256),
            &agent_files(),
        )
        .unwrap();
        assert_eq!(
            child.revision.parent_revision_sha256,
            Some(first.revision.revision_sha256)
        );
        assert_eq!(child.revision.action, RevisionAction::AgentAnswer);
        assert_ne!(
            child.revision.revision_sha256,
            first.revision.revision_sha256
        );
    }

    #[test]
    fn revision_binds_parent() {
        let first = build_local_import(PROJECT, None, &files()).unwrap();
        let second =
            build_local_import(PROJECT, Some(first.revision.revision_sha256), &files()).unwrap();
        assert_ne!(
            first.revision.revision_sha256,
            second.revision.revision_sha256
        );
        let agent = build_agent_edit(PROJECT, first.revision.revision_sha256, &files()).unwrap();
        assert_eq!(agent.revision.action, RevisionAction::AgentOverlayCommit);
        assert_ne!(
            agent.revision.revision_sha256,
            second.revision.revision_sha256
        );
    }

    #[test]
    fn paths_and_aliases_fail_closed() {
        for path in ["/x", "../x", "a/../x", "a\\b", "C:x", "CON", "x. "] {
            let mut value = files();
            value[0].path = path;
            assert_eq!(
                build_local_import(PROJECT, None, &value),
                Err(WorkspaceError::InvalidPath)
            );
        }
        let aliases = [
            SourceFile {
                path: "A.txt",
                classification: Classification::Public,
                media_type: "text/markdown",
                bytes: b"a",
            },
            SourceFile {
                path: "a.TXT",
                classification: Classification::Public,
                media_type: "text/markdown",
                bytes: b"b",
            },
            SourceFile {
                path: "B.txt",
                classification: Classification::Public,
                media_type: "text/plain",
                bytes: b"between under byte ordering",
            },
        ];
        assert_eq!(
            build_local_import(PROJECT, None, &aliases),
            Err(WorkspaceError::DuplicatePath)
        );
    }

    #[test]
    fn quotas_and_hash_mutations_are_rejected() {
        let oversized = vec![0u8; MAX_FILE_BYTES + 1];
        let file = [SourceFile {
            path: "large.bin",
            classification: Classification::LocalOnly,
            media_type: "application/octet-stream",
            bytes: &oversized,
        }];
        assert_eq!(
            build_local_import(PROJECT, None, &file),
            Err(WorkspaceError::FileTooLarge)
        );

        let built = build_local_import(PROJECT, None, &files()).unwrap();
        let mut manifest = built.manifest_payload;
        let index = manifest.len() - 1;
        manifest[index] ^= 1;
        assert!(matches!(
            decode_revision_manifest(&manifest),
            Err(WorkspaceError::HashMismatch)
        ));
        let mut blob = encode_blob_record(b"hello").unwrap();
        *blob.last_mut().unwrap() ^= 1;
        assert_eq!(decode_blob_record(&blob), Err(WorkspaceError::HashMismatch));
    }

    #[test]
    fn record_keys_are_content_addressed_and_namespaced() {
        let hash = sha256_bytes(b"blob");
        let blob = blob_record_key(hash);
        let revision = revision_record_key(hash);
        assert!(is_blob_record_key(blob));
        assert!(is_revision_record_key(revision));
        assert_eq!(blob.record_id, revision.record_id);
        assert_ne!(blob.namespace, revision.namespace);
    }
}
