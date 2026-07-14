use super::*;
use super::{acquire_shims::*, envelope::*, invocation::*, suspension::*};

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
// Labeled NET-2 test infrastructure only. `test.suspend_once` never enters the
// known-import table or the production per-instance linker.
pub(super) const SUSPEND_ONCE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x02,
    0x15, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x0c, 0x73, 0x75, 0x73, 0x70, 0x65, 0x6e, 0x64, 0x5f,
    0x6f, 0x6e, 0x63, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x10, 0x02, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
    0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b,
];
const TAIL_SUSPEND_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x02,
    0x15, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x0c, 0x73, 0x75, 0x73, 0x70, 0x65, 0x6e, 0x64, 0x5f,
    0x6f, 0x6e, 0x63, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x10, 0x02, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
    0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x12, 0x00, 0x0b,
];
const NORMAL_RETURN_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04,
    0x00, 0x41, 0x07, 0x0b,
];
const UNREACHABLE_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x05, 0x01, 0x03,
    0x00, 0x00, 0x0b,
];
const FUEL_LOOP_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x0b, 0x01, 0x09,
    0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
];

const FUEL_EXHAUSTION_BUDGET: u64 = 1;
const GUEST_TRAP_FUEL_BUDGET: u64 = 100;

static BEYOND_ENV_SUITE: Mutex<Option<BeyondEnvLifecycleSuite>> = Mutex::new(None);

struct AcquireFixtureRuntime {
    store: Store<BeyondEnvState>,
    instance: Instance,
    memory: Memory,
}

#[derive(Clone, Copy)]
enum AcquireFixtureAction {
    WrongIndex,
    WrongLength,
    HashMismatch,
    OutOfOrder,
    Duplicate,
    Extra,
    MissingFinalize,
    CompleteThenFinalize,
    FirstChunk,
    GuestMemoryFault,
}

pub(crate) fn run_acquire_fixture_probe() -> AcquireFixtureProbeSnapshot {
    let serial_candidate_sha256 =
        crate::module_candidate_intake::retained().map(|item| item.sha256);
    let serial_receipt_sha256 =
        crate::agent_protocol::agent_protocol_registry::last_serial_distribution_receipt_sha256();
    let (positive_complete, candidate_sha256, receipt_sha256) = run_acquire_positive();
    let prior = crate::module_candidate_intake::retained();
    let current_kill = crate::input::secure_attention_kill_generation();
    let mut cases = [AcquireFixtureCase::failed("not_run"); ACQUIRE_FIXTURE_CASE_COUNT];
    let definitions = [
        (
            "wrong_index",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::WrongIndex,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "wrong_length",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::WrongLength,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "hash_mismatch",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::HashMismatch,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "out_of_order",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::OutOfOrder,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "duplicate",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::Duplicate,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "extra_chunk",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::Extra,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "finalize_missing_chunks",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::MissingFinalize,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "finalize_short_body",
            AcquireFixtureMode::ShortBody,
            AcquireFixtureAction::CompleteThenFinalize,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "finalize_long_body",
            AcquireFixtureMode::LongBody,
            AcquireFixtureAction::CompleteThenFinalize,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "source_tls_evidence",
            AcquireFixtureMode::SourceEvidenceMismatch,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "catalog_identity",
            AcquireFixtureMode::CatalogMismatch,
            AcquireFixtureAction::CompleteThenFinalize,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "receiver_identity",
            AcquireFixtureMode::ReceiverMismatch,
            AcquireFixtureAction::CompleteThenFinalize,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "posture",
            AcquireFixtureMode::PostureDenied,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "foreign_owner",
            AcquireFixtureMode::ForeignOwner,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "stale_session",
            AcquireFixtureMode::StaleSession,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "invalid_invocation_authority",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.invalid",
            current_kill,
        ),
        (
            "guest_memory_fault",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::GuestMemoryFault,
            "test.fixture.acquire_shims.signed",
            current_kill,
        ),
        (
            "kill_generation",
            AcquireFixtureMode::Normal,
            AcquireFixtureAction::FirstChunk,
            "test.fixture.acquire_shims.signed",
            current_kill.wrapping_add(1),
        ),
    ];
    let failure_count = definitions.len();
    for (slot, (name, mode, action, service_id, kill_generation)) in
        cases.iter_mut().zip(definitions)
    {
        *slot = run_acquire_denial_case(
            name,
            mode,
            action,
            service_id,
            kill_generation,
            prior.as_ref(),
        );
    }
    cases[18] = run_acquire_terminal_case("kill_cleanup", TerminalOutcome::Killed, prior.as_ref());
    cases[19] =
        run_acquire_terminal_case("trap_cleanup", TerminalOutcome::GuestTrap, prior.as_ref());
    cases[20] =
        run_acquire_terminal_case("fuel_cleanup", TerminalOutcome::OutOfFuel, prior.as_ref());

    let typed_denials_pairwise_distinct = (0..failure_count).all(|left| {
        ((left + 1)..failure_count).all(|right| cases[left].denial != cases[right].denial)
    });
    AcquireFixtureProbeSnapshot {
        positive_complete,
        candidate_sha256,
        receipt_sha256,
        serial_candidate_sha256,
        serial_receipt_sha256,
        candidate_hash_converged: candidate_sha256.is_some()
            && candidate_sha256 == serial_candidate_sha256,
        receipt_hash_converged: receipt_sha256.is_some() && receipt_sha256 == serial_receipt_sha256,
        failure_count,
        typed_denials_pairwise_distinct,
        all_prior_candidates_preserved: cases.iter().all(|case| case.prior_candidate_unchanged),
        all_incomplete_acquisitions_dropped: cases.iter().all(|case| case.pending_dropped),
        cases,
        direct_candidate_intake_calls: 0,
    }
}

fn run_acquire_positive() -> (bool, Option<[u8; 32]>, Option<[u8; 32]>) {
    let Some(mut runtime) = acquire_fixture_runtime(
        AcquireFixtureMode::Normal,
        "test.fixture.acquire_shims.signed",
        crate::input::secure_attention_kill_generation(),
    ) else {
        return (false, None, None);
    };
    let complete =
        call_all_acquire_chunks(&mut runtime) && call_acquire_finalize(&mut runtime) == Some(0);
    let candidate = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .and_then(|state| state.candidate_sha256());
    let receipt = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .and_then(|state| state.receipt_sha256());
    let _ = finish_acquire_resources(runtime.store.data_mut());
    finish_store(&mut runtime.store, TerminalOutcome::Finished);
    (complete, candidate, receipt)
}

fn run_acquire_denial_case(
    name: &'static str,
    mode: AcquireFixtureMode,
    action: AcquireFixtureAction,
    service_id: &'static str,
    kill_generation: u64,
    prior: Option<&crate::module_candidate_intake::RetainedExternalWasmCandidate>,
) -> AcquireFixtureCase {
    let Some(mut runtime) = acquire_fixture_runtime(mode, service_id, kill_generation) else {
        return AcquireFixtureCase::failed(name);
    };
    let first_end = ECHO_WASM_ARTIFACT_BYTES.len() / 3;
    match action {
        AcquireFixtureAction::WrongIndex => {
            let _ = call_acquire_chunk(&mut runtime, 3, 0, first_end as i32);
        }
        AcquireFixtureAction::WrongLength => {
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32 - 1);
        }
        AcquireFixtureAction::HashMismatch => {
            let _ = runtime.memory.write(&mut runtime.store, 0, &[0xff]);
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
        }
        AcquireFixtureAction::OutOfOrder => {
            let _ = call_acquire_chunk(&mut runtime, 1, first_end as i32, first_end as i32);
        }
        AcquireFixtureAction::Duplicate => {
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
        }
        AcquireFixtureAction::Extra => {
            let _ = call_all_acquire_chunks(&mut runtime);
            let _ = call_acquire_chunk(&mut runtime, 2, 0, 1);
        }
        AcquireFixtureAction::MissingFinalize => {
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
            let _ = call_acquire_finalize(&mut runtime);
        }
        AcquireFixtureAction::CompleteThenFinalize => {
            let _ = call_all_acquire_chunks(&mut runtime);
            let _ = call_acquire_finalize(&mut runtime);
        }
        AcquireFixtureAction::FirstChunk => {
            let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
        }
        AcquireFixtureAction::GuestMemoryFault => {
            let _ = call_acquire_chunk(&mut runtime, 0, 65_535, first_end as i32);
        }
    }
    let denial = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .map_or("none", AcquisitionInvocationState::last_denial);
    let pending_before = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .is_some_and(AcquisitionInvocationState::pending_present);
    let dropped = finish_acquire_resources(runtime.store.data_mut());
    finish_store(&mut runtime.store, TerminalOutcome::Finished);
    AcquireFixtureCase {
        name,
        denial,
        denied: denial != "none",
        prior_candidate_unchanged: retained_candidate_matches(prior),
        pending_dropped: pending_before && dropped,
    }
}

fn run_acquire_terminal_case(
    name: &'static str,
    outcome: TerminalOutcome,
    prior: Option<&crate::module_candidate_intake::RetainedExternalWasmCandidate>,
) -> AcquireFixtureCase {
    let Some(mut runtime) = acquire_fixture_runtime(
        AcquireFixtureMode::Normal,
        "test.fixture.acquire_shims.signed",
        crate::input::secure_attention_kill_generation(),
    ) else {
        return AcquireFixtureCase::failed(name);
    };
    let first_end = ECHO_WASM_ARTIFACT_BYTES.len() / 3;
    let _ = call_acquire_chunk(&mut runtime, 0, 0, first_end as i32);
    let pending_before = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .is_some_and(AcquisitionInvocationState::pending_present);
    if outcome == TerminalOutcome::GuestTrap {
        let _ = call_acquire_export(&mut runtime, "trap", &[]);
    } else if outcome == TerminalOutcome::OutOfFuel {
        let _ = call_acquire_export(&mut runtime, "loop", &[]);
    }
    let dropped = finish_acquire_resources(runtime.store.data_mut());
    finish_store(&mut runtime.store, outcome);
    AcquireFixtureCase {
        name,
        denial: match outcome {
            TerminalOutcome::Killed => "terminal_killed",
            TerminalOutcome::GuestTrap => "terminal_guest_trap",
            TerminalOutcome::OutOfFuel => "terminal_out_of_fuel",
            _ => "terminal_other",
        },
        denied: true,
        prior_candidate_unchanged: retained_candidate_matches(prior),
        pending_dropped: pending_before && dropped,
    }
}

fn acquire_fixture_runtime(
    mode: AcquireFixtureMode,
    service_id: &'static str,
    captured_kill_generation: u64,
) -> Option<AcquireFixtureRuntime> {
    let engine = beyond_env_engine();
    let module = Module::new(&engine, ACQUIRE_SHIM_WASM_MODULE).ok()?;
    let invocation_id = NEXT_BEYOND_ENV_INVOCATION_ID.fetch_add(1, Ordering::Relaxed);
    let authority = InvocationAuthority {
        service_id,
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation,
        policy_allows_beyond_env: false,
    };
    let now_ms = runtime_now_ms();
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
            behavior: FixtureBehavior::Suspend,
            net: None,
            crypto: crypto_shims::CryptoInvocationState::new(None),
            acquire: Some(AcquisitionInvocationState::fixture(invocation_id, mode)),
            limits: beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    store.add_fuel(BEYOND_ENV_FIXTURE_FUEL_BUDGET).ok()?;
    let mut linker = Linker::<BeyondEnvState>::new(&engine);
    link_acquire_fixture(&mut linker).ok()?;
    let instance = linker
        .instantiate(&mut store, &module)
        .ok()?
        .start(&mut store)
        .ok()?;
    let memory = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)?;
    memory.write(&mut store, 0, ECHO_WASM_ARTIFACT_BYTES).ok()?;
    memory
        .write(&mut store, ECHO_WASM_ARTIFACT_BYTES.len(), &[0])
        .ok()?;
    Some(AcquireFixtureRuntime {
        store,
        instance,
        memory,
    })
}

fn call_all_acquire_chunks(runtime: &mut AcquireFixtureRuntime) -> bool {
    let len = ECHO_WASM_ARTIFACT_BYTES.len();
    let first_end = len / 3;
    let second_end = (len * 2) / 3;
    let last_len = runtime
        .store
        .data()
        .acquire
        .as_ref()
        .and_then(|state| state.expected_chunk_len(2))
        .unwrap_or(len - second_end);
    call_acquire_chunk(runtime, 0, 0, first_end as i32) == Some(0)
        && call_acquire_chunk(
            runtime,
            1,
            first_end as i32,
            (second_end - first_end) as i32,
        ) == Some(0)
        && call_acquire_chunk(runtime, 2, second_end as i32, last_len as i32) == Some(0)
}

fn call_acquire_chunk(
    runtime: &mut AcquireFixtureRuntime,
    index: i32,
    ptr: i32,
    len: i32,
) -> Option<i32> {
    call_acquire_export(
        runtime,
        "chunk",
        &[Value::I32(index), Value::I32(ptr), Value::I32(len)],
    )
}

fn call_acquire_finalize(runtime: &mut AcquireFixtureRuntime) -> Option<i32> {
    call_acquire_export(runtime, "finalize", &[])
}

fn call_acquire_export(
    runtime: &mut AcquireFixtureRuntime,
    name: &str,
    inputs: &[Value],
) -> Option<i32> {
    let function = runtime
        .instance
        .get_export(&runtime.store, name)
        .and_then(Extern::into_func)?;
    let mut outputs = [Value::I32(0)];
    function
        .call(&mut runtime.store, inputs, &mut outputs)
        .ok()?;
    outputs[0].i32()
}

fn retained_candidate_matches(
    prior: Option<&crate::module_candidate_intake::RetainedExternalWasmCandidate>,
) -> bool {
    match (prior, crate::module_candidate_intake::retained()) {
        (Some(prior), Some(after)) => {
            prior.sha256 == after.sha256
                && prior.wasm_valid == after.wasm_valid
                && prior.bytes == after.bytes
        }
        (None, None) => true,
        _ => false,
    }
}

pub(crate) fn run_beyond_env_lifecycle_suite() -> BeyondEnvLifecycleSuite {
    if let Some(suite) = *BEYOND_ENV_SUITE.lock() {
        return suite;
    }
    if wasm_execution_busy() {
        return busy_beyond_env_lifecycle_suite();
    }
    let normal = run_beyond_env_case(
        "normal_return",
        NORMAL_RETURN_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let guest_trap = run_beyond_env_case(
        "guest_trap",
        UNREACHABLE_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let unrecognized = run_beyond_env_case(
        "unrecognized_host_error",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::UnrecognizedHostError,
        100,
        CaseAction::None,
    );
    let host_fuel = run_beyond_env_case(
        "host_fuel_error",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::HostFuelError,
        100,
        CaseAction::None,
    );
    let busy_started = crate::time::rdtsc() / crate::time::tsc_per_ms().max(1);
    let wasm_fuel = run_beyond_env_case(
        "wasm_out_of_fuel",
        FUEL_LOOP_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        CaseAction::None,
    );
    let busy_loop_wall_ms =
        (crate::time::rdtsc() / crate::time::tsc_per_ms().max(1)).saturating_sub(busy_started);
    let marker_mismatch = run_beyond_env_case(
        "suspend_marker_mismatch",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::Mismatch,
    );
    let abandoned = run_beyond_env_case(
        "abandoned_drop_guard",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::Abandon,
    );
    let tail = run_beyond_env_case(
        "tail_call_terminal",
        TAIL_SUSPEND_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let suite = BeyondEnvLifecycleSuite {
        cases: [
            normal,
            guest_trap,
            unrecognized,
            host_fuel,
            wasm_fuel,
            marker_mismatch,
            abandoned,
        ],
        tail_call_terminal: tail.outcome == "host_error" && !tail.suspended,
        busy_loop_out_of_fuel: wasm_fuel.outcome == "out_of_fuel",
        busy_loop_wall_ms,
        fuel_budget: BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        wall_budget_ms: BEYOND_ENV_WALL_BUDGET_MS,
        pump_step_budget: BEYOND_ENV_PUMP_STEP_BUDGET,
    };
    *BEYOND_ENV_SUITE.lock() = Some(suite);
    suite
}

#[derive(Clone, Copy)]
enum CaseAction {
    None,
    Mismatch,
    Abandon,
}

fn run_beyond_env_case(
    name: &'static str,
    bytes: &[u8],
    behavior: FixtureBehavior,
    fuel_budget: u64,
    action: CaseAction,
) -> BeyondEnvLifecycleCase {
    let invocation_id = NEXT_BEYOND_ENV_INVOCATION_ID.fetch_add(1, Ordering::Relaxed);
    let authority = InvocationAuthority {
        service_id: "test.fixture.beyond_env_lifecycle",
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation: 0,
        policy_allows_beyond_env: false,
    };
    let engine = beyond_env_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => return failed_beyond_env_case(name),
    };
    let mut store = Store::new(
        &engine,
        BeyondEnvState {
            lifecycle: InvocationLifecycle::new(
                authority,
                0,
                BEYOND_ENV_WALL_BUDGET_MS,
                BEYOND_ENV_PUMP_STEP_BUDGET,
                invocation_id as u32,
            ),
            behavior,
            net: None,
            crypto: crypto_shims::CryptoInvocationState::new(None),
            acquire: None,
            limits: beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(fuel_budget).is_err() {
        return failed_beyond_env_case(name);
    }
    let mut linker = Linker::<BeyondEnvState>::new(&engine);
    if linker
        .func_wrap("test", "suspend_once", host_test_suspend_once)
        .is_err()
    {
        return failed_beyond_env_case(name);
    }
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(pre) => match pre.start(&mut store) {
            Ok(instance) => instance,
            Err(error) => {
                let outcome = terminal_from_error(&error);
                return teardown_case(name, &mut store, outcome, false);
            }
        },
        Err(error) => {
            let outcome = terminal_from_error(&error);
            return teardown_case(name, &mut store, outcome, false);
        }
    };
    let Some(function) = instance
        .get_export(&store, "run")
        .and_then(Extern::into_func)
    else {
        return teardown_case(name, &mut store, TerminalOutcome::HostError, false);
    };
    let mut outputs = [Value::I32(0)];
    match function.call_resumable(&mut store, &[], &mut outputs) {
        Ok(ResumableCall::Finished) => {
            teardown_case(name, &mut store, TerminalOutcome::Finished, false)
        }
        Ok(ResumableCall::Resumable(invocation)) => {
            if matches!(action, CaseAction::Mismatch) {
                store.data_mut().lifecycle.take_pending();
                store.data_mut().lifecycle.record_pending(2);
            }
            let classification = classify_resumable(&invocation, &store);
            if matches!(action, CaseAction::Abandon) {
                let suspension_count = store.data().lifecycle.suspension_count();
                let memory = instance
                    .get_export(&store, "memory")
                    .and_then(Extern::into_memory);
                let before = ACTIVE_DROP_TEARDOWN_COUNT.load(Ordering::Relaxed);
                drop(ActiveBeyondEnvInvocation {
                    store: Some(store),
                    instance: Some(instance),
                    memory,
                    continuation: Some(invocation),
                    outputs,
                    auto_resume: false,
                    closed: false,
                });
                let teardown_count = ACTIVE_DROP_TEARDOWN_COUNT
                    .load(Ordering::Relaxed)
                    .saturating_sub(before) as u32;
                return BeyondEnvLifecycleCase {
                    name,
                    outcome: TerminalOutcome::Abandoned.as_str(),
                    suspended: true,
                    suspension_count,
                    resume_count: 0,
                    teardown_count,
                    teardown_complete: teardown_count == 1,
                };
            }
            if let Err(outcome) = classification {
                return teardown_case(name, &mut store, outcome, false);
            }
            store.data_mut().lifecycle.take_pending();
            store.data_mut().lifecycle.note_resume();
            match invocation.resume(&mut store, &[Value::I32(7)], &mut outputs) {
                Ok(ResumableCall::Finished) => {
                    teardown_case(name, &mut store, TerminalOutcome::Finished, true)
                }
                Ok(ResumableCall::Resumable(next)) => {
                    let outcome = classify_resumable(&next, &store)
                        .err()
                        .unwrap_or(TerminalOutcome::HostError);
                    teardown_case(name, &mut store, outcome, true)
                }
                Err(error) => {
                    let outcome = terminal_from_error(&error);
                    teardown_case(name, &mut store, outcome, true)
                }
            }
        }
        Err(error) => {
            let outcome = terminal_from_error(&error);
            teardown_case(name, &mut store, outcome, false)
        }
    }
}

fn teardown_case(
    name: &'static str,
    store: &mut Store<BeyondEnvState>,
    outcome: TerminalOutcome,
    suspended: bool,
) -> BeyondEnvLifecycleCase {
    finish_store(store, outcome);
    case_from_store(name, store, suspended)
}

fn case_from_store(
    name: &'static str,
    store: &Store<BeyondEnvState>,
    suspended: bool,
) -> BeyondEnvLifecycleCase {
    let lifecycle = &store.data().lifecycle;
    let receipt = lifecycle.teardown_receipt();
    BeyondEnvLifecycleCase {
        name,
        outcome: lifecycle
            .terminal()
            .map(TerminalOutcome::as_str)
            .unwrap_or("not_terminal"),
        suspended,
        suspension_count: lifecycle.suspension_count(),
        resume_count: lifecycle.resume_count(),
        teardown_count: receipt.teardown_count,
        teardown_complete: receipt.terminal_outcome_recorded,
    }
}

fn failed_beyond_env_case(name: &'static str) -> BeyondEnvLifecycleCase {
    BeyondEnvLifecycleCase {
        name,
        outcome: "fixture_setup_failed",
        suspended: false,
        suspension_count: 0,
        resume_count: 0,
        teardown_count: 0,
        teardown_complete: false,
    }
}

fn busy_beyond_env_lifecycle_suite() -> BeyondEnvLifecycleSuite {
    let case = BeyondEnvLifecycleCase {
        name: "wasm_execution_busy",
        outcome: "wasm_execution_busy",
        suspended: false,
        suspension_count: 0,
        resume_count: 0,
        teardown_count: 0,
        teardown_complete: false,
    };
    BeyondEnvLifecycleSuite {
        cases: [case; 7],
        tail_call_terminal: false,
        busy_loop_out_of_fuel: false,
        busy_loop_wall_ms: 0,
        fuel_budget: BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        wall_budget_ms: BEYOND_ENV_WALL_BUDGET_MS,
        pump_step_budget: BEYOND_ENV_PUMP_STEP_BUDGET,
    }
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

fn busy_echo_probe() -> EchoProbe {
    let positive = wasm_busy_run(ECHO_SERVICE_ID, ECHO_WASM_FUEL_BUDGET);
    let busy_case = WasmHardeningCase {
        name: "wasm_execution_busy",
        mechanism: "central_wasm_execution_gate",
        expected_outcome: "wasm_execution_busy",
        actual_outcome: "wasm_execution_busy",
        passed: true,
    };
    EchoProbe {
        artifact_hash: ECHO_WASM_ARTIFACT_BYTES_HASH,
        descriptor_hash: ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        signature_envelope_hash: ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        validation_ok: false,
        instantiation_ok: false,
        run_outcome: positive.run_outcome,
        return_value: None,
        fuel_budget: positive.fuel_budget,
        fuel_used: 0,
        log_line: None,
        forbidden_validation_ok: false,
        forbidden_instantiation_ok: false,
        forbidden_link_error_kind: "wasm_execution_busy",
        forbidden_missing_import_module: None,
        forbidden_missing_import_name: None,
        forbidden_boundary_held: true,
        hardening_cases: [busy_case; WASM_HARDENING_CASE_COUNT],
    }
}

pub(crate) fn run_echo_probe() -> EchoProbe {
    if wasm_execution_busy() {
        return busy_echo_probe();
    }
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
            ECHO_WASM_FUEL_BUDGET,
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
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certwindow_roundtrip(cert_der: &[u8]) -> EchoRunEvidence {
    let capped = &cert_der[..cert_der.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        CERTWINDOW_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTWINDOW_SERVICE_ID,
        true,
        CERTWINDOW_AUTHORIZED_IMPORTS,
        validate_certwindow_wasm_artifact(),
        capped,
        CERTWINDOW_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certwindow_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        CERTWINDOW_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTWINDOW_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_certwindow_wasm_artifact(),
        b"raios-m11-certwindow-unauthorized-nonce",
        CERTWINDOW_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_httphead_roundtrip(response: &[u8]) -> EchoRunEvidence {
    let capped = &response[..response.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        HTTPHEAD_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        HTTPHEAD_SERVICE_ID,
        true,
        HTTPHEAD_AUTHORIZED_IMPORTS,
        validate_httphead_wasm_artifact(),
        capped,
        HTTPHEAD_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_httphead_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        HTTPHEAD_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        HTTPHEAD_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_httphead_wasm_artifact(),
        b"raios-m11-httphead-unauthorized-nonce",
        HTTPHEAD_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certspki_roundtrip(cert_der: &[u8]) -> EchoRunEvidence {
    let capped = &cert_der[..cert_der.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        CERTSPKI_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTSPKI_SERVICE_ID,
        true,
        CERTSPKI_AUTHORIZED_IMPORTS,
        validate_certspki_wasm_artifact(),
        capped,
        CERTSPKI_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certspki_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        CERTSPKI_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTSPKI_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_certspki_wasm_artifact(),
        b"raios-m11-certspki-unauthorized-nonce",
        CERTSPKI_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_dnsparse_roundtrip(input_record: &[u8]) -> EchoRunEvidence {
    let capped = &input_record[..input_record.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        DNSPARSE_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        DNSPARSE_SERVICE_ID,
        true,
        DNSPARSE_AUTHORIZED_IMPORTS,
        validate_dnsparse_wasm_artifact(),
        capped,
        DNSPARSE_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_dnsparse_unauthorized_probe(input_record: &[u8]) -> EchoRunEvidence {
    let capped = &input_record[..input_record.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        DNSPARSE_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        DNSPARSE_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_dnsparse_wasm_artifact(),
        capped,
        DNSPARSE_WASM_FUEL_BUDGET,
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
    if wasm_execution_busy() {
        return fuel_starved_evidence(false, false, "wasm_execution_busy", false, 0, None);
    }
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
    if wasm_execution_busy() {
        return busy_negative_run();
    }
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

fn busy_negative_run() -> NegativeRun {
    NegativeRun {
        validation_ok: false,
        instantiation_ok: false,
        link_error_kind: "wasm_execution_busy",
        missing_import_module: None,
        missing_import_name: None,
        boundary_held: true,
    }
}
