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
    core::{Pages, Trap, TrapCode, ValueType},
    errors::{MemoryError, TableError},
    Config, Engine, Error as WasmiError, ExternType, Linker, Memory, MemoryType, Module,
    Mutability, ResourceLimiter, ResumableCall, Store, Suspension,
};

use super::{
    invocation::{release_thread_job_execution, try_acquire_thread_job_execution},
    wasi_build_storage::{
        materialize_build_storage, project_core_manifest, BuildChunkHandle, BuildChunkStore,
        BuildChunkStoreError, GrantedChunkReadDenied, GrantedChunkReader, UnbackedChunkStore,
    },
    wasi_preview1::{
        define_wasi_imports, ProcExitTrap, ThreadHostMode, ThreadWorld, WasiHostState,
    },
    wasi_thread_pump::{
        WasiThreadJobEnd, WasiThreadJobFailure, WasiThreadJobRunner, WasiThreadRunEvidence,
    },
};

const WASI_BUILD_OK_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_ok.wasm"));
const WASI_BUILD_EXTRA_IMPORT_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_extra_import.wasm"));
const WASI_MEM_GROW_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasi_mem_grow.wasm"));
const WASI_MEM_OVER_CLASS_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_mem_over_class.wasm"));
const WASI_THREAD_FIXTURE_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_thread_fixture.wasm"));
const COMPILER_ARTIFACT_SHA256: &str =
    "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
const COMPILER_BUILD_FS_MANIFEST_SHA256: &str =
    "1b9214df9abd5ea546353a7bea9f996705732f0cf19d3a0ff5cc9f38eebcaf15";
const COMPILER_BUILD_FS_MANIFEST_LEN: u64 = 58_407;
const COMPILER_FILE_LEN: u64 = 95_427_808;
const COMPILER_CHUNK_COUNT: usize = 1_457;
const JOB_MANIFEST_SHA256: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
// docs/architecture/sysroot-buildfs-manifest-13daf6f9.md
const SYSROOT_BUILD_FS_MANIFEST_SHA256: &str =
    "13daf6f9042d07c4d698d60ea16869ed85e2035f762f4b5a048e71e7523b7b15";
const SYSROOT_BUILD_FS_MANIFEST_LEN: u64 = 51_089;
const SYSIMPORT_SAMPLE_TARGET: usize = 32;
const SYSIMPORT_RUN_NONCE: u64 = 301;
const COMPILERLOAD_READ_NONCE: u64 = 302;
const COMPILERLOAD_INSTANCE_NONCE: u64 = 303;
const WASI_THREAD_RUN_NONCE: u64 = 304;
const RUSTCRUN_COMPILER_READ_NONCE: u64 = 305;
const RUSTCRUN_INSTANCE_NONCE: u64 = 306;
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

    const fn failed_materialize(deny: &'static str, detail: &'static str, at: u64) -> Self {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompilerLoadStage {
    Failed,
    Reassembled,
    Parsed,
    Authorized,
    Instantiated,
}

impl CompilerLoadStage {
    const fn token(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Reassembled => "reassembled",
            Self::Parsed => "parsed",
            Self::Authorized => "authorized",
            Self::Instantiated => "instantiated",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompilerFileSha {
    Ok,
    Mismatch,
}

impl CompilerFileSha {
    const fn token(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Mismatch => "mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CompilerLoadEvidence {
    stage: CompilerLoadStage,
    bytes: usize,
    file_sha: CompilerFileSha,
    imports: usize,
    mem_pages: u32,
    reason: &'static str,
}

impl CompilerLoadEvidence {
    const fn failed(reason: &'static str) -> Self {
        Self {
            stage: CompilerLoadStage::Failed,
            bytes: 0,
            file_sha: CompilerFileSha::Mismatch,
            imports: 0,
            mem_pages: 0,
            reason,
        }
    }

    const fn at(
        stage: CompilerLoadStage,
        bytes: usize,
        file_sha: CompilerFileSha,
        imports: usize,
        mem_pages: u32,
        reason: &'static str,
    ) -> Self {
        Self {
            stage,
            bytes,
            file_sha,
            imports,
            mem_pages,
            reason,
        }
    }
}

struct CompilerReassemblyFailure {
    bytes: usize,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RustcRunStage {
    Reassembled,
    Instantiated,
    Started,
    Exited,
    Trapped,
    Deadlocked,
    Ceiling,
}

impl RustcRunStage {
    const fn token(self) -> &'static str {
        match self {
            Self::Reassembled => "reassembled",
            Self::Instantiated => "instantiated",
            Self::Started => "started",
            Self::Exited => "exited",
            Self::Trapped => "trapped",
            Self::Deadlocked => "deadlocked",
            Self::Ceiling => "ceiling",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RustcRunEvidence {
    stage: RustcRunStage,
    file_sha: CompilerFileSha,
    spawns: u32,
    cap_denials: u32,
    rounds: u64,
    stdout_bytes: u64,
    granted_total: u64,
    exit_code: Option<u32>,
    reason: &'static str,
}

impl RustcRunEvidence {
    const fn at(stage: RustcRunStage, file_sha: CompilerFileSha, reason: &'static str) -> Self {
        Self {
            stage,
            file_sha,
            spawns: 0,
            cap_denials: 0,
            rounds: 0,
            stdout_bytes: 0,
            granted_total: 0,
            exit_code: None,
            reason,
        }
    }

    fn completed(run: WasiThreadRunEvidence) -> Self {
        let stdout_bytes = u64::try_from(run.stdout.len()).unwrap_or(u64::MAX);
        let (stage, exit_code, reason) = match run.end {
            WasiThreadJobEnd::JobExited { code } => (
                RustcRunStage::Exited,
                Some(code),
                if code == 0 {
                    "none"
                } else {
                    "guest_exit_nonzero"
                },
            ),
            WasiThreadJobEnd::JobDeadlocked => (RustcRunStage::Deadlocked, None, "thread_deadlock"),
            WasiThreadJobEnd::Failed(WasiThreadJobFailure::FuelCeiling) => {
                (RustcRunStage::Ceiling, None, "fuel_ceiling")
            }
            WasiThreadJobEnd::Failed(WasiThreadJobFailure::RoundLimit) => {
                (RustcRunStage::Ceiling, None, "round_limit")
            }
            WasiThreadJobEnd::Failed(failure) => {
                (RustcRunStage::Trapped, None, rustcrun_pump_failure(failure))
            }
        };
        Self {
            stage,
            file_sha: CompilerFileSha::Ok,
            spawns: run.spawns,
            cap_denials: run.cap_denials,
            rounds: run.rounds,
            stdout_bytes,
            granted_total: run.granted_total,
            exit_code,
            reason,
        }
    }
}

fn rustcrun_pump_failure(failure: WasiThreadJobFailure) -> &'static str {
    match failure {
        WasiThreadJobFailure::Setup => "pump_setup",
        WasiThreadJobFailure::Scheduler => "scheduler_state",
        WasiThreadJobFailure::Fuel => "fuel_accounting",
        WasiThreadJobFailure::FuelCeiling => "fuel_ceiling",
        WasiThreadJobFailure::Engine => "guest_trap",
        WasiThreadJobFailure::MemoryIdentity => "memory_identity",
        WasiThreadJobFailure::HostResultType => "host_result_type",
        WasiThreadJobFailure::Materialization => "worker_materialization",
        WasiThreadJobFailure::RoundLimit => "round_limit",
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
        if self.is_scheduled_thread_job() {
            return Ok(maximum.is_some_and(|maximum| desired <= maximum));
        }
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
    let mut store = Store::new(
        &engine,
        WasiHostState::new(instance, chunk_reader, ThreadHostMode::Deny),
    );
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
    build_instance_with_process(
        authorized,
        class,
        sysroot_manifest,
        src_manifest,
        Vec::new(),
        Vec::new(),
    )
}

fn build_instance_with_process(
    authorized: &AuthorizedBuildJob,
    class: BuildGuestClassV1,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
    args: Vec<Vec<u8>>,
    environment: Vec<(Vec<u8>, Vec<u8>)>,
) -> Option<WasiBuildInstance> {
    let sysroot = project_core_manifest(sysroot_manifest).ok()?;
    let source = project_core_manifest(src_manifest).ok()?;
    let job = JobContext::new(args, environment, authorized.job_manifest_sha256()).ok()?;
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

fn compilerload_store_error(
    error: crate::agent_protocol::artifact_store::BuildFsChunkFrameError,
) -> &'static str {
    match error {
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Missing => {
            "store_frame_missing"
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Bounds => {
            "store_frame_bounds"
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Malformed => {
            "store_frame_malformed"
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::LengthMismatch => {
            "store_frame_length_mismatch"
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Io => "store_io",
    }
}

fn compiler_file_len(file_len: u64) -> Result<usize, &'static str> {
    if file_len > COMPILER_FILE_LEN {
        return Err("file_too_large");
    }
    if file_len != COMPILER_FILE_LEN {
        return Err("file_length_mismatch");
    }
    usize::try_from(file_len).map_err(|_| "file_length_overflow")
}

fn compiler_file_sha_status(actual: [u8; 32], expected: [u8; 32]) -> CompilerFileSha {
    if actual == expected {
        CompilerFileSha::Ok
    } else {
        CompilerFileSha::Mismatch
    }
}

fn compilerload_negative_boundaries(expected_file_sha256: [u8; 32]) -> bool {
    let Some(oversized) = COMPILER_FILE_LEN.checked_add(1) else {
        return false;
    };
    if compiler_file_len(oversized) != Err("file_too_large") {
        return false;
    }
    let mut mismatch = expected_file_sha256;
    mismatch[0] ^= 1;
    compiler_file_sha_status(mismatch, expected_file_sha256) == CompilerFileSha::Mismatch
}

#[derive(Clone)]
struct SharedBuildChunkStore {
    session: Rc<RefCell<crate::agent_protocol::artifact_store::BuildChunkReadSession>>,
    store_instance_id: u64,
    store_generation: u64,
}

impl SharedBuildChunkStore {
    fn new(session: crate::agent_protocol::artifact_store::BuildChunkReadSession) -> Self {
        let store_instance_id = session.store_instance_id();
        let store_generation = session.store_generation();
        Self {
            session: Rc::new(RefCell::new(session)),
            store_instance_id,
            store_generation,
        }
    }

    fn load_manifest(
        &self,
        manifest_sha256: &str,
        manifest_len: u64,
    ) -> Result<BuildFsManifest, &'static str> {
        let manifest_pin = parse_sha256_ref(manifest_sha256).ok_or("manifest_pin_invalid")?;
        let session = self
            .session
            .try_borrow()
            .map_err(|_| "store_session_busy")?;
        let manifest_frame = session
            .resolve_chunk_frame(manifest_pin, manifest_len)
            .map_err(compilerload_store_error)?;
        let manifest_len = usize::try_from(manifest_len).map_err(|_| "manifest_length_overflow")?;
        let mut manifest_bytes = Vec::new();
        manifest_bytes
            .try_reserve_exact(manifest_len)
            .map_err(|_| "manifest_allocation_failed")?;
        manifest_bytes.resize(manifest_len, 0);
        session
            .read_chunk_frame(manifest_frame, &mut manifest_bytes)
            .map_err(compilerload_store_error)?;
        drop(session);
        let recomputed_manifest_sha256 = sha256_bytes(&manifest_bytes);
        if recomputed_manifest_sha256 != manifest_pin {
            return Err("manifest_hash_mismatch");
        }
        let manifest = parse_canonical_buildfs_manifest(&manifest_bytes)?;
        let canonical_manifest_sha256 = manifest.sha256().map_err(|_| "manifest_invalid")?;
        if canonical_manifest_sha256 != recomputed_manifest_sha256
            || canonical_manifest_sha256 != manifest_pin
        {
            return Err("manifest_hash_mismatch");
        }
        Ok(manifest)
    }
}

impl BuildChunkStore for SharedBuildChunkStore {
    fn store_instance_id(&self) -> u64 {
        self.store_instance_id
    }

    fn store_generation(&self) -> u64 {
        self.store_generation
    }

    fn resolve_chunk(
        &mut self,
        sha256: [u8; 32],
        len: u64,
    ) -> Result<BuildChunkHandle, BuildChunkStoreError> {
        let session = self
            .session
            .try_borrow()
            .map_err(|_| BuildChunkStoreError::Io)?;
        let frame = session
            .resolve_chunk_frame(sha256, len)
            .map_err(rustcrun_store_error)?;
        Ok(BuildChunkHandle {
            store_generation: frame.store_generation,
            frame_offset: frame.frame_offset,
            frame_len: frame.frame_len,
            payload_len: frame.payload_len,
            payload_sha256: frame.payload_sha256,
            frame_sha256: frame.frame_sha256,
        })
    }

    fn read_resolved_chunk(
        &mut self,
        handle: BuildChunkHandle,
        destination: &mut [u8],
    ) -> Result<(), BuildChunkStoreError> {
        let session = self
            .session
            .try_borrow()
            .map_err(|_| BuildChunkStoreError::Io)?;
        session
            .read_chunk_frame(
                crate::agent_protocol::artifact_store::BuildFsChunkFrame {
                    store_generation: handle.store_generation,
                    frame_offset: handle.frame_offset,
                    frame_len: handle.frame_len,
                    payload_len: handle.payload_len,
                    payload_sha256: handle.payload_sha256,
                    frame_sha256: handle.frame_sha256,
                },
                destination,
            )
            .map_err(rustcrun_store_error)
    }
}

fn rustcrun_store_error(
    error: crate::agent_protocol::artifact_store::BuildFsChunkFrameError,
) -> BuildChunkStoreError {
    match error {
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Missing => {
            BuildChunkStoreError::Missing
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Bounds
        | crate::agent_protocol::artifact_store::BuildFsChunkFrameError::LengthMismatch => {
            BuildChunkStoreError::Bounds
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Malformed => {
            BuildChunkStoreError::MalformedFrame
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Io => {
            BuildChunkStoreError::Io
        }
    }
}

fn load_compiler_manifest_and_reader() -> Result<(BuildFsManifest, GrantedChunkReader), &'static str>
{
    let manifest_pin =
        parse_sha256_ref(COMPILER_BUILD_FS_MANIFEST_SHA256).ok_or("manifest_pin_invalid")?;
    let session = crate::agent_protocol::artifact_store::begin_build_chunk_read_session(
        BUILD_STORE_INSTANCE_ID,
        BUILD_STORE_GENERATION,
    )
    .map_err(compilerload_store_error)?;
    let manifest_frame = session
        .resolve_chunk_frame(manifest_pin, COMPILER_BUILD_FS_MANIFEST_LEN)
        .map_err(compilerload_store_error)?;
    let manifest_len =
        usize::try_from(COMPILER_BUILD_FS_MANIFEST_LEN).map_err(|_| "manifest_length_overflow")?;
    let mut manifest_bytes = Vec::new();
    manifest_bytes
        .try_reserve_exact(manifest_len)
        .map_err(|_| "manifest_allocation_failed")?;
    manifest_bytes.resize(manifest_len, 0);
    session
        .read_chunk_frame(manifest_frame, &mut manifest_bytes)
        .map_err(compilerload_store_error)?;
    let recomputed_manifest_sha256 = sha256_bytes(&manifest_bytes);
    if recomputed_manifest_sha256 != manifest_pin {
        return Err("manifest_hash_mismatch");
    }
    let compiler_manifest = parse_canonical_buildfs_manifest(&manifest_bytes)?;
    let canonical_manifest_sha256 = compiler_manifest.sha256().map_err(|_| "manifest_invalid")?;
    if canonical_manifest_sha256 != recomputed_manifest_sha256
        || canonical_manifest_sha256 != manifest_pin
    {
        return Err("manifest_hash_mismatch");
    }
    let empty_src = empty_manifest().ok_or("reader_src_manifest_failed")?;
    let reader_job =
        authorize_for_manifests(&compiler_manifest, &empty_src).ok_or("reader_job_unauthorized")?;
    let authority = storage_authority(&reader_job, &compiler_manifest, &empty_src)
        .ok_or("reader_storage_unauthorized")?;
    let nonce =
        BuildRunNonce::kernel_minted(COMPILERLOAD_READ_NONCE).ok_or("reader_nonce_failed")?;
    let reader = materialize_build_storage(
        &authority,
        &compiler_manifest,
        &empty_src,
        nonce,
        Box::new(session),
    )
    .map_err(|error| error.reason())?;
    if reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != COMPILERLOAD_READ_NONCE
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return Err("reader_binding_mismatch");
    }
    Ok((compiler_manifest, reader))
}

fn reassemble_compiler(
    manifest: &BuildFsManifest,
    mut reader: GrantedChunkReader,
    expected_file_sha256: [u8; 32],
) -> Result<Vec<u8>, CompilerReassemblyFailure> {
    if manifest.files.len() != 1 {
        return Err(CompilerReassemblyFailure {
            bytes: 0,
            reason: "manifest_file_count_mismatch",
        });
    }
    let file = &manifest.files[0];
    let file_len = compiler_file_len(file.len)
        .map_err(|reason| CompilerReassemblyFailure { bytes: 0, reason })?;
    if file.sha256 != expected_file_sha256 {
        return Err(CompilerReassemblyFailure {
            bytes: 0,
            reason: "manifest_file_sha_mismatch",
        });
    }
    if file.chunks.len() != COMPILER_CHUNK_COUNT || reader.entry_count() != COMPILER_CHUNK_COUNT {
        return Err(CompilerReassemblyFailure {
            bytes: 0,
            reason: "chunk_count_mismatch",
        });
    }
    let buildfs = project_core_manifest(manifest).map_err(|error| CompilerReassemblyFailure {
        bytes: 0,
        reason: error.reason(),
    })?;
    let path = NormalizedPath::root()
        .resolve(file.path.as_bytes())
        .map_err(|_| CompilerReassemblyFailure {
            bytes: 0,
            reason: "file_path_invalid",
        })?;
    let file_node = buildfs
        .node_for_path(&path)
        .map_err(|_| CompilerReassemblyFailure {
            bytes: 0,
            reason: "file_projection_failed",
        })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(file_len)
        .map_err(|_| CompilerReassemblyFailure {
            bytes: 0,
            reason: "file_allocation_failed",
        })?;
    bytes.resize(file_len, 0);
    let mut offset = 0usize;
    for (chunk_index, chunk) in file.chunks.iter().enumerate() {
        let chunk_len = usize::try_from(chunk.len).map_err(|_| CompilerReassemblyFailure {
            bytes: offset,
            reason: "chunk_length_overflow",
        })?;
        let end = offset
            .checked_add(chunk_len)
            .filter(|end| *end <= file_len)
            .ok_or(CompilerReassemblyFailure {
                bytes: offset,
                reason: "chunk_range_exceeds_file",
            })?;
        let chunk_index = u64::try_from(chunk_index).map_err(|_| CompilerReassemblyFailure {
            bytes: offset,
            reason: "chunk_index_overflow",
        })?;
        let request = ChunkReadRequest {
            mount_id: SYSROOT_MOUNT,
            file: file_node,
            file_sha256: file.sha256,
            chunk_index,
            chunk_sha256: chunk.sha256,
            range_offset: 0,
            range_len: chunk.len,
        };
        if reader.read_chunk(request, &mut bytes[offset..end]).is_err() {
            return Err(CompilerReassemblyFailure {
                bytes: offset,
                reason: reader
                    .last_denial()
                    .map(GrantedChunkReadDenied::reason)
                    .unwrap_or("chunk_read_failed"),
            });
        }
        offset = end;
    }
    if offset != file_len {
        return Err(CompilerReassemblyFailure {
            bytes: offset,
            reason: "reassembled_length_mismatch",
        });
    }
    if compiler_file_sha_status(sha256_bytes(&bytes), expected_file_sha256) != CompilerFileSha::Ok {
        return Err(CompilerReassemblyFailure {
            bytes: offset,
            reason: "whole_file_sha_mismatch",
        });
    }
    Ok(bytes)
}

fn instantiate_compiler(
    engine: Engine,
    module: Module,
    authorized: AuthorizedBuildJob,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
    byte_count: usize,
    import_count: usize,
) -> CompilerLoadEvidence {
    let class = *authorized.guest_class();
    let instance = match build_instance(&authorized, class, sysroot_manifest, src_manifest) {
        Some(instance) => instance,
        None => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "build_instance_failed",
            )
        }
    };
    let authority = match storage_authority(&authorized, sysroot_manifest, src_manifest) {
        Some(authority) => authority,
        None => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "storage_authority_failed",
            )
        }
    };
    let nonce = match BuildRunNonce::kernel_minted(COMPILERLOAD_INSTANCE_NONCE) {
        Some(nonce) => nonce,
        None => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "instance_nonce_failed",
            )
        }
    };
    let reader = match materialize_build_storage(
        &authority,
        sysroot_manifest,
        src_manifest,
        nonce,
        Box::new(UnbackedChunkStore::new(
            BUILD_STORE_INSTANCE_ID,
            BUILD_STORE_GENERATION,
        )),
    ) {
        Ok(reader) => reader,
        Err(error) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                error.reason(),
            )
        }
    };
    if reader.entry_count() != 0
        || reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != COMPILERLOAD_INSTANCE_NONCE
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Instantiated,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            0,
            "instance_reader_binding_mismatch",
        );
    }
    let mut store = Store::new(
        &engine,
        WasiHostState::new(instance, reader, ThreadHostMode::Deny),
    );
    store.limiter(|state| state);
    let memory_type = match MemoryType::new_shared(
        class.shared_memory.initial_pages,
        class.shared_memory.max_pages,
    ) {
        Ok(memory_type) => memory_type,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "memory_type_invalid",
            )
        }
    };
    let memory = match Memory::new(&mut store, memory_type) {
        Ok(memory) => memory,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "memory_allocation_failed",
            )
        }
    };
    let memory_pages = u32::from(memory.current_pages(&store));
    if memory_pages != class.shared_memory.initial_pages {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Instantiated,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            memory_pages,
            "memory_initial_mismatch",
        );
    }
    store.data_mut().install_memory(memory);
    let mut linker = Linker::<WasiHostState>::new(&engine);
    let registered = match define_wasi_imports(&mut linker, memory) {
        Ok(registered) => registered,
        Err(()) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                memory_pages,
                "linker_definition_failed",
            )
        }
    };
    if registered != REQUIRED_IMPORT_COUNT
        || registered != RUSTC_WASM_C6DCCF3E_IMPORTS.len()
        || registered != authorized.authorized_import_count()
    {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Instantiated,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            memory_pages,
            "linker_count_mismatch",
        );
    }
    let pre = match linker.instantiate(&mut store, &module) {
        Ok(pre) => pre,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Instantiated,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                memory_pages,
                "instantiate_failed",
            )
        }
    };
    // The module PARSED, authorized against the exact-30 gate, and its shared
    // memory + linker instantiated to an InstancePre — that IS the
    // load+instantiate milestone. pre.start() then runs any declared start
    // section (the first guest bytecode); unlike the runner it is not
    // resumable, so give the store a whole job's fuel budget up front so a
    // fuel-bounded start section can complete. A remaining error means the
    // start section needs something this isolated load cannot give (files,
    // real threads) — that is the execution milestone, reported honestly.
    // rustc's exported `_start` is never looked up or invoked here.
    let _ = store.add_fuel(RUSTC_BUILD_GUEST_CLASS_V1.max_total_fuel);
    if pre.start(&mut store).is_err() {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Instantiated,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            memory_pages,
            "start_section_trapped",
        );
    }
    CompilerLoadEvidence::at(
        CompilerLoadStage::Instantiated,
        byte_count,
        CompilerFileSha::Ok,
        import_count,
        memory_pages,
        "none",
    )
}

fn run_compilerload() -> CompilerLoadEvidence {
    let Some(expected_file_sha256) = parse_sha256_ref(COMPILER_ARTIFACT_SHA256) else {
        return CompilerLoadEvidence::failed("file_sha_pin_invalid");
    };
    if !compilerload_negative_boundaries(expected_file_sha256) {
        return CompilerLoadEvidence::failed("negative_boundary_failed");
    }
    let (compiler_manifest, reader) = match load_compiler_manifest_and_reader() {
        Ok(loaded) => loaded,
        Err(reason) => return CompilerLoadEvidence::failed(reason),
    };
    let bytes = match reassemble_compiler(&compiler_manifest, reader, expected_file_sha256) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Reassembled,
                failure.bytes,
                CompilerFileSha::Mismatch,
                0,
                0,
                failure.reason,
            )
        }
    };
    let byte_count = bytes.len();
    drop(compiler_manifest);

    let engine = wasi_engine();
    let module = match Module::new(&engine, bytes.as_slice()) {
        Ok(module) => module,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Parsed,
                byte_count,
                CompilerFileSha::Ok,
                0,
                0,
                "module_invalid",
            )
        }
    };
    drop(bytes);
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    let import_count = declarations.len();
    let imports_match = declarations.as_slice() == RUSTC_WASM_C6DCCF3E_IMPORTS;
    let Some(sysroot_manifest) = empty_manifest() else {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Parsed,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            0,
            "sysroot_manifest_failed",
        );
    };
    let Some(src_manifest) = empty_manifest() else {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Parsed,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            0,
            "src_manifest_failed",
        );
    };
    let sysroot_mount_hash = match sysroot_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Parsed,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "sysroot_manifest_invalid",
            )
        }
    };
    let src_mount_hash = match src_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Parsed,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                "src_manifest_invalid",
            )
        }
    };
    let gate = AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
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
    });
    if !imports_match {
        let reason = match gate {
            Err(denied) => denied.reason(),
            Ok(_) => "gate_accepted_import_mismatch",
        };
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Authorized,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            0,
            reason,
        );
    }
    let authorized = match gate {
        Ok(authorized) => authorized,
        Err(denied) => {
            return CompilerLoadEvidence::at(
                CompilerLoadStage::Authorized,
                byte_count,
                CompilerFileSha::Ok,
                import_count,
                0,
                denied.reason(),
            )
        }
    };
    if authorized.compiler_artifact_sha256() != expected_file_sha256
        || authorized.authorized_import_count() != REQUIRED_IMPORT_COUNT
    {
        return CompilerLoadEvidence::at(
            CompilerLoadStage::Authorized,
            byte_count,
            CompilerFileSha::Ok,
            import_count,
            0,
            "gate_binding_mismatch",
        );
    }
    drop(declarations);
    drop(observed);
    instantiate_compiler(
        engine,
        module,
        authorized,
        &sysroot_manifest,
        &src_manifest,
        byte_count,
        import_count,
    )
}

fn rustcrun_args() -> Vec<Vec<u8>> {
    vec![
        b"rustc".to_vec(),
        b"--version".to_vec(),
        b"--sysroot".to_vec(),
        b"/sysroot".to_vec(),
    ]
}

fn rustcrun_trap_code_reason(code: TrapCode) -> &'static str {
    match code {
        TrapCode::UnreachableCodeReached => "unreachable",
        TrapCode::MemoryOutOfBounds => "memory_out_of_bounds",
        TrapCode::TableOutOfBounds => "table_out_of_bounds",
        TrapCode::IndirectCallToNull => "indirect_call_null",
        TrapCode::IntegerDivisionByZero => "integer_division_by_zero",
        TrapCode::IntegerOverflow => "integer_overflow",
        TrapCode::BadConversionToInteger => "bad_integer_conversion",
        TrapCode::StackOverflow => "stack_overflow",
        TrapCode::BadSignature => "bad_signature",
        TrapCode::OutOfFuel => "pre_start_out_of_fuel",
        TrapCode::GrowthOperationLimited => "growth_limited",
        TrapCode::UnalignedAtomic => "unaligned_atomic",
        TrapCode::UnsharedMemoryAtomicWait => "unshared_atomic_wait",
        TrapCode::AtomicSuspendNotResumable => "pre_start_atomic_suspend",
    }
}

fn rustcrun_wasmi_error(error: &WasmiError) -> &'static str {
    match error {
        WasmiError::Trap(trap) => trap
            .trap_code()
            .map(rustcrun_trap_code_reason)
            .unwrap_or("host_trap"),
        WasmiError::Global(_) => "global_error",
        WasmiError::Memory(_) => "memory_error",
        WasmiError::Table(_) => "table_error",
        WasmiError::Linker(_) => "linker_error",
        WasmiError::Instantiation(_) => "instantiation_error",
        WasmiError::Module(_) => "module_error",
        WasmiError::Store(_) => "store_fuel_error",
        WasmiError::Func(_) => "function_error",
        _ => "engine_error",
    }
}

fn rustcrun_pre_start_failure(
    store: &Store<WasiHostState>,
    error: &WasmiError,
) -> RustcRunEvidence {
    if matches!(
        error,
        WasmiError::Trap(trap) if trap.downcast_ref::<ProcExitTrap>().is_some()
    ) {
        if let Some(code) = store.data().terminal_exit() {
            let (spawns, cap_denials, stdout_bytes) = store
                .data()
                .scheduled_world()
                .map(|world| {
                    (
                        world.spawns,
                        world.cap_denials,
                        u64::try_from(world.stdout.len()).unwrap_or(u64::MAX),
                    )
                })
                .unwrap_or((0, 0, 0));
            return RustcRunEvidence {
                stage: RustcRunStage::Exited,
                file_sha: CompilerFileSha::Ok,
                spawns,
                cap_denials,
                rounds: 0,
                stdout_bytes,
                granted_total: 0,
                exit_code: Some(code),
                reason: if code == 0 {
                    "none"
                } else {
                    "pre_start_exit_nonzero"
                },
            };
        }
    }
    RustcRunEvidence::at(
        RustcRunStage::Trapped,
        CompilerFileSha::Ok,
        rustcrun_wasmi_error(error),
    )
}

fn run_rustcrun() -> RustcRunEvidence {
    let Some(expected_file_sha256) = parse_sha256_ref(COMPILER_ARTIFACT_SHA256) else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "file_sha_pin_invalid",
        );
    };
    if !compilerload_negative_boundaries(expected_file_sha256) {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "negative_boundary_failed",
        );
    }

    // Exactly one ARTSTOR scan/index and one held I/O pin back both the
    // compiler reassembly reader and the later real-sysroot reader.
    let session = match crate::agent_protocol::artifact_store::begin_build_chunk_read_session(
        BUILD_STORE_INSTANCE_ID,
        BUILD_STORE_GENERATION,
    ) {
        Ok(session) => session,
        Err(error) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Mismatch,
                compilerload_store_error(error),
            )
        }
    };
    let shared_store = SharedBuildChunkStore::new(session);
    let compiler_manifest = match shared_store.load_manifest(
        COMPILER_BUILD_FS_MANIFEST_SHA256,
        COMPILER_BUILD_FS_MANIFEST_LEN,
    ) {
        Ok(manifest) => manifest,
        Err(reason) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Mismatch,
                reason,
            )
        }
    };
    let Some(empty_compiler_src) = empty_manifest() else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "reader_src_manifest_failed",
        );
    };
    let Some(compiler_reader_job) =
        authorize_for_manifests(&compiler_manifest, &empty_compiler_src)
    else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "reader_job_unauthorized",
        );
    };
    let Some(compiler_authority) = storage_authority(
        &compiler_reader_job,
        &compiler_manifest,
        &empty_compiler_src,
    ) else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "reader_storage_unauthorized",
        );
    };
    let Some(compiler_nonce) = BuildRunNonce::kernel_minted(RUSTCRUN_COMPILER_READ_NONCE) else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "reader_nonce_failed",
        );
    };
    let compiler_reader = match materialize_build_storage(
        &compiler_authority,
        &compiler_manifest,
        &empty_compiler_src,
        compiler_nonce,
        Box::new(shared_store.clone()),
    ) {
        Ok(reader) => reader,
        Err(error) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Mismatch,
                error.reason(),
            )
        }
    };
    if compiler_reader.entry_count() != COMPILER_CHUNK_COUNT
        || compiler_reader.job_binding_sha256() != compiler_authority.job_binding_sha256()
        || compiler_reader.run_nonce() != RUSTCRUN_COMPILER_READ_NONCE
        || compiler_reader.store_generation() != BUILD_STORE_GENERATION
    {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "reader_binding_mismatch",
        );
    }
    let bytes = match reassemble_compiler(&compiler_manifest, compiler_reader, expected_file_sha256)
    {
        Ok(bytes) => bytes,
        Err(failure) => {
            let _ = failure.bytes;
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Mismatch,
                failure.reason,
            );
        }
    };
    drop(compiler_manifest);

    let engine = wasi_engine();
    let module = match Module::new(&engine, bytes.as_slice()) {
        Ok(module) => module,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Ok,
                "module_invalid",
            )
        }
    };
    drop(bytes);

    let sysroot_manifest = match shared_store.load_manifest(
        SYSROOT_BUILD_FS_MANIFEST_SHA256,
        SYSROOT_BUILD_FS_MANIFEST_LEN,
    ) {
        Ok(manifest) => manifest,
        Err(reason) => {
            return RustcRunEvidence::at(RustcRunStage::Reassembled, CompilerFileSha::Ok, reason)
        }
    };
    let Some(src_manifest) = empty_manifest() else {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Ok,
            "src_manifest_failed",
        );
    };
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    if declarations.as_slice() != RUSTC_WASM_C6DCCF3E_IMPORTS {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Ok,
            "imports_mismatch",
        );
    }
    let sysroot_mount_hash = match sysroot_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Ok,
                "sysroot_manifest_invalid",
            )
        }
    };
    let src_mount_hash = match src_manifest.sha256() {
        Ok(hash) => hash,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Ok,
                "src_manifest_invalid",
            )
        }
    };
    let authorized = match AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
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
    }) {
        Ok(authorized) => authorized,
        Err(denied) => {
            return RustcRunEvidence::at(
                RustcRunStage::Reassembled,
                CompilerFileSha::Ok,
                denied.reason(),
            )
        }
    };
    drop(declarations);
    drop(observed);
    if authorized.compiler_artifact_sha256() != expected_file_sha256
        || authorized.sysroot_mount_manifest_sha256() != sysroot_mount_hash
        || authorized.src_mount_manifest_sha256() != src_mount_hash
        || authorized.authorized_import_count() != REQUIRED_IMPORT_COUNT
    {
        return RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Ok,
            "gate_binding_mismatch",
        );
    }

    let class = *authorized.guest_class();
    let instance = match build_instance_with_process(
        &authorized,
        class,
        &sysroot_manifest,
        &src_manifest,
        rustcrun_args(),
        Vec::new(),
    ) {
        Some(instance) => instance,
        None => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "build_instance_failed",
            )
        }
    };
    let Some(authority) = storage_authority(&authorized, &sysroot_manifest, &src_manifest) else {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "storage_authority_failed",
        );
    };
    let Some(nonce) = BuildRunNonce::kernel_minted(RUSTCRUN_INSTANCE_NONCE) else {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "instance_nonce_failed",
        );
    };
    let reader = match materialize_build_storage(
        &authority,
        &sysroot_manifest,
        &src_manifest,
        nonce,
        Box::new(shared_store),
    ) {
        Ok(reader) => reader,
        Err(error) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                error.reason(),
            )
        }
    };
    let expected_sysroot_chunks = match sysimport_manifest_chunk_count(&sysroot_manifest) {
        Some(count) => count,
        None => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "sysroot_chunk_count_overflow",
            )
        }
    };
    if reader.entry_count() != expected_sysroot_chunks
        || reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != RUSTCRUN_INSTANCE_NONCE
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "instance_reader_binding_mismatch",
        );
    }

    let mut store = Store::new(
        &engine,
        WasiHostState::new(
            instance,
            reader,
            ThreadHostMode::Scheduled(ThreadWorld::new(class.thread_cap)),
        ),
    );
    store.limiter(|state| state);
    let memory_type = match MemoryType::new_shared(
        class.shared_memory.initial_pages,
        class.shared_memory.max_pages,
    ) {
        Ok(memory_type) => memory_type,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "memory_type_invalid",
            )
        }
    };
    let memory = match Memory::new(&mut store, memory_type) {
        Ok(memory) => memory,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "memory_allocation_failed",
            )
        }
    };
    if u32::from(memory.current_pages(&store)) != class.shared_memory.initial_pages {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "memory_initial_mismatch",
        );
    }
    let reserve_pages = match class
        .shared_memory
        .max_pages
        .checked_sub(class.shared_memory.initial_pages)
        .and_then(Pages::new)
    {
        Some(pages) => pages,
        None => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "memory_reservation_invalid",
            )
        }
    };
    let previous = match memory.grow(&mut store, reserve_pages) {
        Ok(previous) => previous,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "memory_reservation_failed",
            )
        }
    };
    if u32::from(previous) != class.shared_memory.initial_pages
        || u32::from(memory.current_pages(&store)) != class.shared_memory.max_pages
    {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "memory_reservation_mismatch",
        );
    }
    store.data_mut().install_memory(memory);

    let mut linker = Linker::<WasiHostState>::new(&engine);
    let registered = match define_wasi_imports(&mut linker, memory) {
        Ok(registered) => registered,
        Err(()) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "linker_definition_failed",
            )
        }
    };
    if registered != REQUIRED_IMPORT_COUNT
        || registered != RUSTC_WASM_C6DCCF3E_IMPORTS.len()
        || registered != authorized.authorized_import_count()
    {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "linker_count_mismatch",
        );
    }
    let pre = match linker.instantiate(&mut store, &module) {
        Ok(pre) => pre,
        Err(_) => {
            return RustcRunEvidence::at(
                RustcRunStage::Instantiated,
                CompilerFileSha::Ok,
                "instantiate_failed",
            )
        }
    };
    // The start section runs non-resumably under pre.start; give it a generous
    // one-shot budget (bounded init code) so it can complete, then reset the
    // store's remaining fuel to 0 so the merged pump installs its per-thread
    // escrows from a clean baseline (ADR 0022 §3). replace_remaining_fuel
    // preserves fuel_consumed, so the logical clock stays continuous.
    let _ = store.replace_remaining_fuel(RUSTC_BUILD_GUEST_CLASS_V1.max_total_fuel);
    let wasm_instance = match pre.start(&mut store) {
        Ok(instance) => instance,
        Err(error) => return rustcrun_pre_start_failure(&store, &error),
    };
    let _ = store.replace_remaining_fuel(0);
    let Some(main) = wasm_instance.get_func(&store, "_start") else {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "start_export_missing",
        );
    };
    let main_ty = main.ty(&store);
    if !main_ty.params().is_empty() || !main_ty.results().is_empty() {
        return RustcRunEvidence::at(
            RustcRunStage::Instantiated,
            CompilerFileSha::Ok,
            "start_export_type",
        );
    }
    let runner = match WasiThreadJobRunner::new(store, memory, module, linker, main, class) {
        Ok(runner) => runner,
        Err(failure) => {
            return RustcRunEvidence::at(
                RustcRunStage::Started,
                CompilerFileSha::Ok,
                rustcrun_pump_failure(failure),
            )
        }
    };
    RustcRunEvidence::completed(runner.run())
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

fn run_wasi_thread_fixture_once() -> Result<WasiThreadRunEvidence, ()> {
    let engine = wasi_engine();
    let module = Module::new(&engine, WASI_THREAD_FIXTURE_WASM).map_err(|_| ())?;
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    let sysroot_manifest = empty_manifest().ok_or(())?;
    let src_manifest = empty_manifest().ok_or(())?;
    let request = AuthorizedBuildJobRequest {
        wasi_grant: ScopedWasiBuildGrant {
            compiler_artifact_sha256: COMPILER_ARTIFACT_SHA256,
            job_manifest_sha256: JOB_MANIFEST_SHA256,
            inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
            declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
        },
        observed_imports: &declarations,
        guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
        sysroot_mount_manifest_sha256: sysroot_manifest.sha256().map_err(|_| ())?,
        src_mount_manifest_sha256: src_manifest.sha256().map_err(|_| ())?,
    };
    let authorized = AuthorizedBuildJob::authorize(request).map_err(|_| ())?;
    let class = *authorized.guest_class();
    let authority = storage_authority(&authorized, &sysroot_manifest, &src_manifest).ok_or(())?;
    let nonce = BuildRunNonce::kernel_minted(WASI_THREAD_RUN_NONCE).ok_or(())?;
    let reader = materialize_build_storage(
        &authority,
        &sysroot_manifest,
        &src_manifest,
        nonce,
        Box::new(UnbackedChunkStore::new(
            BUILD_STORE_INSTANCE_ID,
            BUILD_STORE_GENERATION,
        )),
    )
    .map_err(|_| ())?;
    if reader.entry_count() != 0
        || reader.job_binding_sha256() != authority.job_binding_sha256()
        || reader.run_nonce() != WASI_THREAD_RUN_NONCE
        || reader.store_generation() != BUILD_STORE_GENERATION
    {
        return Err(());
    }
    let instance = build_instance(&authorized, class, &sysroot_manifest, &src_manifest).ok_or(())?;
    let mut store = Store::new(
        &engine,
        WasiHostState::new(
            instance,
            reader,
            ThreadHostMode::Scheduled(ThreadWorld::new(class.thread_cap)),
        ),
    );
    store.limiter(|state| state);
    let memory_type = MemoryType::new_shared(
        class.shared_memory.initial_pages,
        class.shared_memory.max_pages,
    )
    .map_err(|_| ())?;
    let memory = Memory::new(&mut store, memory_type).map_err(|_| ())?;
    if u32::from(memory.current_pages(&store)) != class.shared_memory.initial_pages {
        return Err(());
    }
    let reserve_pages = class
        .shared_memory
        .max_pages
        .checked_sub(class.shared_memory.initial_pages)
        .and_then(Pages::new)
        .ok_or(())?;
    let previous = memory.grow(&mut store, reserve_pages).map_err(|_| ())?;
    if u32::from(previous) != class.shared_memory.initial_pages
        || u32::from(memory.current_pages(&store)) != class.shared_memory.max_pages
    {
        return Err(());
    }
    store.data_mut().install_memory(memory);

    let mut linker = Linker::<WasiHostState>::new(&engine);
    let registered = define_wasi_imports(&mut linker, memory).map_err(|_| ())?;
    if registered != REQUIRED_IMPORT_COUNT
        || registered != RUSTC_WASM_C6DCCF3E_IMPORTS.len()
        || registered != authorized.authorized_import_count()
    {
        return Err(());
    }
    let pre = linker.instantiate(&mut store, &module).map_err(|_| ())?;
    let instance = pre.start(&mut store).map_err(|_| ())?;
    let main = instance.get_func(&store, "_start").ok_or(())?;
    let runner =
        WasiThreadJobRunner::new(store, memory, module, linker, main, class).map_err(|_| ())?;
    Ok(runner.run())
}

fn scheduled_thread_cap_boundary() -> bool {
    let class = RUSTC_BUILD_GUEST_CLASS_V1;
    let mut world = ThreadWorld::new(class.thread_cap);
    let Ok(main) = world.scheduler.spawn() else {
        return false;
    };
    if main.get() != 0 {
        return false;
    }
    for start_arg in 0..class.thread_cap.saturating_sub(1) {
        if world.reserve_spawn(start_arg as i32) < 0 {
            return false;
        }
    }
    world.reserve_spawn(-1) == -1
        && world.pending_spawns.len() == class.thread_cap.saturating_sub(1) as usize
        && world.spawns == class.thread_cap.saturating_sub(1)
        && world.cap_denials == 1
}

struct WasiThreadBusyGuard;

impl WasiThreadBusyGuard {
    fn acquire() -> Option<Self> {
        try_acquire_thread_job_execution().then_some(Self)
    }
}

impl Drop for WasiThreadBusyGuard {
    fn drop(&mut self) {
        release_thread_job_execution();
    }
}

pub(crate) fn emit_wasi_thread_selftest() {
    const EXPECTED_STDOUT: &[u8] = b"main\nworker\n";
    let Some(_busy) = WasiThreadBusyGuard::acquire() else {
        crate::serial::write_raw_fmt(format_args!(
            "RAIOS_WASITHREAD selftest=fail spawns=0 trace_det=0 effect_det=0 stdout_bytes=0 exit_code={}\n",
            u32::MAX,
        ));
        return;
    };
    let first = run_wasi_thread_fixture_once();
    let second = run_wasi_thread_fixture_once();
    let trace_deterministic = matches!(
        (&first, &second),
        (Ok(first), Ok(second))
            if first.trace_digest != [0; 32]
                && first.trace_digest == second.trace_digest
                && first.rounds == second.rounds
    );
    let effect_deterministic = matches!(
        (&first, &second),
        (Ok(first), Ok(second))
            if first.effect_digest != [0; 32]
                && first.effect_digest == second.effect_digest
                && first.effect_count == 2
                && second.effect_count == 2
    );
    let stdout_deterministic = matches!(
        (&first, &second),
        (Ok(first), Ok(second))
            if first.stdout == second.stdout && first.stdout == EXPECTED_STDOUT
    );
    let runs_pass = matches!(
        (&first, &second),
        (Ok(first), Ok(second))
            if first.end == (WasiThreadJobEnd::JobExited { code: 0 })
                && second.end == (WasiThreadJobEnd::JobExited { code: 0 })
                && first.spawns == 1
                && second.spawns == 1
                && first.cap_denials == 0
                && second.cap_denials == 0
                && first.granted_total == RUSTC_BUILD_GUEST_CLASS_V1.fuel_quantum * 2
                && second.granted_total == first.granted_total
    );
    let pass = runs_pass
        && trace_deterministic
        && effect_deterministic
        && stdout_deterministic
        && scheduled_thread_cap_boundary();
    let (spawns, stdout_bytes, exit_code) = match &first {
        Ok(evidence) => (
            evidence.spawns,
            evidence.stdout.len(),
            match evidence.end {
                WasiThreadJobEnd::JobExited { code } => code,
                WasiThreadJobEnd::JobDeadlocked | WasiThreadJobEnd::Failed(_) => u32::MAX,
            },
        ),
        Err(()) => (0, 0, u32::MAX),
    };
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_WASITHREAD selftest={} spawns={} trace_det={} effect_det={} stdout_bytes={} exit_code={}\n",
        if pass { "pass" } else { "fail" },
        spawns,
        u8::from(trace_deterministic),
        u8::from(effect_deterministic),
        stdout_bytes,
        exit_code,
    ));
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

pub(crate) fn emit_wasi_compilerload() {
    let evidence = run_compilerload();
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_COMPILERLOAD stage={} bytes={} file_sha={} imports={} mem_pages={} reason={}\n",
        evidence.stage.token(),
        evidence.bytes,
        evidence.file_sha.token(),
        evidence.imports,
        evidence.mem_pages,
        evidence.reason,
    ));
}

fn emit_rustcrun_evidence(evidence: RustcRunEvidence) {
    match evidence.exit_code {
        Some(exit_code) => crate::serial::write_raw_fmt(format_args!(
            "RAIOS_RUSTCRUN stage={} file_sha={} spawns={} cap_denials={} rounds={} stdout_bytes={} granted_total={} exit_code={} reason={}\n",
            evidence.stage.token(),
            evidence.file_sha.token(),
            evidence.spawns,
            evidence.cap_denials,
            evidence.rounds,
            evidence.stdout_bytes,
            evidence.granted_total,
            exit_code,
            evidence.reason,
        )),
        None => crate::serial::write_raw_fmt(format_args!(
            "RAIOS_RUSTCRUN stage={} file_sha={} spawns={} cap_denials={} rounds={} stdout_bytes={} granted_total={} exit_code=na reason={}\n",
            evidence.stage.token(),
            evidence.file_sha.token(),
            evidence.spawns,
            evidence.cap_denials,
            evidence.rounds,
            evidence.stdout_bytes,
            evidence.granted_total,
            evidence.reason,
        )),
    }
}

pub(crate) fn emit_wasi_rustcrun() {
    let Some(_busy) = WasiThreadBusyGuard::acquire() else {
        emit_rustcrun_evidence(RustcRunEvidence::at(
            RustcRunStage::Reassembled,
            CompilerFileSha::Mismatch,
            "runner_busy",
        ));
        return;
    };
    emit_rustcrun_evidence(run_rustcrun());
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
