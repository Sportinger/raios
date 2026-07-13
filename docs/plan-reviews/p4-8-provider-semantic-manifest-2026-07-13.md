# P4-8a — provider semantic manifest

Read-only inventory for P4-8. No Rust or PowerShell emitter may be deleted until
this mapping is represented by host tests and the decisions in section 6 are
resolved. This packet changes no runtime behavior.

Notation:

- `R` = legacy `body.result`; the export denial uses fields directly under `body`.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

Boundary ruling carried from P4-5: P4-5 owns selection and placement of
`memory.context provider_minimal`'s `provider_projection`; P4-8 owns that
projection's internal trust, redaction, classification, packet construction,
and hashes. P4-8 does not move the projection to another memory-context path.

## 1. Response-path inventory

### Dispatch and transport

`seed-kernel/src/agent_protocol_provider.rs` owns these in-scope response groups:

| Group | Methods / embedded response | Current envelope | Current classification |
|---|---|---|---|
| Trust posture | `provider.trust_honesty` | shared legacy `v/t/id/body.result` | implicit local response; posture fields have no per-field labels |
| Public projection | `emit_provider_minimal_projection()` embedded by `memory.context provider_minimal` | nested legacy object | `classification_default:local_only`; included/omitted field specs carry labels |
| Export gate | `provider.context_gate` | shared legacy `v/t/id/body.result` | `local_only` |
| Export-gate selftest | `provider.context_gate_selftest` | shared legacy `v/t/id/body.result` | `local_only` |
| Injection gate | `provider.context_injection_gate` | shared legacy `v/t/id/body.result` | `local_only` |
| Injection selftest | `provider.context_injection_gate_selftest` | shared legacy `v/t/id/body.result` | `local_only` |
| Public-packet selftest | `provider.context_export_packet_selftest` | shared legacy `v/t/id/body.result` | test response local; filtered packet records public |
| Authorized-export selftests | `provider.context_export_authorized_selftest`, `_smuggle` | shared legacy `v/t/id/body.result` | test response local; durable audit local-only |
| Real export denial | `provider.context_export` | hand-written legacy `t:error`; direct `body` fields | mixed: response/local gate state, two embedded records labeled public, durable audit local-only |

The literal `RAIOS_AGENT_BEGIN` count is one because all normal responses use
`begin_response()`; only `provider.context_export` writes the marker literal.
BEGIN/END spelling and CRLF framing remain transport, not vocabulary.

### Current ordered shapes

`provider.trust_honesty` emits, in order:

```text
schema, decision_schema, decision_id, decision_marker,
provider_id, descriptor_id, host, descriptor_sha256,
performed, status, reason, honest, chain_validated, time_validated,
authorizes_provider_request, authorizes_provider_export,
trust_state, chain_policy, time_policy, development_bypass,
claims_chain_validated, claims_time_validated,
owner_sealed, trust_tier, durable_write, capability_granted,
provider_write, transmission
```

The provider-minimal projection emits:

```text
schema, mode, profile, provider_export, redaction_projection,
classification_default, unclassified_field_policy,
packet_evidence {canonicalization, projected_packet_hash,
 exported_field_list_hash, omitted_field_list_hash,
 redaction_policy_hash, field_classification_hash, token_budget_hash},
local_projection_event_id, audit_event_id,
provider_trust_state, provider_trust_positive, can_export, blocked_by,
included_fields[], omitted_fields[], packet
```

`packet` contains schema/purpose/profile/scope/budget/authority order, stable
included IDs, coarse OS/status/provider posture, service/capability/problem
labels, public locator records, and explicit omissions. The provider posture is
selected provider, route, API-key **state only**, phase, endpoint, model, trust
state, pin kind/short IDs/slot/rotation posture, development-bypass flag, and
typed verifier outcome. No API key, prompt, Wi-Fi secret, raw transport detail,
or request correlation value is present.

The two live gate responses have the common ordered skeleton:

```text
schema, scope, classification,
provider_export, automatic_context_injection,
context_attached_to_provider_body, provider_write,
profile, profile_supported, gate_state, candidate, evidence,
blocked_by, [required], final denied booleans
```

The injection gate adds final-authorization state/event IDs and a final body
hash-check state. Both selftests emit the normal safety posture, profile,
case_count/passed, ordered covered rejections and cases, and final false
authority fields. Export packet/authorization selftests emit filtered packet
metadata plus, on the narrow positive test path, a local-only durable export
audit. The real export path remains denied and emits request, gate state,
binding denial/consumption, denial audit, durable denial audit, blocked gates,
requirements, and hash/event evidence.

## 2. Semantic mapping with field classification

Classification means eligibility for the provider-minimal payload, not serial
visibility. `public / gate=include` may enter the public packet. `local_only /
gate=omit` may exist in a local response or local audit but must not enter the
packet. `secret / gate=omit` must never enter the response projection, audit
payload, hash input exposed to the provider, or provider body.

### Legacy envelope

| Legacy field | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| `v`, `t`, `id:"serial"`, `body.method` | shared v1 envelope / `source_method` | local_only transport | omit |
| legacy family `schema` | `constant("raios.evidence_response.v1")` | public schema label | include only when the response itself is an exported packet |
| response/event IDs | typed response `id`, envelope `event_id`, or evidence `source_event_id` | local_only | omit |
| `scope` | envelope `scope` | public posture | include |
| whole local gate/selftest response | envelope `classification:"local_only"` | local_only | omit |

### Trust honesty

| Legacy field(s) | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| `provider_id` | `F.provider_id` | public | include |
| `descriptor_id` | `E[provider_trust_descriptor].facts.descriptor_id` | public | include |
| `host` | `E[provider_trust_descriptor].facts.host` | public | include; canonical provider host only |
| `descriptor_sha256` | `E[provider_trust_descriptor].facts.descriptor_sha256` | local_only | omit; gate evidence only |
| `decision_schema/id/marker` | `E[provider_trust_honesty].facts.*` or retired schema/marker redundancy | local_only | omit |
| `performed` | `E[provider_trust_honesty].status` | public posture | include as common status, not authority |
| `status`, `reason`, `honest` | `E[provider_trust_honesty].status/reason/facts.honest` | public posture | include |
| `chain_validated`, `time_validated` | `E[provider_trust_honesty].facts.*` | public posture | include; false must remain explicit |
| `trust_state`, `chain_policy`, `time_policy` | `E[provider_trust_descriptor].facts.*` | public posture | include |
| `development_bypass` | `E[provider_trust_descriptor].facts.development_bypass` | public posture | include; true blocks export |
| `claims_chain_validated`, `claims_time_validated` | `E[provider_trust_honesty].facts.*` | public posture | include |
| `authorizes_provider_request/export`, `capability_granted` | `retired(single D outcome/grants/effects)` | local_only decision | never infer grant from honesty |
| `owner_sealed`, `trust_tier` | `F.trust_custody.*` | local_only | omit |
| `durable_write`, `provider_write`, `transmission` | `D.effects` membership / absence | local_only decision | omit from public posture |

Trust honesty is observational. Its evaluator explicitly authorizes neither a
request nor an export, so `D=Observed("provider_trust_honesty_evaluated")`; it
must not create a `GrantProof`.

### Provider-minimal public projection: included fields

These rows are the existing `PROVIDER_MINIMAL_INCLUDED_FIELDS` policy and are
not broadened by P4-8.

| Legacy path/pattern | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| `schema`, `purpose`, `profile`, `scope` | public packet root | public | include |
| `budget.target_tokens`, `budget.estimated_tokens` | `F.packet.budget.*` | public | include; bounded |
| `authority_order[]`, `included.*[]` | `F.packet.*` | public | include stable labels/IDs only |
| `current.os.*`, `current.status.*` | `F.packet.current.*` | public | include coarse states only |
| `current.provider.selected` | `F.provider.selected` | public | include |
| `current.provider.route` | `F.provider.route` | public | include canonical route only |
| `current.provider.api_key_state` | `E[key_presence].facts.state` | public | include state marker; key bytes are secret and absent |
| `current.provider.direct_phase` | `F.provider.direct_phase` | public | include coarse phase |
| `current.provider.direct_endpoint` | `F.provider.direct_endpoint` | public | include canonical endpoint |
| `current.provider.direct_model` | `F.provider.direct_model` | public | include model ID |
| `current.provider.trust_state` | `E[provider_trust].facts.trust_state` | public | include; required by gate |
| `pin_kind`, `pin_id`, `pin_slot` | `E[provider_trust].facts.*` | public | include type/short ID/slot only, never pin bytes |
| `pin_rotation_policy`, `pin_rotation_id` | `E[provider_trust].facts.*` | public | include posture/short standby ID only |
| `development_bypass` | `E[provider_trust].facts.development_bypass` | public | include; true rejects export |
| `verifier_decision.{schema,verifier_id,stage,outcome,reason}` | `E[provider_trust_verifier].facts.*` | public | include typed outcome; no certificate bytes |
| `current.services[]` | `F.packet.current.services[]` | public | include stable IDs only |
| `current.capabilities[]` | `F.packet.current.capabilities[]` | public | include IDs/denied posture only |
| `current.problems[].{id,severity,summary}` | `F.packet.current.problems[]` | public | include stable scrubbed summary only |
| `records[].id/kind/authority/classification` | `E[record.<id>].facts.*` | public when source record is public | include only after per-record public check |
| `records[].summary` | `E[record.<id>].facts.summary` | public when source record is public | include scrubbed summary only |

The API-key evidence unit describes only `set|missing`. The key itself is
`secret / gate=omit`, never a `facts` value.

### Provider-minimal omissions

| Legacy path/pattern | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| `source_schemas` | `E[omission.source_schemas]` | local_only | omit |
| `system.snapshot.raw` | `E[omission.system_snapshot_raw]` | local_only | omit |
| `details.*.detail` | `E[omission.detail_strings]` | local_only | omit |
| `network.ip/gateway/dns` | distinct omission evidence | local_only | omit |
| `provider.direct_last_prompt` | `E[omission.raw_prompt]` | secret | omit; never hash/export raw value |
| `provider.direct_last_error/last_event` | distinct omission evidence | local_only | omit |
| `provider.direct_pending_id/direct_last_request_id` | distinct omission evidence | local_only | omit |
| `provider.tcp.*` | `E[omission.provider_tcp]` | local_only | omit |
| `wifi.ssid`, `wifi.passphrase` | distinct omission evidence | secret | omit; never hash/export raw value |
| `system.boot_log.raw`, `boot_log.summary.current` | distinct omission evidence | local_only | omit |
| recovery-lifeline status/locator | distinct omission evidence | local_only | omit; authority-bearing local evidence |
| `records[].source` | `E[omission.record_source]` | local_only | omit source paths/method locators |
| any unclassified field | `E[omission.unclassified]` | local_only by default | omit, fail closed |

Each omission remains explicit evidence with its current reason. No omitted
value is copied into omission facts.

### Projection evidence and export gates

| Legacy field(s) | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| packet canonicalization and six packet/policy hashes | `E[provider_projection_binding].facts.*` | local_only | omit from payload; require exact hash match at gate |
| `local_projection_event_id`, duplicate `audit_event_id` | envelope/source event; duplicate retired | local_only | omit |
| `provider_trust_state/positive` | `E[provider_trust]` status/facts | public posture | state may be included; gate result remains local |
| `profile`, `profile_supported` | `F.profile` + `E[profile].status` | public profile ID / local validation | include ID; omit validation mechanics |
| binding candidate IDs/hashes/status/reason/retained/consumed | ordered request/export/consumption evidence | local_only | omit; require verified current-boot evidence |
| redaction/classification/budget/trust bindings | ordered evidence units | local_only | omit; require verified current-boot evidence |
| final authorization/body-check state and event IDs | final-authorization evidence | local_only | omit; require verified authorization before attachment |
| `blocked_by[]` | `D.blocked_by[]` in evaluator order | local_only | omit |
| `can_export`, `can_attach_context`, `satisfies_*` | `D.outcome`, grants/effects | local_only decision | retire redundant booleans |
| `provider_export`, `automatic_context_injection`, `provider_write`, body attachment | `D.effects` membership/absence | local_only decision | no effect on denied path |
| `required[]` | ordered missing evidence IDs / `D.blocked_by` | local_only | omit |

Both current live gates are denied and must render `grants:[]`, `effects:[]`.
The export evaluator's positive selftest may use a granted decision only from
`evaluate_provider_export_gate()`'s positive result; the real export path still
performs no transmission.

### Selftests, export denial, and audits

| Legacy field(s) | v1 mapping | CLASSIFICATION | Export-gate treatment |
|---|---|---|---|
| selftest `test_infrastructure`, case_count, passed | `F.*` | local_only | omit |
| selftest safety booleans | `F.safety` counts or retired decision redundancy | local_only | omit |
| each case expected/actual status/reason/passed | `F.cases[]` preserving order | local_only | omit |
| filtered packet `packet_all_records_public/count/hash` | packet facts + projection evidence | local_only gate metadata | omit from attached payload |
| `included_public_ids`, `excluded_local_only_count` | ordered selection/omission evidence | local_only | omit |
| export denial `code/message/request` | `D=Denied`, `F.request` | local_only | omit |
| request-binding denial and binding consumption | ordered evidence | local_only pending Decision 3 | omit |
| export denial audit | ordered evidence | local_only pending Decision 3 | omit |
| durable denial/export audit record metadata and hashes | reusable P4-5 durable-record evidence projection | local_only | omit; never enter public packet |
| durable audit `performed` | decision/evidence from the P4-5 append evaluator | local_only | omit |
| `export_performed`, `transmission_performed`, `provider_write` | `D.effects` absence | local_only | omit |

Stored `raios.memory_record.v0` bytes and RECLOG frame hashes are P4-5-owned and
must remain byte-identical. P4-8 reuses their typed evidence projection; it does
not reserialize durable records.

## 3. Evidence units, ordering, and ownership

Ordered provider evidence should be:

1. `provider_profile`;
2. `provider_trust_descriptor` and `provider_trust_honesty`;
3. `key_presence` (state only);
4. `provider_projection_binding` with packet/redaction/classification/budget hashes;
5. request binding, export-audit binding, and binding consumption in evaluator order;
6. final injection authorization and final pre-write body check when applicable;
7. local denial/export audit evidence;
8. reusable durable append evidence, if a real audit append was attempted.

Selftest case order remains the evaluator's order: 20 export-binding cases and
8 final-injection cases. The renderer may not sort or reconstruct blockers.

Kernel-owned work remains dispatch/framing, live system/provider snapshots,
secret custody, event-log locks and recording, binding consumption, durable
I/O/readback, and the actual provider body/write boundary. Core-owned work is
the existing projection classification tables and canonical hashes, trust
descriptor honesty, scoped export evaluation, typed v1 field/evidence tables,
and pure mapping tests. No generic policy evaluator, second serializer, or new
schema layer is required.

Hash invariants:

- `provider_projection_packet_hash()` and its canonical input order remain
  byte-identical;
- exported/omitted field-list, redaction-policy, field-classification, and
  token-budget hashes retain their existing domains and field order;
- `provider_export_packet_hash()` remains over the same `record::Value` packet;
- request envelope/body/binding, trust-evidence, durable payload/frame/readback,
  and audit-binding hashes do not change because response vocabulary moved;
- packet selection and packet hashing must consume one immutable projection,
  not two reads of mutable state.

## 4. Predicate inventory

Only direct P4-8 responses and consumers of P4-8 provider fields are counted.
Provider-named predicates over P4-4 event responses, P4-5 placement-only memory
fields, system capability/service inventories, and direct TLS transport text are
reviewed for donor effects but not assigned to P4-8.

Focused runtime inventory:

| Runtime profile | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| common provider responses | 10 | 52 | 3 | 65 |
| `provider-memory` injection tail | 6 | 20 | 2 | 28 |
| **`provider-memory` total** | **16** | **72** | **5** | **93** |

The measured **93 reviewed / 72 regenerated** is inside the P4 design estimate
of **70-140 / 40-90**.

Additional execution/static consumers:

| Surface | Reviewed shape | Treatment |
|---|---:|---|
| `provider-memory-full` and `full` | 119 runtime predicates (common 65 + active full-provider tail 54) | same 78 regenerated planning count; dormant injection-selftest function is not counted as executed |
| `memory-durable` | 25 cross-family assertions + 10 provider command completions | regenerate response paths; P4-5 retains durable append/placement ownership |
| `quick` | common 65; provider-named quick assertions are P4-4 event output | only common is P4-8 |
| `openai-direct-smoke.ps1` | direct TLS/request markers plus positive gate checks | integration sweep; preserve raw-secret leak checks and regenerate only v1 agent-response needles |

Leaf assertions survive only where the same evaluator-sourced case name,
status/reason, posture label, or hash key/value remains in one scoped evidence
object. Schema needles, parsed `body.result` paths, authority booleans, bare
event IDs, bare classification/reason/hash needles, and whole-log donor matches
must regenerate.

Serialization rules for every regenerated needle:

- top-level envelope fields arrive as CR CR LF; multi-line PowerShell needles
  spell separators as `` `r`r`n ``;
- no needle spans the top-level response `id` field;
- `facts`, `evidence`, and `decision` are single-line `InlineObject`s;
- single-line needles stay inside one such object;
- classification/status/reason/hash assertions bind to the same evidence ID;
- public inclusion assertions bind record classification and exportability in
  the same object; a bare `"classification": "public"` is forbidden.

Required scan evidence (verbatim):

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/agent_protocol_provider.rs
1
```

```text
> rg -l "provider\." vm-harness/
vm-harness/shadow-vm-smoke-profile-common.ps1
vm-harness/shadow-vm-smoke-profile-full-provider-memory.ps1
vm-harness/shadow-vm-smoke-profile-memory-durable.ps1
vm-harness/shadow-vm-smoke-profile-provider-memory.ps1
vm-harness/openai-direct-smoke.ps1
vm-harness/shadow-vm-smoke-profile-quick.ps1
```

Needle-count scan (expanded execution inventory, not merely literal line count):

```text
> <provider-family needle-count scan>
common assertions=62 framing=3 total=65
full-provider-memory assertions=80 framing=3 total=83
provider-memory assertions=26 framing=2 total=28
memory-durable assertions=25 provider-command-completions=10 total=35
common groups:
trust_honesty=6
context_export=46
context_gate=10
full groups:
gate_selftest=35
injection_gate=17
injection_selftest=28
```

The literal full-provider file total includes the 28-assertion injection
selftest function that the `full` and `provider-memory-full` dispatch do not
invoke. Runtime totals exclude it. The active full tail is 35 gate-selftest
assertions + 17 injection-gate assertions + two completions = 54.

## 5. Host/guest selftest strategy

Existing host coverage already proves projection field classifications and
canonical hashes (`provider_context.rs`), trust-descriptor honesty
(`provider_trust_descriptor.rs`), and scoped export first-failure order
(`scoped_provider_export.rs`). Existing guest coverage proves current-boot
binding substitution, stale IDs, every projection-policy hash mismatch,
trust bypass/downgrade, final body-hash mismatch, public-record filtering,
local-only smuggle denial, durable audit append/dedupe, and real export closure.

P4-8b needs the smallest semantic host set:

1. exhaustive old-path-to-v1 mapping rows for every response group;
2. one field-classification test iterating every included and omitted spec;
3. secret fixtures proving API-key bytes, prompt, SSID, passphrase, raw errors,
   request IDs, and transport detail never enter packet/value/hash inputs;
4. public/local-only mixed-record fixtures proving fail-closed filtering;
5. trust present/missing/bypass fixtures with unchanged first failure;
6. packet and all six policy hash goldens byte-identical before/after;
7. binding/final-authorization order and denied empty grants/effects;
8. positive scoped-export selftest requires evaluator proof while transmission
   remains absent;
9. P4-5 provider-projection placement consumes the provider-owned typed value
   without copying its field table or serializer;
10. one v1 render/framing integration per live gate plus both negative selftests.

Guest regeneration remains on `provider-memory`; `provider-memory-full` is the
broader gate-selftest ordering check. This packet runs neither Cargo nor VM.

## 6. Risks and P4-8b STOP-tripwires

1. **Public packet record vocabulary — OWNER/ORCHESTRATOR DECISION NEEDED.**
   `PROVIDER_MINIMAL_INCLUDED_FIELDS` permits `records[].summary`, but
   `provider_export_packet_value()` emits `entity` and `predicate` instead and
   the public fixture relies on them. Decide whether those two fields become
   explicit public-only specs or are removed/replaced by the approved scrubbed
   summary. Treating them as public without that ruling is policy invention and
   a STOP.
2. **Mixed response classification — OWNER/ORCHESTRATOR DECISION NEEDED.** The
   P4-5 memory response is local-only while its nested provider projection is
   public/redacted. Decide the v1 representation for that mixed packet (nested
   evidence classifications versus a local-only envelope plus typed public
   projection). Labeling the whole response public is a STOP.
3. **Public-labeled denial records — OWNER/ORCHESTRATOR DECISION NEEDED.** The
   legacy request-binding denial and export-denial audit label themselves
   `public` while carrying current-boot IDs and event correlations. Existing
   provider projection policy does not authorize those fields for export.
   Decide whether the records become local-only or receive a separately
   redacted public projection. Copying them into public facts is a STOP.
4. **P4-5 placement order — OWNER/ORCHESTRATOR DECISION NEEDED.** P4-5 owns
   `provider_projection` placement; P4-8 owns internals. Decide whether P4-5
   temporarily embeds the existing typed value or P4-8 lands first. Duplicate
   field tables, hash logic, serializers, or competing placement are a STOP.
5. **Positive selftest decision vocabulary — OWNER/ORCHESTRATOR DECISION
   NEEDED.** Name the requested capability/effect for the existing narrow
   `evaluate_provider_export_gate()` positive selftest. It authorizes scoped
   evaluation/audit but performs no transmission. Deriving a grant from
   `authorized:true` or claiming provider transmission is a STOP.
6. **Trust descriptor export boundary — OWNER/ORCHESTRATOR DECISION NEEDED.**
   Existing public policy names trust/verifier posture but not descriptor hash,
   custody tier, or event IDs. This manifest keeps those local-only. Any request
   to export them requires an explicit policy ruling, not an emitter choice.
7. **Export packet hash stability.** Response-vocabulary movement must not alter
   `provider_export_packet_value()`, canonical field order, filtered record
   order, or any packet/policy/binding hash. Regenerating those goldens because
   the response envelope changed is a STOP.
8. **Redaction wiring preservation.** The included/omitted tables,
   `unclassified -> omit`, secret/local-only omission, and same-source hash
   bindings must remain wired into the final gate. A v1 field table that merely
   describes redaction without feeding the gate is a STOP.
9. **Secret custody.** Raw API keys, prompt text, Wi-Fi credentials,
   Authorization values, Content-Length, request IDs, free-form errors/events,
   and TCP diagnostics must never enter public facts, packet hashes exposed to
   the provider, audit prose, or fallback output. No default-to-public path.
10. **Key-presence classification.** `api_key_state:set|missing` is public; key
    bytes are secret and absent. A generic `Value` accessor over the provider
    snapshot must not make the secret reachable from the table.
11. **Decision authority.** Trust honesty and posture observations grant
    nothing. Denied gates have empty grants/effects. Only the scoped export
    evaluator may produce the narrow positive proof; kernel emitters cannot.
12. **First-failure and snapshot coherence.** Gate blockers retain evaluator
    order and all packet/hash/binding facts come from one captured projection.
    Renderer reconstruction, sorting, or rereading mutable provider state is a
    STOP.
13. **P4-5 memory-context boundary.** P4-8 owns projection internals; P4-5 owns
    selection and placement. Moving `memory.context`, durable resolver ordering,
    or provider-projection placement into P4-8 is scope creep and a STOP.
14. **Durable-audit reuse.** Durable append bytes, ordering, readback, dedupe,
    and hashes stay P4-5-owned. Copying their projector into provider code or
    changing stored bytes is a STOP.
15. **Needle donor collapse.** No regenerated predicate may use a bare public
    classification, reason, event ID, or hash. It must bind response family,
    evidence ID, classification, and relevant value in the same source object.
16. **Write-set scope.** P4-8b will need an explicit implementation packet for
    provider projection/evaluator modules and affected harness files. This
    manifest authorizes only this documentation file.

Scope-creep observations:

- `event_log` owns binding records and their current-boot retention; P4-8 may
  project/evaluate them but must not absorb the event ring (P4-4).
- `memory_store` owns real durable audit append execution; P4-8 owns why the
  provider gate requested the audit, not the RECLOG implementation (P4-5).
- direct OpenAI TLS/request-envelope markers are transport/provider-trust
  integration evidence. P4-8 preserves their redaction and ordering checks but
  does not migrate the direct request protocol into the evidence response.
- capability/service/problem inventory emitters remain P4-9/system ownership;
  P4-8 consumes only their already-public stable projection.

## Orchestrator rulings (2026-07-13, binding for P4-8b)

**P1 — the export packet hash is the hard boundary.** `sha256_of_json` hashes an
INDEPENDENTLY BUILT canonical Value at indent 0; it never sees the serial pretty-print
(verified: agent_protocol_provider.rs:1080 -> raios-core/src/record.rs:111). So the
RESPONSE vocabulary may move freely, but the packet-hash INPUT GRAMMAR may not. Every
provider hash — projected_packet_hash, exported_field_list_hash, omitted_field_list_hash,
redaction_policy_hash, field_classification_hash, token_budget_hash,
provider_trust_evidence_hash — must be byte-identical before and after. If one moves, the
export contract broke and that is a STOP, not a golden regeneration.

**P2 — classification: presence is public, bytes are secret, and nothing drifts upward.**
API-key PRESENCE is public. Key BYTES are secret and unreachable — there must be no path,
not even a debug one, by which they reach a fact. Trust honesty is OBSERVATIONAL and grants
nothing. An unknown or missing classification is an explicit REJECTED record; it NEVER
defaults to public (the memory family's M4, applied here). The enclosing response stays
`local_only`; a public child requires an explicit export gate, not a side effect of the
vocabulary move.

**P3 — P4-5's memory.context embeds your typed value verbatim, and that is now load-bearing.**
P4-5b2 extracted `provider_minimal_projection_value` out of the raw serial emitter precisely
so memory could embed it without copying the mapping. Memory does NOT police its contents.
Today that value still carries `outcome`, `blocked_by` and `authorizes_provider_*` as
descriptive FACT keys — pre-v1 vocabulary that PANICKED THE KERNEL when the memory
projection recursed its reserved-key check into it (caught live at
shadow-20260713-174237-10020).

Converting those keys is P4-8's job and it is the ACTUAL fix: in v1, a fact does not get to
say `authorizes_provider_export`. That is a decision's word. Move them:
  outcome / blocked_by            -> the response's DECISION (denied, with the ordered
                                     blocked_by and the first-failure reason)
  authorizes_provider_export      -> NOT a fact. The absence of a grant IS the denial.
  authorizes_provider_request     -> same.
When P4-8 lands, memory's embedded value stops carrying reserved keys on its own — because
its facts stop claiming authority. Do not "rename around" the check; remove the claims.

**P4 — trust honesty grants nothing, and says so.** `scoped_provider_trust_honesty` already
proves this in core ("success_never_authorizes_request_or_export"). The v1 response must
render trust as OBSERVED evidence with no grants and no effects. A pinned certificate is a
fact about a certificate, not a permission.

**P5 — the export gate is a DENIAL with an ordered blocked_by.** It is not a boolean and not
a status string. First-failure reason from the evaluator, `grants: []`, `effects: []`.

**P6 — nothing secret becomes a locator either.** ADR 0004's rule (locators are not
authority) has a twin: a locator must not leak what it points at. A summary or trace hit
that names a secret record may not carry its value, and its classification is the record's,
not the summary's.
