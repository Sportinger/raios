use wasmi::{core::TrapCode, Config, Engine, Instance, Linker, Module, Store};

fn engine_with_threads() -> Engine {
    let mut config = Config::default();
    config.wasm_threads(true);
    Engine::new(&config)
}

fn engine_with_threads_and_fuel() -> Engine {
    let mut config = Config::default();
    config.wasm_threads(true).consume_fuel(true);
    Engine::new(&config)
}

fn module_from_wat(engine: &Engine, wat: &str) -> Module {
    let wasm = wat::parse_str(wat).expect("test WAT compiles");
    Module::new(engine, wasm.as_slice()).expect("module validates and translates")
}

fn instantiate(engine: &Engine, wat: &str) -> (Store<()>, Instance) {
    let module = module_from_wat(engine, wat);
    let mut store = Store::new(engine, ());
    let instance = Linker::new(engine)
        .instantiate(&mut store, &module)
        .expect("atomic module instantiates")
        .start(&mut store)
        .expect("atomic module starts");
    (store, instance)
}

#[derive(Debug, Copy, Clone)]
enum ValueKind {
    I32,
    I64,
}

impl ValueKind {
    fn wat(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Width {
    name: &'static str,
    value_kind: ValueKind,
    rmw_stem: &'static str,
    load: &'static str,
    store: &'static str,
    mask: u64,
    address: i32,
    narrow: bool,
}

const WIDTHS: [Width; 7] = [
    Width {
        name: "i32",
        value_kind: ValueKind::I32,
        rmw_stem: "i32.atomic.rmw",
        load: "i32.atomic.load",
        store: "i32.atomic.store",
        mask: u32::MAX as u64,
        address: 0,
        narrow: false,
    },
    Width {
        name: "i64",
        value_kind: ValueKind::I64,
        rmw_stem: "i64.atomic.rmw",
        load: "i64.atomic.load",
        store: "i64.atomic.store",
        mask: u64::MAX,
        address: 8,
        narrow: false,
    },
    Width {
        name: "i32_8",
        value_kind: ValueKind::I32,
        rmw_stem: "i32.atomic.rmw8",
        load: "i32.atomic.load8_u",
        store: "i32.atomic.store8",
        mask: u8::MAX as u64,
        address: 1,
        narrow: true,
    },
    Width {
        name: "i32_16",
        value_kind: ValueKind::I32,
        rmw_stem: "i32.atomic.rmw16",
        load: "i32.atomic.load16_u",
        store: "i32.atomic.store16",
        mask: u16::MAX as u64,
        address: 2,
        narrow: true,
    },
    Width {
        name: "i64_8",
        value_kind: ValueKind::I64,
        rmw_stem: "i64.atomic.rmw8",
        load: "i64.atomic.load8_u",
        store: "i64.atomic.store8",
        mask: u8::MAX as u64,
        address: 3,
        narrow: true,
    },
    Width {
        name: "i64_16",
        value_kind: ValueKind::I64,
        rmw_stem: "i64.atomic.rmw16",
        load: "i64.atomic.load16_u",
        store: "i64.atomic.store16",
        mask: u16::MAX as u64,
        address: 4,
        narrow: true,
    },
    Width {
        name: "i64_32",
        value_kind: ValueKind::I64,
        rmw_stem: "i64.atomic.rmw32",
        load: "i64.atomic.load32_u",
        store: "i64.atomic.store32",
        mask: u32::MAX as u64,
        address: 16,
        narrow: true,
    },
];

#[derive(Debug, Copy, Clone)]
enum RmwOperation {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Xchg,
}

impl RmwOperation {
    fn wat(self, narrow: bool) -> &'static str {
        match (self, narrow) {
            (Self::Add, false) => "add",
            (Self::Sub, false) => "sub",
            (Self::And, false) => "and",
            (Self::Or, false) => "or",
            (Self::Xor, false) => "xor",
            (Self::Xchg, false) => "xchg",
            (Self::Add, true) => "add_u",
            (Self::Sub, true) => "sub_u",
            (Self::And, true) => "and_u",
            (Self::Or, true) => "or_u",
            (Self::Xor, true) => "xor_u",
            (Self::Xchg, true) => "xchg_u",
        }
    }

    fn expected(self, old: u64, operand: u64, mask: u64) -> u64 {
        let value = match self {
            Self::Add => old.wrapping_add(operand),
            Self::Sub => old.wrapping_sub(operand),
            Self::And => old & operand,
            Self::Or => old | operand,
            Self::Xor => old ^ operand,
            Self::Xchg => operand,
        };
        value & mask
    }
}

const RMW_OPERATIONS: [RmwOperation; 6] = [
    RmwOperation::Add,
    RmwOperation::Sub,
    RmwOperation::And,
    RmwOperation::Or,
    RmwOperation::Xor,
    RmwOperation::Xchg,
];

fn build_rmw_module(memory_type: &str, width: Width, operation: RmwOperation) -> String {
    let value_type = width.value_kind.wat();
    let rmw = format!("{}.{}", width.rmw_stem, operation.wat(width.narrow));
    format!(
        r#"
        (module
            (memory {memory_type})
            (func (export "init") (param i32 {value_type})
                local.get 0
                local.get 1
                {store})
            (func (export "rmw") (param i32 {value_type}) (result {value_type})
                local.get 0
                local.get 1
                {rmw})
            (func (export "load") (param i32) (result {value_type})
                local.get 0
                {load}))
        "#,
        store = width.store,
        load = width.load,
    )
}

fn build_cmpxchg_module(memory_type: &str, width: Width) -> String {
    let value_type = width.value_kind.wat();
    let suffix = if width.narrow { "cmpxchg_u" } else { "cmpxchg" };
    let cmpxchg = format!("{}.{}", width.rmw_stem, suffix);
    format!(
        r#"
        (module
            (memory {memory_type})
            (func (export "init") (param i32 {value_type})
                local.get 0
                local.get 1
                {store})
            (func (export "cmpxchg")
                (param i32 {value_type} {value_type})
                (result {value_type})
                local.get 0
                local.get 1
                local.get 2
                {cmpxchg})
            (func (export "load") (param i32) (result {value_type})
                local.get 0
                {load}))
        "#,
        store = width.store,
        load = width.load,
    )
}

fn call_init(width: Width, instance: &Instance, store: &mut Store<()>, value_bits: u64) {
    match width.value_kind {
        ValueKind::I32 => instance
            .get_typed_func::<(i32, i32), ()>(&*store, "init")
            .expect("i32 init export exists")
            .call(store, (width.address, value_bits as u32 as i32))
            .expect("i32 atomic init succeeds"),
        ValueKind::I64 => instance
            .get_typed_func::<(i32, i64), ()>(&*store, "init")
            .expect("i64 init export exists")
            .call(store, (width.address, value_bits as i64))
            .expect("i64 atomic init succeeds"),
    }
}

fn call_rmw(width: Width, instance: &Instance, store: &mut Store<()>, operand_bits: u64) -> u64 {
    match width.value_kind {
        ValueKind::I32 => instance
            .get_typed_func::<(i32, i32), i32>(&*store, "rmw")
            .expect("i32 rmw export exists")
            .call(store, (width.address, operand_bits as u32 as i32))
            .expect("i32 rmw succeeds") as u32 as u64,
        ValueKind::I64 => instance
            .get_typed_func::<(i32, i64), i64>(&*store, "rmw")
            .expect("i64 rmw export exists")
            .call(store, (width.address, operand_bits as i64))
            .expect("i64 rmw succeeds") as u64,
    }
}

fn call_cmpxchg(
    width: Width,
    instance: &Instance,
    store: &mut Store<()>,
    expected_bits: u64,
    replacement_bits: u64,
) -> u64 {
    match width.value_kind {
        ValueKind::I32 => instance
            .get_typed_func::<(i32, i32, i32), i32>(&*store, "cmpxchg")
            .expect("i32 cmpxchg export exists")
            .call(
                store,
                (
                    width.address,
                    expected_bits as u32 as i32,
                    replacement_bits as u32 as i32,
                ),
            )
            .expect("i32 cmpxchg succeeds") as u32 as u64,
        ValueKind::I64 => instance
            .get_typed_func::<(i32, i64, i64), i64>(&*store, "cmpxchg")
            .expect("i64 cmpxchg export exists")
            .call(
                store,
                (width.address, expected_bits as i64, replacement_bits as i64),
            )
            .expect("i64 cmpxchg succeeds") as u64,
    }
}

fn call_load(width: Width, instance: &Instance, store: &mut Store<()>) -> u64 {
    match width.value_kind {
        ValueKind::I32 => instance
            .get_typed_func::<i32, i32>(&*store, "load")
            .expect("i32 load export exists")
            .call(store, width.address)
            .expect("i32 atomic load succeeds") as u32 as u64,
        ValueKind::I64 => instance
            .get_typed_func::<i32, i64>(&*store, "load")
            .expect("i64 load export exists")
            .call(store, width.address)
            .expect("i64 atomic load succeeds") as u64,
    }
}

fn outside_width_bit(width: Width) -> u64 {
    if width.narrow {
        width.mask + 1
    } else {
        0
    }
}

fn assert_rmw_case(memory_type: &str, width: Width, operation: RmwOperation) {
    let engine = engine_with_threads();
    let wat = build_rmw_module(memory_type, width, operation);
    let (mut store, instance) = instantiate(&engine, &wat);
    let old = width.mask.wrapping_sub(1);
    let operand = 3;
    let outside = outside_width_bit(width);

    call_init(width, &instance, &mut store, old | outside);
    let returned = call_rmw(width, &instance, &mut store, operand | outside);
    let stored = call_load(width, &instance, &mut store);

    assert_eq!(
        returned, old,
        "{operation:?} {} must return the zero-extended old value",
        width.name
    );
    assert_eq!(
        stored,
        operation.expected(old, operand, width.mask),
        "{operation:?} {} must store the width-wrapped result",
        width.name
    );
}

#[test]
fn every_rmw_family_and_width_returns_old_and_stores_new() {
    for operation in RMW_OPERATIONS {
        for width in WIDTHS {
            assert_rmw_case("1 1 shared", width, operation);
            if !width.narrow {
                assert_rmw_case("1 1", width, operation);
            }
        }
    }
}

fn assert_cmpxchg_case(memory_type: &str, width: Width) {
    let engine = engine_with_threads();
    let wat = build_cmpxchg_module(memory_type, width);
    let (mut store, instance) = instantiate(&engine, &wat);
    let old = width.mask.wrapping_sub(1);
    let replacement = 5;
    let outside = outside_width_bit(width);

    call_init(width, &instance, &mut store, old | outside);
    let success_old = call_cmpxchg(
        width,
        &instance,
        &mut store,
        old | outside,
        replacement | outside,
    );
    assert_eq!(
        success_old, old,
        "successful cmpxchg {} must return the zero-extended old value",
        width.name
    );
    assert_eq!(
        call_load(width, &instance, &mut store),
        replacement,
        "successful cmpxchg {} must store the wrapped replacement",
        width.name
    );

    call_init(width, &instance, &mut store, old | outside);
    let failure_old = call_cmpxchg(
        width,
        &instance,
        &mut store,
        old.wrapping_sub(1) | outside,
        replacement | outside,
    );
    assert_eq!(
        failure_old, old,
        "failed cmpxchg {} must return the zero-extended old value",
        width.name
    );
    assert_eq!(
        call_load(width, &instance, &mut store),
        old,
        "failed cmpxchg {} must leave memory unchanged",
        width.name
    );
}

#[test]
fn cmpxchg_covers_success_failure_and_narrow_expected_wrapping() {
    for width in WIDTHS {
        assert_cmpxchg_case("1 1 shared", width);
        if !width.narrow {
            assert_cmpxchg_case("1 1", width);
        }
    }
}

#[test]
fn unaligned_rmw_and_cmpxchg_trap_with_identity() {
    let engine = engine_with_threads();
    let (mut store, instance) = instantiate(
        &engine,
        r#"
        (module
            (memory 1 1 shared)
            (func (export "rmw") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.atomic.rmw.add offset=4 align=4)
            (func (export "cmpxchg") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                i32.atomic.rmw.cmpxchg offset=4 align=4))
        "#,
    );
    let rmw = instance
        .get_typed_func::<(i32, i32), i32>(&store, "rmw")
        .expect("rmw export exists");
    let cmpxchg = instance
        .get_typed_func::<(i32, i32, i32), i32>(&store, "cmpxchg")
        .expect("cmpxchg export exists");

    let rmw_error = rmw
        .call(&mut store, (1, 1))
        .expect_err("effective address 1 + 4 is not naturally aligned");
    assert!(matches!(
        rmw_error.trap_code(),
        Some(TrapCode::UnalignedAtomic)
    ));

    let cmpxchg_error = cmpxchg
        .call(&mut store, (1, 0, 1))
        .expect_err("effective address 1 + 4 is not naturally aligned");
    assert!(matches!(
        cmpxchg_error.trap_code(),
        Some(TrapCode::UnalignedAtomic)
    ));
}

#[test]
fn cmpxchg_fuel_is_constant_for_success_and_failure() {
    let engine = engine_with_threads_and_fuel();
    let width = WIDTHS[0];
    let wat = build_cmpxchg_module("1 1 shared", width);
    let (mut store, instance) = instantiate(&engine, &wat);
    store.add_fuel(1_000_000).expect("fuel can be added");

    call_init(width, &instance, &mut store, 7);
    let before_success = store.fuel_consumed().expect("fuel metering is enabled");
    assert_eq!(call_cmpxchg(width, &instance, &mut store, 7, 11), 7);
    let after_success = store.fuel_consumed().expect("fuel metering is enabled");

    call_init(width, &instance, &mut store, 7);
    let before_failure = store.fuel_consumed().expect("fuel metering is enabled");
    assert_eq!(call_cmpxchg(width, &instance, &mut store, 6, 11), 7);
    let after_failure = store.fuel_consumed().expect("fuel metering is enabled");

    assert_eq!(
        after_success - before_success,
        after_failure - before_failure,
        "cmpxchg must charge load plus store independent of comparison result"
    );
}
