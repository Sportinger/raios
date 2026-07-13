# P4-5a — memory semantic manifest

Read-only inventory for P4-5. No Rust or PowerShell emitter may be deleted from
the memory family until this mapping is represented by host tests and the
orchestrator has resolved the decisions called out below.

Notation:

- `R` = legacy `body.result`; legacy mutation denials use `body` directly.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

Boundary ruling applied: the entire `memory.recent_events` response, including
its event-ring metadata and event records, belongs to P4-4. It is inventoried in
`p4-4-event-evidence-semantic-manifest-2026-07-13.md` and is not claimed or
counted here. P4-5 owns `memory.profile`, `memory.context`, `memory.query`,
`memory.trace`, the generic memory-mutation denials, and durable-memory
record/denial projections. The P4-4 manifest's former P4-5 boundary decision is
therefore resolved by the orchestrator ruling supplied with this packet.

## 1. Response-path inventory

### Dispatch and transport

`seed-kernel/src/agent_protocol.rs:348-353,368-372,544-548` dispatches the
in-scope methods. Normal responses use `begin_response()` / `end_response()` and
the legacy envelope order `v`, `t`, `id`, `body`, then `body.method`,
`body.result`. `emit_memory_capability_denied()` writes the same envelope by hand
with `t:error` and fields directly under `body`. The literal
`RAIOS_AGENT_BEGIN` count is one because normal responses use the helper.

### `memory.profile`

Emitter: `seed-kernel/src/agent_protocol_memory.rs:24-42`.

Current `R` field order:

```text
schema, scope, profiles, read_methods, mutation_policy
```

`profiles` is ordered `diagnostic`, `planning`, `provider_minimal`.

```text
diagnostic/planning:
 id, available, target_tokens, provider_export, summary

provider_minimal:
 id, available, local_projection, target_tokens, provider_export,
 blocked_by, summary
```

`read_methods` is ordered `memory.context`, `memory.query`, `memory.trace`,
`memory.recent_events`, `audit.events`. The last two are advertised methods, not
ownership of their P4-4 response content.

### `memory.context [diagnostic|planning|provider_minimal]`

Emitter: `agent_protocol_memory.rs:44-230`; pure profile/budget plan:
`raios-core/src/memory_context.rs`. Acquisition combines one `SystemSnapshot`,
one provider snapshot, the dispatch-acquired event ID, static locators, and a
durable RECLOG projection.

Current `R` field order:

```text
schema, purpose, profile, scope, provider_export,
context_event_id, audit_event_id, source_schemas, budget,
authority_order, included, current,
[provider_projection only for provider_minimal],
records, durable_records, omitted
```

Nested order:

```text
budget:
 target_tokens, estimated_tokens

included:
 identity, policy, current, summaries

current:
 snapshot_id, status, provider_trust_state, provider_api_key_state,
 capability_posture, services, problems

current.status:
 framebuffer, entropy, usb_xhci, wifi, network, input

records[i]:
 id, kind, authority, classification, summary, source

durable_records[i]:
 id, kind, entity, predicate, classification, authority, scope, exportable

omitted static entries:
 kind, reason

omitted durable fold variants, in evaluator/projection order:
 durable_superseded {kind, ids}
 durable_r1_ignored_supersede {kind, reason, links[{source_id,target_id}]}
 durable_dangling_supersede {kind, ids}
 durable_audit_id_reused {kind, ids}
 durable_records_over_budget {kind}
 durable_frame_dropped {kind, count}
 durable_local_only_value {kind}
```

The optional `provider_projection` is built by provider-owned projection code.
Its presence in the context packet is P4-5 selection/enveloping; its internal
trust, redaction, field-classification, and packet-hash semantics remain P4-8
provider ownership. See Decision 1 below.

The eight static `records` entries, in order, are:

```text
mem.fact.identity.stage0
snapshot.current
capabilities.current_boot
service.inventory.current
problem.list.current
boot_log.summary.current
adr.0001
adr.0004
```

`summary` records and any future RAG/semantic result are locators only. They do
not authorize, grant, rank authority above their cited source, or become
decision evidence merely by being selected.

### `memory.query`

Emitter: `agent_protocol_memory.rs:232-304`.

Current `R` field order:

```text
schema, scope, query, records, semantic_index
```

Each ordered `records[i]` is `id, kind, classification, summary`. The nine IDs
are the context list plus `snapshot.current.provider_minimal` after
`snapshot.current`. `semantic_index` is the constant
`not_implemented_locator_only`: this is a static locator index, not semantic
authority and not a claim that RAG exists.

### `memory.trace [id]`

Emitter: `agent_protocol_memory.rs:306-362,11181-11271`.

Current `R` order:

```text
schema, requested_id, scope, records
```

An omitted ID returns six ordered default locators: `snapshot.current`, its
provider-minimal projection, service inventory, problem list, ADR 0001, ADR
0004. A known ID returns one record with `id, found:true, source_method, source`.
An unknown ID returns `id, found:false, reason`. Trace results are provenance
locators only; a source file path or method name is not authority.

### Generic durable-memory mutation denials

Methods: `memory.record_observation`, `memory.propose_policy`,
`memory.supersede_fact`, `memory.redact`, and `memory.compact`.

Emitter: `agent_protocol_memory.rs:405-436`. Current direct `body` order:

```text
method, event_id, audit_event_id, code, message, required
```

`required` is ordered `raios.audit_record.v0`, `policy_ledger`,
`source_retention`, `redaction_transaction`, `raios.memory_persistence.v0`,
`rollback_plan`. All five methods are absolute denials and grant/effect nothing.

### Durable-memory record/denial surfaces

The shared record model is `raios-core/src/memory_record.rs`; append evaluator is
`raios-core/src/scoped_memory_record_append.rs`; kernel acquisition and I/O stay
in `seed-kernel/src/durable_store.rs`; response construction is in
`seed-kernel/src/memory_store.rs`.

In-scope responses are:

```text
memory.record_log_append
memory.decision_problem_log_append
memory.observation_log_append
memory.provider_export_public_fixture_append (test infrastructure record append)
memory.record_log_append_selftest
memory.broker_resolve_selftest
```

Provider export denial/audit and Wasm import-grant decisions may carry durable
memory append evidence, but their owning responses remain P4-8/provider and
loader/runtime families. P4-5 owns only the shared durable record/evidence
projection and how their metadata appears in `memory.context`.

Single append success order after `schema, query_method`:

```text
durable_append, performed, reason, authority,
record_id, kind, classification, record_authority,
record_schema, region_marker, target_id,
seq, write_offset, frame_len,
payload_sha256, frame_sha256, readback_sha256, reparse_valid,
tail_seq_before, count_before, tail_seq_after, count_after,
owner_sealed, persistence_claimed, trust_tier
```

Constructor/preflight denial order is:

```text
schema, query_method, durable_append, performed, reason, authority,
record_schema, region_marker, target_id, trust_tier,
owner_sealed, persistence_claimed
```

`memory.decision_problem_log_append` returns `schema, query_method,
durable_append, performed, reason, records`; each of its three ordered records
(A, P, B) carries the shared append fields plus `supersedes`. Its construction
denial adds `authority`, `records:[]`, `owner_sealed`,
`persistence_claimed` after the common status fields.

The durable record bytes are independently canonical and must not change:
`schema, id, kind, entity, predicate, value, classification, authority, boot_id,
sequence, source, evidence, tags, supersedes, created_at`
(`memory_record.rs:314-345,389-405`). The response migration does not migrate or
rehash these stored bytes.

## 2. Semantic mapping

### Legacy envelope

```text
legacy v/t/id/body/method
  -> retired(shared raios.evidence_response.v1 envelope)
legacy schema fields
  -> retired(replaced by constant(raios.evidence_response.v1))
legacy response method
  -> constant(source_method)
legacy response event/audit duplicate IDs
  -> envelope event_id from the one dispatch-acquired event
```

### `memory.profile`

```text
R.scope                                      -> envelope scope
R.profiles                                   -> F.profiles (same order)
R.profiles[i].id                             -> F.profiles[i].id
R.profiles[i].available                      -> F.profiles[i].available
R.profiles[i].local_projection               -> F.profiles[i].local_projection (explicit null where inapplicable)
R.profiles[i].target_tokens                  -> F.profiles[i].target_tokens
R.profiles[i].provider_export                -> F.profiles[i].provider_export
R.profiles[i].blocked_by                     -> E[profile.provider_export_omission].reason for provider_minimal
R.profiles[i].summary                        -> F.profiles[i].summary (locator description only)
R.read_methods                               -> F.read_methods
R.mutation_policy                            -> F.mutation_policy
                                              + D=Observed("memory_profile_returned")
```

The profile response describes policy; it does not authorize export or mutation.

### `memory.context`

```text
R.purpose                                    -> F.purpose
R.profile                                    -> F.profile
R.scope                                      -> envelope scope
R.provider_export                            -> F.provider_export
R.context_event_id                           -> envelope event_id
R.audit_event_id                             -> retired(duplicate of the same envelope event_id)
R.source_schemas                             -> F.source_schemas
R.budget.target_tokens                       -> F.budget.target_tokens
R.budget.estimated_tokens                    -> F.budget.estimated_tokens
R.authority_order                            -> F.authority_order
R.included.identity                          -> F.included.identity
R.included.policy                            -> F.included.policy
R.included.current                           -> F.included.current
R.included.summaries                         -> F.included.summaries (locator IDs only)
R.current.*                                  -> F.current.* from the same captured snapshots
R.provider_projection                        -> F.provider_projection (provider-owned typed projection; Decision 1)
R.records[i].id/kind/authority/classification-> E[record.<id>].facts.*
R.records[i].summary/source                  -> E[record.<id>].facts.summary/source_locator
R.durable_records[i].*                       -> E[durable_record.<id>].facts.*
R.omitted[i].kind                            -> E[omission.<ordered-index>].facts.kind
R.omitted[i].reason                          -> E[omission.<ordered-index>].reason
R.omitted[i].ids/links/count                 -> E[omission.<ordered-index>].facts.ids/links/count
                                              + D=Observed("memory_context_returned")
```

Every included static or durable record is an ordered locator evidence unit. Its
`status` is `present`, its `reason` is `selected_for_context`, and its
classification is copied from the selected record. Omission evidence uses
`status:not_applicable` for deliberate budget/redaction/export exclusions,
`status:rejected` for invalid/dropped durable frames or forbidden supersede
attempts, and the existing precise reason. No omitted item is silently dropped.

Summaries, query candidates, trace hits, and future semantic/RAG hits are
locators only. They can identify `source_locator` or `source_event_id`, but may
not populate `grants`, `effects`, an authority hash, or an authorizing decision.

### `memory.query`

```text
R.scope                                      -> envelope scope
R.query                                      -> F.query
R.records[i].id                              -> E[query_locator.<id>].facts.record_id
R.records[i].kind                            -> E[query_locator.<id>].facts.kind
R.records[i].classification                  -> E[query_locator.<id>].classification
R.records[i].summary                         -> E[query_locator.<id>].facts.summary
R.semantic_index                             -> F.semantic_index
                                              + constant(locator_only=true)
                                              + D=Observed("memory_query_returned")
```

### `memory.trace`

```text
R.requested_id                               -> F.requested_id (explicit null when absent)
R.scope                                      -> envelope scope
R.records[i].id                              -> E[trace.<id>].facts.record_id
R.records[i].found                           -> E[trace.<id>].status present|missing
R.records[i].source_method                   -> E[trace.<id>].facts.source_method
R.records[i].source                          -> E[trace.<id>].facts.source_locator
R.records[i].reason                          -> E[trace.<id>].reason
                                              + D=Observed("memory_trace_returned")
```

An unknown locator is observed missing data, not a capability denial.

### Generic memory-mutation denials

```text
body.method                                  -> envelope source_method
body.event_id                                -> envelope event_id
body.audit_event_id                          -> retired(duplicate event_id)
body.code                                    -> D.outcome="denied"
body.message                                 -> F.message
body.required                                -> ordered E[required.<index>] records
D.reason                                     -> first missing prerequisite reason
D.requested_capability                       -> constant(cap.memory.mutate)
D.grants                                     -> constant([])
D.effects                                    -> constant([])
D.blocked_by                                 -> same ordered prerequisite evidence IDs/reasons
```

The current emitter has one generic message and no per-prerequisite evaluator
status. P4-5b must not invent a false first failure. It must either introduce a
typed evaluator returning the ordered missing prerequisites and first reason, or
retain a stable generic reason such as `durable_memory_authority_missing` and
list all required evidence as missing. This is Decision 2.

### Durable append responses

```text
R.query_method                               -> envelope source_method
R.durable_append                             -> E[durable_append].facts.status_detail
R.performed                                  -> D.outcome granted|denied and E[durable_append].facts.performed
R.reason                                     -> D.reason and E[durable_append].reason
R.authority                                  -> E[durable_append].facts.authority
R.record_id/kind/classification              -> E[memory_record].facts.* / classification
R.record_authority                           -> E[memory_record].facts.authority
R.record_schema                              -> E[memory_record].facts.record_schema
R.region_marker/target_id/trust_tier         -> E[durable_append].facts.*
R.seq/write_offset/frame_len                 -> E[durable_append].facts.*
R.payload_sha256                             -> E[memory_record].facts.payload_sha256
R.frame_sha256/readback_sha256/reparse_valid -> E[durable_append].facts.*
R.tail_seq_before/count_before               -> F.store_before.*
R.tail_seq_after/count_after                 -> F.store_after.*
R.owner_sealed/persistence_claimed           -> F.posture.*
R.records                                    -> ordered E[memory_record.<index>] append projections
R.records[i].supersedes                      -> E[memory_record.<index>].facts.supersedes
```

A denied append maps to `D` with the evaluator's first-failure reason,
`grants:[]`, `effects:[]`, and ordered `blocked_by`. A performed append is a real
scoped mutation. It may use a granted decision only from the existing
`evaluate_scoped_memory_record_append()` positive result and must describe the
exact durable append effect; an emitter cannot derive a grant from `performed`
alone. Constructor/input denials occur before that evaluator and remain denied
with no effects.

## 3. Evidence-unit design and ownership

### Ordered evidence

For context/query/trace, preserve this ordering:

1. static selected record locators in their existing declared order;
2. durable visible records in ascending RECLOG frame sequence;
3. omission records in the current fold order;
4. within a fold, IDs/links retain evaluator order.

`resolve_durable_memory()` owns durable ordering and fold semantics:

- LOW-1 and identity selection: `memory_record_resolve.rs:53-99`;
- deterministic ascending frame sequence: lines 101-102;
- R1 supersede/dangling order: lines 104-144;
- visible projection order: lines 146-161.

The append denial evaluator's first-failure order is source order in
`scoped_memory_record_append.rs:125-340`: method, target, schema, region, frame
shape/bounds, sequence, previous hash, payload/frame hashes, write/readback,
classification, kind/supersede rules, agent confinement, trust posture, quota.
P4-5b must consume that order; it must not reconstruct or sort blockers in the
renderer.

Evidence IDs must be stable and distinguish source:

```text
record.<static-id>
durable_record.<record-id>
query_locator.<record-id>
trace.<requested-id>
omission.<zero-padded-index>
append.input.<prerequisite>
durable_append
memory_record
```

The envelope `event_id` is acquisition provenance. An evidence
`source_event_id` is the event that sourced that evidence record. They are not
interchangeable. Validity and presence also remain distinct: a record locator's
presence does not prove its canonical durable frame or authorization valid.

### Kernel-owned

The kernel retains:

- dispatch, framing, command/argument acquisition, and the event-log mutex;
- live `SystemSnapshot`, provider snapshot, service/problem acquisition;
- durable-device scan, frame read/write/readback, quota state, and append call;
- the mutation point and the exact dispatch-acquired event ID;
- secret custody and classification enforcement at export boundaries.

### W8/core relocation candidates

Move only pure projections/evaluators while touching this family:

- profile selection and token budget plan already in `memory_context.rs`;
- static locator tables for profile/context/query/trace;
- typed context selection result containing included and omitted records;
- durable `MemoryRecordView` to metadata-only evidence projection;
- omission-reason projection over `ResolvedMemory`;
- typed first-failure decision adapters over the existing append evaluator.

Do not move kernel locks, I/O, snapshots, mutable quota, provider secrets, or
append execution into `raios-core`. Do not create a second serializer, generic
policy engine, semantic index, or RAG implementation.

## 4. Predicate inventory

Only predicates asserting P4-5 responses or their durable-memory projections
are counted. All `memory.recent_events`/`audit.events` predicates belong to P4-4
and are excluded. Direct provider, Wasm, durable-scan, and service predicates
were reviewed for donor/cross-family effects; only those consuming P4-5 append
or context fields are counted.

Runtime counts expand:

- the 10-entry append field loop;
- the 8-entry supersede check loop for each of A/P/B (24);
- the 11-entry authorized observation loop;
- the five malformed observation cases;
- helper invocations for five boot-1 append records;
- repeated child-VM command arrays and mutation-denial helpers.

| Profile file | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| `shadow-vm-smoke-profile-common.ps1` | 37 | 5 | 3 | 45 |
| `shadow-vm-smoke-profile-full-provider-memory.ps1` | 3 | 12 | 7 | 22 |
| `shadow-vm-smoke-profile-memory-durable.ps1` | 0 | 100 | 27 | 127 |
| `shadow-vm-persistence-reboot.ps1` | 8 | 17 | 5 | 30 |
| **Total** | **48** | **134** | **42** | **224** |

The measured 224 reviewed / 134 regenerated falls inside the design estimate
of 180-320 / 110-220.

Leaf assertions survive only when the same leaf pair retains the same semantic
source: profile IDs/budgets, locator IDs, trace source method, classification,
hash values, and durable scan facts. Parsed paths under `body.result` do not
survive merely because the scalar value is unchanged.

Regenerate legacy schema assertions, direct `body` denial paths, context/audit
event-ID duplicates, every parsed `body.result` append/context/selftest path,
and any assertion that treats `performed`, `provider_export`, `found`, or an
omission kind as a decision without its specific evidence anchor.

All 42 completion waits for their existing `RAIOS_AGENT_END <method>` marker
survive. P4-4 owns every completion wait ending in
`RAIOS_AGENT_END memory.recent_events`; those are excluded here.

### Collapse hazards and distinguishing needles

| Legacy assertion | Required distinguishing v1 needle |
|---|---|
| context/audit event ID | envelope `event_id`; never span or match response `id` |
| static/durable record ID | one `E[record-id]` InlineObject with `status` and classification |
| summary/query/trace hit | evidence ID + `kind:"locator"`; never an authority or grant needle |
| included versus omitted | distinct evidence status/reason, not ID presence alone |
| local-only versus public | same evidence record's classification, not a whole-log bare classification |
| append denial | complete `D` denial with first reason, requested capability, empty grants/effects |
| append success | append evidence ID + evaluator-created granted decision/effect |
| payload/frame/readback hash | evidence ID plus the specific hash key; never a bare `sha256:` |
| envelope event versus source event | `event_id` versus evidence `source_event_id`; never substitute one for the other |
| record presence versus validity | evidence `status:present` versus `status:verified`; use the source accessor being tested |

Serialization rules for regenerated needles:

- top-level envelope fields arrive through serial as CR CR LF separated lines;
  multi-line PowerShell needles spell separators as `` `r`r`n ``;
- no needle spans the top-level response `id` field;
- `facts`, `evidence`, and `decision` are single-line `InlineObject`s;
- single-line needles stay inside one such object;
- same-source checks bind validity versus presence and envelope `event_id`
  versus evidence `source_event_id` explicitly.

Safe examples:

```powershell
'"id": "trace.snapshot.current", "kind": "locator", "status": "present"'
'"id": "omission.00000003", "status": "not_applicable", "reason": "secret_values_never_included"'
'"id": "durable_append", "kind": "write_evidence", "status": "verified", "reason": "authorized_memory_record_append_readback_reparse_verified"'
```

### Required scan evidence

Emitter marker scan:

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/agent_protocol_memory.rs
1
```

Harness sweep:

```text
> rg -l "memory\.(profile|context|query|trace)|memory_durable" vm-harness/
vm-harness/shadow-vm-smoke-profile-full-provider-memory.ps1
vm-harness/shadow-vm-smoke-profile-common.ps1
vm-harness/shadow-vm-persistence-reboot.ps1
vm-harness/shadow-vm-smoke-profile-memory-durable.ps1
```

Needle-count scan (the output is the expanded inventory above, not merely
literal assertion-line counts):

```text
> $rows=@(@('shadow-vm-smoke-profile-common.ps1',37,5,3),@('shadow-vm-smoke-profile-full-provider-memory.ps1',3,12,7),@('shadow-vm-smoke-profile-memory-durable.ps1',0,100,27),@('shadow-vm-persistence-reboot.ps1',8,17,5)); foreach($r in $rows){"$($r[0]) leaf=$($r[1]) regen=$($r[2]) framing=$($r[3]) assertions=$($r[1]+$r[2]+$r[3])"}
shadow-vm-smoke-profile-common.ps1 leaf=37 regen=5 framing=3 assertions=45
shadow-vm-smoke-profile-full-provider-memory.ps1 leaf=3 regen=12 framing=7 assertions=22
shadow-vm-smoke-profile-memory-durable.ps1 leaf=0 regen=100 framing=27 assertions=127
shadow-vm-persistence-reboot.ps1 leaf=8 regen=17 framing=5 assertions=30
```

Literal/helper expansion audit used to prevent undercounting the dominant
profile:

```text
> $f='vm-harness/shadow-vm-smoke-profile-memory-durable.ps1'; "add_literal=$((Select-String -Path $f -Pattern '^\s*Add-Predicate').Count)"; "append_loop=10 supersede_loop=24 observation_loop=11 malformed_loop=5 mutation_helper_memory_calls=3"
add_literal=72
append_loop=10 supersede_loop=24 observation_loop=11 malformed_loop=5 mutation_helper_memory_calls=3
```

## 5. Selftest strategy

Existing host selftests:

- `raios-core/src/memory_context.rs`: mutation vocabulary denied, profiles and
  budgets, event-limit parsing, and no export/mutation authorization;
- `memory_record.rs`: constructor invariants, canonical serialization and pinned
  hash, parser field order/errors, classifications, kinds, and record shape;
- `memory_record_resolve.rs`: LOW-1 frame ordering, R1 audit immutability,
  supersede/dangling behavior, audit ID shadowing, and LOW-3 source spoofing;
- `scoped_memory_record_append.rs`: positive append and exhaustive ordered
  mutation/denial cases;
- `durable_record_frame.rs`: RECLOG walking and memory-record reparsing.

Existing guest selftests:

- `memory.record_log_append_selftest`: constructor, scoped append, quota,
  classification, kind, supersede, source/status denials; RAM-only;
- `memory.broker_resolve_selftest`: R1, LOW-1, audit ID shadow, LOW-3; RAM-only.

P4-5b adds the smallest host semantic set:

1. one exhaustive mapping test per response group;
2. profile/context/query/trace present, missing, and provider-minimal fixtures;
3. token budget and included/omitted ordering from one immutable selection;
4. locator-only rule: summaries/semantic hits cannot construct decisions;
5. public/local-only classification fixtures and secret exclusion;
6. durable visible/fold ordering from the existing resolver;
7. append denial first-failure order and absolute empty grants/effects;
8. positive append requires the evaluator-created proof and preserves effect;
9. canonical durable record payload and all pinned hashes remain unchanged;
10. envelope/framing render through the existing `record::Value` path.

Guest regeneration should retain the current focused `memory-durable` profile
and only a small integration set for profile, diagnostic/provider-minimal
context, query, trace found/missing, one denied mutation, one denied append, one
verified append, durable fold visibility, and both existing selftests. This
packet runs no VM or Cargo command.

## 6. Risks and P4-5b STOP-tripwires

1. **Provider projection ownership — OWNER/ORCHESTRATOR DECISION NEEDED.** P4-5
   owns selection and placement of `provider_projection`; P4-8 owns its internal
   trust/redaction/hash projection. Choose whether P4-5 temporarily embeds the
   existing typed provider value or P4-8 lands first. Duplicating its mapping or
   serializer is a STOP.
2. **Mutation first failure — OWNER/ORCHESTRATOR DECISION NEEDED.** The generic
   denial currently lists prerequisites but has no evaluator-owned individual
   statuses. Choose a stable generic first reason or authorize a small typed
   evaluator. Inventing ordered failures in the emitter is a STOP.
3. **Positive durable append vocabulary — OWNER/ORCHESTRATOR DECISION NEEDED.**
   A real performed append is an effect, while the family-table shorthand says
   “preserve durable denial wiring.” Confirm the requested capability/effect
   names for the existing narrow positive append; converting it to Observed or
   deriving a grant from booleans is a STOP.
4. **Classification correctness.** `public` and `local_only` are copied from the
   selected record; `secret` must never enter durable memory or provider output.
   Defaulting a missing/unknown classification to public, merging whole-response
   classification with mixed record classifications, or exporting local-only
   detail is a STOP.
5. **Locator-only authority.** Summaries, query candidates, trace hits, source
   paths, and future RAG results locate records only. Using relevance, presence,
   or prose as grant evidence is a STOP.
6. **Durable hash stability.** `memory_record.rs:314-355` serializes and hashes
   the stored record; its pinned test is at lines 1035-1048. Response-vocabulary
   work must not alter record bytes, RECLOG frames, payload/frame/readback hashes,
   supersede identity, or persistence goldens. Regenerating those hashes because
   response JSON moved is a STOP.
7. **First-failure and effects.** Denials use evaluator order with
   `grants:[]`, `effects:[]`. A verified append effect requires positive evaluator
   proof. Reconstructing either decision from rendered fields is a STOP.
8. **P4-4 boundary.** `emit_recent_events`, event-ring metadata, event records,
   event bindings, and all `audit.events` needles remain P4-4. P4-5 may advertise
   or locate that method but may not regenerate or delete its content. Double
   ownership is a STOP.
9. **Snapshot coherence.** Context currently combines separately acquired
   system/provider/durable views. W8 relocation needs one typed selection input;
   core may not reread mutable kernel state or invent event provenance.
10. **Provider-minimal mixed classification.** The response contains local-only
    facts plus a public/redacted provider projection. One envelope
    `classification` cannot falsely label every nested record public. Mixed
    classification policy requires explicit decision before deletion.
11. **Omission semantics.** Budget, redaction, invalid frame, supersede, and
    provider-export omissions have different reasons/statuses. Collapsing them
    into one count or silently dropping `ids`, `links`, or `count` is a STOP.
12. **Persistence is real, not a denial-only surface.** The narrow append paths
    perform durable writes today. P4-5 must preserve real positive behavior and
    fail-closed denials; treating all durable memory as denied contradicts code
    and focused VM evidence.
13. **Needle donor collapse.** Bare `classification`, `reason`, `performed`,
    `event_id`, or hash needles can pass from provider/event/durable-scan output.
    Every regenerated assertion must be response-scoped and evidence-specific.
14. **Write-set scope.** P4-5b will need explicit authorization for
    `memory_store.rs` and selected core projection modules in addition to
    `agent_protocol_memory.rs`. Deleting only the design-row headline file
    cannot migrate durable response surfaces.

Scope-creep observations:

- `seed-kernel/src/memory_store.rs`, `durable_store.rs`,
  `raios-core/src/memory_record_resolve.rs`, and
  `scoped_memory_record_append.rs` are material family dependencies not fully
  named by the P4 row. Most need reading and host tests; kernel I/O need not move.
- `agent_protocol_memory.rs` is 11,272 lines chiefly because it also contains the
  P4-4 event/binding renderer. P4-5 deletion estimates must exclude those lines.
- Provider export and Wasm responses embed shared durable append evidence. Their
  owner slices should reuse one P4-5 projection rather than copy it, but P4-5
  must not absorb their decisions.
- `memory_context.rs` still parses `memory.recent_events` limits for shared
  command policy. That helper dependency does not transfer event response
  ownership back to P4-5.

## Orchestrator rulings (2026-07-13, binding for P4-5b)

**M1 — provider projection: EMBED the existing typed value; do not re-derive it.**
P4-8 has not landed. P4-5 owns WHERE the provider projection sits inside the
memory response; P4-8 owns WHAT is inside it. So P4-5 places the EXISTING typed
provider `Value` unchanged (it is already a typed record — no mapping, no
serializer, no re-derivation), and P4-8 later converts its internals in place.
Duplicating its mapping or hashing it again is a STOP. This deliberately avoids a
landing-order deadlock without either slice reaching into the other.

**M2 — the mutation prerequisites get a SMALL TYPED CORE EVALUATOR.** The generic
denial lists prerequisites but has no evaluator-owned per-item statuses, and the
manifest is right that inventing ordered failures in the emitter is a STOP. The
answer is not "pick a stable generic first reason" — that is the emitter guessing
which gate failed. Authorize a small typed evaluator in raios-core that owns the
ordered prerequisite statuses and the first failure, exactly as every landed
family does (core owns order; the emitter only renders). It is fail-closed by
construction: no positive/authorizing outcome may be constructible from it.

**M3 — the positive durable append stays GRANTED, from the evaluator that already
exists.** `raios-core/src/scoped_rollback_apply.rs:563`
(`evaluate_scoped_rollback_authorized_append`, schema
`raios.scoped_rollback_authorized_append.v0`) is a REAL evaluator with a real
proof. The narrow positive append therefore renders a granted decision whose
requested capability and effect names are READ FROM THAT EVALUATOR — never
invented, never derived from emitter booleans (that is the fail-closed violation),
and never downgraded to `observed` (that would HIDE a real durable effect, which
is precisely the failure this vocabulary exists to prevent — same principle as the
P4-9 D1 carve-out). An append that actually happened must say so.

**M4 — classification.** A missing or unknown classification NEVER defaults to
public; it is an explicit rejected/unknown record. Whole-response classification
is the MAXIMUM of the selected records' classifications, never a flattening of
them. `secret` may not enter durable memory or provider output.

**M5 — locators are not authority.** Summaries, query candidates, trace hits and
source paths LOCATE records. Relevance, presence, or prose may never appear as
grant evidence. (ADR 0004's rule, restated because the vocabulary makes it easy
to violate by accident: an evidence record with `status: verified` must mean an
evaluator verified it, not that a search found it.)

**M6 — P4-4 boundary confirmed.** `emit_recent_events`, the event ring, event
records and event bindings are P4-4's (now landed on main). P4-5 may advertise or
locate that method; it may not regenerate or delete its content.

### M3 CORRECTED (orchestrator, after the P4-5b1 worker's STOP)

The worker STOPPED and was right to: my M3 assumed
`evaluate_scoped_rollback_authorized_append` produced a proof carrying a capability
and effects. It does not. It returns `{performed: bool, status, reason}`. Nothing in
the allowed write set could have read a capability from it, and inventing one would
have been the back door. Good stop.

But the conclusion survives, sharpened. Two facts settle it:

1. The evaluator IS a real, fail-closed gate. Its body verifies a chain — scope
   decision authorized, scope-decision hash, append-record hash, sector-plan hash,
   write-readback hash, inspection hash, padding zeroed, target span verified,
   inspection verified — and returns `performed: true` ONLY when the whole chain
   holds. It already decides authority; it simply never had to SAY so.
2. The capability is NOT missing. `HELLO_ROLLBACK_APPLY_CAPABILITY` already exists
   and the emitters already declare it as `requested_capability`
   (seed-kernel/src/hello_service/emitters.rs:248, :328, :387, :453).

So the correct move is neither of the two things I forbade. Draw the line precisely:

- **Inventing authority** = creating a new gate, or fabricating a proof for an
  action nothing gates. FORBIDDEN, always.
- **Typing authority that already exists** = giving a real, already-passing gate a
  machine-readable proof that names the capability its caller already declares.
  REQUIRED — because without it the vocabulary literally cannot express a true
  positive without lying, and the only alternatives are to hide a durable effect
  (`observed` + `effects: []`) or to synthesize a grant from booleans.

**M3-corrected:** the write set is widened to `raios-core/src/scoped_rollback_apply.rs`
for ONE change: the evaluator gains a proof output. The GATE LOGIC DOES NOT CHANGE —
not one condition, not one hash, not one reason string. The requested capability and
the effect names are passed IN (they belong to the caller, which already has them);
core never mints a capability, it CERTIFIES one: "this chain verifies, therefore the
capability you asked for is proven, with these effects" — or it denies with the
existing reason. Fail-closed is preserved because the proof remains unconstructible
without a passing chain.

## P4-5b1 notes (2026-07-13)

Capability: core callers can now project one typed memory snapshot into ordered
evidence-v1 read responses, fail-closed mutation denials, and a real granted
durable-append decision backed only by the existing rollback-append evaluator.

- M1: `EmbeddedProviderProjection` places the provider-owned `Value` unchanged;
  `provider_projection_value_is_embedded_unchanged` pins the behavior.
- M2: `evaluate_memory_mutation` owns the six prerequisite positions, statuses,
  blockers, and first failure; its result type has no positive variant.
  `mutation_order_and_first_failure_are_evaluator_owned` and
  `mutation_evaluator_cannot_authorize_even_when_all_are_present` guard this.
- M3: `ScopedRollbackAuthorizedAppendProof` is private-field proof emitted only
  on the existing evaluator's unchanged successful path. The caller supplies the
  requested capability and effects. `project_authorized_append` requires the
  proof and renders granted, never observed. The positive, negative, and
  per-chain-element proof tests guard this boundary.
- M4: record classifications remain per-record; response classification is the
  maximum. Unknown classification and secret durable records are rejected.
- M5: query candidates and trace hits use `located`, never `verified`; all read
  decisions are observational.
- M6: P4-4 event content is absent. The profile projection can advertise
  `memory.recent_events` and `audit.events` without projecting their content.
- Durable payload, frame, and readback digests are accepted as `[u8; 32]` and
  passed directly to `Value::Sha256`; no durable record serializer or hash was
  touched.

UNCERTAIN: the manifest does not assign a wire classification string for an
unknown source classification. This slice uses explicit `classification:
"unknown"`, `status:"rejected"`, and treats `Unknown` as the conservative maximum.
Kernel acquisition/emission integration remains outside P4-5b1 by packet scope.
