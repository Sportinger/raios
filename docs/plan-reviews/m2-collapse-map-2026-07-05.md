# M2 Collapse Map (2026-07-05)

Read-only scoping analysis (packet M2-24): line-mass census of the
~105.6k agent-layer lines, the four collapse design sketches
(dispatch table, shared CommandBindings, generic selftest runner, event
binding types), a 6-batch plan, and the reality check — byte-identical
collapse bottoms out around 55-75k lines; reaching ~20k requires
output-shape/vocabulary changes (batch 6, OWNER DECISION required, needs
harness needle updates and arguably an ADR).

**Census**
Approx nonblank LOC for `agent_protocol*` plus [event_log_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:143): ~105.6k.

| Family | LOC | Main mass |
|---|---:|---|
| recovery | 40.7k | eval 14.7k, emit 10.1k, selftest data 9.1k, dispatch/types/scaffolding 6.8k |
| module_loader | 12.9k | eval 7.3k, emit 3.4k, selftest data 1.7k |
| module_load_gate | 10.7k | emit 3.6k, selftest data 2.6k, scaffolding 2.1k, eval 2.1k |
| module_evidence | 9.2k | eval 2.4k, emit 1.9k, types 1.8k, selftest data 1.1k |
| write_boundary | 8.5k | eval 3.1k, emit 2.9k, selftest data 2.0k |
| memory | 8.3k | mostly event-log binding emit |
| service_slot | 6.1k | eval 3.1k, emit 2.0k |
| event_log_types | 3.8k | mostly structs/enums |
| provider | 2.2k | emit/eval/context gates |
| system/support/dispatch | 3.1k | dispatch/plumbing + small emit |

Overlapping field-threading mass: ~7-10k. The concrete evidence is 48 large command structs, 1.25k lines just for their field declarations, 30 positional reference parsers, 435 `parts.next()` calls, and repeated check constructors.

**Top Heavy Patterns**
1. Flattened event binding emitter: [agent_protocol_memory.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:484), `emit_event_bindings`, ~7.1k lines; Hello branch alone ~4.6k.
2. Flattened `HelloServiceLifecycleBinding`: [event_log_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:143), ~1.9k field lines, plus [lifecycle_binding.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/lifecycle_binding.rs:3), ~1.2k constructor lines.
3. Hello rollback writer storage chain: [storage_authority_gate.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/storage_authority_gate.rs:3), ~3.6k hash lines; [rollback_writer_bindings.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_bindings.rs:129), ~2.9k binder lines; [emitters.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/emitters.rs:600), ~2.3k emitter lines.
4. Recovery selftest case factories: [agent_protocol_recovery_command_reference_selftests.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_recovery_command_reference_selftests.rs:9), ~9.1k recovery case data.
5. Module load-gate selftest family: [agent_protocol_module_load_gate_selftest_emit.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_selftest_emit.rs:20), ~4.7k data/scaffolding.
6. Module loader runtime eval/source-evidence matrix: [agent_protocol_module_loader_runtime.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_loader_runtime.rs:8008), ~7.3k eval.
7. Positional command-reference parsers: [agent_protocol_recovery_command_effect_reference_eval.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_recovery_command_effect_reference_eval.rs:14), ~2.1k parser lines plus copied field checks.
8. Per-stage command structs: [agent_protocol_recovery_command_effect_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_recovery_command_effect_types.rs:4), [agent_protocol_recovery_command_dispatch_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs:10), 48 large structs.
9. Agent dispatch chain: [agent_protocol.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol.rs:292), 168 branches, ~933-line function; [console.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/console.rs:1296), 28 command arms.
10. Method predicate helpers: 150 `*_method()` helpers under `agent_protocol*`, plus 15 Hello `is_*_method()` helpers in [runtime.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/runtime.rs:86).

**Design Sketches**
Dispatch table:

```rust
type MethodHandler = fn(MethodCall<'_>, ui::RuntimeStatus) -> DispatchOutcome;

struct MethodEntry {
    canonical: &'static str,
    aliases: &'static [&'static str],
    match_kind: MatchKind,
    envelope_capability: Option<&'static str>,
    envelope_response_id: Option<&'static str>,
    handler: MethodHandler,
}

enum MatchKind {
    Exact,
    Head,
    Predicate(fn(&str) -> Option<&'static str>),
}
```

`dispatch()` becomes a loop over `AGENT_METHODS`. `console.rs` command aliases and the 19-entry command-envelope target table should point at the same entries. Deletes the 168-branch chain, most duplicate console routing, and many method predicate helpers. Estimate: 2-3k lines.

Shared command bindings:

```rust
struct CommandBindings<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    command_id: Option<&'a str>,
    argument_schema: Option<&'a str>,
    argument_hash: Option<[u8; 32]>,
    target_locator: Option<&'a str>,
    command_envelope_reference_hash: Option<[u8; 32]>,
    command_body_canonicalization_hash: Option<[u8; 32]>,
    handler_binding_hash: Option<[u8; 32]>,
    status_read_handler_hash: Option<[u8; 32]>,
    rollback_preview_authorization_hash: Option<[u8; 32]>,
    rollback_apply_authorization_hash: Option<[u8; 32]>,
    disable_module_target_binding_hash: Option<[u8; 32]>,
    restart_last_good_target_binding_hash: Option<[u8; 32]>,
    load_artifact_by_hash_target_binding_hash: Option<[u8; 32]>,
    recovery_memory_write_authority_hash: Option<[u8; 32]>,
    durable_audit_rollback_write_authority_hash: Option<[u8; 32]>,
    service_inventory_side_effect_boundary_hash: Option<[u8; 32]>,
    command_dispatch_behavior_hash: Option<[u8; 32]>,
    executor_capability_table_hash: Option<[u8; 32]>,
    side_effect_gate_hash: Option<[u8; 32]>,
}

struct StageBinding<'a> {
    stage_hash: Option<[u8; 32]>,
    expected_hash: Option<[u8; 32]>,
    retained_previous_event_id: Option<&'a str>,
    stage_id: Option<&'a str>,
    projection_hash: Option<[u8; 32]>,
}
```

Replace the `Recovery*Input` / `Recovery*ReferenceCheck` field clones with `CommandBindings + StageBinding + status/reason`. Add one named `key=value` parser, not 30 positional parsers. Estimate: 8-12k deletion.

Selftest collapse:

```rust
struct CaseSpec {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CaseMutation,
    require_live_retained: bool,
}

struct SelftestSpec<I, C> {
    method: &'static str,
    schema: &'static str,
    base_input: fn() -> I,
    apply_case: fn(&mut I, CaseMutation),
    eval: fn(I, bool) -> C,
    case_record: fn(&CaseSpec, C) -> Value<'static>,
}
```

Cases become const data tables. One runner builds valid input, applies the mutation, evaluates, and emits a shared report. Estimate: recovery -6 to -8k, module load-gate -3k, write-boundary -1.5k, other module/provider families -2k; total ~12-16k.

Types collapse assessment: record-based Hello lifecycle is feasible byte-identically, but do not store `Value<Vec<_>>` directly inside `Event`; the ring is currently `Copy`. Use compact typed subrecords or a field-descriptor emitter that preserves key order and old key/value hash conventions. This can delete a lot, but it is high-risk because [agent_protocol_memory.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:487) and [event_log_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:143) are huge and output-sensitive.

**Batch Plan**
1. Dispatch table + shared console/envelope metadata. Delete 2-3k. Verify future: `cargo test -p raios-core`, then quick profile.
2. Named args + `CommandBindings` for recovery command references. Delete 8-12k. Verify: host parser tests, recovery profile.
3. Generic selftest runner, recovery first then module/load/write. Delete 12-16k. Verify each focused profile, full after batch.
4. Event binding emitter/type collapse. Delete 8-14k. Verify quick + hello-rollback-dry-run + full.
5. Hello rollback writer/hash field tables. Delete 6-10k. Verify hello-rollback-dry-run and full because rollback/storage authority is risky.
6. Optional non-byte-identical vocabulary compaction. Delete 30k+, but requires harness needle updates.

**Reality Check**
Byte-identical collapse probably bottoms out around 55-75k agent-layer lines. Getting near ~20k requires changing output shape or vocabulary: compact event bindings, parameterized `boundary.inspect name=...`, moving most negative selftests to host tests, and deleting denial strata rather than representing them byte-for-byte.

Skipped: writing `-o` report file, git status, QEMU/smoke, cargo fmt, because this was explicitly read-only.