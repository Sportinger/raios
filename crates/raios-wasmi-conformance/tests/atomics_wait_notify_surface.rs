use wasmi::{
    core::TrapCode, errors::ModuleError, AtomicSuspendRequest, Config, Engine, Error, Instance,
    Linker, Memory, Module, Store, Suspension, TypedResumableCall, Value,
};

const WAIT_NOTIFY_SURFACE: &str = r#"
    (module
        (memory (export "memory") 1 1 shared)
        (func (export "notify") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            memory.atomic.notify)
        (func (export "wait32") (param i32 i32 i64) (result i32)
            local.get 0
            local.get 1
            local.get 2
            memory.atomic.wait32)
        (func (export "wait64") (param i32 i64 i64) (result i32)
            local.get 0
            local.get 1
            local.get 2
            memory.atomic.wait64)
        (func (export "wait32_store") (param i32 i32 i64) (result i32)
            (local i32)
            local.get 0
            local.get 1
            local.get 2
            memory.atomic.wait32 offset=8
            local.set 3
            i32.const 0
            local.get 3
            i32.store
            local.get 3))
"#;

fn engine_with_threads() -> Engine {
    let mut config = Config::default();
    config.wasm_threads(true);
    Engine::new(&config)
}

fn module_from_wat(engine: &Engine, wat: &str) -> Result<Module, Error> {
    let wasm = wat::parse_str(wat).expect("test WAT compiles");
    Module::new(engine, wasm.as_slice())
}

fn instantiate_surface() -> (Store<()>, Instance) {
    instantiate(WAIT_NOTIFY_SURFACE)
}

fn instantiate_unshared_surface() -> (Store<()>, Instance) {
    let wat = WAIT_NOTIFY_SURFACE.replace("1 1 shared", "1 1");
    instantiate(&wat)
}

fn instantiate(wat: &str) -> (Store<()>, Instance) {
    let engine = engine_with_threads();
    let module =
        module_from_wat(&engine, wat).expect("wait/notify surface validates and translates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("wait/notify module instantiates")
        .start(&mut store)
        .expect("wait/notify module starts");
    (store, instance)
}

fn exported_memory(store: &Store<()>, instance: Instance) -> Memory {
    instance
        .get_memory(store, "memory")
        .expect("memory export exists")
}

fn write_i32(store: &mut Store<()>, memory: Memory, offset: usize, value: i32) {
    memory
        .write(store, offset, &value.to_le_bytes())
        .expect("i32 write is in bounds");
}

fn read_i32(store: &Store<()>, memory: Memory, offset: usize) -> i32 {
    let mut bytes = [0_u8; 4];
    memory
        .read(store, offset, &mut bytes)
        .expect("i32 read is in bounds");
    i32::from_le_bytes(bytes)
}

fn assert_validation_error(wat: &str) {
    let engine = engine_with_threads();
    let error = module_from_wat(&engine, wat).expect_err("invalid operands must not validate");
    assert!(
        matches!(&error, Error::Module(ModuleError::Translation(_))),
        "unexpected error: {error:?}"
    );
}

#[test]
fn wait_notify_surface_translates_with_threads_enabled() {
    let engine = engine_with_threads();
    module_from_wat(&engine, WAIT_NOTIFY_SURFACE)
        .expect("all wait/notify instructions must translate");
}

#[test]
fn wait_notify_surface_is_rejected_with_threads_disabled() {
    let engine = Engine::default();
    let unshared_surface = WAIT_NOTIFY_SURFACE.replace("1 1 shared", "1 1");
    let error = module_from_wat(&engine, &unshared_surface)
        .expect_err("thread instructions require the threads feature");
    assert!(
        matches!(&error, Error::Module(ModuleError::Translation(_))),
        "unexpected error: {error:?}"
    );
    assert!(
        error.to_string().contains("threads support is not enabled"),
        "unexpected validation error: {error}"
    );
}

#[test]
fn notify_rejects_an_i64_count() {
    assert_validation_error(
        r#"
        (module
            (memory 1 1 shared)
            (func (param i32 i64) (result i32)
                local.get 0
                local.get 1
                memory.atomic.notify))
        "#,
    );
}

#[test]
fn wait32_rejects_an_i32_timeout() {
    assert_validation_error(
        r#"
        (module
            (memory 1 1 shared)
            (func (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                memory.atomic.wait32))
        "#,
    );
}

#[test]
fn wait64_rejects_an_i32_expected_value() {
    assert_validation_error(
        r#"
        (module
            (memory 1 1 shared)
            (func (param i32 i32 i64) (result i32)
                local.get 0
                local.get 1
                local.get 2
                memory.atomic.wait64))
        "#,
    );
}

#[test]
fn wait32_unequal_shared_value_returns_one_without_suspending() {
    let (mut store, instance) = instantiate_surface();
    let memory = exported_memory(&store, instance);
    write_i32(&mut store, memory, 0, 7);
    let wait32 = instance
        .get_typed_func::<(i32, i32, i64), i32>(&store, "wait32")
        .expect("wait32 export exists");

    let result = wait32
        .call(&mut store, (0, 5, -1))
        .expect("unequal wait must stay on the inline fast path");

    assert_eq!(result, 1);
}

#[test]
fn wait_trap_order_is_alignment_then_bounds_then_sharedness() {
    let (mut shared_store, shared_instance) = instantiate_surface();
    let wait32 = shared_instance
        .get_typed_func::<(i32, i32, i64), i32>(&shared_store, "wait32")
        .expect("wait32 export exists");

    let unaligned = wait32
        .call(&mut shared_store, (1, 5, -1))
        .expect_err("misaligned wait must trap before comparing");
    assert!(matches!(
        unaligned.trap_code(),
        Some(TrapCode::UnalignedAtomic)
    ));

    let out_of_bounds = wait32
        .call(&mut shared_store, (65_536, 5, -1))
        .expect_err("out-of-bounds wait must trap before sharedness or comparison");
    assert!(matches!(
        out_of_bounds.trap_code(),
        Some(TrapCode::MemoryOutOfBounds)
    ));

    let (mut zero_store, zero_instance) = instantiate_surface();
    let zero_notify = zero_instance
        .get_typed_func::<(i32, i32), i32>(&zero_store, "notify")
        .expect("notify export exists");
    let zero_invocation = match zero_notify
        .call_resumable(&mut zero_store, (0, 0))
        .expect("shared count-zero notify must use the scheduler path")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("shared count-zero notify must suspend"),
    };
    assert!(matches!(
        zero_invocation.suspension(),
        Suspension::Atomic(suspend)
            if matches!(suspend.request, AtomicSuspendRequest::Notify { count: 0 })
    ));
    assert!(matches!(
        zero_invocation
            .resume(&mut zero_store, &[Value::I32(0)])
            .expect("scheduler must resolve count-zero notify"),
        TypedResumableCall::Finished(0)
    ));

    let (mut unshared_store, unshared_instance) = instantiate_unshared_surface();
    let unshared_memory = exported_memory(&unshared_store, unshared_instance);
    write_i32(&mut unshared_store, unshared_memory, 0, 7);
    let unshared_wait32 = unshared_instance
        .get_typed_func::<(i32, i32, i64), i32>(&unshared_store, "wait32")
        .expect("wait32 export exists");
    let unshared = unshared_wait32
        .call(&mut unshared_store, (0, 5, -1))
        .expect_err("unshared wait must trap even when the value is unequal");
    assert!(matches!(
        unshared.trap_code(),
        Some(TrapCode::UnsharedMemoryAtomicWait)
    ));
}

#[test]
fn notify_checks_alignment_and_bounds_at_count_zero_and_unshared_returns_zero() {
    let (mut shared_store, shared_instance) = instantiate_surface();
    let notify = shared_instance
        .get_typed_func::<(i32, i32), i32>(&shared_store, "notify")
        .expect("notify export exists");

    let unaligned = notify
        .call(&mut shared_store, (1, 0))
        .expect_err("count zero does not skip notify alignment checks");
    assert!(matches!(
        unaligned.trap_code(),
        Some(TrapCode::UnalignedAtomic)
    ));

    let out_of_bounds = notify
        .call(&mut shared_store, (65_536, 0))
        .expect_err("count zero does not skip notify bounds checks");
    assert!(matches!(
        out_of_bounds.trap_code(),
        Some(TrapCode::MemoryOutOfBounds)
    ));

    let (mut unshared_store, unshared_instance) = instantiate_unshared_surface();
    let unshared_notify = unshared_instance
        .get_typed_func::<(i32, i32), i32>(&unshared_store, "notify")
        .expect("notify export exists");
    let result = unshared_notify
        .call(&mut unshared_store, (0, 0))
        .expect("unshared notify must return inline");
    assert_eq!(result, 0);
}

#[test]
fn matching_root_wait_parks_with_payload_and_resumes_with_both_results() {
    for scheduler_result in [0, 2] {
        let (mut store, instance) = instantiate_surface();
        let memory = exported_memory(&store, instance);
        write_i32(&mut store, memory, 12, 7);
        let wait32_store = instance
            .get_typed_func::<(i32, i32, i64), i32>(&store, "wait32_store")
            .expect("wait32_store export exists");
        let invocation = match wait32_store
            .call_resumable(&mut store, (4, 7, -1))
            .expect("matching wait must be resumable")
        {
            TypedResumableCall::Resumable(invocation) => invocation,
            TypedResumableCall::Finished(_) => panic!("matching wait must park"),
        };

        assert!(invocation.host_func().is_none());
        assert!(invocation.host_error().is_none());
        match invocation.suspension() {
            Suspension::Atomic(suspend) => {
                assert_eq!(read_i32(&store, suspend.memory, 12), 7);
                assert_eq!(suspend.addr, 12);
                assert!(matches!(
                    suspend.request,
                    AtomicSuspendRequest::Wait { timeout_ns: -1 }
                ));
            }
            Suspension::Host { .. } => panic!("wait must use typed atomic suspension"),
            Suspension::FuelQuantum => panic!("wait must not surface as a fuel suspension"),
        }

        let resumed = invocation
            .resume(&mut store, &[Value::I32(scheduler_result)])
            .expect("root-body wait must resume from its advanced instruction pointer");
        match resumed {
            TypedResumableCall::Finished(result) => assert_eq!(result, scheduler_result),
            TypedResumableCall::Resumable(_) => panic!("guest must finish after one resume"),
        }
        assert_eq!(read_i32(&store, memory, 0), scheduler_result);
    }
}

#[test]
fn matching_wait_in_non_resumable_call_fails_deterministically() {
    let (mut store, instance) = instantiate_surface();
    let wait32 = instance
        .get_typed_func::<(i32, i32, i64), i32>(&store, "wait32")
        .expect("wait32 export exists");
    let error = wait32
        .call(&mut store, (0, 0, -1))
        .expect_err("non-resumable matching wait must fail instead of hanging");
    assert!(matches!(
        error.trap_code(),
        Some(TrapCode::AtomicSuspendNotResumable)
    ));
}

#[test]
fn shared_notify_parks_and_delivers_scheduler_wake_count() {
    let (mut store, instance) = instantiate_surface();
    let notify = instance
        .get_typed_func::<(i32, i32), i32>(&store, "notify")
        .expect("notify export exists");
    let invocation = match notify
        .call_resumable(&mut store, (4, 2))
        .expect("shared notify must be resumable")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("shared notify must suspend"),
    };

    match invocation.suspension() {
        Suspension::Atomic(suspend) => {
            assert_eq!(suspend.addr, 4);
            assert!(matches!(
                suspend.request,
                AtomicSuspendRequest::Notify { count: 2 }
            ));
        }
        Suspension::Host { .. } => panic!("notify must use typed atomic suspension"),
        Suspension::FuelQuantum => panic!("notify must not surface as a fuel suspension"),
    }

    let resumed = invocation
        .resume(&mut store, &[Value::I32(1)])
        .expect("scheduler wake count must resume notify");
    match resumed {
        TypedResumableCall::Finished(result) => assert_eq!(result, 1),
        TypedResumableCall::Resumable(_) => panic!("notify guest must finish after one resume"),
    }
}

#[test]
fn atomic_resume_rejects_wrong_value_type_and_arity() {
    let (mut wrong_type_store, wrong_type_instance) = instantiate_surface();
    let wrong_type_wait = wrong_type_instance
        .get_typed_func::<(i32, i32, i64), i32>(&wrong_type_store, "wait32")
        .expect("wait32 export exists");
    let wrong_type_invocation = match wrong_type_wait
        .call_resumable(&mut wrong_type_store, (0, 0, -1))
        .expect("matching wait must suspend")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("matching wait must park"),
    };
    wrong_type_invocation
        .resume(&mut wrong_type_store, &[Value::I64(0)])
        .expect_err("atomic resume must reject i64");

    let (mut empty_store, empty_instance) = instantiate_surface();
    let empty_wait = empty_instance
        .get_typed_func::<(i32, i32, i64), i32>(&empty_store, "wait32")
        .expect("wait32 export exists");
    let empty_invocation = match empty_wait
        .call_resumable(&mut empty_store, (0, 0, -1))
        .expect("matching wait must suspend")
    {
        TypedResumableCall::Resumable(invocation) => invocation,
        TypedResumableCall::Finished(_) => panic!("matching wait must park"),
    };
    empty_invocation
        .resume(&mut empty_store, &[])
        .expect_err("atomic resume must reject missing i32 result");
}
