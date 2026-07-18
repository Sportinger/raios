# M9C-2 Provider Export Gating — Refined Map (2026-07-08)

Refined against HEAD `209c450` (M9C-1a) plus the in-flight M9C-1b working tree
(durable_store.rs / memory_store.rs / agent_protocol.rs / agent_protocol_memory.rs /
memory-durable profile — implemented, awaiting orchestrator VM proof + commit).
Supersedes the M9C-2 section of `m9-durable-memory-map-2026-07-06.md` for
execution; the original map stays as authored (do not edit it).

**Milestone capability sentence (M9C-2).** "A `provider_minimal` context can be
bound into a real provider request — but ONLY through the complete export gate
chain (positive trust, known profile, public-only classification, redaction,
budget), and every export or denial leaves a durable `export_audit` /
`capability_denial` record appended-and-readback-verified BEFORE any byte
leaves the machine. The default remains fail-closed everywhere."

**THIS IS A SECURITY SLICE.** It is the first time memory-derived bytes can
leave the machine. Max-effort adversarial review is MANDATORY on the authority
flip (2c). Everything before the flip grants nothing.

## 1. Current reality (verified at HEAD, 2026-07-08)

| Surface | Reality | file:line |
|---|---|---|
| Export method | `provider.context_export` is blanket-denied via `MethodAction::DeniedProviderContextExport` | `seed-kernel/src/agent_protocol.rs:524` (handler `:683-685`) |
| Block reason | `provider_context_block_reason(trust)` → `provider_trust_not_positive` else `provider_context_export_audit_binding_missing` | `seed-kernel/src/agent_protocol_provider.rs:2262-2268` |
| Positive trust def | `pinned_cert_verified \| pinned_spki_verified \| webpki_verified` (dev bypass is NOT positive) | `agent_protocol_provider.rs:2255-2260`, `seed-kernel/src/openai.rs:443-449` |
| Broker export label | `memory.context` emits `provider_export: "disabled"` (hard string) | `seed-kernel/src/agent_protocol_memory.rs:64` (+ profile rows `:33-35`, omitted row `:226`) |
| Ask-path envelope | `context_attached_to_provider_body:false`, `provider_minimal_context.attached:false`, `binding_status:"not_bound"` hardcoded into envelope hash AND emission | `openai.rs:497-517` (hash), `:558-583` (emit) |
| Export-audit BINDING (RAM) | On the positive-trust ask path the kernel ALREADY creates `ProviderRequestBinding` + `ProviderExportAuditBinding` events with `export_audit_binding_hash` — RAM events only, no context attached, no durable record | `openai.rs:608-679` (`record_positive_provider_context_bindings`), schemas rendered at `agent_protocol_memory.rs:2166-2232` |
| Redaction projection | `emit_provider_minimal_projection` exists (fixed redacted fields, `provider_export:"disabled"`) | `agent_protocol_provider.rs:379-434` |
| Durable writer | `append_memory_record` through the shared reclog gauntlet, authorized ONLY by `evaluate_scoped_memory_record_append`; per-boot RAM quota 128 records / 32 KiB | `seed-kernel/src/durable_store.rs` (M9A-2b), `raios-core/src/scoped_memory_record_append.rs` |
| `export_audit` kind | Already in the `MemoryKind` 8-value allowlist AND in the scoped evaluator's kind scope; audit kinds can never be authored as superseding (M9A-3a) and can never be hidden by a reader (R1, resolver) | `raios-core/src/memory_record.rs`, `raios-core/src/memory_record_resolve.rs:53` |
| Classification firewall | `Classification` has NO `Secret` variant — a secret plaintext is structurally un-constructable and can never become durable; unknown → `local_only` | `raios-core/src/memory_record.rs` (M9A-1) |
| Broker exportability (M9C-1b, in flight) | Per durable record `exportable = classification=="public"`; `local_only` surfaces locally only; raw record `value` is NEVER emitted by the broker | working tree `durable_store.rs::copy_durable_memory_record`, cap `MAX_DURABLE_RECORDS_SURFACED=64` |

## 2. Drift vs the 2026-07-06 map (M9C-2 packet)

1. **The binding vocabulary already exists.** The map's "audit record …
   appended before transmit" reads as if the export-audit concept must be
   invented; in reality `raios.provider_context_export_audit_binding.v0` and
   `raios.provider_context_export_denial_audit.v0` already exist as RAM events
   on the ask path. M9C-2's real job: make the audit DURABLE and LOAD-BEARING
   (readback-verified before transmit) and actually attach context — not new
   vocabulary.
2. **The "single provider request binding call site (from Slice 0)" is now
   precisely known:** `openai.rs:608` (`record_positive_provider_context_bindings`)
   plus the envelope hash/emit pair `openai.rs:417-534/:558-583`.
3. **Positive trust is unreachable inside VM profiles.** The provider-memory
   profile runs keyless/pinless → trust never enters the positive set in QEMU.
   The map's `export-audit-durable` needle family ("audit appended+readback
   before transmit marker") cannot run as-is in a profile. Re-scoped below into
   (a) a NO-TRANSMIT in-VM selftest proving the ordering machinery and
   (b) ONE owner-run live smoke for the true end-to-end (mirror of the M11-5
   honesty rule).
4. **New write-side rules the map predates:** export_audit may never supersede
   (M9A-3a), the resolver protects audit kinds read-side (R1/M9C-1a), the
   durable write quota exists and denial-audit spam could exhaust it (new
   design decision D-A below).
5. **Cadence:** aggressive-fast (AGENTS.md 2026-07-07) — focused profile per
   sub-slice, adversarial review only on the risky flip, full only at M9 close.

## 3. DECISION — does M9C-2 depend on M10 provider trust? **NO.**

- The gate consumes the EXISTING positive-trust definition
  (`pinned_cert_verified | pinned_spki_verified | webpki_verified`,
  `agent_protocol_provider.rs:2255`). Pin-verified SPKI trust is real,
  reachable today via a local key image (`vm-harness/openai-direct-smoke.ps1
  -ExpectSpkiPinnedTrust`), and is the same trust tier under which `ask`
  already sends the user's prompt off-machine.
- ADR 0004 Phase F requires "the provider transport is positively verified and
  the redaction profile is implemented in code" — pin-only satisfies the first
  clause; M9C-2 implements the second.
- The load-bearing security boundary is the CLASSIFICATION FIREWALL + the
  durable audit, not the TLS tier. M10 later upgrades trust QUALITY (real
  chains, honest time, second provider) behind the SAME unchanged gate chain.
- **Sequencing: ship M9C-2 BEFORE M10.** Benefit: M10B-2's byte-identical
  adapter split then automatically covers (and its needles protect) the export
  path; M10C's second provider inherits the identical export denial chain for
  free through the generic layer.

## 4. The exact gate chain (fail-closed default at every step)

Order is normative. Any failure → typed denial naming the gate + durable
denial audit (per D-A) + local operation continues; nothing transmits.

1. **Method gate** — only `provider.context_export` is evaluated; every other
   `memory.*` mutation and provider method stays exactly as denied today.
2. **Profile gate** — only `provider_minimal`.
3. **Trust gate** — `provider_trust::snapshot()` state must be in the positive
   set. `tls_certificate_verification_bypassed` (dev bypass) is NOT positive —
   already structurally true; pin a needle on it anyway.
4. **Classification gate (THE FIREWALL)** — the export payload is assembled
   EXCLUSIVELY from: (a) durable records with `exportable == true`
   (i.e. `classification == "public"` per the M9C-1b broker), and (b) the
   existing `emit_provider_minimal_projection` redacted current_boot fields.
   `local_only` and secret-marker interiors are structurally excluded (secret
   is un-constructable; local_only is filtered by the broker's exportable
   flag). Raw durable record `value` bodies are never exported (the M9C-1a
   parser never decodes them — keep that property).
5. **Budget gate** — measure the ACTUAL projection bytes (bytes/4, labeled
   `estimate_method:"bytes_div_4"`) against provider_minimal's 2000-token
   target; over-budget → deny (not truncate-silently; explicit omission).
6. **Export-audit gate (load-bearing)** — durably append ONE
   `raios.memory_record.v0` with `kind=export_audit`,
   `classification=local_only`, `authority=core_ledger`, binding: context
   packet sha256, profile, measured budget, trust snapshot (state/pin id),
   destination host, and the RAM `export_audit_binding_hash`. Append +
   readback-verify MUST complete BEFORE the API-key copy and before any TLS
   body write. Append failure / SAFE posture / store absent / quota exhausted
   → the export fails (typed `export_audit_unavailable`), never best-effort.
7. **Denial audit** — any gate 2-6 failure durably appends a
   `kind=capability_denial` record naming the failed gate (see D-A).

**D-A (new design decision — denial-audit quota discipline).** Durable denial
audits are deduped per boot: the FIRST denial per (gate, reason) pair is
durably appended; identical repeats are RAM-ring events that cite the durable
record id. Denials OF the audit append itself are NEVER durably recorded (no
self-recursion — inherited from map decision 1a). This keeps smoke profiles
(which fire the denial chain dozens of times) from exhausting the 128-record
per-boot quota.

## 5. Re-cut slices (dependency-ordered)

### M9C-2a — export-gate evaluator + export_audit record rules (raios-core, grants nothing)

- **Capability:** one host-tested, pairwise-unique-denial evaluator answers
  "may this context packet be exported, and what must the audit record bind"
  — nothing wires it.
- **Boundary/evaluator:** NEW `raios-core/src/scoped_provider_export.rs`
  (style: clone of `scoped_memory_record_append.rs`): pinned EXPECTED_METHOD
  `provider.context_export`, EXPECTED_PROFILE `provider_minimal`, the positive
  trust-state allowlist as data, classification filter rule
  (public-only in, any local_only in the packet → deny), budget rule,
  audit-field completeness rule (packet hash + destination + trust snapshot +
  binding hash all required), and the D-A dedupe key shape.
- **Write set:** `raios-core/src/scoped_provider_export.rs` (new),
  `raios-core/src/lib.rs` (export line), host tests. Nothing else.
- **Key risk:** none at runtime (unwired). Design risk: denial reasons must be
  pairwise-unique and stable (they become needles).
- **Verify:** `cargo test --locked -p raios-core` + rustfmt. No VM.
- **Ready-to-scope:** YES immediately after M9C-1b commits (reuses its
  exportable/broker surface as input shape).

### M9C-2b — durable denial audits, export still denied (kernel; dispatch row touch)

- **Capability:** every `provider.context_export` attempt is now evaluated
  through the full chain and the DENIAL is durably recorded (deduped per D-A)
  — raiOS can prove after reboot that it refused an export and why. Nothing
  transmits; `memory.context` keeps `provider_export:"disabled"`.
- **Boundary:** flips `agent_protocol.rs:524` from
  `MethodAction::DeniedProviderContextExport` to a Read handler that runs the
  evaluator and appends via the EXISTING `append_memory_record`
  (kind=capability_denial, quota-charged, no new writer). Deny-before-append:
  a malformed request denies RAM-only without burning quota.
- **Write set:** `agent_protocol.rs` (one method row),
  `agent_protocol_provider.rs` (evaluation + emission),
  `seed-kernel/src/memory_store.rs` (denial-audit driver + dedupe table),
  `vm-harness/shadow-vm-smoke-profile-provider-memory.ps1` (needles),
  `docs/PROJECT_STATUS.md`.
- **Key risk:** dispatch-table row change on a shared file (review the diff);
  quota exhaustion if D-A dedupe is wrong (needle: fire the same denial 3x,
  reclog advances exactly +1).
- **Verify:** focused `provider-memory` + `memory-durable` regression; cheap
  review (grants nothing, but shared dispatch file).
- **Ready-to-scope:** YES after 2a. Independent of M9D.

### M9C-2c — THE AUTHORITY FLIP: audited public-only export attaches to a real request (SECURITY)

- **Capability:** with every gate positive, `ask`-path provider requests can
  carry the provider_minimal context: durable export_audit appended +
  readback-verified FIRST, then the public-only projection is attached to the
  request body (`context_attached_to_provider_body:true`,
  `provider_minimal_context.attached:true`, `binding_status:"bound"` — all
  honest, envelope hash updated), THEN transmit. The RAM binding events now
  cite the durable audit record id/hash.
- **Boundary:** `openai.rs` positive path only (`:608-679` region + envelope
  builders + body build); the evaluator from 2a is the sole authorizer; the
  key copy and TLS body write move BEHIND the audit readback (they already sit
  behind the trust gate — ordering is additive).
- **In-VM proof (keyless):** a NO-TRANSMIT selftest
  (`provider.context_export_selftest`, labeled test infrastructure) drives the
  full positive chain with a synthetic trust snapshot up to but excluding the
  socket write: proves audit-append-before-attach ordering, the exact
  projection content (public-only; an upstream-injected local_only record
  never appears — `export-public-only` family), and the denial matrix.
  PLUS ordering serial markers on the real path (audit-appended marker before
  the TLS-write marker) for the live smoke to assert.
- **Live proof (owner-run, REQUIRED for closure):**
  `vm-harness/openai-direct-smoke.ps1` gains `-ExpectContextExport`: with a
  local key image, one real request carries the context, the durable
  export_audit lands before transmit, the response returns. M9C-2 cannot be
  called closed without this one green run (honest requirement; needs the
  owner's OpenAI key image — routine, NOT the sealing ceremony).
- **Write set:** `openai.rs`, `agent_protocol_provider.rs`,
  `memory_store.rs`, `agent_protocol_memory.rs` (the `provider_export` label
  becomes state-derived: `"disabled"` until the gates pass — derive, don't
  hardcode true), `vm-harness/shadow-vm-smoke-profile-provider-memory.ps1`,
  `vm-harness/openai-direct-smoke.ps1`, `docs/PROJECT_STATUS.md`,
  `docs/OWNER_DASHBOARD.md`.
- **Key risk (flag loudly):** memory bytes leaving the machine — a local_only
  leak through projection assembly; transmit-before-audit ordering bug; the
  envelope hash must change (it hashes `context_attached…=false` today at
  `openai.rs:497-517`) — needles for the envelope are ground truth, derive new
  ones from observed serial, never invent.
- **Verify:** focused `provider-memory` + `memory-durable` + `quick`;
  **max-effort adversarial review MANDATORY**; owner-run live smoke.
- **Ready-to-scope:** YES after 2b; schedule the owner's key-image availability
  with the dispatch.

## 6. M9D validation (scope confirmed, brief)

The pasted M9D scope is CONFIRMED against HEAD, with these anchors:

- Harness pattern exists: `vm-harness/shadow-vm-persistence-reboot.ps1`
  (two boots on one persist disk; boot-2 assert style at `:603`
  `Assert-Boot2Repromotion`, `:739` `Assert-Boot2LoadArtifactByHash`, main
  flow `:962-973`). Persist-disk builder `scripts/make-gpt-persist-image.py`
  has the `--seed-reclog-fixture` support (M7B-1) for the torn-tail fixture.
- **Dependency CONFIRMED: M9D consumes M9C-1b.** Boot 2's resolve-after-reboot
  proof needs the broker surface (`memory.context` durable_records +
  superseded/omission folds). Boot 1 should write through the EXISTING methods
  (`memory.decision_problem_log_append` A/P/B incl. the supersede); boot 2
  asserts B visible + A hidden (`durable_superseded`) + frame survival
  (count/tail_seq/status/endpoint-hash) — read-only, no boot-2 write.
- Torn-reclog fail-closed: torn fixture → `durable.record_log_scan` reports
  the torn tail, appends stay denied; the broker surfaces only
  integrity-verified frames (M7B semantics, unchanged).
- `boot_id` stays `"current_boot"` honestly; a persistent boot-id is a NEW
  durable write authority — correctly deferred; record it as a named known gap
  in the M9 close entry.
- **Parallelism:** M9D (harness+docs only) is dispatch-parallel with
  M9C-2a/2b (disjoint write sets). M9 close requires M9C-2c + M9D + FULL
  profile + recovery byte-identical + dashboard update (Red Gate applies).

## 7. Sequencing (decisive)

```text
M9C-1b (in flight: VM proof + commit)
  ├─> M9C-2a (raios-core) ─> M9C-2b (denial audits) ─> M9C-2c (FLIP + live smoke)
  └─> M9D (two-boot read-only proof)          [parallel lane]
M9 CLOSE = M9C-2c + M9D + full + recovery-byte-identical
  └─> M10 (same gate chain, better trust)  └─> M11
```
