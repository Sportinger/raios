use super::*;

use raios_core::host_import_abi_v1::HOST_IMPORT_ERROR_CAPABILITY_DENIED;
use raios_core::wasm_import_grant_event::{GrantEvent, HostImportId as DurableHostImportId};

use super::grant_table::HostImportId;

const SELFTEST_SERVICE_ID: &str = "test.fixture.revoke_selftest";
const CALL_COUNTER_EXPORT: &str = "call_counter";
const CALL_LOG_EXPORT: &str = "call_log";
const DURABLE_DOMAIN_INSTANCE: u64 = 1;

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
    durable_appended: bool,
    replayed: bool,
    invalid_projection: bool,
    call_attempted: bool,
    projection_sha256: [u8; 32],
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
    let boot_projection =
        crate::agent_protocol::durable_store::load_durable_wasm_grant_projection();
    let durable_available = boot_projection.valid;
    let invalid_projection = !durable_available
        && matches!(
            boot_projection.reason,
            "reclog_full_region_invalid"
                | "grant_event_malformed"
                | "grant_fold_malformed"
                | "grant_fold_missing_parent"
                | "grant_fold_malformed_link"
                | "grant_fold_fork"
                | "grant_fold_epoch_non_monotonic"
                | "grant_fold_ambiguous_history"
                | "grant_fold_capacity_overflow"
        );
    let (boot_valid, projection_sha256, boot_event_count) =
        super::grant_table::boot_projection_evidence();
    let binding_sha256 = raios_core::sha256_bytes(REVOKE_WASM);
    let projected_state = super::grant_table::durable_import_state(
        SELFTEST_SERVICE_ID,
        DURABLE_DOMAIN_INSTANCE,
        binding_sha256,
        HostImportId::EnvCounterGet,
    );
    let replayed = boot_valid
        && boot_event_count != 0
        && projected_state == super::grant_table::DurableImportState::Revoked;

    let state = if durable_available || invalid_projection {
        envelope::limited_state_for_durable_domain(
            WORKSPACE_MEMORY_LIMIT_BYTES,
            SELFTEST_SERVICE_ID,
            DURABLE_DOMAIN_INSTANCE,
            binding_sha256,
        )
    } else {
        // A deliberately absent persistence device keeps the pre-Slice-3 quick
        // fixture behavior. Malformed durable history never reaches this arm.
        envelope::limited_state(WORKSPACE_MEMORY_LIMIT_BYTES)
    };
    let mut store = Store::new(&engine, state);
    store.limiter(|state| &mut state.limits);
    if !store.data_mut().grant_import(HostImportId::EnvLog) {
        return Err("grant_log");
    }

    let grant_epoch = boot_projection.next_epoch;
    let grant_id = alloc::format!("cap.grant.revoke_selftest.{grant_epoch}");
    let grant = GrantEvent::grant(
        &grant_id,
        SELFTEST_SERVICE_ID,
        DURABLE_DOMAIN_INSTANCE,
        binding_sha256,
        DurableHostImportId::EnvCounterGet,
        1,
        grant_epoch,
    );
    let durable_appended = if invalid_projection || !durable_available {
        false
    } else if replayed {
        true
    } else {
        crate::agent_protocol::durable_store::append_durable_wasm_grant_event(&grant)
    };
    if durable_available && !durable_appended {
        return Err("durable_grant_append");
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
    let denied_on_entry = replayed || invalid_projection;
    let first_call_ok = if denied_on_entry {
        c0 == c1 && first_output[0].i64() == Some(HOST_IMPORT_ERROR_CAPABILITY_DENIED as i64)
    } else {
        c0.checked_add(1) == Some(c1) && first_output[0].i64() == Some(c1 as i64)
    };
    if !first_call_ok {
        return Err("first_call_effect");
    }

    let generation = store.data().instance_generation();
    let revoke_epoch = grant_epoch.checked_add(1).ok_or("revoke_epoch_overflow")?;
    let revoke_id = alloc::format!("cap.revoke.revoke_selftest.{revoke_epoch}");
    let durable_revoke = if !durable_available || denied_on_entry {
        true
    } else {
        let grant_hash = grant.record_sha256().map_err(|_| "grant_hash")?;
        let revoke = GrantEvent::revoke(&revoke_id, &grant, revoke_epoch, grant_hash);
        crate::agent_protocol::durable_store::append_durable_wasm_grant_event(&revoke)
    };
    if !durable_revoke {
        return Err("durable_revoke_append");
    }
    // Ordering is load-bearing: durable revoke first, then the RAM slot flip.
    let revoked = (denied_on_entry || store.data_mut().revoke_import(HostImportId::EnvCounterGet))
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

    let final_projection =
        crate::agent_protocol::durable_store::load_durable_wasm_grant_projection();
    if durable_available
        && !invalid_projection
        && (!final_projection.valid
            || !final_projection.slots.iter().any(|slot| {
                slot.service_id == SELFTEST_SERVICE_ID
                    && slot.domain_instance == DURABLE_DOMAIN_INSTANCE
                    && slot.host_import_id == DurableHostImportId::EnvCounterGet
                    && slot.revoked
            }))
    {
        return Err("durable_projection_not_revoked");
    }

    Ok(RevokeEvidence {
        first_call_ok,
        revoked,
        next_call_denied,
        host_effect_delta,
        peer_surface_ok,
        same_instance,
        logged,
        durable_appended,
        replayed,
        invalid_projection,
        call_attempted: true,
        projection_sha256: if durable_available {
            final_projection.sha256
        } else {
            projection_sha256
        },
    })
}

pub(crate) fn emit_revoke_selftest() {
    let counter_before = envelope::current_boot_counter();
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
            let projection_hex = raios_core::sha256_hex(&evidence.projection_sha256);
            let projection = core::str::from_utf8(&projection_hex).unwrap_or("invalid");
            serial::write_fmt(format_args!(
                "RAIOS_REVOKE_DURABLE selftest=pass replay={} invalid_projection={} append_before_flip={} projection=sha256:{} call_attempted={} gate=env.counter_get denied_next={} host_effect_delta={}\r\n",
                u8::from(evidence.replayed),
                u8::from(evidence.invalid_projection),
                u8::from(evidence.durable_appended),
                projection,
                u8::from(evidence.call_attempted),
                u8::from(evidence.next_call_denied),
                evidence.host_effect_delta,
            ));
        }
        Err(reason) => {
            let counter_after = envelope::current_boot_counter();
            let line = alloc::format!(
                "RAIOS_REVOKE selftest=fail rv_reason={reason} host_effect_delta={}",
                counter_after.saturating_sub(counter_before)
            );
            serial::write_raw_line(&line);
        }
    }
}
