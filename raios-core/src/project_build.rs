use alloc::{string::String, vec::Vec};

use crate::{project_workspace::ProjectId, sha256_bytes};

pub const BUILD_TARGET: &str = "wasm32-unknown-unknown";
pub const BUILD_FLAGS_CONTRACT: &str = "cargo_command=cargo build --frozen --offline --locked --manifest-path=<input>/Cargo.toml --target=wasm32-unknown-unknown --release --target-dir=<fresh>\n\
rustflags=-Cpanic=abort\u{001f}-Copt-level=z\u{001f}-Ccodegen-units=1\u{001f}-Cdebuginfo=0\u{001f}-Cstrip=symbols\u{001f}-Clinker=<toolchain>/lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe\u{001f}--remap-path-prefix=<input>=/raios/source\n";
pub const BUILD_ENVIRONMENT_CONTRACT: &str =
    "CARGO_INCREMENTAL=0\nCARGO_NET_OFFLINE=true\nRUST_BACKTRACE=0\nSOURCE_DATE_EPOCH=0\nTZ=UTC\n";
pub const BUILD_CODE_POLICY: &str = "deny_all_build_scripts_and_proc_macros_v1";
pub const BUILD_EVIDENCE_QUALITY: &str = "builder_attested_not_local_rebuild";
pub const PINNED_TOOLCHAIN_TRUST: &str = "pinned_dev_workstation_not_owner_sealed";
pub const PINNED_TOOLCHAIN_MANIFEST: &str =
    "schema=raios.owner_workstation_toolchain_manifest.v1\n\
cargo_version=cargo 1.84.0-nightly (15fbd2f60 2024-10-08)\n\
cargo_sha256=beb0e989a1597b624895907b918127bc08da919107b39bd9e954b3a1836aa13a\n\
rustc_version=rustc 1.84.0-nightly (9322d183f 2024-10-14)\n\
rustc_sha256=4fee4125dcdae28d01f0314a3820a09e9f9f717ee72494dc31ffe2d6de7eebf9\n\
rust_lld_sha256=375f02b38440bc5d7b0c3974abeb0fef080b654dc2fb87ef23887df7116129d2\n\
rustc_driver_sha256=e4aad069c7b57352c818aef521509ca42cc26cbfa5a20589ea9645bb4e111810\n\
host_std_sha256=e54497532d6dcc306ada9a6086f30a2de5ed21740ab636414e569e017726fdfa\n\
wasm_target_manifest_sha256=032c9d596654633f8ab159236fff270bde984a15541763ebc6af5e2aaa560557\n\
target=wasm32-unknown-unknown\n\
target_lib_set_sha256=0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde\n";

const SNAPSHOT_MAGIC: &[u8; 8] = b"RAIOBS01";
const RECEIPT_MAGIC: &[u8; 8] = b"RAIOBR01";
const SNAPSHOT_DOMAIN: &[u8] = b"raios.project_build_snapshot.v1";
const RECEIPT_DOMAIN: &[u8] = b"raios.project_build_receipt.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildSourceFile {
    pub path: String,
    pub byte_len: usize,
    pub blob_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildDependencyChunk {
    pub byte_len: usize,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildDependencyFile {
    pub path: String,
    pub whole_byte_len: usize,
    pub whole_sha256: [u8; 32],
    pub chunks: Vec<BuildDependencyChunk>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildDependencyBundle {
    pub bundle_sha256: [u8; 32],
    pub tree_sha256: [u8; 32],
    pub files: Vec<BuildDependencyFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectBuildSnapshot {
    pub project_id: ProjectId,
    pub project_revision_sha256: [u8; 32],
    pub source_tree_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub source_files: Vec<BuildSourceFile>,
    pub dependencies: Vec<BuildDependencyBundle>,
    pub input_manifest_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildRun {
    pub exit_code: i32,
    pub stdout_sha256: [u8; 32],
    pub stderr_sha256: [u8; 32],
    pub output_byte_len: usize,
    pub output_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectBuildReceipt {
    pub snapshot: ProjectBuildSnapshot,
    pub toolchain_manifest_sha256: [u8; 32],
    pub target: String,
    pub flags_contract_sha256: [u8; 32],
    pub environment_contract_sha256: [u8; 32],
    pub build_code_policy: String,
    pub toolchain_trust: String,
    pub run_one: BuildRun,
    pub run_two: BuildRun,
    pub candidate_sha256: [u8; 32],
    pub candidate_byte_len: usize,
    pub candidate_wasm_valid: bool,
    pub evidence_quality: String,
    pub host_build_attested: bool,
    pub reproducible: bool,
    pub independently_verified: bool,
    pub authorizes_promotion: bool,
    pub authorizes_install: bool,
    pub authorizes_load: bool,
    pub authorizes_execution: bool,
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectBuildError {
    EmptySource,
    MissingCargoToml,
    MissingCargoLock,
    DuplicateSource,
    DuplicateBundle,
    DuplicateDependencyFile,
    InvalidPath,
    InvalidLength,
    InvalidHash,
    TooManyEntries,
    ToolchainMismatch,
    BuildFailed,
    OutputMismatch,
    CandidateMismatch,
    ContractMismatch,
    InvalidEncoding,
}

impl ProjectBuildError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::EmptySource => "build_source_empty",
            Self::MissingCargoToml => "build_cargo_toml_missing",
            Self::MissingCargoLock => "build_cargo_lock_missing",
            Self::DuplicateSource => "build_source_path_collision",
            Self::DuplicateBundle => "build_dependency_bundle_duplicate",
            Self::DuplicateDependencyFile => "build_dependency_path_collision",
            Self::InvalidPath => "build_path_invalid",
            Self::InvalidLength => "build_length_invalid",
            Self::InvalidHash => "build_hash_invalid",
            Self::TooManyEntries => "build_input_quota_exceeded",
            Self::ToolchainMismatch => "build_toolchain_manifest_mismatch",
            Self::BuildFailed => "build_run_failed",
            Self::OutputMismatch => "build_outputs_not_reproducible",
            Self::CandidateMismatch => "build_candidate_mismatch",
            Self::ContractMismatch => "build_contract_mismatch",
            Self::InvalidEncoding => "build_receipt_invalid",
        }
    }
}

pub fn build_flags_contract_sha256() -> [u8; 32] {
    sha256_bytes(BUILD_FLAGS_CONTRACT.as_bytes())
}

pub fn build_environment_contract_sha256() -> [u8; 32] {
    sha256_bytes(BUILD_ENVIRONMENT_CONTRACT.as_bytes())
}

pub fn pinned_toolchain_manifest_sha256() -> [u8; 32] {
    sha256_bytes(PINNED_TOOLCHAIN_MANIFEST.as_bytes())
}

pub fn build_snapshot(
    project_id: ProjectId,
    project_revision_sha256: [u8; 32],
    source_tree_sha256: [u8; 32],
    cargo_lock_sha256: [u8; 32],
    mut source_files: Vec<BuildSourceFile>,
    mut dependencies: Vec<BuildDependencyBundle>,
) -> Result<ProjectBuildSnapshot, ProjectBuildError> {
    if source_files.is_empty() {
        return Err(ProjectBuildError::EmptySource);
    }
    source_files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    dependencies.sort_by_key(|item| item.bundle_sha256);
    validate_sources(&source_files, cargo_lock_sha256)?;
    validate_dependencies(&mut dependencies)?;
    require_hash(project_revision_sha256)?;
    require_hash(source_tree_sha256)?;
    require_hash(cargo_lock_sha256)?;
    let mut snapshot = ProjectBuildSnapshot {
        project_id,
        project_revision_sha256,
        source_tree_sha256,
        cargo_lock_sha256,
        source_files,
        dependencies,
        input_manifest_sha256: [0; 32],
    };
    snapshot.input_manifest_sha256 = snapshot_hash(&snapshot)?;
    Ok(snapshot)
}

pub fn build_receipt(
    snapshot: ProjectBuildSnapshot,
    toolchain_manifest_sha256: [u8; 32],
    run_one: BuildRun,
    run_two: BuildRun,
    candidate_sha256: [u8; 32],
    candidate_byte_len: usize,
    candidate_wasm_valid: bool,
) -> Result<ProjectBuildReceipt, ProjectBuildError> {
    validate_snapshot(&snapshot)?;
    if toolchain_manifest_sha256 != pinned_toolchain_manifest_sha256() {
        return Err(ProjectBuildError::ToolchainMismatch);
    }
    validate_run(&run_one)?;
    validate_run(&run_two)?;
    if run_one.exit_code != 0 || run_two.exit_code != 0 {
        return Err(ProjectBuildError::BuildFailed);
    }
    if run_one.output_sha256 != run_two.output_sha256
        || run_one.output_byte_len != run_two.output_byte_len
    {
        return Err(ProjectBuildError::OutputMismatch);
    }
    if !candidate_wasm_valid
        || candidate_sha256 != run_one.output_sha256
        || candidate_byte_len != run_one.output_byte_len
    {
        return Err(ProjectBuildError::CandidateMismatch);
    }
    let mut receipt = ProjectBuildReceipt {
        snapshot,
        toolchain_manifest_sha256,
        target: String::from(BUILD_TARGET),
        flags_contract_sha256: build_flags_contract_sha256(),
        environment_contract_sha256: build_environment_contract_sha256(),
        build_code_policy: String::from(BUILD_CODE_POLICY),
        toolchain_trust: String::from(PINNED_TOOLCHAIN_TRUST),
        run_one,
        run_two,
        candidate_sha256,
        candidate_byte_len,
        candidate_wasm_valid,
        evidence_quality: String::from(BUILD_EVIDENCE_QUALITY),
        host_build_attested: true,
        reproducible: true,
        independently_verified: false,
        authorizes_promotion: false,
        authorizes_install: false,
        authorizes_load: false,
        authorizes_execution: false,
        receipt_sha256: [0; 32],
    };
    receipt.receipt_sha256 = receipt_hash(&receipt)?;
    Ok(receipt)
}

pub fn encode_receipt(receipt: &ProjectBuildReceipt) -> Result<Vec<u8>, ProjectBuildError> {
    validate_receipt(receipt)?;
    let mut out = Vec::new();
    out.extend_from_slice(RECEIPT_MAGIC);
    encode_receipt_body(&mut out, receipt)?;
    out.extend_from_slice(&receipt.receipt_sha256);
    Ok(out)
}

pub fn decode_receipt(bytes: &[u8]) -> Result<ProjectBuildReceipt, ProjectBuildError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(8)? != RECEIPT_MAGIC {
        return Err(ProjectBuildError::InvalidEncoding);
    }
    let snapshot = decode_snapshot_body(&mut cursor)?;
    let toolchain_manifest_sha256 = cursor.hash()?;
    let target = cursor.string()?;
    let flags_contract_sha256 = cursor.hash()?;
    let environment_contract_sha256 = cursor.hash()?;
    let build_code_policy = cursor.string()?;
    let toolchain_trust = cursor.string()?;
    let run_one = decode_run(&mut cursor)?;
    let run_two = decode_run(&mut cursor)?;
    let candidate_sha256 = cursor.hash()?;
    let candidate_byte_len = cursor.u32()? as usize;
    let candidate_wasm_valid = cursor.boolean()?;
    let evidence_quality = cursor.string()?;
    let host_build_attested = cursor.boolean()?;
    let reproducible = cursor.boolean()?;
    let independently_verified = cursor.boolean()?;
    let authorizes_promotion = cursor.boolean()?;
    let authorizes_install = cursor.boolean()?;
    let authorizes_load = cursor.boolean()?;
    let authorizes_execution = cursor.boolean()?;
    let receipt_sha256 = cursor.hash()?;
    if !cursor.done() {
        return Err(ProjectBuildError::InvalidEncoding);
    }
    let receipt = ProjectBuildReceipt {
        snapshot,
        toolchain_manifest_sha256,
        target,
        flags_contract_sha256,
        environment_contract_sha256,
        build_code_policy,
        toolchain_trust,
        run_one,
        run_two,
        candidate_sha256,
        candidate_byte_len,
        candidate_wasm_valid,
        evidence_quality,
        host_build_attested,
        reproducible,
        independently_verified,
        authorizes_promotion,
        authorizes_install,
        authorizes_load,
        authorizes_execution,
        receipt_sha256,
    };
    validate_receipt(&receipt)?;
    Ok(receipt)
}

pub fn validate_build_receipt(receipt: &ProjectBuildReceipt) -> Result<(), ProjectBuildError> {
    validate_receipt(receipt)
}

fn validate_receipt(receipt: &ProjectBuildReceipt) -> Result<(), ProjectBuildError> {
    validate_snapshot(&receipt.snapshot)?;
    if receipt.toolchain_manifest_sha256 != pinned_toolchain_manifest_sha256() {
        return Err(ProjectBuildError::ToolchainMismatch);
    }
    if receipt.target != BUILD_TARGET
        || receipt.flags_contract_sha256 != build_flags_contract_sha256()
        || receipt.environment_contract_sha256 != build_environment_contract_sha256()
        || receipt.build_code_policy != BUILD_CODE_POLICY
        || receipt.toolchain_trust != PINNED_TOOLCHAIN_TRUST
        || receipt.evidence_quality != BUILD_EVIDENCE_QUALITY
        || !receipt.host_build_attested
        || !receipt.reproducible
        || receipt.authorizes_promotion
        || receipt.authorizes_install
        || receipt.authorizes_load
        || receipt.authorizes_execution
    {
        return Err(ProjectBuildError::ContractMismatch);
    }
    if receipt.run_one.exit_code != 0 || receipt.run_two.exit_code != 0 {
        return Err(ProjectBuildError::BuildFailed);
    }
    if receipt.run_one.output_sha256 != receipt.run_two.output_sha256
        || receipt.run_one.output_byte_len != receipt.run_two.output_byte_len
    {
        return Err(ProjectBuildError::OutputMismatch);
    }
    if !receipt.candidate_wasm_valid
        || receipt.candidate_sha256 != receipt.run_one.output_sha256
        || receipt.candidate_byte_len != receipt.run_one.output_byte_len
    {
        return Err(ProjectBuildError::CandidateMismatch);
    }
    if receipt.receipt_sha256 != receipt_hash(receipt)? {
        return Err(ProjectBuildError::InvalidHash);
    }
    Ok(())
}

fn validate_snapshot(snapshot: &ProjectBuildSnapshot) -> Result<(), ProjectBuildError> {
    let rebuilt = build_snapshot(
        snapshot.project_id,
        snapshot.project_revision_sha256,
        snapshot.source_tree_sha256,
        snapshot.cargo_lock_sha256,
        snapshot.source_files.clone(),
        snapshot.dependencies.clone(),
    )?;
    if rebuilt != *snapshot {
        return Err(ProjectBuildError::InvalidHash);
    }
    Ok(())
}

fn validate_sources(
    files: &[BuildSourceFile],
    cargo_lock_sha256: [u8; 32],
) -> Result<(), ProjectBuildError> {
    if files.len() > 32 {
        return Err(ProjectBuildError::TooManyEntries);
    }
    let mut cargo_toml = false;
    let mut cargo_lock = false;
    for (index, file) in files.iter().enumerate() {
        validate_path(&file.path)?;
        if index > 0 && aliases(&files[index - 1].path, &file.path) {
            return Err(ProjectBuildError::DuplicateSource);
        }
        if file.byte_len > 32 * 1024 {
            return Err(ProjectBuildError::InvalidLength);
        }
        require_hash(file.blob_sha256)?;
        cargo_toml |= file.path == "Cargo.toml";
        cargo_lock |= file.path == "Cargo.lock" && file.blob_sha256 == cargo_lock_sha256;
    }
    if !cargo_toml {
        return Err(ProjectBuildError::MissingCargoToml);
    }
    if !cargo_lock {
        return Err(ProjectBuildError::MissingCargoLock);
    }
    Ok(())
}

fn validate_dependencies(items: &mut [BuildDependencyBundle]) -> Result<(), ProjectBuildError> {
    if items.len() > 32 {
        return Err(ProjectBuildError::TooManyEntries);
    }
    for index in 0..items.len() {
        if index > 0 && items[index - 1].bundle_sha256 == items[index].bundle_sha256 {
            return Err(ProjectBuildError::DuplicateBundle);
        }
        require_hash(items[index].bundle_sha256)?;
        require_hash(items[index].tree_sha256)?;
        items[index]
            .files
            .sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
        if items[index].files.len() > 32 {
            return Err(ProjectBuildError::TooManyEntries);
        }
        for file_index in 0..items[index].files.len() {
            let file = &items[index].files[file_index];
            validate_path(&file.path)?;
            if file_index > 0 && aliases(&items[index].files[file_index - 1].path, &file.path) {
                return Err(ProjectBuildError::DuplicateDependencyFile);
            }
            require_hash(file.whole_sha256)?;
            if file.whole_byte_len > 512 * 1024 || file.chunks.len() > 64 {
                return Err(ProjectBuildError::InvalidLength);
            }
            let mut total = 0usize;
            for chunk in &file.chunks {
                require_hash(chunk.sha256)?;
                if chunk.byte_len == 0 || chunk.byte_len > 24 * 1024 {
                    return Err(ProjectBuildError::InvalidLength);
                }
                total = total
                    .checked_add(chunk.byte_len)
                    .ok_or(ProjectBuildError::InvalidLength)?;
            }
            if total != file.whole_byte_len {
                return Err(ProjectBuildError::InvalidLength);
            }
        }
    }
    Ok(())
}

fn validate_run(run: &BuildRun) -> Result<(), ProjectBuildError> {
    if run.output_byte_len == 0 || run.output_byte_len > 262_144 {
        return Err(ProjectBuildError::InvalidLength);
    }
    require_hash(run.stdout_sha256)?;
    require_hash(run.stderr_sha256)?;
    require_hash(run.output_sha256)
}

fn snapshot_hash(snapshot: &ProjectBuildSnapshot) -> Result<[u8; 32], ProjectBuildError> {
    let mut bytes = Vec::from(SNAPSHOT_DOMAIN);
    encode_snapshot_body(&mut bytes, snapshot)?;
    Ok(sha256_bytes(&bytes))
}

fn receipt_hash(receipt: &ProjectBuildReceipt) -> Result<[u8; 32], ProjectBuildError> {
    let mut bytes = Vec::from(RECEIPT_DOMAIN);
    encode_receipt_body(&mut bytes, receipt)?;
    Ok(sha256_bytes(&bytes))
}

fn encode_snapshot_body(
    out: &mut Vec<u8>,
    snapshot: &ProjectBuildSnapshot,
) -> Result<(), ProjectBuildError> {
    out.extend_from_slice(SNAPSHOT_MAGIC);
    out.extend_from_slice(&snapshot.project_id.bytes());
    out.extend_from_slice(&snapshot.project_revision_sha256);
    out.extend_from_slice(&snapshot.source_tree_sha256);
    out.extend_from_slice(&snapshot.cargo_lock_sha256);
    put_u16(out, snapshot.source_files.len())?;
    for file in &snapshot.source_files {
        put_string(out, &file.path)?;
        put_u32(out, file.byte_len)?;
        out.extend_from_slice(&file.blob_sha256);
    }
    put_u16(out, snapshot.dependencies.len())?;
    for dependency in &snapshot.dependencies {
        out.extend_from_slice(&dependency.bundle_sha256);
        out.extend_from_slice(&dependency.tree_sha256);
        put_u16(out, dependency.files.len())?;
        for file in &dependency.files {
            put_string(out, &file.path)?;
            put_u32(out, file.whole_byte_len)?;
            out.extend_from_slice(&file.whole_sha256);
            put_u16(out, file.chunks.len())?;
            for chunk in &file.chunks {
                put_u32(out, chunk.byte_len)?;
                out.extend_from_slice(&chunk.sha256);
            }
        }
    }
    out.extend_from_slice(&snapshot.input_manifest_sha256);
    Ok(())
}

fn encode_receipt_body(
    out: &mut Vec<u8>,
    receipt: &ProjectBuildReceipt,
) -> Result<(), ProjectBuildError> {
    encode_snapshot_body(out, &receipt.snapshot)?;
    out.extend_from_slice(&receipt.toolchain_manifest_sha256);
    put_string(out, &receipt.target)?;
    out.extend_from_slice(&receipt.flags_contract_sha256);
    out.extend_from_slice(&receipt.environment_contract_sha256);
    put_string(out, &receipt.build_code_policy)?;
    put_string(out, &receipt.toolchain_trust)?;
    encode_run(out, &receipt.run_one)?;
    encode_run(out, &receipt.run_two)?;
    out.extend_from_slice(&receipt.candidate_sha256);
    put_u32(out, receipt.candidate_byte_len)?;
    out.push(u8::from(receipt.candidate_wasm_valid));
    put_string(out, &receipt.evidence_quality)?;
    for value in [
        receipt.host_build_attested,
        receipt.reproducible,
        receipt.independently_verified,
        receipt.authorizes_promotion,
        receipt.authorizes_install,
        receipt.authorizes_load,
        receipt.authorizes_execution,
    ] {
        out.push(u8::from(value));
    }
    Ok(())
}

fn encode_run(out: &mut Vec<u8>, run: &BuildRun) -> Result<(), ProjectBuildError> {
    out.extend_from_slice(&run.exit_code.to_le_bytes());
    out.extend_from_slice(&run.stdout_sha256);
    out.extend_from_slice(&run.stderr_sha256);
    put_u32(out, run.output_byte_len)?;
    out.extend_from_slice(&run.output_sha256);
    Ok(())
}

fn decode_snapshot_body(
    cursor: &mut Cursor<'_>,
) -> Result<ProjectBuildSnapshot, ProjectBuildError> {
    if cursor.take(8)? != SNAPSHOT_MAGIC {
        return Err(ProjectBuildError::InvalidEncoding);
    }
    let project_id = ProjectId::new(cursor.array::<16>()?);
    let project_revision_sha256 = cursor.hash()?;
    let source_tree_sha256 = cursor.hash()?;
    let cargo_lock_sha256 = cursor.hash()?;
    let source_count = cursor.u16()? as usize;
    let mut source_files = Vec::with_capacity(source_count);
    for _ in 0..source_count {
        source_files.push(BuildSourceFile {
            path: cursor.string()?,
            byte_len: cursor.u32()? as usize,
            blob_sha256: cursor.hash()?,
        });
    }
    let dependency_count = cursor.u16()? as usize;
    let mut dependencies = Vec::with_capacity(dependency_count);
    for _ in 0..dependency_count {
        let bundle_sha256 = cursor.hash()?;
        let tree_sha256 = cursor.hash()?;
        let file_count = cursor.u16()? as usize;
        let mut files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let path = cursor.string()?;
            let whole_byte_len = cursor.u32()? as usize;
            let whole_sha256 = cursor.hash()?;
            let chunk_count = cursor.u16()? as usize;
            let mut chunks = Vec::with_capacity(chunk_count);
            for _ in 0..chunk_count {
                chunks.push(BuildDependencyChunk {
                    byte_len: cursor.u32()? as usize,
                    sha256: cursor.hash()?,
                });
            }
            files.push(BuildDependencyFile {
                path,
                whole_byte_len,
                whole_sha256,
                chunks,
            });
        }
        dependencies.push(BuildDependencyBundle {
            bundle_sha256,
            tree_sha256,
            files,
        });
    }
    let input_manifest_sha256 = cursor.hash()?;
    let snapshot = ProjectBuildSnapshot {
        project_id,
        project_revision_sha256,
        source_tree_sha256,
        cargo_lock_sha256,
        source_files,
        dependencies,
        input_manifest_sha256,
    };
    validate_snapshot(&snapshot)?;
    Ok(snapshot)
}

fn decode_run(cursor: &mut Cursor<'_>) -> Result<BuildRun, ProjectBuildError> {
    Ok(BuildRun {
        exit_code: i32::from_le_bytes(cursor.array::<4>()?),
        stdout_sha256: cursor.hash()?,
        stderr_sha256: cursor.hash()?,
        output_byte_len: cursor.u32()? as usize,
        output_sha256: cursor.hash()?,
    })
}

fn validate_path(value: &str) -> Result<(), ProjectBuildError> {
    if value.is_empty()
        || value.len() > 240
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains('\\')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        Err(ProjectBuildError::InvalidPath)
    } else {
        Ok(())
    }
}

fn aliases(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn require_hash(hash: [u8; 32]) -> Result<(), ProjectBuildError> {
    if hash == [0; 32] {
        Err(ProjectBuildError::InvalidHash)
    } else {
        Ok(())
    }
}

fn put_u16(out: &mut Vec<u8>, value: usize) -> Result<(), ProjectBuildError> {
    out.extend_from_slice(
        &u16::try_from(value)
            .map_err(|_| ProjectBuildError::TooManyEntries)?
            .to_le_bytes(),
    );
    Ok(())
}

fn put_u32(out: &mut Vec<u8>, value: usize) -> Result<(), ProjectBuildError> {
    out.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| ProjectBuildError::InvalidLength)?
            .to_le_bytes(),
    );
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), ProjectBuildError> {
    put_u16(out, value.len())?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8], ProjectBuildError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(ProjectBuildError::InvalidEncoding)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ProjectBuildError::InvalidEncoding)?;
        self.offset = end;
        Ok(value)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], ProjectBuildError> {
        self.take(N)?
            .try_into()
            .map_err(|_| ProjectBuildError::InvalidEncoding)
    }
    fn hash(&mut self) -> Result<[u8; 32], ProjectBuildError> {
        self.array::<32>()
    }
    fn u16(&mut self) -> Result<u16, ProjectBuildError> {
        Ok(u16::from_le_bytes(self.array::<2>()?))
    }
    fn u32(&mut self) -> Result<u32, ProjectBuildError> {
        Ok(u32::from_le_bytes(self.array::<4>()?))
    }
    fn string(&mut self) -> Result<String, ProjectBuildError> {
        let len = self.u16()? as usize;
        let value = core::str::from_utf8(self.take(len)?)
            .map_err(|_| ProjectBuildError::InvalidEncoding)?;
        Ok(String::from(value))
    }
    fn boolean(&mut self) -> Result<bool, ProjectBuildError> {
        match self.take(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ProjectBuildError::InvalidEncoding),
        }
    }
    fn done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ProjectBuildSnapshot {
        let chunk = BuildDependencyChunk {
            byte_len: 3,
            sha256: sha256_bytes(b"dep"),
        };
        build_snapshot(
            ProjectId::new([1; 16]),
            [2; 32],
            [3; 32],
            sha256_bytes(b"lock"),
            vec![
                BuildSourceFile {
                    path: String::from("src/lib.rs"),
                    byte_len: 3,
                    blob_sha256: sha256_bytes(b"src"),
                },
                BuildSourceFile {
                    path: String::from("Cargo.lock"),
                    byte_len: 4,
                    blob_sha256: sha256_bytes(b"lock"),
                },
                BuildSourceFile {
                    path: String::from("Cargo.toml"),
                    byte_len: 5,
                    blob_sha256: sha256_bytes(b"cargo"),
                },
            ],
            vec![BuildDependencyBundle {
                bundle_sha256: [4; 32],
                tree_sha256: [5; 32],
                files: vec![BuildDependencyFile {
                    path: String::from("src/lib.rs"),
                    whole_byte_len: 3,
                    whole_sha256: sha256_bytes(b"dep"),
                    chunks: vec![chunk],
                }],
            }],
        )
        .unwrap()
    }

    fn sample_run() -> BuildRun {
        BuildRun {
            exit_code: 0,
            stdout_sha256: sha256_bytes(b"stdout"),
            stderr_sha256: sha256_bytes(b"stderr"),
            output_byte_len: 8,
            output_sha256: sha256_bytes(b"\0asmtest"),
        }
    }

    #[test]
    fn receipt_roundtrip_is_deterministic_and_non_authorizing() {
        let run = sample_run();
        let receipt = build_receipt(
            sample_snapshot(),
            pinned_toolchain_manifest_sha256(),
            run,
            run,
            run.output_sha256,
            run.output_byte_len,
            true,
        )
        .unwrap();
        let bytes = encode_receipt(&receipt).unwrap();
        assert_eq!(decode_receipt(&bytes).unwrap(), receipt);
        assert_eq!(encode_receipt(&receipt).unwrap(), bytes);
        assert!(!receipt.independently_verified);
        assert!(!receipt.authorizes_promotion);
        assert!(!receipt.authorizes_install);
        assert!(!receipt.authorizes_load);
        assert!(!receipt.authorizes_execution);
    }

    #[test]
    fn snapshot_order_normalizes_and_duplicates_deny() {
        let snapshot = sample_snapshot();
        let mut reversed = snapshot.source_files.clone();
        reversed.reverse();
        let rebuilt = build_snapshot(
            snapshot.project_id,
            snapshot.project_revision_sha256,
            snapshot.source_tree_sha256,
            snapshot.cargo_lock_sha256,
            reversed,
            snapshot.dependencies.clone(),
        )
        .unwrap();
        assert_eq!(
            rebuilt.input_manifest_sha256,
            snapshot.input_manifest_sha256
        );
        let mut duplicate = snapshot.source_files.clone();
        duplicate.push(duplicate[0].clone());
        assert_eq!(
            build_snapshot(
                snapshot.project_id,
                snapshot.project_revision_sha256,
                snapshot.source_tree_sha256,
                snapshot.cargo_lock_sha256,
                duplicate,
                snapshot.dependencies.clone(),
            ),
            Err(ProjectBuildError::DuplicateSource)
        );
    }

    #[test]
    fn receipt_rejects_mutation_toolchain_and_output_mismatch() {
        let run = sample_run();
        assert_eq!(
            build_receipt(
                sample_snapshot(),
                [9; 32],
                run,
                run,
                run.output_sha256,
                8,
                true
            ),
            Err(ProjectBuildError::ToolchainMismatch)
        );
        let mut other = run;
        other.output_sha256 = [7; 32];
        assert_eq!(
            build_receipt(
                sample_snapshot(),
                pinned_toolchain_manifest_sha256(),
                run,
                other,
                run.output_sha256,
                8,
                true,
            ),
            Err(ProjectBuildError::OutputMismatch)
        );
        let receipt = build_receipt(
            sample_snapshot(),
            pinned_toolchain_manifest_sha256(),
            run,
            run,
            run.output_sha256,
            8,
            true,
        )
        .unwrap();
        let mut bytes = encode_receipt(&receipt).unwrap();
        bytes[20] ^= 1;
        assert!(decode_receipt(&bytes).is_err());
    }

    #[test]
    fn workstation_contract_hashes_are_pinned() {
        assert_eq!(
            pinned_toolchain_manifest_sha256(),
            crate::parse_sha256_ref(
                "45723cc79127b688781d83eb3b042825fc2501c76ac02e4f8fee9f2904589e36"
            )
            .unwrap()
        );
        assert_eq!(
            build_flags_contract_sha256(),
            crate::parse_sha256_ref(
                "34ae7dd2f0bfdc316745e7306377b5b6252451b5aca16559be8aa2058e6e0ed1"
            )
            .unwrap()
        );
        assert_eq!(
            build_environment_contract_sha256(),
            crate::parse_sha256_ref(
                "2e9fc15d9adabd8e7c2a59b108ed91a70306e593ca08ad7475cba2b8465f79dc"
            )
            .unwrap()
        );
    }
}
