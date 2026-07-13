# P4-4a — event evidence semantic manifest

Read-only inventory for P4-4b. No Rust or harness file was changed, built, or
tested. This manifest owns the event-record vocabulary rendered through
`memory.recent_events` / `audit.events`; it does not claim the direct response
predicates already assigned to P4-1, P4-2, P4-3, P4-5, P4-6, P4-7, or P4-8.

Notation:

- `R` = legacy `body.result` of `memory.recent_events`.
- `F` = v1 event-family response `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 decision.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

## 1. Response-path inventory

### Dispatch and acquisition

One canonical method and two aliases reach the same renderer:

```text
memory.recent_events [limit]
audit.events [limit]
events [limit]
```

The method table is at `seed-kernel/src/agent_protocol.rs:372`. All three use
`MethodAction::ReadMethod`; dispatch first calls
`record_read(call.canonical)` and discards its `EventId`, then calls
`emit_recent_events(call.input)` (`agent_protocol.rs:656-660`). The canonical
recorded `source_method` is therefore `memory.recent_events`, including alias
calls.

`event_limit_arg()` supplies the requested limit. `normalize_limit()` maps zero
to 32 and caps larger values at 256 (`event_log.rs:6987-6992`). Under the ring
mutex, `EventLog::recent_events()` calculates the newest bounded window and its
metadata in this exact evaluator order (`event_log.rs:205-229`):

```text
limit, want/len, skip, oldest, total_count,
dropped_before_sequence, first_slot
```

`EventLog::recent_event()` then reads each slot from oldest to newest within the
selected window (`event_log.rs:232-238`). The public accessors each acquire the
global mutex independently (`event_log.rs:6506-6512`), so the current renderer
does not hold one immutable snapshot lock across metadata acquisition and every
event copy.

### Current response and field order

`emit_recent_events()` is at `agent_protocol_memory.rs:364-403`. The transport
is:

```text
RAIOS_AGENT_BEGIN memory.recent_events
{
  v, t, id,
  body: { method, result: { ... } }
}
RAIOS_AGENT_END memory.recent_events
```

`begin_response()` / `end_response()` own that legacy envelope. The result
field order is exactly:

```text
schema, record_schema, scope, retention, persistence, provider_export,
bounded, limit, capacity, event_count, returned, dropped_before_sequence,
events
```

The aliases still frame as `memory.recent_events`, not the input alias.

Each array member is emitted by `emit_event()`
(`agent_protocol_memory.rs:438-474`) as one single-line object in this exact
order:

```text
schema, id, scope, sequence, kind, source_method, source_transport,
classification, outcome, requested_capability, risk, subject, resource,
reason, evidence, created_at {clock, millis}, [bindings], persistence
```

`bindings` is omitted only for `EventBindings::None` and for the intentionally
unrendered `ModulePromotionSignatureReference` variant. All other binding
objects are single-line inline objects. `EventBindings` declares 90 variants
including `None` (`event_log_types.rs:3699-3864`); the exhaustive renderer match
is `agent_protocol_memory.rs:2016-2265`.

### Binding projection inventory

The current binding order is not struct layout order. It is the order of each
`*_FIELDS` declaration consumed by `emit_binding_object()` or
`emit_binding_object_direct()` (`agent_protocol_memory.rs:490-563`). P4-4b must
use those declarations as the old-order oracle. The inventory below is
exhaustive by projection class; variants sharing
`ModuleLoaderLiveLoadBoundaryBindingField` share its order while retaining
their distinct schema/boundary values.

| Projection class | Variants / current ordered field count |
|---|---|
| no binding | `None`; `ModulePromotionSignatureReference` (intentionally emits nothing) |
| Hello lifecycle | `HelloServiceLifecycle`; 1,079 ordered fields declared from `agent_protocol_memory.rs:633` through the lifecycle macro close; includes descriptor/artifact/state/hot-swap/rollback/storage facts |
| Hello inspect source | `HelloRecoveryRollbackInspectSourceReference`; 10: `schema, status, scope, classification, source_event_id, reference_hash, inspection_hash, source_sector_plan_hash, source_target_region_write_readback_hash, authorizes_rollback_apply` |
| command envelope | `AgentCommandEnvelopeDecision`; 24: `schema, scope, classification, command_schema, schema_ok, target_method, target_method_allowed, requested_capability, requested_capability_allowed, submitted_classification, classification_allowed, accepted, code, reason, dispatches_existing_agent_method, creates_parallel_dispatcher, provider_write, loads_candidate_bytes, writes_persistent_state, writes_durable_audit_log, installs_rollback_plan, grants_broad_mutation, source_evidence_retained, retention` |
| provider | request envelope 9; request bound 13; export-audit bound 17; consumption 14; injection authorization 18; request denial 5; export denial 6 (`agent_protocol_memory.rs:3123-3639`) |
| module retained references | manifest 15; artifact 16; VM report 19; attestation 19; approval 20; computed grant 13; audit/rollback 18; service-slot reservation 18 (`agent_protocol_memory.rs:3641-4465`) |
| allocator source evidence | fact 27; prerequisite 30; authority 30; allocation intent 35; authority input 36; authority decision 36; registry commit gate 46 (`agent_protocol_memory.rs:4467-5579`) |
| loader source evidence | identity 29; artifact-hash binding 31; generic fact 37; execution commit gate 50; descriptor intake 49; artifact-byte intake 53; execution authorization 58; registry mutation 51 (`agent_protocol_memory.rs:5581-7299`) |
| loader live boundary | 32 distinct `EventBindings` variants share the 159-field `ModuleLoaderLiveLoadBoundaryBindingField` order beginning `schema, boundary_schema, boundary_id, status, reason, scope, classification, requested_capability, load_mode, target, source_method, source_fact_locator, ...` and ending `..., records_load_result, load_attempted, authorizes_load` (`agent_protocol_memory.rs:2271-2945`) |
| recovery artifact/lifeline | identity 14; trust 15; VM test 17; approval 18; loader 20; rollback evidence 22; request 26; envelope 24; canonical body 25; handler 26; status handler 27; preview 29; apply 29; disable 31; restart 33; load-by-hash 34; memory write 35; durable audit/rollback write 37; inventory boundary 38; dispatch 38; executor 38; side-effect gate 38; execution stage 39; status result 34 (`agent_protocol_memory.rs:7303-11089`) |
| module load gate | `ModuleLoadGate`; 22 top-level ordered fields: `schema, status, load_mode, requested_capability, risk, target, subject, gate_state, retained_module_manifest_reference, retained_candidate_artifact_reference, retained_vm_test_report_reference, retained_local_attestation_reference, retained_local_approval_reference, retained_computed_grant_reference, retained_audit_rollback_reference, retained_service_slot_reservation, service_slot_allocator_readiness, loader_runtime_readiness, audit_rollback_requirements, blocked_by, required, evidence`; emitted outside the named P4 files at `agent_protocol_module_load_gate_render.rs:5561-5664` |

The nested load-gate order and semantics remain those inventoried by P4-2a.
P4-4 owns only their event-wrapped projection. Direct load-gate response
predicates remain P4-2 and are not counted again here. The same discipline
applies to P4-1 module references and P4-3 allocator/loader facts.

### Material disagreement with the P4 design row — DECISION NEEDED

The design row names `event_log.rs`, `event_log_types.rs`, and “event evidence
and family binding projections.” The actual serial emitter is
`agent_protocol_memory.rs:364-11089`; the load-gate event projection is in
`agent_protocol_module_load_gate_render.rs:5561-5664`; ordered evidence-label
sources live in `event_log_evidence.rs`. Deleting emitters only in the two named
files cannot produce P4-4. P4-4b must receive an explicit expanded write set or
STOP.

## 2. Semantic mapping

### Batch-shape decision — DECISION NEEDED

The design grammar provides one `decision` per `raios.evidence_response.v1`,
but `memory.recent_events` returns zero to 256 historical events, each with its
own outcome/reason. Flattening them into the response's one `D` loses denial
semantics and provenance.

Recommended minimal resolution:

```text
top-level evidence response:
  family = "event"
  F.ring = ring metadata
  F.events = ordered array of event projections
  D = Observed("event_window_returned")

each F.events[i]:
  event_id
  facts
  evidence
  decision
```

The nested projection must be built from the same typed v1 facts/evidence/
decision constructors; it must not introduce another serializer or a new
hand-written schema. The alternative—an array of complete
`raios.evidence_response.v1` envelopes—is semantically valid but repeats
response metadata up to 256 times. P4-4b must not delete `emit_event()` until
the orchestrator chooses one of these shapes.

### Collection mapping

```text
legacy v/t/id/body/method
  -> retired(shared raios.evidence_response.v1 envelope)
R.schema
  -> retired(event.log.v0 replaced by constant(raios.evidence_response.v1))
R.record_schema
  -> retired(audit.event.v0 is response vocabulary, not stored-record identity)
R.scope
  -> scope
R.retention
  -> F.ring.retention
R.persistence
  -> F.ring.persistence
R.provider_export
  -> F.ring.provider_export
R.bounded
  -> F.ring.bounded
R.limit
  -> F.ring.limit
R.capacity
  -> F.ring.capacity
R.event_count
  -> F.ring.total_count
R.returned
  -> F.ring.returned_count
R.dropped_before_sequence
  -> F.ring.dropped_before_sequence
R.events[]
  -> F.events[] (same oldest-to-newest order)
top-level event_id
  -> the `record_read("memory.recent_events")` EventId, after dispatch threads it
D
  -> Observed("event_window_returned")
```

`event_count` means total events assigned in the current boot, not current ring
length. Renaming it to `total_count` removes the ambiguity without changing the
accessor.

### Per-event mapping

For every legacy `R.events[i]`, let `V` be its nested v1 event projection:

```text
R.events[i].schema
  -> retired(audit.event.v0 response-only tag)
R.events[i].id
  -> V.event_id
R.events[i].scope
  -> constant(current_boot)
R.events[i].sequence
  -> V.facts.sequence
R.events[i].kind
  -> V.facts.kind
R.events[i].source_method
  -> V.facts.source_method
R.events[i].source_transport
  -> V.facts.source_transport
R.events[i].classification
  -> V.classification
R.events[i].outcome
  -> V.facts.status_detail plus V.decision as specified below
R.events[i].requested_capability
  -> V.facts.requested_capability
R.events[i].risk
  -> V.facts.risk
R.events[i].subject
  -> V.facts.subject
R.events[i].resource
  -> V.facts.resource
R.events[i].reason
  -> V.facts.status_reason and V.decision.reason
R.events[i].evidence[]
  -> V.evidence["source"].facts.labels[] (same declared order)
R.events[i].created_at.clock
  -> constant(sequence_only)
R.events[i].created_at.millis
  -> constant(null)
R.events[i].bindings
  -> V.evidence["binding"]
R.events[i].persistence
  -> constant(none) and F.ring.persistence
```

The event's own identity is `V.event_id`. `E[source].source_event_id` and
`E[binding].source_event_id` both cite that same event. Any dependency event ID
inside a binding maps to its evidence unit's `facts.*_source_event_id`; it is
provenance, never an effect. Null dependency IDs remain explicit nulls.

### Binding mapping, exhaustive mechanical rules

These rules cover every ordered field catalogued in section 1:

```text
R.events[i].bindings.schema
  -> E[binding].facts.record_schema
R.events[i].bindings.status
  -> E[binding].facts.status_detail
R.events[i].bindings.reason / *_reason
  -> E[binding].reason / E[binding].facts.*_reason
R.events[i].bindings.scope
  -> retired(duplicate of enclosing current_boot event scope)
R.events[i].bindings.classification
  -> retired(duplicate of E[binding].classification)
R.events[i].bindings.event_id / *_event_id / source_event_id
  -> E[binding].source_event_id for the binding's own record;
     dependency IDs -> E[binding].facts.*_source_event_id
R.events[i].bindings.hashes.* / *_hash / *_sha256
  -> E[binding].facts at the same leaf name and same typed hash accessor
R.events[i].bindings identifiers, locators, counts, states, booleans that
describe already-observed input or state
  -> E[binding].facts at the same leaf name and in declaration order
R.events[i].bindings.blocked_by[]
  -> V.decision.blocked_by[] in existing evaluator order
R.events[i].bindings.required[]
  -> E[binding].facts.required[] in existing order
R.events[i].bindings.gate_state / retained_* / *_readiness / *_boundary
  -> ordered E units named for those semantic units; nested descriptive leaves
     map to each unit's facts
```

Legacy action/authority booleans do not become facts or effects merely because
they were nested in a binding:

```text
grants_*, authorizes_*, can_load*, command_execution_enabled,
satisfies_*_gate, positive_export_authorization
  -> retired(redundant with evaluator-created V.decision and blocked_by)

load_attempted, writes_*, installs_*, allocates_*, creates_*, loads_*,
maps_*, jumps_*, starts_*, marks_*, records_*, commits_*, unloads_*,
cleans_*, applies_*, mutates_*, dispatches_*, executes_*, provider_write
  -> positive observed action becomes V.decision.effects only when the existing
     event records that action as performed;
     false/not_attempted becomes a boundary-specific evidence status/reason,
     never a generic effect and never one shared zero

service_inventory_change
  -> E[service_registry].facts.service_inventory_change
retention / source_evidence_retained
  -> E[binding].facts.retention plus E[binding].source_event_id
```

This intentionally does not copy the 1,079 Hello keys into a second prose list.
The ordered declaration is the source of completeness, and the rules above
partition every key by name and semantic role. A host test must iterate every
old `*_FIELDS` key and fail if it matches no carrier rule.

### Event decisions and exact existing denials

Events are observations. `V.decision` is `Observed(V.facts.status_reason)` for
`response`, health strings, readiness strings, retained-reference states,
binding states, test-media results, and other already-recorded observations—even
when a descriptive status contains the word `denied`.

Use `Denied` only where the existing acquisition path records a denied request:

1. `record_capability_denied()` (`event_log.rs:4409-4428`): outcome
   `capability_denied`, reason `missing_evidence`.
2. `record_module_load_ephemeral_denied()` (`event_log.rs:4693-4713`): outcome
   `capability_denied`, reason `missing_evidence`; its ordered blockers come
   from the existing module-load evaluator, not the renderer.
3. `record_provider_request_binding_denied()` (`event_log.rs:6374-6391`):
   outcome `denied_not_bound`, first failure
   `provider_request_binding_requires_real_request_envelope`.
4. `record_provider_context_export_denial_audit()`
   (`event_log.rs:6488-6503`): outcome `denied_no_provider_write`, first failure
   `provider_context_export_not_authorized`.
5. `record_agent_command_envelope_decision()` (`event_log.rs:4379-4406`) when
   `binding.accepted == false`: reason is `binding.reason`; `binding.code`
   distinguishes `capability_denied` from `invalid_envelope`. Accepted envelopes
   remain Observed.
6. Generic service lifecycle callers that pass `capability_denied`, including
   Hello hot-swap/reset and rollback paths (`hello_service/state_machine.rs:257-269,
   422-430, 476`), Echo denied load/start/stop/drop paths
   (`echo_service.rs:403-412, 454-463, 526-535, 558-567`), and granted-candidate
   denied load/start/stop/drop/rollback paths
   (`granted_candidate_service.rs:496-506, 648-657, 719-728, 765-774,
   818-827, 879-888, 941-946`). Their existing `reason` is the first failure.

Denied decisions always emit empty grants/effects. A status string containing
`*_still_denied` is not by itself a denied request; converting all such strings
to `D.outcome:"denied"` is a STOP.

## 3. Evidence-unit design and ownership

For each event, evidence order is evaluator-owned and fixed:

1. `E[source]`: `kind:"source"`, status `verified`, reason
   `event_recorded_current_boot`; `facts.labels[]` preserves the event's
   existing `event_log_evidence.rs` slice order exactly.
2. `E[binding]` when `EventBindings != None`: `kind:"binding"`, common status
   derived from the current binding status, reason from the existing binding
   evaluator, `source_event_id` equal to the enclosing event ID, and ordered
   facts/projection units from section 2.
3. `E[blocked prerequisite]` units only where the existing evaluator already
   exposes ordered blockers. The decision reuses that order; the renderer must
   not sort or reconstruct it.

Relevant evaluator order remains in the owning code, notably provider binding
first failure at `event_log.rs:240-496`, provider injection first failure at
`event_log.rs:555-864`, module-load snapshot construction at
`event_log.rs:4074-4164`, and Hello inspect-source checking at
`event_log.rs:4166-4350`. P4-4b projects their results; it does not create a new
policy evaluator.

Kernel-owned and immovable:

```text
static LOG: Mutex<EventLog>                         event_log.rs:144
mutable ring arrays/cursors/sequence                event_log.rs:156-164
sequence allocation and slot replacement           event_log.rs:179-189
lock acquisition and immutable snapshot copying
event-id acquisition for record_read
serial framing/backpressure/chunking
```

W7 relocation candidates into `raios-core`:

```text
immutable Event/EventBindings DTO projection
ring-window calculation over an immutable snapshot
event facts/evidence/decision projection
binding FieldSpec tables and same-source mapping tests
first-failure decision conversion from an already-evaluated Event
```

The mutex, mutable ring state, slot writes, sequence allocator, and device/
serial access never move. Core must receive one immutable copied snapshot and
must not acquire kernel state itself.

P4-4 owns the event record vocabulary inside `memory.recent_events`, including
ring metadata and every family binding as observed event evidence. P4-5 owns
the memory family (`profile`, context budget, query, trace, omission selection,
memory mutation denial wiring) and may route to this projection, but it must not
re-render or redefine event records. This boundary is required because P4-1/2/3
explicitly deferred their `full-audit` event assertions here, while the P4-5 row
also names `recent events` broadly.

## 4. Predicate inventory

Runtime counts expand the 11-entry `full-audit` source loop and the seven-entry
`full-module-evidence` structured loop. Only event response predicates and
their command completion waits are counted. Direct module/memory/provider/
Hello response predicates are excluded.

| Profile file | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| `shadow-vm-smoke-profile-full-audit.ps1` | 135 | 258 | 0 | 393 |
| `shadow-vm-smoke-profile-full-provider-memory.ps1` | 23 | 16 | 2 | 41 |
| `shadow-vm-smoke-profile-hello-rollback-dry-run.ps1` | 28 | 22 | 1 | 51 |
| `shadow-vm-smoke-profile-full-module-evidence.ps1` | 0 | 7 | 9 | 16 |
| `shadow-vm-smoke-profile-quick.ps1` | 4 | 0 | 2 | 6 |
| `shadow-vm-smoke-profile-m8-lifeline.ps1` | 2 | 0 | 1 | 3 |
| `shadow-vm-smoke-profile-full-module-load-gate.ps1` | 0 | 0 | 1 | 1 |
| **Total** | **192** | **303** | **16** | **511** |

Thus 511 predicates were reviewed and 303 are likely regenerated. The design
estimate of 120-220 reviewed / 70-140 regenerated is disproved by the current
393-predicate `full-audit` file alone.

Surviving leaf needles are classification, kind, source method/locator,
requested capability, risk, subject/resource, reason/evidence labels, hashes,
ring numeric leaves, and explicit service IDs where the exact leaf pair is
unchanged.

Regenerate every scoped assertion using a legacy schema, `bindings` or legacy
object anchor, legacy outcome/status path, legacy event-ID path, or an
authority/action boolean retired by section 2. `full-module-evidence`'s seven
structured checks currently navigate
`body.result.events[].bindings`; they must navigate the chosen nested v1 event
shape.

The `quick` profile also contains non-reporting `if/throw` checks over ten
command-envelope events (`quick.ps1:2691-2745` and following). They do not add
VM report predicates and therefore are not in the 511 total, but every
`body.result.events`, `.bindings`, `.outcome`, and `.id` path must regenerate.

### Collapse hazards and distinguishing needles

| Legacy assertion | Required distinguishing v1 needle |
|---|---|
| event `id` / retained event ID | event-array anchor plus exact `event_id` or binding `source_event_id`; never span the response `id` |
| `*_audit_source`, `*_audit_kind` | one single-line nested event facts needle containing both `kind` and `source_method` |
| `*_audit_outcome` | nested event anchor plus `decision.outcome` and first-failure `decision.reason`; descriptive retained statuses use `facts.status_detail` instead |
| `*_binding_schema` | `E[binding]` anchor plus `facts.record_schema` |
| repeated `*_no_load`, `*_no_write`, `*_no_apply` | boundary-specific evidence ID/reason or a specific action count; never one generic `effects:[]` |
| hash echoes | binding evidence ID plus the specific hash leaf when the same hash appears in multiple events |
| provider positive/negative absence | provider binding evidence ID plus exact decision outcome; do not scan the whole serial history for a bare boolean |
| ring metadata | family/source-method anchor plus `F.ring.<field>` |

Serialization lesson: top-level envelope fields render one per line and arrive
through the QEMU serial pipeline separated as CR CR LF. A multi-line needle
must spell separators as PowerShell `` `r`r`n ``. A single-line needle must
stay within one `facts`, `evidence`, or `decision` InlineObject and must never
span the top-level response `id` line. Facts/evidence/decision nested objects
are single-line `InlineObject`s.

Example safe needles:

```powershell
# top-level multi-line; does not cross the response id
"`"family`": `"event`",`r`r`n  `"scope`": `"current_boot`",`r`r`n  `"classification`": `"local_only`""

# nested single-line event identity/provenance
'"kind": "module.manifest_reference.retained", "source_method": "module.manifest_diagnostic"'

# binding-specific denial, not a generic effects needle
'"id": "module_load_gate", "kind": "binding", "status": "rejected", "reason": "candidate_artifact_missing"'
```

### Required scan evidence

Emitter-marker scan (zero is printed explicitly for files with no literal):

```text
> $files='seed-kernel/src/event_log.rs','seed-kernel/src/agent_protocol_memory.rs','seed-kernel/src/agent_protocol_module_load_gate_render.rs'; foreach($f in $files){$n=(rg -c "RAIOS_AGENT_BEGIN" $f);if($LASTEXITCODE -eq 1){$n='0'};"$f`:$n"}
seed-kernel/src/event_log.rs:0
seed-kernel/src/agent_protocol_memory.rs:1
seed-kernel/src/agent_protocol_module_load_gate_render.rs:1
```

The one marker in `agent_protocol_memory.rs` is the memory-mutation denial
emitter; the event collection uses shared `begin_response()`. The one marker in
the load-gate renderer is its direct response, not its binding fragment.

Required harness sweep:

```text
> rg -l "recent_events|event_log|source_event_id" vm-harness/
vm-harness/shadow-vm-smoke-profile-common.ps1
vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1
vm-harness/shadow-vm-smoke-profile-full-module-load-gate.ps1
vm-harness/shadow-vm-persistence-reboot.ps1
vm-harness/shadow-vm-smoke-profile-full-module-audit-rollback.ps1
vm-harness/shadow-vm-smoke-profile-full-provider-memory.ps1
vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1
vm-harness/shadow-vm-smoke-profile-memory-durable.ps1
vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1
vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1
vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1
vm-harness/shadow-vm-smoke-profile-quick.ps1
```

`full-audit` does not contain any of those three search terms, but it was swept
because every assertion in it runs against the most recently acquired event
response. Files found only because they consume `source_event_id` from direct
P4-1/2/3 responses, or assert a selftest's `event_log_write_count`, are excluded
from this family total.

Needle-count scan (classification regex is the same semantic list used by
P4-2/P4-3; `loopExtra=20` expands two static source assertions from one loop
body to 22 runtime assertions):

```text
> $regen='schema|record_schema|bindings|": \{|outcome|status|event_id|source_event_id|load_attempted|service_inventory_change|grants_|authorizes_|allocates_|creates_|loads_|produces_|validates_|parses_|maps_|jumps_|starts_|marks_|mutates_|writes_|installs_|records_|commits_|unloads_|cleans_|accepts_|applies_|executes_|dispatches_|provider_write|satisfies_|positive_export|transaction_append_performed'; function Split-Lines($name,$lines,$loopExtra=0){$a=@($lines|?{$_ -match '^\s*(Assert-LogContains|Assert-LogDoesNotContain|Add-Predicate)'});$r=@($a|?{$_ -match $regen}).Count;$l=$a.Count-$r+$loopExtra;"$name leaf=$l regen=$r framing=0 assertions=$($l+$r)"}; $x=Get-Content vm-harness/shadow-vm-smoke-profile-full-audit.ps1; Split-Lines full-audit $x 20; $x=Get-Content vm-harness/shadow-vm-smoke-profile-full-provider-memory.ps1; $a=Select-String vm-harness/shadow-vm-smoke-profile-full-provider-memory.ps1 -Pattern '-Name "protocol:(memory_recent_events|no_positive_|audit_events_)'|% Line; Split-Lines full-provider-memory $a; $x=Get-Content vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1; Split-Lines hello-rollback-dry-run $x[311..360]; $x=Get-Content vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1; $a=Select-String vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1 -Pattern '-Name "m8-lifeline:wedge_audit_'|% Line; Split-Lines m8-lifeline $a; $x=Get-Content vm-harness/shadow-vm-smoke-profile-quick.ps1; $a=Select-String vm-harness/shadow-vm-smoke-profile-quick.ps1 -Pattern '-Name "quick:echo_lifecycle_audit_'|% Line; Split-Lines quick $a; 'full-module-evidence leaf=0 regen=7 framing=0 assertions=7';
full-audit leaf=135 regen=258 framing=0 assertions=393
full-provider-memory leaf=23 regen=16 framing=0 assertions=39
hello-rollback-dry-run leaf=28 regen=22 framing=0 assertions=50
m8-lifeline leaf=2 regen=0 framing=0 assertions=2
quick leaf=4 regen=0 framing=0 assertions=4
full-module-evidence leaf=0 regen=7 framing=0 assertions=7
```

Framing scan:

```text
> $files=Get-ChildItem vm-harness -Filter *.ps1; foreach($f in $files){$n=(Select-String -Path $f.FullName -Pattern 'Send-AgentCommand.*(audit\.events|memory\.recent_events)').Count;if($n){"$($f.Name) $n"}}
shadow-vm-smoke-profile-full-module-evidence.ps1 9
shadow-vm-smoke-profile-full-module-load-gate.ps1 1
shadow-vm-smoke-profile-full-provider-memory.ps1 2
shadow-vm-smoke-profile-hello-rollback-dry-run.ps1 1
shadow-vm-smoke-profile-m8-lifeline.ps1 1
shadow-vm-smoke-profile-quick.ps1 2
```

All 16 completion waits for `RAIOS_AGENT_END memory.recent_events` survive.

## 5. Selftest strategy

There is no `#[test]`, event-renderer selftest, ring-window selftest, or
semantic-mapping selftest in `event_log.rs`, `event_log_types.rs`, or
`agent_protocol_memory.rs`.

Existing isolated event-log evaluator selftests are:

| Selftest | Cases | Current source and treatment |
|---|---:|---|
| provider context binding gate | 20 | `event_log_provider_selftest.rs:14-38`; local `EventLog` fixtures cover missing, stale/dropped, previous-boot, wrong variant, substituted record, ID/hash mismatch, trust bypass; evaluator order survives |
| provider injection gate | 8 | `event_log_provider_selftest.rs:41-53`; local fixtures cover missing/stale/substituted authorization, hash/trust downgrade, unauthorized attachment; evaluator order survives |
| Hello rollback inspect source reference | 7 | `event_log_hello_selftest.rs:12-23`; dedicated `SELFTEST_LOG`, cleared per case at lines 26-31; source/audit IDs, wrong variants, substituted hashes, and authorizing-source denial survive |

They test ring lookup and family gates, not event serialization. P4-4b should
leave them behaviorally unchanged and add the smallest host-test set in
`raios-core`:

1. ring window: empty/default, capped limit, wrap/drop, oldest-to-newest order;
2. every legacy event base field has one mapping carrier;
3. every `EventBindings` variant is projected or explicitly retired;
4. every old ordered binding key matches one carrier rule;
5. source-label and blocker order are preserved;
6. observed versus the exact denied paths in section 2;
7. event/source IDs remain eight-digit current-boot IDs;
8. renderer/hash use the same `Value` tree.

Guest coverage should stay small: default `memory.recent_events`, alias with
explicit limit, one observed event with binding, one denied event with binding,
and one wrapped/dropped ring case. The required focused sequence remains
`quick`, then `provider-memory`; this packet does not run either.

## 6. Risks and P4-4b STOP-tripwires

1. **Write-set mismatch — DECISION NEEDED.** The real renderer files are outside
   the design row. No deletion until the orchestrator explicitly includes
   `agent_protocol_memory.rs`, the load-gate binding fragment, and any chosen
   core projection files.
2. **Batch decision shape — DECISION NEEDED.** Choose nested per-event
   facts/evidence/decision or repeated full envelopes. One top-level decision
   may not erase historical denials.
3. **Acquisition event ID.** `ReadMethod` currently discards the read event ID.
   P4-4b must thread that exact ID to the response or explicitly choose
   top-level `event_id:null`; inventing or aliasing the newest returned event is
   a STOP.
4. **Snapshot atomicity.** Metadata and each event are copied under separate
   locks. W7 relocation requires one immutable snapshot; moving the mutex or
   allowing core to reread mutable kernel state is a STOP.
5. **Stored/persisted hash stability.** The current `audit.event.v0` exists only
   in the serial renderer, but durable records retain event IDs and independent
   canonical hashes in `durable_store.rs`. Response-vocabulary changes must not
   alter any durable record, rollback transaction, artifact, descriptor,
   authority, or future persisted-event hash grammar. Regenerating such hashes
   because JSON moved is a STOP.
6. **Provenance is not effect.** Event retention and every dependency event ID
   map through `event_id` / `source_event_id`; adding them to `effects` is a
   STOP.
7. **First failure.** Event decisions reuse the owning evaluator reason and
   blocker order. Recomputing denial from rendered booleans is a STOP.
8. **False denial inflation.** Retained/readiness observations containing the
   word `denied` remain Observed unless they are one of the exact denied request
   paths in section 2.
9. **Family double claim.** Direct module reference, load-gate,
   allocator/loader, memory, Hello, rollback, and provider responses remain
   their owning P4 slices. P4-4 changes only their event-wrapped projection and
   its harness assertions.
10. **P4-5 boundary — DECISION NEEDED.** Confirm that P4-4 owns the event record
    and ring projection inside `memory.recent_events`, while P4-5 owns memory
    selection/enveloping but must reuse it. Letting both slices regenerate the
    same record is a STOP.
11. **Unrendered signature variant.** Decide whether
    `ModulePromotionSignatureReference` remains explicitly retired or gains a
    binding evidence unit. Silent omission is a STOP.
12. **Needle transport.** Multi-line needles require `` `r`r`n ``; single-line
    needles cannot cross response `id`; bulk replacement with bare
    `effects:[]`, `event_id`, or `reason` is a STOP.
13. **Estimate invalidation.** The real 511-predicate inventory and 1,079-field
    Hello event binding exceed the design estimate. P4-4b must be split by
    projection ownership if one reviewable deletion cannot preserve all
    mappings; splitting must not leave two serializers for one binding.

Scope-creep observation: `event_log_evidence.rs` (ordered source-label arrays),
`event_log_provider_selftest.rs`, and `event_log_hello_selftest.rs` are real
event-family dependencies absent from the P4 row. They need not all change in
P4-4b, but treating them as unrelated would make the semantic and selftest
inventory incomplete.

## Orchestrator rulings (2026-07-13, binding for P4-4b)

**R1 — write set follows the emitter, not the design row.** The design row named
`event_log.rs`/`event_log_types.rs` because that is where the event TYPES live;
the serial emitter is `agent_protocol_memory.rs`. P4-4b's write set is the real
renderer set: `agent_protocol_memory.rs`, the load-gate event-binding fragment
in `agent_protocol_module_load_gate_render.rs` (the 26-function compact cluster
restored verbatim during P4-2b2a and explicitly deferred to P4-4), and
`event_log_evidence.rs` for the ordered evidence labels. Deleting emitters in
the two named files alone would leave the vocabulary split across two grammars.

**R2 — nested per-event evidence records; ONE decision about the READ.**
`memory.recent_events` returns 0..256 historical events, each with its own
outcome/reason. Flattening those into the envelope's single `decision` would
erase historical denials, and repeating full envelopes would break the one-
envelope-per-response grammar. Ruling: each event renders as an ORDERED EVIDENCE
RECORD carrying its own status/reason/source_event_id/classification; the
envelope's single `decision` is `observed("recent_events_read")` — the response
decides about the READ, not about the history. This is the same principle
already ratified for retention (commit b6e13bc: retention is provenance, not an
effect): a historical denial is evidence, not a fresh effect of this response.

**R3 — envelope `event_id: null`.** `ReadMethod` discards the read event ID.
Do NOT alias the newest returned event into the envelope (that would make one
event's provenance stand for the whole batch). Bind `event_id: null` exactly as
P4-3 does for observational responses; per-event provenance lives on each
evidence record's `source_event_id`. Threading a real read-event ID is a later,
separate change with its own evidence.

**R4 — P4-5 boundary confirmed as stated.** P4-4 OWNS the event record and ring
projection inside `memory.recent_events`. P4-5 owns memory selection/enveloping
and must REUSE P4-4's projection. Both slices regenerating the same record is a
STOP.

**R5 — `ModulePromotionSignatureReference` gets a real binding evidence unit.**
It is a signature reference; silently omitting a signature from the evidence
stream is precisely the appearance-only gap that P4-2b2b had to correct in the
substituted-approval fixture. If the evaluator genuinely has no consumer for it,
retire it EXPLICITLY with a named manifest note and a reason — never by omission.

## P4-4b1 notes

- `raios-core::event_evidence_projection` now accepts one immutable typed ring
  snapshot and projects its captured events in input order; it never sorts,
  deduplicates, reacquires kernel state, or constructs a granted decision.
- Each historical event is one evidence record with its own status, reason,
  source event ID, classification, base facts, ordered source labels, and
  optional classified binding facts. The response decision is only
  `observed("recent_events_read")`; top-level `event_id` remains the envelope's
  ruled `null`.
- Unknown or empty event kinds and public records carrying secret binding facts
  become explicit rejected evidence. The promotion-signature projection class
  is named and host-tested so R5 cannot regress to silent omission.
- The load-gate event adapter reuses `project_load_gate_denial()`'s twelve
  ordered evidence units but intentionally discards its direct-response
  decision: historical blockers remain event evidence, not a second decision.
- The manifest names `dropped_before_sequence` but no separate `truncated`
  response leaf. P4-4b1 therefore carries the named drop boundary and does not
  invent a truncation field; kernel integration should resolve that only if a
  distinct evaluator-owned fact exists.

## P4-4b2a notes

- Capability: an agent can now read the current-boot event ring as one
  `raios.evidence_response.v1` envelope whose ordered historical events are
  evidence records and whose only decision is
  `observed("recent_events_read")`; historical denials no longer decide the
  read.
- `event_log::recent_events_snapshot()` captures the window metadata and every
  selected event under one `LOG` lock. This closes the latent TOCTOU race in
  which `recent_events()` and repeated `recent_event()` calls could observe
  different ring states; capacity, eviction, and oldest-to-newest order are
  unchanged.
- The envelope has `event_id: null`. Every event record carries its own
  `status`, `reason`, `source_event_id`, and `classification`. Compact binding
  bodies remain byte-for-byte on their existing emitters under `facts.binding`;
  P4-4b2b owns their typed field-table conversion. The load-gate binding alone
  uses `project_load_gate_event_binding()` and intentionally drops its direct
  response decision. `ModulePromotionSignatureReference`, which has no compact
  emitter, is an explicit rejected `unknown_event` record rather than a silent
  skip.
- Stash reuse was limited to the atomic snapshot, `project_recent_events()`
  routing, and the load-gate projection-input adapter. The incomplete generic
  binding projection (including event-kind-as-`record_schema`) was deliberately
  not restored.

### Harness predicate accounting

| Bucket | Exact predicates/carriers |
| --- | --- |
| Regenerated | `protocol:memory_recent_events_schema`, `protocol:memory_recent_events_family` (replaces `protocol:memory_recent_events_record_schema`), `protocol:memory_recent_events_read_outcome`, `protocol:memory_recent_events_provider_request_binding_denied_outcome`, `protocol:memory_recent_events_provider_export_audit_outcome`, `protocol:memory_recent_events_request_denial_bindings`, `protocol:memory_recent_events_export_denial_bindings`, `protocol:audit_events_alias_schema`, `protocol:audit_events_denied_outcome`, `protocol:module_manifest_audit_outcome`, `protocol:module_manifest_audit_binding_schema`, `protocol:module_artifact_audit_outcome`, `protocol:module_artifact_audit_binding_schema`, `protocol:module_vm_report_audit_outcome`, `protocol:module_vm_report_audit_binding_schema`, `protocol:module_attestation_audit_outcome`, `protocol:module_attestation_audit_binding_schema`, `protocol:module_approval_audit_outcome`, `protocol:module_approval_audit_binding_schema`, `protocol:module_grant_audit_outcome`, `protocol:module_grant_audit_binding_schema`, `protocol:module_audit_rollback_audit_outcome`, `protocol:module_audit_rollback_audit_binding_schema`, `protocol:module_service_slot_audit_outcome`, `protocol:module_service_slot_audit_binding_schema`, `protocol:module_load_audit_binding_schema`, `quick:audit_events_schema`; `full-module-evidence` invocation carrier now reads `evidence[].facts.binding`. |
| Honest merge | Quick-profile event-path checks share one explicit v1-to-check-record adapter after directly asserting schema/family/null-event-id/observed-read decision. Merged names: `envelopeAuditEvents`, `acceptedEnvelopeEvent`, `systemSnapshotEnvelopeEvent`, `bootLogEnvelopeEvent`, `deviceGraphEnvelopeEvent`, `serviceInventoryEnvelopeEvent`, `systemCapabilitiesEnvelopeEvent`, `problemListEnvelopeEvent`, `badEnvelopeEvent`, `overCapEnvelopeEvent`, `mismatchEnvelopeEvent`, `helloEvents`, `helloStateEvents`, `helloResetDeniedEvents`, `helloRestartEvents`, `helloHotSwapEvents`, `helloHotSwapV2Events`, `helloHotSwapV2MigrationEvents`, `helloHotSwapV2ProbationEvents`, `helloRollbackPreviewEvents`, `helloRollbackApplyEvents`, `helloRollbackApplyWriterStorageFoundationEvents`, `helloRollbackApplyWriterReadinessEvents`, `helloRollbackApplyWriterWritePathGateEvents`, `helloDescriptorEvents`, `helloDescriptorHashEvents`, `helloDescriptorSourceEvents`, `helloDescriptorEnvelopeEvents`, `helloArtifactIdentityEvents`, `helloArtifactContentEvents`, `helloArtifactReferenceEvents`, `helloLoadPlanPreflightEvents`, `helloServiceSlotActivationEvents`, `helloServiceSlotActivationStatuses`, `hostDescriptorHashEvents`, `hostDescriptorSourceEvents`, `hostLoadPlanPreflightEvents`, `hostServiceSlotActivationEvents`, `hostServiceSlotActivationStatuses`, `helloHealthEvents`, `helloHealthStateEvents`, `helloHealthEnvelopeEvents`, `helloHealthArtifactIdentityEvents`, `helloHealthArtifactContentEvents`, `helloHealthArtifactReferenceEvents`, `helloHealthLoadPlanPreflightEvents`, `helloHealthServiceSlotActivationEvents`, `helloHealthServiceSlotActivationStatuses`, `hostHealthEvents`, `hostHealthLoadPlanPreflightEvents`, `hostHealthServiceSlotActivationEvents`. |
| Retired | None. |

Donor exposure verdict: quick-profile legacy event paths in the former
2687-2914 block were broken by the envelope and are carried by the named honest
merge above; compact binding leaf checks remain unchanged. Provider-memory
lines 102, 103, and 143 were broken and regenerated to v1 schema/family/schema.
Module-evidence lines 635-636 were broken and now select
`evidence[].facts.binding`. No listed donor exposure remains merely reported
outside the write set.

## P4-4b2a-fix notes

The full-audit load-gate checks now select the last
`binding.record_schema == raios.module_load_gate.v0` serial line and search
only that line. This prevents a response decision or another event from
donating a whole-log match. The binding has no decision; its twelve ordered
evidence records are the authority for prerequisite status and reason.

### Exact 63-predicate bucket table

| Old predicate | Bucket | v1 carrier / replacement |
| --- | --- | --- |
| `protocol:module_load_audit_binding_status` | RETIRED | No binding-wide status exists; replaced by per-record `evidence[].status` and `reason`. Coverage carrier: `module_manifest` id + status + reason. |
| `protocol:module_load_audit_requirements_schema` | RETIRED | The requirements object is gone; replaced by the twelve ordered evidence records. Coverage carrier: `candidate_artifact` id + status + reason. |
| `protocol:module_load_audit_requirements_no_load` | RETIRED | No requirements-wide status exists; blockers are individual evidence statuses/reasons. Coverage carrier: `local_approval` id + status + reason. |
| `protocol:module_load_audit_retained_grant_state` | HONEST MERGE | Merged with `protocol:module_load_audit_retained_grant_binding` into `protocol:module_load_audit_computed_grant_evidence` -> `evidence[id=computed_capability_grant]` id + status + reason. |
| `protocol:module_load_audit_retained_grant_binding` | HONEST MERGE | Same carrier as the preceding row; no second predicate retained. |
| `protocol:module_load_audit_retained_vm_report_state` | HONEST MERGE | Merged with `protocol:module_load_audit_retained_vm_report_binding` into `protocol:module_load_audit_vm_test_report_evidence` -> `evidence[id=vm_test_report]` id + status + reason. |
| `protocol:module_load_audit_retained_vm_report_binding` | HONEST MERGE | Same carrier as the preceding row; no second predicate retained. |
| `protocol:module_load_audit_retained_attestation_state` | HONEST MERGE | Merged with `protocol:module_load_audit_retained_attestation_binding` into `protocol:module_load_audit_local_attestation_evidence` -> `evidence[id=local_attestation]` id + status + reason. |
| `protocol:module_load_audit_retained_attestation_binding` | HONEST MERGE | Same carrier as the preceding row; no second predicate retained. |
| `protocol:module_load_audit_retained_audit_rollback_binding` | RETIRED | The combined reference is gone; replaced by separate `durable_audit_record` and `rollback_plan` evidence carriers. |
| `protocol:module_load_audit_retained_audit_state` | REGENERATED | `protocol:module_load_audit_durable_audit_record_evidence` -> `evidence[id=durable_audit_record]` id + status + reason. |
| `protocol:module_load_audit_retained_rollback_state` | REGENERATED | `protocol:module_load_audit_rollback_plan_evidence` -> `evidence[id=rollback_plan]` id + status + reason. |
| `protocol:module_load_audit_retained_service_slot_state` | HONEST MERGE | Merged with `protocol:module_load_audit_retained_service_slot_binding` into `protocol:module_load_audit_service_slot_evidence` -> `evidence[id=service_slot]` id + status + reason. |
| `protocol:module_load_audit_retained_service_slot_binding` | HONEST MERGE | Same carrier as the preceding row; no second predicate retained. |
| `protocol:module_load_audit_service_slot_allocator_state` | REGENERATED | `protocol:module_load_audit_service_slot_allocator_evidence` -> `evidence[id=service_slot_allocator]` id + status + reason. |
| `protocol:module_load_audit_service_slot_allocator_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.record_schema`. |
| `protocol:module_load_audit_service_slot_allocator_authority_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.allocator_authority_boundary.record_schema`. |
| `protocol:module_load_audit_service_slot_allocation_intent_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.allocation_intent_boundary.record_schema`. |
| `protocol:module_load_audit_policy_decision_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_input_boundaries.policy_decision.record_schema`. |
| `protocol:module_load_audit_registry_write_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_input_boundaries.registry_write_authority.record_schema`. |
| `protocol:module_load_audit_loader_contract_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_input_boundaries.loader_runtime_contract.record_schema`. |
| `protocol:module_load_audit_health_monitor_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_input_boundaries.health_monitor_binding.record_schema`. |
| `protocol:module_load_audit_cleanup_authority_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_input_boundaries.unload_cleanup_authority.record_schema`. |
| `protocol:module_load_audit_authority_decision_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.authority_decision.record_schema`. |
| `protocol:module_load_audit_registry_commit_gate_schema` | REGENERATED | `evidence[id=service_slot_allocator].facts.registry_write_commit_gate.record_schema`. |
| `protocol:module_load_audit_loader_runtime_state` | REGENERATED | `protocol:module_load_audit_loader_runtime_evidence` -> `evidence[id=loader_runtime]` id + status + reason. |
| `protocol:module_load_audit_loader_runtime_schema` | REGENERATED | `evidence[id=loader_runtime].facts.record_schema`. |
| `protocol:module_load_audit_loader_runtime_execution_commit_gate` | REGENERATED | `evidence[id=loader_runtime].facts.execution_commit_gate.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_intake_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_intake_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_artifact_byte_intake_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.artifact_byte_intake_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_execution_authorization_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.execution_authorization_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_registry_mutation_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_registry_mutation_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_load_attempt_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.load_attempt_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_artifact_load_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.artifact_load_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_mapping_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_mapping_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_entrypoint_transfer_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.entrypoint_transfer_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_start_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_start_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_health_binding_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_health_binding_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_running_state_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_running_state_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_start_audit_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_start_audit_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_service_unload_cleanup_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.service_unload_cleanup_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_live_load_commit_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.live_load_commit_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_commit_audit_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.commit_audit_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_commit_rollback_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.commit_rollback_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_commit_result_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.commit_result_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_acceptance_authority_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_parser_contract_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_parser_result_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_schema_validation_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_capability_validation_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_load_plan_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_load_plan_authority_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_load_plan_result_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_image_layout_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_image_layout_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_page_mapping_plan_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_page_mapping_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.descriptor_executable_page_binding_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_entrypoint_binding_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_entrypoint_transfer_authorization_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_entrypoint_transfer_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_entrypoint_handoff_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_executable_entrypoint_invocation_boundary` | REGENERATED | `evidence[id=loader_runtime].facts.executable_entrypoint_invocation_boundary.record_schema`. |
| `protocol:module_load_audit_loader_runtime_no_load` | RETIRED | The old `loader_runtime_readiness` wrapper is gone; replaced by `protocol:module_load_audit_loader_evidence` -> `evidence[id=loader]` id + status + reason. |

Coverage after merges is explicit for all twelve prerequisites: `module_manifest`,
`candidate_artifact`, `vm_test_report`, `local_attestation`, `local_approval`,
`computed_capability_grant`, `durable_audit_record`, `rollback_plan`,
`service_slot`, `service_slot_allocator`, `loader_runtime`, and `loader` each
have a binding-scoped id + status + reason predicate.

No one of the 63 failing predicates pinned a hash. The existing byte-identical
hash needles were deliberately left untouched: computed grant, local
attestation reference, VM report reference/report, audit record, rollback plan,
reservation, and service-slot reservation hashes. Every other non-listed
full-audit needle was also left untouched because the orchestrator's live-log
failure set was exact and its carrier bytes did not move.
