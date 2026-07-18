//! Scaffold test: proves the crate wiring (patched wasmi + wat) works on host.

#[test]
fn engine_builds_and_runs_flat_module() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "answer") (result i32)
                i32.const 42))
        "#,
    )
    .expect("wat compiles");

    let config = raios_wasmi_conformance::base_config();
    let engine = wasmi::Engine::new(&config);
    let module = wasmi::Module::new(&engine, wasm.as_slice()).expect("module validates");
    let mut store = wasmi::Store::new(&engine, ());
    let instance = wasmi::Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("instantiates")
        .start(&mut store)
        .expect("starts");
    let answer = instance
        .get_typed_func::<(), i32>(&store, "answer")
        .expect("export exists");
    assert_eq!(answer.call(&mut store, ()).expect("call succeeds"), 42);
}
