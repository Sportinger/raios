use super::*;

use raios_core::host_import_abi_v1::HOST_IMPORT_ERROR_CAPABILITY_DENIED;

use super::grant_table::HostImportId;

const SELFTEST_SERVICE_ID: &str = "test.fixture.revoke_selftest";
const CALL_COUNTER_EXPORT: &str = "call_counter";
const CALL_LOG_EXPORT: &str = "call_log";

// (module
//   (import "env" "counter_get" (func $counter_get (result i64)))
//   (import "env" "log" (func $log (param i32 i32)))
//   (memory (export "memory") 1)
//   (data (i32.const 0) "rv")
//   (func (export "call_counter") (result i64) call $counter_get)
//   (func (export "call_log") i32.const 0 i32.const 2 call $log))
// Verified entrypoints are call_counter and call_log; the fixture exports both
// with those exact names plus the memory export required by env.log.
const REVOKE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0d, 0x03, 0x60, 0x00, 0x01, 0x7e, 0x60,
    0x02, 0x7f, 0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x1d, 0x02, 0x03, 0x65, 0x6e, 0x76, 0x0b, 0x63,
    0x6f, 0x75, 0x6e, 0x74, 0x65, 0x72, 0x5f, 0x67, 0x65, 0x74, 0x00, 0x00, 0x03, 0x65, 0x6e, 0x76,
    0x03, 0x6c, 0x6f, 0x67, 0x00, 0x01, 0x03, 0x03, 0x02, 0x00, 0x02, 0x05, 0x03, 0x01, 0x00, 0x01,
    0x07, 0x24, 0x03, 0x0c, 0x63, 0x61, 0x6c, 0x6c, 0x5f, 0x63, 0x6f, 0x75, 0x6e, 0x74, 0x65, 0x72,
    0x00, 0x02, 0x08, 0x63, 0x61, 0x6c, 0x6c, 0x5f, 0x6c, 0x6f, 0x67, 0x00, 0x03, 0x06, 0x6d, 0x65,
    0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x0f, 0x02, 0x04, 0x00, 0x10, 0x00, 0x0b, 0x08, 0x00,
    0x41, 0x00, 0x41, 0x02, 0x10, 0x01, 0x0b, 0x0b, 0x08, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x02, 0x72,
    0x76,
];

struct RevokeEvidence {
    first_call_ok: bool,
    revoked: bool,
    next_call_denied: bool,
    host_effect_delta: u64,
    peer_surface_ok: bool,
    same_instance: bool,
    logged: bool,
}

fn fixture_contract(module: &Module) -> bool {
    let mut imports = module.imports();
    let imports_match = imports
        .next()
        .is_some_and(|import| import.module() == "env" && import.name() == "counter_get")
        && imports
            .next()
            .is_some_and(|import| import.module() == "env" && import.name() == "log")
        && imports.next().is_none();
    let mut counter = false;
    let mut log = false;
    let mut memory = false;
    for export in module.exports() {
        match export.name() {
            CALL_COUNTER_EXPORT => counter = true,
            CALL_LOG_EXPORT => log = true,
            "memory" => memory = true,
            _ => {}
        }
    }
    imports_match && counter && log && memory
}

fn run_revoke_selftest() -> Result<RevokeEvidence, &'static str> {
    const IMPORTS: &[(&str, &str)] = &[("env", "counter_get"), ("env", "log")];

    let engine = envelope::metered_engine();
    let module = Module::new(&engine, REVOKE_WASM).map_err(|_| "fixture_decode")?;
    if !fixture_contract(&module) {
        return Err("fixture_exports");
    }
    let authorized = envelope::authorize_wasm_imports(SELFTEST_SERVICE_ID, true, IMPORTS)
        .map_err(|_| "grant_authorization")?;
    let mut store = Store::new(
        &engine,
        envelope::limited_state(WORKSPACE_MEMORY_LIMIT_BYTES),
    );
    store.limiter(|state| &mut state.limits);
    if !store.data_mut().grant_import(HostImportId::EnvCounterGet) {
        return Err("grant_counter_get");
    }
    if !store.data_mut().grant_import(HostImportId::EnvLog) {
        return Err("grant_log");
    }
    if store.add_fuel(WORKSPACE_FUEL_BUDGET).is_err() {
        return Err("fuel_metering");
    }
    let mut linker = Linker::<envelope::EnvelopeState>::new(&engine);
    if envelope::define_granted_imports(&mut linker, &authorized) != Ok(2) {
        return Err("host_import_link");
    }
    let pre = linker
        .instantiate(&mut store, &module)
        .map_err(|_| "instantiate")?;
    let instance = pre.start(&mut store).map_err(|_| "start")?;
    let call_counter = instance
        .get_export(&store, CALL_COUNTER_EXPORT)
        .and_then(Extern::into_func)
        .ok_or("call_counter_export")?;
    let call_log = instance
        .get_export(&store, CALL_LOG_EXPORT)
        .and_then(Extern::into_func)
        .ok_or("call_log_export")?;

    let c0 = envelope::current_boot_counter();
    let mut first_output = [Value::I64(0)];
    call_counter
        .call(&mut store, &[], &mut first_output)
        .map_err(|_| "first_call_trap")?;
    let c1 = envelope::current_boot_counter();
    let first_call_ok = c0.checked_add(1) == Some(c1) && first_output[0].i64() == Some(c1 as i64);
    if !first_call_ok {
        return Err("first_call_effect");
    }

    let generation = store.data().instance_generation();
    let revoked = store.data_mut().revoke_import(HostImportId::EnvCounterGet)
        && !store.data().import_is_live(HostImportId::EnvCounterGet)
        && store.data().instance_generation() == generation;
    if !revoked {
        return Err("revoke_counter_get");
    }
    if !store.data().import_is_live(HostImportId::EnvLog) {
        return Err("peer_surface_revoked");
    }

    let mut denied_output = [Value::I64(0)];
    call_counter
        .call(&mut store, &[], &mut denied_output)
        .map_err(|_| "next_call_trap")?;
    let after_denied = envelope::current_boot_counter();
    let next_call_denied =
        denied_output[0].i64() == Some(HOST_IMPORT_ERROR_CAPABILITY_DENIED as i64);
    let host_effect_delta = after_denied.saturating_sub(c1);
    if !next_call_denied {
        return Err("next_call_reason");
    }
    if host_effect_delta != 0 {
        return Err("revoked_host_effect");
    }

    let mut no_outputs: [Value; 0] = [];
    call_log
        .call(&mut store, &[], &mut no_outputs)
        .map_err(|_| "peer_surface_trap")?;
    let peer_surface_ok = store.data().log_line.as_deref() == Some("rv");
    if !peer_surface_ok {
        return Err("peer_surface_effect");
    }
    let same_instance = store.data().instance_generation() == generation;
    if !same_instance {
        return Err("same_instance");
    }
    let logged = store.data().log_line.is_some();
    if !logged {
        return Err("peer_surface_logged");
    }

    Ok(RevokeEvidence {
        first_call_ok,
        revoked,
        next_call_denied,
        host_effect_delta,
        peer_surface_ok,
        same_instance,
        logged,
    })
}

pub(crate) fn emit_revoke_selftest() {
    match run_revoke_selftest() {
        Ok(evidence) => {
            let line = alloc::format!(
                "RAIOS_REVOKE selftest={} surface=env.counter_get first_call={} revoked={} next_call={} host_effect_delta={} peer_surface={} same_instance={} logged={}",
                "pass",
                if evidence.first_call_ok { "ok" } else { "failed" },
                u8::from(evidence.revoked),
                if evidence.next_call_denied { "denied" } else { "failed" },
                evidence.host_effect_delta,
                if evidence.peer_surface_ok { "ok" } else { "failed" },
                u8::from(evidence.same_instance),
                u8::from(evidence.logged),
            );
            serial::write_raw_line(&line);
        }
        Err(reason) => {
            let line = alloc::format!("RAIOS_REVOKE selftest=fail rv_reason={reason}");
            serial::write_raw_line(&line);
        }
    }
}
