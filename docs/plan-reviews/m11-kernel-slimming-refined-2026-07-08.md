# M11 Kernel Slimming / Services-out-of-kernel — Refined Map (2026-07-08)

Refined against HEAD `209c450`. Companion to (does not replace)
`m11-kernel-slimming-map-2026-07-06.md`; where they conflict, THIS file wins.
Execution preconditions unchanged: M6+M7 closed (TRUE), M10 closed (pending) —
M11-2/-3 strictly need only M6+M7D and may be pulled earlier ONLY by explicit
owner instruction.

**Milestone capability sentence (unchanged, reaffirmed).** No application-layer
internet content — TLS records/handshake, HTTP responses, provider JSON — is
parsed by kernel-resident code; it runs in fuel-metered Wasm services with a
narrow granted import surface, and the provider path is a promoted, persistent,
replaceable service. The kernel measurably shrinks.

## 1. Drift vs the 2026-07-06 map

1. **The M6/M7/M8 interfaces the map guessed are now REAL — names confirmed:**
   - Candidate delivery: `module.submit_candidate_chunk` /
     `module.submit_candidate_finalize` (`seed-kernel/src/module_candidate_channel.rs`),
     sink `intake_external_wasm_candidate` (`module_candidate_intake.rs`),
     bounded base64 over serial, paced sends (UART FIFO lesson, M9B-1b).
   - Grant: `agent_protocol_module_grant.rs` — evidence-bound, dev-key P-256
     verified, `trust_tier=dev_key_not_owner_sealed` (`:451`, `:866`).
   - Load/run: `granted_candidate_service.rs` (M6C) under the UNCHANGED M4
     envelope; un-promote via `service.rollback_apply svc.dev.granted_candidate`
     (M6D-1).
   - Persistence: durable promotion transaction (M6D-2,
     `scoped_promotion_transaction_append`), ARTSTOR blob + `artifact_persist`
     record (M7D-1, `artifact_store.rs`), boot-time re-verify `repromotion.rs`
     (M7D-2), re-instate-by-hash `recovery.load_artifact_by_hash` (M8D, shares
     `reverify_persisted_artifact`).
   M11-2's promotion/persist/reboot chain is therefore fully buildable against
   real methods — the two-boot harness (`vm-harness/shadow-vm-persistence-reboot.ps1`)
   already proves the exact reboot-re-promotion pattern M11-2 must pass.
2. **CRITICAL GAP CONFIRMED — the map's M11-2 STOP-tripwire FIRES at planning
   time:** the M6 grant vocabulary does NOT express per-service import
   surfaces. The capability envelope is a FIXED linker set —
   `define_capability_envelope` defines exactly `env.log` + `env.counter_get`
   (`seed-kernel/src/wasm_runtime.rs:680-682`) and `granted_candidate_service.rs`
   reuses it. "svc.net.httpparse gets env.log ONLY" and M11-4's grantable
   net/crypto imports are inexpressible today. Resolution: NEW slice
   **M11-2a** below (pre-answered so the orchestrator can present one decision
   to the owner together with D1).
3. **Ledger baseline moved:** `wasm_runtime.rs` is 737 lines (map: 578 — grew
   with M6C execution + M8A-3 crash-latch work). Provider files unchanged:
   `openai.rs` 1,624 / `tls_io.rs` 105 / `openai_trust.rs` 342 / `net.rs` 756.
   Whole-kernel baseline: `seed-kernel/src` = **149,979 lines** (largest:
   `agent_protocol_module_loader_runtime.rs` 10,156). The parse block
   `openai.rs:1322-1597` is likely still accurate (file untouched since the
   map) — re-locate in M11-0 anyway.
4. **M10 will reshape the target files BEFORE M11 runs** (roadmap order kept):
   M10B-2 splits `openai.rs` into generic `provider_client.rs` + wire
   `openai_adapter.rs`. M11-2/-3 packets must be re-pointed at
   `provider_client.rs`'s response-parse seam at M11-0 revalidation. This is a
   FEATURE: the M10B-2 seam (response bytes in, typed result out) is exactly
   where M11-3 routes bytes to the guest. If the owner ever re-orders M11
   before M10, M11-3 must cut against monolithic openai.rs — more work;
   RECOMMEND keeping M10 first.
5. **M9C-2 export gating exists by M11 time:** the audit-before-transmit
   ordering and context-attach live in the generic client layer. M11-5/-6/-7
   must keep the secret-header splice AND the export-audit ordering
   kernel-side — the guest sees neither key bytes nor the pre-audit window.
   Add to §8 "never leaves the core": the M9C-2 export gate + durable audit
   append.
6. **Cadence:** aggressive-fast — focused profile per slice; FULL only at the
   M11-3 cutover, M11-6, and M11-7 (the map already matched this roughly);
   `recovery` byte-identical is the enforcement tool for "the lifeline never
   depends on any M11 service".
7. **M8 exists:** the recovery lifeline is real now (pinned table, own
   dispatch path, `vocabulary_sha256` fence). Any M11 slice that would touch
   lifeline files = STOP (unchanged rule, now concrete).

## 2. Reaffirmed design verdicts

- **Crypto primitives stay narrow HOST imports; parsing moves to Wasm** —
  reaffirmed; M11-1 measures before any TLS design freezes.
- **Opaque session handles, keys never in guest memory, kernel splices the
  secret header mid-stream** — reaffirmed, still unproven (M11-5 risk).
- **D1** (record the session-handle/crypto-import design): RECOMMEND one short
  ADR covering BOTH D1 and the M11-2a import-surface mechanism (they are the
  same trust-model addition: "what a grant can express about imports").
- **D2** (cert verdicts stay kernel-side): reaffirmed for M11.
- **D3** (guest authorship): (a) in-repo source submitted at runtime through
  the REAL M6 serial channel — reaffirmed; the channel + pacing discipline
  exist and are proven.

## 3. Slice re-cut (dependency-ordered)

### M11-0 — map revalidation (MANDATORY)

As mapped, narrowed: re-locate the parse block inside post-M10B-2
`provider_client.rs`; re-check `wasm_runtime.rs` linker/envelope line numbers;
confirm the M10D-1 handoff doc exists and what the embedded-tls patch exposes;
snapshot the LOC ledger (per-file + `seed-kernel/src` total) as the shrink
baseline. **Ready-to-scope:** YES when M10 closes.

### M11-1 — interpreted-crypto measurement guest

As mapped (packet `M11-1-crypto-bench-guest` stands). Submit the bench
artifact through `module.submit_candidate_chunk/finalize` (real path, paced
sends). **Verify:** quick + bench needles. **Ready-to-scope:** YES after
M11-0; parallel with M11-2a (disjoint: guest crate + probe vs grant
vocabulary).

### M11-2a — NEW: import-surface grant vocabulary (the un-blocked STOP)

- **Capability:** a service's granted capability envelope is DATA, not a
  hardcoded linker set — the grant/descriptor carries an explicit import
  allowlist, the linker builds the envelope from it, and an import not on the
  list still fails AT INSTANTIATION (M4's negative proof, now per-service).
- **Design (pre-answered for the owner, with D1 in one short ADR):** an
  `import_surface` list of pinned import ids (e.g. `env.log`,
  `env.counter_get`) bound into the computed-grant hash; unknown name → typed
  denial at grant evaluation; the existing built-in services keep their exact
  current envelopes byte-identically (echo/hello regression needles).
- **Split:** 2a-i raios-core rules + grant-hash binding (grants nothing) →
  2a-ii kernel envelope-from-grant flip.
- **Write set:** `raios-core/src/**` (grant/import-surface rules),
  `seed-kernel/src/wasm_runtime.rs` (envelope builder),
  `agent_protocol_module_grant.rs` + `granted_candidate_service.rs` (consume),
  quick/m6c-promotion needles.
- **Key risk:** SECURITY boundary — widening the envelope for existing
  services by accident; the import allowlist becoming advisory instead of the
  linker source of truth. Max review on 2a-ii.
- **Verify:** focused `m6c-promotion` + `quick`; negative needle (module
  importing an unlisted fn fails at instantiation).
- **Ready-to-scope:** after the D1+2a ADR is approved. BLOCKS M11-2/-4.

### M11-2 — httpparse service through the real loop (parallel proof)

As mapped (packet `M11-2-httpparse-service-parallel` stands) with:
import surface = `env.log` only via M11-2a; promotion + M7D persistence +
two-boot re-promotion needles reuse the existing persistence-reboot pattern;
old parser stays authoritative. **Verify:** new `net-parser` profile + quick +
host `net-parser-core` tests. **Ready-to-scope:** after M11-2a + D3 confirm.

### M11-3 — cutover: kernel stops parsing HTTP/JSON (first real shrink)

As mapped (packet `M11-3-httpparse-cutover` stands), re-pointed at
`provider_client.rs` post-M10B-2. Delete the parse functions (~275 lines at
the 2026-07-06 count) + the comparison harness; no in-kernel fallback exists
afterwards; parser-absent/failed → typed fail-closed. **Verify:** net-parser +
quick + **FULL checkpoint**; owner-run live smoke optional but recommended.
**Ready-to-scope:** after M11-2 green.

### M11-4 — crypto/session host-import surface + tls-record-core (vectors only)

As mapped (packet `M11-4-crypto-session-imports` stands); precondition D1 ADR
(now the combined D1+M11-2a ADR). Imports granted to NOBODY this slice; handle
tables generation-checked; no import returns key bytes. **Verify:** host
vectors + quick (ungranted-import instantiation-failure needle).
**Ready-to-scope:** after M11-3 + ADR.

### M11-5 — live provider record layer through the guest

As mapped (packet `M11-5-record-layer-live` stands). Riskiest seam: exporting
negotiated traffic secrets from vendored embedded-tls into the kernel session
table — merge-with-M11-6 fallback stays pre-authorized (>~200 vendored-line
surgery = STOP). Secret header via `env.provider_write_secret_header`; M9C-2
audit ordering stays kernel-side ahead of any guest-visible byte. Closure
REQUIRES one green owner-run live smoke. **Ready-to-scope:** after M11-4.

### M11-6 — handshake in the guest; embedded-tls leaves the kernel

As mapped (packet `M11-6-handshake-guest-embedded-tls-out` stands); D2(a)
kernel verdicts via `env.tls_verify_cert_chain` — which by now can consult the
M10D host-proven chain-validation logic RUN IN-GUEST? NO — verdicts stay
kernel (pinned + M10D-informed policy); the M10D handoff doc names the import
surface. Largest net deletion of M11 (embedded-tls out of the dependency
graph, `tls_io.rs` deleted/shrunk). **Verify:** host handshake vectors,
provider-service profile, quick, live smoke (owner), **FULL checkpoint**.
**Ready-to-scope:** after M11-5 (or merged with it per the fallback).

### M11-7 — provider path as ONE promoted, persistent, replaceable service

As mapped (packet `M11-7-provider-as-service` stands); if no inter-service
call mechanism exists (none does today), MERGE into one artifact and record
composition as an M12+ direction note. Un-promote → fail-closed
provider-absent; reboot re-promotion under M7D gates. Kernel keeps: sockets/
DNS, crypto/session imports, secret splice, trust verdicts, export gate +
durable audits, evidence emission, UI glue. **Verify:** provider-service +
net-parser + quick + live smoke + **FULL before closing M11**.
**Ready-to-scope:** after M11-6.

## 4. Kernel-shrink ledger (standing rule, updated baseline)

Baseline at 2026-07-08: `seed-kernel/src` = 149,979 lines; targets:
`openai.rs` 1,624 (parse block ~275), `tls_io.rs` 105, `net.rs` 756 (DNS
parse `:579-735` = next candidate post-M11), `openai_trust.rs` 342, vendored
embedded-tls 0.17 (out at M11-6). Every extraction slice reports: net
`seed-kernel/src` delta, deleted-function list, byte-identical provider
needles. M11-1/-4 are the only allowed net-additive slices; additions repaid
by M11-3/-5/-6.

Honest note: M11's ~1,500-line direct deletion is small against a 150k-line
kernel — the real wins are (a) attacker-influenced parsing leaves ring 0,
(b) the provider path becomes replaceable WITHOUT a kernel rebuild, (c) the
test-time structural fix (a change to the provider service re-verifies only
its own service profile, not an 8k-predicate full kernel run). State all three
in the M11 close, not just LOC.

## 5. STOP-tripwires (delta)

All map §10 tripwires stand. Add: any M11 slice touching the M9C-2 export
gate/audit ordering; any grant-vocabulary change beyond the approved
import-surface ADR; `recovery` profile not byte-identical on ANY M11 slice
(lifeline independence violated).

## 6. Dependency graph

```text
M10 close (adapter split + chain-validation handoff)
  └─> M11-0 → M11-1 ∥ M11-2a(ADR: D1 + import-surface) → M11-2 → M11-3 (FULL)
                                        └────────────────> M11-4 → M11-5 → M11-6 (FULL) → M11-7 (FULL, close)
Post-M11 candidates: DNS parse out; X.509-parse-in-Wasm (needs ADR); console text rendering.
M11 preferred-before: M12+ external distribution (download client as a Wasm service).
```
