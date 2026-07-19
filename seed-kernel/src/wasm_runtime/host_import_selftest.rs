use super::*;

use raios_core::{
    host_import_abi_v1::HOST_IMPORT_ERROR_CAPABILITY_DENIED,
    scoped_wasm_import_grant::KNOWN_HOST_IMPORTS_DOTTED, tls13_session::Tls13Denial,
};
use wasmi::core::ValueType;

const SELFTEST_SERVICE_ID: &str = "test.fixture.host_import_selftest";
const SELFTEST_ENTRYPOINT: &str = "raios_service_main";

// Imports env.counter_get but receives only env.log. Export/entrypoint:
// raios_service_main / raios_service_main. The body would mutate the current-
// boot counter if the module-vs-grant preflight ever failed open.
const MISSING_IMPORT_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x00, 0x01, 0x7e, 0x60,
    0x00, 0x01, 0x7f, 0x02, 0x13, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0b, 0x63, 0x6f, 0x75, 0x6e, 0x74,
    0x65, 0x72, 0x5f, 0x67, 0x65, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x07, 0x16, 0x01, 0x12,
    0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61,
    0x69, 0x6e, 0x00, 0x01, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x10, 0x00, 0x1a, 0x41, 0x00, 0x0b,
];

// Imports a device-shaped authority that is absent from the known host-import
// vocabulary. Export/entrypoint: raios_service_main / raios_service_main. If
// the import-grant preflight ever failed open, the body would attempt the
// device.mmio_read call; the executor must refuse before instantiation.
const DEVICE_ABSENT_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x02,
    0x14, 0x01, 0x06, 0x64, 0x65, 0x76, 0x69, 0x63, 0x65, 0x09, 0x6d, 0x6d, 0x69, 0x6f, 0x5f, 0x72,
    0x65, 0x61, 0x64, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69,
    0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00,
    0x01, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b,
];

// Imports the real granted env.log surface as (i64, i32) -> () instead of
// (i32, i32) -> (). Export/entrypoint: raios_service_main / never entered;
// wasmi's linker must reject the incompatible import before instantiation.
const WRONG_SIGNATURE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x02, 0x7e, 0x7f, 0x00,
    0x60, 0x00, 0x01, 0x7f, 0x02, 0x0b, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x03, 0x6c, 0x6f, 0x67, 0x00,
    0x00, 0x03, 0x02, 0x01, 0x01, 0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73,
    0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x0a, 0x06, 0x01,
    0x04, 0x00, 0x41, 0x00, 0x0b,
];

// Calls the real linked env.log with ptr=-1,len=0. Export/entrypoint:
// raios_service_main / raios_service_main; memory is exported as "memory".
const BAD_OFFSET_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x02, 0x7f, 0x7f, 0x00,
    0x60, 0x00, 0x01, 0x7f, 0x02, 0x0b, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x03, 0x6c, 0x6f, 0x67, 0x00,
    0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x1f, 0x02, 0x12, 0x72, 0x61,
    0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e,
    0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x0c, 0x01, 0x0a, 0x00,
    0x41, 0x7f, 0x41, 0x00, 0x10, 0x00, 0x41, 0x00, 0x0b,
];

// Calls the real linked env.output_write with ptr=0,len=4097. Export/entrypoint:
// raios_service_main / raios_service_main; memory is exported as "memory".
const BAD_LENGTH_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0b, 0x02, 0x60, 0x02, 0x7f, 0x7f, 0x01,
    0x7f, 0x60, 0x00, 0x01, 0x7f, 0x02, 0x14, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0c, 0x6f, 0x75, 0x74,
    0x70, 0x75, 0x74, 0x5f, 0x77, 0x72, 0x69, 0x74, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05,
    0x03, 0x01, 0x00, 0x01, 0x07, 0x1f, 0x02, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65,
    0x72, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d,
    0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x0e, 0x01, 0x0c, 0x00, 0x41, 0x00, 0x41, 0x81, 0x20, 0x10,
    0x00, 0x1a, 0x41, 0x00, 0x0b,
];

// Calls the real crypto.sha256 shim with fabricated session handle 0x1234.
// Export/entrypoint: raios_service_main / raios_service_main; memory is
// exported as "memory" so the shim can validate the guest range.
const BAD_HANDLE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0d, 0x02, 0x60, 0x04, 0x7f, 0x7f, 0x7f,
    0x7f, 0x01, 0x7f, 0x60, 0x00, 0x01, 0x7f, 0x02, 0x11, 0x01, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74,
    0x6f, 0x06, 0x73, 0x68, 0x61, 0x32, 0x35, 0x36, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03,
    0x01, 0x00, 0x01, 0x07, 0x1f, 0x02, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72,
    0x76, 0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f,
    0x72, 0x79, 0x02, 0x00, 0x0a, 0x0f, 0x01, 0x0d, 0x00, 0x41, 0xb4, 0x24, 0x41, 0x00, 0x41, 0x00,
    0x41, 0x00, 0x10, 0x00, 0x0b,
];

#[derive(Clone, Copy)]
struct CaseEvidence {
    expected_reason: &'static str,
    actual_reason: &'static str,
    denied: bool,
    surface_unchanged: bool,
}

impl CaseEvidence {
    const fn failed(expected_reason: &'static str, actual_reason: &'static str) -> Self {
        Self {
            expected_reason,
            actual_reason,
            denied: false,
            surface_unchanged: false,
        }
    }

    fn passed(self) -> bool {
        self.denied && self.actual_reason == self.expected_reason && self.surface_unchanged
    }
}

fn fixture_contract(
    bytes: &[u8],
    expected_module: &str,
    expected_name: &str,
    memory_exported: bool,
) -> bool {
    let engine = envelope::metered_engine();
    let Ok(module) = Module::new(&engine, bytes) else {
        return false;
    };
    let mut imports = module.imports();
    let exact_import = imports
        .next()
        .is_some_and(|import| import.module() == expected_module && import.name() == expected_name)
        && imports.next().is_none();

    let mut export_count = 0usize;
    let mut exact_entrypoint = false;
    let mut has_memory = false;
    for export in module.exports() {
        export_count += 1;
        if export.name() == SELFTEST_ENTRYPOINT {
            exact_entrypoint = export
                .ty()
                .func()
                .is_some_and(|ty| ty.params().is_empty() && ty.results() == [ValueType::I32]);
        } else if export.name() == "memory" {
            has_memory = true;
        }
    }
    exact_import
        && exact_entrypoint
        && has_memory == memory_exported
        && export_count == if memory_exported { 2 } else { 1 }
}

fn run_missing_case() -> CaseEvidence {
    let run = envelope::execute_module_bytes(
        MISSING_IMPORT_WASM,
        SELFTEST_ENTRYPOINT,
        SELFTEST_SERVICE_ID,
        true,
        &[("env", "log")],
    );
    let surface_unchanged = !run.instantiation_ok
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let denied = fixture_contract(MISSING_IMPORT_WASM, "env", "counter_get", false)
        && run.validation_ok
        && run.import_grant_performed
        && run.import_grant_status == "import_grant_authorized"
        && run.authorized_import_count == 1
        && run.linked_host_import_count == 1
        && !run.module_imports_within_authorized_list
        && run.missing_import_module.as_deref() == Some("env")
        && run.missing_import_name.as_deref() == Some("counter_get")
        && run.run_outcome == "module_import_not_authorized"
        && surface_unchanged;
    CaseEvidence {
        expected_reason: "module_import_not_authorized",
        actual_reason: run.run_outcome,
        denied,
        surface_unchanged,
    }
}

fn run_device_absent_case() -> CaseEvidence {
    const REASON: &str = "unknown_host_import";
    const REQUESTED_IMPORTS: &[(&str, &str)] = &[("device", "mmio_read")];
    let run = envelope::execute_module_bytes(
        DEVICE_ABSENT_WASM,
        SELFTEST_ENTRYPOINT,
        SELFTEST_SERVICE_ID,
        true,
        REQUESTED_IMPORTS,
    );

    // device.mmio_read cannot be named by a grant. This is the same executor
    // refusal path as capability.selftest's service.start case: no host import
    // is linked and no instance, guest body, or host/device effect is reached.
    let surface_unchanged = !run.instantiation_ok
        && run.linked_host_import_count == 0
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let denied = fixture_contract(DEVICE_ABSENT_WASM, "device", "mmio_read", false)
        && !run.validation_ok
        && !run.import_grant_performed
        && run.import_grant_status == "denied"
        && run.import_grant_reason == REASON
        && run.authorized_import_count == 0
        && !run.module_imports_within_authorized_list
        && run.run_outcome == "import_grant_denied"
        && run.missing_import_module.is_none()
        && run.missing_import_name.is_none()
        && surface_unchanged;
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: run.import_grant_reason,
        denied,
        surface_unchanged,
    }
}

fn known_host_import_device_surface_count() -> usize {
    // Enumerate the same source-of-truth dotted vocabulary exported by
    // agent_protocol_honesty; device authority must remain absent from it.
    KNOWN_HOST_IMPORTS_DOTTED
        .iter()
        .filter(|host_import| {
            host_import
                .split(|separator| separator == '.' || separator == '_')
                .any(|component| {
                    matches!(
                        component,
                        "device"
                            | "devices"
                            | "irq"
                            | "irqs"
                            | "interrupt"
                            | "interrupts"
                            | "mmio"
                            | "dma"
                            | "ioport"
                            | "port"
                            | "ports"
                    )
                })
        })
        .count()
}

fn run_wrong_signature_case() -> CaseEvidence {
    const REASON: &str = "signature_mismatch";
    const REQUESTED_IMPORTS: &[(&str, &str)] = &[("env", "log")];
    let run = envelope::execute_module_bytes(
        WRONG_SIGNATURE_WASM,
        SELFTEST_ENTRYPOINT,
        SELFTEST_SERVICE_ID,
        true,
        REQUESTED_IMPORTS,
    );
    let engine = envelope::metered_engine();
    let Ok(module) = Module::new(&engine, WRONG_SIGNATURE_WASM) else {
        return CaseEvidence::failed(REASON, "module_compile_failed");
    };
    let Ok(authorized) =
        envelope::authorize_wasm_imports(SELFTEST_SERVICE_ID, true, REQUESTED_IMPORTS)
    else {
        return CaseEvidence::failed(REASON, "grant_authorization_failed");
    };
    let mut store = Store::new(
        &engine,
        envelope::limited_state(WORKSPACE_MEMORY_LIMIT_BYTES),
    );
    let mut linker = Linker::<envelope::EnvelopeState>::new(&engine);
    let linked = envelope::define_granted_imports(&mut linker, &authorized) == Ok(1);
    let linker_signature_mismatch = linked
        && matches!(
            linker.instantiate(&mut store, &module),
            Err(wasmi::Error::Linker(LinkerError::FuncTypeMismatch {
                name,
                expected,
                found,
            })) if name.module() == "env"
                && name.name() == "log"
                && expected.params() == [ValueType::I64, ValueType::I32]
                && expected.results().is_empty()
                && found.params() == [ValueType::I32, ValueType::I32]
                && found.results().is_empty()
        );
    let surface_unchanged = !run.instantiation_ok
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty()
        && store.data().log_line.is_none();
    let executor_reached_instantiation = run.validation_ok
        && run.import_grant_performed
        && run.import_grant_status == "import_grant_authorized"
        && run.authorized_import_count == 1
        && run.linked_host_import_count == 1
        && run.module_imports_within_authorized_list
        && run.missing_import_module.is_none()
        && run.missing_import_name.is_none()
        && run.run_outcome == "instantiation_failed";
    let denied = fixture_contract(WRONG_SIGNATURE_WASM, "env", "log", false)
        && executor_reached_instantiation
        && linker_signature_mismatch
        && surface_unchanged;
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: if !executor_reached_instantiation {
            "executor_did_not_reach_instantiation"
        } else if linker_signature_mismatch {
            REASON
        } else {
            "linker_signature_mismatch_missing"
        },
        denied,
        surface_unchanged,
    }
}

struct EnvTrapDiagnostic {
    exact_trap: bool,
    guest_memory_unchanged: bool,
    log_unchanged: bool,
}

fn diagnose_env_trap(
    bytes: &[u8],
    requested_imports: &[(&str, &str)],
    expected_trap: &str,
) -> EnvTrapDiagnostic {
    let Ok(authorized) =
        envelope::authorize_wasm_imports(SELFTEST_SERVICE_ID, true, requested_imports)
    else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let engine = envelope::metered_engine();
    let Ok(module) = Module::new(&engine, bytes) else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let mut store = Store::new(
        &engine,
        envelope::limited_state(WORKSPACE_MEMORY_LIMIT_BYTES),
    );
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(WORKSPACE_FUEL_BUDGET).is_err() {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    }
    let mut linker = Linker::<envelope::EnvelopeState>::new(&engine);
    if envelope::define_granted_imports(&mut linker, &authorized).is_err() {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    }
    let Ok(pre) = linker.instantiate(&mut store, &module) else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let Ok(instance) = pre.start(&mut store) else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let Some(memory) = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)
    else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let before = Vec::from(memory.data(&store));
    let Some(function) = instance
        .get_export(&store, SELFTEST_ENTRYPOINT)
        .and_then(Extern::into_func)
    else {
        return EnvTrapDiagnostic {
            exact_trap: false,
            guest_memory_unchanged: false,
            log_unchanged: false,
        };
    };
    let mut outputs = [Value::I32(0)];
    let exact_trap = matches!(
        function.call(&mut store, &[], &mut outputs),
        Err(wasmi::Error::Trap(trap)) if alloc::format!("{trap}") == expected_trap
    );
    EnvTrapDiagnostic {
        exact_trap,
        guest_memory_unchanged: memory.data(&store) == before.as_slice(),
        log_unchanged: store.data().log_line.is_none(),
    }
}

fn run_bad_offset_case() -> CaseEvidence {
    const REASON: &str = "env_log_negative_pointer_or_length";
    let diagnostic = diagnose_env_trap(
        BAD_OFFSET_WASM,
        &[("env", "log")],
        "env.log negative pointer or length",
    );
    let run = envelope::execute_module_bytes(
        BAD_OFFSET_WASM,
        SELFTEST_ENTRYPOINT,
        SELFTEST_SERVICE_ID,
        true,
        &[("env", "log")],
    );
    let surface_unchanged = diagnostic.guest_memory_unchanged
        && diagnostic.log_unchanged
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let denied = fixture_contract(BAD_OFFSET_WASM, "env", "log", true)
        && run.validation_ok
        && run.instantiation_ok
        && run.linked_host_import_count == 1
        && run.module_imports_within_authorized_list
        && run.run_outcome == "trap"
        && run.return_value.is_none()
        && diagnostic.exact_trap
        && surface_unchanged;
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: if diagnostic.exact_trap {
            REASON
        } else {
            "env_log_trap_mismatch"
        },
        denied,
        surface_unchanged,
    }
}

fn run_bad_length_case() -> CaseEvidence {
    const REASON: &str = "env_output_write_length_exceeds_4096_bytes";
    let diagnostic = diagnose_env_trap(
        BAD_LENGTH_WASM,
        &[("env", "output_write")],
        "env.output_write length exceeds 4096 bytes",
    );
    let run = envelope::execute_module_bytes(
        BAD_LENGTH_WASM,
        SELFTEST_ENTRYPOINT,
        SELFTEST_SERVICE_ID,
        true,
        &[("env", "output_write")],
    );
    let surface_unchanged = diagnostic.guest_memory_unchanged
        && diagnostic.log_unchanged
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let denied = fixture_contract(BAD_LENGTH_WASM, "env", "output_write", true)
        && run.validation_ok
        && run.instantiation_ok
        && run.linked_host_import_count == 1
        && run.module_imports_within_authorized_list
        && run.run_outcome == "trap"
        && run.return_value.is_none()
        && diagnostic.exact_trap
        && surface_unchanged;
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: if diagnostic.exact_trap {
            REASON
        } else {
            "env_output_write_trap_mismatch"
        },
        denied,
        surface_unchanged,
    }
}

fn run_bad_handle_case() -> CaseEvidence {
    const REASON: &str = "crypto_foreign_session";
    if !fixture_contract(BAD_HANDLE_WASM, "crypto", "sha256", true) {
        return CaseEvidence::failed(REASON, "fixture_contract_invalid");
    }
    let engine = invocation::beyond_env_engine();
    let Ok(module) = Module::new(&engine, BAD_HANDLE_WASM) else {
        return CaseEvidence::failed(REASON, "module_compile_failed");
    };
    let invocation_id = invocation::NEXT_BEYOND_ENV_INVOCATION_ID
        .fetch_add(1, Ordering::Relaxed)
        .max(1);
    let authority = InvocationAuthority {
        service_id: SELFTEST_SERVICE_ID,
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation: crate::input::secure_attention_kill_generation(),
        policy_allows_beyond_env: false,
    };
    let mut store = Store::new(
        &engine,
        invocation::BeyondEnvState {
            lifecycle: InvocationLifecycle::new(
                authority,
                invocation::runtime_now_ms(),
                invocation::BEYOND_ENV_WALL_BUDGET_MS,
                invocation::BEYOND_ENV_PUMP_STEP_BUDGET,
                invocation_id as u32,
            ),
            behavior: invocation::FixtureBehavior::Crypto(
                crypto_shims::CryptoFixtureScenario::ForeignSession,
            ),
            net: None,
            crypto: crypto_shims::CryptoInvocationState::new(None),
            acquire: None,
            limits: invocation::beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    if store
        .add_fuel(invocation::BEYOND_ENV_FIXTURE_FUEL_BUDGET)
        .is_err()
    {
        return CaseEvidence::failed(REASON, "fuel_metering_unavailable");
    }
    let mut linker = Linker::<invocation::BeyondEnvState>::new(&engine);
    if linker
        .func_wrap("crypto", "sha256", crypto_shims::host_crypto_sha256)
        .is_err()
    {
        return CaseEvidence::failed(REASON, "link_failed");
    }
    let Ok(pre) = linker.instantiate(&mut store, &module) else {
        return CaseEvidence::failed(REASON, "instantiation_failed");
    };
    let Ok(instance) = pre.start(&mut store) else {
        return CaseEvidence::failed(REASON, "start_failed");
    };
    let Some(memory) = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)
    else {
        return CaseEvidence::failed(REASON, "memory_missing");
    };
    let before = Vec::from(memory.data(&store));
    let Some(function) = instance
        .get_export(&store, SELFTEST_ENTRYPOINT)
        .and_then(Extern::into_func)
    else {
        return CaseEvidence::failed(REASON, "entrypoint_missing");
    };
    let mut outputs = [Value::I32(0)];
    let call_ok = function.call(&mut store, &[], &mut outputs).is_ok();
    let denied = call_ok
        && outputs[0].i32() == Some(HOST_IMPORT_ERROR_CAPABILITY_DENIED)
        && store.data().crypto.last_denial == Some(Tls13Denial::ForeignSession)
        && store.data().crypto.call_count[1] == 1;
    let surface_unchanged =
        memory.data(&store) == before.as_slice() && !store.data().crypto.slot.is_open();
    invocation::finish_store(&mut store, TerminalOutcome::Finished);
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: store
            .data()
            .crypto
            .last_denial
            .map_or("crypto_denial_missing", Tls13Denial::as_str),
        denied: denied && surface_unchanged,
        surface_unchanged,
    }
}

fn run_bad_index_case() -> CaseEvidence {
    const REASON: &str = "acquire_chunk_index_mismatch";
    let fixture = probes::run_acquire_fixture_probe();
    let Some(case) = fixture.cases.iter().find(|case| case.name == "wrong_index") else {
        return CaseEvidence::failed(REASON, "wrong_index_case_missing");
    };
    let engine = invocation::beyond_env_engine();
    let fixture_export_matches = Module::new(&engine, acquire_shims::ACQUIRE_SHIM_WASM_MODULE)
        .ok()
        .is_some_and(|module| {
            module.exports().any(|export| {
                export.name() == "chunk"
                    && export.ty().func().is_some_and(|ty| {
                        ty.params() == [ValueType::I32, ValueType::I32, ValueType::I32]
                            && ty.results() == [ValueType::I32]
                    })
            })
        });
    let surface_unchanged = case.prior_candidate_unchanged
        && case.pending_dropped
        && fixture.all_prior_candidates_preserved
        && fixture.direct_candidate_intake_calls == 0;
    CaseEvidence {
        expected_reason: REASON,
        actual_reason: case.denial,
        denied: fixture_export_matches && case.denied && surface_unchanged,
        surface_unchanged,
    }
}

fn record_case(name: &str, evidence: CaseEvidence) -> bool {
    serial::write_raw_line(&alloc::format!(
        "RAIOS_HOSTIMPORT case={} typed_reason={} denied={} recorded=1 surface_unchanged={}",
        name,
        evidence.actual_reason,
        u8::from(evidence.denied),
        u8::from(evidence.surface_unchanged),
    ));
    evidence.denied
}

fn append_first_failure(line: &mut String, cases: &[(&str, CaseEvidence)]) {
    if let Some((name, evidence)) = cases.iter().find(|(_, evidence)| !evidence.passed()) {
        line.push_str(" hi_reason=");
        line.push_str(name);
        line.push('=');
        line.push_str(evidence.actual_reason);
    }
}

pub(crate) fn emit_host_import_selftest() {
    let controller = crate::pci::find_mass_storage_controller();
    let disk_before = controller.map(wasi_build_job::persist_region_hashes);

    let missing = run_missing_case();
    let wrong_sig = run_wrong_signature_case();
    let bad_offset = run_bad_offset_case();
    let bad_length = run_bad_length_case();
    let bad_handle = run_bad_handle_case();
    let bad_index = run_bad_index_case();
    let device_absent = run_device_absent_case();
    let device_surface_count = known_host_import_device_surface_count();
    let device_surfaces_absent = device_surface_count == 0;

    let missing_logged = record_case("missing", missing);
    let wrong_sig_logged = record_case("wrong_sig", wrong_sig);
    let bad_offset_logged = record_case("bad_offset", bad_offset);
    let bad_length_logged = record_case("bad_length", bad_length);
    let bad_handle_logged = record_case("bad_handle", bad_handle);
    let bad_index_logged = record_case("bad_index", bad_index);
    let device_absent_logged = record_case("device_absent", device_absent);
    let logged = missing_logged
        && wrong_sig_logged
        && bad_offset_logged
        && bad_length_logged
        && bad_handle_logged
        && bad_index_logged
        && device_absent_logged;

    let disk = wasi_build_job::compare_persist_region_hashes(controller, disk_before);
    let persistent_effect_free = match &disk {
        wasi_build_job::StorageSelftestDiskEvidence::Absent => true,
        wasi_build_job::StorageSelftestDiskEvidence::Present {
            reclog_unchanged,
            artstor_unchanged,
        } => *reclog_unchanged && *artstor_unchanged,
    };
    let host_effect_free = missing.surface_unchanged
        && wrong_sig.surface_unchanged
        && bad_offset.surface_unchanged
        && bad_length.surface_unchanged
        && bad_handle.surface_unchanged
        && device_absent.surface_unchanged;
    let peer_effect_free = bad_handle.surface_unchanged && bad_index.surface_unchanged;
    let partial_effect_free = host_effect_free && peer_effect_free && persistent_effect_free;
    let pass = missing.passed()
        && wrong_sig.passed()
        && bad_offset.passed()
        && bad_length.passed()
        && bad_handle.passed()
        && bad_index.passed()
        && device_absent.passed()
        && device_surfaces_absent
        && logged
        && partial_effect_free;

    let mut line = alloc::format!(
        "RAIOS_HOSTIMPORT selftest={} missing={} wrong_sig={} bad_offset={} bad_length={} bad_handle={} bad_index={} device_import={} device_surfaces={} logged={} host_effect={} peer_effect={} persistent_effect={} partial_effect={}",
        if pass { "pass" } else { "fail" },
        if missing.passed() { "refused" } else { "failed" },
        if wrong_sig.passed() { "signature_mismatch" } else { "failed" },
        if bad_offset.passed() { "trapped" } else { "failed" },
        if bad_length.passed() { "denied" } else { "failed" },
        if bad_handle.passed() { "denied" } else { "failed" },
        if bad_index.passed() { "denied" } else { "failed" },
        if device_absent.passed() { "refused" } else { "failed" },
        device_surface_count,
        u8::from(logged),
        if host_effect_free { 0 } else { 1 },
        if peer_effect_free { 0 } else { 1 },
        if persistent_effect_free { 0 } else { 1 },
        if partial_effect_free { 0 } else { 1 },
    );
    append_first_failure(
        &mut line,
        &[
            ("missing", missing),
            ("wrong_sig", wrong_sig),
            ("bad_offset", bad_offset),
            ("bad_length", bad_length),
            ("bad_handle", bad_handle),
            ("bad_index", bad_index),
            ("device_absent", device_absent),
        ],
    );
    if missing.passed()
        && wrong_sig.passed()
        && bad_offset.passed()
        && bad_length.passed()
        && bad_handle.passed()
        && bad_index.passed()
        && device_absent.passed()
        && !device_surfaces_absent
    {
        line.push_str(" hi_reason=device_surfaces=");
        line.push_str(&alloc::format!("{device_surface_count}"));
    }
    if missing.passed()
        && wrong_sig.passed()
        && bad_offset.passed()
        && bad_length.passed()
        && bad_handle.passed()
        && bad_index.passed()
        && device_absent.passed()
        && device_surfaces_absent
        && !persistent_effect_free
    {
        line.push_str(" hi_reason=persistent=persist_region_changed");
    }
    serial::write_raw_line(&line);

    let disk_line = match disk {
        wasi_build_job::StorageSelftestDiskEvidence::Absent => {
            String::from("RAIOS_HOSTIMPORT disk=absent")
        }
        wasi_build_job::StorageSelftestDiskEvidence::Present {
            reclog_unchanged,
            artstor_unchanged,
        } => alloc::format!(
            "RAIOS_HOSTIMPORT disk={} reclog_unchanged={} artstor_unchanged={}",
            if reclog_unchanged && artstor_unchanged {
                "pass"
            } else {
                "fail"
            },
            u8::from(reclog_unchanged),
            u8::from(artstor_unchanged),
        ),
    };
    serial::write_raw_line(&disk_line);
}
