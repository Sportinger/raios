use super::*;

const OOB_STORE_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_store.wasm"));
const OOB_LOAD_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_load.wasm"));
const OOB_OFFSET_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_offset.wasm"));

const HOST_GUARD: [u8; 32] = [0xa5; 32];

#[derive(Clone, Copy)]
struct IsolationCaseEvidence {
    memory_out_of_bounds: bool,
    guest_memory_unchanged: bool,
    host_memory_unchanged: bool,
    host_exposed_bytes: u64,
    trap_recorded: bool,
}

impl IsolationCaseEvidence {
    fn failed_before_call(host_memory_unchanged: bool) -> Self {
        Self {
            memory_out_of_bounds: false,
            guest_memory_unchanged: false,
            host_memory_unchanged,
            host_exposed_bytes: 0,
            trap_recorded: false,
        }
    }

    fn passed(self) -> bool {
        self.memory_out_of_bounds
            && self.guest_memory_unchanged
            && self.host_memory_unchanged
            && self.host_exposed_bytes == 0
            && self.trap_recorded
    }

    fn token(self) -> &'static str {
        if self.memory_out_of_bounds {
            "trapped"
        } else {
            "failed"
        }
    }
}

fn run_hostile_fixture(bytes: &[u8], bytes_exposed_on_success: u64) -> IsolationCaseEvidence {
    let host_guard = core::hint::black_box(HOST_GUARD);
    if invocation::wasm_execution_busy() {
        return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
    }
    let engine = envelope::metered_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => {
            return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
        }
    };

    // This is the existing no-import workspace execution shape: a metered,
    // memory-limited Store and an empty Linker. With no linked host surface the
    // guest cannot name, read, or write a host object.
    if module.imports().next().is_some() {
        return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
    }
    let mut store = Store::new(
        &engine,
        envelope::limited_state(WORKSPACE_MEMORY_LIMIT_BYTES),
    );
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(WORKSPACE_FUEL_BUDGET).is_err() {
        return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
    }
    let linker = Linker::<envelope::EnvelopeState>::new(&engine);
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(pre) => match pre.start(&mut store) {
            Ok(instance) => instance,
            Err(_) => {
                return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
            }
        },
        Err(_) => {
            return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
        }
    };
    let memory = match instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)
    {
        Some(memory) => memory,
        None => {
            return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
        }
    };
    let memory_before = Vec::from(memory.data(&store));
    let function = match instance
        .get_export(&store, WORKSPACE_ENTRYPOINT)
        .and_then(Extern::into_func)
    {
        Some(function) => function,
        None => {
            return IsolationCaseEvidence::failed_before_call(host_guard == HOST_GUARD);
        }
    };

    let mut output = [Value::I32(0)];
    let result = function.call(&mut store, &[], &mut output);
    let (memory_out_of_bounds, host_exposed_bytes) = match result {
        Err(wasmi::Error::Trap(trap)) => (
            matches!(trap.trap_code(), Some(TrapCode::MemoryOutOfBounds)),
            0,
        ),
        Ok(()) => (false, bytes_exposed_on_success),
        Err(_) => (false, 0),
    };
    let guest_memory_unchanged = memory.data(&store) == memory_before.as_slice();

    IsolationCaseEvidence {
        memory_out_of_bounds,
        guest_memory_unchanged,
        host_memory_unchanged: host_guard == HOST_GUARD,
        host_exposed_bytes,
        // The exact trap code is retained in this per-case evidence and the
        // single serial record below logs its machine-readable case token.
        trap_recorded: memory_out_of_bounds,
    }
}

pub(crate) fn emit_isolation_selftest() {
    let oob_store = run_hostile_fixture(OOB_STORE_WASM, 0);
    let oob_load = run_hostile_fixture(OOB_LOAD_WASM, 4);
    let oob_offset = run_hostile_fixture(OOB_OFFSET_WASM, 4);
    let host_exposed = oob_store
        .host_exposed_bytes
        .saturating_add(oob_load.host_exposed_bytes)
        .saturating_add(oob_offset.host_exposed_bytes);
    let logged = oob_store.trap_recorded && oob_load.trap_recorded && oob_offset.trap_recorded;
    let pass = oob_store.passed()
        && oob_load.passed()
        && oob_offset.passed()
        && logged
        && host_exposed == 0;

    let line = alloc::format!(
        "RAIOS_ISOLATION selftest={} oob_store={} oob_load={} oob_offset={} logged={} host_exposed={}",
        if pass { "pass" } else { "fail" },
        oob_store.token(),
        oob_load.token(),
        oob_offset.token(),
        if logged { 1 } else { 0 },
        host_exposed,
    );
    serial::write_raw_line(&line);
}
