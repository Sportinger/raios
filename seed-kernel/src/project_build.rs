use alloc::{string::String, vec::Vec};

use raios_core::{
    parse_sha256_ref,
    project_build::{
        build_receipt, build_snapshot, pinned_toolchain_manifest_sha256, BuildDependencyBundle,
        BuildDependencyChunk, BuildDependencyFile, BuildRun, BuildSourceFile, ProjectBuildReceipt,
        ProjectBuildSnapshot,
    },
    project_dependency::DependencyBundle,
    project_workspace::{ProjectId, MAX_PATH_BYTES},
};
use spin::Mutex;

use crate::{
    module_candidate_channel::decode_base64_chunk, module_candidate_intake,
    project_dependency_store, project_store,
};

const MAX_BUILD_RECEIPTS: usize = 8;
const MAX_READ_BYTES: usize = 512;

static SESSION: Mutex<Option<BuildSession>> = Mutex::new(None);
static RECEIPTS: Mutex<Vec<ProjectBuildReceipt>> = Mutex::new(Vec::new());

#[derive(Clone)]
struct BuildSession {
    snapshot: ProjectBuildSnapshot,
    toolchain_manifest_sha256: [u8; 32],
    run_one: Option<BuildRun>,
    run_two: Option<BuildRun>,
}

pub(crate) struct BuildView {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) snapshot: Option<ProjectBuildSnapshot>,
    pub(crate) toolchain_manifest_sha256: Option<[u8; 32]>,
    pub(crate) run_one: Option<BuildRun>,
    pub(crate) run_two: Option<BuildRun>,
    pub(crate) receipt: Option<ProjectBuildReceipt>,
    pub(crate) receipts: Vec<ProjectBuildReceipt>,
}

impl BuildView {
    fn denied(reason: &'static str) -> Self {
        Self {
            accepted: false,
            reason,
            snapshot: None,
            toolchain_manifest_sha256: None,
            run_one: None,
            run_two: None,
            receipt: None,
            receipts: Vec::new(),
        }
    }

    fn session(session: &BuildSession, reason: &'static str) -> Self {
        Self {
            accepted: true,
            reason,
            snapshot: Some(session.snapshot.clone()),
            toolchain_manifest_sha256: Some(session.toolchain_manifest_sha256),
            run_one: session.run_one,
            run_two: session.run_two,
            receipt: None,
            receipts: Vec::new(),
        }
    }
}

pub(crate) struct ReadView {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) project_id: Option<[u8; 16]>,
    pub(crate) revision_sha256: Option<[u8; 32]>,
    pub(crate) path: Option<String>,
    pub(crate) file_byte_len: usize,
    pub(crate) blob_sha256: Option<[u8; 32]>,
    pub(crate) bundle_sha256: Option<[u8; 32]>,
    pub(crate) tree_sha256: Option<[u8; 32]>,
    pub(crate) whole_sha256: Option<[u8; 32]>,
    pub(crate) dependency_chunks: Vec<BuildDependencyChunk>,
    pub(crate) offset: usize,
    pub(crate) requested_len: usize,
    pub(crate) bytes: Vec<u8>,
    pub(crate) eof: bool,
}

impl ReadView {
    fn denied(reason: &'static str) -> Self {
        Self {
            accepted: false,
            reason,
            project_id: None,
            revision_sha256: None,
            path: None,
            file_byte_len: 0,
            blob_sha256: None,
            bundle_sha256: None,
            tree_sha256: None,
            whole_sha256: None,
            dependency_chunks: Vec::new(),
            offset: 0,
            requested_len: 0,
            bytes: Vec::new(),
            eof: false,
        }
    }
}

pub(crate) fn begin(input: &str) -> BuildView {
    let mut guard = SESSION.lock();
    *guard = None;
    let args = match args::<5>(input, "project.build_begin") {
        Ok(value) => value,
        Err(reason) => return BuildView::denied(reason),
    };
    let project_id = match hex16(args[0]) {
        Some(value) => value,
        None => return BuildView::denied("project_id_invalid"),
    };
    let revision_sha256 = match parse_sha256_ref(args[1]) {
        Some(value) => value,
        None => return BuildView::denied("build_revision_invalid"),
    };
    let toolchain_manifest_sha256 = match parse_sha256_ref(args[2]) {
        Some(value) if value == pinned_toolchain_manifest_sha256() => value,
        _ => return BuildView::denied("build_toolchain_manifest_mismatch"),
    };
    if parse_sha256_ref(args[3]) != Some(raios_core::project_build::build_flags_contract_sha256()) {
        return BuildView::denied("build_flags_contract_mismatch");
    }
    if parse_sha256_ref(args[4])
        != Some(raios_core::project_build::build_environment_contract_sha256())
    {
        return BuildView::denied("build_environment_contract_mismatch");
    }
    let snapshot = match snapshot_exact(project_id, revision_sha256, true) {
        Ok(value) => value,
        Err(reason) => return BuildView::denied(reason),
    };
    let session = BuildSession {
        snapshot,
        toolchain_manifest_sha256,
        run_one: None,
        run_two: None,
    };
    let view = BuildView::session(&session, "build_session_started");
    *guard = Some(session);
    view
}

pub(crate) fn source_read(input: &str) -> ReadView {
    let args = match args::<5>(input, "project.build_source_read") {
        Ok(value) => value,
        Err(reason) => return ReadView::denied(reason),
    };
    let id = match hex16(args[0]) {
        Some(value) => value,
        None => return ReadView::denied("project_id_invalid"),
    };
    let revision = match parse_sha256_ref(args[1]) {
        Some(value) => value,
        None => return ReadView::denied("build_revision_invalid"),
    };
    let path = match text_b64(args[2], MAX_PATH_BYTES) {
        Ok(value) => value,
        Err(reason) => return ReadView::denied(reason),
    };
    let offset = match parse_usize(args[3]) {
        Some(value) => value,
        None => return ReadView::denied("build_read_offset_invalid"),
    };
    let requested_len = match parse_usize(args[4]) {
        Some(value) if value <= MAX_READ_BYTES => value,
        _ => return ReadView::denied("build_read_limit_exceeded"),
    };
    let loaded = match project_store::load(ProjectId::new(id)) {
        Ok(Some(value)) => value,
        Ok(None) => return ReadView::denied("project_not_found"),
        Err(error) => return ReadView::denied(error.reason()),
    };
    if loaded.revision.revision_sha256 != revision {
        return ReadView::denied("build_project_revision_stale");
    }
    let Some(file) = loaded.files.iter().find(|file| file.entry.path == path) else {
        return ReadView::denied("build_source_path_not_found");
    };
    let Some((bytes, eof)) = bounded_range(&file.bytes, offset, requested_len) else {
        return ReadView::denied("build_read_offset_out_of_bounds");
    };
    ReadView {
        accepted: true,
        reason: "build_source_bytes_verified",
        project_id: Some(id),
        revision_sha256: Some(revision),
        path: Some(path),
        file_byte_len: file.bytes.len(),
        blob_sha256: Some(file.entry.blob_sha256),
        bundle_sha256: None,
        tree_sha256: Some(loaded.revision.tree_sha256),
        whole_sha256: Some(file.entry.blob_sha256),
        dependency_chunks: Vec::new(),
        offset,
        requested_len,
        bytes,
        eof,
    }
}

pub(crate) fn dependency_read(input: &str) -> ReadView {
    let args = match args::<6>(input, "project.build_dependency_read") {
        Ok(value) => value,
        Err(reason) => return ReadView::denied(reason),
    };
    let id = match hex16(args[0]) {
        Some(value) => value,
        None => return ReadView::denied("project_id_invalid"),
    };
    let revision = match parse_sha256_ref(args[1]) {
        Some(value) => value,
        None => return ReadView::denied("build_revision_invalid"),
    };
    let bundle_sha256 = match parse_sha256_ref(args[2]) {
        Some(value) => value,
        None => return ReadView::denied("build_dependency_bundle_invalid"),
    };
    let path = match text_b64(args[3], MAX_PATH_BYTES) {
        Ok(value) => value,
        Err(reason) => return ReadView::denied(reason),
    };
    let offset = match parse_usize(args[4]) {
        Some(value) => value,
        None => return ReadView::denied("build_read_offset_invalid"),
    };
    let requested_len = match parse_usize(args[5]) {
        Some(value) if value <= MAX_READ_BYTES => value,
        _ => return ReadView::denied("build_read_limit_exceeded"),
    };
    let loaded = match project_store::load(ProjectId::new(id)) {
        Ok(Some(value)) => value,
        Ok(None) => return ReadView::denied("project_not_found"),
        Err(error) => return ReadView::denied(error.reason()),
    };
    if loaded.revision.revision_sha256 != revision {
        return ReadView::denied("build_project_revision_stale");
    }
    let loaded = match project_dependency_store::load_bundle_file(
        ProjectId::new(id),
        revision,
        bundle_sha256,
        &path,
    ) {
        Ok(Some(value)) => value,
        Ok(None) => return ReadView::denied("build_dependency_path_not_found"),
        Err(error) => return ReadView::denied(error.reason()),
    };
    let Some((bytes, eof)) = bounded_range(&loaded.bytes, offset, requested_len) else {
        return ReadView::denied("build_read_offset_out_of_bounds");
    };
    let dependency_chunks = loaded
        .file
        .chunks
        .iter()
        .map(|chunk| BuildDependencyChunk {
            byte_len: chunk.byte_len,
            sha256: chunk.sha256,
        })
        .collect();
    ReadView {
        accepted: true,
        reason: "build_dependency_bytes_verified",
        project_id: Some(id),
        revision_sha256: Some(revision),
        path: Some(path),
        file_byte_len: loaded.bytes.len(),
        blob_sha256: None,
        bundle_sha256: Some(loaded.bundle.bundle_sha256),
        tree_sha256: Some(loaded.bundle.tree_sha256),
        whole_sha256: Some(loaded.file.whole_sha256),
        dependency_chunks,
        offset,
        requested_len,
        bytes,
        eof,
    }
}

pub(crate) fn record_run(input: &str) -> BuildView {
    let args = match args::<6>(input, "project.build_run") {
        Ok(value) => value,
        Err(reason) => return BuildView::denied(reason),
    };
    let slot = match args[0] {
        "1" => 1,
        "2" => 2,
        _ => return BuildView::denied("build_run_slot_invalid"),
    };
    let exit_code = match parse_i32(args[1]) {
        Some(value) => value,
        None => return BuildView::denied("build_run_exit_code_invalid"),
    };
    let stdout_sha256 = match parse_sha256_ref(args[2]) {
        Some(value) => value,
        None => return BuildView::denied("build_run_stdout_hash_invalid"),
    };
    let stderr_sha256 = match parse_sha256_ref(args[3]) {
        Some(value) => value,
        None => return BuildView::denied("build_run_stderr_hash_invalid"),
    };
    let output_byte_len = match parse_usize(args[4]) {
        Some(value) => value,
        None => return BuildView::denied("build_run_output_length_invalid"),
    };
    let output_sha256 = match parse_sha256_ref(args[5]) {
        Some(value) => value,
        None => return BuildView::denied("build_run_output_hash_invalid"),
    };
    let run = BuildRun {
        exit_code,
        stdout_sha256,
        stderr_sha256,
        output_byte_len,
        output_sha256,
    };
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return BuildView::denied("build_session_not_active");
    };
    let target = if slot == 1 {
        &mut session.run_one
    } else {
        &mut session.run_two
    };
    if target.is_some() {
        return BuildView::denied("build_run_already_recorded");
    }
    *target = Some(run);
    BuildView::session(session, "build_run_recorded")
}

pub(crate) fn commit() -> BuildView {
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_ref().cloned() else {
        return BuildView::denied("build_session_not_active");
    };
    let result = (|| {
        let current = snapshot_exact(
            session.snapshot.project_id.bytes(),
            session.snapshot.project_revision_sha256,
            true,
        )?;
        if current != session.snapshot {
            return Err("build_input_snapshot_changed");
        }
        let run_one = session.run_one.ok_or("build_run_missing")?;
        let run_two = session.run_two.ok_or("build_run_missing")?;
        if run_one.exit_code != 0 || run_two.exit_code != 0 {
            return Err("build_run_failed");
        }
        if run_one.output_sha256 != run_two.output_sha256
            || run_one.output_byte_len != run_two.output_byte_len
        {
            return Err("build_outputs_not_reproducible");
        }
        let candidate = module_candidate_intake::retained().ok_or("build_candidate_missing")?;
        if !candidate.wasm_valid {
            return Err("build_candidate_not_wasm_valid");
        }
        if candidate.sha256 != run_one.output_sha256
            || candidate.bytes.len() != run_one.output_byte_len
        {
            return Err("build_candidate_mismatch");
        }
        build_receipt(
            session.snapshot.clone(),
            session.toolchain_manifest_sha256,
            run_one,
            run_two,
            candidate.sha256,
            candidate.bytes.len(),
            candidate.wasm_valid,
        )
        .map_err(|error| error.reason())
    })();
    *guard = None;
    match result {
        Ok(receipt) => {
            retain_receipt(receipt.clone());
            BuildView {
                accepted: true,
                reason: "build_receipt_ready",
                snapshot: Some(receipt.snapshot.clone()),
                toolchain_manifest_sha256: Some(receipt.toolchain_manifest_sha256),
                run_one: Some(receipt.run_one),
                run_two: Some(receipt.run_two),
                receipt: Some(receipt),
                receipts: Vec::new(),
            }
        }
        Err(reason) => BuildView::denied(reason),
    }
}

pub(crate) fn discard() -> BuildView {
    let mut guard = SESSION.lock();
    let view = guard
        .as_ref()
        .map(|session| BuildView::session(session, "build_session_discarded"))
        .unwrap_or_else(|| BuildView::denied("build_session_not_active"));
    *guard = None;
    view
}

pub(crate) fn receipts(input: &str) -> BuildView {
    let args = match args::<2>(input, "project.build_receipts") {
        Ok(value) => value,
        Err(reason) => return BuildView::denied(reason),
    };
    let id = match hex16(args[0]) {
        Some(value) => value,
        None => return BuildView::denied("project_id_invalid"),
    };
    let revision = match parse_sha256_ref(args[1]) {
        Some(value) => value,
        None => return BuildView::denied("build_revision_invalid"),
    };
    let current = match snapshot_exact(id, revision, false) {
        Ok(value) => value,
        Err(reason) => return BuildView::denied(reason),
    };
    let retained: Vec<_> = RECEIPTS
        .lock()
        .iter()
        .filter(|receipt| {
            receipt.snapshot.project_id.bytes() == id
                && receipt.snapshot.project_revision_sha256 == revision
        })
        .cloned()
        .collect();
    let receipts: Vec<_> = retained
        .iter()
        .filter(|receipt| receipt.snapshot == current)
        .cloned()
        .collect();
    if !retained.is_empty() && receipts.is_empty() {
        let mut view = BuildView::denied("build_receipt_input_snapshot_stale");
        view.snapshot = Some(current);
        return view;
    }
    let mut view = BuildView::denied("build_receipts_present");
    view.accepted = true;
    view.snapshot = receipts.first().map(|receipt| receipt.snapshot.clone());
    view.toolchain_manifest_sha256 = receipts
        .first()
        .map(|receipt| receipt.toolchain_manifest_sha256);
    view.receipt = receipts.last().cloned();
    view.receipts = receipts;
    view
}

fn snapshot_exact(
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    enforce_build_code_policy: bool,
) -> Result<ProjectBuildSnapshot, &'static str> {
    let loaded = project_store::load(ProjectId::new(project_id))
        .map_err(|error| error.reason())?
        .ok_or("project_not_found")?;
    if loaded.revision.revision_sha256 != revision_sha256 {
        return Err("build_project_revision_stale");
    }
    if enforce_build_code_policy
        && loaded
            .files
            .iter()
            .any(|file| is_build_script(&file.entry.path))
    {
        return Err("build_source_build_script_denied");
    }
    let cargo_lock = loaded
        .files
        .iter()
        .find(|file| file.entry.path == "Cargo.lock")
        .ok_or("build_cargo_lock_missing")?;
    if !loaded
        .files
        .iter()
        .any(|file| file.entry.path == "Cargo.toml")
    {
        return Err("build_cargo_toml_missing");
    }
    let bundles =
        project_dependency_store::load_bundles(ProjectId::new(project_id), revision_sha256)
            .map_err(|error| error.reason())?;
    if enforce_build_code_policy && bundles.iter().any(|bundle| bundle.build_script_present) {
        return Err("build_dependency_build_script_denied");
    }
    let source_files = loaded
        .files
        .iter()
        .map(|file| BuildSourceFile {
            path: file.entry.path.clone(),
            byte_len: file.bytes.len(),
            blob_sha256: file.entry.blob_sha256,
        })
        .collect();
    let dependencies = bundles.iter().map(build_dependency).collect();
    build_snapshot(
        ProjectId::new(project_id),
        revision_sha256,
        loaded.revision.tree_sha256,
        cargo_lock.entry.blob_sha256,
        source_files,
        dependencies,
    )
    .map_err(|error| error.reason())
}

fn build_dependency(bundle: &DependencyBundle) -> BuildDependencyBundle {
    BuildDependencyBundle {
        bundle_sha256: bundle.bundle_sha256,
        tree_sha256: bundle.tree_sha256,
        files: bundle
            .files
            .iter()
            .map(|file| BuildDependencyFile {
                path: file.path.clone(),
                whole_byte_len: file.whole_byte_len,
                whole_sha256: file.whole_sha256,
                chunks: file
                    .chunks
                    .iter()
                    .map(|chunk| BuildDependencyChunk {
                        byte_len: chunk.byte_len,
                        sha256: chunk.sha256,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn retain_receipt(receipt: ProjectBuildReceipt) {
    let mut receipts = RECEIPTS.lock();
    if receipts
        .iter()
        .any(|existing| existing.receipt_sha256 == receipt.receipt_sha256)
    {
        return;
    }
    if receipts.len() == MAX_BUILD_RECEIPTS {
        receipts.remove(0);
    }
    receipts.push(receipt);
}

fn bounded_range(bytes: &[u8], offset: usize, len: usize) -> Option<(Vec<u8>, bool)> {
    if offset > bytes.len() {
        return None;
    }
    let end = offset.saturating_add(len).min(bytes.len());
    Some((bytes[offset..end].to_vec(), end == bytes.len()))
}

fn is_build_script(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("build.rs"))
}

fn args<'a, const N: usize>(input: &'a str, method: &str) -> Result<[&'a str; N], &'static str> {
    let mut words = input.trim().split_ascii_whitespace();
    if !words
        .next()
        .is_some_and(|value| value.eq_ignore_ascii_case(method))
    {
        return Err("build_command_malformed");
    }
    let mut out = [""; N];
    for value in &mut out {
        *value = words.next().ok_or("build_command_malformed")?;
    }
    if words.next().is_some() {
        return Err("build_command_malformed");
    }
    Ok(out)
}

fn text_b64(value: &str, max: usize) -> Result<String, &'static str> {
    if value.len() > max.saturating_add(2) / 3 * 4 {
        return Err("build_path_invalid");
    }
    let bytes = decode_base64_chunk(value).map_err(|_| "build_path_invalid")?;
    if bytes.is_empty() || bytes.len() > max {
        return Err("build_path_invalid");
    }
    let text = core::str::from_utf8(&bytes).map_err(|_| "build_path_invalid")?;
    Ok(String::from(text))
}

fn hex16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut out = [0; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(out)
}

fn nibble(value: u8) -> Option<u8> {
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

fn parse_i32(value: &str) -> Option<i32> {
    let (negative, digits) = value
        .strip_prefix('-')
        .map(|digits| (true, digits))
        .unwrap_or((false, value));
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let parsed = digits.bytes().try_fold(0i64, |parsed, byte| {
        parsed.checked_mul(10)?.checked_add((byte - b'0') as i64)
    })?;
    i32::try_from(if negative { -parsed } else { parsed }).ok()
}
