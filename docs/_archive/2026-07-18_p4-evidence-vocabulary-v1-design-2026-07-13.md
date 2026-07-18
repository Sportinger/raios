# P4-DESIGN — Evidence Vocabulary v1

Read-only design. No files changed, committed, pushed, or merged.

## 1. Vocabulary v1 record grammar

### Decision

ADR 0006 is reopened. Its byte-identical floor remains historical evidence, but its decision is superseded for emitted protocol vocabulary:

> All surviving agent-protocol responses use evidence vocabulary v1: shared typed envelopes, ordered evidence and decision records, field-table construction, and the existing single `record::Value` serializer/hasher.

This does not create another serializer. `raios_core::record::write_json()` remains the only JSON renderer, and `sha256_of_json()` continues hashing the exact same `Value` tree.

### Shared response shape

Every method returns one top-level shape:

```json
{
  "schema": "raios.evidence_response.v1",
  "id": "response.current_boot.00000042",
  "family": "module.load_gate",
  "scope": "current_boot",
  "classification": "local_only",
  "source_method": "module.load_gate",
  "event_id": "event.current_boot.00000042",
  "facts": {},
  "evidence": [],
  "decision": {}
}
```

Fields are always ordered as shown. Nullable values are emitted as `null`; optional fields are not silently omitted.

Shared envelope fields:

| Field | Type | Source |
|---|---|---|
| `schema` | static string | vocabulary constant |
| `id` | typed current-boot ID | response/event sequence |
| `family` | static string | method table |
| `scope` | enum string | evaluator/snapshot; normally `current_boot` |
| `classification` | enum string | record classification |
| `source_method` | static string | dispatch entry |
| `event_id` | event ID or null | kernel-acquired event |
| `facts` | object | family field table |
| `evidence` | ordered array | evaluator evidence sequence |
| `decision` | typed decision | evaluator result |

Hashes remain `Value::Sha256`; event IDs remain `Value::EventSequence`. Existing exact eight-digit current-boot grammar is unchanged.

### Evidence record

```json
{
  "id": "module.manifest_reference",
  "kind": "reference",
  "status": "verified",
  "reason": "retained_hash_reference_only",
  "source_event_id": "event.current_boot.00000012",
  "classification": "local_only",
  "facts": {
    "reference_hash": "sha256:...",
    "subject_hash": "sha256:..."
  }
}
```

The fixed portion is described once. Family tables describe only `facts`.

Statuses use one vocabulary:

```text
present | missing | rejected | verified | unavailable | not_applicable
```

Existing family-specific statuses remain available under `facts.status_detail` where they carry meaning not represented by the common status.

### Decision and denial records

Observational methods:

```json
{
  "outcome": "observed",
  "reason": "snapshot_returned"
}
```

Denied authority:

```json
{
  "outcome": "denied",
  "reason": "candidate_artifact_missing",
  "requested_capability": "cap.module.load_ephemeral",
  "grants": [],
  "effects": [],
  "blocked_by": [
    {
      "evidence_id": "candidate_artifact",
      "status": "missing",
      "reason": "candidate_artifact_missing"
    }
  ]
}
```

Granted authority:

```json
{
  "outcome": "granted",
  "reason": "scoped_policy_granted",
  "requested_capability": "cap.example",
  "grants": ["cap.example"],
  "effects": ["scoped_effect"]
}
```

`blocked_by` preserves evaluator order. It is never sorted by the renderer.

Scattered fields such as `can_load`, `authorizes_load`, `load_attempted`, and `service_inventory_change` move into the decision/effects representation. Every old field must have an entry in a family semantic mapping:

```text
old JSON path -> v1 JSON path, constant invariant, or intentionally retired redundancy
```

No old semantic fact may simply disappear.

### Field-table declaration

The minimum new substrate belongs in `raios-core/src/record.rs` or a small sibling `record_table.rs`:

```rust
pub enum Cell<'a> {
    Null,
    Bool(bool),
    U64(u64),
    Str(&'a str),
    Sha256([u8; 32]),
    EventSequence(u64),
    Value(Value<'a>),
}

pub struct FieldSpec<T> {
    pub key: &'static str,
    pub get: for<'a> fn(&'a T) -> Cell<'a>,
}

pub fn object_from<T>(table: &[FieldSpec<T>], source: &T) -> Value<'_>;
```

Declarations remain typed and local to their owning evaluator:

```rust
const REFERENCE_FIELDS: &[FieldSpec<ReferenceView>] = &[
    field!("state", str <- ReferenceView::state),
    field!("validation_status", str <- ReferenceView::status),
    field!("validation_reason", str <- ReferenceView::reason),
    field!("reference_hash", sha256_opt <- ReferenceView::reference_hash),
];
```

The `field!` macro only checks accessor/output compatibility and removes boilerplate. It must not contain policy or conditional evaluation.

Nested objects use another table or an existing `Value` accessor. Arrays are produced by typed projection functions because their lengths and ordering are runtime data.

### Emission driver

The complete path becomes:

```text
kernel acquires live snapshot/event IDs
    -> raios-core evaluator returns typed projection/decision
    -> field tables build one record::Value tree
    -> record::write_json writes that tree
```

There is one generic table walker and the existing one generic JSON renderer. No family may call `raw()`, `json_str()`, or `raw_bool()` inside a JSON object after conversion.

### What stays hand-written

Only transport framing remains outside the record tree:

- `RAIOS_AGENT_BEGIN <method>`
- `RAIOS_AGENT_END <method>`
- CRLF between framing and JSON
- serial backpressure and bounded writes
- response chunking where a payload exceeds the serial buffer
- command parsing and dispatch
- kernel acquisition of locks, devices, stores, event IDs, and mutable state

Chunk boundaries are transport details and must not affect the `Value` tree or its hash.

### Family mapping

| Family | `facts` content | Evidence units |
|---|---|---|
| Module references | request/reference/retained hashes | manifest, artifact, report, attestation, approval |
| Load gate | requested load and retained snapshot | one ordered prerequisite per gate input |
| Loader runtime/facts | descriptor, mapping, entrypoint, registry facts | one item per execution boundary |
| Event | event payload and ring metadata | binding/source evidence |
| Memory | profile, budget, included/omitted records | record locators and omission reasons |
| Provider | public provider posture only | trust, key-presence classification, export gate |
| Hello lifecycle | state, migration, probation, health | descriptor, artifact, state-transition evidence |
| Rollback/write boundary | target, layout, append/readback facts | policy, storage, append, inspection, scoped apply |
| System/status | device/service/problem/status facts | provenance/event locators where applicable |

## 2. Semantic-equivalence and golden regeneration

### Host-test contract

Before replacing a family, capture a semantic manifest for every old response path. Each Cargo test must prove:

1. Every old semantic field maps to a v1 field, a constant invariant, or an explicitly approved redundant-field retirement.
2. The v1 value comes from the same snapshot or evaluator accessor as the old value.
3. Present, missing, rejected, and substituted fixtures produce the same values.
4. The final status and reason match the evaluator.
5. First-failure and `blocked_by` ordering match the evaluator’s existing order.
6. All hashes use the existing canonical input grammar; response-vocabulary changes do not alter authority hashes.
7. Null versus present semantics remain explicit.
8. Classification, scope, and event provenance remain unchanged.
9. Denied cases emit no grants or effects.
10. Granted cases require an evaluator-created grant proof.

Each family leaves:

- exhaustive evaluator tables in `raios-core` tests;
- table-rendering tests for envelope, facts, evidence, and decision shapes;
- one test iterating the semantic mapping and failing on unmapped old fields;
- a small number of guest integration cases proving kernel acquisition and framing.

The known non-functional `substituted_local_approval_reference_record` fixture must be corrected or explicitly retired before its family closes; appearance-only coverage is not semantic evidence.

### Golden procedure per slice

1. Inventory affected predicates by command and old JSON path.
2. Commit a host-side semantic manifest before deleting the old emitter.
3. Run `cargo test -p raios-core` against old fixtures and new v1 projections.
4. Switch one response family.
5. Regenerate only that family’s harness expectations.
6. Run the named focused VM profile.
7. Inspect the serial transcript and report; do not bulk accept replacements.
8. Record:
   - renamed schemas;
   - moved paths;
   - retired redundant fields;
   - unchanged reasons and ordering;
   - affected predicate names.
9. Run source-size, format, and secret checks at slice close.

### Harness impact

Needle predicates fall into three groups:

- Leaf needles such as `"reason": "candidate_artifact_missing"` survive when the leaf pair is unchanged.
- Schema, nesting, and redundant-authority-field needles must be regenerated.
- BEGIN/END and command-completion predicates survive unchanged.

Exact-shape assertions include:

- `ConvertFrom-Json` property-path comparisons;
- compact object/string comparisons;
- field-count or array-order assertions;
- response or payload SHA-256 goldens.

These must be rewritten against v1 paths. Persistent-record, rollback-log, artifact, descriptor, and authority hashes are not regenerated merely because the serial response changed.

Estimated runtime predicate churn:

| Family | Predicates reviewed | Likely regenerated |
|---|---:|---:|
| Module reference/evidence | 350–550 | 180–320 |
| Load gate | 400–650 | 250–450 |
| Loader facts/runtime | 300–500 | 180–350 |
| Event evidence | 120–220 | 70–140 |
| Memory | 180–320 | 110–220 |
| Hello lifecycle | 140–260 | 90–180 |
| Rollback/write boundary | 300–500 | 200–360 |
| Provider | 70–140 | 40–90 |
| System/status remainder | 150–300 | 80–180 |
| **Total** | **2,010–3,440** | **1,200–2,290** |

These are execution estimates, not static PowerShell occurrence counts. Each slice must generate its exact predicate inventory before editing.

## 3. Slice cut

| Slice | Files/ownership | Estimated kernel delta | Focused profile | Golden scope | Risk |
|---|---|---:|---|---|---|
| P4-0 substrate | `raios-core/src/record.rs`, optional `record_table.rs`, support wrappers | +200 to +450 | Cargo tests + `quick` | synthetic renderer goldens only | Low |
| P4-1 module references/evidence | `agent_protocol_module_reference.rs`, approval/attestation/grant/audit/service-slot files, `module_evidence.rs`, module types | −2.2k to −3.4k | `module-audit-rollback` | evidence + selftest fragments | Medium; fold useful W3 immutable evidence/DTO relocation |
| P4-2 load-gate render | `agent_protocol_module_load_gate_render.rs`, selftest emit/eval, load-gate facade | −4.5k to −5.5k | `module-audit-rollback` plus load-gate fragment | load-gate and load-gate selftests | High; large ordered denial surface |
| P4-3 allocator, loader facts/runtime | allocator/projection, loader identity/hash/fact files, `agent_protocol_module_loader_runtime*` | −5k to −9k | load-gate fragment / `module-audit-rollback` | allocator, loader facts, runtime | High; fold W4–W5 pure evaluators, never mapping/apply |
| P4-4 event evidence | `event_log.rs`, `event_log_types.rs`, event evidence and family binding projections | −2.5k to −5.5k | `quick`, then `provider-memory` | event and binding paths | High; fold W7 ring/evidence relocation, kernel retains mutex |
| P4-5 memory | `agent_protocol_memory.rs`, existing `memory_context`/`memory_record` modules | −3.5k to −6k | `memory-durable` | profile/context/query/trace/recent events | Medium-high; fold W8 projections, preserve durable denial wiring |
| P4-6 hello lifecycle | `hello_service/emitters.rs`, lifecycle/model/preflight/state files | −2.2k to −4k | `quick` | lifecycle, migration, probation, health | High; fold non-rollback W9 model relocation and regenerate hello source attestation |
| P4-7 rollback and write authority | `hello_service` rollback emitters/foundation plus all `agent_protocol_module_write_boundary*` | −5.5k to −9k | `module-audit-rollback`, then `m6d-rollback` and rollback dry-run | write boundary and rollback | Critical; fold W6 only, keep I/O/apply in kernel |
| P4-8 provider | `agent_protocol_provider.rs`, existing `provider_context` and `scoped_provider_export` | −0.7k to −1.5k | `provider-memory` | provider and provider-memory | High classification risk; fold provider part of W9 |
| P4-9 system/status remainder | surviving system/device/service/problem/status/event emitters and common support | −1.5k to −3.5k | `quick` | common/full-audit remainder | Medium; remove last JSON `raw()` sites and obsolete support helpers |

No slice creates a generic policy evaluator. P4-7 remains isolated because it touches real storage, rollback, and authority.

Full plus recovery close points:

- After P4-3: module/load block close.
- After P4-7: authority and rollback block close.
- After P4-9: vocabulary-v1 phase close.

Every ordinary slice otherwise runs only its named focused profile, matching the owner’s aggressive-fast cadence.

## 4. Fail-closed type guarantees

A free-form `FieldSpec` must not be able to emit authority.

The v1 model separates facts from decisions:

```rust
pub enum EvaluatedDecision<'a> {
    Observed(&'a Observation),
    Denied(&'a DenialDecision),
    Granted(&'a GrantDecision),
}
```

Required constraints:

- `DenialDecision` and `GrantDecision` constructors are private to their evaluator modules.
- A `GrantDecision` contains an unforgeable `GrantProof` returned only by the evaluator’s positive branch.
- Family field tables can build only `facts` and `evidence`.
- Reserved decision keys—`outcome`, `grants`, `effects`, `blocked_by`, and all `authorizes_*` concepts—cannot appear in `FieldSpec`.
- The generic driver owns decision rendering:
  - `Denied` always emits `grants: []` and `effects: []`;
  - `Granted` derives grants/effects only from `GrantProof`;
  - `Observed` cannot carry grants.
- A denial record accepts an ordered evaluator-produced `DeniedBy` slice. Emitters cannot reconstruct or reorder reasons.
- Kernel acquisition code cannot construct `GrantProof`.
- Deserialization or externally supplied strings never become a decision type.

Thus a table can lie only about descriptive facts if its accessor is wrong; it cannot turn a denial into authority. Same-source Cargo tests cover the descriptive accessor mapping.

## 5. Projected line trajectory

Baseline requested by the packet: **176,331 physical lines in `seed-kernel/src`**.

The old W3–W9 reductions are not reliable forecasts after `0a3a66d`; W2a demonstrated that much apparent duplication is distinct emission vocabulary. Therefore the trajectory separates firm P4 opportunity from optional relocation credit.

| Close | Planning center |
|---|---:|
| Baseline | 176,331 |
| P4-0 substrate | 176,681 |
| P4-1 references/evidence | 173,881 |
| P4-2 load gate | 168,881 |
| P4-3 loader facts/runtime | 161,881 |
| P4-4 event evidence | 157,881 |
| P4-5 memory | 153,181 |
| P4-6 hello lifecycle | 150,081 |
| P4-7 rollback/write boundary | 142,881 |
| P4-8 provider | 141,781 |
| P4-9 system/status close | **139,281** |

Forecast bands:

- P4 vocabulary replacement alone, using the measured preview: **149,681–155,181**.
- Combined P4 plus only evaluator relocations proven useful while touching each family: planning center **~139,300**.
- Honest combined band: **~129,000–155,000**.

The lower end requires substantial W3–W9 evaluator/test relocation to remain genuinely deletive after adapters are added. It must not be claimed in advance. Record the measured line count after every slice and reforecast after P4-3 and P4-7.

The `≤120,000` target is not a credible P4 promise from the current evidence. If P4 closes above it, the next reduction must come from measured ownership moves—not another vocabulary layer or broader generic framework.