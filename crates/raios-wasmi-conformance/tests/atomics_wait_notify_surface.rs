use wasmi::{
    core::TrapCode, errors::ModuleError, Config, Engine, Error, Instance, Linker, Module, Store,
};

const WAIT_NOTIFY_SURFACE: &str = r#"
    (module
        (memory 1 1 shared)
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
            memory.atomic.wait64))
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
    let engine = engine_with_threads();
    let module = module_from_wat(&engine, WAIT_NOTIFY_SURFACE)
        .expect("wait/notify surface validates and translates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("wait/notify module instantiates")
        .start(&mut store)
        .expect("wait/notify module starts");
    (store, instance)
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

// T1-d-2 replaces the following terminal package-boundary traps with AtomicSuspend.
#[test]
fn notify_execution_traps_until_t1_d_2() {
    let (mut store, instance) = instantiate_surface();
    let notify = instance
        .get_typed_func::<(i32, i32), i32>(&store, "notify")
        .expect("notify export exists");
    let error = notify
        .call(&mut store, (0, 1))
        .expect_err("T1-d-2 will replace this terminal placeholder trap");
    assert!(matches!(
        error.trap_code(),
        Some(TrapCode::AtomicSuspendNotResumable)
    ));
}

#[test]
fn wait32_execution_traps_until_t1_d_2() {
    let (mut store, instance) = instantiate_surface();
    let wait32 = instance
        .get_typed_func::<(i32, i32, i64), i32>(&store, "wait32")
        .expect("wait32 export exists");
    let error = wait32
        .call(&mut store, (0, 0, -1))
        .expect_err("T1-d-2 will replace this terminal placeholder trap");
    assert!(matches!(
        error.trap_code(),
        Some(TrapCode::AtomicSuspendNotResumable)
    ));
}

#[test]
fn wait64_execution_traps_until_t1_d_2() {
    let (mut store, instance) = instantiate_surface();
    let wait64 = instance
        .get_typed_func::<(i32, i64, i64), i32>(&store, "wait64")
        .expect("wait64 export exists");
    let error = wait64
        .call(&mut store, (0, 0, -1))
        .expect_err("T1-d-2 will replace this terminal placeholder trap");
    assert!(matches!(
        error.trap_code(),
        Some(TrapCode::AtomicSuspendNotResumable)
    ));
}
