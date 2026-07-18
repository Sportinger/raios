use alloc::{string::String, vec::Vec};

use crate::{project_workspace::ProjectId, sha256_bytes, structured_store::RecordKey};

pub const DEPENDENCY_CHUNK_BYTES: usize = 24 * 1024;
pub const MAX_DEPENDENCY_CHUNKS: usize = 64;
pub const MAX_DEPENDENCY_FILES: usize = 32;
pub const MAX_DEPENDENCY_FILE_BYTES: usize = 512 * 1024;
pub const MAX_DEPENDENCY_NAME_BYTES: usize = 128;
pub const MAX_DEPENDENCY_VERSION_BYTES: usize = 64;
pub const MAX_DEPENDENCY_LICENSE_BYTES: usize = 128;
pub const MAX_DEPENDENCY_ORIGIN_BYTES: usize = 512;
pub const DEPENDENCY_ORIGIN: &str = "owner_local_serial";
pub const DEPENDENCY_CHUNK_NAMESPACE: [u8; 16] = *b"project.depchnk!";
pub const DEPENDENCY_MANIFEST_NAMESPACE: [u8; 16] = *b"project.depmanf!";

const CHUNK_MAGIC: &[u8; 8] = b"RAIODC01";
const MANIFEST_MAGIC: &[u8; 8] = b"RAIODM01";
const TREE_DOMAIN: &[u8] = b"raios.dependency_tree.v1";
const BUNDLE_DOMAIN: &[u8] = b"raios.dependency_bundle.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkRef {
    pub byte_len: usize,
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct DependencyFile<'a> {
    pub path: &'a str,
    pub media_type: &'a str,
    pub whole_byte_len: usize,
    pub whole_sha256: [u8; 32],
    pub chunks: &'a [ChunkRef],
}

#[derive(Clone, Copy)]
pub struct DependencyBundleInput<'a> {
    pub project_id: ProjectId,
    pub project_revision_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub name: &'a str,
    pub version: &'a str,
    pub origin: &'a str,
    pub license_expression: &'a str,
    pub license_path: &'a str,
    pub license_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyFileEntry {
    pub path: String,
    pub media_type: String,
    pub whole_byte_len: usize,
    pub whole_sha256: [u8; 32],
    pub chunks: Vec<ChunkRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyBundle {
    pub project_id: ProjectId,
    pub project_revision_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub name: String,
    pub version: String,
    pub origin: String,
    pub license_expression: String,
    pub license_path: String,
    pub license_sha256: [u8; 32],
    pub files: Vec<DependencyFileEntry>,
    pub tree_sha256: [u8; 32],
    pub bundle_sha256: [u8; 32],
    pub build_script_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltDependencyBundle {
    pub bundle: DependencyBundle,
    pub manifest_payload: Vec<u8>,
    pub manifest_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedDependencyChunk {
    pub sha256: [u8; 32],
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyError {
    InvalidMetadata,
    InvalidPath,
    InvalidMediaType,
    NoFiles,
    TooManyFiles,
    DuplicatePath,
    FileTooLarge,
    ChunkTooLarge,
    TooManyChunks,
    ChunkLengthMismatch,
    FileLengthMismatch,
    LicenseFileMissing,
    LicenseHashMismatch,
    Malformed,
    HashMismatch,
    LimitExceeded,
}

impl DependencyError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::InvalidMetadata => "dependency_metadata_invalid",
            Self::InvalidPath => "dependency_path_invalid",
            Self::InvalidMediaType => "dependency_media_type_invalid",
            Self::NoFiles => "dependency_files_empty",
            Self::TooManyFiles => "dependency_file_quota_exceeded",
            Self::DuplicatePath => "dependency_path_collision",
            Self::FileTooLarge => "dependency_file_too_large",
            Self::ChunkTooLarge => "dependency_chunk_too_large",
            Self::TooManyChunks => "dependency_chunk_quota_exceeded",
            Self::ChunkLengthMismatch => "dependency_chunk_length_mismatch",
            Self::FileLengthMismatch => "dependency_file_length_mismatch",
            Self::LicenseFileMissing => "dependency_license_file_missing",
            Self::LicenseHashMismatch => "dependency_license_hash_mismatch",
            Self::Malformed => "dependency_record_malformed",
            Self::HashMismatch => "dependency_record_hash_mismatch",
            Self::LimitExceeded => "dependency_record_limit_exceeded",
        }
    }
}

pub fn validate_dependency_metadata(
    input: DependencyBundleInput<'_>,
) -> Result<(), DependencyError> {
    validate_text(input.name, MAX_DEPENDENCY_NAME_BYTES)?;
    validate_text(input.version, MAX_DEPENDENCY_VERSION_BYTES)?;
    validate_text(input.license_expression, MAX_DEPENDENCY_LICENSE_BYTES)?;
    validate_path(input.license_path)?;
    validate_text(input.origin, MAX_DEPENDENCY_ORIGIN_BYTES)?;
    if input.project_revision_sha256 == [0; 32]
        || input.cargo_lock_sha256 == [0; 32]
        || input.license_sha256 == [0; 32]
    {
        return Err(DependencyError::InvalidMetadata);
    }
    Ok(())
}

pub fn build_dependency_bundle(
    input: DependencyBundleInput<'_>,
    files: &[DependencyFile<'_>],
) -> Result<BuiltDependencyBundle, DependencyError> {
    validate_dependency_metadata(input)?;
    if files.is_empty() {
        return Err(DependencyError::NoFiles);
    }
    if files.len() > MAX_DEPENDENCY_FILES {
        return Err(DependencyError::TooManyFiles);
    }
    let mut entries = Vec::with_capacity(files.len());
    let mut chunk_count = 0usize;
    for file in files {
        validate_path(file.path)?;
        validate_media_type(file.media_type)?;
        if file.whole_byte_len > MAX_DEPENDENCY_FILE_BYTES {
            return Err(DependencyError::FileTooLarge);
        }
        let mut length = 0usize;
        for chunk in file.chunks {
            if chunk.byte_len == 0 || chunk.byte_len > DEPENDENCY_CHUNK_BYTES {
                return Err(DependencyError::ChunkTooLarge);
            }
            length = length
                .checked_add(chunk.byte_len)
                .ok_or(DependencyError::LimitExceeded)?;
        }
        chunk_count = chunk_count
            .checked_add(file.chunks.len())
            .ok_or(DependencyError::LimitExceeded)?;
        if chunk_count > MAX_DEPENDENCY_CHUNKS {
            return Err(DependencyError::TooManyChunks);
        }
        if length != file.whole_byte_len
            || (file.whole_byte_len == 0 && !file.chunks.is_empty())
            || (file.whole_byte_len != 0 && file.chunks.is_empty())
        {
            return Err(DependencyError::FileLengthMismatch);
        }
        entries.push(DependencyFileEntry {
            path: String::from(file.path),
            media_type: String::from(file.media_type),
            whole_byte_len: file.whole_byte_len,
            whole_sha256: file.whole_sha256,
            chunks: file.chunks.to_vec(),
        });
    }
    entries.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    for pair in entries.windows(2) {
        if aliases(&pair[0].path, &pair[1].path) {
            return Err(DependencyError::DuplicatePath);
        }
    }
    let license = entries
        .iter()
        .find(|file| file.path == input.license_path)
        .ok_or(DependencyError::LicenseFileMissing)?;
    if license.whole_sha256 != input.license_sha256 {
        return Err(DependencyError::LicenseHashMismatch);
    }
    let tree_sha256 = tree_hash(&entries)?;
    let bundle_sha256 = bundle_hash(input, tree_sha256)?;
    let bundle = DependencyBundle {
        project_id: input.project_id,
        project_revision_sha256: input.project_revision_sha256,
        cargo_lock_sha256: input.cargo_lock_sha256,
        name: String::from(input.name),
        version: String::from(input.version),
        origin: String::from(input.origin),
        license_expression: String::from(input.license_expression),
        license_path: String::from(input.license_path),
        license_sha256: input.license_sha256,
        build_script_present: entries.iter().any(|file| is_build_script(&file.path)),
        files: entries,
        tree_sha256,
        bundle_sha256,
    };
    let manifest_payload = encode_dependency_manifest(&bundle)?;
    let manifest_sha256 = sha256_bytes(&manifest_payload);
    Ok(BuiltDependencyBundle {
        bundle,
        manifest_payload,
        manifest_sha256,
    })
}

pub fn encode_dependency_chunk(bytes: &[u8]) -> Result<Vec<u8>, DependencyError> {
    if bytes.is_empty() || bytes.len() > DEPENDENCY_CHUNK_BYTES {
        return Err(DependencyError::ChunkTooLarge);
    }
    let mut out = Vec::with_capacity(44 + bytes.len());
    out.extend_from_slice(CHUNK_MAGIC);
    out.extend_from_slice(&sha256_bytes(bytes));
    put_u32(&mut out, bytes.len())?;
    out.extend_from_slice(bytes);
    Ok(out)
}

pub fn decode_dependency_chunk(payload: &[u8]) -> Result<DecodedDependencyChunk, DependencyError> {
    if payload.len() < 44 || &payload[..8] != CHUNK_MAGIC {
        return Err(DependencyError::Malformed);
    }
    let mut hash = [0; 32];
    hash.copy_from_slice(&payload[8..40]);
    let len = u32::from_le_bytes(
        payload[40..44]
            .try_into()
            .map_err(|_| DependencyError::Malformed)?,
    ) as usize;
    if len == 0 || len > DEPENDENCY_CHUNK_BYTES || payload.len() != 44 + len {
        return Err(DependencyError::Malformed);
    }
    let bytes = payload[44..].to_vec();
    if sha256_bytes(&bytes) != hash {
        return Err(DependencyError::HashMismatch);
    }
    Ok(DecodedDependencyChunk {
        sha256: hash,
        bytes,
    })
}

pub fn encode_dependency_manifest(bundle: &DependencyBundle) -> Result<Vec<u8>, DependencyError> {
    validate_bundle(bundle)?;
    let mut out = Vec::new();
    out.extend_from_slice(MANIFEST_MAGIC);
    out.extend_from_slice(&bundle.project_id.bytes());
    out.extend_from_slice(&bundle.project_revision_sha256);
    out.extend_from_slice(&bundle.cargo_lock_sha256);
    put_string(&mut out, &bundle.name)?;
    put_string(&mut out, &bundle.version)?;
    put_string(&mut out, &bundle.origin)?;
    put_string(&mut out, &bundle.license_expression)?;
    put_string(&mut out, &bundle.license_path)?;
    out.extend_from_slice(&bundle.license_sha256);
    out.extend_from_slice(&bundle.tree_sha256);
    out.extend_from_slice(&bundle.bundle_sha256);
    out.push(u8::from(bundle.build_script_present));
    put_u16(&mut out, bundle.files.len())?;
    encode_files(&mut out, &bundle.files)?;
    if out.len() > crate::structured_store::MAX_RECORD_PAYLOAD_LEN {
        return Err(DependencyError::LimitExceeded);
    }
    Ok(out)
}

pub fn decode_dependency_manifest(payload: &[u8]) -> Result<DependencyBundle, DependencyError> {
    let mut cursor = Cursor::new(payload);
    if cursor.take(8)? != MANIFEST_MAGIC {
        return Err(DependencyError::Malformed);
    }
    let mut project = [0; 16];
    project.copy_from_slice(cursor.take(16)?);
    let revision = cursor.hash()?;
    let cargo_lock = cursor.hash()?;
    let name = String::from(cursor.string(MAX_DEPENDENCY_NAME_BYTES)?);
    let version = String::from(cursor.string(MAX_DEPENDENCY_VERSION_BYTES)?);
    let origin = String::from(cursor.string(64)?);
    let license_expression = String::from(cursor.string(MAX_DEPENDENCY_LICENSE_BYTES)?);
    let license_path = String::from(cursor.string(240)?);
    let license_sha256 = cursor.hash()?;
    let tree_sha256 = cursor.hash()?;
    let bundle_sha256 = cursor.hash()?;
    let build_script_present = match cursor.u8()? {
        0 => false,
        1 => true,
        _ => return Err(DependencyError::Malformed),
    };
    let count = cursor.u16()? as usize;
    if count == 0 || count > MAX_DEPENDENCY_FILES {
        return Err(DependencyError::LimitExceeded);
    }
    let mut files = Vec::with_capacity(count);
    for _ in 0..count {
        let path = String::from(cursor.string(240)?);
        let media_type = String::from(cursor.string(64)?);
        let whole_byte_len = cursor.u32()? as usize;
        let whole_sha256 = cursor.hash()?;
        let chunk_count = cursor.u16()? as usize;
        let mut chunks = Vec::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            chunks.push(ChunkRef {
                byte_len: cursor.u32()? as usize,
                sha256: cursor.hash()?,
            });
        }
        files.push(DependencyFileEntry {
            path,
            media_type,
            whole_byte_len,
            whole_sha256,
            chunks,
        });
    }
    if !cursor.finished() {
        return Err(DependencyError::Malformed);
    }
    let bundle = DependencyBundle {
        project_id: ProjectId::new(project),
        project_revision_sha256: revision,
        cargo_lock_sha256: cargo_lock,
        name,
        version,
        origin,
        license_expression,
        license_path,
        license_sha256,
        files,
        tree_sha256,
        bundle_sha256,
        build_script_present,
    };
    validate_bundle(&bundle)?;
    Ok(bundle)
}

pub fn dependency_chunk_record_key(hash: [u8; 32]) -> RecordKey {
    RecordKey {
        namespace: DEPENDENCY_CHUNK_NAMESPACE,
        record_id: prefix(hash),
    }
}
pub fn dependency_manifest_record_key(hash: [u8; 32]) -> RecordKey {
    RecordKey {
        namespace: DEPENDENCY_MANIFEST_NAMESPACE,
        record_id: prefix(hash),
    }
}
pub fn is_dependency_manifest_record_key(key: RecordKey) -> bool {
    key.namespace == DEPENDENCY_MANIFEST_NAMESPACE
}

fn validate_bundle(bundle: &DependencyBundle) -> Result<(), DependencyError> {
    let refs: Vec<_> = bundle
        .files
        .iter()
        .map(|f| DependencyFile {
            path: &f.path,
            media_type: &f.media_type,
            whole_byte_len: f.whole_byte_len,
            whole_sha256: f.whole_sha256,
            chunks: &f.chunks,
        })
        .collect();
    let input = DependencyBundleInput {
        project_id: bundle.project_id,
        project_revision_sha256: bundle.project_revision_sha256,
        cargo_lock_sha256: bundle.cargo_lock_sha256,
        name: &bundle.name,
        version: &bundle.version,
        origin: &bundle.origin,
        license_expression: &bundle.license_expression,
        license_path: &bundle.license_path,
        license_sha256: bundle.license_sha256,
    };
    let rebuilt = build_dependency_bundle_unencoded(input, &refs)?;
    if rebuilt.tree_sha256 != bundle.tree_sha256
        || rebuilt.bundle_sha256 != bundle.bundle_sha256
        || rebuilt.build_script_present != bundle.build_script_present
        || rebuilt.files != bundle.files
    {
        return Err(DependencyError::HashMismatch);
    }
    Ok(())
}

fn build_dependency_bundle_unencoded(
    input: DependencyBundleInput<'_>,
    files: &[DependencyFile<'_>],
) -> Result<DependencyBundle, DependencyError> {
    // Reuse the public validator without recursively encoding.
    validate_dependency_metadata(input)?;
    let mut entries = Vec::new();
    let mut total_chunks = 0usize;
    if files.is_empty() || files.len() > MAX_DEPENDENCY_FILES {
        return Err(DependencyError::NoFiles);
    }
    for f in files {
        validate_path(f.path)?;
        validate_media_type(f.media_type)?;
        if f.whole_byte_len > MAX_DEPENDENCY_FILE_BYTES {
            return Err(DependencyError::FileTooLarge);
        }
        let mut len = 0usize;
        for chunk in f.chunks {
            if chunk.byte_len == 0
                || chunk.byte_len > DEPENDENCY_CHUNK_BYTES
                || chunk.sha256 == [0; 32]
            {
                return Err(DependencyError::ChunkTooLarge);
            }
            len = len
                .checked_add(chunk.byte_len)
                .ok_or(DependencyError::LimitExceeded)?;
        }
        total_chunks += f.chunks.len();
        if total_chunks > MAX_DEPENDENCY_CHUNKS {
            return Err(DependencyError::TooManyChunks);
        }
        if len != f.whole_byte_len
            || (f.whole_byte_len == 0
                && (f.whole_sha256 != sha256_bytes(&[]) || !f.chunks.is_empty()))
            || (f.whole_byte_len != 0 && f.chunks.is_empty())
        {
            return Err(DependencyError::FileLengthMismatch);
        }
        entries.push(DependencyFileEntry {
            path: String::from(f.path),
            media_type: String::from(f.media_type),
            whole_byte_len: f.whole_byte_len,
            whole_sha256: f.whole_sha256,
            chunks: f.chunks.to_vec(),
        });
    }
    entries.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    for p in entries.windows(2) {
        if aliases(&p[0].path, &p[1].path) {
            return Err(DependencyError::DuplicatePath);
        }
    }
    let license = entries
        .iter()
        .find(|f| f.path == input.license_path)
        .ok_or(DependencyError::LicenseFileMissing)?;
    if license.whole_sha256 != input.license_sha256 {
        return Err(DependencyError::LicenseHashMismatch);
    }
    let tree_sha256 = tree_hash(&entries)?;
    let bundle_sha256 = bundle_hash(input, tree_sha256)?;
    Ok(DependencyBundle {
        project_id: input.project_id,
        project_revision_sha256: input.project_revision_sha256,
        cargo_lock_sha256: input.cargo_lock_sha256,
        name: String::from(input.name),
        version: String::from(input.version),
        origin: String::from(input.origin),
        license_expression: String::from(input.license_expression),
        license_path: String::from(input.license_path),
        license_sha256: input.license_sha256,
        build_script_present: entries.iter().any(|f| is_build_script(&f.path)),
        files: entries,
        tree_sha256,
        bundle_sha256,
    })
}

fn tree_hash(files: &[DependencyFileEntry]) -> Result<[u8; 32], DependencyError> {
    let mut out = Vec::new();
    out.extend_from_slice(TREE_DOMAIN);
    put_u16(&mut out, files.len())?;
    encode_files(&mut out, files)?;
    Ok(sha256_bytes(&out))
}
fn bundle_hash(
    input: DependencyBundleInput<'_>,
    tree: [u8; 32],
) -> Result<[u8; 32], DependencyError> {
    let mut out = Vec::new();
    out.extend_from_slice(BUNDLE_DOMAIN);
    out.extend_from_slice(&input.project_id.bytes());
    out.extend_from_slice(&input.project_revision_sha256);
    out.extend_from_slice(&input.cargo_lock_sha256);
    put_string(&mut out, input.name)?;
    put_string(&mut out, input.version)?;
    put_string(&mut out, input.origin)?;
    put_string(&mut out, input.license_expression)?;
    put_string(&mut out, input.license_path)?;
    out.extend_from_slice(&input.license_sha256);
    out.extend_from_slice(&tree);
    Ok(sha256_bytes(&out))
}
fn encode_files(out: &mut Vec<u8>, files: &[DependencyFileEntry]) -> Result<(), DependencyError> {
    for f in files {
        put_string(out, &f.path)?;
        put_string(out, &f.media_type)?;
        put_u32(out, f.whole_byte_len)?;
        out.extend_from_slice(&f.whole_sha256);
        put_u16(out, f.chunks.len())?;
        for c in &f.chunks {
            put_u32(out, c.byte_len)?;
            out.extend_from_slice(&c.sha256);
        }
    }
    Ok(())
}
fn validate_text(v: &str, max: usize) -> Result<(), DependencyError> {
    if v.is_empty() || v.len() > max || v.bytes().any(|b| b < 0x20 || b == 0x7f) {
        Err(DependencyError::InvalidMetadata)
    } else {
        Ok(())
    }
}
fn validate_media_type(v: &str) -> Result<(), DependencyError> {
    if v.is_empty()
        || v.len() > 64
        || !v.contains('/')
        || v.bytes().any(|b| !(0x21..=0x7e).contains(&b))
    {
        Err(DependencyError::InvalidMediaType)
    } else {
        Ok(())
    }
}
fn validate_path(v: &str) -> Result<(), DependencyError> {
    if v.is_empty()
        || v.len() > 240
        || v.starts_with('/')
        || v.contains('\\')
        || v.contains(':')
        || v.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
        || v.split('/').any(|s| s.is_empty() || s == "." || s == "..")
    {
        Err(DependencyError::InvalidPath)
    } else {
        Ok(())
    }
}
fn aliases(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(x, y)| x.eq_ignore_ascii_case(&y))
}
fn is_build_script(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("build.rs"))
}
fn put_u16(out: &mut Vec<u8>, v: usize) -> Result<(), DependencyError> {
    out.extend_from_slice(
        &u16::try_from(v)
            .map_err(|_| DependencyError::LimitExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}
fn put_u32(out: &mut Vec<u8>, v: usize) -> Result<(), DependencyError> {
    out.extend_from_slice(
        &u32::try_from(v)
            .map_err(|_| DependencyError::LimitExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}
fn put_string(out: &mut Vec<u8>, v: &str) -> Result<(), DependencyError> {
    put_u16(out, v.len())?;
    out.extend_from_slice(v.as_bytes());
    Ok(())
}
fn prefix(h: [u8; 32]) -> [u8; 16] {
    let mut p = [0; 16];
    p.copy_from_slice(&h[..16]);
    p
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], DependencyError> {
        let e = self
            .offset
            .checked_add(n)
            .ok_or(DependencyError::Malformed)?;
        let v = self
            .bytes
            .get(self.offset..e)
            .ok_or(DependencyError::Malformed)?;
        self.offset = e;
        Ok(v)
    }
    fn u8(&mut self) -> Result<u8, DependencyError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, DependencyError> {
        Ok(u16::from_le_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| DependencyError::Malformed)?,
        ))
    }
    fn u32(&mut self) -> Result<u32, DependencyError> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| DependencyError::Malformed)?,
        ))
    }
    fn hash(&mut self) -> Result<[u8; 32], DependencyError> {
        let mut h = [0; 32];
        h.copy_from_slice(self.take(32)?);
        Ok(h)
    }
    fn string(&mut self, max: usize) -> Result<&'a str, DependencyError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(DependencyError::LimitExceeded);
        }
        core::str::from_utf8(self.take(n)?).map_err(|_| DependencyError::Malformed)
    }
    fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_roundtrip() {
        let p = encode_dependency_chunk(b"abc").unwrap();
        assert_eq!(decode_dependency_chunk(&p).unwrap().bytes, b"abc");
    }
    #[test]
    fn bundle_roundtrip_and_hashes() {
        let bytes = b"MIT";
        let c = ChunkRef {
            byte_len: 3,
            sha256: sha256_bytes(bytes),
        };
        let chunks = [c];
        let files = [
            DependencyFile {
                path: "LICENSE",
                media_type: "text/plain",
                whole_byte_len: 3,
                whole_sha256: c.sha256,
                chunks: &chunks,
            },
            DependencyFile {
                path: "tools/BUILD.RS",
                media_type: "text/rust",
                whole_byte_len: 3,
                whole_sha256: c.sha256,
                chunks: &chunks,
            },
        ];
        let input = DependencyBundleInput {
            project_id: ProjectId::new([1; 16]),
            project_revision_sha256: [2; 32],
            cargo_lock_sha256: [3; 32],
            name: "demo",
            version: "1.0.0",
            origin: "serial://owner/dependencies/demo-1.0.0.bundle",
            license_expression: "MIT",
            license_path: "LICENSE",
            license_sha256: c.sha256,
        };
        let b = build_dependency_bundle(input, &files).unwrap();
        assert!(b.bundle.build_script_present);
        assert_eq!(
            decode_dependency_manifest(&b.manifest_payload).unwrap(),
            b.bundle
        );
    }
}
