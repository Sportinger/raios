use wasmi::{
    core::{Trap, TrapCode},
    Config, Engine, Instance, Linker, Memory, Module, Store, Suspension, TypedResumableCall, Value,
};

const TERMINAL_GUEST: &str = r#"
    (module
        (func (export "run") (result i32)
            i32.const 7))
"#;

const COUNTER_GUEST: &str = r#"
    (module
        (memory (export "memory") 1)
        (func (export "run") (param $limit i32) (result i32)
            (local $iteration i32)
            (loop $count
                i32.const 0
                i32.const 0
                i32.load
                i32.const 1
                i32.add
                i32.store

                local.get $iteration
                i32.const 1
                i32.add
                local.tee $iteration
                local.get $limit
                i32.lt_s
                br_if $count)
            i32.const 0
            i32.load))
"#;

const WAIT_GUEST: &str = r#"
    (module
        (memory 1 1 shared)
        (func (export "wait") (result i32)
            i32.const 0
            i32.const 0
            i64.const -1
            memory.atomic.wait32))
"#;

const COUNTER_LIMIT: i32 = 6;
const FUEL_QUANTUM: u64 = 20;

fn engine_with_fuel(resumable_fuel: bool) -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true).resumable_fuel(resumable_fuel);
    Engine::new(&config)
}

fn instantiate(engine: &Engine, wat: &str) -> (Store<()>, Instance) {
    let wasm = wat::parse_str(wat).expect("test WAT compiles");
    let module = Module::new(engine, wasm.as_slice()).expect("test module validates");
    let mut store = Store::new(engine, ());
    let instance = Linker::new(engine)
        .instantiate(&mut store, &module)
        .expect("test module instantiates")
        .start(&mut store)
        .expect("test module starts");
    (store, instance)
}

fn exported_memory(store: &Store<()>, instance: Instance) -> Memory {
    instance
        .get_memory(store, "memory")
        .expect("memory export exists")
}

fn read_counter(store: &Store<()>, memory: Memory) -> i32 {
    let mut bytes = [0_u8; 4];
    memory
        .read(store, 0, &mut bytes)
        .expect("counter read is in bounds");
    i32::from_le_bytes(bytes)
}

fn terminal_error(explicit_off: bool, resumable_call: bool) -> Trap {
    let mut config = Config::default();
    config.consume_fuel(true);
    if explicit_off {
        config.resumable_fuel(false);
    }
    let engine = Engine::new(&config);
    let (mut store, instance) = instantiate(&engine, TERMINAL_GUEST);
    let run = instance
        .get_typed_func::<(), i32>(&store, "run")
        .expect("run export exists");
    if resumable_call {
        run.call_resumable(&mut store, ())
            .expect_err("default fuel exhaustion must be terminal")
    } else {
        run.call(&mut store, ())
            .expect_err("default fuel exhaustion must trap")
    }
}

fn run_counter_trace() -> (Vec<i32>, i32, i32) {
    let engine = engine_with_fuel(true);
    let (mut store, instance) = instantiate(&engine, COUNTER_GUEST);
    let memory = exported_memory(&store, instance);
    let run = instance
        .get_typed_func::<i32, i32>(&store, "run")
        .expect("run export exists");
    store
        .add_fuel(FUEL_QUANTUM)
        .expect("initial fuel can be added");

    let mut trace = Vec::new();
    let mut outcome = run
        .call_resumable(&mut store, COUNTER_LIMIT)
        .expect("counter call must start");
    for _ in 0..32 {
        match outcome {
            TypedResumableCall::Finished(result) => {
                return (trace, result, read_counter(&store, memory));
            }
            TypedResumableCall::Resumable(invocation) => {
                assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
                assert!(invocation.host_func().is_none());
                assert!(invocation.host_error().is_none());
                trace.push(read_counter(&store, memory));
                store
                    .add_fuel(FUEL_QUANTUM)
                    .expect("fuel quantum can be refilled");
                outcome = invocation
                    .resume(&mut store, &[])
                    .expect("fuel quantum resumes without values");
            }
        }
    }
    panic!("counter did not finish within the suspension bound")
}

#[test]
fn resumable_fuel_is_disabled_by_default_for_call_and_call_resumable() {
    for resumable_call in [false, true] {
        let default_error = terminal_error(false, resumable_call);
        let explicit_off_error = terminal_error(true, resumable_call);
        assert!(matches!(
            default_error.trap_code(),
            Some(TrapCode::OutOfFuel)
        ));
        assert!(matches!(
            explicit_off_error.trap_code(),
            Some(TrapCode::OutOfFuel)
        ));
        assert_eq!(
            default_error.to_string().as_bytes(),
            explicit_off_error.to_string().as_bytes(),
            "explicit false must preserve the terminal trap bytes"
        );
        assert_eq!(
            format!("{default_error:?}").as_bytes(),
            format!("{explicit_off_error:?}").as_bytes(),
            "explicit false must preserve the terminal debug bytes"
        );
    }
}

#[test]
fn fuel_quantum_resumes_at_the_same_instruction_and_is_deterministic() {
    let first = run_counter_trace();
    let second = run_counter_trace();

    assert_eq!(first, second, "identical fuel schedules must match exactly");
    assert_eq!(first.0, vec![1, 2, 4, 5]);
    assert_eq!(first.0.len(), 4, "suspension count is part of the trace");
    assert_eq!(first.1, COUNTER_LIMIT);
    assert_eq!(first.2, COUNTER_LIMIT);
}

#[test]
fn failed_fuel_charge_is_not_consumed_and_resume_requires_no_values() {
    let engine = engine_with_fuel(true);
    let (mut store, instance) = instantiate(&engine, TERMINAL_GUEST);
    let run = instance
        .get_typed_func::<(), i32>(&store, "run")
        .expect("run export exists");
    let invocation = match run
        .call_resumable(&mut store, ())
        .expect("fuel exhaustion must suspend")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("zero fuel must not finish"),
    };
    assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
    assert_eq!(store.fuel_consumed(), Some(0));

    let invocation = match invocation
        .resume(&mut store, &[])
        .expect("empty fuel resume must suspend again")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("refill is required to finish"),
    };
    assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
    assert_eq!(store.fuel_consumed(), Some(0));

    store.add_fuel(100).expect("fuel can be refilled");
    assert!(matches!(
        invocation
            .resume(&mut store, &[])
            .expect("refilled invocation must resume"),
        TypedResumableCall::Finished(7)
    ));
}

#[test]
fn fuel_and_atomic_suspensions_keep_their_resume_types_separate() {
    let fuel_engine = engine_with_fuel(true);
    let (mut fuel_store, fuel_instance) = instantiate(&fuel_engine, TERMINAL_GUEST);
    let fuel_run = fuel_instance
        .get_typed_func::<(), i32>(&fuel_store, "run")
        .expect("run export exists");
    let fuel_invocation = match fuel_run
        .call_resumable(&mut fuel_store, ())
        .expect("fuel exhaustion must suspend")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("zero fuel must suspend"),
    };
    assert!(matches!(
        fuel_invocation.suspension(),
        Suspension::FuelQuantum
    ));
    fuel_invocation
        .resume(&mut fuel_store, &[Value::I32(0)])
        .expect_err("fuel quantum must reject an atomic scheduler result");

    let mut atomic_config = Config::default();
    atomic_config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    let atomic_engine = Engine::new(&atomic_config);
    let (mut atomic_store, atomic_instance) = instantiate(&atomic_engine, WAIT_GUEST);
    let wait = atomic_instance
        .get_typed_func::<(), i32>(&atomic_store, "wait")
        .expect("wait export exists");
    atomic_store
        .add_fuel(1_000)
        .expect("wait guest can be fueled");
    let atomic_invocation = match wait
        .call_resumable(&mut atomic_store, ())
        .expect("matching wait must suspend")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("matching wait must park"),
    };
    assert!(matches!(
        atomic_invocation.suspension(),
        Suspension::Atomic(_)
    ));
    atomic_invocation
        .resume(&mut atomic_store, &[])
        .expect_err("atomic park must reject a fuel-quantum response");
}

#[test]
fn non_resumable_call_stays_terminal_when_resumable_fuel_is_enabled() {
    let engine = engine_with_fuel(true);
    let (mut store, instance) = instantiate(&engine, TERMINAL_GUEST);
    let run = instance
        .get_typed_func::<(), i32>(&store, "run")
        .expect("run export exists");
    let error = run
        .call(&mut store, ())
        .expect_err("non-resumable call must not yield or hang");
    assert!(matches!(error.trap_code(), Some(TrapCode::OutOfFuel)));
}
