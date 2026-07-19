use wasmi::{
    errors::{MemoryError, TableError},
    Config, Engine, Instance, Linker, Memory, Module, ResourceLimiter, Store, Suspension,
    TypedResumableCall,
};

const BURN_GUEST: &str = r#"
    (module
        (func (export "burn") (param $limit i32) (result i32)
            (local $iteration i32)
            (block $done
                (loop $again
                    local.get $iteration
                    local.get $limit
                    i32.ge_u
                    br_if $done

                    local.get $iteration
                    i32.const 1
                    i32.add
                    local.set $iteration
                    br $again))
            local.get $iteration))
"#;

const MEMORY_GROW_GUEST: &str = r#"
    (module
        (memory (export "memory") 1 3)
        (func (export "grow") (result i32)
            i32.const 1
            memory.grow))
"#;

fn metered_engine(resumable: bool) -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true).resumable_fuel(resumable);
    Engine::new(&config)
}

fn burn_fixture() -> (Store<()>, Instance) {
    let engine = metered_engine(true);
    let wasm = wat::parse_str(BURN_GUEST).expect("burn fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("burn fixture validates");
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("burn fixture instantiates")
        .start(&mut store)
        .expect("burn fixture starts");
    (store, instance)
}

fn burn_one_slice(
    store: &mut Store<()>,
    instance: Instance,
    escrow: u64,
    iterations: i32,
) -> (u64, u64) {
    assert_eq!(
        store
            .replace_remaining_fuel(escrow)
            .expect("thread escrow installs"),
        0,
        "the previous thread must have swept its escrow"
    );
    let consumed_before = store.fuel_consumed().expect("fuel metering is enabled");
    let result = instance
        .get_typed_func::<i32, i32>(&*store, "burn")
        .expect("burn export exists")
        .call(&mut *store, iterations)
        .expect("slice has sufficient escrow");
    assert_eq!(result, iterations);
    let consumed_after = store.fuel_consumed().expect("fuel metering is enabled");
    let residual = store
        .replace_remaining_fuel(0)
        .expect("thread escrow sweeps out");
    assert_eq!(
        store.fuel_consumed(),
        Some(consumed_after),
        "sweeping an escrow must not fabricate consumption"
    );
    let actual_consumption = consumed_after
        .checked_sub(consumed_before)
        .expect("consumption is monotonic");
    assert_eq!(
        escrow - residual,
        actual_consumption,
        "only executed Wasm may reduce the installed escrow"
    );
    (residual, actual_consumption)
}

#[test]
fn remaining_fuel_swap_requires_resumable_metering() {
    let default_engine = Engine::new(&Config::default());
    let mut default_store = Store::new(&default_engine, ());
    assert!(default_store.replace_remaining_fuel(17).is_err());
    assert_eq!(default_store.fuel_consumed(), None);

    let metered_engine = metered_engine(false);
    let mut metered_store = Store::new(&metered_engine, ());
    assert!(metered_store.replace_remaining_fuel(17).is_err());
    assert_eq!(metered_store.fuel_consumed(), Some(0));
}

#[test]
fn swap_preserves_fuel_consumed_and_clock_delta_tracks_only_execution() {
    const ESCROW: u64 = 50_000;
    const FRESH_ESCROW: u64 = 80_000;

    let (mut store, instance) = burn_fixture();
    let consumed_before = store.fuel_consumed().expect("fuel metering is enabled");
    assert_eq!(
        store
            .replace_remaining_fuel(ESCROW)
            .expect("escrow installs"),
        0
    );
    assert_eq!(store.fuel_consumed(), Some(consumed_before));

    let result = instance
        .get_typed_func::<i32, i32>(&store, "burn")
        .expect("burn export exists")
        .call(&mut store, 32)
        .expect("burn call has sufficient escrow");
    assert_eq!(result, 32);

    let consumed_after_execution = store.fuel_consumed().expect("fuel metering is enabled");
    let remaining = store
        .replace_remaining_fuel(0)
        .expect("remaining escrow sweeps out");
    let logical_delta = ESCROW - remaining;
    assert!(logical_delta > 0 && logical_delta < ESCROW);
    assert_eq!(
        consumed_after_execution - consumed_before,
        logical_delta,
        "the fuel-derived clock counter is total-minus-remaining and may advance only by execution"
    );
    assert_eq!(
        store.fuel_consumed(),
        Some(consumed_after_execution),
        "sweeping remaining fuel must not move the clock-relevant counter"
    );

    assert_eq!(
        store
            .replace_remaining_fuel(FRESH_ESCROW)
            .expect("fresh escrow installs"),
        0
    );
    assert_eq!(
        store.fuel_consumed(),
        Some(consumed_after_execution),
        "the fresh escrow amount must not appear as elapsed logical fuel"
    );
    assert_eq!(
        store
            .replace_remaining_fuel(0)
            .expect("fresh escrow sweeps out"),
        FRESH_ESCROW
    );
    assert!(
        store.replace_remaining_fuel(u64::MAX).is_err(),
        "an unrepresentable accounting rebase must be rejected"
    );
    assert_eq!(store.fuel_consumed(), Some(consumed_after_execution));
    assert_eq!(store.replace_remaining_fuel(0).unwrap(), 0);
}

#[test]
fn remaining_fuel_swap_round_trips_exact_values() {
    let engine = metered_engine(true);
    let mut store = Store::new(&engine, ());

    assert_eq!(store.replace_remaining_fuel(111).unwrap(), 0);
    assert_eq!(store.replace_remaining_fuel(222).unwrap(), 111);
    assert_eq!(store.replace_remaining_fuel(111).unwrap(), 222);
    assert_eq!(store.replace_remaining_fuel(0).unwrap(), 111);
    assert_eq!(store.fuel_consumed(), Some(0));
}

#[test]
fn alternating_thread_escrows_resume_only_their_own_residual() {
    const THREAD_A_ESCROW: u64 = 40_000;
    const THREAD_B_ESCROW: u64 = 60_000;

    let (mut store, instance) = burn_fixture();
    let (thread_a_after_first, thread_a_first_spend) =
        burn_one_slice(&mut store, instance, THREAD_A_ESCROW, 11);
    let (thread_b_after_first, thread_b_first_spend) =
        burn_one_slice(&mut store, instance, THREAD_B_ESCROW, 23);
    assert_eq!(thread_a_after_first, THREAD_A_ESCROW - thread_a_first_spend);
    assert_eq!(thread_b_after_first, THREAD_B_ESCROW - thread_b_first_spend);

    let (thread_a_after_second, thread_a_second_spend) =
        burn_one_slice(&mut store, instance, thread_a_after_first, 17);
    let (thread_b_after_second, thread_b_second_spend) =
        burn_one_slice(&mut store, instance, thread_b_after_first, 5);
    assert_eq!(
        thread_a_after_second,
        thread_a_after_first - thread_a_second_spend,
        "thread A must resume from exactly A's swept residual"
    );
    assert_eq!(
        thread_b_after_second,
        thread_b_after_first - thread_b_second_spend,
        "thread B must resume from exactly B's swept residual"
    );
    assert_ne!(thread_a_after_second, thread_b_after_second);
}

#[derive(Debug, Default)]
struct GrowState {
    memory_grows: Vec<(usize, usize, Option<usize>)>,
}

impl ResourceLimiter for GrowState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, MemoryError> {
        self.memory_grows.push((current, desired, maximum));
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> Result<bool, TableError> {
        Ok(true)
    }
}

fn memory_pages(memory: Memory, store: &Store<GrowState>) -> u32 {
    u32::from(memory.current_pages(store))
}

#[test]
fn memory_grow_banks_swapped_quantums_without_fabricated_consumption() {
    const QUANTUM: u64 = 100;
    const MAX_PARKS: usize = 32;

    let engine = metered_engine(true);
    let wasm = wat::parse_str(MEMORY_GROW_GUEST).expect("memory.grow fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("memory.grow fixture validates");
    let mut store = Store::new(&engine, GrowState::default());
    store.limiter(|state| state);
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .expect("memory.grow fixture instantiates")
        .start(&mut store)
        .expect("memory.grow fixture starts");
    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export exists");
    let grow = instance
        .get_typed_func::<(), i32>(&store, "grow")
        .expect("grow export exists");
    let limiter_baseline = store.data().memory_grows.len();

    let mut granted_total = QUANTUM;
    assert_eq!(store.replace_remaining_fuel(QUANTUM).unwrap(), 0);
    let mut outcome = grow
        .call_resumable(&mut store, ())
        .expect("memory.grow starts resumably");
    let mut parks = 0;
    let result = loop {
        match outcome {
            TypedResumableCall::Finished(result) => break result,
            TypedResumableCall::Resumable(invocation) => {
                assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
                parks += 1;
                assert!(parks <= MAX_PARKS, "memory.grow exceeded the park bound");
                assert_eq!(memory_pages(memory, &store), 1);
                assert_eq!(
                    store.data().memory_grows.len(),
                    limiter_baseline,
                    "the memory.grow limiter call and effect must wait for the full dynamic charge"
                );

                let consumed_at_park = store.fuel_consumed().expect("fuel metering is enabled");
                let residual = store
                    .replace_remaining_fuel(0)
                    .expect("parked escrow sweeps out");
                assert_eq!(store.fuel_consumed(), Some(consumed_at_park));
                let banked = residual
                    .checked_add(QUANTUM)
                    .expect("bounded test escrow cannot overflow");
                granted_total += QUANTUM;
                assert_eq!(store.replace_remaining_fuel(banked).unwrap(), 0);
                assert_eq!(
                    store.fuel_consumed(),
                    Some(consumed_at_park),
                    "banking a quantum outside the store must not advance logical fuel"
                );
                outcome = invocation
                    .resume(&mut store, &[])
                    .expect("banked memory.grow resumes");
            }
        }
    };

    assert_eq!(result, 1);
    assert!(parks > 1, "the test must prove multi-round quantum banking");
    assert_eq!(memory_pages(memory, &store), 2);
    assert_eq!(store.data().memory_grows.len(), limiter_baseline + 1);
    assert_eq!(
        store.data().memory_grows.last(),
        Some(&(65_536, 131_072, Some(196_608)))
    );

    let consumed_after_execution = store.fuel_consumed().expect("fuel metering is enabled");
    let final_residual = store
        .replace_remaining_fuel(0)
        .expect("completed escrow sweeps out");
    assert_eq!(store.fuel_consumed(), Some(consumed_after_execution));
    assert_eq!(
        consumed_after_execution,
        granted_total - final_residual,
        "only actual block and bulk charges consume the externally granted escrow"
    );
}
