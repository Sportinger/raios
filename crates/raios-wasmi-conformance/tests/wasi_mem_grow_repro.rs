use std::{fmt, fs, path::PathBuf};

use wasmi::{
    core::{HostError, Trap, TrapCode},
    errors::{MemoryError, TableError},
    Caller, Config, Engine, Error, Linker, Memory, MemoryType, Module, ResourceLimiter,
    ResumableCall, Store, Suspension,
};

// Mirrors seed-kernel/src/wasm_runtime/wasi_build_job.rs lines 60-70 and
// inspect_memory_evidence at lines 391-424. Keep these copies visibly tied to
// the kernel evidence ABI instead of deriving them from the fixture.
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

// Mirrors RUSTC_BUILD_GUEST_CLASS_V1 in
// crates/raios-core/src/build_guest_class.rs lines 151-158. These are the
// values passed to the kernel runner whose loop is reproduced below.
const INITIAL_PAGES: u32 = 399;
const MAX_PAGES: u32 = 16_384;
const FUEL_QUANTUM: u64 = 1_000_000;
const MAX_TOTAL_FUEL: u64 = 10_000_000_000_000;
// Diagnostic control only: larger than the fixture's total dynamic grow cost
// (15,985 pages * 64 KiB / 64 bytes-per-fuel). Kernel-faithful profiles always
// use FUEL_QUANTUM above.
const WHOLE_FIXTURE_FUEL_QUANTUM: u64 = 32 * 1024 * 1024;
const WASM_PAGE_BYTES: usize = 64 * 1024;
const CONSTRAINED_FREE_BYTES: usize = 200 * 1024 * 1024;
const REQUIRED_IMPORT_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LimiterReason {
    WithinMaximumAndFree,
    DesiredWithinCurrent,
    DeclaredMaximum,
    RequiredFreeOverflow,
    FreeBudget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GrowDecision {
    current: usize,
    desired: usize,
    maximum: Option<usize>,
    required_free: Option<usize>,
    free: usize,
    verdict: bool,
    reason: LimiterReason,
}

#[derive(Debug)]
struct HostState {
    free_bytes: usize,
    grow_log: Vec<GrowDecision>,
    memory: Option<Memory>,
    stdout: Vec<u8>,
    stdout_bytes: u64,
    terminal_exit: Option<u32>,
}

impl HostState {
    fn new(free_bytes: usize) -> Self {
        Self {
            free_bytes,
            grow_log: Vec::new(),
            memory: None,
            stdout: Vec::new(),
            stdout_bytes: 0,
            terminal_exit: None,
        }
    }
}

// Mirrors WasiHostState::memory_growing in wasi_build_job.rs lines 207-228:
// declared maximum first, then desired+current+1 MiB <= allocator free().
impl ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, MemoryError> {
        let (verdict, reason, required_free) = if maximum.is_some_and(|max| desired > max) {
            (false, LimiterReason::DeclaredMaximum, None)
        } else if desired <= current {
            (true, LimiterReason::DesiredWithinCurrent, None)
        } else {
            match desired
                .checked_add(current)
                .and_then(|bytes| bytes.checked_add(MEMORY_GROW_SAFETY_MARGIN_BYTES))
            {
                None => (false, LimiterReason::RequiredFreeOverflow, None),
                Some(required) if required > self.free_bytes => {
                    (false, LimiterReason::FreeBudget, Some(required))
                }
                Some(required) => (true, LimiterReason::WithinMaximumAndFree, Some(required)),
            }
        };
        self.grow_log.push(GrowDecision {
            current,
            desired,
            maximum,
            required_free,
            free: self.free_bytes,
            verdict,
            reason,
        });
        Ok(verdict)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        maximum: Option<u32>,
    ) -> Result<bool, TableError> {
        Ok(maximum.is_none_or(|max| desired <= max))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProcExitTrap {
    code: u32,
}

impl fmt::Display for ProcExitTrap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "proc_exit({})", self.code)
    }
}

impl HostError for ProcExitTrap {}

#[derive(Debug, Eq, PartialEq)]
enum RunEnd {
    ProcExit(u32),
    Trap(String),
    FuelExhaustion { consumed: u64, display: String },
    FinishedWithoutProcExit,
}

impl fmt::Display for RunEnd {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcExit(code) => write!(formatter, "proc_exit({code})"),
            Self::Trap(display) => write!(formatter, "trap: {display}"),
            Self::FuelExhaustion { consumed, display } => {
                write!(formatter, "fuel_exhaustion(consumed={consumed}): {display}")
            }
            Self::FinishedWithoutProcExit => formatter.write_str("finished without proc_exit"),
        }
    }
}

#[derive(Debug)]
struct Observation {
    profile: &'static str,
    free_bytes: usize,
    grow_log: Vec<GrowDecision>,
    final_pages: u32,
    control: [u8; MEMORY_CONTROL_BYTES],
    stdout: Vec<u8>,
    stdout_bytes: u64,
    end: RunEnd,
    fuel_quantum: u64,
    fuel_consumed: u64,
    fuel_suspensions: usize,
    granted_total: u64,
    fuel_parks: Vec<FuelPark>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FuelPark {
    completed_grow_attempts: usize,
    consumed: u64,
    granted_total: u64,
}

impl Observation {
    fn control_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.control[offset..offset + 4]
                .try_into()
                .expect("control field is four bytes"),
        )
    }

    fn growth_log(&self) -> &[GrowDecision] {
        // Memory::new invokes the limiter for the imported memory's initial
        // allocation. All later entries correspond one-for-one to the
        // fixture's memory.grow instructions.
        &self.grow_log[1..]
    }

    fn successful_growths(&self) -> usize {
        self.growth_log()
            .iter()
            .filter(|decision| {
                decision.verdict && decision.desired <= self.final_pages as usize * WASM_PAGE_BYTES
            })
            .count()
    }

    fn dump(&self) {
        eprintln!(
            "\n=== wasi_mem_grow profile={} free={} fuel_quantum={} ===",
            self.profile, self.free_bytes, self.fuel_quantum
        );
        for (index, decision) in self.grow_log.iter().enumerate() {
            let kind = if index == 0 { "create" } else { "memory.grow" };
            let actual = if index == 0 {
                "created"
            } else if decision.verdict
                && decision.desired <= self.final_pages as usize * WASM_PAGE_BYTES
            {
                "grew"
            } else if decision.verdict {
                "returned -1 after limiter allowed"
            } else {
                "returned -1 after limiter denied"
            };
            eprintln!(
                "limiter[{index:03}] kind={kind} current={} desired={} maximum={:?} required_free={:?} free={} verdict={} reason={:?} actual={actual}",
                decision.current,
                decision.desired,
                decision.maximum,
                decision.required_free,
                decision.free,
                decision.verdict,
                decision.reason,
            );
        }
        eprintln!("final_pages={}", self.final_pages);
        eprintln!("control[28]={:02x?}", self.control);
        eprintln!(
            "control_fields initial={} final={} step_count={} grow_denied={} stdout_iovec_ptr={} stdout_iovec_len={} stdout_written={}",
            self.control_u32(MEMORY_INITIAL_PAGES_OFFSET),
            self.control_u32(MEMORY_FINAL_PAGES_OFFSET),
            self.control_u32(MEMORY_GROW_STEP_COUNT_OFFSET),
            self.control_u32(MEMORY_GROW_DENIED_OFFSET),
            self.control_u32(MEMORY_STDOUT_IOVEC_PTR_OFFSET),
            self.control_u32(MEMORY_STDOUT_IOVEC_LEN_OFFSET),
            self.control_u32(MEMORY_STDOUT_WRITTEN_OFFSET),
        );
        eprintln!(
            "stdout_bytes={} stdout={:02x?} utf8={:?}",
            self.stdout_bytes,
            self.stdout,
            String::from_utf8_lossy(&self.stdout),
        );
        eprintln!(
            "end={} fuel_consumed={} fuel_suspensions={} granted_total={} fuel_parks={:?}",
            self.end,
            self.fuel_consumed,
            self.fuel_suspensions,
            self.granted_total,
            self.fuel_parks,
        );
        if matches!(self.end, RunEnd::FuelExhaustion { .. }) {
            eprintln!(
                "terminal_memory.grow outcome=trap_before_limiter current={} desired={} (no limiter entry)",
                self.final_pages as usize * WASM_PAGE_BYTES,
                (self.final_pages + MEMORY_GROW_STEP_PAGES) as usize * WASM_PAGE_BYTES,
            );
        }
    }
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../seed-kernel/fixtures/wasi_mem_grow.wat")
}

fn real_fixture_wat() -> String {
    fs::read_to_string(fixture_path()).expect("real wasi_mem_grow.wat fixture must be readable")
}

// Mirrors wasi_engine in wasi_build_job.rs lines 198-205.
fn kernel_engine() -> Engine {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    Engine::new(&config)
}

fn default_fuel_engine() -> Engine {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(false);
    Engine::new(&config)
}

fn unexpected_import(name: &'static str) -> Trap {
    Trap::new(format!("unexpected fixture import called: {name}"))
}

macro_rules! trap_i32_stub {
    ($linker:ident, $module:literal, $name:literal, ($($arg:ident : $ty:ty),* $(,)?)) => {{
        $linker
            .func_wrap($module, $name, |$($arg: $ty),*| -> Result<i32, Trap> {
                let _ = ($($arg,)*);
                Err(unexpected_import($name))
            })
            .expect(concat!("define ", $module, ".", $name));
    }};
}

fn define_fixture_imports(linker: &mut Linker<HostState>, memory: Memory) {
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "random_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "args_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "args_sizes_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "clock_time_get", (a: i32, b: i64, c: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_filestat_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_read", (a: i32, b: i32, c: i32, d: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_readdir", (a: i32, b: i32, c: i32, d: i64, e: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_seek", (a: i32, b: i64, c: i32, d: i32));
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |mut caller: Caller<'_, HostState>,
             fd: i32,
             iovs: i32,
             iovs_len: i32,
             nwritten: i32|
             -> Result<i32, Trap> {
                if fd != 1 {
                    return Err(Trap::new(format!("fd_write used unexpected fd {fd}")));
                }
                let memory = caller
                    .data()
                    .memory
                    .ok_or_else(|| Trap::new("fd_write called before memory installation"))?;
                let iovs = iovs as u32 as usize;
                let iovs_len = iovs_len as u32 as usize;
                let mut output = Vec::new();
                for index in 0..iovs_len {
                    let iovec_offset = index
                        .checked_mul(8)
                        .and_then(|offset| iovs.checked_add(offset))
                        .ok_or_else(|| Trap::new("fd_write iovec offset overflow"))?;
                    let mut iovec = [0_u8; 8];
                    memory
                        .read(&caller, iovec_offset, &mut iovec)
                        .map_err(|error| {
                            Trap::new(format!("fd_write iovec read failed: {error}"))
                        })?;
                    let ptr = u32::from_le_bytes(iovec[0..4].try_into().unwrap()) as usize;
                    let len = u32::from_le_bytes(iovec[4..8].try_into().unwrap()) as usize;
                    let old_len = output.len();
                    output
                        .try_reserve(len)
                        .map_err(|_| Trap::new("fd_write output allocation failed"))?;
                    output.resize(old_len + len, 0);
                    memory
                        .read(&caller, ptr, &mut output[old_len..])
                        .map_err(|error| {
                            Trap::new(format!("fd_write payload read failed: {error}"))
                        })?;
                }
                let written = u32::try_from(output.len())
                    .map_err(|_| Trap::new("fd_write byte count does not fit u32"))?;
                memory
                    .write(
                        &mut caller,
                        nwritten as u32 as usize,
                        &written.to_le_bytes(),
                    )
                    .map_err(|error| Trap::new(format!("fd_write nwritten failed: {error}")))?;
                let state = caller.data_mut();
                state.stdout_bytes = state
                    .stdout_bytes
                    .checked_add(u64::from(written))
                    .ok_or_else(|| Trap::new("fd_write stdout counter overflow"))?;
                state.stdout.extend_from_slice(&output);
                Ok(0)
            },
        )
        .expect("define wasi_snapshot_preview1.fd_write");
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_create_directory", (a: i32, b: i32, c: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_filestat_get", (a: i32, b: i32, c: i32, d: i32, e: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_link", (a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_open", (a: i32, b: i32, c: i32, d: i32, e: i32, f: i64, g: i64, h: i32, i: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_readlink", (a: i32, b: i32, c: i32, d: i32, e: i32, f: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_remove_directory", (a: i32, b: i32, c: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_rename", (a: i32, b: i32, c: i32, d: i32, e: i32, f: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "path_unlink_file", (a: i32, b: i32, c: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "poll_oneoff", (a: i32, b: i32, c: i32, d: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "sched_yield", ());
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "environ_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "environ_sizes_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_close", (a: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_fdstat_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_filestat_set_size", (a: i32, b: i64));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_pread", (a: i32, b: i32, c: i32, d: i64, e: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_prestat_get", (a: i32, b: i32));
    trap_i32_stub!(linker, "wasi_snapshot_preview1", "fd_prestat_dir_name", (a: i32, b: i32, c: i32));
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "proc_exit",
            |mut caller: Caller<'_, HostState>, code: i32| -> Result<(), Trap> {
                let code = code as u32;
                caller.data_mut().terminal_exit = Some(code);
                Err(ProcExitTrap { code }.into())
            },
        )
        .expect("define wasi_snapshot_preview1.proc_exit");
    trap_i32_stub!(linker, "wasi", "thread-spawn", (a: i32));
    linker
        .define("env", "memory", memory)
        .expect("define exact shared env.memory import");
}

fn classify_error(store: &Store<HostState>, error: Error) -> RunEnd {
    if matches!(
        &error,
        Error::Trap(trap) if matches!(trap.trap_code(), Some(TrapCode::OutOfFuel))
    ) {
        return RunEnd::FuelExhaustion {
            consumed: store.fuel_consumed().unwrap_or(0),
            display: error.to_string(),
        };
    }
    RunEnd::Trap(error.to_string())
}

fn add_next_quantum(
    store: &mut Store<HostState>,
    fuel_quantum: u64,
    granted_total: &mut u64,
) -> Result<(), &'static str> {
    let remaining = MAX_TOTAL_FUEL
        .checked_sub(*granted_total)
        .filter(|remaining| *remaining != 0)
        .ok_or("WASI build fuel ceiling reached")?;
    let grant = fuel_quantum.min(remaining);
    store
        .add_fuel(grant)
        .map_err(|_| "WASI build fuel refill failed")?;
    *granted_total = granted_total
        .checked_add(grant)
        .ok_or("WASI build fuel ceiling reached")?;
    Ok(())
}

// Mirrors run_start and add_next_quantum in wasi_build_job.rs lines 449-505:
// initial quantum, dynamic resumable call, fresh quantum only after
// FuelQuantum, and typed ProcExitTrap recognition from host suspension.
fn run_start(
    store: &mut Store<HostState>,
    start: wasmi::Func,
    fuel_quantum: u64,
) -> (RunEnd, usize, u64, Vec<FuelPark>) {
    let mut granted_total = 0;
    let mut fuel_parks = Vec::new();
    if let Err(display) = add_next_quantum(store, fuel_quantum, &mut granted_total) {
        return (
            RunEnd::FuelExhaustion {
                consumed: store.fuel_consumed().unwrap_or(0),
                display: display.to_string(),
            },
            0,
            granted_total,
            fuel_parks,
        );
    }
    let mut outputs = [];
    let mut outcome = match start.call_resumable(&mut *store, &[], &mut outputs) {
        Ok(outcome) => outcome,
        Err(error) => return (classify_error(store, error), 0, granted_total, fuel_parks),
    };
    let mut fuel_suspensions = 0;
    loop {
        let invocation = match outcome {
            ResumableCall::Finished => {
                return (
                    RunEnd::FinishedWithoutProcExit,
                    fuel_suspensions,
                    granted_total,
                    fuel_parks,
                )
            }
            ResumableCall::Resumable(invocation) => invocation,
        };
        match invocation.suspension() {
            Suspension::FuelQuantum => {
                fuel_suspensions += 1;
                fuel_parks.push(FuelPark {
                    completed_grow_attempts: store.data().grow_log.len().saturating_sub(1),
                    consumed: store.fuel_consumed().unwrap_or(0),
                    granted_total,
                });
                if let Err(display) = add_next_quantum(store, fuel_quantum, &mut granted_total) {
                    return (
                        RunEnd::FuelExhaustion {
                            consumed: store.fuel_consumed().unwrap_or(0),
                            display: display.to_string(),
                        },
                        fuel_suspensions,
                        granted_total,
                        fuel_parks,
                    );
                }
                outcome = match invocation.resume(&mut *store, &[], &mut outputs) {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        return (
                            classify_error(store, error),
                            fuel_suspensions,
                            granted_total,
                            fuel_parks,
                        )
                    }
                };
            }
            Suspension::Host { host_error, .. } => {
                if let Some(exit) = host_error.downcast_ref::<ProcExitTrap>() {
                    return if store.data().terminal_exit == Some(exit.code) {
                        (
                            RunEnd::ProcExit(exit.code),
                            fuel_suspensions,
                            granted_total,
                            fuel_parks,
                        )
                    } else {
                        (
                            RunEnd::Trap(format!(
                                "proc_exit trap/state mismatch: trap={} state={:?}",
                                exit.code,
                                store.data().terminal_exit
                            )),
                            fuel_suspensions,
                            granted_total,
                            fuel_parks,
                        )
                    };
                }
                return (
                    RunEnd::Trap(host_error.to_string()),
                    fuel_suspensions,
                    granted_total,
                    fuel_parks,
                );
            }
            Suspension::Atomic(suspension) => {
                return (
                    RunEnd::Trap(format!("unexpected atomic suspension: {suspension:?}")),
                    fuel_suspensions,
                    granted_total,
                    fuel_parks,
                )
            }
        }
    }
}

fn run_profile(
    engine: &Engine,
    wasm: &[u8],
    profile: &'static str,
    free_bytes: usize,
    fuel_quantum: u64,
) -> Observation {
    let module = Module::new(engine, wasm).expect("real fixture module must validate");
    assert_eq!(
        module.imports().count(),
        REQUIRED_IMPORT_COUNT,
        "fixture must retain the exact 30-import surface"
    );
    let mut store = Store::new(engine, HostState::new(free_bytes));
    store.limiter(|state| state);
    let memory_type = MemoryType::new_shared(INITIAL_PAGES, MAX_PAGES)
        .expect("399/16384 is a valid shared memory type");
    let memory = Memory::new(&mut store, memory_type)
        .expect("profile must permit the imported memory's initial allocation");
    store.data_mut().memory = Some(memory);
    let mut linker = Linker::new(engine);
    define_fixture_imports(&mut linker, memory);
    let (end, fuel_suspensions, granted_total, fuel_parks) =
        match linker.instantiate(&mut store, &module) {
            Err(error) => (
                RunEnd::Trap(format!("instantiation failed: {error}")),
                0,
                0,
                Vec::new(),
            ),
            Ok(pre) => match pre.start(&mut store) {
                Err(error) => (
                    RunEnd::Trap(format!("start section failed: {error}")),
                    0,
                    0,
                    Vec::new(),
                ),
                Ok(instance) => match instance.get_func(&store, "_start") {
                    None => (
                        RunEnd::Trap("missing _start export".to_string()),
                        0,
                        0,
                        Vec::new(),
                    ),
                    Some(start)
                        if !start.ty(&store).params().is_empty()
                            || !start.ty(&store).results().is_empty() =>
                    {
                        (
                            RunEnd::Trap("_start must have type [] -> []".to_string()),
                            0,
                            0,
                            Vec::new(),
                        )
                    }
                    Some(start) => run_start(&mut store, start, fuel_quantum),
                },
            },
        };
    let final_pages = u32::from(memory.current_pages(&store));
    let mut control = [0_u8; MEMORY_CONTROL_BYTES];
    let control_end = memory.read(&store, 0, &mut control).err();
    let end = match control_end {
        Some(error) => RunEnd::Trap(format!("control block read failed after {end}: {error}")),
        None => end,
    };
    Observation {
        profile,
        free_bytes,
        grow_log: store.data().grow_log.clone(),
        final_pages,
        control,
        stdout: store.data().stdout.clone(),
        stdout_bytes: store.data().stdout_bytes,
        end,
        fuel_quantum,
        fuel_consumed: store.fuel_consumed().unwrap_or(0),
        fuel_suspensions,
        granted_total,
        fuel_parks,
    }
}

fn assert_common_success(observation: &Observation) {
    assert_eq!(observation.end, RunEnd::ProcExit(0));
    assert_eq!(
        observation.control_u32(MEMORY_INITIAL_PAGES_OFFSET),
        INITIAL_PAGES
    );
    assert_eq!(
        observation.control_u32(MEMORY_FINAL_PAGES_OFFSET),
        observation.final_pages
    );
    assert_eq!(
        observation.control_u32(MEMORY_GROW_DENIED_OFFSET),
        1,
        "terminal memory.grow must return -1 and be marked gracefully"
    );
    assert_eq!(
        observation.control_u32(MEMORY_GROW_STEP_COUNT_OFFSET) as usize,
        observation.successful_growths()
    );
    assert_eq!(
        observation.control_u32(MEMORY_STDOUT_IOVEC_LEN_OFFSET),
        observation.stdout.len() as u32
    );
    assert_eq!(
        observation.control_u32(MEMORY_STDOUT_WRITTEN_OFFSET),
        observation.stdout.len() as u32
    );
    assert_eq!(observation.stdout_bytes, observation.stdout.len() as u64);
    assert!(!observation.stdout.is_empty());
    assert!(observation.stdout.len() <= MEMORY_DECIMAL_CAPACITY);
    assert_eq!(
        String::from_utf8(observation.stdout.clone())
            .unwrap()
            .parse::<u32>()
            .unwrap(),
        observation.final_pages
    );
    assert_eq!(
        observation.grow_log[0],
        GrowDecision {
            current: 0,
            desired: INITIAL_PAGES as usize * WASM_PAGE_BYTES,
            maximum: Some(MAX_PAGES as usize * WASM_PAGE_BYTES),
            required_free: Some(
                INITIAL_PAGES as usize * WASM_PAGE_BYTES + MEMORY_GROW_SAFETY_MARGIN_BYTES
            ),
            free: observation.free_bytes,
            verdict: true,
            reason: LimiterReason::WithinMaximumAndFree,
        }
    );
}

fn assert_strict_linker_boundary(engine: &Engine, fixture_wat: &str) {
    let changed = fixture_wat.replacen("\"random_get\"", "\"random_get_extra\"", 1);
    assert_ne!(changed, fixture_wat, "negative fixture mutation must apply");
    let wasm = wat::parse_str(&changed).expect("negative fixture WAT must compile");
    let module = Module::new(engine, wasm.as_slice()).expect("negative fixture must validate");
    let mut store = Store::new(engine, HostState::new(usize::MAX));
    store.limiter(|state| state);
    let memory = Memory::new(
        &mut store,
        MemoryType::new_shared(INITIAL_PAGES, MAX_PAGES).unwrap(),
    )
    .expect("negative fixture initial memory allocation");
    store.data_mut().memory = Some(memory);
    let mut linker = Linker::new(engine);
    define_fixture_imports(&mut linker, memory);
    let error = linker
        .instantiate(&mut store, &module)
        .expect_err("unknown import must be rejected without a fallback")
        .to_string();
    eprintln!("strict_linker_negative={error}");
    assert!(
        error.contains("random_get_extra"),
        "linker error must name the rejected import: {error}"
    );
}

fn assert_default_config_freeze(observation: &Observation) {
    assert_eq!(observation.fuel_quantum, FUEL_QUANTUM);
    assert_eq!(
        observation.end,
        RunEnd::FuelExhaustion {
            consumed: 942_337,
            display: "all fuel consumed by WebAssembly".to_string(),
        }
    );
    assert_eq!(observation.fuel_suspensions, 0);
    assert_eq!(observation.granted_total, FUEL_QUANTUM);
    assert!(observation.fuel_parks.is_empty());
    assert_eq!(
        observation.final_pages,
        INITIAL_PAGES + 8 * MEMORY_GROW_STEP_PAGES
    );
    assert_eq!(
        observation.control_u32(MEMORY_INITIAL_PAGES_OFFSET),
        INITIAL_PAGES
    );
    assert_eq!(
        observation.control_u32(MEMORY_FINAL_PAGES_OFFSET),
        observation.final_pages
    );
    assert_eq!(observation.control_u32(MEMORY_GROW_STEP_COUNT_OFFSET), 8);
    assert_eq!(observation.control_u32(MEMORY_GROW_DENIED_OFFSET), 0);
    assert_eq!(observation.control_u32(MEMORY_STDOUT_IOVEC_PTR_OFFSET), 0);
    assert_eq!(observation.control_u32(MEMORY_STDOUT_IOVEC_LEN_OFFSET), 0);
    assert_eq!(observation.control_u32(MEMORY_STDOUT_WRITTEN_OFFSET), 0);
    assert!(observation.stdout.is_empty());
    assert_eq!(observation.stdout_bytes, 0);
    assert_eq!(observation.successful_growths(), 8);
    assert_eq!(observation.grow_log.len(), 9, "creation plus eight grows");
    let last_logged_grow = observation.growth_log().last().unwrap();
    assert!(last_logged_grow.verdict);
    assert_eq!(last_logged_grow.reason, LimiterReason::WithinMaximumAndFree);
    assert_eq!(
        last_logged_grow.desired,
        observation.final_pages as usize * WASM_PAGE_BYTES
    );
}

fn assert_park_count_formula(observation: &Observation) {
    const DYNAMIC_GROW_CHARGE: u64 = MEMORY_GROW_STEP_PAGES as u64 * WASM_PAGE_BYTES as u64 / 64;

    assert_eq!(observation.fuel_suspensions, observation.fuel_parks.len());
    assert!(!observation.fuel_parks.is_empty());
    let mut cursor = 0;
    while cursor < observation.fuel_parks.len() {
        let first = observation.fuel_parks[cursor];
        let residual = first
            .granted_total
            .checked_sub(first.consumed)
            .expect("consumption cannot exceed cumulative grants");
        assert!(
            residual < DYNAMIC_GROW_CHARGE,
            "a recorded park must be at an underfunded dynamic grow"
        );
        let expected = (DYNAMIC_GROW_CHARGE - residual).div_ceil(observation.fuel_quantum);
        let mut end = cursor + 1;
        while end < observation.fuel_parks.len()
            && observation.fuel_parks[end].completed_grow_attempts == first.completed_grow_attempts
        {
            end += 1;
        }
        assert_eq!(
            (end - cursor) as u64,
            expected,
            "grow attempt {} must park ceil((C-r0)/Q) times",
            first.completed_grow_attempts
        );
        cursor = end;
    }
}

#[test]
fn default_config_real_fixture_freezes_upstream_out_of_fuel_bytes() {
    let fixture_wat = real_fixture_wat();
    let wasm = wat::parse_str(&fixture_wat).expect("real wasi_mem_grow.wat must compile via wat");
    let engine = default_fuel_engine();
    let observation = run_profile(
        &engine,
        wasm.as_slice(),
        "default_config_freeze",
        usize::MAX,
        FUEL_QUANTUM,
    );
    observation.dump();
    assert_default_config_freeze(&observation);
}

#[test]
fn real_wasi_mem_grow_fixture_parks_and_preserves_pacing() {
    let fixture_wat = real_fixture_wat();
    let wasm = wat::parse_str(&fixture_wat).expect("real wasi_mem_grow.wat must compile via wat");
    let engine = kernel_engine();

    let kernel_generous = run_profile(
        &engine,
        wasm.as_slice(),
        "kernel_generous",
        usize::MAX,
        FUEL_QUANTUM,
    );
    kernel_generous.dump();
    let kernel_constrained = run_profile(
        &engine,
        wasm.as_slice(),
        "kernel_constrained_200_mib",
        CONSTRAINED_FREE_BYTES,
        FUEL_QUANTUM,
    );
    kernel_constrained.dump();

    // These controls change only the runner quantum. Equal consumption pins
    // that parking neither charges early nor double-charges on retry.
    let control_generous = run_profile(
        &engine,
        wasm.as_slice(),
        "control_generous_32_mib_quantum",
        usize::MAX,
        WHOLE_FIXTURE_FUEL_QUANTUM,
    );
    control_generous.dump();
    let control_constrained = run_profile(
        &engine,
        wasm.as_slice(),
        "control_constrained_200_mib_32_mib_quantum",
        CONSTRAINED_FREE_BYTES,
        WHOLE_FIXTURE_FUEL_QUANTUM,
    );
    control_constrained.dump();
    assert_strict_linker_boundary(&engine, &fixture_wat);

    assert_common_success(&kernel_generous);
    assert_eq!(kernel_generous.final_pages, MAX_PAGES);
    assert_eq!(kernel_generous.stdout, MAX_PAGES.to_string().as_bytes());
    assert_eq!(kernel_generous.fuel_suspensions, 16);
    assert_park_count_formula(&kernel_generous);

    assert_common_success(&kernel_constrained);
    assert_eq!(kernel_constrained.final_pages, 1_549);
    assert_eq!(kernel_constrained.stdout, b"1549");
    assert_eq!(kernel_constrained.fuel_suspensions, 1);
    assert_park_count_formula(&kernel_constrained);

    assert_common_success(&control_generous);
    assert_eq!(control_generous.final_pages, MAX_PAGES);
    assert_eq!(
        control_generous.control_u32(MEMORY_GROW_STEP_COUNT_OFFSET),
        (MAX_PAGES - INITIAL_PAGES) / MEMORY_GROW_STEP_PAGES
    );
    assert_eq!(control_generous.stdout, MAX_PAGES.to_string().as_bytes());
    assert_eq!(control_generous.fuel_suspensions, 0);
    assert_eq!(
        kernel_generous.fuel_consumed,
        control_generous.fuel_consumed
    );
    let generous_last = control_generous.growth_log().last().unwrap();
    assert!(!generous_last.verdict);
    assert_eq!(generous_last.reason, LimiterReason::DeclaredMaximum);
    assert_eq!(generous_last.current, MAX_PAGES as usize * WASM_PAGE_BYTES);

    assert_common_success(&control_constrained);
    assert!(control_constrained.final_pages > INITIAL_PAGES);
    assert!(control_constrained.final_pages < MAX_PAGES);
    assert_eq!(control_constrained.final_pages, 1_549);
    assert_eq!(
        control_constrained.control_u32(MEMORY_GROW_STEP_COUNT_OFFSET),
        10
    );
    assert_eq!(control_constrained.stdout, b"1549");
    assert_eq!(control_constrained.fuel_suspensions, 0);
    assert_eq!(
        kernel_constrained.fuel_consumed,
        control_constrained.fuel_consumed
    );
    let constrained_last = control_constrained.growth_log().last().unwrap();
    assert!(!constrained_last.verdict);
    assert_eq!(constrained_last.reason, LimiterReason::FreeBudget);
    assert!(constrained_last.required_free.unwrap() > CONSTRAINED_FREE_BYTES);
}
