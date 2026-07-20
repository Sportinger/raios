use std::panic::{catch_unwind, AssertUnwindSafe};

use wasmi::{
    core::TrapCode,
    errors::{MemoryError, TableError},
    Config, Engine, Error, FuelConsumptionMode, Func, Instance, Linker, Memory, Module,
    ResourceLimiter, ResumableCall, Store, Suspension, Table, Value,
};

const MEMORY_PAGES: u32 = 2;
const MEMORY_MAX_PAGES: u32 = 3;
const MEMORY_BYTES: usize = MEMORY_PAGES as usize * 64 * 1024;
const BULK_BYTES: usize = 8 * 1024;
const BULK_ELEMENTS: u32 = 1024;
const TABLE_SIZE: u32 = 2048;
const TABLE_MAX: u32 = 4096;
const INITIAL_FUEL: u64 = 100;
const LARGE_REFILL: u64 = 2_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryGrowCall {
    current: usize,
    desired: usize,
    maximum: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TableGrowCall {
    current: u32,
    desired: u32,
    maximum: Option<u32>,
}

#[derive(Debug, Default)]
struct HostState {
    memory_grows: Vec<MemoryGrowCall>,
    table_grows: Vec<TableGrowCall>,
}

impl ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, MemoryError> {
        self.memory_grows.push(MemoryGrowCall {
            current,
            desired,
            maximum,
        });
        Ok(true)
    }

    fn table_growing(
        &mut self,
        current: u32,
        desired: u32,
        maximum: Option<u32>,
    ) -> Result<bool, TableError> {
        self.table_grows.push(TableGrowCall {
            current,
            desired,
            maximum,
        });
        Ok(true)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParkStep {
    consumed: u64,
    completed_effects: i32,
}

#[derive(Debug)]
struct RunTrace {
    result: i32,
    parks: Vec<ParkStep>,
    fuel_consumed: u64,
}

fn engine(resumable: bool) -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true).resumable_fuel(resumable);
    Engine::new(&config)
}

fn wat_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("\\{byte:02x}"))
        .collect::<String>()
}

fn fixture_wat() -> (String, Vec<u8>) {
    let data = (0..BULK_BYTES + 32)
        .map(|index| (index % 251) as u8)
        .collect::<Vec<_>>();
    let active_elements = (0..TABLE_SIZE)
        .map(|index| match index % 3 {
            0 => "$v1",
            1 => "$v2",
            _ => "$v3",
        })
        .collect::<Vec<_>>()
        .join(" ");
    let passive_elements = (0..TABLE_SIZE)
        .map(|index| match index % 3 {
            0 => "$v3",
            1 => "$v2",
            _ => "$v1",
        })
        .collect::<Vec<_>>()
        .join(" ");
    let wat = format!(
        r#"
        (module
            (type $ret_i32 (func (result i32)))
            (memory (export "memory") {MEMORY_PAGES} {MEMORY_MAX_PAGES})
            (data $data "{}")

            (func $v1 (export "v1") (type $ret_i32) (result i32) i32.const 1)
            (func $v2 (export "v2") (type $ret_i32) (result i32) i32.const 2)
            (func $v3 (export "v3") (type $ret_i32) (result i32) i32.const 3)
            (func $v7 (export "v7") (type $ret_i32) (result i32) i32.const 7)
            (func $v9 (export "v9") (type $ret_i32) (result i32) i32.const 9)

            (table $grow (export "grow_table") 32 {TABLE_MAX} funcref)
            (table $fill (export "fill_table") {TABLE_SIZE} {TABLE_MAX} funcref)
            (table $copy_src {TABLE_SIZE} {TABLE_MAX} funcref)
            (table $copy_dst (export "copy_table") {TABLE_SIZE} {TABLE_MAX} funcref)
            (table $init (export "init_table") {TABLE_SIZE} {TABLE_MAX} funcref)
            (elem (table $copy_src) (i32.const 0) func {active_elements})
            (elem $passive func {passive_elements})

            (func $bump (result i32)
                i32.const 0
                i32.const 0
                i32.load
                i32.const 1
                i32.add
                i32.store
                i32.const 0
                i32.load)

            (func (export "memory_grow") (param $delta i32) (result i32)
                (local $old i32)
                local.get $delta
                memory.grow
                local.set $old
                call $bump
                drop
                local.get $old)

            (func (export "memory_fill") (param $dst i32) (param $value i32) (param $len i32) (result i32)
                local.get $dst
                local.get $value
                local.get $len
                memory.fill
                call $bump)

            (func (export "memory_copy") (param $dst i32) (param $src i32) (param $len i32) (result i32)
                local.get $dst
                local.get $src
                local.get $len
                memory.copy
                call $bump)

            (func (export "memory_init") (param $dst i32) (param $src i32) (param $len i32) (result i32)
                local.get $dst
                local.get $src
                local.get $len
                memory.init $data
                call $bump)

            (func (export "table_grow") (param $delta i32) (result i32)
                (local $old i32)
                ref.func $v9
                local.get $delta
                table.grow $grow
                local.set $old
                call $bump
                drop
                local.get $old)

            (func (export "table_fill") (param $dst i32) (param $len i32) (result i32)
                local.get $dst
                ref.func $v7
                local.get $len
                table.fill $fill
                call $bump)

            (func (export "table_copy") (param $dst i32) (param $src i32) (param $len i32) (result i32)
                local.get $dst
                local.get $src
                local.get $len
                table.copy $copy_dst $copy_src
                call $bump)

            (func (export "table_init") (param $dst i32) (param $src i32) (param $len i32) (result i32)
                local.get $dst
                local.get $src
                local.get $len
                table.init $init $passive
                call $bump)

            (func (export "park_heavy") (result i32)
                (local $iteration i32)
                (loop $again
                    i32.const 40000
                    local.get $iteration
                    i32.const 17
                    i32.mul
                    i32.add
                    local.get $iteration
                    i32.const 31
                    i32.mul
                    i32.const 68
                    i32.add
                    i32.const {BULK_BYTES}
                    memory.fill
                    call $bump
                    drop
                    local.get $iteration
                    i32.const 1
                    i32.add
                    local.tee $iteration
                    i32.const 4
                    i32.lt_u
                    br_if $again)
                i32.const 0
                i32.load))
        "#,
        wat_bytes(&data),
    );
    (wat, data)
}

fn module(engine: &Engine) -> (Module, Vec<u8>) {
    let (wat, data) = fixture_wat();
    let wasm = wat::parse_str(&wat).expect("bulk fixture WAT compiles");
    (
        Module::new(engine, wasm.as_slice()).expect("bulk fixture validates"),
        data,
    )
}

fn instantiate(engine: &Engine, module: &Module) -> (Store<HostState>, Instance) {
    let mut store = Store::new(engine, HostState::default());
    store.limiter(|state| state);
    let instance = Linker::new(engine)
        .instantiate(&mut store, module)
        .expect("bulk fixture instantiates")
        .start(&mut store)
        .expect("bulk fixture starts");
    (store, instance)
}

fn exported_func(store: &Store<HostState>, instance: Instance, name: &str) -> Func {
    instance
        .get_func(store, name)
        .unwrap_or_else(|| panic!("missing {name} export"))
}

fn exported_memory(store: &Store<HostState>, instance: Instance) -> Memory {
    instance
        .get_memory(store, "memory")
        .expect("memory export exists")
}

fn exported_table(store: &Store<HostState>, instance: Instance, name: &str) -> Table {
    instance
        .get_table(store, name)
        .unwrap_or_else(|| panic!("missing {name} table export"))
}

fn read_i32(store: &Store<HostState>, memory: Memory, offset: usize) -> i32 {
    let mut bytes = [0_u8; 4];
    memory
        .read(store, offset, &mut bytes)
        .expect("i32 read is in bounds");
    i32::from_le_bytes(bytes)
}

fn read_bytes(store: &Store<HostState>, memory: Memory, offset: usize, len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    memory
        .read(store, offset, &mut bytes)
        .expect("byte read is in bounds");
    bytes
}

fn run_i32(
    store: &mut Store<HostState>,
    instance: Instance,
    name: &str,
    params: &[Value],
    initial_fuel: u64,
    refill: u64,
) -> RunTrace {
    let func = exported_func(store, instance, name);
    let memory = exported_memory(store, instance);
    store
        .add_fuel(initial_fuel)
        .expect("initial fuel can be added");
    let mut outputs = [Value::I32(0)];
    let mut outcome = func
        .call_resumable(&mut *store, params, &mut outputs)
        .unwrap_or_else(|error| panic!("{name} starts resumably: {error}"));
    let mut parks = Vec::new();
    loop {
        match outcome {
            ResumableCall::Finished => {
                let result = match outputs[0] {
                    Value::I32(result) => result,
                    ref other => panic!("{name} returned non-i32 value: {other:?}"),
                };
                return RunTrace {
                    result,
                    parks,
                    fuel_consumed: store.fuel_consumed().expect("fuel enabled"),
                };
            }
            ResumableCall::Resumable(invocation) => {
                assert!(
                    matches!(invocation.suspension(), Suspension::FuelQuantum),
                    "{name} may only park for fuel"
                );
                parks.push(ParkStep {
                    consumed: store.fuel_consumed().expect("fuel enabled"),
                    completed_effects: read_i32(store, memory, 0),
                });
                store.add_fuel(refill).expect("fuel refill succeeds");
                outcome = invocation
                    .resume(&mut *store, &[], &mut outputs)
                    .unwrap_or_else(|error| panic!("{name} resumes: {error}"));
            }
        }
    }
}

fn table_value(store: &mut Store<HostState>, table: Table, index: u32) -> i32 {
    store.add_fuel(100).expect("probe fuel can be added");
    let func = match table.get(&*store, index) {
        Some(Value::FuncRef(funcref)) => *funcref
            .func()
            .unwrap_or_else(|| panic!("table[{index}] must be non-null")),
        other => panic!("table[{index}] must contain a funcref: {other:?}"),
    };
    func.typed::<(), i32>(&mut *store)
        .expect("table function has [] -> i32 type")
        .call(store, ())
        .unwrap_or_else(|error| panic!("table[{index}] call succeeds: {error}"))
}

fn assert_one_dynamic_park(trace: &RunTrace, operation: &str) {
    assert_eq!(
        trace.parks.len(),
        1,
        "{operation} must reach its dynamic charge, park once, and resume"
    );
    assert_eq!(
        trace.parks[0].completed_effects, 0,
        "{operation} must not apply its effect before parking"
    );
}

#[test]
fn all_eight_dynamic_ops_restore_distinct_operands_before_park_and_resume() {
    let engine = engine(true);
    let (module, data) = module(&engine);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory = exported_memory(&store, instance);
    let baseline = store.data().memory_grows.len();
    let trace = run_i32(
        &mut store,
        instance,
        "memory_grow",
        &[Value::I32(1)],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "memory.grow");
    assert_eq!(trace.result, MEMORY_PAGES as i32);
    assert_eq!(u32::from(memory.current_pages(&store)), MEMORY_PAGES + 1);
    assert_eq!(store.data().memory_grows.len(), baseline + 1);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory = exported_memory(&store, instance);
    let dst = 123;
    let value = 0xA7;
    let trace = run_i32(
        &mut store,
        instance,
        "memory_fill",
        &[
            Value::I32(dst),
            Value::I32(value),
            Value::I32(BULK_BYTES as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "memory.fill");
    assert_eq!(trace.result, 1);
    assert_eq!(
        read_bytes(&store, memory, dst as usize, BULK_BYTES),
        vec![value as u8; BULK_BYTES]
    );
    assert_eq!(read_bytes(&store, memory, dst as usize - 1, 1), vec![0]);
    assert_eq!(
        read_bytes(&store, memory, dst as usize + BULK_BYTES, 1),
        vec![0]
    );

    let (mut store, instance) = instantiate(&engine, &module);
    let memory = exported_memory(&store, instance);
    let src = 321;
    let dst = 17_000;
    let source = (0..BULK_BYTES)
        .map(|index| (index % 239) as u8)
        .collect::<Vec<_>>();
    memory
        .write(&mut store, src, &source)
        .expect("copy source write succeeds");
    let trace = run_i32(
        &mut store,
        instance,
        "memory_copy",
        &[
            Value::I32(dst as i32),
            Value::I32(src as i32),
            Value::I32(BULK_BYTES as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "memory.copy");
    assert_eq!(trace.result, 1);
    assert_eq!(read_bytes(&store, memory, dst, BULK_BYTES), source);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory = exported_memory(&store, instance);
    let src = 7;
    let dst = 30_000;
    let trace = run_i32(
        &mut store,
        instance,
        "memory_init",
        &[
            Value::I32(dst as i32),
            Value::I32(src as i32),
            Value::I32(BULK_BYTES as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "memory.init");
    assert_eq!(trace.result, 1);
    assert_eq!(
        read_bytes(&store, memory, dst, BULK_BYTES),
        data[src..src + BULK_BYTES]
    );

    let (mut store, instance) = instantiate(&engine, &module);
    let table = exported_table(&store, instance, "grow_table");
    let baseline = store.data().table_grows.len();
    let trace = run_i32(
        &mut store,
        instance,
        "table_grow",
        &[Value::I32(BULK_ELEMENTS as i32)],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "table.grow");
    assert_eq!(trace.result, 32);
    assert_eq!(table.size(&store), 32 + BULK_ELEMENTS);
    assert_eq!(store.data().table_grows.len(), baseline + 1);
    assert_eq!(table_value(&mut store, table, 777), 9);

    let (mut store, instance) = instantiate(&engine, &module);
    let table = exported_table(&store, instance, "fill_table");
    let dst = 137;
    let trace = run_i32(
        &mut store,
        instance,
        "table_fill",
        &[Value::I32(dst), Value::I32(BULK_ELEMENTS as i32)],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "table.fill");
    assert_eq!(trace.result, 1);
    assert_eq!(table_value(&mut store, table, dst as u32), 7);
    assert_eq!(
        table_value(&mut store, table, (dst + BULK_ELEMENTS as i32 - 1) as u32),
        7
    );

    let (mut store, instance) = instantiate(&engine, &module);
    let table = exported_table(&store, instance, "copy_table");
    let src = 211;
    let dst = 733;
    let trace = run_i32(
        &mut store,
        instance,
        "table_copy",
        &[
            Value::I32(dst),
            Value::I32(src),
            Value::I32(BULK_ELEMENTS as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "table.copy");
    assert_eq!(trace.result, 1);
    assert_eq!(table_value(&mut store, table, dst as u32), 2);
    assert_eq!(
        table_value(&mut store, table, (dst + BULK_ELEMENTS as i32 - 1) as u32),
        2
    );

    let (mut store, instance) = instantiate(&engine, &module);
    let table = exported_table(&store, instance, "init_table");
    let src = 157;
    let dst = 321;
    let trace = run_i32(
        &mut store,
        instance,
        "table_init",
        &[
            Value::I32(dst),
            Value::I32(src),
            Value::I32(BULK_ELEMENTS as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "table.init");
    assert_eq!(trace.result, 1);
    assert_eq!(table_value(&mut store, table, dst as u32), 2);
    assert_eq!(
        table_value(&mut store, table, (dst + BULK_ELEMENTS as i32 - 1) as u32),
        2
    );
}

#[test]
fn charge_larger_than_a_quantum_banks_residual_for_the_exact_round_count() {
    const QUANTUM: u64 = 50;
    const DYNAMIC_CHARGE: u64 = BULK_BYTES as u64 / 64;

    let engine = engine(true);
    let (module, _) = module(&engine);
    let (mut store, instance) = instantiate(&engine, &module);
    let paced = run_i32(
        &mut store,
        instance,
        "memory_fill",
        &[
            Value::I32(4096),
            Value::I32(0x5A),
            Value::I32(BULK_BYTES as i32),
        ],
        QUANTUM,
        QUANTUM,
    );
    let residual_at_first_park = QUANTUM
        .checked_sub(paced.parks[0].consumed)
        .expect("pre-effect block cost fits in the first quantum");
    let expected_parks = (DYNAMIC_CHARGE - residual_at_first_park).div_ceil(QUANTUM);
    assert!(DYNAMIC_CHARGE > QUANTUM);
    assert_eq!(paced.parks.len() as u64, expected_parks);
    assert_eq!(paced.result, 1);

    let (mut control_store, control_instance) = instantiate(&engine, &module);
    let control = run_i32(
        &mut control_store,
        control_instance,
        "memory_fill",
        &[
            Value::I32(4096),
            Value::I32(0x5A),
            Value::I32(BULK_BYTES as i32),
        ],
        10_000,
        10_000,
    );
    assert!(control.parks.is_empty());
    assert_eq!(
        paced.fuel_consumed, control.fuel_consumed,
        "pacing must not charge on park or double-charge on resume"
    );
}

#[test]
fn park_then_drop_is_effect_free_and_refill_applies_once_without_limiter_calls() {
    let engine = engine(true);
    let (module, _) = module(&engine);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory = exported_memory(&store, instance);
    let baseline_memory_calls = store.data().memory_grows.len();
    let baseline_table_calls = store.data().table_grows.len();
    let dst = 20_000;
    let before = vec![0x11; BULK_BYTES];
    memory
        .write(&mut store, dst, &before)
        .expect("sentinel write succeeds");
    store.add_fuel(INITIAL_FUEL).expect("initial fuel succeeds");
    let mut outputs = [Value::I32(0)];
    let parked = exported_func(&store, instance, "memory_fill")
        .call_resumable(
            &mut store,
            &[
                Value::I32(dst as i32),
                Value::I32(0x6C),
                Value::I32(BULK_BYTES as i32),
            ],
            &mut outputs,
        )
        .expect("fill reaches fuel park");
    let invocation = match parked {
        ResumableCall::Resumable(invocation) => invocation,
        ResumableCall::Finished => panic!("insufficient dynamic fuel must park"),
    };
    assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
    assert_eq!(read_bytes(&store, memory, dst, BULK_BYTES), before);
    assert_eq!(read_i32(&store, memory, 0), 0);
    assert_eq!(store.data().memory_grows.len(), baseline_memory_calls);
    assert_eq!(store.data().table_grows.len(), baseline_table_calls);
    drop(invocation);
    assert_eq!(read_bytes(&store, memory, dst, BULK_BYTES), before);

    let (mut resumed_store, resumed_instance) = instantiate(&engine, &module);
    let resumed_memory = exported_memory(&resumed_store, resumed_instance);
    resumed_memory
        .write(&mut resumed_store, dst, &before)
        .expect("second sentinel write succeeds");
    let trace = run_i32(
        &mut resumed_store,
        resumed_instance,
        "memory_fill",
        &[
            Value::I32(dst as i32),
            Value::I32(0x6C),
            Value::I32(BULK_BYTES as i32),
        ],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert_one_dynamic_park(&trace, "memory.fill refill");
    assert_eq!(trace.result, 1, "post-effect counter proves one execution");
    assert_eq!(
        read_bytes(&resumed_store, resumed_memory, dst, BULK_BYTES),
        vec![0x6C; BULK_BYTES]
    );
}

fn assert_doomed_trap(
    engine: &Engine,
    module: &Module,
    name: &str,
    params: &[Value],
    expected: TrapCode,
) {
    let (mut store, instance) = instantiate(engine, module);
    let memory_baseline = store.data().memory_grows.len();
    let table_baseline = store.data().table_grows.len();
    store.add_fuel(INITIAL_FUEL).expect("initial fuel succeeds");
    let mut outputs = [Value::I32(0)];
    let error = exported_func(&store, instance, name)
        .call_resumable(&mut store, params, &mut outputs)
        .expect_err("internally doomed operation must fast-fail, not park");
    let actual = match &error {
        Error::Trap(trap) => trap
            .trap_code()
            .unwrap_or_else(|| panic!("{name} returned a host trap: {error}")),
        other => panic!("{name} returned a non-trap error: {other}"),
    };
    assert_eq!(
        std::mem::discriminant(&actual),
        std::mem::discriminant(&expected),
        "{name} trap code: {error}"
    );
    assert_eq!(store.data().memory_grows.len(), memory_baseline);
    assert_eq!(store.data().table_grows.len(), table_baseline);
}

#[test]
fn internally_doomed_ops_fast_fail_without_fuel_escrow_or_limiter_calls() {
    let engine = engine(true);
    let (module, _) = module(&engine);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory_baseline = store.data().memory_grows.len();
    let table_baseline = store.data().table_grows.len();
    let memory_grow = run_i32(
        &mut store,
        instance,
        "memory_grow",
        &[Value::I32(2)],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert!(memory_grow.parks.is_empty());
    assert_eq!(memory_grow.result, -1);
    assert_eq!(store.data().memory_grows.len(), memory_baseline);
    assert_eq!(store.data().table_grows.len(), table_baseline);

    let (mut store, instance) = instantiate(&engine, &module);
    let memory_baseline = store.data().memory_grows.len();
    let table_baseline = store.data().table_grows.len();
    let table_grow = run_i32(
        &mut store,
        instance,
        "table_grow",
        &[Value::I32(TABLE_MAX as i32)],
        INITIAL_FUEL,
        LARGE_REFILL,
    );
    assert!(table_grow.parks.is_empty());
    assert_eq!(table_grow.result, -1);
    assert_eq!(store.data().memory_grows.len(), memory_baseline);
    assert_eq!(store.data().table_grows.len(), table_baseline);

    assert_doomed_trap(
        &engine,
        &module,
        "memory_fill",
        &[
            Value::I32((MEMORY_BYTES - 4) as i32),
            Value::I32(0xAA),
            Value::I32(BULK_BYTES as i32),
        ],
        TrapCode::MemoryOutOfBounds,
    );
    assert_doomed_trap(
        &engine,
        &module,
        "memory_copy",
        &[
            Value::I32(1000),
            Value::I32((MEMORY_BYTES - 4) as i32),
            Value::I32(BULK_BYTES as i32),
        ],
        TrapCode::MemoryOutOfBounds,
    );
    assert_doomed_trap(
        &engine,
        &module,
        "memory_init",
        &[
            Value::I32(1000),
            Value::I32(BULK_BYTES as i32),
            Value::I32(BULK_BYTES as i32),
        ],
        TrapCode::MemoryOutOfBounds,
    );
    assert_doomed_trap(
        &engine,
        &module,
        "table_fill",
        &[Value::I32(1500), Value::I32(BULK_ELEMENTS as i32)],
        TrapCode::TableOutOfBounds,
    );
    assert_doomed_trap(
        &engine,
        &module,
        "table_copy",
        &[
            Value::I32(1500),
            Value::I32(0),
            Value::I32(BULK_ELEMENTS as i32),
        ],
        TrapCode::TableOutOfBounds,
    );
    assert_doomed_trap(
        &engine,
        &module,
        "table_init",
        &[
            Value::I32(0),
            Value::I32(1500),
            Value::I32(BULK_ELEMENTS as i32),
        ],
        TrapCode::TableOutOfBounds,
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CeilingTrace {
    parks: Vec<ParkStep>,
    granted_total: u64,
    fuel_consumed: u64,
    bytes: Vec<u8>,
    completed_effects: i32,
    end: &'static str,
}

fn run_with_granted_ceiling(engine: &Engine, module: &Module) -> CeilingTrace {
    const QUANTUM: u64 = 50;
    const CEILING: u64 = 120;

    let (mut store, instance) = instantiate(engine, module);
    let memory = exported_memory(&store, instance);
    let dst = 50_000;
    let sentinel = vec![0x33; BULK_BYTES];
    memory
        .write(&mut store, dst, &sentinel)
        .expect("ceiling sentinel write succeeds");
    let mut granted_total = QUANTUM.min(CEILING);
    store
        .add_fuel(granted_total)
        .expect("initial ceiling grant succeeds");
    let mut outputs = [Value::I32(0)];
    let mut outcome = exported_func(&store, instance, "memory_fill")
        .call_resumable(
            &mut store,
            &[
                Value::I32(dst as i32),
                Value::I32(0x55),
                Value::I32(BULK_BYTES as i32),
            ],
            &mut outputs,
        )
        .expect("ceiling workload starts");
    let mut parks = Vec::new();
    let end = loop {
        match outcome {
            ResumableCall::Finished => break "unexpectedly finished",
            ResumableCall::Resumable(invocation) => {
                assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
                parks.push(ParkStep {
                    consumed: store.fuel_consumed().expect("fuel enabled"),
                    completed_effects: read_i32(&store, memory, 0),
                });
                let Some(remaining) = CEILING
                    .checked_sub(granted_total)
                    .filter(|remaining| *remaining != 0)
                else {
                    break "WASI build fuel ceiling reached";
                };
                let grant = QUANTUM.min(remaining);
                store.add_fuel(grant).expect("bounded refill succeeds");
                granted_total = granted_total
                    .checked_add(grant)
                    .expect("granted total stays within the ceiling");
                outcome = invocation
                    .resume(&mut store, &[], &mut outputs)
                    .expect("ceiling workload resumes");
            }
        }
    };
    CeilingTrace {
        parks,
        granted_total,
        fuel_consumed: store.fuel_consumed().expect("fuel enabled"),
        bytes: read_bytes(&store, memory, dst, BULK_BYTES),
        completed_effects: read_i32(&store, memory, 0),
        end,
    }
}

#[test]
fn granted_total_ceiling_shortfall_is_deterministic_and_effect_free() {
    let engine = engine(true);
    let (module, _) = module(&engine);
    let first = run_with_granted_ceiling(&engine, &module);
    let second = run_with_granted_ceiling(&engine, &module);
    assert_eq!(first, second);
    assert_eq!(first.end, "WASI build fuel ceiling reached");
    assert_eq!(first.granted_total, 120);
    assert!(first.fuel_consumed < first.granted_total);
    assert_eq!(first.bytes, vec![0x33; BULK_BYTES]);
    assert_eq!(first.completed_effects, 0);
    assert!(first.parks.iter().all(|step| step.completed_effects == 0));
}

fn park_heavy_trace(engine: &Engine, module: &Module) -> (Vec<ParkStep>, usize, i32, u64) {
    let (mut store, instance) = instantiate(engine, module);
    let trace = run_i32(&mut store, instance, "park_heavy", &[], 50, 50);
    (
        trace.parks.clone(),
        trace.parks.len(),
        trace.result,
        trace.fuel_consumed,
    )
}

#[test]
fn park_heavy_double_run_has_identical_trace_and_resume_step_count() {
    let engine = engine(true);
    let (module, _) = module(&engine);
    let first = park_heavy_trace(&engine, &module);
    let second = park_heavy_trace(&engine, &module);
    assert_eq!(first, second);
    assert!(first.1 > 4, "workload must park repeatedly");
    assert_eq!(first.2, 4, "each of four effects applies exactly once");
}

#[test]
fn engine_rejects_resumable_fuel_with_eager_bulk_charging() {
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let mut config = Config::default();
        config
            .consume_fuel(true)
            .resumable_fuel(true)
            .fuel_consumption_mode(FuelConsumptionMode::Eager);
        Engine::new(&config)
    }))
    .expect_err("resumable fuel with eager charging must panic at engine construction");
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic");
    assert!(message.contains("resumable fuel requires lazy fuel consumption"));
}
