use super::invocation::{release_thread_job_execution, try_acquire_thread_job_execution};
use alloc::vec::Vec;
use raios_core::thread_scheduler::{
    Addr, JobState, JobThreadScheduler, MemSlot, Runde, SchedulerError, ThreadId, ThreadState,
    TraceEvent,
};
use wasmi::{
    core::ValueType, AtomicSuspend, AtomicSuspendRequest, Config, Engine, Func, Linker, Memory,
    MemoryType, Module, ResumableCall, ResumableInvocation, Store, Suspension, Value,
};

const THREAD_JOB_FIXTURE_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/thread_job_fixture.wasm"));
const FIXTURE_EXPECTED_SUM: i32 = 32;
const FIXTURE_THREAD_COUNT: usize = 2;
const FIXTURE_PUMP_ROUND_LIMIT: usize = 4_096;
const FUEL_QUANTUM: u64 = 20;

/// The scheduler's virtual clock uses the engine contract that one fuel is one ns.
/// Therefore a positive atomic timeout is rounded up in units of this fixed quantum.
const NS_PER_ROUND: u64 = FUEL_QUANTUM;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadJobFailure {
    Setup,
    Scheduler,
    Fuel,
    Engine,
    MemoryIdentity,
    HostResultType,
    RoundLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadJobEnd {
    JobExited { code: u32 },
    JobDeadlocked,
    Failed(ThreadJobFailure),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SuspensionCounters {
    waits: u32,
    notifies: u32,
    fuel_yields: u32,
    zero_notifies: u32,
    zero_notify_woken: u32,
}

struct ThreadSlot {
    start: Func,
    continuation: Option<ResumableInvocation>,
    started: bool,
    exited: bool,
    open_host_op: bool,
}

impl ThreadSlot {
    fn new(start: Func) -> Self {
        Self {
            start,
            continuation: None,
            started: false,
            exited: false,
            open_host_op: false,
        }
    }
}

enum SuspensionKind {
    FuelQuantum,
    Atomic(AtomicSuspend),
    Host,
}

/// Fixed-thread runner for one job. All instances and continuations share this
/// one Store, imported Memory and fuel counter; thread spawning is deliberately
/// absent from this T2-b2a package.
struct ThreadJobRunner<T> {
    store: Store<T>,
    memory: Memory,
    threads: Vec<ThreadSlot>,
    scheduler: JobThreadScheduler,
    round: u64,
    exited_threads: usize,
    counters: SuspensionCounters,
    terminal: Option<ThreadJobEnd>,
}

impl<T> ThreadJobRunner<T> {
    fn new(
        mut store: Store<T>,
        memory: Memory,
        starts: Vec<Func>,
    ) -> Result<Self, ThreadJobFailure> {
        if starts.is_empty() || !memory.ty(&store).is_shared() {
            return Err(ThreadJobFailure::Setup);
        }
        for start in &starts {
            let ty = start.ty(&store);
            if !ty.params().is_empty() || !ty.results().is_empty() {
                return Err(ThreadJobFailure::Setup);
            }
        }
        let remaining = store.consume_fuel(0).map_err(|_| ThreadJobFailure::Fuel)?;
        if remaining != 0 {
            store
                .consume_fuel(remaining)
                .map_err(|_| ThreadJobFailure::Fuel)?;
        }

        let mut scheduler = JobThreadScheduler::new();
        for _ in &starts {
            scheduler.spawn().map_err(|_| ThreadJobFailure::Scheduler)?;
        }
        let threads = starts.into_iter().map(ThreadSlot::new).collect();
        Ok(Self {
            store,
            memory,
            threads,
            scheduler,
            round: 0,
            exited_threads: 0,
            counters: SuspensionCounters::default(),
            terminal: None,
        })
    }

    fn pump(&mut self) -> bool {
        if self.terminal.is_some() {
            return true;
        }
        if let Err(failure) = self.pump_inner() {
            if self.scheduler.job_state() == JobState::Running {
                let _ = self.scheduler.proc_exit(1);
            }
            self.terminal = Some(ThreadJobEnd::Failed(failure));
        }
        self.terminal.is_some()
    }

    fn pump_inner(&mut self) -> Result<(), ThreadJobFailure> {
        match self.scheduler.job_state() {
            JobState::JobExited { code } => {
                self.terminal = Some(ThreadJobEnd::JobExited { code });
                return Ok(());
            }
            JobState::JobDeadlocked => {
                self.terminal = Some(ThreadJobEnd::JobDeadlocked);
                return Ok(());
            }
            JobState::Running => {}
        }

        let round = self.round;
        self.round = self
            .round
            .checked_add(1)
            .ok_or(ThreadJobFailure::Scheduler)?;
        self.scheduler
            .begin_round(Runde::new(round))
            .map_err(|_| ThreadJobFailure::Scheduler)?;

        let Some(thread) = self
            .scheduler
            .next_runnable()
            .map_err(|_| ThreadJobFailure::Scheduler)?
        else {
            let has_open_host_op = self.threads.iter().any(|slot| slot.open_host_op);
            match self.scheduler.check_deadlock(has_open_host_op) {
                Err(SchedulerError::JobDeadlocked) => {
                    self.terminal = Some(ThreadJobEnd::JobDeadlocked);
                    return Ok(());
                }
                Err(_) => return Err(ThreadJobFailure::Scheduler),
                Ok(()) => return Ok(()),
            }
        };

        let wake_result = match self
            .scheduler
            .thread_state(thread)
            .map_err(|_| ThreadJobFailure::Scheduler)?
        {
            ThreadState::Woken { .. } => Some(
                self.scheduler
                    .take_wake_result(thread)
                    .map_err(|_| ThreadJobFailure::Scheduler)?,
            ),
            ThreadState::Runnable => None,
            _ => return Err(ThreadJobFailure::Scheduler),
        };

        self.refill_quantum()?;
        let mut wake_input = [Value::I32(0)];
        let inputs = if let Some(result) = wake_result {
            wake_input[0] = Value::I32(result as i32);
            &wake_input[..]
        } else {
            &[]
        };
        let outcome = self.resume_selected(thread, inputs)?;
        self.handle_outcome(thread, outcome)
    }

    fn refill_quantum(&mut self) -> Result<(), ThreadJobFailure> {
        let remaining = self
            .store
            .consume_fuel(0)
            .map_err(|_| ThreadJobFailure::Fuel)?;
        if remaining != 0 {
            self.store
                .consume_fuel(remaining)
                .map_err(|_| ThreadJobFailure::Fuel)?;
        }
        self.store
            .add_fuel(FUEL_QUANTUM)
            .map_err(|_| ThreadJobFailure::Fuel)
    }

    fn resume_selected(
        &mut self,
        thread: ThreadId,
        inputs: &[Value],
    ) -> Result<ResumableCall, ThreadJobFailure> {
        let index = thread.get() as usize;
        let slot = self
            .threads
            .get_mut(index)
            .ok_or(ThreadJobFailure::Scheduler)?;
        let mut outputs = [];
        if let Some(continuation) = slot.continuation.take() {
            return continuation
                .resume(&mut self.store, inputs, &mut outputs)
                .map_err(|_| ThreadJobFailure::Engine);
        }
        if slot.started || !inputs.is_empty() {
            return Err(ThreadJobFailure::Scheduler);
        }
        slot.started = true;
        slot.start
            .call_resumable(&mut self.store, &[], &mut outputs)
            .map_err(|_| ThreadJobFailure::Engine)
    }

    fn handle_outcome(
        &mut self,
        thread: ThreadId,
        mut outcome: ResumableCall,
    ) -> Result<(), ThreadJobFailure> {
        loop {
            let invocation = match outcome {
                ResumableCall::Finished => {
                    self.finish_thread(thread)?;
                    return Ok(());
                }
                ResumableCall::Resumable(invocation) => invocation,
            };
            let kind = match invocation.suspension() {
                Suspension::FuelQuantum => SuspensionKind::FuelQuantum,
                Suspension::Atomic(atomic) => SuspensionKind::Atomic(*atomic),
                Suspension::Host { .. } => SuspensionKind::Host,
            };
            match kind {
                SuspensionKind::FuelQuantum => {
                    self.store_continuation(thread, invocation)?;
                    self.scheduler
                        .on_quantum_end(thread)
                        .map_err(|_| ThreadJobFailure::Scheduler)?;
                    self.counters.fuel_yields = self.counters.fuel_yields.saturating_add(1);
                    return Ok(());
                }
                SuspensionKind::Atomic(atomic) => {
                    let mem = self.memory_slot(atomic.memory)?;
                    let addr = Addr::new(atomic.addr);
                    match atomic.request {
                        AtomicSuspendRequest::Wait { timeout_ns } => {
                            self.store_continuation(thread, invocation)?;
                            self.scheduler
                                .park_wait(thread, mem, addr, timeout_rounds(timeout_ns))
                                .map_err(|_| ThreadJobFailure::Scheduler)?;
                            self.counters.waits = self.counters.waits.saturating_add(1);
                            return Ok(());
                        }
                        AtomicSuspendRequest::Notify { count } => {
                            let woken = self
                                .scheduler
                                .notify(mem, addr, count)
                                .map_err(|_| ThreadJobFailure::Scheduler)?;
                            self.counters.notifies = self.counters.notifies.saturating_add(1);
                            if count == 0 {
                                self.counters.zero_notifies =
                                    self.counters.zero_notifies.saturating_add(1);
                                self.counters.zero_notify_woken =
                                    self.counters.zero_notify_woken.saturating_add(woken);
                            }
                            let mut outputs = [];
                            outcome = invocation
                                .resume(&mut self.store, &[Value::I32(woken as i32)], &mut outputs)
                                .map_err(|_| ThreadJobFailure::Engine)?;
                        }
                    }
                }
                SuspensionKind::Host => {
                    let host_func = invocation.host_func().ok_or(ThreadJobFailure::Engine)?;
                    if host_func.ty(&self.store).results() != &[ValueType::I32] {
                        return Err(ThreadJobFailure::HostResultType);
                    }
                    self.store_continuation(thread, invocation)?;
                    let slot = self
                        .threads
                        .get_mut(thread.get() as usize)
                        .ok_or(ThreadJobFailure::Scheduler)?;
                    slot.open_host_op = true;
                    self.scheduler
                        .park_host(thread)
                        .map_err(|_| ThreadJobFailure::Scheduler)?;
                    return Ok(());
                }
            }
        }
    }

    fn store_continuation(
        &mut self,
        thread: ThreadId,
        invocation: ResumableInvocation,
    ) -> Result<(), ThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(ThreadJobFailure::Scheduler)?;
        if slot.continuation.replace(invocation).is_some() {
            return Err(ThreadJobFailure::Scheduler);
        }
        Ok(())
    }

    fn memory_slot(&self, memory: Memory) -> Result<MemSlot, ThreadJobFailure> {
        let expected = self.memory.data(&self.store);
        let actual = memory.data(&self.store);
        if expected.as_ptr() == actual.as_ptr() && expected.len() == actual.len() {
            return Ok(MemSlot::new(0));
        }
        Err(ThreadJobFailure::MemoryIdentity)
    }

    fn finish_thread(&mut self, thread: ThreadId) -> Result<(), ThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(ThreadJobFailure::Scheduler)?;
        if slot.exited {
            return Err(ThreadJobFailure::Scheduler);
        }
        slot.exited = true;
        slot.open_host_op = false;
        self.exited_threads = self.exited_threads.saturating_add(1);
        self.scheduler
            .thread_exit(thread)
            .map_err(|_| ThreadJobFailure::Scheduler)?;
        if self.exited_threads == self.threads.len() {
            self.scheduler
                .proc_exit(0)
                .map_err(|_| ThreadJobFailure::Scheduler)?;
            self.terminal = Some(ThreadJobEnd::JobExited { code: 0 });
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn complete_host(&mut self, thread: ThreadId, result: u32) -> Result<(), ThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(ThreadJobFailure::Scheduler)?;
        if !slot.open_host_op {
            return Err(ThreadJobFailure::Scheduler);
        }
        self.scheduler
            .wake_host(thread, result)
            .map_err(|_| ThreadJobFailure::Scheduler)?;
        slot.open_host_op = false;
        Ok(())
    }

    fn read_i32(&self, offset: usize) -> Result<i32, ThreadJobFailure> {
        let mut bytes = [0_u8; 4];
        self.memory
            .read(&self.store, offset, &mut bytes)
            .map_err(|_| ThreadJobFailure::MemoryIdentity)?;
        Ok(i32::from_le_bytes(bytes))
    }
}

fn timeout_rounds(timeout_ns: i64) -> Option<u64> {
    if timeout_ns < 0 {
        return None;
    }
    let timeout_ns = timeout_ns as u64;
    if timeout_ns == 0 {
        return Some(0);
    }
    Some(timeout_ns / NS_PER_ROUND + u64::from(timeout_ns % NS_PER_ROUND != 0))
}

#[derive(Clone, Debug)]
struct FixtureRunEvidence {
    end: ThreadJobEnd,
    counters: SuspensionCounters,
    trace: Vec<TraceEvent>,
    switches: u32,
    sum: i32,
    rounds: u64,
}

fn thread_job_engine() -> Engine {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    Engine::new(&config)
}

fn run_fixture_once() -> Result<FixtureRunEvidence, ThreadJobFailure> {
    let engine = thread_job_engine();
    let module =
        Module::new(&engine, THREAD_JOB_FIXTURE_WASM).map_err(|_| ThreadJobFailure::Setup)?;
    let mut store = Store::new(&engine, ());
    let memory_type = MemoryType::new_shared(1, 1).map_err(|_| ThreadJobFailure::Setup)?;
    let memory = Memory::new(&mut store, memory_type).map_err(|_| ThreadJobFailure::Setup)?;
    let mut linker = Linker::<()>::new(&engine);
    linker
        .define("env", "memory", memory)
        .map_err(|_| ThreadJobFailure::Setup)?;
    let thread0_pre = linker
        .instantiate(&mut store, &module)
        .map_err(|_| ThreadJobFailure::Setup)?;
    let thread0_instance = thread0_pre
        .start(&mut store)
        .map_err(|_| ThreadJobFailure::Setup)?;
    let thread1_pre = linker
        .instantiate(&mut store, &module)
        .map_err(|_| ThreadJobFailure::Setup)?;
    let thread1_instance = thread1_pre
        .start(&mut store)
        .map_err(|_| ThreadJobFailure::Setup)?;
    let thread0 = thread0_instance
        .get_func(&store, "thread0")
        .ok_or(ThreadJobFailure::Setup)?;
    let thread1 = thread1_instance
        .get_func(&store, "thread1")
        .ok_or(ThreadJobFailure::Setup)?;
    let mut runner = ThreadJobRunner::new(store, memory, alloc::vec![thread0, thread1])?;
    if runner.threads.len() != FIXTURE_THREAD_COUNT {
        return Err(ThreadJobFailure::Setup);
    }

    for _ in 0..FIXTURE_PUMP_ROUND_LIMIT {
        if runner.pump() {
            break;
        }
    }
    let end = runner.terminal.ok_or(ThreadJobFailure::RoundLimit)?;
    let trace = runner.scheduler.trace().to_vec();
    let switches = trace
        .iter()
        .filter(|event| matches!(event, TraceEvent::Switch { .. }))
        .count() as u32;
    Ok(FixtureRunEvidence {
        end,
        counters: runner.counters,
        trace,
        switches,
        sum: runner.read_i32(8)?,
        rounds: runner.round,
    })
}

struct ThreadJobBusyGuard;

impl ThreadJobBusyGuard {
    fn acquire() -> Option<Self> {
        try_acquire_thread_job_execution().then_some(Self)
    }
}

impl Drop for ThreadJobBusyGuard {
    fn drop(&mut self) {
        release_thread_job_execution();
    }
}

pub(crate) fn emit_threads_selftest() {
    let Some(_busy) = ThreadJobBusyGuard::acquire() else {
        write_selftest_line(false, SuspensionCounters::default(), 0, 0, 0);
        return;
    };
    let first = run_fixture_once();
    let second = run_fixture_once();
    let pass = matches!(
        (&first, &second),
        (Ok(first), Ok(second))
            if first.end == (ThreadJobEnd::JobExited { code: 0 })
                && second.end == (ThreadJobEnd::JobExited { code: 0 })
                && first.sum == FIXTURE_EXPECTED_SUM
                && second.sum == FIXTURE_EXPECTED_SUM
                && !first.trace.is_empty()
                && first.trace == second.trace
                && first.counters.waits >= 1
                && first.counters.notifies >= 1
                && first.counters.fuel_yields >= 1
                && first.counters.zero_notifies >= 1
                && first.counters.zero_notify_woken == 0
                && first.counters == second.counters
                && first.rounds == second.rounds
    );
    let (counters, switches, sum, rounds) = match first {
        Ok(evidence) => (
            evidence.counters,
            evidence.switches,
            evidence.sum,
            evidence.rounds,
        ),
        Err(_) => (SuspensionCounters::default(), 0, 0, 0),
    };
    write_selftest_line(pass, counters, switches, sum, rounds);
}

fn write_selftest_line(
    pass: bool,
    counters: SuspensionCounters,
    switches: u32,
    sum: i32,
    rounds: u64,
) {
    crate::serial::write_fmt(format_args!(
        "RAIOS_THREADS selftest={} waits={} notifies={} fuel_yields={} switches={} sum={} rounds={}\r\n",
        if pass { "pass" } else { "fail" },
        counters.waits,
        counters.notifies,
        counters.fuel_yields,
        switches,
        sum,
        rounds,
    ));
}
