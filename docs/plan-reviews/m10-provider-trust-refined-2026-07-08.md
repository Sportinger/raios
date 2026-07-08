# M10 Provider Trust Hardening & Adapters — Refined Map (2026-07-08)

Refined against HEAD `209c450` + the in-flight M9C-1b tree. Companion to (does
not replace) `m10-provider-trust-map-2026-07-06.md`; where this file and the
original conflict, THIS file wins. Original slice packets remain usable with
the corrections below.

**Milestone capability sentence (unchanged, reaffirmed).** Two providers
(OpenAI + Anthropic) through one typed descriptor-driven contract, per-provider
SPKI-pinned trust, per-provider RAM-only keys, an honest owner-attested time
authority, and a HOST-proven WebPKI chain-validation core staged for M11 —
without weakening a single existing denial.

## 1. Drift vs the 2026-07-06 map

**Headline: the provider surface did NOT drift.** M7/M8/M9 never touched it —
the map's baseline table is confirmed at HEAD almost line-for-line:

| Claim (map) | HEAD 2026-07-08 |
|---|---|
| `provider_trust.rs` TrustState 9 states, `WebPkiVerified` unreachable | CONFIRMED — enum at `seed-kernel/src/provider_trust.rs:28`; pins from `RAIOS_OPENAI_*` env `:3-6`; singleton `static STATE` `:8`; `OPENAI_PINNED_TLS_VERIFIER_METADATA` hardcodes `api.openai.com`, `chain_policy="pin_only_no_webpki_chain_validation"`, `time_policy="not_validated_stage0"` (~`:89-101`) |
| `openai.rs` 1,624 lines; consts `:21-24`; hashing/binding sites | CONFIRMED — exactly 1,624 lines; `API_PATH="/v1/responses"` `:23`, `MODEL="gpt-5.4"` `:24`; positive-context-binding site `:608-679` |
| `openai_trust.rs` 342 lines, P-256/SHA-256-only CertificateVerify | CONFIRMED (342 lines) |
| `provider_config.rs` 102 lines, ONE OpenAI slot | CONFIRMED (102 lines) |
| No time source; `now_ms()` boot-relative | CONFIRMED |
| `tls_io.rs` 105 / `net.rs` 756 | CONFIRMED |

**What DID drift:**

1. **Profile list grew**: `vm-harness/shadow-vm-smoke.ps1:13` ValidateSet is
   now `full, quick, recovery, hello-rollback-dry-run, module-audit-rollback,
   provider-memory, provider-memory-full, candidate-delivery, m6c-promotion,
   m6d-rollback, m8-lifeline, persistence, memory-durable`. New profiles
   (M10's `provider-adapter`) still register there + a dispatch branch.
2. **M9C-2 lands FIRST (per the refined M9C-2 map).** The export gate chain +
   durable export_audit/denial-audit records will live in `openai.rs` /
   `agent_protocol_provider.rs` / `memory_store.rs` BEFORE M10 starts.
   Consequences baked into the slices below:
   - M10B-2's byte-identical split must carry the export-audit path into the
     GENERIC `provider_client.rs` (an adapter must never see context assembly
     or key bytes) — its needles now include the M9C-2 families.
   - M10B-1's per-provider trust table must preserve
     `provider_trust_positive` / `provider_context_block_reason`
     (`agent_protocol_provider.rs:2255-2268`) semantics per provider; the
     OpenAI export gate stays byte-identical.
   - M10C's Anthropic adapter inherits the SAME export denial chain through
     the generic layer — verify with a needle family, not by assumption.
3. **Aggressive-fast cadence** (AGENTS.md, 2026-07-07) postdates the map:
   per sub-slice run ONLY its focused profile; adversarial review only on
   risky steps (trust-state changes, verifier parameterization, key handling);
   `full` + `recovery` byte-identical only at M10 close.
4. **Attested source set:** no provider file is in `seed-kernel/build.rs`'s
   `HELLO_ARTIFACT_SOURCE_SET` (`build.rs:7`) — provider slices need NO
   descriptor-resign (re-confirm in Slice 0; cheap).
5. **Durable memory exists now.** Standing rule for all M10 slices: provider
   trust-state transitions stay RAM/event-only; the ONLY durable records on
   the provider path are M9C-2's export/denial audits. Do not add durable
   trust records in M10 (would need its own scoped-evaluator design — out of
   scope).
6. **raios-core record model** now has ~12 scoped evaluators + record entries
   as precedent; all five new M10 schemas are confirmed cheap as record-model
   entries.

## 2. Decisions — reaffirmed or updated

- **D1 (WebPKI placement): Order B REAFFIRMED** — host-proven core in M10D,
  in-path chain validation ships with M11's TLS service. Nothing at HEAD
  changed the ring-0 attack-surface argument; ADR 0005 still says the kernel
  must not grow internet parsers.
- **D2 (trusted time): owner-attested SET + RTC-unattested input REAFFIRMED.**
  Note: `created_at{clock:"boot_relative"}` in `raios.memory_record.v0` was
  designed to wait for exactly this — after M10A-1, memory records MAY gain a
  wall-time cross-check field in a LATER slice, but M10 itself must not touch
  memory_record.rs (schema stability; not in scope).
- **D3 (adapter/registry shape): REAFFIRMED** unchanged, including the
  owner's local-inference-agnostic constraint.
- **D4 (keys RAM-only, never SEED_DATA): REAFFIRMED** — now load-bearing
  against a REAL durable store; the scoped memory-record evaluator already
  makes secrets un-constructable as durable records, but keys must also never
  appear in export audits (audit binds `api_key_state` set/missing at most).
- **OD-1 (Anthropic key/model for the positive smoke): STILL OPEN — ask
  before M10C-1.** Re-verify model ids at execution (`claude-opus-4-8` /
  `claude-haiku-4-5` were current at authoring).

## 3. Slice re-cut (dependency-ordered)

Lane structure unchanged: M10A and M10D-host are independent lanes; M10B
(1→2→3) is the critical path; M10C is the proof slice.

### M10-0 — map revalidation (MANDATORY, now cheap)

- **Capability:** planning integrity. Most claims pre-confirmed above; verify
  ONLY: the embedded-tls verifier-input patch state
  (`vendor/embedded-tls-0.17.0`, whether intermediate DERs are exposed — feeds
  M10D-1), console SET grammar (`console.rs`, 1,823 lines), whether M9C-2
  moved/renamed any cited line, current Anthropic model ids, and the
  api.anthropic.com leaf key type (`openssl s_client` from the host — R1
  pre-check pulled forward so M10C risk is known at kickoff).
- **Write set:** this file only. **Verify:** none (docs).
- **Ready-to-scope:** YES the moment M9 closes.

### M10A-1 — owner-attested time authority v0 (independent lane)

As mapped (packet `M10A-1-time-authority-v0` stands). Additions:
- `raios.time_authority.v0` is a record-model entry; NO durable write (RAM,
  `current_boot` — a durable time-attestation record would be a new write
  boundary; explicitly out of scope).
- Re-attestation = superseding EVENT (RAM ring), not a durable supersede.
- **Key risk:** scope creep into trust states — forbidden list unchanged.
- **Verify:** quick + 4 time needles. **Review:** none (grants nothing).
- **Ready-to-scope:** YES, parallel with M10B-1 (disjoint files).

### M10B-1 — provider descriptors + per-provider trust table

As mapped (packet `M10B-1-provider-descriptor-trust` stands) plus:
- MUST preserve `provider_trust_positive`/`provider_context_block_reason`
  per-provider semantics; the M9C-2 export gate reads the ACTIVE provider's
  state — with one provider registered pinless, prove the export gate denies
  with `provider_trust_not_positive` for it while OpenAI is unaffected
  (one new needle).
- OpenAI serial output byte-identical (incl. M9C-2 needles) — needles are
  ground truth.
- **Key risk:** singleton→table refactor silently changing
  `allows_provider_request` semantics. **Review:** yes (trust boundary).
- **Verify:** focused `provider-memory` + `quick`.
- **Ready-to-scope:** YES after M10-0.

### M10B-2 — typed completion contract + adapter extraction

As mapped (packet `M10B-2-adapter-extraction` stands) plus:
- The M9C-2 export machinery (gate evaluation, audit append call, context
  attach, ordering markers) lands in `provider_client.rs` — GENERIC. The
  adapter interface stays bytes/strings only; it can never trigger, skip, or
  observe the export gate, the audit, or key bytes.
- Bar unchanged: ZERO needle diffs across `provider-memory`,
  `provider-memory-full`, `quick` (now including all M9C-2 needles).
- **Key risk:** the 1,624-line interleave hides an ordering dependency
  (trust gate → audit → key copy → body write must stay exactly ordered).
- **Verify:** the three profiles byte-identical; review on the diff.
- **Ready-to-scope:** YES after M10B-1.

### M10B-3 — per-provider RAM key slots

As mapped (packet `M10B-3-provider-key-slots` stands). Additions:
- Zeroizing clear; keys never in ANY durable record (grep the export-audit
  emission too); `scan-secrets.ps1` learns `sk-ant-` shapes.
- **Ready-to-scope:** YES after M10B-2 (or after M10B-1 if the adapter split
  slips — key slots only touch provider_config/console/packaging).

### M10C-1 — Anthropic adapter + pins + provider smoke matrix (proof slice)

As mapped (packet `M10C-1-anthropic-adapter` stands) plus:
- OD-1 answered first; R1 (leaf key alg) already pre-checked in M10-0 — if it
  fired, this slice is BLOCKED pending owner sign-off on verifier widening
  (its own focused slice; trust-surface extension).
- New: assert the M9C-2 export denial chain fires identically for
  `provider=anthropic` (family `F-export`: export denied
  `provider_trust_not_positive` pinless; with pins but no explicit owner
  export request nothing auto-attaches — automatic context injection remains
  disabled per ADR 0004).
- ONE full profile before claiming M10C (block close per cadence).
- **Ready-to-scope:** after M10B-3 + OD-1.

### M10D-1 — WebPKI groundwork: host-proven core + chain evidence

As mapped (packet `M10D-1-webpki-host-core` stands). Additions:
- The kernel evidence capture (`raios.provider_chain_evidence.v0`, hashes
  only, `validation_authority=none_stage0`) must NOT feed the trust state
  machine — needle-assert the trust state is unchanged by capture.
- Handoff doc target name: `docs/plan-reviews/m11-tls-service-handoff-<date>.md`
  — M11-4/-6 consume it.
- **Ready-to-scope:** parallel to M10C after M10B-2 (host crate + vendored
  patch check are disjoint from adapter work).

## 4. Cross-milestone dependencies

```text
M9C-2 (export gate, pin-only trust)  ──ships BEFORE M10──┐
                                                          v
M10-0 → M10A-1 ∥ (M10B-1 → M10B-2 → M10B-3) → M10C-1 ∥ M10D-1 → M10 close (full)
                                                          │
   M10D-1 handoff ────────────────────────────────────────┴──> M11-4/-6 (chain validation in-guest)
   M10 closed ──────────────────────────────────────────────> M12+ external distribution ADR
```

- M9C-2 ← M10: NO hard dependency (decided in the M9C-2 refined map); M10
  upgrades trust quality behind the same gate.
- M11 assumes M10B-2's provider_client/adapter split exists (its parser
  extraction targets `provider_client.rs`); keep M10 before M11.
- Owner-physical items in M10: NONE structural. Owner-run live smokes (OpenAI
  key image; Anthropic key per OD-1) are routine key handling, not the sealing
  ceremony. The sealing ceremony does not gate M10.

## 5. STOP-tripwires (delta to the original list)

All seven original tripwires stand. Add:
8. Any M10 slice proposing a DURABLE provider-trust record (new write
   boundary — needs its own scoped-evaluator design + owner decision).
9. Any adapter-visible key/context surface in the M10B-2 interface.
10. The M9C-2 export needles changing in ANY M10 slice (reconcile first —
    it means the export boundary moved).
