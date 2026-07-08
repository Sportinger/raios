use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt::Write;
use raios_core::{
    scoped_wasm_import_grant::{
        authorized_import_list_sha256, evaluate_wasm_import_grant, WasmImportGrantDecision,
        WasmImportGrantInput,
    },
    sha256_bytes,
};
use spin::Mutex;
use wasmi::{
    core::{Trap, TrapCode},
    errors::LinkerError,
    Caller, Config, Engine, Extern, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, Value,
};

use crate::serial;

include!(concat!(env!("OUT_DIR"), "/echo_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/bufecho_wasm_artifact.rs"));

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";
const FORBIDDEN_WRITE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x02, 0x17,
    0x01, 0x03, 0x65, 0x6e, 0x76, 0x0f, 0x66, 0x6f, 0x72, 0x62, 0x69, 0x64, 0x64, 0x65, 0x6e, 0x5f,
    0x77, 0x72, 0x69, 0x74, 0x65, 0x00, 0x00,
];
const MALFORMED_WASM_MODULE: &[u8] = b"\0bsm\x01\0\0\0";
const OVER_MEMORY_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x02,
];
const FUEL_LOOP_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00,
    0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b,
];
const UNREACHABLE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x05, 0x01, 0x03, 0x00,
    0x00, 0x0b,
];
const MAX_WASM_LOG_BYTES: usize = 256;
// Keep in lockstep with the Phase-B guest buffer size.
const MAX_WASM_INPUT_BYTES: usize = 4096;
const MAX_WASM_OUTPUT_BYTES: usize = 4096;
const WASM_MEMORY_PAGE_BYTES: usize = 64 * 1024;
pub(crate) const ECHO_WASM_FUEL_BUDGET: u64 = 10_000;
/// Deliberately-tiny fuel budget for the labeled fuel-starvation fault injection
/// (`run_echo_fuel_starved`). A real echo invoke under this budget exhausts fuel
/// after a single metered step and traps with a genuine wasmi `OutOfFuel`.
pub(crate) const ECHO_WASM_FUEL_STARVED_BUDGET: u64 = 1;
const FUEL_EXHAUSTION_BUDGET: u64 = 1;
const GUEST_TRAP_FUEL_BUDGET: u64 = 100;
pub(crate) const WASM_HARDENING_CASE_COUNT: usize = 4;
pub(crate) const FORBIDDEN_IMPORT_MODULE: &str = "env";
pub(crate) const FORBIDDEN_IMPORT_NAME: &str = "forbidden_write";
pub(crate) const ECHO_SERVICE_ID: &str = "svc.demo.echo";
pub(crate) const ECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] =
    &[("env", "log"), ("env", "counter_get")];
pub(crate) const BUFECHO_SERVICE_ID: &str = "svc.demo.bufecho";
pub(crate) const BUFECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
const ZERO_SHA256: [u8; 32] = [0; 32];

static CURRENT_BOOT_COUNTER: Mutex<u64> = Mutex::new(0);

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;
#[used]
static ECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_echo_wasm_artifact;
#[used]
static BUFECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_bufecho_wasm_artifact;

pub(crate) fn validate_empty_module_bytes() -> bool {
    let wasm = Vec::from(EMPTY_WASM_MODULE).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    validate_module_bytes(bytes)
}

pub(crate) fn validate_echo_wasm_artifact() -> bool {
    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == ECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_bufecho_wasm_artifact() -> bool {
    let wasm = Vec::from(BUFECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == BUFECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) struct EchoProbe {
    pub(crate) artifact_hash: [u8; 32],
    pub(crate) descriptor_hash: [u8; 32],
    pub(crate) signature_envelope_hash: [u8; 32],
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
    pub(crate) forbidden_validation_ok: bool,
    pub(crate) forbidden_instantiation_ok: bool,
    pub(crate) forbidden_link_error_kind: &'static str,
    pub(crate) forbidden_missing_import_module: Option<&'static str>,
    pub(crate) forbidden_missing_import_name: Option<&'static str>,
    pub(crate) forbidden_boundary_held: bool,
    pub(crate) hardening_cases: [WasmHardeningCase; WASM_HARDENING_CASE_COUNT],
}

pub(crate) struct EchoRunEvidence {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
    pub(crate) import_grant_performed: bool,
    pub(crate) import_grant_status: &'static str,
    pub(crate) import_grant_reason: &'static str,
    pub(crate) authorized_import_count: u64,
    pub(crate) authorized_import_list_sha256: [u8; 32],
    pub(crate) captured_output_len: u64,
    pub(crate) captured_output_sha256: [u8; 32],
    pub(crate) linked_host_import_count: u64,
    pub(crate) module_imports_within_authorized_list: bool,
    pub(crate) missing_import_module: Option<String>,
    pub(crate) missing_import_name: Option<String>,
}

pub(crate) struct BufechoRoundtripEvidence {
    pub(crate) run: EchoRunEvidence,
    pub(crate) input_len: u64,
    pub(crate) input_sha256: [u8; 32],
}

/// Evidence from a labeled fuel-starvation fault injection against the real echo
/// module. `out_of_fuel` is TRUE only when the caught trap was specifically a
/// wasmi `OutOfFuel`; any other trap kind is reported honestly in `run_outcome`
/// with `out_of_fuel=false` so a caller never mislabels a different fault as a
/// fuel wedge.
pub(crate) struct EchoFuelStarvedEvidence {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) out_of_fuel: bool,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct WasmHardeningCase {
    pub(crate) name: &'static str,
    pub(crate) mechanism: &'static str,
    pub(crate) expected_outcome: &'static str,
    pub(crate) actual_outcome: &'static str,
    pub(crate) passed: bool,
}

pub(crate) fn run_echo_probe() -> EchoProbe {
    let positive = run_echo_service();
    let negative = instantiate_forbidden_import_module();
    let hardening_cases = run_hardening_cases();

    EchoProbe {
        artifact_hash: ECHO_WASM_ARTIFACT_BYTES_HASH,
        descriptor_hash: ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        signature_envelope_hash: ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        validation_ok: positive.validation_ok,
        instantiation_ok: positive.instantiation_ok,
        run_outcome: positive.run_outcome,
        return_value: positive.return_value,
        fuel_budget: positive.fuel_budget,
        fuel_used: positive.fuel_used,
        log_line: positive.log_line,
        forbidden_validation_ok: negative.validation_ok,
        forbidden_instantiation_ok: negative.instantiation_ok,
        forbidden_link_error_kind: negative.link_error_kind,
        forbidden_missing_import_module: negative.missing_import_module,
        forbidden_missing_import_name: negative.missing_import_name,
        forbidden_boundary_held: negative.boundary_held,
        hardening_cases,
    }
}

pub(crate) fn run_echo_service() -> EchoRunEvidence {
    execute_echo_module(validate_echo_wasm_artifact())
}

pub(crate) fn run_bufecho_roundtrip(input: &[u8]) -> BufechoRoundtripEvidence {
    let capped_len = input.len().min(MAX_WASM_INPUT_BYTES);
    let capped = &input[..capped_len];
    BufechoRoundtripEvidence {
        run: execute_validated_module_bytes(
            BUFECHO_WASM_ARTIFACT_BYTES,
            "raios_service_main",
            BUFECHO_SERVICE_ID,
            true,
            BUFECHO_AUTHORIZED_IMPORTS,
            validate_bufecho_wasm_artifact(),
            capped,
        ),
        input_len: capped_len as u64,
        input_sha256: sha256_bytes(capped),
    }
}

pub(crate) fn run_bufecho_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        BUFECHO_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        BUFECHO_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_bufecho_wasm_artifact(),
        b"raios-m11-bufecho-unauthorized-nonce",
    )
}

/// Labeled fault injection: run the REAL echo artifact (`raios_service_main`)
/// through a metered store carrying only `ECHO_WASM_FUEL_STARVED_BUDGET` fuel, so
/// the invoke genuinely traps with wasmi `OutOfFuel` (never simulated). The trap
/// is an `Err` value that is CAUGHT here — never unwrapped/panicked — and
/// classified via `classify_trap_error`; the cooperative kernel loop is unharmed.
/// Reuses `metered_engine`/`default_state`/`define_granted_imports` exactly
/// like the healthy path so the ONLY difference is the fuel budget.
pub(crate) fn run_echo_fuel_starved() -> EchoFuelStarvedEvidence {
    if !validate_echo_wasm_artifact() {
        return fuel_starved_evidence(false, false, "validation_failed", false, 0, None);
    }
    let authorized = match authorize_wasm_imports(ECHO_SERVICE_ID, true, ECHO_AUTHORIZED_IMPORTS) {
        Ok(authorized) => authorized,
        Err(_) => return fuel_starved_evidence(true, false, "import_grant_denied", false, 0, None),
    };

    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return fuel_starved_evidence(true, false, "module_compile_failed", false, 0, None)
        }
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    if store.add_fuel(ECHO_WASM_FUEL_STARVED_BUDGET).is_err() {
        return fuel_starved_evidence(true, false, "fuel_metering_unavailable", false, 0, None);
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    if let Err(reason) = define_granted_imports(&mut linker, &authorized) {
        return fuel_starved_evidence(true, false, reason, false, 0, None);
    }
    if first_unauthorized_module_import(&module, &authorized).is_some() {
        return fuel_starved_evidence(true, false, "module_import_not_authorized", false, 0, None);
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(error) => {
                let outcome = classify_trap_error(error, ExpectedTrap::OutOfFuel);
                return fuel_starved_evidence(
                    true,
                    false,
                    outcome,
                    outcome == "fuel_exhausted",
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                );
            }
        },
        Err(_) => {
            return fuel_starved_evidence(
                true,
                false,
                "instantiation_failed",
                false,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, "raios_service_main")
        .and_then(Extern::into_func)
    else {
        return fuel_starved_evidence(
            true,
            true,
            "entrypoint_missing",
            false,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => fuel_starved_evidence(
            true,
            true,
            "run_success_unexpected",
            false,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
        ),
        Err(error) => {
            let outcome = classify_trap_error(error, ExpectedTrap::OutOfFuel);
            fuel_starved_evidence(
                true,
                true,
                outcome,
                outcome == "fuel_exhausted",
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
    }
}

fn fuel_starved_evidence(
    validation_ok: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    out_of_fuel: bool,
    fuel_used: u64,
    log_line: Option<String>,
) -> EchoFuelStarvedEvidence {
    EchoFuelStarvedEvidence {
        validation_ok,
        instantiation_ok,
        run_outcome,
        out_of_fuel,
        fuel_budget: ECHO_WASM_FUEL_STARVED_BUDGET,
        fuel_used,
        log_line,
    }
}

pub(crate) fn loader_available() -> bool {
    true
}

pub(crate) fn validate_module_bytes(bytes: &[u8]) -> bool {
    let engine = Box::new(wasmi::Engine::default());
    wasmi::Module::new(&engine, bytes).is_ok()
}

struct EnvelopeState {
    log_line: Option<String>,
    staged_input: Vec<u8>,
    captured_output: Vec<u8>,
    limits: StoreLimits,
}

struct AuthorizedWasmImports<'a> {
    imports: &'a [(&'a str, &'a str)],
    decision: WasmImportGrantDecision,
    import_list_sha256: [u8; 32],
}

struct ImportGrantEvidence {
    performed: bool,
    status: &'static str,
    reason: &'static str,
    authorized_import_count: u64,
    authorized_import_list_sha256: [u8; 32],
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import_module: Option<String>,
    missing_import_name: Option<String>,
}

pub(crate) struct NegativeRun {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) link_error_kind: &'static str,
    pub(crate) missing_import_module: Option<&'static str>,
    pub(crate) missing_import_name: Option<&'static str>,
    pub(crate) boundary_held: bool,
}

fn execute_echo_module(validation_ok: bool) -> EchoRunEvidence {
    execute_validated_module_bytes(
        ECHO_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        ECHO_SERVICE_ID,
        true,
        ECHO_AUTHORIZED_IMPORTS,
        validation_ok,
        &[],
    )
}

pub(crate) fn execute_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
) -> EchoRunEvidence {
    execute_validated_module_bytes(
        bytes,
        entrypoint,
        service_id,
        artifact_sha256_present,
        requested_imports,
        validate_module_bytes(bytes),
        &[],
    )
}

fn execute_validated_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
    validation_ok: bool,
    staged_input: &[u8],
) -> EchoRunEvidence {
    let authorized =
        match authorize_wasm_imports(service_id, artifact_sha256_present, requested_imports) {
            Ok(authorized) => authorized,
            Err(decision) => {
                return positive_run(
                    service_id,
                    false,
                    false,
                    "import_grant_denied",
                    None,
                    0,
                    None,
                    import_grant_denied_evidence(decision),
                )
            }
        };
    if !validation_ok {
        return positive_run(
            service_id,
            false,
            false,
            "validation_failed",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }

    let wasm = Vec::from(bytes).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return positive_run(
                service_id,
                true,
                false,
                "module_compile_failed",
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    let mut store = Box::new(Store::new(&engine, buffer_state(staged_input)));
    if store.add_fuel(ECHO_WASM_FUEL_BUDGET).is_err() {
        return positive_run(
            service_id,
            true,
            false,
            "fuel_metering_unavailable",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let linked_host_import_count = match define_granted_imports(&mut linker, &authorized) {
        Ok(count) => count,
        Err(reason) => {
            return positive_run(
                service_id,
                true,
                false,
                reason,
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    if let Some(missing) = first_unauthorized_module_import(&module, &authorized) {
        return positive_run(
            service_id,
            true,
            false,
            "module_import_not_authorized",
            None,
            0,
            None,
            import_grant_evidence(&authorized, linked_host_import_count, false, Some(missing)),
        );
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(_) => {
                return positive_run(
                    service_id,
                    true,
                    false,
                    "instantiation_start_trap",
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                    import_grant_evidence(&authorized, linked_host_import_count, true, None),
                )
            }
        },
        Err(_) => {
            return positive_run(
                service_id,
                true,
                false,
                "instantiation_failed",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, entrypoint)
        .and_then(Extern::into_func)
    else {
        return positive_run(
            service_id,
            true,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
            import_grant_evidence(&authorized, linked_host_import_count, true, None),
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => {
            let return_value = outputs[0].i32();
            let outcome = if return_value.is_some() {
                "success"
            } else {
                "bad_return_type"
            };
            let mut ev = positive_run(
                service_id,
                true,
                true,
                outcome,
                return_value,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev
        }
        Err(_) => {
            let mut ev = positive_run(
                service_id,
                true,
                true,
                "trap",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev
        }
    }
}

fn instantiate_forbidden_import_module() -> NegativeRun {
    let wasm = Vec::from(FORBIDDEN_WRITE_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return NegativeRun {
                validation_ok: false,
                instantiation_ok: false,
                link_error_kind: "module_compile_failed",
                missing_import_module: None,
                missing_import_name: None,
                boundary_held: false,
            }
        }
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let authorized = match authorize_wasm_imports(ECHO_SERVICE_ID, true, ECHO_AUTHORIZED_IMPORTS) {
        Ok(authorized) => authorized,
        Err(_) => {
            return NegativeRun {
                validation_ok: true,
                instantiation_ok: false,
                link_error_kind: "import_grant_denied",
                missing_import_module: None,
                missing_import_name: None,
                boundary_held: false,
            }
        }
    };
    if define_granted_imports(&mut linker, &authorized).is_err() {
        return NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "missing_host_import_implementation",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        };
    }

    match linker.instantiate(&mut *store, &module) {
        Ok(_) => NegativeRun {
            validation_ok: true,
            instantiation_ok: true,
            link_error_kind: "none",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
        Err(wasmi::Error::Linker(LinkerError::MissingDefinition { name, .. })) => {
            let module_ok = name.module() == FORBIDDEN_IMPORT_MODULE;
            let name_ok = name.name() == FORBIDDEN_IMPORT_NAME;
            NegativeRun {
                validation_ok: true,
                instantiation_ok: false,
                link_error_kind: "missing_definition",
                missing_import_module: module_ok.then_some(FORBIDDEN_IMPORT_MODULE),
                missing_import_name: name_ok.then_some(FORBIDDEN_IMPORT_NAME),
                boundary_held: module_ok && name_ok,
            }
        }
        Err(wasmi::Error::Linker(_)) => NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "other_link_error",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
        Err(_) => NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "non_link_error",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
    }
}

pub(crate) fn forbidden_import_link_failure_evidence() -> NegativeRun {
    instantiate_forbidden_import_module()
}

fn run_hardening_cases() -> [WasmHardeningCase; WASM_HARDENING_CASE_COUNT] {
    [
        malformed_bytes_case(),
        over_memory_case(),
        fuel_exhaustion_case(),
        guest_trap_case(),
    ]
}

fn malformed_bytes_case() -> WasmHardeningCase {
    let wasm = Vec::from(MALFORMED_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let actual = if Module::new(&engine, &*wasm).is_err() {
        "module_new_error"
    } else {
        "module_new_ok"
    };

    hardening_case(
        "malformed_bytes",
        "wasmi::Module::new",
        "module_new_error",
        actual,
    )
}

fn over_memory_case() -> WasmHardeningCase {
    let wasm = Vec::from(OVER_MEMORY_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return hardening_case(
                "over_memory",
                "wasmi::StoreLimitsBuilder::memory_size+Store::limiter",
                "limiter_instantiation_error",
                "module_new_error",
            )
        }
    };
    let mut store = Box::new(Store::new(&engine, limited_state(WASM_MEMORY_PAGE_BYTES)));
    store.limiter(|state| &mut state.limits);
    let linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let actual = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(_) => "instantiation_ok",
            Err(_) => "start_trap",
        },
        Err(wasmi::Error::Instantiation(_)) | Err(wasmi::Error::Memory(_)) => {
            "limiter_instantiation_error"
        }
        Err(_) => "other_instantiation_error",
    };

    hardening_case(
        "over_memory",
        "wasmi::StoreLimitsBuilder::memory_size+Store::limiter",
        "limiter_instantiation_error",
        actual,
    )
}

fn fuel_exhaustion_case() -> WasmHardeningCase {
    hardening_case(
        "fuel_exhaustion",
        "wasmi::Config::consume_fuel+Store::add_fuel",
        "fuel_exhausted",
        run_trap_module(
            FUEL_LOOP_WASM_MODULE,
            FUEL_EXHAUSTION_BUDGET,
            ExpectedTrap::OutOfFuel,
        ),
    )
}

fn guest_trap_case() -> WasmHardeningCase {
    hardening_case(
        "guest_trap",
        "wasm_unreachable_trap",
        "guest_trap",
        run_trap_module(
            UNREACHABLE_WASM_MODULE,
            GUEST_TRAP_FUEL_BUDGET,
            ExpectedTrap::Unreachable,
        ),
    )
}

fn run_trap_module(bytes: &[u8], fuel_budget: u64, expected: ExpectedTrap) -> &'static str {
    let wasm = Vec::from(bytes).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => return "module_new_error",
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    if store.add_fuel(fuel_budget).is_err() {
        return "fuel_metering_unavailable";
    }
    let linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(error) => return classify_trap_error(error, expected),
        },
        Err(_) => return "instantiation_error",
    };
    let Some(func) = instance
        .get_export(&*store, "run")
        .and_then(Extern::into_func)
    else {
        return "entrypoint_missing";
    };
    let mut outputs = Vec::<Value>::new().into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => "run_success",
        Err(error) => classify_trap_error(error, expected),
    }
}

#[derive(Clone, Copy)]
enum ExpectedTrap {
    OutOfFuel,
    Unreachable,
}

fn classify_trap_error(error: wasmi::Error, expected: ExpectedTrap) -> &'static str {
    let wasmi::Error::Trap(trap) = error else {
        return "run_error";
    };
    match (expected, trap.trap_code()) {
        (ExpectedTrap::OutOfFuel, Some(TrapCode::OutOfFuel)) => "fuel_exhausted",
        (ExpectedTrap::Unreachable, Some(TrapCode::UnreachableCodeReached)) => "guest_trap",
        (_, Some(TrapCode::OutOfFuel)) => "fuel_exhausted",
        (_, Some(TrapCode::UnreachableCodeReached)) => "guest_trap",
        (_, Some(_)) => "other_trap",
        (_, None) => "trap_without_code",
    }
}

fn hardening_case(
    name: &'static str,
    mechanism: &'static str,
    expected_outcome: &'static str,
    actual_outcome: &'static str,
) -> WasmHardeningCase {
    WasmHardeningCase {
        name,
        mechanism,
        expected_outcome,
        actual_outcome,
        passed: actual_outcome == expected_outcome,
    }
}

fn positive_run(
    service_id: &str,
    validation_ok: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    log_line: Option<String>,
    import_grant: ImportGrantEvidence,
) -> EchoRunEvidence {
    let evidence = EchoRunEvidence {
        validation_ok,
        instantiation_ok,
        run_outcome,
        return_value,
        fuel_budget: ECHO_WASM_FUEL_BUDGET,
        fuel_used,
        log_line,
        import_grant_performed: import_grant.performed,
        import_grant_status: import_grant.status,
        import_grant_reason: import_grant.reason,
        authorized_import_count: import_grant.authorized_import_count,
        authorized_import_list_sha256: import_grant.authorized_import_list_sha256,
        captured_output_len: 0,
        captured_output_sha256: ZERO_SHA256,
        linked_host_import_count: import_grant.linked_host_import_count,
        module_imports_within_authorized_list: import_grant.module_imports_within_authorized_list,
        missing_import_module: import_grant.missing_import_module,
        missing_import_name: import_grant.missing_import_name,
    };
    let _ = (
        evidence.captured_output_len,
        evidence.captured_output_sha256,
    );
    emit_import_grant_marker(service_id, &evidence);
    evidence
}

fn metered_engine() -> Box<Engine> {
    let mut config = Config::default();
    config.consume_fuel(true);
    Box::new(Engine::new(&config))
}

fn default_state() -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: Vec::new(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new().build(),
    }
}

fn limited_state(memory_size: usize) -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: Vec::new(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new()
            .memory_size(memory_size)
            .instances(1)
            .memories(1)
            .tables(0)
            .build(),
    }
}

fn buffer_state(staged_input: &[u8]) -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: staged_input.to_vec(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new().build(),
    }
}

fn authorize_wasm_imports<'a>(
    service_id: &'a str,
    artifact_sha256_present: bool,
    requested_imports: &'a [(&'a str, &'a str)],
) -> Result<AuthorizedWasmImports<'a>, WasmImportGrantDecision> {
    let input = WasmImportGrantInput {
        service_id: Some(service_id),
        artifact_sha256_present,
        requested_imports,
        policy_allows_beyond_env: false,
    };
    let decision = evaluate_wasm_import_grant(&input);
    if !decision.performed {
        return Err(decision);
    }
    Ok(AuthorizedWasmImports {
        imports: requested_imports,
        decision,
        import_list_sha256: authorized_import_list_sha256(service_id, requested_imports),
    })
}

fn define_granted_imports(
    linker: &mut Linker<EnvelopeState>,
    authorized: &AuthorizedWasmImports<'_>,
) -> Result<u64, &'static str> {
    let mut linked = 0u64;
    let mut idx = 0usize;
    while idx < authorized.imports.len() {
        match authorized.imports[idx] {
            ("env", "log") => {
                linker
                    .func_wrap("env", "log", host_log)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "counter_get") => {
                linker
                    .func_wrap("env", "counter_get", host_counter_get)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "input_len") => {
                linker
                    .func_wrap("env", "input_len", host_input_len)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "input_read") => {
                linker
                    .func_wrap("env", "input_read", host_input_read)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "output_write") => {
                linker
                    .func_wrap("env", "output_write", host_output_write)
                    .map_err(|_| "host_import_link_failed")?;
            }
            _ => return Err("missing_host_import_implementation"),
        }
        linked += 1;
        idx += 1;
    }
    Ok(linked)
}

fn first_unauthorized_module_import(
    module: &Module,
    authorized: &AuthorizedWasmImports<'_>,
) -> Option<(String, String)> {
    for import in module.imports() {
        if !authorized_contains(authorized, import.module(), import.name()) {
            return Some((String::from(import.module()), String::from(import.name())));
        }
    }
    None
}

fn authorized_contains(authorized: &AuthorizedWasmImports<'_>, module: &str, name: &str) -> bool {
    authorized
        .imports
        .iter()
        .any(|(authorized_module, authorized_name)| {
            *authorized_module == module && *authorized_name == name
        })
}

fn import_grant_denied_evidence(decision: WasmImportGrantDecision) -> ImportGrantEvidence {
    ImportGrantEvidence {
        performed: decision.performed,
        status: decision.status,
        reason: decision.reason,
        authorized_import_count: decision.authorized_import_count as u64,
        authorized_import_list_sha256: ZERO_SHA256,
        linked_host_import_count: 0,
        module_imports_within_authorized_list: false,
        missing_import_module: None,
        missing_import_name: None,
    }
}

fn import_grant_evidence(
    authorized: &AuthorizedWasmImports<'_>,
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import: Option<(String, String)>,
) -> ImportGrantEvidence {
    let (missing_import_module, missing_import_name) = match missing_import {
        Some((module, name)) => (Some(module), Some(name)),
        None => (None, None),
    };
    ImportGrantEvidence {
        performed: authorized.decision.performed,
        status: authorized.decision.status,
        reason: authorized.decision.reason,
        authorized_import_count: authorized.decision.authorized_import_count as u64,
        authorized_import_list_sha256: authorized.import_list_sha256,
        linked_host_import_count,
        module_imports_within_authorized_list,
        missing_import_module,
        missing_import_name,
    }
}

fn emit_import_grant_marker(service_id: &str, evidence: &EchoRunEvidence) {
    let mut line = String::new();
    let _ = write!(
        &mut line,
        "WASM_IMPORT_GRANT {{\"service_id\":\"{}\",\"import_grant_performed\":{},\"import_grant_status\":\"{}\",\"import_grant_reason\":\"{}\",\"authorized_import_count\":{},\"authorized_import_list_sha256\":\"sha256:",
        service_id,
        if evidence.import_grant_performed { "true" } else { "false" },
        evidence.import_grant_status,
        evidence.import_grant_reason,
        evidence.authorized_import_count
    );
    push_sha256_hex(&mut line, evidence.authorized_import_list_sha256);
    let _ = write!(
        &mut line,
        "\",\"linked_host_import_count\":{},\"module_imports_within_authorized_list\":{},\"run_outcome\":\"{}\",\"missing_import_module\":",
        evidence.linked_host_import_count,
        if evidence.module_imports_within_authorized_list {
            "true"
        } else {
            "false"
        },
        evidence.run_outcome
    );
    push_json_string_or_null(&mut line, evidence.missing_import_module.as_deref());
    line.push_str(",\"missing_import_name\":");
    push_json_string_or_null(&mut line, evidence.missing_import_name.as_deref());
    line.push('}');
    serial::write_raw_line(&line);
}

fn push_json_string_or_null(out: &mut String, value: Option<&str>) {
    let Some(value) = value else {
        out.push_str("null");
        return;
    };
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn push_sha256_hex(out: &mut String, value: [u8; 32]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut idx = 0usize;
    while idx < value.len() {
        let byte = value[idx];
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
        idx += 1;
    }
}

fn host_log(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<(), Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.log negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_LOG_BYTES {
        return Err(Trap::new("env.log length exceeds 256 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.log pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.log memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;

    let mut line = String::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let byte = bytes[idx];
        line.push(if (0x20..=0x7e).contains(&byte) {
            byte as char
        } else {
            ' '
        });
        idx += 1;
    }

    serial::write_raw_str("WASM_GUEST_LOG ");
    serial::write_raw_line(&line);
    caller.data_mut().log_line = Some(line);
    Ok(())
}

fn host_input_len(mut caller: Caller<'_, EnvelopeState>) -> Result<i32, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let len = caller.data().staged_input.len();
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_len staged input exceeds max"));
    }
    Ok(len as i32)
}

fn host_input_read(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.input_read negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_read length exceeds 4096 bytes"));
    }
    let count = len.min(caller.data().staged_input.len());
    ptr.checked_add(count)
        .ok_or_else(|| Trap::new("env.input_read pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.input_read memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&caller.data().staged_input[..count]);
    memory
        .write(&mut caller, ptr, &bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    Ok(count as i32)
}

// Raw guest bytes: surfaced only as len+sha256 later. This env-module output
// channel is bounded + per-service-declared + subset-checked, and by design does
// NOT arm the policy_allows_beyond_env owner gate; move output_write to its own
// module if the owner later wants policy-gated output.
fn host_output_write(
    mut caller: Caller<'_, EnvelopeState>,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.output_write negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_OUTPUT_BYTES {
        return Err(Trap::new("env.output_write length exceeds 4096 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.output_write pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.output_write memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    caller
        .data()
        .captured_output
        .len()
        .checked_add(len)
        .filter(|total| *total <= MAX_WASM_OUTPUT_BYTES)
        .ok_or_else(|| Trap::new("env.output_write captured output exceeds 4096 bytes"))?;
    caller.data_mut().captured_output.extend_from_slice(&bytes);
    Ok(len as i32)
}

fn host_counter_get(mut caller: Caller<'_, EnvelopeState>) -> Result<i64, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let mut counter = CURRENT_BOOT_COUNTER.lock();
    *counter = counter.saturating_add(1);
    Ok((*counter).min(i64::MAX as u64) as i64)
}
