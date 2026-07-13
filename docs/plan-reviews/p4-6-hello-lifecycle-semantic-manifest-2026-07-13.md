# P4-6a — Hello lifecycle semantic manifest

Read-only inventory for P4-6b. This packet changes no source, harness, descriptor,
signature, build, status, roadmap, or release file.

Notation:

- `R` = legacy response `body.result` (or legacy error `body`).
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

The P4-6 boundary is lifecycle, migration, probation, health, and their
descriptor/artifact/state-transition evidence. Rollback preview, rollback apply,
recovery rollback inspection/materialization, durable append/write foundations,
and every `agent_protocol_module_write_boundary*` surface are P4-7.

## 1. Response-path inventory

### Files and ownership boundary

The in-scope Hello files and portions are:

| File | P4-6 ownership |
|---|---|
| `hello_service/command_targets.rs` | built-in/host-bound load and hot-swap request selection only |
| `hello_service/constants.rs` | non-rollback descriptor, preflight, slot, state, migration, probation, and selftest constants only |
| `hello_service/descriptor_identity.rs` | descriptor/artifact hash and signature projections (`:23-78`) |
| `hello_service/emitters.rs` | health (`:88-186`), reset-state migration denial (`:188-231`), lifecycle response and nested records (`:4429-5086`) |
| `hello_service/hash_support.rs` | canonical hash helpers used by the in-scope records; do not relocate if doing so changes any canonical grammar |
| `hello_service/lifecycle_binding.rs` | only the non-rollback lifecycle binding projection; the large rollback-default tail is not P4-6 authority |
| `hello_service/preflight.rs` | artifact load-plan preflight, service-slot activation, validator, and eight-case selftest (`:3-287`, `:297-410`); rollback-inspect selftest hash at `:289-295` is excluded |
| `hello_service/records.rs` | `LoadDescriptor` (`:4`), `ArtifactLoadPlanPreflightRecord` (`:25`), `ServiceSlotActivationRecord` (`:792`), `HelloStateMigrationRecord` (`:817`), `HelloHotSwapProbationRecord` (`:840`), preflight case (`:877`), and `LoadRequest` (`:886`) only |
| `hello_service/runtime.rs` | lifecycle dispatch/acquisition (`:98-235`) and descriptor/artifact/preflight selftests (`:391-633`) |
| `hello_service/state_machine.rs` | load/start/restart/hot-swap/reset-denial/stop/drop/health (`:16-400`) |
| `hello_service/state_records.rs` | state, migration, and probation record construction/hash (`:3-248`) |

`hello_service.rs` and `current_boot_service.rs` remain attested dependencies,
not an excuse to broaden P4-6. `agent_protocol_system.rs` calls the nested Hello
emitters for `service.inventory`; that response belongs to P4-9. P4-6b must keep
a compatibility projection for those callers or obtain an explicit scope change.

Explicitly excluded whole files:

```text
seed-kernel/src/hello_service/rollback_authority_gates.rs
seed-kernel/src/hello_service/rollback_bindings.rs
seed-kernel/src/hello_service/rollback_hashes_a.rs
seed-kernel/src/hello_service/rollback_hashes_b.rs
seed-kernel/src/hello_service/rollback_writer_bindings.rs
seed-kernel/src/hello_service/rollback_writer_gate.rs
seed-kernel/src/hello_service/storage_authority_gate.rs
seed-kernel/src/hello_service/storage_gate_hash.rs
```

They construct or consume rollback/write authority. They are P4-7 even where
they embed migration/probation facts. Also excluded are `emitters.rs:233-4428`,
`runtime.rs:238-389`, and `state_machine.rs:402-EOF`: those are rollback preview,
apply, inspection, materialization, and their authority evidence.

### Transport and dispatch

All successful legacy paths use `begin_response()`/`end_response()`:

```text
RAIOS_AGENT_BEGIN <method>
{
  v,
  t,
  id,
  body: {
    method,
    result: { ... }
  }
}
RAIOS_AGENT_END <method>
```

The exact legacy order is `v,t,id,body`; body order is `method,result`
(`agent_protocol_support.rs:733-750`). The reset-state hot-swap denial instead
uses `t:error` and emits its fields directly under `body`
(`agent_protocol_support.rs:753-765`, `emitters.rs:188-231`). P4 v1 requires both
to become the shared evidence-response envelope while preserving BEGIN/END.

Dispatch/acquisition paths are:

| Method/path | Acquisition and emitter |
|---|---|
| `module.load_ephemeral svc.demo.hello` and `host_bound:svc.demo.hello` | `runtime.rs:170-181` -> `state_machine::load_start` -> `emit_response` |
| `service.start` | `runtime.rs:190-193` -> `state_machine::start` -> `emit_response` |
| `service.restart` | `runtime.rs:196-205` -> `state_machine::restart` -> `emit_response` |
| `service.stop` | `runtime.rs:184-187` -> `state_machine::stop` -> `emit_response` |
| `service.drop` | `runtime.rs:226-229` -> `state_machine::drop_service` -> `emit_response` |
| `service.hot_swap` built-in/v2 | `runtime.rs:207-216` -> `state_machine::hot_swap` -> `emit_response` |
| `service.hot_swap ...reset_state` | `runtime.rs:217-220` -> `denied_reset_state_hot_swap` -> `emit_hot_swap_state_migration_denied` |
| `service.health` | `runtime.rs:232-235` -> `health_probe` -> `emit_health_response` |
| descriptor-source trust selftest | `runtime.rs:391-467` raw selftest emitter |
| artifact-reference trust selftest | `runtime.rs:469-547` raw selftest emitter |
| load-plan preflight selftest | `runtime.rs:549-633` raw selftest emitter |

An `external:svc.demo.hello` load/hot-swap is rejected before these lifecycle
emitters and is not claimed here.

### Current lifecycle response order

`emit_response` (`emitters.rs:4429-4473`) emits, in order:

```text
R:
 schema, scope, classification, persistence, action,
 event_id, audit_event_id,
 load_request, load_descriptor, artifact_load_plan_preflight,
 service_slot_activation, state, state_migration, hot_swap_probation,
 service, lifecycle, loader, denied_surfaces
```

Nested order:

```text
load_request:
 schema, scope, classification,
 descriptor_schema, descriptor_id, descriptor_source_locator,
 descriptor_source_kind, descriptor_source_validated, descriptor_source_hash,
 descriptor_source_signature_envelope,
 artifact_identity_id, artifact_identity_hash,
 artifact_identity_signature_envelope,
 artifact_content_binding_id, artifact_content_binding_hash,
 artifact_content_source_hash, artifact_content_trust_envelope_id,
 artifact_content_trust_envelope_hash,
 artifact_reference_id, artifact_reference_hash, artifact_bytes_sha256,
 artifact_reference_content_binding_hash,
 artifact_reference_trust_envelope_id,
 artifact_reference_trust_envelope_hash,
 artifact_load_plan_preflight_id, artifact_load_plan_preflight_hash,
 artifact_load_plan_preflight_status,
 service_slot_intent_id, ram_only_service_slot_id,
 binds_source_locator, binds_source_kind, binds_source_hash,
 service_id, accepted

load_descriptor:
 schema, id,
 source {canonicalization, locator, kind, validated, sha256,
         binds_source_locator, binds_source_kind, binds_source_hash,
         signature_envelope, text},
 service_id, artifact_id, artifact_kind, artifact_identity,
 artifact_load_plan_preflight, scope, classification, persistence,
 accepts_external_artifact_bytes, loads_external_artifact,
 maps_executable_pages, writes_persistent_state

service:
 id, artifact_id, version,
 artifact identity/content/reference IDs, hashes and signature envelopes,
 preflight ID/hash/status, activation ID/hash/status/active,
 slot ID, descriptor ID/source kind/validated/hash/signature,
 kind, loaded, running, generation, state, health, capabilities

lifecycle:
 last_action, reason, service_inventory_change,
 load_event_id, start_event_id, hot_swap_event_id,
 stop_event_id, drop_event_id

loader:
 kind, descriptor ID/locator/kind/validated/hash/signature,
 artifact identity/content/reference IDs, hashes and signature envelopes,
 preflight ID/hash/status, activation ID/hash/status/active,
 slot intent ID, slot ID, bound source locator/kind/hash,
 accepts_external_artifact_bytes, loads_external_artifact,
 maps_executable_pages, writes_persistent_state,
 writes_durable_audit_log, installs_rollback_plan, grants_broad_mutation

denied_surfaces:
 general_module_load, external_artifact_load, persistent_install,
 durable_audit, rollback_install, broad_mutation
```

The common nested record order is source-authoritative at:

- preflight: `emitters.rs:4745-4787`;
- slot activation: `emitters.rs:4790-4824`;
- state: `emitters.rs:4831-4846`;
- migration: `emitters.rs:4849-4875`;
- probation: `emitters.rs:4882-4928`;
- descriptor envelope: `emitters.rs:4935-4955`;
- artifact identity/content/reference/signature: `emitters.rs:4958-5085`.

### Health and reset-denial order

Health (`emitters.rs:88-121`) emits:

```text
R:
 schema, scope, classification, persistence, action,
 event_id, audit_event_id,
 service_slot_activation, state, state_migration, hot_swap_probation,
 service, load_descriptor, denied_surfaces

service:
 id, kind, version, loaded, running, health, generation, state,
 last_action, last_reason,
 service_slot_activation_id/hash/status/active, capabilities
```

Reset-state hot-swap denial (`emitters.rs:194-230`) emits:

```text
body:
 method, event_id, audit_event_id, code, reason, message,
 service_id, target, active_generation, active_descriptor_id,
 state, state_migration, required[], denied_surfaces
```

### Hello source-attestation regeneration

`HELLO_ARTIFACT_SOURCE_SET` contains the root, `current_boot_service.rs`, and
all 20 Hello modules in declaration order (`seed-kernel/build.rs:8-32`). Every
P4-6 source edit therefore changes the attested ordered byte snapshot even when
the source-set membership does not change.

“Regenerate the Hello source attestation” concretely means:

1. frame every source-set file as decimal path length, LF, path, LF, decimal
   byte length, LF, raw bytes, LF, in manifest order
   (`seed-kernel/build.rs:1761-1780`);
2. recompute the source-set SHA-256 (`build.rs:198-199`), content-binding text
   and hash (`build.rs:200-214`), and artifact-reference text and hash
   (`build.rs:217-233`);
3. update the four changed pins in both v1/v2 identity descriptors while
   leaving the artifact-bytes hash unchanged unless the artifact bytes changed
   (`build.rs:240-298`);
4. use `descriptor-resign` to generate fresh ephemeral development P-256
   public-key/signature tuples for both exact raw identity descriptor files,
   then run its `verify` mode;
5. let `build.rs` independently verify both signatures and all pins before the
   kernel build (`build.rs:176-182`, signature verifier `:1787-1790`).

No old private key exists or is needed. The current-image descriptor tuple is
unchanged unless its raw descriptor changes. The locator remains
`seed-kernel/src/hello_service.rs` (`build.rs:205`, generated constant `:431`).

## 2. Semantic mapping

### Envelope

```text
v -> retired(v1 schema is the protocol version)
t -> retired(v1 decision carries observed/denied/granted outcome)
id:"serial" -> v1.id (typed response.current_boot.NNNNNNNN)
body.method -> source_method
body.result -> flattened v1 root

schema -> constant("raios.evidence_response.v1")
family -> constant(method-table family)
scope -> same evaluator/descriptor source
classification -> same evaluator/descriptor source
event_id -> kernel-acquired lifecycle/health event ID
```

Proposed families:

```text
hello.lifecycle
hello.health
hello.lifecycle.migration_denial
hello.descriptor_source.selftest
hello.artifact_reference.selftest
hello.artifact_load_plan_preflight.selftest
```

### Common facts

```text
R.persistence -> F.persistence
R.action -> F.action
R.service.id -> F.service.id
R.service.artifact_id -> F.service.artifact_id
R.service.version -> F.service.version
R.service.kind -> F.service.kind
R.service.loaded -> F.service.loaded
R.service.running -> F.service.running
R.service.generation -> F.service.generation
R.service.health -> F.health.status_detail
R.service.capabilities -> F.service.capabilities
R.lifecycle.last_action -> F.lifecycle.last_action
R.lifecycle.reason -> F.lifecycle.reason
R.lifecycle.load_event_id -> F.lifecycle.load_event_id
R.lifecycle.start_event_id -> F.lifecycle.start_event_id
R.lifecycle.hot_swap_event_id -> F.lifecycle.hot_swap_event_id
R.lifecycle.stop_event_id -> F.lifecycle.stop_event_id
R.lifecycle.drop_event_id -> F.lifecycle.drop_event_id
```

`R.audit_event_id` is the same event as `R.event_id` on every in-scope path:

```text
R.event_id -> event_id
R.audit_event_id -> retired(single envelope event_id)
state-transition provenance -> E[state_transition].source_event_id
```

Recording/retention is provenance, never a decision effect. A denied migration
may still have a non-null `event_id`; `D.effects` must remain empty.

### Descriptor evidence

All repeated descriptor projections map to one ordered evidence unit:

```text
R.load_request.descriptor_* -> E[descriptor].facts.request.*
R.load_descriptor.schema/id -> E[descriptor].facts.schema/id
R.load_descriptor.source.canonicalization -> E[descriptor].facts.canonicalization
R.load_descriptor.source.locator/kind -> E[descriptor].facts.source_locator/source_kind
R.load_descriptor.source.sha256 -> E[descriptor].facts.source_hash
R.load_descriptor.source.binds_source_* -> E[descriptor].facts.binds_source_*
R.load_descriptor.source.text -> E[descriptor].facts.source_text
R.*.descriptor_source_signature_envelope.*
    -> E[descriptor].facts.signature_envelope.*
R.*.descriptor_source_validated / source.validated
    -> E[descriptor].status="verified" or "rejected"
R.*.descriptor_source_signature_envelope.signature_verified
    -> E[descriptor].facts.signature_verified and common evidence status
```

The repeated service/loader descriptor IDs, hashes, kinds, validation booleans,
and signature envelopes are retired duplicates of `E[descriptor]`.

### Artifact evidence

One `E[artifact]` carries the selected identity, content binding, artifact
reference, preflight, and slot activation, in that order:

```text
R.*.artifact_identity.schema/id/canonicalization/sha256
    -> E[artifact].facts.identity.*
R.*.artifact_identity.service_id/artifact_id/artifact_kind
    -> E[artifact].facts.identity.*
R.*.artifact_identity.content_binding.*
    -> E[artifact].facts.content_binding.*
R.*.artifact_identity.artifact_reference.*
    -> E[artifact].facts.reference.*
R.*.artifact_identity.signature_envelope.*
    -> E[artifact].facts.signature_envelope.*
R.* artifact identity/content/reference validated or signature_verified
    -> E[artifact].status plus facts.status_detail/signature_verified
R.artifact_load_plan_preflight descriptive IDs/hashes/source bindings
    -> E[artifact].facts.preflight.*
R.service_slot_activation descriptive IDs/hashes/status/active
    -> E[artifact].facts.service_slot_activation.*
```

The same-source rule is strict: identity validity comes from
`validate_builtin_hello_artifact_identity`, signature validity comes from
`artifact_identity_signature_verified`, preflight acceptance comes from
`validate_artifact_load_plan_preflight_record`, and slot active/status comes
from the captured lifecycle snapshot. “Present” cannot substitute for
“verified”.

### State-transition evidence

```text
R.state.schema/id/scope/classification/persistence -> E[state_transition].facts.state.*
R.state.service_id/version/ram_only_service_slot_id -> E[state_transition].facts.state.*
R.state.state_counter/state_hash -> E[state_transition].facts.state.*
R.state.loaded/running -> E[state_transition].facts.state.*
R.state_migration=null -> E[state_transition].facts.migration=null
R.state_migration.* -> E[state_transition].facts.migration.*
R.hot_swap_probation=null -> E[state_transition].facts.probation=null
R.hot_swap_probation.* -> E[state_transition].facts.probation.*
R.service.state -> retired(duplicate of E[state_transition].facts.state)
```

Migration/probation booleans map as follows:

```text
state_preserved -> E[state_transition].facts.migration.state_preserved
accepted migration/probation -> E[state_transition].status + facts.status_detail
loads_candidate_bytes/maps_executable_pages=false
    -> retired(no matching effect in D.effects)
writes_persistent_state/writes_durable_audit_log/installs_rollback_plan=false
    -> retired(no matching effect in D.effects)
applies_rollback=false -> retired(no "rollback_applied" effect)
```

The canonical migration and probation hashes remain facts. Their input grammar
in `state_records.rs:56-92` and `:143-247` must not change merely because the
serial response vocabulary changes.

### Authority, decisions, and first failure

Authority-ish legacy booleans do not remain free facts:

```text
R.load_request.accepted
R.artifact_load_plan_preflight.accepted
R.artifact_load_plan_preflight.authorizes_builtin_current_boot_start
R.service_slot_activation.accepted_preflight
R.service_slot_activation.authorizes_builtin_current_boot_start
    -> evaluator-owned D.outcome/grants/effects

authorizes_candidate_artifact_execution=false
accepts_external_artifact_bytes=false
loads_external_artifact/loads_candidate_bytes=false
maps_executable_pages=false
writes_persistent_state=false
writes_durable_audit_log=false
installs_rollback_plan=false
grants_broad_mutation=false
R.denied_surfaces.*
    -> retired(single fail-closed D.grants/D.effects plus evidence statuses)

R.lifecycle.service_inventory_change
    -> D.effects membership only for an evaluator-authorized lifecycle action;
       "none" means no inventory effect
```

Denied reset-state migration becomes:

```text
body.code="capability_denied" -> D.outcome="denied"
body.reason -> D.reason="state_migration_would_reset_state"
body.required[] -> ordered evidence requirements / D.blocked_by
body.state -> E[state_transition].facts.state
body.state_migration -> E[state_transition].facts.migration
D.grants=[]
D.effects=[]
D.blocked_by[0] = {
  evidence_id:"state_transition",
  status:"rejected",
  reason:"state_migration_would_reset_state"
}
```

Existing evaluator/reason order is source order and must remain unchanged:

- load: already running, loaded/stopped, missing (`state_machine.rs:18-24`);
- start: already running, loaded/stopped, not loaded (`:68-74`);
- restart: loaded, not loaded (`:123-127`);
- hot swap: loaded, not loaded (`:175-179`);
- reset denial: state transition rejection (`:247-271`);
- stop: running, stopped, not loaded (`:277-283`);
- drop: loaded, not loaded (`:327-330`);
- health: running, loaded/stopped, missing (`:378-384`);
- preflight validation: schema through final broad-mutation denial in exact
  `&&` order (`preflight.rs:245-278`).

`D.reason` is the first failed condition. `D.blocked_by` preserves that same
order. Emitters may not reconstruct or sort it.

**ORCHESTRATOR DECISION NEEDED:** successful lifecycle methods already mutate
current-boot state but do not all name a requested capability. Choose either
(a) a real core evaluator that returns `GrantProof` and a pre-existing exact
capability/effect set per action, or (b) an `observed` post-action decision with
inventory/state changes remaining facts. Do not invent a new capability string
in P4-6b, and do not emit `granted` without proof. Health is unambiguously
`observed`; its event recording is provenance, not an effect.

### Selftests

For all three in-scope selftests:

```text
legacy schema/id -> v1 envelope schema/id + family
scope/classification/persistence -> envelope/F
read_only -> constant(true)
mutates_global_event_log=false -> event_id=null
diagnostic_hash -> F.diagnostic_hash
service_id/artifact_id/slot IDs -> F.*
case_count -> F.case_count
passed_count -> F.passed_count
all_passed -> F.passed
cases[].name -> F.cases[].case
cases[].expected_accept -> F.cases[].expected.status
cases[].actual_accept -> F.cases[].actual.status
cases[].reason -> F.cases[].actual.reason (and expected reason where fixed)
cases[].passed -> F.cases[].passed
denied_surfaces -> retired(observation has no grants/effects)
decision -> {outcome:"observed",reason:"selftest_completed"}
evidence -> descriptor or artifact evidence for the live record displayed
```

## 3. Evidence-unit design and ownership

Every non-selftest lifecycle/health response has exactly three ordered units:

| Order | ID | Kind/status | Facts and provenance |
|---:|---|---|---|
| 1 | `descriptor` | `descriptor`; `verified/rejected/missing` | selected descriptor, source/bound-source hashes, signature envelope, validation detail |
| 2 | `artifact` | `artifact`; `verified/rejected/missing` | identity, content binding, reference, bytes hash, signature, preflight, slot activation |
| 3 | `state_transition` | `state_transition`; `verified/rejected/not_applicable` | captured pre/post state, action/reason, migration/probation, inventory transition; `source_event_id` equals the recorded lifecycle/health event |

Health uses `state_transition.status="not_applicable"` with
`facts.status_detail="health_observation"`; it does not pretend a mutation
occurred. A rejected reset-state migration uses `status="rejected"` and is the
first blocker.

The evaluator must capture descriptor/artifact validation, pre-state, planned
transition, post-state, lifecycle reason, inventory effect, and event binding
as one projection. Today `state_machine.rs` builds the event binding before
mutating `STATE`, then `emit_response` reads the post-mutation snapshot. P4-6b
must not introduce a second lock/read that can mix generations or descriptors.

### Non-rollback W9 relocation candidates

Safe candidates for `raios-core` while P4-6 files are already touched:

- typed DTOs/projections for descriptor, artifact/preflight/activation, state,
  migration, probation, health, and lifecycle outcome;
- pure state/migration/probation hashers and validators, preserving their exact
  canonical input grammar;
- pure first-reason evaluation for load/start/restart/hot-swap/stop/drop/health;
- v1 field tables and the three evidence projections;
- host fixtures for state transitions and the existing 5/5/8 selftest cases.

Kernel-owned and not candidates in this slice:

- `STATE`/mutex ownership and all state mutation (`runtime.rs:93-105`,
  `state_machine.rs`);
- command parsing/dispatch and target selection;
- event-log mutation, event-ID acquisition, and event retention;
- build-generated descriptor/artifact bytes and signature-key acquisition;
- serial framing/backpressure;
- rollback preview/apply/materialization/inspection, storage I/O, durable append,
  and every rollback/write-authority projection.

`lifecycle_binding.rs` is mixed ownership: its lifecycle prefix may project
from a core DTO, but its rollback-default fields and every rollback-populated
field remain P4-7. Moving the whole struct or changing its hash/binding shape is
scope creep.

## 4. Predicate inventory

### Sweep and classification

The required case-sensitive sweep finds four files:

```text
> rg -l "hello" vm-harness/
vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1
vm-harness/shadow-vm-smoke-profile-project-workspace.ps1
vm-harness/shadow-vm-smoke-profile-quick.ps1
vm-harness/shadow-vm-smoke.ps1
```

Classification:

- `shadow-vm-smoke-profile-quick.ps1`: lifecycle/health/descriptor/artifact/
  preflight assertions are counted below. Rollback preview/apply/inspect/
  materialize and audit-event binding assertions are excluded for P4-7/P4-4;
  `service.inventory` assertions are P4-9.
- `shadow-vm-smoke-profile-hello-rollback-dry-run.ps1`: only direct load,
  health, hot-swap probation, post-rollback health, and drop assertions are
  counted. All rollback preview/apply/inspect/materialize, authorized append,
  storage, audit, and write assertions are P4-7.
- `shadow-vm-smoke.ps1`: profile-name/dispatch plumbing only (`:13`,
  `:371-372`, `:395`); no family assertion.
- `shadow-vm-smoke-profile-project-workspace.ps1`: lowercase “hello” is fixture
  source/search text (`:566`, `:593`, `:725`, `:1216`); no Hello-service
  assertion.

`full` sources both `quick` and `hello-rollback-dry-run`; source assertions are
counted once by owning file, not duplicated as a second manifest row.

Unlike P4-1/P4-4, most Hello goldens are `ConvertFrom-Json` throw assertions,
not named `Assert-LogContains` predicates. The count therefore treats each
`if (...) { throw }`, `Assert-CurrentBootEventId`, and invoked named Hello
assertion helper as one assertion site. The three expected-case loops are
expanded from one body to 5, 5, and 8 runtime checks (`+15`). A compound throw
remains one assertion because it has one failure identity.

| Profile file | leaf comparison survives | must regenerate | framing survives | Total reviewed |
|---|---:|---:|---:|---:|
| `shadow-vm-smoke-profile-quick.ps1` | 53 | 241 | 22 | 316 |
| `shadow-vm-smoke-profile-hello-rollback-dry-run.ps1` | 0 | 5 | 5 | 10 |
| **Total** | **53** | **246** | **27** | **326** |

This exceeds the design estimate (140-260 reviewed / 90-180 regenerated).
The estimate missed two descriptor-source load variants, repeated health
checks across running/stopped/missing/post-preview/post-apply states, and the
three expanded selftest case-name loops.

“Leaf survives” means the expected comparison and source semantic remain; its
shared acquisition path may still move. Every legacy schema, `body.result`
path, nested descriptor/artifact/state path, redundant authority boolean, and
error-body path is in “must regenerate”.

### Needle-count scan evidence

The exact scan used for the table (`$regen` is only the leaf/regeneration
classifier; the selected ranges are the ownership boundary above):

```text
> $quick='vm-harness/shadow-vm-smoke-profile-quick.ps1'; $dry='vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1'; $regen='body\.|schema|accepted|authoriz|writes_|loads_|maps_|installs_|event_id|service_inventory_change|loaded|running|health|state_migration|hot_swap_probation|service_slot_activation|load_descriptor|artifact|source|denied_surfaces|persistence|signature|preflight'; function Split-Range($name,$file,$ranges,$loopLeaf,$loopRegen){$lines=Get-Content $file;$sel=@();foreach($r in $ranges){$sel += $lines[($r[0]-1)..($r[1]-1)]};$a=@($sel|?{$_ -match '^\s*if \(' -or $_ -match 'Assert-CurrentBootEventId' -or $_ -match '& \$AssertHello'});$r=@($a|?{$_ -match $regen}).Count+$loopRegen;$l=$a.Count-($r-$loopRegen)+$loopLeaf;$f=@($sel|?{$_ -match '^\s*Send-AgentCommand '}).Count;"$name leaf_survives=$l must_regenerate=$r framing_survives=$f total=$($l+$r+$f)"}; Split-Range quick $quick @(@(1539,1646),@(1693,1980),@(2021,2229),@(2237,2346),@(2362,2371),@(2447,2495),@(2503,2592),@(2620,2689)) 15 0; Split-Range hello-rollback-dry-run $dry @(@(38,42),@(45,45),@(50,53),@(68,72),@(299,309)) 0 0
quick leaf_survives=53 must_regenerate=241 framing_survives=22 total=316
hello-rollback-dry-run leaf_survives=0 must_regenerate=5 framing_survives=5 total=10
```

Required emitter-marker command and verbatim stdout:

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/hello_service/emitters.rs
```

There is no stdout; ripgrep exits 1 for zero matches. An explicit-zero wrapper
prints:

```text
> $n=(rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/hello_service/emitters.rs); if($LASTEXITCODE -eq 1){$n='0'}; $n
0
```

The literal framing lives in shared support (`agent_protocol_support.rs:733-765`),
not `emitters.rs`.

### Regeneration rules and collapse hazards

The serial pipeline renders record-model object newlines as CRLF
(`raios-core/src/record.rs:69,135,143,185,195`), and the observed QEMU stream
adds another carriage return. Multi-line PowerShell needles must therefore use
explicit `` `r`r`n ``. Facts, each evidence record, and decision are
`InlineObject`s and should remain single-line needles.

No needle may span the response `id` field: its current-boot sequence changes.
Envelope needles should anchor either before `id` or from `family` onward.

Collapse hazards and required distinguishing needles:

1. `loaded/running` alone collapses load, start, restart, hot-swap, and health.
   Anchor action plus state-transition status/reason.
2. v1/v2 hot swaps share the same action. Anchor artifact identity ID and
   migration `from_version/to_version`.
3. stopped and missing health share `running:false`. Anchor
   `F.health.status_detail` plus descriptor/artifact evidence status.
4. built-in and host-bound loads share artifact identity. Anchor descriptor
   `source_locator/source_kind/binds_source_hash` in `E[descriptor]`.
5. state and service.state used to duplicate the same hash. Keep one
   `E[state_transition].facts.state.state_hash` needle and one explicit
   same-source comparison; do not retain two vacuous needles.
6. migration/probation `accepted:true` cannot become a generic
   `"status":"verified"` needle. Anchor evidence ID + versions + hash.
7. all retired false authority fields cannot collapse to a bare
   `"effects":[]`. Anchor the complete decision reason/requested capability
   where a denial exists, or the observation reason for health/selftests.
8. descriptor/artifact `present` must not stand in for verified signature/hash
   evidence. Anchor evidence ID, `status`, reason, and signature/hash carrier.
9. `event_id` and `source_event_id` are not interchangeable. Envelope event ID
   is the recorded action; state-transition evidence uses that same captured
   event as provenance.
10. assertions after rollback apply remain P4-6 only when they inspect the
    common health/drop response vocabulary. Assertions proving rollback state
    mutation or authority remain P4-7.

## 5. Selftest strategy

Existing selftests:

| Selftest | Cases | Source |
|---|---:|---|
| descriptor-source trust | 5 | evaluator in `descriptor_sources`; response `runtime.rs:391-467` |
| artifact-reference trust | 5 | evaluator in `descriptor_sources`; response `runtime.rs:469-547` |
| artifact load-plan preflight | 8 | fixtures `preflight.rs:297-387`; response `runtime.rs:549-633` |

There is no dedicated lifecycle/state-machine, state-migration, probation, or
health selftest. Those semantics currently exist only in the `quick` guest
sequence.

P4-6b should leave the smallest complete host-test set in `raios-core`:

1. table-driven load/start/restart/stop/drop reason and state transitions for
   missing, loaded-stopped, and running inputs;
2. hot-swap preserve-state and reset-state rejection, including unchanged
   first-failure reason;
3. health running/stopped/missing projections;
4. exact state/migration/probation canonical hash fixtures;
5. the existing 5/5/8 descriptor/artifact/preflight fixtures, with the same
   case order and actual source accessors;
6. one exhaustive legacy-semantic-group mapping test;
7. one same-source test proving response event, transition evidence event,
   post-state generation, descriptor, and artifact all come from one captured
   projection;
8. denied reset migration always has empty grants/effects and ordered blocker;
9. successful authority, if chosen, cannot render `granted` without
   evaluator-created `GrantProof`.

Guest coverage remains the existing focused `quick` path: both descriptor
sources, lifecycle actions, v1/v2 migration/probation, reset rejection, all
health states, and the three selftests. P4-6a runs no Cargo or VM command.

## 6. Risks and P4-6b STOP-tripwires

1. **Lifecycle positive decision — ORCHESTRATOR DECISION NEEDED.** Resolve the
   `granted` versus `observed` choice in section 2 before emitting v1. STOP if a
   worker would invent a capability or construct authority in an emitter.
2. **Shared-core attestation boundary — OWNER/ORCHESTRATOR DECISION NEEDED.**
   If non-rollback W9 logic moves to `raios-core`, decide whether the ordered
   Hello source set includes the new core files or whether immutable-kernel
   trust explicitly owns them. STOP rather than silently reducing what the
   Hello artifact attestation covers.
3. **Mixed `lifecycle_binding.rs` ownership.** STOP if the proposed deletion or
   relocation touches rollback/write fields, their order, or hashes. That is
   P4-7.
4. **Probation is a P4-7 input.** P4-6 owns the probation model/response, but
   rollback preview/apply consumes its exact hashes and fields. Preserve the
   canonical probation/migration grammar byte-for-byte; run P4-7 profiles only
   when that authority boundary is intentionally touched.
5. **Rollback-produced lifecycle state.** `state_machine.rs:435-EOF` and every
   applied-rollback transition are P4-7. A health response after rollback uses
   the common P4-6 renderer, but P4-6 must not claim or rewrite the apply logic.
6. **Attestation regeneration correctness.** Any changed attested byte without
   both descriptor pin updates, fresh v1/v2 signatures, signer verification,
   build verification, and focused `quick` evidence is a STOP. Never hand-edit
   generated Rust constants or reuse an unrelated key.
7. **Same-source state/event race.** STOP if descriptor/artifact/state/event
   facts are reacquired independently after releasing `STATE`; one captured
   evaluator projection is required.
8. **Hash grammar drift.** Response hashes use the v1 `Value` tree, but state,
   migration, probation, preflight, activation, descriptor, and artifact
   authority hashes retain their existing canonical inputs. STOP on any hash
   change not explicitly caused by the source-attestation regeneration.
9. **P4-9 donor exposure.** `agent_protocol_system.rs` consumes nested Hello
   emit helpers. Before deleting one, sweep all callers and either preserve a
   compatibility projection or obtain a P4-9 scope decision.
10. **P4-4 event ownership.** Lifecycle bindings inside `memory.recent_events`
    are event-family output. Do not regenerate or claim those predicates in
    P4-6 unless the already-committed P4-4 semantic boundary explicitly hands
    them back.
11. **Serialization needles.** STOP if a regenerated multi-line needle uses
    plain CRLF, spans `id`, or treats an InlineObject as multiline. Inspect the
    real serial transcript before accepting changes.
12. **No P4-7 authority claims.** P4-6 must not claim rollback preview/apply,
    durable audit/store writes, transaction append/readback, inspection,
    installed rollback state, or any `agent_protocol_module_write_boundary*`
    behavior.

Scope-creep observation: the design row’s phrase “lifecycle/model/preflight/
state files” is not a clean file boundary. `records.rs`, `runtime.rs`,
`state_machine.rs`, `lifecycle_binding.rs`, and especially `emitters.rs` are
mixed with live P4-7 authority. P4-6b must cut by the functions and record types
listed here, not by whole-file ownership.
