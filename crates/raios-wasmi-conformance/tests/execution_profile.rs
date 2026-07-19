use wasmi::{
    Config, Engine, ExecutionProfile, Linker, Module, Store, Suspension, TypedResumableCall,
};

const PROFILE_FIXTURE: &str = r#"
    (module
        (import "host" "noop" (func $noop))
        (func $leaf (param $value i32) (result i32)
            local.get $value
            i32.const 1
            i32.add)
        (func $worker (param $limit i32) (result i32)
            (local $iteration i32)
            (local $value i32)
            (loop $again
                local.get $value
                call $leaf
                local.set $value
                local.get $iteration
                i32.const 1
                i32.add
                local.tee $iteration
                local.get $limit
                i32.lt_u
                br_if $again)
            local.get $value)
        (func (export "run") (param $limit i32) (result i32)
            local.get $limit
            call $worker))
"#;

const LIMIT: i32 = 64;
const FUEL_QUANTUM: u64 = 25;
const EMPTY_PROFILE: ExecutionProfile = ExecutionProfile::empty();

#[derive(Debug)]
struct RunEvidence {
    result: i32,
    trace_digest: [u8; 32],
    fuel_consumed: u64,
    profile: ExecutionProfile,
}

fn digest_event(digest: &mut [u8; 32], event: u64) {
    for (index, byte) in event.to_le_bytes().into_iter().enumerate() {
        let lane = (index * 5 + event as usize) % digest.len();
        digest[lane] = digest[lane]
            .rotate_left(1)
            .wrapping_add(byte)
            .wrapping_add(index as u8);
    }
}

fn run_fixture(profile: bool) -> RunEvidence {
    let mut config = Config::default();
    config.consume_fuel(true).resumable_fuel(true);
    let engine = Engine::new(&config);
    let wasm = wat::parse_str(PROFILE_FIXTURE).expect("profile fixture WAT compiles");
    let module = Module::new(&engine, wasm.as_slice()).expect("profile fixture validates");
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("host", "noop", || {})
        .expect("noop import is unique");
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("profile fixture instantiates")
        .start(&mut store)
        .expect("profile fixture starts");
    let run = instance
        .get_typed_func::<i32, i32>(&store, "run")
        .expect("run export exists");
    if profile {
        store.start_execution_profiling();
    }
    store
        .add_fuel(FUEL_QUANTUM)
        .expect("initial fuel quantum installs");

    let mut trace_digest = [0_u8; 32];
    let mut suspensions = 0_u64;
    let mut outcome = run
        .call_resumable(&mut store, LIMIT)
        .expect("profile fixture call starts");
    let result = loop {
        match outcome {
            TypedResumableCall::Finished(result) => break result,
            TypedResumableCall::Resumable(invocation) => {
                assert!(matches!(invocation.suspension(), Suspension::FuelQuantum));
                suspensions += 1;
                digest_event(&mut trace_digest, suspensions);
                digest_event(
                    &mut trace_digest,
                    store
                        .fuel_consumed()
                        .expect("fixture fuel metering remains enabled"),
                );
                store
                    .add_fuel(FUEL_QUANTUM)
                    .expect("next fuel quantum installs");
                outcome = invocation
                    .resume(&mut store, &[])
                    .expect("fuel quantum resumes without values");
            }
        }
    };
    digest_event(&mut trace_digest, result as u64);
    digest_event(&mut trace_digest, suspensions);
    RunEvidence {
        result,
        trace_digest,
        fuel_consumed: store
            .fuel_consumed()
            .expect("fixture fuel metering remains enabled"),
        profile: store.execution_profile(),
    }
}

fn count(profile: &ExecutionProfile, function_index: u32) -> u64 {
    profile
        .entries()
        .iter()
        .find(|entry| entry.function_index() == function_index)
        .map_or(0, |entry| entry.count())
}

#[test]
fn execution_profile_is_side_channel_only_and_attributes_module_indices() {
    let control = run_fixture(false);
    let profiled = run_fixture(true);

    assert_eq!(profiled.result, LIMIT);
    assert_eq!(
        profiled.result, control.result,
        "profiling changed the result"
    );
    assert_eq!(
        profiled.trace_digest, control.trace_digest,
        "profiling changed the fuel-suspension trace digest"
    );
    assert_eq!(
        profiled.fuel_consumed, control.fuel_consumed,
        "profiling changed fuel accounting"
    );
    assert_eq!(control.profile.total_samples(), 0);
    assert_eq!(control.profile, EMPTY_PROFILE);
    assert!(
        control.profile.entries().is_empty(),
        "the default-disabled profiler must record nothing"
    );

    // The imported noop occupies index 0; definitions are leaf=1, worker=2,
    // exported run=3. This proves reported indices use the module index space.
    assert!(
        count(&profiled.profile, 1) >= LIMIT as u64,
        "every leaf entry must be attributed to function index 1"
    );
    assert!(
        count(&profiled.profile, 2) > 0,
        "worker execution must be attributed to function index 2"
    );
    assert!(
        count(&profiled.profile, 3) > 0,
        "root execution must be attributed to function index 3"
    );
    assert_eq!(
        profiled.profile.total_samples(),
        profiled
            .profile
            .entries()
            .iter()
            .map(|entry| entry.count())
            .sum(),
        "the exact, non-overflowed fixture profile must account for every sample"
    );
}
