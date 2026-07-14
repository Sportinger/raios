use super::*;
use super::{
    envelope::BUFFER_SERVICE_MAX_MEMORY_BYTES, net_shims::*, probes::SUSPEND_ONCE_WASM_MODULE,
    suspension::*,
};

pub(super) const BEYOND_ENV_FIXTURE_FUEL_BUDGET: u64 = 1_000_000;
pub(super) const BEYOND_ENV_WALL_BUDGET_MS: u64 = 90_000;
pub(super) const BEYOND_ENV_PUMP_STEP_BUDGET: u32 = 11_250;

static BEYOND_ENV_REQUEST: AtomicU8 = AtomicU8::new(0);
pub(super) static NEXT_BEYOND_ENV_INVOCATION_ID: AtomicU64 = AtomicU64::new(1);
static BEYOND_ENV_PROBE: Mutex<BeyondEnvProbeSnapshot> =
    Mutex::new(BeyondEnvProbeSnapshot::empty());

pub(super) static ACTIVE_DROP_TEARDOWN_COUNT: AtomicU64 = AtomicU64::new(0);
static ACTIVE_BEYOND_ENV_INVOCATION: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FixtureBehavior {
    Suspend,
    UnrecognizedHostError,
    HostFuelError,
    Net(NetFixtureScenario),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BeyondEnvFixtureRequest {
    HoldForKill,
    ResumeToFinish,
    NetResponsive,
    NetSilentTimeout,
    NetSilentKill,
}

impl BeyondEnvFixtureRequest {
    const fn encode(self) -> u8 {
        match self {
            Self::HoldForKill => 1,
            Self::ResumeToFinish => 2,
            Self::NetResponsive => 3,
            Self::NetSilentTimeout => 4,
            Self::NetSilentKill => 5,
        }
    }

    const fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::HoldForKill),
            2 => Some(Self::ResumeToFinish),
            3 => Some(Self::NetResponsive),
            4 => Some(Self::NetSilentTimeout),
            5 => Some(Self::NetSilentKill),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvProbeSnapshot {
    pub(crate) request_status: &'static str,
    pub(crate) active: bool,
    pub(crate) invocation_id: u64,
    pub(crate) run_count: u64,
    pub(crate) suspended: bool,
    pub(crate) suspended_boot_ms: u64,
    pub(crate) outcome: &'static str,
    pub(crate) terminal_boot_ms: u64,
    pub(crate) kill_observed_boot_ms: u64,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) fuel_used: u64,
    pub(crate) no_resume_after_kill: bool,
    pub(crate) teardown_complete: bool,
    pub(crate) handles_invalid: bool,
    pub(crate) lease_held: bool,
    pub(crate) pending_acquisition_present: bool,
    pub(crate) prior_candidate_unchanged: bool,
    pub(crate) killed_run_count: u64,
    pub(crate) finished_run_count: u64,
}

impl BeyondEnvProbeSnapshot {
    const fn empty() -> Self {
        Self {
            request_status: "idle",
            active: false,
            invocation_id: 0,
            run_count: 0,
            suspended: false,
            suspended_boot_ms: 0,
            outcome: "not_run",
            terminal_boot_ms: 0,
            kill_observed_boot_ms: 0,
            suspension_count: 0,
            resume_count: 0,
            teardown_count: 0,
            fuel_used: 0,
            no_resume_after_kill: false,
            teardown_complete: false,
            handles_invalid: true,
            lease_held: false,
            pending_acquisition_present: false,
            prior_candidate_unchanged: true,
            killed_run_count: 0,
            finished_run_count: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvLifecycleCase {
    pub(crate) name: &'static str,
    pub(crate) outcome: &'static str,
    pub(crate) suspended: bool,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) teardown_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvLifecycleSuite {
    pub(crate) cases: [BeyondEnvLifecycleCase; 7],
    pub(crate) tail_call_terminal: bool,
    pub(crate) busy_loop_out_of_fuel: bool,
    pub(crate) busy_loop_wall_ms: u64,
    pub(crate) fuel_budget: u64,
    pub(crate) wall_budget_ms: u64,
    pub(crate) pump_step_budget: u32,
}

pub(super) struct BeyondEnvState {
    pub(super) lifecycle: InvocationLifecycle,
    pub(super) behavior: FixtureBehavior,
    pub(super) net: Option<NetInvocationState>,
    pub(super) limits: StoreLimits,
}

pub(crate) struct ActiveBeyondEnvInvocation {
    pub(super) store: Option<Store<BeyondEnvState>>,
    pub(super) instance: Option<Instance>,
    pub(super) memory: Option<Memory>,
    pub(super) continuation: Option<ResumableInvocation>,
    pub(super) outputs: [Value; 1],
    pub(super) auto_resume: bool,
    pub(super) closed: bool,
}

pub(crate) fn request_beyond_env_fixture(request: BeyondEnvFixtureRequest) -> &'static str {
    if wasm_execution_busy() {
        return "resource_busy_active_invocation";
    }
    match BEYOND_ENV_REQUEST.compare_exchange(
        0,
        request.encode(),
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            if matches!(
                request,
                BeyondEnvFixtureRequest::NetResponsive
                    | BeyondEnvFixtureRequest::NetSilentTimeout
                    | BeyondEnvFixtureRequest::NetSilentKill
            ) {
                NET_SHIM_PROBE.lock().request_status = "accepted_pending";
            } else {
                BEYOND_ENV_PROBE.lock().request_status = "accepted_pending";
            }
            "accepted_pending"
        }
        Err(_) => "resource_busy_active_invocation",
    }
}

pub(super) fn wasm_execution_busy() -> bool {
    ACTIVE_BEYOND_ENV_INVOCATION.load(Ordering::Acquire) != 0
}

pub(crate) fn take_beyond_env_fixture_request() -> Option<BeyondEnvFixtureRequest> {
    BeyondEnvFixtureRequest::decode(BEYOND_ENV_REQUEST.swap(0, Ordering::AcqRel))
}

pub(crate) fn beyond_env_probe_snapshot() -> BeyondEnvProbeSnapshot {
    *BEYOND_ENV_PROBE.lock()
}

pub(crate) fn start_beyond_env_fixture(
    request: BeyondEnvFixtureRequest,
    now_ms: u64,
    kill_generation: u64,
) -> Option<ActiveBeyondEnvInvocation> {
    let active = try_start_beyond_env_fixture(request, now_ms, kill_generation);
    if active.is_none() {
        if matches!(
            request,
            BeyondEnvFixtureRequest::NetResponsive
                | BeyondEnvFixtureRequest::NetSilentTimeout
                | BeyondEnvFixtureRequest::NetSilentKill
        ) {
            let mut probe = NET_SHIM_PROBE.lock();
            probe.request_status = "fixture_setup_failed";
            probe.active = false;
            probe.outcome = "fixture_setup_failed";
            serial::write_fmt(format_args!(
                "RAIOS_NET_SHIM outcome=fixture_setup_failed teardown_complete=true boot_ms={}\r\n",
                now_ms
            ));
        } else {
            let mut probe = BEYOND_ENV_PROBE.lock();
            probe.request_status = "fixture_setup_failed";
            probe.active = false;
            probe.outcome = "fixture_setup_failed";
            serial::write_fmt(format_args!(
                "RAIOS_BEYOND_ENV_LIFECYCLE outcome=fixture_setup_failed teardown_complete=true boot_ms={}\r\n",
                now_ms
            ));
        }
    }
    active
}

fn try_start_beyond_env_fixture(
    request: BeyondEnvFixtureRequest,
    now_ms: u64,
    kill_generation: u64,
) -> Option<ActiveBeyondEnvInvocation> {
    let invocation_id = NEXT_BEYOND_ENV_INVOCATION_ID.fetch_add(1, Ordering::Relaxed);
    let (service_id, behavior, module_bytes) = match request {
        BeyondEnvFixtureRequest::HoldForKill | BeyondEnvFixtureRequest::ResumeToFinish => (
            "test.fixture.beyond_env_lifecycle",
            FixtureBehavior::Suspend,
            SUSPEND_ONCE_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetResponsive => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::Responsive),
            NET_SHIM_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetSilentTimeout => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::SilentTimeout),
            NET_SHIM_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetSilentKill => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::SilentKill),
            NET_SHIM_WASM_MODULE,
        ),
    };
    let authority = InvocationAuthority {
        service_id,
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation: kill_generation,
        policy_allows_beyond_env: false,
    };
    let engine = beyond_env_engine();
    let module = Module::new(&engine, module_bytes).ok()?;
    let net_state = match behavior {
        FixtureBehavior::Net(scenario) => Some(NetInvocationState::new(scenario, invocation_id)),
        _ => None,
    };
    let mut store = Store::new(
        &engine,
        BeyondEnvState {
            lifecycle: InvocationLifecycle::new(
                authority,
                now_ms,
                BEYOND_ENV_WALL_BUDGET_MS,
                BEYOND_ENV_PUMP_STEP_BUDGET,
                invocation_id as u32,
            ),
            behavior,
            net: net_state,
            limits: beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    store.add_fuel(BEYOND_ENV_FIXTURE_FUEL_BUDGET).ok()?;
    let mut linker = Linker::<BeyondEnvState>::new(&engine);
    match behavior {
        FixtureBehavior::Net(_) => {
            linker
                .func_wrap("net", "tcp_open", host_net_tcp_open)
                .ok()?;
            linker
                .func_wrap("net", "tcp_send", host_net_tcp_send)
                .ok()?;
            linker
                .func_wrap("net", "tcp_recv", host_net_tcp_recv)
                .ok()?;
            linker
                .func_wrap("net", "tcp_close", host_net_tcp_close)
                .ok()?;
        }
        _ => {
            linker
                .func_wrap("test", "suspend_once", host_test_suspend_once)
                .ok()?;
        }
    }
    let instance = linker
        .instantiate(&mut store, &module)
        .ok()?
        .start(&mut store)
        .ok()?;
    let memory = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)?;
    let function = instance.get_export(&store, "run")?.into_func()?;
    let mut outputs = [Value::I32(0)];
    let continuation = match function.call_resumable(&mut store, &[], &mut outputs) {
        Ok(ResumableCall::Resumable(invocation))
            if classify_resumable(&invocation, &store).is_ok() =>
        {
            invocation
        }
        Ok(ResumableCall::Resumable(_)) => {
            finish_store(&mut store, TerminalOutcome::HostError);
            return None;
        }
        Ok(ResumableCall::Finished) => {
            finish_store(&mut store, TerminalOutcome::Finished);
            return None;
        }
        Err(error) => {
            finish_store(&mut store, terminal_from_error(&error));
            return None;
        }
    };
    let suspended_boot_ms = crate::time::rdtsc() / crate::time::tsc_per_ms().max(1);
    if let FixtureBehavior::Net(scenario) = behavior {
        let mut probe = NET_SHIM_PROBE.lock();
        let counts = (
            probe.responsive_run_count,
            probe.timeout_run_count,
            probe.killed_run_count,
        );
        *probe = NetShimProbeSnapshot::empty();
        probe.responsive_run_count = counts.0;
        probe.timeout_run_count = counts.1;
        probe.killed_run_count = counts.2;
        probe.request_status = "running";
        probe.active = true;
        probe.scenario = scenario.as_str();
        probe.invocation_id = invocation_id;
        probe.suspended = true;
        probe.outcome = "pending";
        probe.suspension_count = store.data().lifecycle.suspension_count();
        probe.open_operation_id = store
            .data()
            .lifecycle
            .pending()
            .map_or(0, |pending| pending.operation_id);
        probe.lease_held = store
            .data()
            .net
            .as_ref()
            .is_some_and(|net| net.lease.is_some());
        serial::write_fmt(format_args!(
            "RAIOS_NET_SHIM suspended=true scenario={} operation=tcp_open invocation_id={} boot_ms={}\r\n",
            scenario.as_str(), invocation_id, suspended_boot_ms
        ));
    } else {
        let mut probe = BEYOND_ENV_PROBE.lock();
        probe.request_status = "running";
        probe.active = true;
        probe.invocation_id = invocation_id;
        probe.run_count = probe.run_count.saturating_add(1);
        probe.suspended = true;
        probe.suspended_boot_ms = suspended_boot_ms;
        probe.outcome = "pending";
        probe.terminal_boot_ms = 0;
        probe.kill_observed_boot_ms = 0;
        probe.suspension_count = store.data().lifecycle.suspension_count();
        probe.resume_count = 0;
        probe.teardown_count = 0;
        probe.teardown_complete = false;
        probe.no_resume_after_kill = false;
        serial::write_fmt(format_args!(
            "RAIOS_BEYOND_ENV_LIFECYCLE suspended=true invocation_id={} boot_ms={}\r\n",
            invocation_id, suspended_boot_ms
        ));
    }
    ACTIVE_BEYOND_ENV_INVOCATION.store(1, Ordering::Release);
    Some(ActiveBeyondEnvInvocation {
        store: Some(store),
        instance: Some(instance),
        memory: Some(memory),
        continuation: Some(continuation),
        outputs,
        auto_resume: request == BeyondEnvFixtureRequest::ResumeToFinish,
        closed: false,
    })
}

impl ActiveBeyondEnvInvocation {
    pub(crate) fn pump(&mut self, now_ms: u64, kill_generation: u64, kill_boot_ms: u64) -> bool {
        if self.closed {
            return true;
        }
        let boundary = BoundaryState {
            now_ms,
            kill_generation,
            service_generation: 1,
            instance_generation: 1,
            posture_allows_execution: true,
        };
        let boundary_result = self
            .store
            .as_mut()
            .expect("active beyond-env store missing")
            .data_mut()
            .lifecycle
            .check_boundary(boundary);
        if let Err(outcome) = boundary_result {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        if matches!(
            self.store
                .as_ref()
                .expect("active beyond-env store missing")
                .data()
                .behavior,
            FixtureBehavior::Net(_)
        ) {
            return self.pump_net(now_ms, kill_boot_ms, boundary);
        }
        if !self.auto_resume {
            return false;
        }

        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        if store.data_mut().lifecycle.take_pending().is_none() {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        }
        if let Err(outcome) = store.data_mut().lifecycle.check_boundary(boundary) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        store.data_mut().lifecycle.note_resume();
        let continuation = self
            .continuation
            .take()
            .expect("active beyond-env continuation missing");
        match continuation.resume(&mut *store, &[Value::I32(7)], &mut self.outputs) {
            Ok(ResumableCall::Finished) => self.finish(TerminalOutcome::Finished, now_ms, 0),
            Ok(ResumableCall::Resumable(next)) => match classify_resumable(&next, store) {
                Ok(()) => {
                    self.continuation = Some(next);
                    return false;
                }
                Err(outcome) => self.finish(outcome, now_ms, 0),
            },
            Err(error) => self.finish(terminal_from_error(&error), now_ms, 0),
        }
        true
    }

    fn pump_net(&mut self, now_ms: u64, kill_boot_ms: u64, boundary: BoundaryState) -> bool {
        let completion = {
            let store = self
                .store
                .as_mut()
                .expect("active beyond-env store missing");
            match progress_net_operation(store, now_ms) {
                Ok(Some(completion)) => completion,
                Ok(None) => return false,
                Err(outcome) => {
                    self.finish(outcome, now_ms, kill_boot_ms);
                    return true;
                }
            }
        };
        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        if let Err(outcome) = check_net_resume_boundary(store, &completion, now_ms) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        let Some(pending) = store.data_mut().lifecycle.take_pending() else {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        };
        if pending.invocation_id != store.data().lifecycle.authority().invocation_id {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        }
        if let Err(outcome) = store.data_mut().lifecycle.check_boundary(boundary) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        if let Some(ptr) = completion.recv_ptr {
            let memory = self.memory.expect("active beyond-env memory missing");
            if memory
                .write(&mut *store, ptr, &completion.buffer[..completion.recv_len])
                .is_err()
            {
                self.finish(TerminalOutcome::HostError, now_ms, 0);
                return true;
            }
        }
        store.data_mut().lifecycle.note_resume();
        let continuation = self
            .continuation
            .take()
            .expect("active beyond-env continuation missing");
        match continuation.resume(
            &mut *store,
            &[Value::I32(completion.result)],
            &mut self.outputs,
        ) {
            Ok(ResumableCall::Finished) => self.finish(TerminalOutcome::Finished, now_ms, 0),
            Ok(ResumableCall::Resumable(next)) => match classify_resumable(&next, store) {
                Ok(()) => {
                    self.continuation = Some(next);
                    let mut probe = NET_SHIM_PROBE.lock();
                    probe.suspended = true;
                    probe.suspension_count = store.data().lifecycle.suspension_count();
                    probe.resume_count = store.data().lifecycle.resume_count();
                    probe.lease_held = store
                        .data()
                        .net
                        .as_ref()
                        .is_some_and(|net| net.lease.is_some());
                    return false;
                }
                Err(outcome) => self.finish(outcome, now_ms, 0),
            },
            Err(error) => self.finish(terminal_from_error(&error), now_ms, 0),
        }
        true
    }

    fn finish(&mut self, outcome: TerminalOutcome, now_ms: u64, kill_boot_ms: u64) {
        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        let net_terminal = finish_net_resources(store, now_ms);
        finish_store(store, outcome);
        let lifecycle = &store.data().lifecycle;
        let receipt = lifecycle.teardown_receipt();
        let suspension_count = lifecycle.suspension_count();
        let resume_count = lifecycle.resume_count();
        let fuel_used = store.fuel_consumed().unwrap_or(0);
        let guest_return_value = self.outputs[0].i32().unwrap_or(0).max(0) as u64;

        // Teardown step 9: no raiOS lock is held while wasmi state is dropped.
        drop(self.continuation.take());
        drop(self.memory.take());
        drop(self.instance.take());
        drop(self.store.take());
        self.closed = true;

        if let Some(net_terminal) = net_terminal {
            let mut probe = NET_SHIM_PROBE.lock();
            probe.request_status = "complete";
            probe.active = false;
            probe.suspended = false;
            probe.outcome = if outcome == TerminalOutcome::Killed {
                "killed"
            } else {
                match net_terminal.first_negative_result {
                    Some(HOST_IMPORT_ERROR_TIMED_OUT) => "timed_out",
                    Some(_) => "transport_error",
                    None => "finished",
                }
            };
            probe.terminal_boot_ms = now_ms;
            probe.kill_observed_boot_ms = kill_boot_ms;
            probe.suspension_count = suspension_count;
            probe.resume_count = resume_count;
            probe.teardown_count = receipt.teardown_count;
            probe.teardown_complete = receipt.terminal_outcome_recorded;
            probe.no_resume_after_kill = outcome == TerminalOutcome::Killed && resume_count == 0;
            probe.lease_held = false;
            probe.close_call_count = net_terminal.close_call_count;
            probe.tx_bytes = net_terminal.tx_bytes;
            probe.rx_bytes = net_terminal.rx_bytes;
            probe.guest_return_value = guest_return_value;
            probe.would_block_guest_visible = net_terminal.would_block_guest_visible;
            match probe.outcome {
                "finished" => {
                    probe.responsive_run_count = probe.responsive_run_count.saturating_add(1)
                }
                "timed_out" => probe.timeout_run_count = probe.timeout_run_count.saturating_add(1),
                "killed" => probe.killed_run_count = probe.killed_run_count.saturating_add(1),
                _ => {}
            }
            serial::write_fmt(format_args!(
                "RAIOS_NET_SHIM outcome={} scenario={} teardown_complete=true resume_count={} boot_ms={}\r\n",
                probe.outcome, probe.scenario, resume_count, now_ms
            ));
        } else {
            let mut probe = BEYOND_ENV_PROBE.lock();
            probe.request_status = "complete";
            probe.active = false;
            probe.suspended = false;
            probe.outcome = outcome.as_str();
            probe.terminal_boot_ms = now_ms;
            probe.kill_observed_boot_ms = kill_boot_ms;
            probe.suspension_count = suspension_count;
            probe.resume_count = resume_count;
            probe.teardown_count = receipt.teardown_count;
            probe.fuel_used = fuel_used;
            if outcome == TerminalOutcome::Killed {
                probe.no_resume_after_kill = resume_count == 0;
            }
            probe.teardown_complete = receipt.terminal_outcome_recorded;
            probe.handles_invalid = receipt.handles_invalid;
            probe.lease_held = false;
            probe.pending_acquisition_present = false;
            probe.prior_candidate_unchanged = receipt.prior_candidate_unchanged;
            if outcome == TerminalOutcome::Killed {
                probe.killed_run_count = probe.killed_run_count.saturating_add(1);
            }
            if outcome == TerminalOutcome::Finished {
                probe.finished_run_count = probe.finished_run_count.saturating_add(1);
            }
            serial::write_fmt(format_args!(
                "RAIOS_BEYOND_ENV_LIFECYCLE outcome={} teardown_complete=true handles_invalid=true resume_count={} boot_ms={}\r\n",
                outcome.as_str(), resume_count, now_ms
            ));
        }
        ACTIVE_BEYOND_ENV_INVOCATION.store(0, Ordering::Release);
    }
}

impl Drop for ActiveBeyondEnvInvocation {
    fn drop(&mut self) {
        if let Some(store) = self.store.as_mut() {
            let _ = finish_net_resources(store, runtime_now_ms());
            if store
                .data_mut()
                .lifecycle
                .teardown(TerminalOutcome::Abandoned)
            {
                ACTIVE_DROP_TEARDOWN_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        ACTIVE_BEYOND_ENV_INVOCATION.store(0, Ordering::Release);
    }
}

pub(super) fn finish_store(store: &mut Store<BeyondEnvState>, outcome: TerminalOutcome) {
    store.data_mut().lifecycle.teardown(outcome);
}

pub(super) fn host_test_suspend_once(mut caller: Caller<'_, BeyondEnvState>) -> Result<i32, Trap> {
    match caller.data().behavior {
        FixtureBehavior::Suspend => {
            let authority = caller.data().lifecycle.authority();
            let pending = caller
                .data_mut()
                .lifecycle
                .record_pending(1)
                .ok_or_else(|| Trap::new("test suspend pending operation already exists"))?;
            Err(HostSuspend {
                invocation_id: authority.invocation_id,
                operation_id: pending.operation_id,
            }
            .into())
        }
        FixtureBehavior::UnrecognizedHostError => Err(FixtureHostError.into()),
        FixtureBehavior::HostFuelError => {
            caller
                .consume_fuel(u64::MAX)
                .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
            Ok(0)
        }
        FixtureBehavior::Net(_) => Err(Trap::new("test.suspend_once unavailable to net fixture")),
    }
}

pub(super) fn runtime_now_ms() -> u64 {
    crate::time::rdtsc() / crate::time::tsc_per_ms().max(1)
}

pub(super) fn beyond_env_engine() -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true).wasm_tail_call(true);
    Engine::new(&config)
}

pub(super) fn beyond_env_limits() -> StoreLimits {
    StoreLimitsBuilder::new()
        .memory_size(BUFFER_SERVICE_MAX_MEMORY_BYTES)
        .instances(1)
        .memories(1)
        .tables(1)
        .table_elements(64)
        .build()
}
