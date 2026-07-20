use wasmi::{
    core::TrapCode, AtomicSuspendRequest, Config, Engine, Linker, Module, ResumableCall, Store,
    Suspension, Value,
};

const ATOMIC_START: &str = r#"
    (module
        (memory (export "memory") 1 1 shared)
        (func $init
            i32.const 0
            i32.const 0
            i64.const -1
            memory.atomic.wait32
            drop

            i32.const 4
            i32.const 1
            i32.atomic.store

            i32.const 0
            i32.const 1
            memory.atomic.notify
            drop

            i32.const 8
            i32.const 1
            i32.atomic.store)
        (start $init))
"#;

fn atomic_start_module() -> (Engine, Module) {
    let mut config = Config::default();
    config.wasm_threads(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(ATOMIC_START).expect("atomic start fixture compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("atomic start fixture translates");
    (engine, module)
}

#[test]
fn split_start_runs_to_completion_through_resumable_atomic_driver() {
    let (engine, module) = atomic_start_module();
    let mut store = Store::new(&engine, ());
    let pre = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("atomic start fixture instantiates");
    let (instance, start) = pre
        .start_split(&mut store)
        .expect("split initialization succeeds");
    let start = start.expect("fixture has a start section");
    let mut outputs = [];
    let mut outcome = start
        .call_resumable(&mut store, &[], &mut outputs)
        .expect("start section enters the resumable driver");
    let mut saw_wait = false;
    let mut saw_notify = false;

    loop {
        let invocation = match outcome {
            ResumableCall::Finished => break,
            ResumableCall::Resumable(invocation) => invocation,
        };
        let scheduler_result = match invocation.suspension() {
            Suspension::Atomic(atomic) => match atomic.request {
                AtomicSuspendRequest::Wait { timeout_ns: -1 } => {
                    saw_wait = true;
                    0
                }
                AtomicSuspendRequest::Notify { count: 1 } => {
                    saw_notify = true;
                    0
                }
                request => panic!("unexpected atomic request: {request:?}"),
            },
            suspension => panic!("unexpected start suspension: {suspension:?}"),
        };
        outcome = invocation
            .resume(&mut store, &[Value::I32(scheduler_result)], &mut outputs)
            .expect("scheduler result resumes the start section");
    }

    assert!(saw_wait, "matching wait reached the resumable driver");
    assert!(saw_notify, "notify reached the resumable driver");
    let memory = instance
        .get_memory(&store, "memory")
        .expect("fixture exports memory");
    let mut markers = [0; 8];
    memory
        .read(&store, 4, &mut markers)
        .expect("completion markers are readable");
    assert_eq!(i32::from_le_bytes(markers[..4].try_into().unwrap()), 1);
    assert_eq!(i32::from_le_bytes(markers[4..].try_into().unwrap()), 1);
}

#[test]
fn frozen_start_path_still_rejects_atomic_suspension() {
    let (engine, module) = atomic_start_module();
    let mut store = Store::new(&engine, ());
    let pre = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("atomic start fixture instantiates");
    let error = pre
        .start(&mut store)
        .expect_err("non-resumable start must reject the matching atomic wait");

    assert!(matches!(
        error,
        wasmi::Error::Trap(trap)
            if matches!(trap.trap_code(), Some(TrapCode::AtomicSuspendNotResumable))
    ));
}
