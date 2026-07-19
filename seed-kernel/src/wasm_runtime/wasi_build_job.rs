use alloc::{
    boxed::Box,
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::cell::RefCell;

use raios_core::{
    authorized_build_job::{AuthorizedBuildJob, AuthorizedBuildJobRequest, BuildJobDenied},
    build_guest_class::{BuildGuestClassV1, RUSTC_BUILD_GUEST_CLASS_V1},
    build_storage_authority::{
        authorize_build_storage, evaluate_scoped_build_output_commit, BuildOutputCommitDenied,
        BuildRunNonce, BuildStorageAuthority, OutputLeaseDescriptorV1,
        ScopedBuildOutputCommitDecision, ScopedBuildOutputCommitInput,
        BUILD_OUTPUT_LEASE_TARGET_MARKER_V1, RUSTC_BUILD_MOUNT_BUDGET_V1,
    },
    buildfs_manifest::{
        BuildFsChunk, BuildFsDirectory, BuildFsFile, BuildFsManifest, BUILD_FS_CHUNK_SIZE,
        BUILD_FS_MANIFEST_V1,
    },
    parse_sha256_ref,
    scoped_wasi_artifact_egress::{
        evaluate_scoped_wasi_artifact_egress, ScopedWasiArtifactEgress,
        ScopedWasiArtifactEgressDecision, WasiArtifactEgressPlan,
    },
    scoped_wasi_build_grant::{ScopedWasiBuildGrant, RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256},
    sha256_bytes,
    wasi_build_output::FrozenOutput,
    wasi_preview1_import_abi::{
        WasiImportDeclaration, WasiImportKind, WasiValueType, RUSTC_WASM_C6DCCF3E_IMPORTS,
    },
};
use raios_wasi_preview1::{
    ramfs::RamQuotas, ChunkRead, ChunkReadError, ChunkReadRequest, JobContext, NormalizedPath,
    WasiBuildInstance, WasiBuildLimits, SYSROOT_MOUNT,
};
use wasmi::{
    core::{Trap, ValueType},
    errors::{MemoryError, TableError},
    Config, Engine, ExternType, Linker, Memory, MemoryType, Module, Mutability, ResourceLimiter,
    ResumableCall, Store, Suspension,
};

use super::{
    wasi_build_storage::{
        materialize_build_storage, project_core_manifest, BuildChunkHandle, BuildChunkStore,
        BuildChunkStoreError, GrantedChunkReadDenied, GrantedChunkReader, UnbackedChunkStore,
    },
    wasi_preview1::{define_wasi_imports, ProcExitTrap, WasiHostState},
};

const WASI_BUILD_OK_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_ok.wasm"));
const WASI_BUILD_EXTRA_IMPORT_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_extra_import.wasm"));
const WASI_MEM_GROW_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasi_mem_grow.wasm"));
const WASI_MEM_OVER_CLASS_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_mem_over_class.wasm"));
const COMPILER_ARTIFACT_SHA256: &str =
    "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
const JOB_MANIFEST_SHA256: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
// docs/architecture/sysroot-buildfs-manifest-13daf6f9.md
const SYSROOT_BUILD_FS_MANIFEST_SHA256: &str =
    "13daf6f9042d07c4d698d60ea16869ed85e2035f762f4b5a048e71e7523b7b15";
const SYSROOT_BUILD_FS_MANIFEST_LEN: u64 = 51_089;
const SYSIMPORT_SAMPLE_TARGET: usize = 32;
const SYSIMPORT_RUN_NONCE: u64 = 301;
const REQUIRED_IMPORT_COUNT: usize = 30;
const BUILD_STORE_INSTANCE_ID: u64 = 11;
const BUILD_STORE_GENERATION: u64 = 13;
const BUILD_OUTPUT_LEASE_ID: u64 = 7;
const MEMORY_GROW_STEP_PAGES: u32 = 115;
const MEMORY_GROW_SAFETY_MARGIN_BYTES: usize = 1024 * 1024;
const MEMORY_CONTROL_BYTES: usize = 28;
const MEMORY_INITIAL_PAGES_OFFSET: usize = 0;
const MEMORY_FINAL_PAGES_OFFSET: usize = 4;
const MEMORY_GROW_STEP_COUNT_OFFSET: usize = 8;
const MEMORY_GROW_DENIED_OFFSET: usize = 12;
const MEMORY_STDOUT_IOVEC_PTR_OFFSET: usize = 16;
const MEMORY_STDOUT_IOVEC_LEN_OFFSET: usize = 20;
const MEMORY_STDOUT_WRITTEN_OFFSET: usize = 24;
const MEMORY_DECIMAL_CAPACITY: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WasiJobEnd {
    ProcExit(u32),
    Trap,
    Denied(BuildJobDenied),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WasiMemoryEvidence {
    current_pages: u32,
    initial_pages: u32,
    final_pages: u32,
    grow_step_count: u32,
    reported_pages: Option<u32>,
    grow_denied: bool,
    stdout: [u8; MEMORY_DECIMAL_CAPACITY],
    stdout_len: usize,
}

impl WasiMemoryEvidence {
    const fn missing() -> Self {
        Self {
            current_pages: 0,
            initial_pages: 0,
            final_pages: 0,
            grow_step_count: 0,
            reported_pages: None,
            grow_denied: false,
            stdout: [0; MEMORY_DECIMAL_CAPACITY],
            stdout_len: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WasiJobEvidence {
    end: WasiJobEnd,
    instantiated: bool,
    stdout_bytes: u64,
    registered: usize,
    frozen_output_entries: usize,
    frozen_output: Option<FrozenOutput>,
    output_bundle_len: u64,
    memory: WasiMemoryEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SysimportEvidence {
    manifest: &'static str,
    chunks: usize,
    deny: &'static str,
    // Diagnostic-only: inner chunk-store error variant and the anchored
    // manifest chunk index for a materialization failure ("none"/u64::MAX
    // otherwise). Emitted so a red run pinpoints the failing chunk.
    detail: &'static str,
    at: u64,
    passed: bool,
}

impl SysimportEvidence {
    const fn failed(manifest: &'static str, chunks: usize, deny: &'static str) -> Self {
        Self {
            manifest,
            chunks,
            deny,
            detail: "none",
            at: u64::MAX,
            passed: false,
        }
    }

    const fn failed_materialize(
        deny: &'static str,
        detail: &'static str,
        at: u64,
    ) -> Self {
        Self {
            manifest: "ok",
            chunks: 0,
            deny,
            detail,
            at,
            passed: false,
        }
    }
}

impl WasiJobEvidence {
    const fn terminal(end: WasiJobEnd) -> Self {
        Self {
            end,
            instantiated: false,
            stdout_bytes: 0,
            registered: 0,
            frozen_output_entries: 0,
            frozen_output: None,
            output_bundle_len: 0,
            memory: WasiMemoryEvidence::missing(),
        }
    }
}

enum ObservedKind {
    Func {
        params: Vec<WasiValueType>,
        results: Vec<WasiValueType>,
    },
    Memory {
        initial: u64,
        maximum: Option<u64>,
        shared: bool,
    },
    Global {
        value_type: WasiValueType,
        mutable: bool,
    },
    Table {
        element_type: WasiValueType,
        initial: u64,
        maximum: Option<u64>,
    },
}

struct ObservedImport<'module> {
    module: &'module str,
    name: &'module str,
    kind: ObservedKind,
}

impl ObservedImport<'_> {
    fn declaration(&self) -> WasiImportDeclaration<'_> {
        let kind = match &self.kind {
            ObservedKind::Func { params, results } => WasiImportKind::Func { params, results },
            ObservedKind::Memory {
                initial,
                maximum,
                shared,
            } => WasiImportKind::Memory {
                initial: *initial,
                maximum: *maximum,
                shared: *shared,
            },
            ObservedKind::Global {
                value_type,
                mutable,
            } => WasiImportKind::Global {
                value_type: *value_type,
                mutable: *mutable,
            },
            ObservedKind::Table {
                element_type,
                initial,
                maximum,
            } => WasiImportKind::Table {
                element_type: *element_type,
                initial: *initial,
                maximum: *maximum,
            },
        };
        WasiImportDeclaration {
            module: self.module,
            name: self.name,
            kind,
        }
    }
}

fn wasi_engine() -> Engine {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    Engine::new(&config)
}

impl ResourceLimiter for WasiHostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, MemoryError> {
        if maximum.is_some_and(|maximum| desired > maximum) {
            return Ok(false);
        }
        // Vec::resize reserves max(2*capacity, desired) as a NEW allocation while
        // the current buffer stays live — and the current buffer is already inside
        // ALLOCATOR used, i.e. excluded from free(). So admission gates on the new
        // reservation only, plus 1 MiB for fragmentation. The observed 1.56-GiB
        // allocation (Layout size 1673527296 = 2x the ~0.78-GiB capacity) pins the
        // doubling model; counting `current` here too would double-book it against
        // free() and wrongly deny the final grow to the declared maximum.
        let Some(required_free) = current
            .checked_mul(2)
            .map(|doubled| doubled.max(desired))
            .and_then(|reservation| reservation.checked_add(MEMORY_GROW_SAFETY_MARGIN_BYTES))
        else {
            return Ok(false);
        };
        Ok(required_free <= crate::ALLOCATOR.lock().free())
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        maximum: Option<u32>,
    ) -> Result<bool, TableError> {
        Ok(maximum.is_none_or(|maximum| desired <= maximum))
    }
}

fn run_build_job(bytes: &[u8], nonce_value: u64) -> WasiJobEvidence {
    let engine = wasi_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    let sysroot_manifest = match empty_manifest() {
        Some(manifest) => manifest,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let src_manifest = match empty_manifest() {
        Some(manifest) => manifest,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let sysroot_mount_hash = match sysroot_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let src_mount_hash = match src_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let request = AuthorizedBuildJobRequest {
        wasi_grant: ScopedWasiBuildGrant {
            compiler_artifact_sha256: COMPILER_ARTIFACT_SHA256,
            job_manifest_sha256: JOB_MANIFEST_SHA256,
            inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
            declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
        },
        observed_imports: &declarations,
        guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
        sysroot_mount_manifest_sha256: sysroot_mount_hash,
        src_mount_manifest_sha256: src_mount_hash,
    };
    let authorized = match AuthorizedBuildJob::authorize(request) {
        Ok(authorized) => authorized,
        Err(denied) => return WasiJobEvidence::terminal(WasiJobEnd::Denied(denied)),
    };
    let authority = match storage_authority(&authorized, &sysroot_manifest, &src_manifest) {
        Some(authority) => authority,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let nonce = match BuildRunNonce::kernel_minted(nonce_value) {
        Some(nonce) => nonce,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let reader = match materialize_build_storage(
        &authority,
        &sysroot_manifest,
        &src_manifest,
        nonce,
        Box::new(UnbackedChunkStore::new(
            BUILD_STORE_INSTANCE_ID,
            BUILD_STORE_GENERATION,
        )),
    ) {
        Ok(reader) => reader,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    if reader.entry_count() != 0
        || reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != nonce_value
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return WasiJobEvidence::terminal(WasiJobEnd::Trap);
    }
    instantiate_authorized(
        engine,
        module,
        authorized,
        &sysroot_manifest,
        &src_manifest,
        reader,
    )
}

fn instantiate_authorized(
    engine: Engine,
    module: Module,
    authorized: AuthorizedBuildJob,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
    chunk_reader: GrantedChunkReader,
) -> WasiJobEvidence {
    let class = *authorized.guest_class();
    let instance = match build_instance(&authorized, class, sysroot_manifest, src_manifest) {
        Some(instance) => instance,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let mut store = Store::new(&engine, WasiHostState::new(instance, chunk_reader));
    store.limiter(|state| state);
    let memory_type = match MemoryType::new_shared(
        class.shared_memory.initial_pages,
        class.shared_memory.max_pages,
    ) {
        Ok(memory_type) => memory_type,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let memory = match Memory::new(&mut store, memory_type) {
        Ok(memory) => memory,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    store.data_mut().install_memory(memory);
    let mut linker = Linker::<WasiHostState>::new(&engine);
    let registered = match define_wasi_imports(&mut linker, memory) {
        Ok(registered) => registered,
        Err(()) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    if registered != REQUIRED_IMPORT_COUNT
        || registered != RUSTC_WASM_C6DCCF3E_IMPORTS.len()
        || registered != authorized.authorized_import_count()
    {
        return WasiJobEvidence::terminal(WasiJobEnd::Trap);
    }
    let pre = match linker.instantiate(&mut store, &module) {
        Ok(pre) => pre,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let wasm_instance = match pre.start(&mut store) {
        Ok(instance) => instance,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let start = match wasm_instance.get_func(&store, "_start") {
        Some(start) => start,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let ty = start.ty(&store);
    if !ty.params().is_empty() || !ty.results().is_empty() {
        return WasiJobEvidence::terminal(WasiJobEnd::Trap);
    }
    let end = run_start(&mut store, start, class);
    let memory_evidence = inspect_memory_evidence(memory, &store);
    let (end, frozen_output_entries, frozen_output, output_bundle_len) =
        match store.data().freeze_output_evidence() {
            Ok((entries, output, bundle_len)) => (end, entries, Some(output), bundle_len),
            Err(_) => (WasiJobEnd::Trap, 0, None, 0),
        };
    WasiJobEvidence {
        end,
        instantiated: true,
        stdout_bytes: store.data().stdout_bytes(),
        registered,
        frozen_output_entries,
        frozen_output,
        output_bundle_len,
        memory: memory_evidence,
    }
}

fn inspect_memory_evidence(memory: Memory, store: &Store<WasiHostState>) -> WasiMemoryEvidence {
    let current_pages = u32::from(memory.current_pages(store));
    let mut evidence = WasiMemoryEvidence {
        current_pages,
        ..WasiMemoryEvidence::missing()
    };
    let mut control = [0u8; MEMORY_CONTROL_BYTES];
    if memory.read(store, 0, &mut control).is_err() {
        return evidence;
    }
    evidence.initial_pages = read_control_u32(&control, MEMORY_INITIAL_PAGES_OFFSET);
    evidence.final_pages = read_control_u32(&control, MEMORY_FINAL_PAGES_OFFSET);
    evidence.grow_step_count = read_control_u32(&control, MEMORY_GROW_STEP_COUNT_OFFSET);
    evidence.grow_denied = read_control_u32(&control, MEMORY_GROW_DENIED_OFFSET) == 1;

    let stdout_ptr = read_control_u32(&control, MEMORY_STDOUT_IOVEC_PTR_OFFSET) as usize;
    let stdout_len = read_control_u32(&control, MEMORY_STDOUT_IOVEC_LEN_OFFSET) as usize;
    let stdout_written = read_control_u32(&control, MEMORY_STDOUT_WRITTEN_OFFSET) as usize;
    if stdout_len == 0
        || stdout_len > MEMORY_DECIMAL_CAPACITY
        || stdout_written != stdout_len
        || stdout_ptr.checked_add(stdout_len).is_none()
    {
        return evidence;
    }
    if memory
        .read(store, stdout_ptr, &mut evidence.stdout[..stdout_len])
        .is_err()
    {
        return evidence;
    }
    evidence.stdout_len = stdout_len;
    evidence.reported_pages = parse_decimal_u32(&evidence.stdout[..stdout_len]);
    evidence
}

fn read_control_u32(control: &[u8; MEMORY_CONTROL_BYTES], offset: usize) -> u32 {
    u32::from_le_bytes([
        control[offset],
        control[offset + 1],
        control[offset + 2],
        control[offset + 3],
    ])
}

fn parse_decimal_u32(digits: &[u8]) -> Option<u32> {
    let mut value = 0u32;
    for digit in digits {
        if !digit.is_ascii_digit() {
            return None;
        }
        value = value
            .checked_mul(10)?
            .checked_add(u32::from(*digit - b'0'))?;
    }
    Some(value)
}

fn run_start(
    store: &mut Store<WasiHostState>,
    start: wasmi::Func,
    class: BuildGuestClassV1,
) -> WasiJobEnd {
    let mut granted_total = 0;
    if add_next_quantum(store, class, &mut granted_total).is_err() {
        return WasiJobEnd::Trap;
    }
    let mut outputs = [];
    let mut outcome = match start.call_resumable(&mut *store, &[], &mut outputs) {
        Ok(outcome) => outcome,
        Err(_) => return WasiJobEnd::Trap,
    };
    loop {
        let invocation = match outcome {
            ResumableCall::Finished => return WasiJobEnd::Trap,
            ResumableCall::Resumable(invocation) => invocation,
        };
        match invocation.suspension() {
            Suspension::FuelQuantum => {
                if add_next_quantum(store, class, &mut granted_total).is_err() {
                    return WasiJobEnd::Trap;
                }
                outcome = match invocation.resume(&mut *store, &[], &mut outputs) {
                    Ok(outcome) => outcome,
                    Err(_) => return WasiJobEnd::Trap,
                };
            }
            Suspension::Host { host_error, .. }
                if host_error.downcast_ref::<ProcExitTrap>().is_some() =>
            {
                return match store.data().terminal_exit() {
                    Some(code) => WasiJobEnd::ProcExit(code),
                    None => WasiJobEnd::Trap,
                };
            }
            Suspension::Host { .. } | Suspension::Atomic(_) => return WasiJobEnd::Trap,
        }
    }
}

fn add_next_quantum(
    store: &mut Store<WasiHostState>,
    class: BuildGuestClassV1,
    granted_total: &mut u64,
) -> Result<(), Trap> {
    let remaining = class
        .max_total_fuel
        .checked_sub(*granted_total)
        .filter(|remaining| *remaining != 0)
        .ok_or_else(|| Trap::new("WASI build fuel ceiling reached"))?;
    let grant = class.fuel_quantum.min(remaining);
    store
        .add_fuel(grant)
        .map_err(|_| Trap::new("WASI build fuel refill failed"))?;
    *granted_total = granted_total
        .checked_add(grant)
        .ok_or_else(|| Trap::new("WASI build fuel ceiling reached"))?;
    Ok(())
}

fn build_instance(
    authorized: &AuthorizedBuildJob,
    class: BuildGuestClassV1,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
) -> Option<WasiBuildInstance> {
    let sysroot = project_core_manifest(sysroot_manifest).ok()?;
    let source = project_core_manifest(src_manifest).ok()?;
    let job = JobContext::new(Vec::new(), Vec::new(), authorized.job_manifest_sha256()).ok()?;
    let max_files = u64::from(class.max_files_per_arena);
    let max_directories = u64::from(class.max_dirs_per_arena);
    let scratch = RamQuotas::new(
        class.tmp_max_bytes,
        max_files,
        max_directories,
        class.max_file_bytes,
    );
    let output = RamQuotas::new(
        class.out_max_bytes,
        max_files,
        max_directories,
        class.max_file_bytes,
    );
    let max_open_fds = class.max_files_per_arena.checked_add(FdFloor::COUNT)?;
    WasiBuildInstance::new(
        sysroot,
        source,
        job,
        WasiBuildLimits::new(max_open_fds, scratch, output),
    )
    .ok()
}

struct FdFloor;

impl FdFloor {
    const COUNT: u32 = 4;
}

fn empty_manifest() -> Option<BuildFsManifest> {
    BuildFsManifest::new(Vec::new(), Vec::new()).ok()
}

struct BuildFsManifestCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BuildFsManifestCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], &'static str> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or("manifest_parse_failed")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("manifest_parse_failed")?;
        self.offset = end;
        Ok(value)
    }

    fn take_u64(&mut self) -> Result<u64, &'static str> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| "manifest_parse_failed")?,
        ))
    }

    fn take_len(&mut self) -> Result<usize, &'static str> {
        usize::try_from(self.take_u64()?).map_err(|_| "manifest_parse_failed")
    }

    fn take_bytes(&mut self) -> Result<&'a [u8], &'static str> {
        let len = self.take_len()?;
        self.take(len)
    }

    fn take_string(&mut self) -> Result<String, &'static str> {
        let bytes = self.take_bytes()?;
        let value = core::str::from_utf8(bytes).map_err(|_| "manifest_parse_failed")?;
        Ok(value.to_string())
    }

    fn take_sha256(&mut self) -> Result<[u8; 32], &'static str> {
        self.take(32)?
            .try_into()
            .map_err(|_| "manifest_parse_failed")
    }

    const fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

/// Decodes the transport bytes into core's authority type, then requires core
/// validation and byte-for-byte canonical re-encoding before use.
fn parse_canonical_buildfs_manifest(bytes: &[u8]) -> Result<BuildFsManifest, &'static str> {
    let mut cursor = BuildFsManifestCursor::new(bytes);
    if cursor.take_bytes()? != BUILD_FS_MANIFEST_V1.as_bytes()
        || cursor.take_u64()? != BUILD_FS_CHUNK_SIZE
    {
        return Err("manifest_parse_failed");
    }

    let directory_count = cursor.take_len()?;
    if directory_count > bytes.len() {
        return Err("manifest_parse_failed");
    }
    let mut directories = Vec::new();
    for _ in 0..directory_count {
        directories.push(BuildFsDirectory {
            path: cursor.take_string()?,
        });
    }

    let file_count = cursor.take_len()?;
    if file_count > bytes.len() {
        return Err("manifest_parse_failed");
    }
    let mut files = Vec::new();
    for _ in 0..file_count {
        let path = cursor.take_string()?;
        let len = cursor.take_u64()?;
        let sha256 = cursor.take_sha256()?;
        let chunk_count = cursor.take_len()?;
        if chunk_count > bytes.len() {
            return Err("manifest_parse_failed");
        }
        let mut chunks = Vec::new();
        for _ in 0..chunk_count {
            chunks.push(BuildFsChunk {
                len: cursor.take_u64()?,
                sha256: cursor.take_sha256()?,
            });
        }
        files.push(BuildFsFile {
            path,
            len,
            sha256,
            chunks,
        });
    }
    if !cursor.finished() {
        return Err("manifest_parse_failed");
    }
    let manifest = BuildFsManifest { directories, files };
    manifest.validate().map_err(|_| "manifest_invalid")?;
    let canonical = manifest.canonical_bytes().map_err(|_| "manifest_invalid")?;
    if canonical.as_slice() != bytes {
        return Err("manifest_noncanonical");
    }
    Ok(manifest)
}

fn storage_authority(
    authorized: &AuthorizedBuildJob,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
) -> Option<BuildStorageAuthority> {
    authorize_build_storage(
        authorized,
        sysroot_manifest,
        src_manifest,
        RUSTC_BUILD_MOUNT_BUDGET_V1,
        OutputLeaseDescriptorV1::kernel_minted(
            BUILD_OUTPUT_LEASE_ID,
            BUILD_STORE_INSTANCE_ID,
            BUILD_STORE_GENERATION,
            RUSTC_BUILD_MOUNT_BUDGET_V1.max_output_bytes,
            BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
        ),
    )
    .ok()
}

fn observed_imports(module: &Module) -> Vec<ObservedImport<'_>> {
    module
        .imports()
        .map(|import| ObservedImport {
            module: import.module(),
            name: import.name(),
            kind: match import.ty() {
                ExternType::Func(function) => ObservedKind::Func {
                    params: function.params().iter().copied().map(value_type).collect(),
                    results: function.results().iter().copied().map(value_type).collect(),
                },
                ExternType::Memory(memory) => ObservedKind::Memory {
                    initial: u64::from(u32::from(memory.initial_pages())),
                    maximum: memory.maximum_pages().map(u32::from).map(u64::from),
                    shared: memory.is_shared(),
                },
                ExternType::Global(global) => ObservedKind::Global {
                    value_type: value_type(global.content()),
                    mutable: global.mutability() == Mutability::Var,
                },
                ExternType::Table(table) => ObservedKind::Table {
                    element_type: value_type(table.element()),
                    initial: u64::from(table.minimum()),
                    maximum: table.maximum().map(u64::from),
                },
            },
        })
        .collect()
}

fn value_type(value: ValueType) -> WasiValueType {
    match value {
        ValueType::I32 => WasiValueType::I32,
        ValueType::I64 => WasiValueType::I64,
        ValueType::F32 => WasiValueType::F32,
        ValueType::F64 => WasiValueType::F64,
        ValueType::FuncRef => WasiValueType::FuncRef,
        ValueType::ExternRef => WasiValueType::ExternRef,
    }
}

#[derive(Clone, Copy)]
struct StorageSelftestEvidence {
    materialize: &'static str,
    grant_read: &'static str,
    out_of_grant: &'static str,
    wrong_range: &'static str,
    tamper: &'static str,
}

impl StorageSelftestEvidence {
    const fn failed(materialize: &'static str) -> Self {
        Self {
            materialize,
            grant_read: "missing",
            out_of_grant: "missing",
            wrong_range: "missing",
            tamper: "missing",
        }
    }
}

#[derive(Clone, Copy)]
struct EgressCommitSelftestEvidence {
    egress: &'static str,
    commit: &'static str,
    commit_deny: &'static str,
}

impl EgressCommitSelftestEvidence {
    const fn failed(egress: &'static str) -> Self {
        Self {
            egress,
            commit: "missing",
            commit_deny: "missing",
        }
    }
}

struct RamChunkFrame {
    bytes: Vec<u8>,
}

struct RamChunkStoreState {
    frames: Vec<RamChunkFrame>,
}

struct RamChunkStore {
    state: Rc<RefCell<RamChunkStoreState>>,
}

impl BuildChunkStore for RamChunkStore {
    fn store_instance_id(&self) -> u64 {
        BUILD_STORE_INSTANCE_ID
    }

    fn store_generation(&self) -> u64 {
        BUILD_STORE_GENERATION
    }

    fn resolve_chunk(
        &mut self,
        sha256: [u8; 32],
        len: u64,
    ) -> Result<BuildChunkHandle, BuildChunkStoreError> {
        let state = self.state.borrow();
        let Some((index, frame)) = state.frames.iter().enumerate().find(|(_, frame)| {
            u64::try_from(frame.bytes.len()).ok() == Some(len)
                && sha256_bytes(&frame.bytes) == sha256
        }) else {
            return Err(BuildChunkStoreError::Missing);
        };
        Ok(BuildChunkHandle {
            store_generation: BUILD_STORE_GENERATION,
            frame_offset: u64::try_from(index).map_err(|_| BuildChunkStoreError::Bounds)?,
            frame_len: len,
            payload_len: len,
            payload_sha256: sha256,
            frame_sha256: sha256_bytes(&frame.bytes),
        })
    }

    fn read_resolved_chunk(
        &mut self,
        handle: BuildChunkHandle,
        destination: &mut [u8],
    ) -> Result<(), BuildChunkStoreError> {
        let index =
            usize::try_from(handle.frame_offset).map_err(|_| BuildChunkStoreError::Bounds)?;
        let state = self.state.borrow();
        let frame = state
            .frames
            .get(index)
            .ok_or(BuildChunkStoreError::Missing)?;
        if frame.bytes.len() != destination.len() {
            return Err(BuildChunkStoreError::Bounds);
        }
        destination.copy_from_slice(&frame.bytes);
        Ok(())
    }
}

fn synthetic_storage_manifests() -> Option<(
    BuildFsManifest,
    BuildFsManifest,
    Rc<RefCell<RamChunkStoreState>>,
)> {
    let first_chunk = vec![0x41; BUILD_FS_CHUNK_SIZE as usize];
    let second_chunk = b"tail".to_vec();
    let source_chunk = b"fn main() {}".to_vec();
    let mut sysroot_file = first_chunk.clone();
    sysroot_file.extend_from_slice(&second_chunk);
    let sysroot = BuildFsManifest::new(
        vec![BuildFsDirectory {
            path: "lib".to_string(),
        }],
        vec![BuildFsFile {
            path: "lib/multi".to_string(),
            len: u64::try_from(sysroot_file.len()).ok()?,
            sha256: sha256_bytes(&sysroot_file),
            chunks: vec![
                BuildFsChunk {
                    len: u64::try_from(first_chunk.len()).ok()?,
                    sha256: sha256_bytes(&first_chunk),
                },
                BuildFsChunk {
                    len: u64::try_from(second_chunk.len()).ok()?,
                    sha256: sha256_bytes(&second_chunk),
                },
            ],
        }],
    )
    .ok()?;
    let src = BuildFsManifest::new(
        vec![BuildFsDirectory {
            path: "project".to_string(),
        }],
        vec![BuildFsFile {
            path: "project/main.rs".to_string(),
            len: u64::try_from(source_chunk.len()).ok()?,
            sha256: sha256_bytes(&source_chunk),
            chunks: vec![BuildFsChunk {
                len: u64::try_from(source_chunk.len()).ok()?,
                sha256: sha256_bytes(&source_chunk),
            }],
        }],
    )
    .ok()?;
    let state = Rc::new(RefCell::new(RamChunkStoreState {
        frames: vec![
            RamChunkFrame { bytes: first_chunk },
            RamChunkFrame {
                bytes: second_chunk,
            },
            RamChunkFrame {
                bytes: source_chunk,
            },
        ],
    }));
    Some((sysroot, src, state))
}

fn authorize_for_manifests(
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
) -> Option<AuthorizedBuildJob> {
    AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
        wasi_grant: ScopedWasiBuildGrant {
            compiler_artifact_sha256: COMPILER_ARTIFACT_SHA256,
            job_manifest_sha256: JOB_MANIFEST_SHA256,
            inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
            declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
        },
        observed_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
        guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
        sysroot_mount_manifest_sha256: sysroot_manifest.sha256().ok()?,
        src_mount_manifest_sha256: src_manifest.sha256().ok()?,
    })
    .ok()
}

fn storage_selftest() -> StorageSelftestEvidence {
    let Some((sysroot_manifest, src_manifest, state)) = synthetic_storage_manifests() else {
        return StorageSelftestEvidence::failed("fixture_failed");
    };
    let Some(job) = authorize_for_manifests(&sysroot_manifest, &src_manifest) else {
        return StorageSelftestEvidence::failed("authority_failed");
    };
    let Some(authority) = storage_authority(&job, &sysroot_manifest, &src_manifest) else {
        return StorageSelftestEvidence::failed("authority_failed");
    };
    let Some(nonce) = BuildRunNonce::kernel_minted(101) else {
        return StorageSelftestEvidence::failed("nonce_failed");
    };
    let mut reader = match materialize_build_storage(
        &authority,
        &sysroot_manifest,
        &src_manifest,
        nonce,
        Box::new(RamChunkStore {
            state: Rc::clone(&state),
        }),
    ) {
        Ok(reader) => reader,
        Err(error) => return StorageSelftestEvidence::failed(error.reason()),
    };
    if reader.entry_count() != 3
        || reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != 101
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return StorageSelftestEvidence::failed("table_binding_mismatch");
    }

    let Some(file) = sysroot_manifest.files.first() else {
        return StorageSelftestEvidence::failed("fixture_failed");
    };
    let Some(chunk) = file.chunks.first() else {
        return StorageSelftestEvidence::failed("fixture_failed");
    };
    let buildfs = match project_core_manifest(&sysroot_manifest) {
        Ok(buildfs) => buildfs,
        Err(error) => return StorageSelftestEvidence::failed(error.reason()),
    };
    let path = match NormalizedPath::root().resolve(file.path.as_bytes()) {
        Ok(path) => path,
        Err(_) => return StorageSelftestEvidence::failed("fixture_failed"),
    };
    let file_node = match buildfs.node_for_path(&path) {
        Ok(node) => node,
        Err(_) => return StorageSelftestEvidence::failed("fixture_failed"),
    };
    let request = ChunkReadRequest {
        mount_id: SYSROOT_MOUNT,
        file: file_node,
        file_sha256: file.sha256,
        chunk_index: 0,
        chunk_sha256: chunk.sha256,
        range_offset: 0,
        range_len: chunk.len,
    };
    let mut bytes = vec![0; chunk.len as usize];
    let grant_read =
        if reader.read_chunk(request, &mut bytes).is_ok() && sha256_bytes(&bytes) == chunk.sha256 {
            "ok"
        } else {
            "failed"
        };

    let mut absent = request;
    absent.chunk_index = 99;
    let out_of_grant = if reader.read_chunk(absent, &mut bytes) == Err(ChunkReadError::NotCapable)
        && reader.last_denial() == Some(GrantedChunkReadDenied::AbsentEntry)
    {
        GrantedChunkReadDenied::AbsentEntry.reason()
    } else {
        "failed"
    };

    let mut shortened = request;
    shortened.range_len -= 1;
    let shortened_len = match usize::try_from(shortened.range_len) {
        Ok(len) => len,
        Err(_) => return StorageSelftestEvidence::failed("fixture_failed"),
    };
    let mut shortened_bytes = vec![0; shortened_len];
    let wrong_range = if reader.read_chunk(shortened, &mut shortened_bytes)
        == Err(ChunkReadError::NotCapable)
        && reader.last_denial() == Some(GrantedChunkReadDenied::WrongRange)
    {
        GrantedChunkReadDenied::WrongRange.reason()
    } else {
        "failed"
    };

    let mut borrowed = state.borrow_mut();
    let Some(first) = borrowed.frames.first_mut() else {
        return StorageSelftestEvidence::failed("fixture_failed");
    };
    let Some(byte) = first.bytes.first_mut() else {
        return StorageSelftestEvidence::failed("fixture_failed");
    };
    *byte ^= 1;
    drop(borrowed);
    let mut tampered_destination = vec![0x5a; chunk.len as usize];
    let tamper = if reader.read_chunk(request, &mut tampered_destination) == Err(ChunkReadError::Io)
        && reader.last_denial() == Some(GrantedChunkReadDenied::HashMismatch)
        && tampered_destination.iter().all(|byte| *byte == 0x5a)
    {
        GrantedChunkReadDenied::HashMismatch.reason()
    } else {
        "failed"
    };

    StorageSelftestEvidence {
        materialize: "ok",
        grant_read,
        out_of_grant,
        wrong_range,
        tamper,
    }
}

fn sysimport_manifest_frame_error(
    error: crate::agent_protocol::artifact_store::BuildFsChunkFrameError,
) -> &'static str {
    match error {
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Missing => "missing",
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Bounds => "bounds",
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Malformed => "malformed",
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::LengthMismatch => {
            "length_mismatch"
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Io => "io",
    }
}

fn sysimport_manifest_chunk_count(manifest: &BuildFsManifest) -> Option<usize> {
    manifest
        .files
        .iter()
        .try_fold(0usize, |total, file| total.checked_add(file.chunks.len()))
}

fn run_sysimport_granted_reads(
    manifest: &BuildFsManifest,
    mut reader: GrantedChunkReader,
) -> SysimportEvidence {
    let Some(total_chunks) = sysimport_manifest_chunk_count(manifest) else {
        return SysimportEvidence::failed("ok", 0, "chunk_count_overflow");
    };
    if total_chunks == 0 || reader.entry_count() != total_chunks {
        return SysimportEvidence::failed("ok", 0, "entry_count_mismatch");
    }
    let buildfs = match project_core_manifest(manifest) {
        Ok(buildfs) => buildfs,
        Err(error) => return SysimportEvidence::failed("ok", 0, error.reason()),
    };
    let last = total_chunks - 1;
    let intervals = SYSIMPORT_SAMPLE_TARGET - 1;
    let stride = if total_chunks <= SYSIMPORT_SAMPLE_TARGET {
        1
    } else {
        (last + intervals - 1) / intervals
    };
    let mut flat_index = 0usize;
    let mut selected_count = 0usize;
    let mut successful_reads = 0usize;
    let mut first_request = None;

    for file in &manifest.files {
        let path = match NormalizedPath::root().resolve(file.path.as_bytes()) {
            Ok(path) => path,
            Err(_) => return SysimportEvidence::failed("ok", successful_reads, "path_failed"),
        };
        let file_node = match buildfs.node_for_path(&path) {
            Ok(node) => node,
            Err(_) => {
                return SysimportEvidence::failed("ok", successful_reads, "projection_failed")
            }
        };
        for (chunk_index, chunk) in file.chunks.iter().enumerate() {
            let selected = flat_index == 0 || flat_index == last || flat_index % stride == 0;
            if selected {
                selected_count += 1;
                let chunk_index = match u64::try_from(chunk_index) {
                    Ok(index) => index,
                    Err(_) => {
                        return SysimportEvidence::failed(
                            "ok",
                            successful_reads,
                            "chunk_index_overflow",
                        )
                    }
                };
                let request = ChunkReadRequest {
                    mount_id: SYSROOT_MOUNT,
                    file: file_node,
                    file_sha256: file.sha256,
                    chunk_index,
                    chunk_sha256: chunk.sha256,
                    range_offset: 0,
                    range_len: chunk.len,
                };
                let chunk_len = match usize::try_from(chunk.len) {
                    Ok(len) => len,
                    Err(_) => {
                        return SysimportEvidence::failed(
                            "ok",
                            successful_reads,
                            "chunk_length_overflow",
                        )
                    }
                };
                let mut bytes = vec![0; chunk_len];
                if reader.read_chunk(request, &mut bytes).is_err() {
                    let reason = reader
                        .last_denial()
                        .map(GrantedChunkReadDenied::reason)
                        .unwrap_or("sample_read_failed");
                    return SysimportEvidence::failed("ok", successful_reads, reason);
                }
                if sha256_bytes(&bytes) != chunk.sha256 {
                    return SysimportEvidence::failed(
                        "ok",
                        successful_reads,
                        "sample_hash_mismatch",
                    );
                }
                successful_reads += 1;
                if first_request.is_none() {
                    first_request = Some(request);
                }
            }
            flat_index += 1;
        }
    }
    if flat_index != total_chunks
        || selected_count != successful_reads
        || selected_count != SYSIMPORT_SAMPLE_TARGET
    {
        return SysimportEvidence::failed("ok", successful_reads, "sample_count_mismatch");
    }

    let Some(first_request) = first_request else {
        return SysimportEvidence::failed("ok", successful_reads, "sample_missing");
    };
    let first_len = match usize::try_from(first_request.range_len) {
        Ok(len) => len,
        Err(_) => {
            return SysimportEvidence::failed("ok", successful_reads, "chunk_length_overflow")
        }
    };
    let mut denied_destination = vec![0; first_len];
    let mut absent = first_request;
    absent.chunk_index = u64::MAX;
    if reader.read_chunk(absent, &mut denied_destination) != Err(ChunkReadError::NotCapable)
        || reader.last_denial() != Some(GrantedChunkReadDenied::AbsentEntry)
    {
        return SysimportEvidence::failed("ok", successful_reads, "absent_entry_failed");
    }

    let Some(short_len) = first_len.checked_sub(1) else {
        return SysimportEvidence::failed("ok", successful_reads, "wrong_range_fixture_failed");
    };
    let mut wrong_range = first_request;
    wrong_range.range_len -= 1;
    let mut shortened_destination = vec![0; short_len];
    if reader.read_chunk(wrong_range, &mut shortened_destination) != Err(ChunkReadError::NotCapable)
        || reader.last_denial() != Some(GrantedChunkReadDenied::WrongRange)
    {
        return SysimportEvidence::failed("ok", successful_reads, "wrong_range_failed");
    }

    SysimportEvidence {
        manifest: "ok",
        chunks: successful_reads,
        deny: "absent_entry+wrong_range",
        detail: "none",
        at: u64::MAX,
        passed: true,
    }
}

fn run_sysimport_selftest() -> SysimportEvidence {
    let Some(manifest_pin) = parse_sha256_ref(SYSROOT_BUILD_FS_MANIFEST_SHA256) else {
        return SysimportEvidence::failed("pin_invalid", 0, "not_run");
    };
    let session = match crate::agent_protocol::artifact_store::begin_build_chunk_read_session(
        BUILD_STORE_INSTANCE_ID,
        BUILD_STORE_GENERATION,
    ) {
        Ok(session) => session,
        Err(error) => {
            return SysimportEvidence::failed(sysimport_manifest_frame_error(error), 0, "not_run")
        }
    };
    let manifest_frame = match session
        .resolve_chunk_frame(manifest_pin, SYSROOT_BUILD_FS_MANIFEST_LEN)
    {
        Ok(frame) => frame,
        Err(error) => {
            return SysimportEvidence::failed(sysimport_manifest_frame_error(error), 0, "not_run")
        }
    };
    let manifest_len = match usize::try_from(SYSROOT_BUILD_FS_MANIFEST_LEN) {
        Ok(len) => len,
        Err(_) => return SysimportEvidence::failed("length_overflow", 0, "not_run"),
    };
    let mut manifest_bytes = vec![0; manifest_len];
    if let Err(error) = session.read_chunk_frame(manifest_frame, &mut manifest_bytes) {
        return SysimportEvidence::failed(sysimport_manifest_frame_error(error), 0, "not_run");
    }
    let recomputed_manifest_sha256 = sha256_bytes(&manifest_bytes);
    if recomputed_manifest_sha256 != manifest_pin {
        return SysimportEvidence::failed("hash_mismatch", 0, "not_run");
    }
    let sysroot_manifest = match parse_canonical_buildfs_manifest(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(reason) => return SysimportEvidence::failed(reason, 0, "not_run"),
    };
    let canonical_manifest_sha256 = match sysroot_manifest.sha256() {
        Ok(sha256) => sha256,
        Err(_) => return SysimportEvidence::failed("manifest_invalid", 0, "not_run"),
    };
    if canonical_manifest_sha256 != recomputed_manifest_sha256
        || canonical_manifest_sha256 != manifest_pin
    {
        return SysimportEvidence::failed("hash_mismatch", 0, "not_run");
    }
    let Some(src_manifest) = empty_manifest() else {
        return SysimportEvidence::failed("ok", 0, "src_manifest_failed");
    };
    let Some(job) = authorize_for_manifests(&sysroot_manifest, &src_manifest) else {
        return SysimportEvidence::failed("ok", 0, "job_authority_failed");
    };
    if job.sysroot_mount_manifest_sha256() != manifest_pin
        || job.sysroot_mount_manifest_sha256() != recomputed_manifest_sha256
    {
        return SysimportEvidence::failed("ticket_mismatch", 0, "not_run");
    }
    let Some(authority) = storage_authority(&job, &sysroot_manifest, &src_manifest) else {
        return SysimportEvidence::failed("ok", 0, "storage_authority_failed");
    };
    if authority.sysroot_mount_manifest_sha256() != manifest_pin
        || authority.sysroot_mount_manifest_sha256() != job.sysroot_mount_manifest_sha256()
    {
        return SysimportEvidence::failed("authority_mismatch", 0, "not_run");
    }
    let Some(nonce) = BuildRunNonce::kernel_minted(SYSIMPORT_RUN_NONCE) else {
        return SysimportEvidence::failed("ok", 0, "nonce_failed");
    };
    let reader = match materialize_build_storage(
        &authority,
        &sysroot_manifest,
        &src_manifest,
        nonce,
        Box::new(session),
    ) {
        Ok(reader) => reader,
        Err(error) => {
            return SysimportEvidence::failed_materialize(
                error.reason(),
                error.chunk_store_detail(),
                error.failing_chunk_index(),
            )
        }
    };
    if reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != SYSIMPORT_RUN_NONCE
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return SysimportEvidence::failed("ok", 0, "reader_binding_mismatch");
    }
    run_sysimport_granted_reads(&sysroot_manifest, reader)
}

fn empty_authority_for_ok_fixture() -> Option<BuildStorageAuthority> {
    let engine = wasi_engine();
    let module = Module::new(&engine, WASI_BUILD_OK_WASM).ok()?;
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    let sysroot_manifest = empty_manifest()?;
    let src_manifest = empty_manifest()?;
    let authorized = AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
        wasi_grant: ScopedWasiBuildGrant {
            compiler_artifact_sha256: COMPILER_ARTIFACT_SHA256,
            job_manifest_sha256: JOB_MANIFEST_SHA256,
            inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
            declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
        },
        observed_imports: &declarations,
        guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
        sysroot_mount_manifest_sha256: sysroot_manifest.sha256().ok()?,
        src_mount_manifest_sha256: src_manifest.sha256().ok()?,
    })
    .ok()?;
    storage_authority(&authorized, &sysroot_manifest, &src_manifest)
}

fn egress_commit_selftest(
    first: WasiJobEvidence,
    second: WasiJobEvidence,
) -> EgressCommitSelftestEvidence {
    let (Some(run_one), Some(run_two), Some(authority)) = (
        first.frozen_output,
        second.frozen_output,
        empty_authority_for_ok_fixture(),
    ) else {
        return EgressCommitSelftestEvidence::failed("missing");
    };
    let input = ScopedWasiArtifactEgress {
        run_one,
        run_two,
        run_one_exit_status: match first.end {
            WasiJobEnd::ProcExit(code) => code as i32,
            WasiJobEnd::Trap | WasiJobEnd::Denied(_) => -1,
        },
        run_two_exit_status: match second.end {
            WasiJobEnd::ProcExit(code) => code as i32,
            WasiJobEnd::Trap | WasiJobEnd::Denied(_) => -1,
        },
        run_one_logical_content_size: first.output_bundle_len,
        run_two_logical_content_size: second.output_bundle_len,
    };
    let plan = match evaluate_scoped_wasi_artifact_egress(&input, &authority) {
        ScopedWasiArtifactEgressDecision::Planned(plan) if !plan.authorizes_load() => plan,
        ScopedWasiArtifactEgressDecision::Planned(_) => {
            return EgressCommitSelftestEvidence::failed("authorizes_load")
        }
        ScopedWasiArtifactEgressDecision::Denied(rejection) => {
            return EgressCommitSelftestEvidence::failed(rejection.reason())
        }
    };
    evaluate_commit_claims(&authority, &plan)
}

fn evaluate_commit_claims(
    authority: &BuildStorageAuthority,
    plan: &WasiArtifactEgressPlan,
) -> EgressCommitSelftestEvidence {
    let lease = authority.output_lease();
    let bundle_len = plan.logical_content_size();
    if bundle_len == 0 {
        return EgressCommitSelftestEvidence::failed("empty_output_bundle");
    }
    let valid = ScopedBuildOutputCommitInput {
        job_binding_sha256: authority.job_binding_sha256(),
        lease_id: lease.lease_id(),
        store_instance_id: lease.store_instance_id(),
        store_generation: lease.store_generation(),
        lease_max_bytes: lease.max_bytes(),
        lease_target_marker: lease.target_marker(),
        span_offset: 4_096,
        span_len: 512,
        artstor_region_offset: 0,
        artstor_region_len: 128 * 1024 * 1024,
        alignment: 512,
        bundle_len,
        output_manifest_sha256: plan.output_manifest_sha256(),
        chunk_count: ((bundle_len - 1) / BUILD_FS_CHUNK_SIZE) + 1,
    };
    let commit = match evaluate_scoped_build_output_commit(&valid, authority, plan) {
        ScopedBuildOutputCommitDecision::Authorized(commit)
            if commit.job_binding_sha256() == authority.job_binding_sha256()
                && commit.lease_id() == lease.lease_id()
                && commit.bundle_len() == bundle_len
                && commit.output_manifest_sha256() == plan.output_manifest_sha256() =>
        {
            "authorized"
        }
        ScopedBuildOutputCommitDecision::Authorized(_) => "mismatch",
        ScopedBuildOutputCommitDecision::Denied(rejection) => rejection.reason(),
    };
    let mut out_of_lease = valid;
    out_of_lease.span_len = lease.max_bytes() + 512;
    let commit_deny = match evaluate_scoped_build_output_commit(&out_of_lease, authority, plan) {
        ScopedBuildOutputCommitDecision::Denied(
            BuildOutputCommitDenied::OutputSpanLengthExceedsLease,
        ) => BuildOutputCommitDenied::OutputSpanLengthExceedsLease.reason(),
        ScopedBuildOutputCommitDecision::Denied(rejection) => rejection.reason(),
        ScopedBuildOutputCommitDecision::Authorized(_) => "missing",
    };
    EgressCommitSelftestEvidence {
        egress: "planned",
        commit,
        commit_deny,
    }
}

fn memory_growth_arithmetic_valid(memory: WasiMemoryEvidence) -> bool {
    let expected_final = memory
        .grow_step_count
        .checked_mul(MEMORY_GROW_STEP_PAGES)
        .and_then(|grown_pages| memory.initial_pages.checked_add(grown_pages));
    memory.initial_pages == RUSTC_BUILD_GUEST_CLASS_V1.shared_memory.initial_pages
        && memory.grow_step_count >= 1
        && memory.final_pages > memory.initial_pages
        && memory.final_pages == memory.current_pages
        && expected_final == Some(memory.final_pages)
}

fn decimal_digit_count(mut value: u32) -> u64 {
    let mut digits = 1u64;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn memory_run_pass(run: WasiJobEvidence) -> bool {
    run.instantiated
        && run.end == WasiJobEnd::ProcExit(0)
        && run.registered == REQUIRED_IMPORT_COUNT
        && run.frozen_output_entries == 0
        && run.memory.grow_denied
        && run.memory.reported_pages == Some(run.memory.final_pages)
        && run.stdout_bytes == run.memory.stdout_len as u64
        && run.stdout_bytes == decimal_digit_count(run.memory.final_pages)
        && memory_growth_arithmetic_valid(run.memory)
}

pub(crate) fn emit_wasi_mem_selftest() {
    let first = run_build_job(WASI_MEM_GROW_WASM, 201);
    let second = run_build_job(WASI_MEM_GROW_WASM, 202);
    let over_class = run_build_job(WASI_MEM_OVER_CLASS_WASM, 203);
    let over_class_reason = match over_class.end {
        WasiJobEnd::Denied(denied) => denied.reason(),
        WasiJobEnd::ProcExit(_) | WasiJobEnd::Trap => "missing",
    };
    let grow_denied_gracefully = memory_run_pass(first) && memory_run_pass(second);
    let deterministic = first.memory.reported_pages.is_some()
        && second.memory.reported_pages.is_some()
        && first.memory.initial_pages == second.memory.initial_pages
        && first.memory.final_pages == second.memory.final_pages
        && first.memory.grow_step_count == second.memory.grow_step_count
        && first.memory.stdout_len == second.memory.stdout_len
        && first.memory.stdout == second.memory.stdout
        && first.stdout_bytes == second.stdout_bytes;
    let over_class_denied = !over_class.instantiated
        && matches!(
            over_class.end,
            WasiJobEnd::Denied(BuildJobDenied::ImportsMismatch { .. })
        )
        && over_class_reason == "imports_mismatch";
    let pass = grow_denied_gracefully && deterministic && over_class_denied;
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_WASIMEM selftest={} pages_initial={} pages_max={} grow_denied_gracefully={} over_class={} det={}\n",
        if pass { "pass" } else { "fail" },
        RUSTC_BUILD_GUEST_CLASS_V1.shared_memory.initial_pages,
        first.memory.reported_pages.unwrap_or(0),
        if grow_denied_gracefully { 1 } else { 0 },
        over_class_reason,
        if deterministic { 1 } else { 0 },
    ));
}

pub(crate) fn emit_wasi_sysimport() {
    let evidence = run_sysimport_selftest();
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_SYSIMPORT selftest={} manifest={} chunks={} deny={} detail={} at={}\n",
        if evidence.passed { "pass" } else { "fail" },
        evidence.manifest,
        evidence.chunks,
        evidence.deny,
        evidence.detail,
        evidence.at,
    ));
}

pub(crate) fn emit_wasi_selftest() {
    let positive = run_build_job(WASI_BUILD_OK_WASM, 1);
    let positive_two = run_build_job(WASI_BUILD_OK_WASM, 2);
    let negative = run_build_job(WASI_BUILD_EXTRA_IMPORT_WASM, 3);
    let storage = storage_selftest();
    let egress_commit = egress_commit_selftest(positive, positive_two);
    let ok_exit = match positive.end {
        WasiJobEnd::ProcExit(code) => code,
        WasiJobEnd::Trap | WasiJobEnd::Denied(_) => u32::MAX,
    };
    let deny = match negative.end {
        WasiJobEnd::Denied(denied) => denied.reason(),
        WasiJobEnd::ProcExit(_) | WasiJobEnd::Trap => "missing",
    };
    let pass = positive.instantiated
        && positive.end == WasiJobEnd::ProcExit(0)
        && positive.stdout_bytes == 3
        && positive.registered == REQUIRED_IMPORT_COUNT
        && positive.frozen_output_entries == 0
        && positive_two.instantiated
        && positive_two.end == WasiJobEnd::ProcExit(0)
        && positive_two.stdout_bytes == positive.stdout_bytes
        && positive_two.registered == positive.registered
        && positive_two.frozen_output == positive.frozen_output
        && positive_two.output_bundle_len == positive.output_bundle_len
        && !negative.instantiated
        && matches!(
            negative.end,
            WasiJobEnd::Denied(BuildJobDenied::ImportsMismatch { .. })
        )
        && deny == "imports_mismatch"
        && storage.materialize == "ok"
        && storage.grant_read == "ok"
        && storage.out_of_grant == "absent_entry"
        && storage.wrong_range == "wrong_range"
        && storage.tamper == "hash_mismatch"
        && egress_commit.egress == "planned"
        && egress_commit.commit == "authorized"
        && egress_commit.commit_deny == "output_span_length_exceeds_lease";
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_WASI selftest={} ok_exit={} ok_stdout={} deny={} registered={} materialize={} grant_read={} out_of_grant={} wrong_range={} tamper={} egress={} commit={} commit_deny={}\n",
        if pass { "pass" } else { "fail" },
        ok_exit,
        positive.stdout_bytes,
        deny,
        positive.registered,
        storage.materialize,
        storage.grant_read,
        storage.out_of_grant,
        storage.wrong_range,
        storage.tamper,
        egress_commit.egress,
        egress_commit.commit,
        egress_commit.commit_deny,
    ));
}
