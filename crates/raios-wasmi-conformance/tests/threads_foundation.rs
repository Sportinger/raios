use wasmi::{
    errors::LinkerError, Config, Engine, Error, Linker, Memory, MemoryType, Module, Store,
};

fn engine_with_threads() -> Engine {
    let mut config = Config::default();
    config.wasm_threads(true);
    Engine::new(&config)
}

fn module_from_wat(engine: &Engine, wat: &str) -> Result<Module, Error> {
    let wasm = wat::parse_str(wat).expect("test WAT compiles");
    Module::new(engine, wasm.as_slice())
}

#[test]
fn shared_memory_is_rejected_by_default() {
    let engine = Engine::default();
    let error = module_from_wat(&engine, "(module (memory 1 1 shared))")
        .expect_err("threads must remain disabled by default");

    assert!(
        error
            .to_string()
            .contains("threads must be enabled for shared memories"),
        "unexpected validation error: {error}"
    );
}

#[test]
fn shared_memory_validates_instantiates_and_reports_its_type() {
    let engine = engine_with_threads();
    let module = module_from_wat(&engine, "(module (memory (export \"memory\") 1 1 shared))")
        .expect("shared memory validates when threads are enabled");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("shared memory instantiates")
        .start(&mut store)
        .expect("instance starts");
    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export exists");

    assert!(memory.ty(&store).is_shared());
}

#[test]
fn shared_memory_without_maximum_is_rejected() {
    // The shared bit without the maximum-present bit is representable in the
    // binary format even though text-format tooling rejects it up front.
    const SHARED_WITHOUT_MAXIMUM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, // header
        0x05, 0x03, 0x01, 0x02, 0x01, // memory section: shared, min 1, no max
    ];
    let engine = engine_with_threads();
    let error = Module::new(&engine, SHARED_WITHOUT_MAXIMUM)
        .expect_err("a shared memory without a maximum must not validate");

    assert!(
        error.to_string().contains("maximum"),
        "unexpected validation error: {error}"
    );
}

fn assert_memory_link_error(engine: &Engine, module_wat: &str, supplied_type: MemoryType) {
    let module = module_from_wat(engine, module_wat).expect("import module validates");
    let mut store = Store::new(engine, ());
    let memory = Memory::new(&mut store, supplied_type).expect("host memory allocates");
    let mut linker = Linker::new(engine);
    linker
        .define("env", "memory", memory)
        .expect("memory definition is unique");
    let error = linker
        .instantiate(&mut store, &module)
        .expect_err("sharedness mismatch must be a link error");

    assert!(
        matches!(
            error,
            Error::Linker(LinkerError::InvalidMemorySubtype { .. })
        ),
        "unexpected link error: {error:?}"
    );
}

#[test]
fn shared_and_unshared_imports_do_not_match_in_either_direction() {
    let engine = engine_with_threads();
    assert_memory_link_error(
        &engine,
        "(module (import \"env\" \"memory\" (memory 1 1 shared)))",
        MemoryType::new(1, Some(1)).expect("unshared type is valid"),
    );
    assert_memory_link_error(
        &engine,
        "(module (import \"env\" \"memory\" (memory 1 1)))",
        MemoryType::new_shared(1, 1).expect("shared type is valid"),
    );
}

#[test]
fn two_instances_observe_plain_writes_through_one_shared_memory() {
    let engine = engine_with_threads();
    let module = module_from_wat(
        &engine,
        r#"
        (module
            (import "env" "memory" (memory 1 1 shared))
            (func (export "write") (param i32)
                i32.const 0
                local.get 0
                i32.store)
            (func (export "read") (result i32)
                i32.const 0
                i32.load))
        "#,
    )
    .expect("shared-memory import module validates");
    let mut store = Store::new(&engine, ());
    let shared = Memory::new(
        &mut store,
        MemoryType::new_shared(1, 1).expect("shared type is valid"),
    )
    .expect("shared memory allocates");
    let mut linker = Linker::new(&engine);
    linker
        .define("env", "memory", shared)
        .expect("memory definition is unique");

    let instance_a = linker
        .instantiate(&mut store, &module)
        .expect("shared import matches for instance A")
        .start(&mut store)
        .expect("instance A starts");
    let instance_b = linker
        .instantiate(&mut store, &module)
        .expect("shared import matches for instance B")
        .start(&mut store)
        .expect("instance B starts");
    let write_a = instance_a
        .get_typed_func::<i32, ()>(&store, "write")
        .expect("write export exists");
    let read_b = instance_b
        .get_typed_func::<(), i32>(&store, "read")
        .expect("read export exists");

    write_a
        .call(&mut store, 0x1234_5678)
        .expect("plain store succeeds");
    assert_eq!(
        read_b.call(&mut store, ()).expect("plain load succeeds"),
        0x1234_5678
    );
}

#[test]
fn wait_and_notify_translate_when_threads_are_enabled() {
    let engine = engine_with_threads();
    module_from_wat(
        &engine,
        r#"
        (module
            (memory 1 1 shared)
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
            (func (export "notify") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                memory.atomic.notify))
        "#,
    )
    .expect("wait32, wait64, and notify translate when threads are enabled");
}
