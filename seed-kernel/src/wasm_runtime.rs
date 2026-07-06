use alloc::{boxed::Box, string::String, vec::Vec};
use raios_core::sha256_bytes;
use spin::Mutex;
use wasmi::{
    core::{Trap, TrapCode},
    errors::LinkerError,
    Caller, Config, Engine, Extern, Linker, Module, Store, Value,
};

use crate::serial;

include!(concat!(env!("OUT_DIR"), "/echo_wasm_artifact.rs"));

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";
const FORBIDDEN_WRITE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x02, 0x17,
    0x01, 0x03, 0x65, 0x6e, 0x76, 0x0f, 0x66, 0x6f, 0x72, 0x62, 0x69, 0x64, 0x64, 0x65, 0x6e, 0x5f,
    0x77, 0x72, 0x69, 0x74, 0x65, 0x00, 0x00,
];
const MAX_WASM_LOG_BYTES: usize = 256;
pub(crate) const ECHO_WASM_FUEL_BUDGET: u64 = 10_000;
pub(crate) const FORBIDDEN_IMPORT_MODULE: &str = "env";
pub(crate) const FORBIDDEN_IMPORT_NAME: &str = "forbidden_write";

static CURRENT_BOOT_COUNTER: Mutex<u64> = Mutex::new(0);

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;
#[used]
static ECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_echo_wasm_artifact;

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
}

pub(crate) fn run_echo_probe() -> EchoProbe {
    let validation_ok = validate_echo_wasm_artifact();
    let positive = execute_echo_module(validation_ok);
    let negative = instantiate_forbidden_import_module();

    EchoProbe {
        artifact_hash: ECHO_WASM_ARTIFACT_BYTES_HASH,
        descriptor_hash: ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        signature_envelope_hash: ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        validation_ok,
        instantiation_ok: positive.instantiation_ok,
        run_outcome: positive.run_outcome,
        return_value: positive.return_value,
        fuel_budget: ECHO_WASM_FUEL_BUDGET,
        fuel_used: positive.fuel_used,
        log_line: positive.log_line,
        forbidden_validation_ok: negative.validation_ok,
        forbidden_instantiation_ok: negative.instantiation_ok,
        forbidden_link_error_kind: negative.link_error_kind,
        forbidden_missing_import_module: negative.missing_import_module,
        forbidden_missing_import_name: negative.missing_import_name,
        forbidden_boundary_held: negative.boundary_held,
    }
}

fn validate_module_bytes(bytes: &[u8]) -> bool {
    let engine = Box::new(wasmi::Engine::default());
    wasmi::Module::new(&engine, bytes).is_ok()
}

struct EnvelopeState {
    log_line: Option<String>,
}

struct PositiveRun {
    instantiation_ok: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    log_line: Option<String>,
}

struct NegativeRun {
    validation_ok: bool,
    instantiation_ok: bool,
    link_error_kind: &'static str,
    missing_import_module: Option<&'static str>,
    missing_import_name: Option<&'static str>,
    boundary_held: bool,
}

fn execute_echo_module(validation_ok: bool) -> PositiveRun {
    if !validation_ok {
        return positive_run(false, "validation_failed", None, 0, None);
    }

    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => return positive_run(false, "module_compile_failed", None, 0, None),
    };
    let mut store = Box::new(Store::new(&engine, EnvelopeState { log_line: None }));
    if store.add_fuel(ECHO_WASM_FUEL_BUDGET).is_err() {
        return positive_run(false, "fuel_metering_unavailable", None, 0, None);
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    if !define_capability_envelope(&mut linker) {
        return positive_run(
            false,
            "capability_envelope_definition_failed",
            None,
            0,
            None,
        );
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(_) => {
                return positive_run(
                    false,
                    "instantiation_start_trap",
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                )
            }
        },
        Err(_) => {
            return positive_run(
                false,
                "instantiation_failed",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, "raios_service_main")
        .and_then(Extern::into_func)
    else {
        return positive_run(
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
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
            positive_run(
                true,
                outcome,
                return_value,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
        Err(_) => positive_run(
            true,
            "trap",
            None,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
        ),
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
    let mut store = Box::new(Store::new(&engine, EnvelopeState { log_line: None }));
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    if !define_capability_envelope(&mut linker) {
        return NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "capability_envelope_definition_failed",
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

fn positive_run(
    instantiation_ok: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    log_line: Option<String>,
) -> PositiveRun {
    PositiveRun {
        instantiation_ok,
        run_outcome,
        return_value,
        fuel_used,
        log_line,
    }
}

fn metered_engine() -> Box<Engine> {
    let mut config = Config::default();
    config.consume_fuel(true);
    Box::new(Engine::new(&config))
}

fn define_capability_envelope(linker: &mut Linker<EnvelopeState>) -> bool {
    linker.func_wrap("env", "log", host_log).is_ok()
        && linker
            .func_wrap("env", "counter_get", host_counter_get)
            .is_ok()
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

fn host_counter_get(mut caller: Caller<'_, EnvelopeState>) -> Result<i64, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let mut counter = CURRENT_BOOT_COUNTER.lock();
    *counter = counter.saturating_add(1);
    Ok((*counter).min(i64::MAX as u64) as i64)
}
