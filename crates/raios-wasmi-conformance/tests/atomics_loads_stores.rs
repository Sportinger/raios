use wasmi::{core::TrapCode, Config, Engine, Instance, Linker, Module, Store};

fn engine_with_threads() -> Engine {
    let mut config = Config::default();
    config.wasm_threads(true);
    Engine::new(&config)
}

fn module_from_wat(engine: &Engine, wat: &str) -> Module {
    let wasm = wat::parse_str(wat).expect("test WAT compiles");
    Module::new(engine, wasm.as_slice()).expect("module validates and translates")
}

fn assert_i32_roundtrip(
    instance: &Instance,
    store: &mut Store<()>,
    name: &str,
    address: i32,
    value: i32,
    expected: i32,
) {
    let func = instance
        .get_typed_func::<(i32, i32), i32>(&*store, name)
        .expect("i32 roundtrip export exists");
    assert_eq!(
        func.call(store, (address, value))
            .expect("atomic i32 roundtrip succeeds"),
        expected
    );
}

fn assert_i64_roundtrip(
    instance: &Instance,
    store: &mut Store<()>,
    name: &str,
    address: i32,
    value: i64,
    expected: i64,
) {
    let func = instance
        .get_typed_func::<(i32, i64), i64>(&*store, name)
        .expect("i64 roundtrip export exists");
    assert_eq!(
        func.call(store, (address, value))
            .expect("atomic i64 roundtrip succeeds"),
        expected
    );
}

fn assert_all_widths_roundtrip(memory_type: &str) {
    let engine = engine_with_threads();
    let wat = format!(
        r#"
        (module
            (memory {memory_type})
            (func (export "i32") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.atomic.store
                local.get 0
                i32.atomic.load)
            (func (export "i32_8") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.atomic.store8
                local.get 0
                i32.atomic.load8_u)
            (func (export "i32_16") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.atomic.store16
                local.get 0
                i32.atomic.load16_u)
            (func (export "i64") (param i32 i64) (result i64)
                local.get 0
                local.get 1
                i64.atomic.store
                local.get 0
                i64.atomic.load)
            (func (export "i64_8") (param i32 i64) (result i64)
                local.get 0
                local.get 1
                i64.atomic.store8
                local.get 0
                i64.atomic.load8_u)
            (func (export "i64_16") (param i32 i64) (result i64)
                local.get 0
                local.get 1
                i64.atomic.store16
                local.get 0
                i64.atomic.load16_u)
            (func (export "i64_32") (param i32 i64) (result i64)
                local.get 0
                local.get 1
                i64.atomic.store32
                local.get 0
                i64.atomic.load32_u))
        "#
    );
    let module = module_from_wat(&engine, &wat);
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("atomic module instantiates")
        .start(&mut store)
        .expect("atomic module starts");

    assert_i32_roundtrip(
        &instance,
        &mut store,
        "i32",
        0,
        0x89AB_CDEF_u32 as i32,
        0x89AB_CDEF_u32 as i32,
    );
    assert_i32_roundtrip(&instance, &mut store, "i32_8", 1, 0x1AB, 0xAB);
    assert_i32_roundtrip(&instance, &mut store, "i32_16", 2, 0x1_2345, 0x2345);
    assert_i64_roundtrip(
        &instance,
        &mut store,
        "i64",
        8,
        0x0123_4567_89AB_CDEF,
        0x0123_4567_89AB_CDEF,
    );
    assert_i64_roundtrip(&instance, &mut store, "i64_8", 3, 0x1AB, 0xAB);
    assert_i64_roundtrip(&instance, &mut store, "i64_16", 4, 0x1_2345, 0x2345);
    assert_i64_roundtrip(
        &instance,
        &mut store,
        "i64_32",
        16,
        0x0000_0001_89AB_CDEF,
        0x0000_0000_89AB_CDEF,
    );
}

#[test]
fn all_atomic_widths_roundtrip_on_shared_memory() {
    assert_all_widths_roundtrip("1 1 shared");
}

#[test]
fn all_atomic_widths_roundtrip_on_unshared_memory() {
    assert_all_widths_roundtrip("1 1");
}

#[test]
fn unaligned_effective_address_traps_with_identity() {
    let engine = engine_with_threads();
    let module = module_from_wat(
        &engine,
        r#"
        (module
            (memory 1 1 shared)
            (func (export "load") (param i32) (result i32)
                local.get 0
                i32.atomic.load offset=4 align=4))
        "#,
    );
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("unaligned test module instantiates")
        .start(&mut store)
        .expect("unaligned test module starts");
    let load = instance
        .get_typed_func::<i32, i32>(&store, "load")
        .expect("load export exists");
    let error = load
        .call(&mut store, 1)
        .expect_err("effective address 1 + 4 is not naturally aligned");

    assert!(matches!(error.trap_code(), Some(TrapCode::UnalignedAtomic)));
}

#[test]
fn atomic_fence_executes_without_effect() {
    let engine = engine_with_threads();
    let module = module_from_wat(
        &engine,
        r#"
        (module
            (func (export "fence") (param i32) (result i32)
                atomic.fence
                local.get 0))
        "#,
    );
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("fence module instantiates")
        .start(&mut store)
        .expect("fence module starts");
    let fence = instance
        .get_typed_func::<i32, i32>(&store, "fence")
        .expect("fence export exists");

    assert_eq!(
        fence.call(&mut store, 42).expect("atomic fence executes"),
        42
    );
}
