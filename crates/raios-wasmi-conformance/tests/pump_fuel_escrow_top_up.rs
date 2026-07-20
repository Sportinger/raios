use wasmi::{
    Config, Engine, ExecutionTrace, ExecutionTraceConfig, Linker, Module, Store, Suspension,
    TypedResumableCall,
};

const LEFTOVER: u64 = 100;
const FUEL_QUANTUM: u64 = 4_096;
const MAX_TOTAL_FUEL: u64 = FUEL_QUANTUM * 2;

#[derive(Debug)]
struct RunEvidence {
    grants: Vec<u64>,
    trace: ExecutionTrace,
    result: i32,
    stored: u8,
    final_escrow: u64,
}

fn fixture_wat() -> String {
    let charged_work = "i32.const 1\ndrop\n".repeat(256);
    format!(
        r#"
        (module
            (memory (export "memory") 1 1 shared)
            (func (export "run") (result i32)
                {charged_work}
                i32.const 1
                i32.const 42
                i32.store8
                i32.const 42))
        "#
    )
}

fn run_starvation_case() -> RunEvidence {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(fixture_wat()).expect("top-up fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("top-up fixture validates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("top-up fixture instantiates")
        .start(&mut store)
        .expect("top-up fixture starts");
    let run = instance
        .get_typed_func::<(), i32>(&store, "run")
        .expect("run export exists");
    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export exists");
    store.start_execution_tracing(ExecutionTraceConfig::new(0, 1, [4, 8]));

    store.set_execution_trace_round(0);
    let invocation = match run
        .call_resumable(&mut store, ())
        .expect("zero-fuel call parks before the charged block")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("zero-fuel fixture unexpectedly finished"),
    };
    assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
    let first_park = store.execution_trace().fuel_park_records()[0];
    assert!(
        first_park.debit() > LEFTOVER,
        "the swept leftover must be below the pending block debit"
    );
    assert!(
        first_park.debit() < FUEL_QUANTUM,
        "one topped-up activation must cover the pending block"
    );

    store.set_execution_trace_round(1);
    assert_eq!(store.replace_remaining_fuel(LEFTOVER).unwrap(), 0);
    let invocation = match invocation
        .resume(&mut store, &[])
        .expect("the insufficient leftover parks again")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("sub-debit leftover unexpectedly advanced"),
    };
    assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
    let swept_escrow = store
        .replace_remaining_fuel(0)
        .expect("the parked thread escrow sweeps back out");
    assert_eq!(swept_escrow, LEFTOVER, "park-before-charge spends no fuel");

    let mut granted_total = FUEL_QUANTUM;
    let remaining_under_max = MAX_TOTAL_FUEL
        .checked_sub(granted_total)
        .expect("the modeled pump accounting stays below its ceiling");
    let grant = (FUEL_QUANTUM - swept_escrow).min(remaining_under_max);
    granted_total = granted_total
        .checked_add(grant)
        .expect("the modeled grant total cannot overflow");
    let topped_up = swept_escrow
        .checked_add(grant)
        .expect("the modeled thread escrow cannot overflow");
    assert_eq!(topped_up, FUEL_QUANTUM);
    assert!(granted_total <= MAX_TOTAL_FUEL);

    store.set_execution_trace_round(2);
    assert_eq!(store.replace_remaining_fuel(topped_up).unwrap(), 0);
    let result = match invocation
        .resume(&mut store, &[])
        .expect("the topped-up activation resumes")
    {
        TypedResumableCall::Finished(result) => result,
        TypedResumableCall::Resumable(_) => panic!("a full quantum did not pass the block"),
    };
    let final_escrow = store
        .replace_remaining_fuel(0)
        .expect("the completed thread escrow sweeps back out");
    let trace = store.execution_trace();
    let parks = trace.fuel_park_records();
    assert_eq!(parks.len(), 2);
    assert_eq!(parks[0].debit(), parks[1].debit());
    assert_eq!(parks[0].bytecode_ip(), parks[1].bytecode_ip());
    assert_eq!(parks[1].round(), 1);
    let stores = trace.store8_records();
    assert_eq!(stores.len(), 1, "the guest store must execute exactly once");
    assert!(
        stores[0].bytecode_ip() > parks[1].bytecode_ip(),
        "the topped-up activation must advance beyond the parked block entry"
    );

    RunEvidence {
        grants: vec![grant],
        trace,
        result,
        stored: memory.data(&store)[1],
        final_escrow,
    }
}

#[test]
fn swept_sub_debit_escrow_is_topped_up_and_advances_deterministically() {
    let first = run_starvation_case();
    let second = run_starvation_case();

    assert_eq!(first.grants, second.grants, "grant schedules diverged");
    assert_eq!(first.trace, second.trace, "execution traces diverged");
    assert_eq!(first.result, second.result);
    assert_eq!(first.stored, second.stored);
    assert_eq!(first.final_escrow, second.final_escrow);
    assert_eq!(first.grants, vec![FUEL_QUANTUM - LEFTOVER]);
    assert_eq!(first.result, 42);
    assert_eq!(first.stored, 42);
}
