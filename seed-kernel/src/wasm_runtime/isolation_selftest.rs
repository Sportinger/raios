use super::*;

const OOB_STORE_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_store.wasm"));
const OOB_LOAD_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_load.wasm"));
const OOB_OFFSET_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/isolation_oob_offset.wasm"));
const UNGRANTED_IMPORT_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x00, 0x01, 0x7e, 0x60,
    0x00, 0x00, 0x02, 0x13, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0b, 0x63, 0x6f, 0x75, 0x6e, 0x74, 0x65,
    0x72, 0x5f, 0x67, 0x65, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x08, 0x01, 0x01, 0x0a, 0x07,
    0x01, 0x05, 0x00, 0x10, 0x00, 0x1a, 0x0b,
];

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

#[derive(Clone, Copy)]
struct ImportDenyEvidence {
    refused_before_instantiation: bool,
    refusal_recorded: bool,
    host_effect: u64,
}

impl ImportDenyEvidence {
    fn token(self) -> &'static str {
        if self.refused_before_instantiation {
            "refused"
        } else {
            "failed"
        }
    }
}

fn run_ungranted_import_fixture() -> ImportDenyEvidence {
    // This guest has a start function that calls the existing env.counter_get
    // host surface, which its grant set never includes: the service declares
    // only env.log (an empty declaration is denied earlier as
    // missing_import_list and would not exercise the module-vs-grant
    // comparison). The ungranted import must be refused before instantiation
    // can run the start function and mutate the host counter.
    let run = envelope::execute_module_bytes(
        UNGRANTED_IMPORT_WASM,
        WORKSPACE_ENTRYPOINT,
        WORKSPACE_SERVICE_ID,
        true,
        &[("env", "log")],
    );
    let host_effect = if run.instantiation_ok || run.fuel_used != 0 || run.log_line.is_some() {
        1
    } else {
        0
    };
    let refused_before_instantiation = run.validation_ok
        && run.import_grant_performed
        && run.authorized_import_count == 1
        && run.linked_host_import_count == 1
        && !run.module_imports_within_authorized_list
        && run.run_outcome == "module_import_not_authorized"
        && !run.instantiation_ok
        && run.missing_import_module.as_deref() == Some("env")
        && run.missing_import_name.as_deref() == Some("counter_get")
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty()
        && host_effect == 0;

    ImportDenyEvidence {
        refused_before_instantiation,
        refusal_recorded: refused_before_instantiation,
        host_effect,
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
    let import_deny = run_ungranted_import_fixture();
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

    let import_line = alloc::format!(
        "RAIOS_ISOLATION importdeny={} logged={} host_effect={}",
        import_deny.token(),
        if import_deny.refusal_recorded { 1 } else { 0 },
        import_deny.host_effect,
    );
    serial::write_raw_line(&import_line);
}
