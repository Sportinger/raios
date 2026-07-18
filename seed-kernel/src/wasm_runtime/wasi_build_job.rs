use alloc::vec::Vec;

use raios_core::{
    authorized_build_job::{AuthorizedBuildJob, AuthorizedBuildJobRequest, BuildJobDenied},
    build_guest_class::{BuildGuestClassV1, RUSTC_BUILD_GUEST_CLASS_V1},
    buildfs_manifest::{BuildFsManifest, BUILD_FS_CHUNK_SIZE},
    scoped_wasi_build_grant::{ScopedWasiBuildGrant, RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256},
    wasi_preview1_import_abi::{
        WasiImportDeclaration, WasiImportKind, WasiValueType, RUSTC_WASM_C6DCCF3E_IMPORTS,
    },
};
use raios_wasi_preview1::{
    ramfs::RamQuotas, BuildFs, BuildFsManifestView, JobContext, WasiBuildInstance, WasiBuildLimits,
};
use wasmi::{
    core::{Trap, ValueType},
    Config, Engine, ExternType, Linker, Memory, MemoryType, Module, Mutability, ResumableCall,
    Store, Suspension,
};

use super::wasi_preview1::{define_wasi_imports, ProcExitTrap, WasiHostState};

const WASI_BUILD_OK_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_ok.wasm"));
const WASI_BUILD_EXTRA_IMPORT_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wasi_build_extra_import.wasm"));
const COMPILER_ARTIFACT_SHA256: &str =
    "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
const JOB_MANIFEST_SHA256: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const REQUIRED_IMPORT_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WasiJobEnd {
    ProcExit(u32),
    Trap,
    Denied(BuildJobDenied),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WasiJobEvidence {
    end: WasiJobEnd,
    instantiated: bool,
    stdout_bytes: u64,
    registered: usize,
    frozen_output_entries: usize,
}

impl WasiJobEvidence {
    const fn terminal(end: WasiJobEnd) -> Self {
        Self {
            end,
            instantiated: false,
            stdout_bytes: 0,
            registered: 0,
            frozen_output_entries: 0,
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

struct EmptyManifest;

impl BuildFsManifestView for EmptyManifest {
    fn chunk_size(&self) -> u64 {
        BUILD_FS_CHUNK_SIZE
    }

    fn directory_count(&self) -> usize {
        0
    }

    fn directory_path(&self, _index: usize) -> Option<&str> {
        None
    }

    fn file_count(&self) -> usize {
        0
    }

    fn file_path(&self, _file: usize) -> Option<&str> {
        None
    }

    fn file_len(&self, _file: usize) -> Option<u64> {
        None
    }

    fn file_sha256(&self, _file: usize) -> Option<[u8; 32]> {
        None
    }

    fn chunk_count(&self, _file: usize) -> Option<usize> {
        None
    }

    fn chunk_len(&self, _file: usize, _chunk: usize) -> Option<u64> {
        None
    }

    fn chunk_sha256(&self, _file: usize, _chunk: usize) -> Option<[u8; 32]> {
        None
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

fn run_build_job(bytes: &[u8]) -> WasiJobEvidence {
    let engine = wasi_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let observed = observed_imports(&module);
    let declarations: Vec<_> = observed.iter().map(ObservedImport::declaration).collect();
    let empty_mount_hash = match empty_manifest_hash() {
        Some(hash) => hash,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
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
        sysroot_mount_manifest_sha256: empty_mount_hash,
        src_mount_manifest_sha256: empty_mount_hash,
    };
    let authorized = match AuthorizedBuildJob::authorize(request) {
        Ok(authorized) => authorized,
        Err(denied) => return WasiJobEvidence::terminal(WasiJobEnd::Denied(denied)),
    };
    instantiate_authorized(engine, module, authorized)
}

fn instantiate_authorized(
    engine: Engine,
    module: Module,
    authorized: AuthorizedBuildJob,
) -> WasiJobEvidence {
    let class = *authorized.guest_class();
    let instance = match build_instance(&authorized, class) {
        Some(instance) => instance,
        None => return WasiJobEvidence::terminal(WasiJobEnd::Trap),
    };
    let mut store = Store::new(&engine, WasiHostState::new(instance));
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
    let (end, frozen_output_entries) = match store.data().freeze_output_entries() {
        Ok(entries) => (end, entries),
        Err(_) => (WasiJobEnd::Trap, 0),
    };
    WasiJobEvidence {
        end,
        instantiated: true,
        stdout_bytes: store.data().stdout_bytes(),
        registered,
        frozen_output_entries,
    }
}

fn run_start(
    store: &mut Store<WasiHostState>,
    start: wasmi::Func,
    class: BuildGuestClassV1,
) -> WasiJobEnd {
    if add_next_quantum(store, class).is_err() {
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
                if add_next_quantum(store, class).is_err() {
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
) -> Result<(), Trap> {
    let consumed = store
        .fuel_consumed()
        .ok_or_else(|| Trap::new("fuel disabled"))?;
    let remaining = class
        .max_total_fuel
        .checked_sub(consumed)
        .filter(|remaining| *remaining != 0)
        .ok_or_else(|| Trap::new("WASI build fuel ceiling reached"))?;
    store
        .add_fuel(class.fuel_quantum.min(remaining))
        .map_err(|_| Trap::new("WASI build fuel refill failed"))
}

fn build_instance(
    authorized: &AuthorizedBuildJob,
    class: BuildGuestClassV1,
) -> Option<WasiBuildInstance> {
    let sysroot = BuildFs::project(&EmptyManifest).ok()?;
    let source = BuildFs::project(&EmptyManifest).ok()?;
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

fn empty_manifest_hash() -> Option<[u8; 32]> {
    BuildFsManifest::new(Vec::new(), Vec::new())
        .ok()?
        .sha256()
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

pub(crate) fn emit_wasi_selftest() {
    let positive = run_build_job(WASI_BUILD_OK_WASM);
    let negative = run_build_job(WASI_BUILD_EXTRA_IMPORT_WASM);
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
        && !negative.instantiated
        && matches!(
            negative.end,
            WasiJobEnd::Denied(BuildJobDenied::ImportsMismatch { .. })
        )
        && deny == "imports_mismatch";
    crate::serial::write_raw_fmt(format_args!(
        "RAIOS_WASI selftest={} ok_exit={} ok_stdout={} deny={} registered={}\n",
        if pass { "pass" } else { "fail" },
        ok_exit,
        positive.stdout_bytes,
        deny,
        positive.registered,
    ));
}
