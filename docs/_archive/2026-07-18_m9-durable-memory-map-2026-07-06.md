# M9 Durable Memory & Context Broker v1 — Design Map (2026-07-06)

**Header / execution preconditions.** Authored 2026-07-06 AHEAD of execution as
pre-planning. Do NOT execute any M9 slice until: (a) M6 Promotion Loop v0 is
CLOSED green (M6A-D per `docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`
plus a passing full profile), (b) M7 Persistence Foundation is CLOSED — M9 hard-depends
on M7B (SEED_DATA append-only durable record store with M3-style
append/readback/inspect discipline) and uses M7C's boot-generation counter from
`control.json` for cross-boot ordering, (c) the M7 map's final interface names
are known. M8 (recovery lifeline) is NOT a precondition, but one OWNER DECISION
below touches it. This map was written before M7 exists: every reference to
"the M7B store" is an assumed interface; the consistency pass and Slice 0 must
align it with what M7 actually shipped.

**MANDATORY Slice 0 = map revalidation.** Before any implementation slice:
re-check every file:line claim in this map against HEAD (files WILL have moved —
`agent_protocol_memory.rs` was 3,241 lines and `event_log_types.rs` 3,899 lines
at authoring time, both near split thresholds and likely restructured by
M6/M7/M8 work). Confirm the real M7B store API (module name, append/readback
function signatures, namespace/stream concept, two-boot harness support).
Update this map FIRST if reality diverged, commit the map update, then start
implementation slices.

## What M9 is

ADR 0004 Phase D made real: the first durable typed memory records persisted to
SEED_DATA with provenance and classification, the transition of memory mutation
methods from blanket denial to scoped authorization behind an M3/M6-style gate
chain, and a context broker that assembles budgeted `raios.agent_context.v0`
packets from durable + current_boot facts with explicit omission reporting and
end-to-end provider-export gating.

What exists today (verified at authoring): read-only memory surface in
`seed-kernel/src/agent_protocol_memory.rs` — `memory.profile` (:28),
`memory.context` (:48, emits `raios.agent_context.v0` from live snapshot only,
`scope: current_boot`, `provider_export: disabled`), `memory.query`,
`memory.trace` (static id index at :3066), `memory.recent_events`. All five
mutation methods are denied via `MEMORY_MUTATION_METHODS` (:20). Profiles
diagnostic/planning/provider_minimal exist; `recovery_minimal` does NOT.
`estimated_tokens` are hard-coded constants (:3178) — placeholders, not
measurements. Provider gating machinery exists as bindings/gates in
`event_log_types.rs` (ProviderExportAuditBinding :91,
ProviderContextInjectionAuthorization :125) and
`agent_protocol_provider.rs` (`provider_context_block_reason`,
`emit_provider_minimal_projection`). Record model with single JSON
serializer+hasher lives in `raios-core/src/record.rs` (Value :13, Field :29,
`sha256_of_json` :109).

## Design decisions (fixed by this map — no judgment left open)

1. **First durable record kinds — smallest authority-bearing set.** In order:
   (a) capability grant records plus a BOUNDED subset of denial records —
   durable denial records are written ONLY for authority-changing denials
   (module load, rollback apply, memory.* mutations, provider export);
   routine protocol denials (the hundreds the smoke profiles fire per run
   across ~180 methods) stay RAM-ring events only, or the per-boot quota
   exhausts minutes into any profile. Denials OF durable memory writes
   themselves (e.g. `memory_write_quota_exhausted`) are NEVER durably
   recorded — RAM event only, no self-recording recursion. (b)
   promotion/rollback transaction mirror records (locators binding to the M6
   durable transactions expected in the audit region — hash references, NOT
   copies; Slice 0 confirms what M6 actually persisted, since M6 was open at
   authoring), (c) decision records (owner approvals, ADR references), (d)
   problem records (open/resolve via supersede). Raw events stay in the RAM
   ring of 256; periodic durable event snapshots are explicitly OUT of M9
   (future direction only). Chat history is never a durable record kind.
2. **Schema = record-model entry only** (mechanism-before-vocabulary, ADR 0005
   §3). `raios.memory_record.v0` is implemented in `raios-core` on the existing
   Value/Field model with the single serializer+hasher; no hand-rolled emit/hash
   code in the kernel. Field set per ADR 0004: schema, id, kind, entity,
   predicate, value, classification, authority, boot_id, sequence, source,
   evidence[], tags[], supersedes[], created_at{clock:"boot_relative",ticks} —
   no wall-clock time until M10 trusted-time exists.
3. **Classification stamped at write time, fail-closed.** Every durable write
   carries `public`/`local_only`/`secret`. `secret` is NEVER durable — typed
   `capability_denied` reason `secret_never_durable_until_sealed_secret_design`.
   Secrets may only appear as state markers (`set`/`missing`) inside non-secret
   records. Unknown/unclassified → `local_only`.
4. **Supersede, never overwrite.** No durable record is mutated in place.
   Problem resolution, fact correction, and stale decisions are new records with
   `supersedes: [old_id]`. Deletion does not exist in M9 (`memory.redact` stays
   denied — a redaction transaction is future work).
5. **Gate chain for durable memory writes** (mirrors M3/M6): (1) record-model
   schema validation + hash via raios-core; (2) classification gate (secret →
   deny, unknown → local_only); (3) kind/authority gate (agent-proposed records
   may only be kind `observation` with authority `event`; policy/decision/grant
   kinds are system-authored only; the agent path may also only SUPERSEDE
   records that are themselves agent-authored `observation`/authority-`event`
   records — a `supersedes` reference to any system-authored or
   higher-authority record id → typed denial `supersede_authority_exceeded`,
   otherwise agent observations could suppress decisions/grants/denials from
   future context, a second weaker authority system beside the capability
   ledger); (4) quota gate — per-boot durable memory
   write budget, default 128 records / 32 KiB per boot, exhaustion → typed
   denial `memory_write_quota_exhausted` (defaults revisable later, fail-closed
   now); (5) append to the M7B store; (6) readback + hash compare; (7) inspect
   evidence event in the event log. Acknowledge ONLY after (6)+(7). Any gate
   failure → typed denial naming the missing evidence, nothing written.
6. **Context broker v1.** `memory.context` assembles from durable records +
   current_boot facts under ADR 0004 budgets. Profiles: `recovery_minimal`
   (new, 512-1500 target), `provider_minimal`, `diagnostic`, `planning`.
   Authority order gains `core_ledger`/`evidence` tiers now that durable
   records exist. Every packet reports an explicit `omitted` array with
   reasons. `estimated_tokens` becomes a measured estimate (emitted bytes / 4
   heuristic — honest: it is a heuristic, label it `estimate_method:
   "bytes_div_4"`), replacing the static constants.
7. **Summaries/RAG are locators ONLY. No vector store in the kernel, ever.**
   The kernel does no embedding, no BM25, no semantic ranking. Retrieval in M9
   = structured selectors (id/kind/entity/classification), recency window, and
   severity boost. Semantic indexing is AT MOST a future host-side tool that
   reads exported public/local_only records over the serial protocol and hands
   back candidate record IDS which the broker must trace to typed records
   before inclusion. That host tool is M12+ direction, not M9 scope. ADRs/docs
   are referenced by stable path + content hash as locator records, never
   imported as record bodies.
8. **Provider export gating end-to-end.** Nothing derived from memory leaves
   the machine unless ALL pass: positive provider trust state, known profile,
   classification filter (public only), redaction applied, token budget
   applied, and a durable export-audit memory record (append+readback) written
   BEFORE the request is transmitted. Denied exports also produce a durable
   denial-audit record. If the audit append fails, the export fails — the
   audit is not best-effort.
9. **No fake long-term memory.** Facts assembled from RAM stay labeled
   `current_boot`; facts from the durable store are labeled `durable` only when
   backed by readback+hash evidence, and the broker packet distinguishes the
   two per included record.

## OWNER DECISIONS (ask before the affected slice)

- **OD-1 (before M9B-1): may agent-proposed observations land durable in v1?**
  Covers BOTH `memory.record_observation` and `memory.supersede_fact` (the
  supersede-authority rule in decision 5(3) applies in every option).
  Options: (a) yes, restricted — kind `observation`, authority `event`,
  classification ≤ local_only, quota-bounded [RECOMMENDED — it is the smallest
  real capability and every gate is fail-closed]; (b) no — agent proposals stay
  current_boot-retained only, durable writes remain system-authored in v1;
  under (b), slice M9B-1 is replaced by "proposals retained current_boot-only,
  both methods stay denied" and the M9 close statement drops the agent-write
  clause; (c) yes plus durable `decision` proposals — REJECT,
  policy-by-provider risk.
- **OD-2 (before M9C-1): is `recovery_minimal` exposed over the M8 recovery
  lifeline?** Options: (a) no — profile exists on the rich path only, lifeline
  keeps its pinned protocol untouched [RECOMMENDED for M9; lifeline changes
  belong to M8's owner]; (b) yes, read-only — requires touching the pinned
  lifeline protocol, which is a trust-surface change and likely a STOP anyway.

## Sub-milestones and slices

M9A durable typed records (system-authored) → M9B scoped agent writes →
M9C context broker + provider export → M9D cross-boot proof + close.

Verification uses a NEW focused profile `memory-durable`
(`vm-harness/shadow-vm-smoke-profile-memory-durable.ps1`), created in M9A-2 and
extended each slice. Adding it requires editing the `ValidateSet` in
`vm-harness/shadow-vm-smoke.ps1` (line 12 at authoring) and a dispatch branch
(~line 167). Golden needles are ground truth; worker-reported diffs are not.

Global STOP-tripwires (all slices): anything requiring a new ADR (sealed
secrets, kernel-side semantic indexing, unparking ota/registry/fake-cloud);
any trust-model or lifeline-protocol change; any disk write outside the M7B
append discipline or the designated SEED_DATA region; overwriting
`release/raios-stage0.img`; M7B interface mismatch discovered at revalidation;
full profile red (Red Gate Rule); a secret value observed in any durable
record during testing; any packet needing to touch attested descriptor sources
in a way `target/descriptor-resign` cannot cleanly re-sign.

---

### Slice 0 — Map revalidation (MANDATORY, first)

Capability: none (planning integrity). Verify every file:line above, the M7B
store API, harness two-boot support, and profile list; update + commit this map
before any code. STOP if M7B has no two-boot verification pattern (M9D depends
on it; escalate to owner with the M7 map open).

```text
Packet id: M9-0-map-revalidation
Goal: Re-verify every file:line and interface claim in docs/plan-reviews/m9-durable-memory-map-2026-07-06.md against HEAD; update the map where reality diverged; commit the map update only.
Read first: docs/plan-reviews/m9-durable-memory-map-2026-07-06.md; docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md; the M7 map in docs/plan-reviews/; seed-kernel/src/agent_protocol_memory.rs; seed-kernel/src/agent_protocol_provider.rs; raios-core/src/record.rs; vm-harness/shadow-vm-smoke.ps1; docs/PROJECT_STATUS.md
Allowed write set: docs/plan-reviews/m9-durable-memory-map-2026-07-06.md only
Forbidden: any source, harness, or release file change; renumbering milestones or slices
Constraints: for each claim record CONFIRMED or DIVERGED with the new file:line; name the real M7B append/readback API; check whether shadow-vm-smoke.ps1 supports a two-boot (write, reboot, readback) sequence; ADDITIONALLY enumerate and write into this map, as exact file:line lists, the call sites later packets reference: (a) the authority-changing grant/denial decision sites M9A-2 hooks (module load, rollback apply, memory mutations, provider export — per design decision 1), (b) the M6 promotion/rollback transaction commit sites and what M6 actually persisted to RAIOS_AUDITRB_V0, (c) the problem open/resolve emission sites, (d) the single provider request binding call site in the openai path; flag any of these that live in the attested source set (list in seed-kernel/build.rs)
Definition of done: map updated with all four call-site lists and no remaining "(from Slice 0)" placeholders in any packet; a DIVERGED list (possibly empty) exists in the report
Report format: table of claim -> CONFIRMED/DIVERGED(new location); the four call-site lists; M7B API summary; two-boot support yes/no; attested-source flags; STOP flags if any
```

### M9A-1 — `raios.memory_record.v0` in the record model (host-side)

Capability: raiOS has one canonical, host-tested serialization + hash for
durable memory records — every later write path reuses it instead of inventing
emit code. Touch: `raios-core/src/` new `memory_record.rs` (+ `lib.rs` export);
host tests. No kernel change. Verify at execution time whether M7/M8 already
added adjacent record-model entries to align with.

Verification: `cargo test --locked -p raios-core` (all green, still fast);
`cargo fmt --all -- --check`. No VM profile needed for a host-only slice.
Fail-closed: constructor rejects `secret` classification for durable intent;
rejects unknown kind strings; rejects empty entity/source for observation kind.

```text
Packet id: M9A-1-memory-record-schema
Goal: Add raios.memory_record.v0 as a typed record-model entry in raios-core using the existing Value/Field model and single serializer+hasher, with host tests.
Read first: raios-core/src/record.rs; raios-core/src/scoped_rollback_apply.rs (existing evaluator style); docs/architecture-decisions/0004-system-memory-and-agent-context.md (Memory Store Shape section); this map's design decisions 2-4
Allowed write set: raios-core/src/memory_record.rs (new); raios-core/src/lib.rs (export line only); raios-core host tests
Forbidden: any seed-kernel change; any hand-rolled JSON emit or hash code (must go through record.rs write_json/sha256_of_json); wall-clock timestamps; any new dependency
Constraints: no_std compatible like the rest of raios-core; fields exactly: schema,id,kind,entity,predicate,value,classification,authority,boot_id,sequence,source,evidence,tags,supersedes,created_at{clock:"boot_relative",ticks}; kinds limited to capability_grant,capability_denial,promotion_tx_ref,rollback_tx_ref,decision,problem,observation,export_audit; classification limited to public,local_only; a secret classification or unknown kind must return a typed constructor error, not panic
Definition of done: cargo test --locked -p raios-core green with new tests covering serialize+hash stability, supersedes link, secret rejection, unknown-kind rejection; cargo fmt --all -- --check clean; commit with capability sentence
Report format: test names + pass counts; the sha256 of one fixed sample record (so later slices can pin it as a needle)
```

### M9A-2 — First system-authored durable records + `memory-durable` profile

Capability: the kernel durably records capability grants/denials and
promotion/rollback transaction references to SEED_DATA with append+readback+hash
evidence — memory that survives reboot exists for the first time. Touch: NEW
`seed-kernel/src/memory_store.rs` (do NOT grow `agent_protocol_memory.rs`,
already near split threshold); call sites where grants/denials and M6
promotion/rollback transactions are decided (locations from Slice 0); M7B store
API; new harness profile file + `shadow-vm-smoke.ps1` ValidateSet/dispatch.
Verify at execution time: exact M7B namespace mechanism.

Verification: NEW focused profile —
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile memory-durable`
New needle families: `memory-durable-append` (append accepted, readback hash
match, inspect event; ≥12 needles), `memory-durable-secret-denied` (secret
write attempt → typed denial, nothing appended; ≥4), `memory-durable-quota`
(quota exhaustion → typed denial; ≥3). Plus quick profile stays green.
Fail-closed: all five `memory.*` mutation methods STILL denied (this slice is
kernel-authored writes only); secret and over-quota writes denied; store
append failure → grant/denial still functions but records
`memory_durable_write_failed` problem (memory failure must not brick the
capability system — record the degradation honestly).

```text
Packet id: M9A-2-system-authored-durable-records
Goal: Kernel writes capability grant/denial records and promotion/rollback transaction reference records as raios.memory_record.v0 to the M7B SEED_DATA store with append+readback+hash+inspect evidence, and a new focused VM profile proves it.
Read first: this map decisions 1,3,5; raios-core/src/memory_record.rs; the M7B store module (name from Slice 0); seed-kernel/src/agent_protocol_memory.rs:20-46; vm-harness/shadow-vm-smoke-profile-quick.ps1 (profile file shape); vm-harness/shadow-vm-smoke.ps1:12
Allowed write set: seed-kernel/src/memory_store.rs (new); minimal hook lines at the grant/denial and M6 transaction call sites named by Slice 0; vm-harness/shadow-vm-smoke-profile-memory-durable.ps1 (new); vm-harness/shadow-vm-smoke.ps1 (ValidateSet + dispatch branch only); docs/PROJECT_STATUS.md
Forbidden: enabling any memory.* mutation method; touching attested descriptor sources (if unavoidable, STOP and report; re-sign via target/descriptor-resign is required and must be stated in the commit); writing outside the M7B append API; copying transaction bodies (references+hashes only); growing agent_protocol_memory.rs
Constraints: gate chain order exactly as map decision 5; acknowledge only after readback hash match + inspect event; quota default 128 records/32 KiB per boot; failure to append must degrade to a typed problem record in RAM, never a panic and never silent
Definition of done: memory-durable profile green with the three new needle families; quick profile green; report filenames named in commit; capability sentence in commit
Report format: profile report filename + needle counts per family; the durable record ids written during the smoke; any STOP flags
```

### M9A-3 — Decision + problem records with supersede-not-overwrite

Capability: owner decisions and problem open/resolve become durable history —
after reboot the system can still say which problems were resolved, by which
superseding record, on which evidence. Touch: `memory_store.rs`; problem
lifecycle call sites; extend `memory-durable` profile. Fail-closed: resolving a
problem without naming the superseded record id → typed denial; superseding a
nonexistent id → typed denial; no in-place mutation anywhere.

Verification: `-Profile memory-durable` extended with family
`memory-durable-supersede` (open → resolve chain readable with both records
present, old record intact; ≥8 needles).

```text
Packet id: M9A-3-decision-problem-supersede
Goal: Durable decision records and problem open/resolve via supersedes-links, never overwriting, proven by extended memory-durable needles.
Read first: this map decision 4; seed-kernel/src/memory_store.rs; problem list emission sites (from Slice 0, agent_protocol_system.rs area); vm-harness/shadow-vm-smoke-profile-memory-durable.ps1
Allowed write set: seed-kernel/src/memory_store.rs; problem lifecycle call sites named by Slice 0; vm-harness/shadow-vm-smoke-profile-memory-durable.ps1; docs/PROJECT_STATUS.md
Forbidden: mutation or deletion of an existing durable record; enabling memory.* mutation methods; new schemas outside the record model
Constraints: a resolve record must carry supersedes=[open_record_id] and evidence linking to what proved resolution; both records must remain readable afterwards
Definition of done: memory-durable profile green including memory-durable-supersede family; quick green; commit names report file + capability sentence
Report format: report filename; the supersede chain ids from the smoke run
```

### M9B-1 — `memory.record_observation` + `memory.supersede_fact` scoped-authorized

Capability: an agent can durably record a scoped observation and supersede a
prior fact through the full gate chain — the first agent-driven memory write in
raiOS history. REQUIRES OD-1 answered. Touch: `agent_protocol_memory.rs`
(`MEMORY_MUTATION_METHODS` handling at :20 and dispatch — route the two methods
to `memory_store.rs` gates; keep the file from growing: new logic lives in
`memory_store.rs` / a new `agent_protocol_memory_write.rs`); `memory.profile`
text at :44 must change from blanket-denied to naming the scoped policy.
Fail-closed: `memory.propose_policy`, `memory.redact`, `memory.compact` STAY
denied with the existing typed reasons; observation kind/authority restrictions
per decision 5(3); secret → denied; quota applies; malformed body → typed
denial naming the failed gate, nothing appended.

Verification: `-Profile memory-durable` extended with families
`memory-write-authorized` (accepted observation: full gate chain evidence,
readback, inspect; ≥10), `memory-write-denied` (each gate failing individually
— schema, classification, kind, quota — each with its own typed reason; ≥8),
`memory-mutation-still-denied` (the three still-denied methods; ≥3).

```text
Packet id: M9B-1-scoped-memory-writes
Goal: Transition memory.record_observation and memory.supersede_fact from denied to scoped-authorized behind the five-gate chain; keep propose_policy/redact/compact denied; prove both directions with needles.
Read first: this map decisions 3-5 and OD-1 resolution; seed-kernel/src/agent_protocol_memory.rs:20-46,3146-3155; seed-kernel/src/memory_store.rs; the M6 grant gate pattern (agent_protocol_module_grant.rs area) as the denial->authorized exemplar
Allowed write set: seed-kernel/src/agent_protocol_memory.rs (dispatch + profile text only); seed-kernel/src/memory_store.rs; seed-kernel/src/agent_protocol_memory_write.rs (new, if dispatch exceeds ~50 lines); vm-harness/shadow-vm-smoke-profile-memory-durable.ps1; docs/PROJECT_STATUS.md
Forbidden: authorizing propose_policy, redact, or compact; accepting kind other than observation or authority other than event from the agent path; accepting secret classification; skipping readback before acknowledging; growing agent_protocol_memory.rs beyond +100 lines
Constraints: every denial must name the exact failed gate; every acceptance must return the durable record id + hash; provider responses must not be able to trigger these methods implicitly (serial agent path only)
Definition of done: memory-durable green with the three new families; quick green; OWNER DECISION OD-1 outcome quoted in the commit message; capability sentence
Report format: report filename; accepted record ids + hashes; the per-gate denial reasons observed
```

### M9C-1 — Context broker v1: durable + current_boot assembly, omissions, `recovery_minimal`

Capability: `memory.context` packets now draw on durable records with per-record
`durable`/`current_boot` scope labels, report explicit omissions with reasons,
add the `recovery_minimal` profile, and carry measured (not hard-coded) token
estimates; `memory.trace` resolves durable record ids to their store evidence.
REQUIRES OD-2 answered (recommended: rich path only). Touch:
`agent_protocol_memory.rs` context/profile/trace emitters (:48, :3066, :3157-3186);
`memory_store.rs` read/selector API. Fail-closed: broker never includes a
record whose readback hash fails (omit with reason `durable_evidence_failed`);
`provider_export` stays `disabled` in this slice; summaries appear only as
locator ids.

Verification: `-Profile memory-durable` extended with families
`broker-durable-included` (durable record included with scope label + trace
resolves it; ≥8), `broker-omission` (over-budget and not-relevant omissions
listed with reasons; ≥5), `broker-recovery-minimal` (packet within 512-1500
target, minimal invariants only; ≥5). Also run quick.

```text
Packet id: M9C-1-context-broker-v1
Goal: memory.context assembles budgeted packets from durable + current_boot facts across recovery_minimal/provider_minimal/diagnostic/planning with explicit omission reporting, measured token estimates, and memory.trace over durable ids.
Read first: this map decisions 6,7,9 and OD-2 resolution; docs/architecture-decisions/0004-system-memory-and-agent-context.md (Context Packet, Token Budget, Retrieval Strategy, Always-Included sections); seed-kernel/src/agent_protocol_memory.rs:48-160,3066-3200; seed-kernel/src/memory_store.rs
Allowed write set: seed-kernel/src/agent_protocol_memory.rs (context/profile/trace emitters); seed-kernel/src/memory_store.rs (read selectors: by id, kind, entity, recency, severity); vm-harness/shadow-vm-smoke-profile-memory-durable.ps1; docs/PROJECT_STATUS.md
Forbidden: any semantic/embedding/BM25 code in the kernel; treating a summary as includable authority (locator ids only); enabling provider export; touching the recovery lifeline protocol; including any record without hash-verified readback
Constraints: replace the static estimated_tokens constants with a bytes/4 estimate labeled estimate_method="bytes_div_4"; every included durable record labeled scope="durable", RAM facts scope="current_boot"; omitted array mandatory even when empty; authority_order gains core_ledger and evidence tiers
Definition of done: memory-durable green with broker-durable-included, broker-omission, broker-recovery-minimal families; quick green; capability sentence
Report format: report filename; one full recovery_minimal packet excerpt from the smoke; measured vs target token numbers per profile
```

### M9C-2 — Provider export gating end-to-end

Capability: a `provider_minimal` context can actually be bound into a provider
request — but only through the complete gate chain, and every export (or
denial) leaves a durable audit record BEFORE bytes leave the machine. Touch:
`agent_protocol_provider.rs` (`provider_context_block_reason`,
`emit_provider_minimal_projection`), provider request binding path
(`openai.rs` binding site — verify at execution time), `memory_store.rs`
(export_audit record kind). Reuses + extends the existing `provider-memory`
profile rather than a new one. Fail-closed: missing/negative provider trust →
export denied + durable denial audit; any local_only field surviving redaction
→ export blocked + `redaction_error` problem; audit append failure → export
fails (audit is load-bearing, not best-effort); framebuffer/serial local
context remains available when provider gates fail.

Verification:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile provider-memory`
extended with families `export-audit-durable` (audit record appended+readback
before transmit marker; ≥6), `export-denied-audited` (each gate failure →
denial + durable denial audit; ≥8), `export-public-only` (local_only field
injected upstream never appears in the export projection; ≥4). Then
`-Profile memory-durable` regression green.

```text
Packet id: M9C-2-provider-export-gating
Goal: End-to-end provider export gating: positive trust + public-only classification filter + redaction + budget + durable export/denial audit records appended-and-readback-verified before any provider transmission.
Read first: this map decision 8; docs/architecture-decisions/0004-system-memory-and-agent-context.md (Provider Boundary); seed-kernel/src/agent_protocol_provider.rs; the provider request binding structs in event_log_types.rs (:71-:141 region at authoring); seed-kernel/src/memory_store.rs; vm-harness/shadow-vm-smoke-profile-provider-memory.ps1
Allowed write set: seed-kernel/src/agent_protocol_provider.rs; the single provider request binding call site (from Slice 0); seed-kernel/src/memory_store.rs (export_audit kind only); vm-harness/shadow-vm-smoke-profile-provider-memory.ps1; docs/PROJECT_STATUS.md
Forbidden: exporting local_only or secret-marker interiors; transmitting before the audit record's readback verifies; weakening provider trust checks; touching TLS/pinning code (that is M10/M11); adding provider-triggered memory writes
Constraints: audit record must bind context packet hash, profile, budget, trust state, and destination; denial audits carry the failed gate name; if the durable store is unavailable, provider export is unavailable (typed denial), local operation continues
Definition of done: provider-memory profile green with the three new families; memory-durable profile green; quick green; capability sentence
Report format: report filenames; one export audit record id + hash and the transmit-ordering needle names proving audit-before-export
```

### M9D-1 — Cross-boot persistence proof + M9 close

Capability: a memory record written in boot N is read back in boot N+1 with
matching hash, ordered by boot generation — durable memory is proven durable,
not asserted. Touch: harness only (two-boot sequence in the memory-durable
profile: boot, write marker observation, clean shutdown, boot again, readback +
hash needle), plus `docs/PROJECT_STATUS.md`, `docs/ROADMAP.md`,
`docs/OWNER_DASHBOARD.md` (plain language: "raiOS can now remember decisions
and problems across restarts, and can prove it"). Verify at execution time:
reuse M7B/M7D two-boot pattern if M7 built one; if none exists this slice
builds it in the profile script. STOP if a clean two-boot flow requires image
layout changes (that is M7 territory, not M9).

Verification: `-Profile memory-durable` (now two-boot) family
`memory-cross-boot` (boot-N record readable in boot N+1, hash match, boot_id
differs, sequence/generation ordering correct; ≥8 needles). THEN the full
checkpoint:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile full`
green before M9 may be called closed (Red Gate Rule applies).

```text
Packet id: M9D-1-cross-boot-proof-and-close
Goal: Extend the memory-durable profile to a two-boot sequence proving a boot-N durable memory record reads back in boot N+1 with hash match; then run the full profile and close M9 in the docs.
Read first: vm-harness/shadow-vm-smoke-profile-memory-durable.ps1; vm-harness/shadow-vm-smoke-support.ps1 (VM lifecycle helpers); any M7 two-boot profile (from Slice 0); docs/OWNER_DASHBOARD.md; docs/ROADMAP.md
Allowed write set: vm-harness/shadow-vm-smoke-profile-memory-durable.ps1; vm-harness/shadow-vm-smoke-support.ps1 (only if a reboot helper is genuinely missing); docs/PROJECT_STATUS.md; docs/ROADMAP.md; docs/OWNER_DASHBOARD.md
Forbidden: kernel changes (if the proof fails, STOP and report — that is repair work, not harness tuning); overwriting release/raios-stage0.img; deleting or reformatting SEED_DATA between the two boots; loosening needles to make the proof pass
Constraints: the two boots must use the same disk image state; needles must assert record id + hash equality across boots and differing boot_id; classify any failure host-transport vs guest-behavior per the Failure Classification Rule before retry
Definition of done: memory-durable two-boot green; FULL profile green with report filename in the closing commit; OWNER_DASHBOARD updated in plain language; ROADMAP cursor moves M9->closed
Report format: both report filenames; the cross-boot record id/hash pair; full-profile needle total
```

---

## Honest uncertainties carried into execution

- The M7B store API, namespace model, quota interaction, and two-boot harness
  support are ASSUMED; Slice 0 must reconcile and may reshape M9A-2/M9D-1.
- `agent_protocol_memory.rs` and `event_log_types.rs` were near split
  thresholds at authoring; line references will drift and files may be split
  by M6-M8 work before M9 starts.
- Token estimation is a heuristic (bytes/4), labeled as such — real tokenizer
  parity is out of scope until a provider adapter (M10) makes it testable.
- Boot-generation ordering assumes M7C exposes a counter; if it does not, M9A-2
  falls back to store append order + boot_id inequality and says so in the
  record.
- Quota defaults (128 records / 32 KiB per boot) are engineering picks, not
  measured; revisit after first real usage data, fail-closed in the meantime.

Estimate: 7 implementation slices + Slice 0; one new focused profile, one
extended existing profile, one full checkpoint. The hard part is not the store
(M7B delivers it) — it is keeping every write path behind the same five-gate
chain without letting "memory" become a second, weaker authority system beside
the capability ledger.
