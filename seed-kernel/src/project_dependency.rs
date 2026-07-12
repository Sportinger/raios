use alloc::{string::String, vec::Vec};

use raios_core::{
    parse_sha256_ref,
    project_dependency::{
        build_dependency_bundle, validate_dependency_metadata, ChunkRef, DependencyBundle,
        DependencyBundleInput, DependencyFile, DEPENDENCY_CHUNK_BYTES, MAX_DEPENDENCY_CHUNKS,
        MAX_DEPENDENCY_FILES, MAX_DEPENDENCY_FILE_BYTES,
    },
    project_workspace::ProjectId,
};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{
    module_candidate_channel::decode_base64_chunk, project_dependency_store, project_store,
};

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub(crate) struct ResultView {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) project_id: Option<[u8; 16]>,
    pub(crate) revision_sha256: Option<[u8; 32]>,
    pub(crate) cargo_lock_sha256: Option<[u8; 32]>,
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) origin: Option<String>,
    pub(crate) license_expression: Option<String>,
    pub(crate) license_path: Option<String>,
    pub(crate) license_sha256: Option<[u8; 32]>,
    pub(crate) file_count: usize,
    pub(crate) chunk_count: usize,
    pub(crate) active_path: Option<String>,
    pub(crate) active_byte_len: usize,
    pub(crate) expected_byte_len: usize,
    pub(crate) chunk_sha256: Option<[u8; 32]>,
    pub(crate) chunk_byte_len: usize,
    pub(crate) chunk_persisted: bool,
    pub(crate) bundle_visible: bool,
    pub(crate) orphan_chunk_count: usize,
    pub(crate) bundle: Option<DependencyBundle>,
    pub(crate) bundles: Vec<DependencyBundle>,
    pub(crate) storage_write_attempted: bool,
    pub(crate) writes_persistent_state: bool,
}
impl ResultView {
    fn denied(reason: &'static str) -> Self {
        Self {
            accepted: false,
            reason,
            project_id: None,
            revision_sha256: None,
            cargo_lock_sha256: None,
            name: None,
            version: None,
            origin: None,
            license_expression: None,
            license_path: None,
            license_sha256: None,
            file_count: 0,
            chunk_count: 0,
            active_path: None,
            active_byte_len: 0,
            expected_byte_len: 0,
            chunk_sha256: None,
            chunk_byte_len: 0,
            chunk_persisted: false,
            bundle_visible: false,
            orphan_chunk_count: 0,
            bundle: None,
            bundles: Vec::new(),
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }
    fn session(session: &Session, reason: &'static str) -> Self {
        Self {
            accepted: true,
            reason,
            project_id: Some(session.project_id),
            revision_sha256: Some(session.revision_sha256),
            cargo_lock_sha256: Some(session.cargo_lock_sha256),
            name: Some(session.name.clone()),
            version: Some(session.version.clone()),
            origin: Some(session.origin.clone()),
            license_expression: Some(session.license_expression.clone()),
            license_path: Some(session.license_path.clone()),
            license_sha256: Some(session.license_sha256),
            file_count: session.files.len(),
            chunk_count: session.chunk_count(),
            active_path: session.active.as_ref().map(|f| f.path.clone()),
            active_byte_len: session.active.as_ref().map(|f| f.byte_len).unwrap_or(0),
            expected_byte_len: session.active.as_ref().map(|f| f.whole_len).unwrap_or(0),
            chunk_sha256: None,
            chunk_byte_len: 0,
            chunk_persisted: false,
            bundle_visible: false,
            orphan_chunk_count: session.chunk_count(),
            bundle: None,
            bundles: Vec::new(),
            storage_write_attempted: false,
            writes_persistent_state: false,
        }
    }
}

struct Session {
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    cargo_lock_sha256: [u8; 32],
    name: String,
    version: String,
    origin: String,
    license_expression: String,
    license_path: String,
    license_sha256: [u8; 32],
    files: Vec<StagedFile>,
    active: Option<PendingFile>,
}
impl Session {
    fn chunk_count(&self) -> usize {
        self.files.iter().map(|f| f.chunks.len()).sum::<usize>()
            + self.active.as_ref().map(|f| f.chunks.len()).unwrap_or(0)
    }
    fn input(&self) -> DependencyBundleInput<'_> {
        DependencyBundleInput {
            project_id: ProjectId::new(self.project_id),
            project_revision_sha256: self.revision_sha256,
            cargo_lock_sha256: self.cargo_lock_sha256,
            name: &self.name,
            version: &self.version,
            origin: &self.origin,
            license_expression: &self.license_expression,
            license_path: &self.license_path,
            license_sha256: self.license_sha256,
        }
    }
}
struct StagedFile {
    path: String,
    media_type: String,
    whole_len: usize,
    whole_sha256: [u8; 32],
    chunks: Vec<ChunkRef>,
}
struct PendingFile {
    path: String,
    media_type: String,
    whole_len: usize,
    whole_sha256: [u8; 32],
    byte_len: usize,
    chunks: Vec<ChunkRef>,
    hash: Sha256,
}

pub(crate) fn begin(input: &str) -> ResultView {
    let mut guard = SESSION.lock();
    *guard = None;
    let args = match args::<8>(input, "project.dependency_begin") {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let project_id = match hex16(args[0]) {
        Some(v) => v,
        None => return ResultView::denied("project_id_invalid"),
    };
    let revision = match parse_sha256_ref(args[1]) {
        Some(v) => v,
        None => return ResultView::denied("dependency_revision_invalid"),
    };
    let name = match text_b64(args[2], 128) {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let version = match text_b64(args[3], 64) {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let origin = match text_b64(args[4], 512) {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let license_expression = match text_b64(args[5], 128) {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let license_path = match text_b64(args[6], 240) {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let license_sha256 = match parse_sha256_ref(args[7]) {
        Some(v) => v,
        None => return ResultView::denied("dependency_license_sha256_invalid"),
    };
    let loaded = match project_store::load(ProjectId::new(project_id)) {
        Ok(Some(v)) => v,
        Ok(None) => return ResultView::denied("project_not_found"),
        Err(e) => return ResultView::denied(e.reason()),
    };
    if loaded.revision.revision_sha256 != revision {
        return ResultView::denied("dependency_project_revision_stale");
    }
    let Some(lock) = loaded.files.iter().find(|f| f.entry.path == "Cargo.lock") else {
        return ResultView::denied("dependency_cargo_lock_missing");
    };
    let session = Session {
        project_id,
        revision_sha256: revision,
        cargo_lock_sha256: lock.entry.blob_sha256,
        name,
        version,
        origin,
        license_expression,
        license_path,
        license_sha256,
        files: Vec::new(),
        active: None,
    };
    if let Err(e) = validate_dependency_metadata(session.input()) {
        return ResultView::denied(e.reason());
    }
    let result = ResultView::session(&session, "dependency_import_started");
    *guard = Some(session);
    result
}

pub(crate) fn file_begin(input: &str) -> ResultView {
    let mut guard = SESSION.lock();
    let result = (|| {
        let s = guard.as_mut().ok_or("dependency_import_not_active")?;
        if s.active.is_some() {
            return Err("dependency_file_already_active");
        }
        if s.files.len() >= MAX_DEPENDENCY_FILES {
            return Err("dependency_file_quota_exceeded");
        }
        let a = args::<4>(input, "project.dependency_file_begin")?;
        let path = text_b64(a[0], 240)?;
        let media = validate_media(a[1])?;
        let len = parse_usize(a[2]).ok_or("dependency_file_length_invalid")?;
        if len > MAX_DEPENDENCY_FILE_BYTES {
            return Err("dependency_file_too_large");
        }
        let hash = parse_sha256_ref(a[3]).ok_or("dependency_file_sha256_invalid")?;
        s.active = Some(PendingFile {
            path,
            media_type: media,
            whole_len: len,
            whole_sha256: hash,
            byte_len: 0,
            chunks: Vec::new(),
            hash: Sha256::new(),
        });
        Ok(ResultView::session(s, "dependency_file_started"))
    })();
    match result {
        Ok(v) => v,
        Err(r) => discard(&mut guard, r),
    }
}

pub(crate) fn chunk(input: &str) -> ResultView {
    let mut guard = SESSION.lock();
    let result = (|| {
        let s = guard
            .as_mut()
            .ok_or(("dependency_import_not_active", false))?;
        let encoded = one_arg(input, "project.dependency_chunk")
            .ok_or(("dependency_chunk_malformed", false))?;
        if encoded.len() > encoded_limit(DEPENDENCY_CHUNK_BYTES) {
            return Err(("dependency_chunk_too_large", false));
        }
        let bytes =
            decode_base64_chunk(encoded).map_err(|_| ("dependency_chunk_invalid", false))?;
        let total_chunks = s.chunk_count();
        let active = s
            .active
            .as_mut()
            .ok_or(("dependency_file_not_active", false))?;
        if bytes.is_empty()
            || bytes.len() > DEPENDENCY_CHUNK_BYTES
            || active
                .byte_len
                .checked_add(bytes.len())
                .filter(|n| *n <= active.whole_len)
                .is_none()
        {
            return Err(("dependency_chunk_too_large", false));
        }
        if total_chunks >= MAX_DEPENDENCY_CHUNKS {
            return Err(("dependency_chunk_quota_exceeded", false));
        }
        let (sha, persisted) =
            project_dependency_store::persist_chunk(&bytes).map_err(|e| (e.reason(), true))?;
        active.hash.update(&bytes);
        active.byte_len += bytes.len();
        active.chunks.push(ChunkRef {
            byte_len: bytes.len(),
            sha256: sha,
        });
        let mut view = ResultView::session(s, "dependency_chunk_persisted");
        view.chunk_sha256 = Some(sha);
        view.chunk_byte_len = bytes.len();
        view.chunk_persisted = persisted.persisted;
        view.storage_write_attempted = persisted.storage_write_attempted;
        view.writes_persistent_state = persisted.storage_write_attempted && persisted.persisted;
        Ok(view)
    })();
    match result {
        Ok(v) => v,
        Err((r, attempted)) => {
            let mut v = discard(&mut guard, r);
            v.storage_write_attempted = attempted;
            v
        }
    }
}

pub(crate) fn file_finalize() -> ResultView {
    let mut guard = SESSION.lock();
    let result = (|| {
        let s = guard.as_mut().ok_or("dependency_import_not_active")?;
        let active = s.active.take().ok_or("dependency_file_not_active")?;
        if active.byte_len != active.whole_len {
            return Err("dependency_file_length_mismatch");
        }
        let actual: [u8; 32] = active.hash.finalize().into();
        if actual != active.whole_sha256 {
            return Err("dependency_file_hash_mismatch");
        }
        s.files.push(StagedFile {
            path: active.path,
            media_type: active.media_type,
            whole_len: active.whole_len,
            whole_sha256: active.whole_sha256,
            chunks: active.chunks,
        });
        Ok(ResultView::session(s, "dependency_file_finalized"))
    })();
    match result {
        Ok(v) => v,
        Err(r) => discard(&mut guard, r),
    }
}

pub(crate) fn commit() -> ResultView {
    let mut guard = SESSION.lock();
    let outcome = (|| {
        let s = guard
            .as_ref()
            .ok_or(("dependency_import_not_active", false, false))?;
        if s.active.is_some() {
            return Err(("dependency_file_incomplete", false, false));
        }
        let loaded = project_store::load(ProjectId::new(s.project_id))
            .map_err(|e| (e.reason(), false, false))?
            .ok_or(("project_not_found", false, false))?;
        if loaded.revision.revision_sha256 != s.revision_sha256 {
            return Err(("dependency_project_revision_stale", false, false));
        }
        let lock = loaded
            .files
            .iter()
            .find(|f| f.entry.path == "Cargo.lock")
            .ok_or(("dependency_cargo_lock_missing", false, false))?;
        if lock.entry.blob_sha256 != s.cargo_lock_sha256 {
            return Err(("dependency_cargo_lock_changed", false, false));
        }
        let refs: Vec<_> = s
            .files
            .iter()
            .map(|f| DependencyFile {
                path: &f.path,
                media_type: &f.media_type,
                whole_byte_len: f.whole_len,
                whole_sha256: f.whole_sha256,
                chunks: &f.chunks,
            })
            .collect();
        let built =
            build_dependency_bundle(s.input(), &refs).map_err(|e| (e.reason(), false, false))?;
        let manifest_written = project_dependency_store::commit_bundle(&built)
            .map_err(|e| (e.reason(), true, false))?;
        let bundles =
            project_dependency_store::load_bundles(ProjectId::new(s.project_id), s.revision_sha256)
                .map_err(|e| (e.reason(), manifest_written, manifest_written))?;
        if !bundles.iter().any(|b| b == &built.bundle) {
            return Err((
                "dependency_commit_reload_mismatch",
                manifest_written,
                manifest_written,
            ));
        }
        Ok((built, manifest_written))
    })();
    let denied = outcome.as_ref().err().and_then(|(r, a, w)| {
        guard.as_ref().map(|s| {
            let mut v = ResultView::session(s, false_reason(r));
            v.accepted = false;
            v.storage_write_attempted = *a;
            v.writes_persistent_state = *w;
            v
        })
    });
    *guard = None;
    match outcome {
        Ok((b, manifest_written)) => {
            let mut v = ResultView::denied(if manifest_written {
                "dependency_bundle_committed"
            } else {
                "dependency_bundle_already_present"
            });
            v.accepted = true;
            v.project_id = Some(b.bundle.project_id.bytes());
            v.revision_sha256 = Some(b.bundle.project_revision_sha256);
            v.cargo_lock_sha256 = Some(b.bundle.cargo_lock_sha256);
            v.name = Some(b.bundle.name.clone());
            v.version = Some(b.bundle.version.clone());
            v.origin = Some(b.bundle.origin.clone());
            v.license_expression = Some(b.bundle.license_expression.clone());
            v.license_path = Some(b.bundle.license_path.clone());
            v.license_sha256 = Some(b.bundle.license_sha256);
            v.file_count = b.bundle.files.len();
            v.chunk_count = b.bundle.files.iter().map(|f| f.chunks.len()).sum();
            v.bundle_visible = true;
            v.bundle = Some(b.bundle);
            v.storage_write_attempted = manifest_written;
            v.writes_persistent_state = manifest_written;
            v
        }
        Err((r, a, w)) => denied.unwrap_or_else(|| {
            let mut v = ResultView::denied(r);
            v.storage_write_attempted = a;
            v.writes_persistent_state = w;
            v
        }),
    }
}

pub(crate) fn discard_session() -> ResultView {
    let mut guard = SESSION.lock();
    let v = guard
        .as_ref()
        .map(|s| ResultView::session(s, "dependency_import_discarded"))
        .unwrap_or_else(|| ResultView::denied("dependency_import_not_active"));
    *guard = None;
    v
}
pub(crate) fn inspect(input: &str) -> ResultView {
    let a = match args::<2>(input, "project.dependencies") {
        Ok(v) => v,
        Err(r) => return ResultView::denied(r),
    };
    let id = match hex16(a[0]) {
        Some(v) => v,
        None => return ResultView::denied("project_id_invalid"),
    };
    let rev = match parse_sha256_ref(a[1]) {
        Some(v) => v,
        None => return ResultView::denied("dependency_revision_invalid"),
    };
    let loaded = match project_store::load(ProjectId::new(id)) {
        Ok(Some(v)) => v,
        Ok(None) => return ResultView::denied("project_not_found"),
        Err(e) => return ResultView::denied(e.reason()),
    };
    if loaded.revision.revision_sha256 != rev {
        return ResultView::denied("dependency_project_revision_stale");
    }
    let bundles = match project_dependency_store::load_bundles(ProjectId::new(id), rev) {
        Ok(v) => v,
        Err(e) => return ResultView::denied(e.reason()),
    };
    let mut v = ResultView::denied("dependency_bundles_verified");
    v.accepted = true;
    v.project_id = Some(id);
    v.revision_sha256 = Some(rev);
    v.cargo_lock_sha256 = loaded
        .files
        .iter()
        .find(|f| f.entry.path == "Cargo.lock")
        .map(|f| f.entry.blob_sha256);
    v.bundle_visible = !bundles.is_empty();
    v.bundles = bundles;
    v
}

fn discard(g: &mut Option<Session>, r: &'static str) -> ResultView {
    *g = None;
    ResultView::denied(r)
}
fn false_reason(r: &&'static str) -> &'static str {
    r
}
fn args<'a, const N: usize>(input: &'a str, m: &str) -> Result<[&'a str; N], &'static str> {
    let mut w = input.trim().split_ascii_whitespace();
    if !w.next().is_some_and(|v| v.eq_ignore_ascii_case(m)) {
        return Err("dependency_command_malformed");
    }
    let mut out = [""; N];
    for v in &mut out {
        *v = w.next().ok_or("dependency_command_malformed")?
    }
    if w.next().is_some() {
        return Err("dependency_command_malformed");
    }
    Ok(out)
}
fn one_arg<'a>(input: &'a str, m: &str) -> Option<&'a str> {
    let mut w = input.trim().split_ascii_whitespace();
    if !w.next()?.eq_ignore_ascii_case(m) {
        return None;
    }
    let v = w.next()?;
    w.next().is_none().then_some(v)
}
fn text_b64(v: &str, max: usize) -> Result<String, &'static str> {
    if v.len() > encoded_limit(max) {
        return Err("dependency_text_limit_exceeded");
    }
    let b = decode_base64_chunk(v).map_err(|_| "dependency_text_invalid")?;
    if b.is_empty() || b.len() > max {
        return Err("dependency_text_invalid");
    }
    Ok(String::from(
        core::str::from_utf8(&b).map_err(|_| "dependency_text_invalid")?,
    ))
}
fn validate_media(v: &str) -> Result<String, &'static str> {
    if v.is_empty()
        || v.len() > 64
        || !v.contains('/')
        || v.bytes().any(|b| !(0x21..=0x7e).contains(&b))
    {
        Err("dependency_media_type_invalid")
    } else {
        Ok(String::from(v))
    }
}
fn hex16(v: &str) -> Option<[u8; 16]> {
    if v.len() != 32 {
        return None;
    }
    let mut o = [0; 16];
    for (i, p) in v.as_bytes().chunks_exact(2).enumerate() {
        o[i] = (nib(p[0])? << 4) | nib(p[1])?
    }
    Some(o)
}
fn nib(v: u8) -> Option<u8> {
    match v {
        b'0'..=b'9' => Some(v - b'0'),
        b'a'..=b'f' => Some(v - b'a' + 10),
        b'A'..=b'F' => Some(v - b'A' + 10),
        _ => None,
    }
}
fn parse_usize(v: &str) -> Option<usize> {
    if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    v.bytes().try_fold(0usize, |n, b| {
        n.checked_mul(10)?.checked_add((b - b'0') as usize)
    })
}
fn encoded_limit(n: usize) -> usize {
    n.saturating_add(2) / 3 * 4
}
