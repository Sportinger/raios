use alloc::{vec, vec::Vec};

use raios_core::{
    build_guest_class::BuildGuestClassV1,
    thread_scheduler::{
        Addr, JobState, JobThreadScheduler, MemSlot, Runde, SchedulerError, ThreadId, ThreadState,
    },
};
use wasmi::{
    core::ValueType, AtomicSuspend, AtomicSuspendRequest, Func, Linker, Memory, Module,
    ResumableCall, ResumableInvocation, Store, Suspension, Value,
};

use super::wasi_preview1::{
    ProcExitTrap, RecentWasiCall, SpawnRequest, WasiHostState, RECENT_WASI_CALL_COUNT,
    WASI_IMPORT_CALL_COUNT,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WasiThreadJobFailure {
    Setup,
    Scheduler,
    Fuel,
    FuelCeiling,
    Engine,
    MemoryIdentity,
    HostResultType,
    Materialization,
    RoundLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WasiThreadJobEnd {
    JobExited { code: u32 },
    JobDeadlocked,
    Failed(WasiThreadJobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WasiThreadRunEvidence {
    pub(crate) end: WasiThreadJobEnd,
    pub(crate) trace_digest: [u8; 32],
    pub(crate) effect_digest: [u8; 32],
    pub(crate) effect_count: u64,
    pub(crate) stdout: Vec<u8>,
    pub(crate) rounds: u64,
    pub(crate) spawns: u32,
    pub(crate) cap_denials: u32,
    pub(crate) granted_total: u64,
}

pub(crate) struct WasiThreadDiagRunEvidence {
    pub(crate) run: WasiThreadRunEvidence,
    pub(crate) import_call_counts: [u64; WASI_IMPORT_CALL_COUNT],
    pub(crate) recent_calls: [Option<RecentWasiCall>; RECENT_WASI_CALL_COUNT],
}

struct ThreadSlot {
    start: Func,
    prologue: Option<Func>,
    start_args: Option<(i32, i32)>,
    continuation: Option<ResumableInvocation>,
    fuel_escrow: u64,
    started: bool,
    exited: bool,
    open_host_op: bool,
}

impl ThreadSlot {
    fn main(start: Func, prologue: Option<Func>) -> Self {
        Self {
            start,
            prologue,
            start_args: None,
            continuation: None,
            fuel_escrow: 0,
            started: false,
            exited: false,
            open_host_op: false,
        }
    }

    fn spawned(start: Func, prologue: Option<Func>, thread: ThreadId, start_arg: i32) -> Self {
        Self {
            start,
            prologue,
            start_args: Some((thread.get() as i32, start_arg)),
            continuation: None,
            fuel_escrow: 0,
            started: false,
            exited: false,
            open_host_op: false,
        }
    }
}

enum SuspensionKind {
    FuelQuantum,
    Atomic(AtomicSuspend),
    ProcExit,
    Host,
}

/// One-store WASI+threads runner. Store data owns all host-visible state;
/// resumable wasmi entities and per-TID fuel escrows remain runner-owned.
pub(crate) struct WasiThreadJobRunner {
    store: Store<WasiHostState>,
    memory: Memory,
    module: Module,
    linker: Linker<WasiHostState>,
    threads: Vec<ThreadSlot>,
    class: BuildGuestClassV1,
    round: u64,
    round_limit: u64,
    exited_threads: usize,
    granted_total: u64,
    terminal: Option<WasiThreadJobEnd>,
}

impl WasiThreadJobRunner {
    pub(crate) fn new(
        mut store: Store<WasiHostState>,
        memory: Memory,
        module: Module,
        linker: Linker<WasiHostState>,
        main: Func,
        start_section: Option<Func>,
        class: BuildGuestClassV1,
    ) -> Result<Self, WasiThreadJobFailure> {
        if class.thread_cap == 0
            || class.fuel_quantum == 0
            || class.max_total_fuel == 0
            || !memory.ty(&store).is_shared()
            || u32::from(memory.current_pages(&store)) != class.shared_memory.max_pages
        {
            return Err(WasiThreadJobFailure::Setup);
        }
        let main_ty = main.ty(&store);
        if !main_ty.params().is_empty() || !main_ty.results().is_empty() {
            return Err(WasiThreadJobFailure::Setup);
        }
        if let Some(start_section) = &start_section {
            let start_ty = start_section.ty(&store);
            if !start_ty.params().is_empty() || !start_ty.results().is_empty() {
                return Err(WasiThreadJobFailure::Setup);
            }
        }
        let world = store
            .data()
            .scheduled_world()
            .ok_or(WasiThreadJobFailure::Setup)?;
        if world.scheduler.thread_cap() != class.thread_cap
            || !world.pending_spawns.is_empty()
            || world.active_thread.is_some()
            || world.active_fuel_checkpoint.is_some()
        {
            return Err(WasiThreadJobFailure::Setup);
        }
        if store
            .replace_remaining_fuel(0)
            .map_err(|_| WasiThreadJobFailure::Fuel)?
            != 0
        {
            return Err(WasiThreadJobFailure::Fuel);
        }
        let main_thread = store
            .data_mut()
            .scheduled_world_mut()
            .ok_or(WasiThreadJobFailure::Setup)?
            .scheduler
            .spawn()
            .map_err(|_| WasiThreadJobFailure::Scheduler)?;
        if main_thread.get() != 0 {
            return Err(WasiThreadJobFailure::Scheduler);
        }
        let round_limit = class
            .max_total_fuel
            .checked_add(class.fuel_quantum - 1)
            .ok_or(WasiThreadJobFailure::Setup)?
            / class.fuel_quantum;
        Ok(Self {
            store,
            memory,
            module,
            linker,
            threads: vec![ThreadSlot::main(main, start_section)],
            class,
            round: 0,
            round_limit,
            exited_threads: 0,
            granted_total: 0,
            terminal: None,
        })
    }

    pub(crate) fn run(mut self) -> WasiThreadRunEvidence {
        while self.terminal.is_none() && self.round < self.round_limit {
            self.pump();
        }
        if self.terminal.is_none() {
            self.fail(WasiThreadJobFailure::RoundLimit);
        }
        let end = self
            .terminal
            .unwrap_or(WasiThreadJobEnd::Failed(WasiThreadJobFailure::RoundLimit));
        let (trace_digest, effect_digest, effect_count, stdout, spawns, cap_denials) = {
            let state = self.store.data();
            match state.scheduled_world() {
                Some(world) => (
                    world.scheduler.trace_digest(),
                    state.effect_digest(),
                    state.effect_count(),
                    world.stdout.clone(),
                    world.spawns,
                    world.cap_denials,
                ),
                None => ([0; 32], [0; 32], 0, Vec::new(), 0, 0),
            }
        };
        WasiThreadRunEvidence {
            end,
            trace_digest,
            effect_digest,
            effect_count,
            stdout,
            rounds: self.round,
            spawns,
            cap_denials,
            granted_total: self.granted_total,
        }
    }

    pub(crate) fn run_capped(mut self, max_rounds: u64) -> WasiThreadDiagRunEvidence {
        let round_limit = self.round_limit.min(max_rounds);
        while self.terminal.is_none() && self.round < round_limit {
            self.pump();
        }
        if self.terminal.is_none() {
            self.fail(WasiThreadJobFailure::RoundLimit);
        }
        let end = self
            .terminal
            .unwrap_or(WasiThreadJobEnd::Failed(WasiThreadJobFailure::RoundLimit));
        let (
            trace_digest,
            effect_digest,
            effect_count,
            stdout,
            spawns,
            cap_denials,
            import_call_counts,
            recent_calls,
        ) = {
            let state = self.store.data();
            let import_call_counts = *state.import_call_counts();
            let recent_calls = state.recent_calls();
            match state.scheduled_world() {
                Some(world) => (
                    world.scheduler.trace_digest(),
                    state.effect_digest(),
                    state.effect_count(),
                    world.stdout.clone(),
                    world.spawns,
                    world.cap_denials,
                    import_call_counts,
                    recent_calls,
                ),
                None => (
                    [0; 32],
                    [0; 32],
                    0,
                    Vec::new(),
                    0,
                    0,
                    import_call_counts,
                    recent_calls,
                ),
            }
        };
        WasiThreadDiagRunEvidence {
            run: WasiThreadRunEvidence {
                end,
                trace_digest,
                effect_digest,
                effect_count,
                stdout,
                rounds: self.round,
                spawns,
                cap_denials,
                granted_total: self.granted_total,
            },
            import_call_counts,
            recent_calls,
        }
    }

    fn pump(&mut self) {
        if self.terminal.is_some() {
            return;
        }
        if let Err(failure) = self.pump_inner() {
            self.fail(failure);
        }
    }

    fn fail(&mut self, failure: WasiThreadJobFailure) {
        if self
            .scheduler()
            .is_ok_and(|scheduler| scheduler.job_state() == JobState::Running)
        {
            if let Ok(scheduler) = self.scheduler_mut() {
                let _ = scheduler.proc_exit(1);
            }
        }
        self.terminal = Some(WasiThreadJobEnd::Failed(failure));
    }

    fn pump_inner(&mut self) -> Result<(), WasiThreadJobFailure> {
        match self.scheduler()?.job_state() {
            JobState::JobExited { code } => {
                self.terminal = Some(WasiThreadJobEnd::JobExited { code });
                return Ok(());
            }
            JobState::JobDeadlocked => {
                self.terminal = Some(WasiThreadJobEnd::JobDeadlocked);
                return Ok(());
            }
            JobState::Running => {}
        }

        let round = self.round;
        self.round = self
            .round
            .checked_add(1)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        self.scheduler_mut()?
            .begin_round(Runde::new(round))
            .map_err(|_| WasiThreadJobFailure::Scheduler)?;

        let Some(thread) = self
            .scheduler_mut()?
            .next_runnable()
            .map_err(|_| WasiThreadJobFailure::Scheduler)?
        else {
            let has_open_host_op = self.threads.iter().any(|slot| slot.open_host_op);
            match self.scheduler_mut()?.check_deadlock(has_open_host_op) {
                Err(SchedulerError::JobDeadlocked) => {
                    self.terminal = Some(WasiThreadJobEnd::JobDeadlocked);
                    return Ok(());
                }
                Err(_) => return Err(WasiThreadJobFailure::Scheduler),
                Ok(()) => return Ok(()),
            }
        };

        let wake_result = match self
            .scheduler()?
            .thread_state(thread)
            .map_err(|_| WasiThreadJobFailure::Scheduler)?
        {
            ThreadState::Woken { .. } => Some(
                self.scheduler_mut()?
                    .take_wake_result(thread)
                    .map_err(|_| WasiThreadJobFailure::Scheduler)?,
            ),
            ThreadState::Runnable => None,
            _ => return Err(WasiThreadJobFailure::Scheduler),
        };

        self.install_thread_fuel(thread)?;
        let mut wake_input = [Value::I32(0)];
        let inputs = if let Some(result) = wake_result {
            wake_input[0] = Value::I32(result as i32);
            &wake_input[..]
        } else {
            &[]
        };
        let execution = self
            .resume_selected(thread, inputs)
            .and_then(|outcome| self.handle_outcome(thread, outcome));
        let sweep = self.sweep_thread_fuel(thread);
        execution?;
        sweep?;
        if self.terminal.is_none() {
            self.materialize_pending_spawns()?;
        }
        Ok(())
    }

    fn install_thread_fuel(&mut self, thread: ThreadId) -> Result<(), WasiThreadJobFailure> {
        let index = thread.get() as usize;
        let slot = self
            .threads
            .get_mut(index)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        let mut escrow = core::mem::take(&mut slot.fuel_escrow);
        if escrow == 0 {
            let remaining = self
                .class
                .max_total_fuel
                .checked_sub(self.granted_total)
                .filter(|remaining| *remaining != 0)
                .ok_or(WasiThreadJobFailure::FuelCeiling)?;
            let grant = self.class.fuel_quantum.min(remaining);
            escrow = escrow
                .checked_add(grant)
                .ok_or(WasiThreadJobFailure::FuelCeiling)?;
            self.granted_total = self
                .granted_total
                .checked_add(grant)
                .ok_or(WasiThreadJobFailure::FuelCeiling)?;
        }
        let old = self
            .store
            .replace_remaining_fuel(escrow)
            .map_err(|_| WasiThreadJobFailure::Fuel)?;
        if old != 0 {
            return Err(WasiThreadJobFailure::Fuel);
        }
        self.store
            .data_mut()
            .activate_thread(thread, escrow)
            .map_err(|_| WasiThreadJobFailure::Fuel)
    }

    fn sweep_thread_fuel(&mut self, thread: ThreadId) -> Result<(), WasiThreadJobFailure> {
        let remaining = self
            .store
            .replace_remaining_fuel(0)
            .map_err(|_| WasiThreadJobFailure::Fuel)?;
        let sync = self.store.data_mut().sync_active_fuel(remaining);
        let deactivate = self.store.data_mut().deactivate_thread();
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        if slot.fuel_escrow != 0 {
            return Err(WasiThreadJobFailure::Fuel);
        }
        slot.fuel_escrow = remaining;
        sync.and(deactivate).map_err(|_| WasiThreadJobFailure::Fuel)
    }

    fn resume_selected(
        &mut self,
        thread: ThreadId,
        inputs: &[Value],
    ) -> Result<ResumableCall, WasiThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        let mut outputs = [];
        if let Some(continuation) = slot.continuation.take() {
            return continuation
                .resume(&mut self.store, inputs, &mut outputs)
                .map_err(|_| WasiThreadJobFailure::Engine);
        }
        if slot.started || !inputs.is_empty() {
            return Err(WasiThreadJobFailure::Scheduler);
        }
        if let Some(prologue) = slot.prologue.take() {
            return prologue
                .call_resumable(&mut self.store, &[], &mut outputs)
                .map_err(|_| WasiThreadJobFailure::Engine);
        }
        slot.started = true;
        let spawned_inputs;
        let start_inputs = if let Some((tid, start_arg)) = slot.start_args {
            spawned_inputs = [Value::I32(tid), Value::I32(start_arg)];
            &spawned_inputs[..]
        } else {
            &[]
        };
        slot.start
            .call_resumable(&mut self.store, start_inputs, &mut outputs)
            .map_err(|_| WasiThreadJobFailure::Engine)
    }

    fn handle_outcome(
        &mut self,
        thread: ThreadId,
        mut outcome: ResumableCall,
    ) -> Result<(), WasiThreadJobFailure> {
        loop {
            let invocation = match outcome {
                ResumableCall::Finished => {
                    let start_started = self
                        .threads
                        .get(thread.get() as usize)
                        .ok_or(WasiThreadJobFailure::Scheduler)?
                        .started;
                    if !start_started {
                        outcome = self.resume_selected(thread, &[])?;
                        continue;
                    }
                    self.finish_thread(thread)?;
                    return Ok(());
                }
                ResumableCall::Resumable(invocation) => invocation,
            };
            let kind = match invocation.suspension() {
                Suspension::FuelQuantum => SuspensionKind::FuelQuantum,
                Suspension::Atomic(atomic) => SuspensionKind::Atomic(*atomic),
                Suspension::Host { host_error, .. }
                    if host_error.downcast_ref::<ProcExitTrap>().is_some() =>
                {
                    SuspensionKind::ProcExit
                }
                Suspension::Host { .. } => SuspensionKind::Host,
            };
            match kind {
                SuspensionKind::FuelQuantum => {
                    self.store_continuation(thread, invocation)?;
                    self.scheduler_mut()?
                        .on_quantum_end(thread)
                        .map_err(|_| WasiThreadJobFailure::Scheduler)?;
                    return Ok(());
                }
                SuspensionKind::Atomic(atomic) => {
                    let mem = self.memory_slot(atomic.memory)?;
                    let addr = Addr::new(atomic.addr);
                    match atomic.request {
                        AtomicSuspendRequest::Wait { timeout_ns } => {
                            self.store_continuation(thread, invocation)?;
                            let timeout = timeout_rounds(timeout_ns, self.class.fuel_quantum);
                            self.scheduler_mut()?
                                .park_wait(thread, mem, addr, timeout)
                                .map_err(|_| WasiThreadJobFailure::Scheduler)?;
                            return Ok(());
                        }
                        AtomicSuspendRequest::Notify { count } => {
                            let woken = self
                                .scheduler_mut()?
                                .notify(mem, addr, count)
                                .map_err(|_| WasiThreadJobFailure::Scheduler)?;
                            let mut outputs = [];
                            outcome = invocation
                                .resume(&mut self.store, &[Value::I32(woken as i32)], &mut outputs)
                                .map_err(|_| WasiThreadJobFailure::Engine)?;
                        }
                    }
                }
                SuspensionKind::ProcExit => {
                    let JobState::JobExited { code } = self.scheduler()?.job_state() else {
                        return Err(WasiThreadJobFailure::Engine);
                    };
                    self.terminal = Some(WasiThreadJobEnd::JobExited { code });
                    return Ok(());
                }
                SuspensionKind::Host => {
                    let host_func = invocation.host_func().ok_or(WasiThreadJobFailure::Engine)?;
                    if host_func.ty(&self.store).results() != &[ValueType::I32] {
                        return Err(WasiThreadJobFailure::HostResultType);
                    }
                    self.store_continuation(thread, invocation)?;
                    self.threads
                        .get_mut(thread.get() as usize)
                        .ok_or(WasiThreadJobFailure::Scheduler)?
                        .open_host_op = true;
                    self.scheduler_mut()?
                        .park_host(thread)
                        .map_err(|_| WasiThreadJobFailure::Scheduler)?;
                    return Ok(());
                }
            }
        }
    }

    fn store_continuation(
        &mut self,
        thread: ThreadId,
        invocation: ResumableInvocation,
    ) -> Result<(), WasiThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        if slot.continuation.replace(invocation).is_some() {
            return Err(WasiThreadJobFailure::Scheduler);
        }
        Ok(())
    }

    fn materialize_pending_spawns(&mut self) -> Result<(), WasiThreadJobFailure> {
        let requests = {
            let world = self
                .store
                .data_mut()
                .scheduled_world_mut()
                .ok_or(WasiThreadJobFailure::Setup)?;
            core::mem::take(&mut world.pending_spawns)
        };
        for SpawnRequest { thread, start_arg } in requests {
            if thread.get() as usize != self.threads.len() {
                return Err(WasiThreadJobFailure::Scheduler);
            }
            let pre = self
                .linker
                .instantiate(&mut self.store, &self.module)
                .map_err(|_| WasiThreadJobFailure::Materialization)?;
            let (instance, start_section) = pre
                .start_split(&mut self.store)
                .map_err(|_| WasiThreadJobFailure::Materialization)?;
            if let Some(start_section) = &start_section {
                let ty = start_section.ty(&self.store);
                if !ty.params().is_empty() || !ty.results().is_empty() {
                    return Err(WasiThreadJobFailure::Materialization);
                }
            }
            let start = instance
                .get_func(&self.store, "wasi_thread_start")
                .ok_or(WasiThreadJobFailure::Materialization)?;
            let ty = start.ty(&self.store);
            if ty.params() != &[ValueType::I32, ValueType::I32] || !ty.results().is_empty() {
                return Err(WasiThreadJobFailure::Materialization);
            }
            self.threads
                .push(ThreadSlot::spawned(start, start_section, thread, start_arg));
        }
        Ok(())
    }

    fn scheduler(&self) -> Result<&JobThreadScheduler, WasiThreadJobFailure> {
        self.store
            .data()
            .scheduled_world()
            .map(|world| &world.scheduler)
            .ok_or(WasiThreadJobFailure::Setup)
    }

    fn scheduler_mut(&mut self) -> Result<&mut JobThreadScheduler, WasiThreadJobFailure> {
        self.store
            .data_mut()
            .scheduled_world_mut()
            .map(|world| &mut world.scheduler)
            .ok_or(WasiThreadJobFailure::Setup)
    }

    fn memory_slot(&self, memory: Memory) -> Result<MemSlot, WasiThreadJobFailure> {
        let expected = self.memory.data(&self.store);
        let actual = memory.data(&self.store);
        if expected.as_ptr() == actual.as_ptr() && expected.len() == actual.len() {
            return Ok(MemSlot::new(0));
        }
        Err(WasiThreadJobFailure::MemoryIdentity)
    }

    fn finish_thread(&mut self, thread: ThreadId) -> Result<(), WasiThreadJobFailure> {
        let slot = self
            .threads
            .get_mut(thread.get() as usize)
            .ok_or(WasiThreadJobFailure::Scheduler)?;
        if slot.exited {
            return Err(WasiThreadJobFailure::Scheduler);
        }
        slot.exited = true;
        slot.open_host_op = false;
        self.exited_threads = self.exited_threads.saturating_add(1);
        self.scheduler_mut()?
            .thread_exit(thread)
            .map_err(|_| WasiThreadJobFailure::Scheduler)?;
        let pending_empty = self
            .store
            .data()
            .scheduled_world()
            .is_some_and(|world| world.pending_spawns.is_empty());
        if self.exited_threads == self.threads.len() && pending_empty {
            self.scheduler_mut()?
                .proc_exit(0)
                .map_err(|_| WasiThreadJobFailure::Scheduler)?;
            self.terminal = Some(WasiThreadJobEnd::JobExited { code: 0 });
        }
        Ok(())
    }
}

fn timeout_rounds(timeout_ns: i64, fuel_quantum: u64) -> Option<u64> {
    if timeout_ns < 0 {
        return None;
    }
    let timeout_ns = timeout_ns as u64;
    if timeout_ns == 0 {
        return Some(0);
    }
    Some(timeout_ns / fuel_quantum + u64::from(timeout_ns % fuel_quantum != 0))
}
