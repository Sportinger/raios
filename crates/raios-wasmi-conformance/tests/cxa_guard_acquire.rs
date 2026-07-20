use wasmi::{
    AtomicSuspendRequest, Config, Engine, Linker, Memory, MemoryType, Module, Store, Suspension,
    TypedResumableCall, Value,
};

const GUARD: usize = 0;
const PREFIX_COUNT: usize = 4;
const PENDING: i32 = 2;
const DONE: i32 = 1;
const FUEL_QUANTUM: u64 = 20;

const GUARD_PATTERN: &str = r#"
    (module
        (import "env" "memory" (memory 1 1 shared))
        (func (export "_start")
            (local $old i32)
            (local $iteration i32)

            i32.const 0
            i32.const 0
            i32.const 2
            i32.atomic.rmw.cmpxchg
            local.tee $old
            i32.eqz
            if
                ;; This prefix must execute exactly once even though the fuel
                ;; quantum expires before the initializer publishes DONE.
                i32.const 4
                i32.const 1
                i32.atomic.rmw.add
                drop
                (loop $burn
                    local.get $iteration
                    i32.const 1
                    i32.add
                    local.tee $iteration
                    i32.const 32
                    i32.lt_u
                    br_if $burn)

                i32.const 0
                i32.const 1
                i32.atomic.store
                i32.const 0
                i32.const 1
                memory.atomic.notify
                drop
                return
            end

            local.get $old
            i32.const 1
            i32.eq
            if
                return
            end

            local.get $old
            i32.const 2
            i32.ne
            if
                unreachable
            end

            (block $done
                (loop $wait
                    i32.const 0
                    i32.const 2
                    i64.const -1
                    memory.atomic.wait32
                    drop
                    i32.const 0
                    i32.atomic.load
                    i32.const 1
                    i32.eq
                    br_if $done
                    br $wait)))
    )
"#;

#[derive(Debug, PartialEq, Eq)]
struct GuardRun {
    fuel_trace: Vec<(i32, i32)>,
    notifications: usize,
    waits: usize,
    guard: i32,
    prefix_count: i32,
}

fn read_i32(store: &Store<()>, memory: Memory, offset: usize) -> i32 {
    let mut bytes = [0; 4];
    memory
        .read(store, offset, &mut bytes)
        .expect("guard fixture read is in bounds");
    i32::from_le_bytes(bytes)
}

fn run_guard_pattern() -> GuardRun {
    let mut config = Config::default();
    config
        .wasm_threads(true)
        .consume_fuel(true)
        .resumable_fuel(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(GUARD_PATTERN).expect("guard-pattern WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("guard-pattern module validates");
    let mut store = Store::new(&engine, ());
    let memory = Memory::new(
        &mut store,
        MemoryType::new_shared(1, 1).expect("one-page shared memory type is valid"),
    )
    .expect("shared guard memory allocates");
    assert_eq!(read_i32(&store, memory, GUARD), 0);
    assert_eq!(read_i32(&store, memory, PREFIX_COUNT), 0);

    let mut linker = Linker::new(&engine);
    linker
        .define("env", "memory", memory)
        .expect("guard memory import is unique");
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("guard-pattern module instantiates")
        .start(&mut store)
        .expect("guard-pattern module has no start section");
    let start = instance
        .get_typed_func::<(), ()>(&store, "_start")
        .expect("guard-pattern _start export exists");
    store
        .add_fuel(FUEL_QUANTUM)
        .expect("initial fuel quantum installs");

    let mut fuel_trace = Vec::new();
    let mut notifications = 0;
    let mut outcome = start
        .call_resumable(&mut store, ())
        .expect("guard-pattern call enters the resumable driver");
    for _ in 0..128 {
        let invocation = match outcome {
            TypedResumableCall::Finished(()) => {
                return GuardRun {
                    fuel_trace,
                    notifications,
                    waits: 0,
                    guard: read_i32(&store, memory, GUARD),
                    prefix_count: read_i32(&store, memory, PREFIX_COUNT),
                };
            }
            TypedResumableCall::Resumable(invocation) => invocation,
        };
        match invocation.suspension() {
            Suspension::FuelQuantum => {
                fuel_trace.push((
                    read_i32(&store, memory, GUARD),
                    read_i32(&store, memory, PREFIX_COUNT),
                ));
                store
                    .add_fuel(FUEL_QUANTUM)
                    .expect("next fuel quantum installs");
                outcome = invocation
                    .resume(&mut store, &[])
                    .expect("fuel suspension resumes the same guard activation");
            }
            Suspension::Atomic(atomic) => match atomic.request {
                AtomicSuspendRequest::Notify { count: 1 } => {
                    notifications += 1;
                    outcome = invocation
                        .resume(&mut store, &[Value::I32(0)])
                        .expect("scheduler resolves guard notification");
                }
                AtomicSuspendRequest::Wait { .. } => {
                    return GuardRun {
                        fuel_trace,
                        notifications,
                        waits: 1,
                        guard: read_i32(&store, memory, GUARD),
                        prefix_count: read_i32(&store, memory, PREFIX_COUNT),
                    };
                }
                request => panic!("unexpected guard atomic request: {request:?}"),
            },
            suspension => panic!("unexpected guard suspension: {suspension:?}"),
        }
    }
    panic!("guard-pattern fixture did not complete within its suspension bound")
}

#[test]
fn lone_guard_acquire_wins_and_resumes_without_waiting() {
    let first = run_guard_pattern();
    let second = run_guard_pattern();

    assert_eq!(
        first, second,
        "identical fuel schedules must be trace-equal"
    );
    assert_eq!(first.waits, 0, "a lone thread must not enter the slow path");
    assert!(
        first
            .fuel_trace
            .iter()
            .any(|&(guard, prefix_count)| guard == PENDING && prefix_count == 1),
        "the fixture must suspend after acquiring the guard and before publishing DONE"
    );
    assert_eq!(first.notifications, 1);
    assert_eq!(first.guard, DONE);
    assert_eq!(first.prefix_count, 1);
}
