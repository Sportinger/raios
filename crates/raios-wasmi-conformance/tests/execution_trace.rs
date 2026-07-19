use wasmi::{
    Config, Engine, ExecutionTrace, ExecutionTraceConfig, Linker, Module, Store, Suspension,
    TypedResumableCall,
};

const ATOMIC_FIXTURE: &str = r#"
    (module
        (memory (export "memory") 1 1 shared)
        (func (export "run") (result i32)
            i32.const 1
            i32.const 1
            i32.store8
            i32.const 1
            i64.const 2
            i64.store8
            i32.const 1
            i32.const 3
            i32.atomic.store8
            i32.const 1
            i64.const 4
            i64.atomic.store8
            i32.const 2
            i32.const 5
            i32.store8
            i32.const 4
            i32.const 0
            i32.const 42
            i32.atomic.rmw.cmpxchg
            drop
            i32.const 4
            i32.const 0
            i32.const 99
            i32.atomic.rmw.cmpxchg))
"#;

const PARK_FIXTURE: &str = r#"
    (module
        (memory 1 1 shared)
        (func (export "run")
            (loop $again
                br $again)))
"#;

#[derive(Debug)]
struct RunEvidence {
    result: i32,
    memory: Vec<u8>,
    fuel_consumed: u64,
    trace: ExecutionTrace,
}

fn run_fixture(traced: bool) -> RunEvidence {
    let mut config = Config::default();
    config.wasm_threads(true).consume_fuel(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(ATOMIC_FIXTURE).expect("atomic fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("atomic fixture validates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("atomic fixture instantiates")
        .start(&mut store)
        .expect("atomic fixture starts");
    let run = instance
        .get_typed_func::<(), i32>(&store, "run")
        .expect("run export exists");
    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export exists");
    if traced {
        store.start_execution_tracing(ExecutionTraceConfig::new(0, 1, [4, 8]));
        store.set_execution_trace_round(17);
    }
    store.add_fuel(10_000).expect("fixture fuel installs");
    let result = run.call(&mut store, ()).expect("atomic fixture runs");
    RunEvidence {
        result,
        memory: memory.data(&store)[..12].to_vec(),
        fuel_consumed: store
            .fuel_consumed()
            .expect("fixture fuel metering remains enabled"),
        trace: store.execution_trace(),
    }
}

#[test]
fn execution_trace_is_opt_in_and_preserves_atomic_execution() {
    let control = run_fixture(false);
    let traced = run_fixture(true);

    assert_eq!(control.result, 42);
    assert_eq!(traced.result, control.result, "tracing changed the result");
    assert_eq!(traced.memory, control.memory, "tracing changed memory");
    assert_eq!(
        traced.fuel_consumed, control.fuel_consumed,
        "tracing changed fuel accounting"
    );
    assert_eq!(control.trace, ExecutionTrace::empty());

    let stores = traced.trace.store8_records();
    assert_eq!(stores.len(), 4, "only stores to the watched byte are kept");
    assert_eq!(
        stores
            .iter()
            .map(|record| record.value())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert!(
        stores
            .windows(2)
            .all(|pair| pair[0].bytecode_ip() < pair[1].bytecode_ip()),
        "store instruction indices must advance"
    );

    let cmpxchg = traced.trace.cmpxchg_records();
    assert_eq!(cmpxchg.len(), 2);
    assert_eq!(cmpxchg[0].round(), 17);
    assert_eq!(cmpxchg[0].function_index(), 0);
    assert_eq!(cmpxchg[0].effective_address(), 4);
    assert_eq!(cmpxchg[0].watch_byte(), 4);
    assert_eq!(cmpxchg[0].before(), 0);
    assert_eq!(cmpxchg[0].expected(), 0);
    assert_eq!(cmpxchg[0].replacement(), 42);
    assert_eq!(cmpxchg[0].returned_old(), 0);
    assert_eq!(cmpxchg[0].after(), 42);

    assert!(cmpxchg[1].bytecode_ip() > cmpxchg[0].bytecode_ip());
    assert_eq!(cmpxchg[1].before(), 42);
    assert_eq!(cmpxchg[1].expected(), 0);
    assert_eq!(cmpxchg[1].replacement(), 99);
    assert_eq!(cmpxchg[1].returned_old(), 42);
    assert_eq!(cmpxchg[1].after(), 42, "failed CAS must not store");
    assert!(traced.trace.fuel_park_records().is_empty());
    assert!(!traced.trace.capped());
}

#[test]
fn execution_trace_records_failed_debit_without_advancing_the_parked_ip() {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(PARK_FIXTURE).expect("park fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("park fixture validates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("park fixture instantiates")
        .start(&mut store)
        .expect("park fixture starts");
    let run = instance
        .get_typed_func::<(), ()>(&store, "run")
        .expect("run export exists");
    store.start_execution_tracing(ExecutionTraceConfig::new(0, 1, [4, 8]));
    store.set_execution_trace_round(9);

    let first = run
        .call_resumable(&mut store, ())
        .expect("zero-fuel call suspends");
    let first = match first {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(()) => panic!("infinite fixture unexpectedly finished"),
    };
    assert!(matches!(first.suspension(), Suspension::FuelQuantum));
    let first_park = store.execution_trace().fuel_park_records()[0];
    assert_eq!(first_park.round(), 9);
    assert!(first_park.debit() > 0);
    assert_eq!(first_park.watch_words(), [0, 0]);
    assert_eq!(first_park.watch_byte(), 0);

    store.set_execution_trace_round(10);
    store
        .add_fuel(first_park.debit().saturating_sub(1))
        .expect("insufficient retry fuel installs");
    let second = first
        .resume(&mut store, &[])
        .expect("insufficient retry suspends");
    let second = match second {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(()) => panic!("infinite fixture unexpectedly finished"),
    };
    assert!(matches!(second.suspension(), Suspension::FuelQuantum));
    let trace = store.execution_trace();
    let parks = trace.fuel_park_records();
    assert_eq!(parks.len(), 2);
    assert_eq!(parks[1].round(), 10);
    assert_eq!(parks[1].debit(), first_park.debit());
    assert_eq!(
        parks[1].bytecode_ip(),
        first_park.bytecode_ip(),
        "park-before-charge must preserve the failed instruction"
    );
}
