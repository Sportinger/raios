# M10 Provider Trust Hardening & Adapters — Design Map (2026-07-06)

**Status header.** Authored 2026-07-06 AHEAD of execution as pre-planning. All
file:line claims were verified against HEAD (329e78b) on 2026-07-06 but the
repo will have moved by execution time. Execution preconditions: M6 Promotion
Loop v0 is CLOSED (map: `docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`),
M7 Persistence Foundation, M8 Recovery Lifeline, and M9 Durable Memory maps are
closed per roadmap order, and the full Shadow profile is green (Red Gate Rule).
Note for the orchestrator: M10 has no hard technical dependency on M7-M9 —
keys stay RAM-only and no slice writes to disk — but the default is strict
roadmap order; resequencing M10 earlier is an owner call, not yours.
**MANDATORY Slice 0 = map revalidation**: re-check every file:line claim in
this map against HEAD, update the map first where reality diverged, and commit
the map update BEFORE starting any implementation slice.

**Milestone capability sentence.** After M10 the system can talk to more than
one AI provider (OpenAI + Anthropic) through one typed, descriptor-driven
provider contract with per-provider SPKI-pinned trust, per-provider RAM-only
keys, an honest owner-attested time authority recorded as evidence, and a
host-proven WebPKI chain-validation core staged for the M11 TLS-service
extraction — all without weakening a single existing denial.

## Baseline (verified 2026-07-06)

| Area | Exists today | Gap for M10 |
|---|---|---|
| Trust states | `seed-kernel/src/provider_trust.rs:28` TrustState (9 states incl. `WebPkiVerified`, which is defined but unreachable — no code path sets it) | Everything is OpenAI-singular: one `RuntimeTrust` static at `:10`, pins from `RAIOS_OPENAI_*` env at `:3-6`, verifier metadata const `OPENAI_PINNED_TLS_VERIFIER_METADATA` at `:88` with host hardcoded `api.openai.com`, `chain_policy: "pin_only_no_webpki_chain_validation"` (`:97`), `time_policy: "not_validated_stage0"` (`:98`) |
| Pin verifier | `seed-kernel/src/openai_trust.rs:14` `OpenAiPinnedCertVerifier` implements embedded-tls `TlsVerifier<Aes128GcmSha256>` (`:40`); SPKI/leaf SHA-256 pin match + TLS1.3 CertificateVerify checked with **ECDSA P-256/SHA-256 only** (`extract_p256_spki` `:213`); non-P-256 leaf → `pin_verifier_unavailable` | Verifier is host-fixed and algorithm-narrow; a second provider needs a parameterized verifier and MAY need more CertificateVerify algorithms (see risk R1) |
| Provider client | `seed-kernel/src/openai.rs` (1,624 lines): consts `:21-24` (host from verifier metadata, path `/v1/responses`, model `gpt-5.4`), `submit_request` `:185`, request envelope + binding/export-audit hashing `:417/:814/:884`, HTTP build `:1286/:1296`, JSON extract `:1435`, `KernelRng` `:1599` | Generic gating machinery (envelope hash, trust gate, binding emission, injection gate) is interleaved with OpenAI wire format in one file; no internal completion contract |
| Keys | `seed-kernel/src/provider_config.rs` (102 lines): ONE key slot, `provider_name` hardcoded `"OPENAI"` (`:36`), 256-byte capacity, `copy_api_key` `:50`; SET-mode intake + `-EmbedOpenAiApiKeyFromEnv` per `docs/SECRETS.md` | No per-provider slots, no explicit clear op, secret scanner knows only OpenAI key shapes |
| Time | Nothing. `now_ms()` in openai.rs `:412` is boot-relative; verifier metadata honestly says `not_validated_stage0` | No wall time at all — certificates cannot be validity-checked, evidence has no honest wall-clock anchor |
| Harness | `vm-harness/openai-direct-smoke.ps1` (modes `-ExpectProviderResponse/-ExpectPinnedTrust/-ExpectSpkiPinnedTrust/-ExpectPinMismatch`, lines 6-9); `vm-harness/shadow-vm-smoke.ps1:12` profile ValidateSet incl. `provider-memory`, `provider-memory-full` | Smoke is OpenAI-only; positive trust marker format `openai: TLS provider trust verified: pinned_spki sha256:<pin-id>` is provider-prefixed already (good — generalizes) |
| TLS vendoring | `vendor/embedded-tls-0.17.0` carries a narrow verifier-input patch (leaf cert DER + CertificateVerify bytes exposed) per `device-protocol/provider-trust-v0.md` | Patch exposes the LEAF only; intermediate certificates of the chain are (assumed) not retained — verify at execution time |
| Contract doc | `device-protocol/provider-trust-v0.md`: trust-state table, acceptance criteria, explicit rule "WebPKI path only after anchors, time, hostname, chain handling are specified and tested" | v0 doc is OpenAI-worded; needs a provider-generic revision |

## Decision D1 — where does WebPKI chain validation land?

Two orders were analyzed:

- **Order A: chain validation in the kernel now (M10), extract to Wasm later
  (M11).** Requires vendoring `rustls-webpki` + `rustls-pki-types` +
  `webpki-roots` + RustCrypto signature verifiers (`p256`, `p384`, `rsa`) into
  the no_std kernel on pinned nightly-2024-10-15. Feasibility is real but not
  cheap: rustls-webpki ≥0.103 accepts pluggable `SignatureVerificationAlgorithm`
  impls, so ring is avoidable, but RSA verify + P-384 are new crypto surface,
  and X.509 path building is exactly the kind of internet-facing parser the
  project has already declared should NOT live in ring 0 (the standing
  "acknowledged violation" note). Doing it in-kernel deepens the violation M11
  exists to remove, and the work is then redone when TLS moves into a Wasm
  service.
- **Order B: M10 does the groundwork (host-proven validation core, real chain
  evidence capture, time authority); the in-path chain validation lands WITH
  M11's TLS-as-a-service extraction**, where a chain-parsing bug is contained
  by fuel-metered Wasm isolation instead of being a ring-0 RCE.

**RECOMMENDATION: Order B — firm, not marked OWNER DECISION** because the
tradeoff is not close: Order A duplicates work, grows the ring-0 attack
surface against ADR 0005 direction, and buys only an earlier `webpki_verified`
label. Cost of B, stated honestly: `webpki_verified` stays unreachable until
M11, and pin rotation remains an operational burden (mitigated by the existing
`RAIOS_OPENAI_SPKI_SHA256_NEXT` rotation-window mechanism, generalized per
provider in M10B). The owner may override; if so, stop and re-plan M10D as a
kernel vendoring milestone.

## Decision D2 — trusted time v0

Certificates need wall time; today there is none. Options considered:
(a) owner-attested time via SET mode + TSC monotonic drift bounds;
(b) authenticated network time (NTS/Roughtime) — new protocol + crypto in the
kernel, circular with TLS, too heavy for v0;
(c) TLS/HTTP-derived bounds (leaf notBefore, Date header of a pin-verified
response) — usable only as cross-check, circular as primary authority.

**DECIDED: (a) as the v0 authority, with two honest supporting inputs.**
1. CMOS RTC read (ports 0x70/0x71) recorded as `time_authority="rtc_unattested"`
   — real hardware input, NOT authority (owner-settable, unauthenticated).
2. Owner attests wall time in SET mode (same trusted intake surface as the API
   key); kernel binds it to the current TSC reading; authority becomes
   `owner_attested_current_boot`. RAM-only, lost on reboot, labeled
   `current_boot` — no fake persistence.
3. Date header from a pin-verified provider response recorded as
   `provider_response_header_cross_check` evidence only, never authority.
Fail-closed rule: with no attestation, every wall-time-dependent claim is an
explicit denial (`capability_denied` reason `wall_time_unattested`), and
`time_policy` in verifier metadata stays `not_validated_stage0` — M10 does NOT
start claiming certificate-validity checks. Stronger time (authenticated
network time) is explicitly LATER work, post-M11, in the network service.

## Decision D3 — adapter and registry shape

- Internal typed contract (Rust types, host-testable in raios-core):
  `ProviderCompletionRequest { provider_id, model, prompt, max_output_tokens }`
  → `ProviderCompletionResponse { output_text, finish, http_status }`. New
  schemas (`raios.provider_descriptor.v0`,
  `raios.provider_completion_request.v0` / `_response.v0`,
  `raios.time_authority.v0`, `raios.provider_chain_evidence.v0`,
  `raios.provider_chain_validation.v0`) are **record-model entries in
  raios-core only** — mechanism-before-vocabulary, no hand-rolled emit/hash.
- Provider registry is descriptor-driven, NOT hardcoded: a
  `ProviderDescriptor` binds `provider_id`, host, port, path, transport,
  pin-env names, auth header scheme id (`bearer` vs `x-api-key+version`), and
  wire-format id (`openai_responses_v1` / `anthropic_messages_v1`). The wire
  builders/parsers remain code selected by format id — honest: descriptors
  configure identity/endpoint/trust, they do not interpret arbitrary formats.
- Trust becomes per-provider: `RuntimeTrust` singleton → fixed-size table
  keyed by provider_id; verifier metadata derives from the descriptor; the
  positive marker keeps its existing shape with the provider prefix
  (`anthropic: TLS provider trust verified: pinned_spki sha256:<id>`), and the
  OpenAI markers stay BYTE-IDENTICAL (needle regression guard).

## Decision D4 — key lifecycle

Keys stay RAM-only and SET-provisioned (or env-embedded into ignored local
images per `docs/SECRETS.md`). M10 adds per-provider slots, an explicit clear
operation that zeroizes the slot, and `api_key_set` per provider in snapshots
(never key material). **Sealed durable secrets are OUT of scope**: even after
M7 SEED_DATA exists, keys MUST NOT be written to it — durable secrets are
blocked on a future hardware-backed sealing design (TPM on the bonded Surface
Pro 4 is unexplored) which requires its own ADR. Classification `secret`,
provider export denied, always.

## OWNER DECISION OD-1 — second-provider positive smoke

Proving the Anthropic adapter live needs an Anthropic API key and spends real
tokens. Options: (a) owner provides a key; positive smoke runs locally like
today's OpenAI smoke, default model `claude-opus-4-8`, or `claude-haiku-4-5`
if the owner prefers the cheapest smoke ($1/$5 per MTok) — model id is a
descriptor field either way, never hardcoded in the parser; (b) fail-closed
verification only (pins + denial paths, no live response) — weaker, the
adapter is then not proven end-to-end; (c) different second provider.
**Recommendation: (a) with `claude-haiku-4-5` for routine smokes** (a smoke
needs 128 output tokens, capability tier is irrelevant). Ask the owner before
starting M10C.

## Sub-milestones

- **M10A Trusted Time v0** — Slice 1. Independent of everything else.
- **M10B Provider-Agnostic Core** — Slices 2-4 (descriptor + per-provider
  trust; adapter extraction; per-provider key slots). Order: 2 → 3 → 4.
- **M10C Second Provider (Anthropic)** — Slice 5. Needs M10B complete + OD-1.
- **M10D Chain-Validation Groundwork** — Slice 6. Host-side; can run parallel
  to M10C after Slice 2. Ends with a written M11 handoff.

## Slice 0 — map revalidation (MANDATORY, first)

Re-verify against HEAD: every file:line in the Baseline table; the profile
ValidateSet in `vm-harness/shadow-vm-smoke.ps1`; whether M7-M9 execution moved
provider files; whether `vendor/embedded-tls-0.17.0` still carries the
verifier-input patch unchanged; whether any provider source is in the attested
source SET of `seed-kernel/build.rs` (grep the ordered list — if yes, every
kernel slice below needs the `target/descriptor-resign` step). Update this map,
commit (`M10-0: map revalidation`), only then start Slice 1.

---

## Slice 1 (M10A-1) — owner-attested time authority v0

**Capability.** The owner can attest wall time once per boot in SET mode; the
system can then answer "what time is it and why do you believe that" with a
typed, honestly-labeled `raios.time_authority.v0` record (rtc_unattested /
owner_attested_current_boot / cross-check), and every wall-time-dependent
capability stays explicitly denied until attestation.

**Files (verify at execution time).** New `seed-kernel/src/time_authority.rs`
(RTC read, TSC binding, state); `seed-kernel/src/console.rs` (SET-mode command
— match the existing `set key` grammar, verify exact grammar in console.rs
first); `seed-kernel/src/agent_protocol_system.rs` or a new small module for
read-only `time.status`; `raios-core/src/` record-model entry + host tests;
event-log constructor via the existing typed-event lattice pattern
(`seed-kernel/src/event_log.rs`).

**Verification.** Host: `cargo test --locked -p raios-core` (record hash
vectors). VM: focused `provider-memory` profile is the wrong boundary; use
quick profile PLUS new needles:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick`
New needles (added to the quick profile): `time: rtc_unattested` (RTC read
line), `time: wall time unattested - denied` (denial before attestation),
`time: owner attested wall time accepted` + `time_authority=owner_attested_current_boot`
after the harness sends the SET command over serial, and `time.status` typed
output. QEMU `-rtc base=utc` makes the RTC needle deterministic-ish.

**Fail-closed.** No attestation → denial with reason `wall_time_unattested`;
attestation never upgrades any provider trust state; `time_policy` stays
`not_validated_stage0`; attested time is `current_boot` and must never be
persisted by later milestones without provenance.

**STOP-tripwires.** If implementing requires touching provider trust states or
verifier metadata semantics → stop (trust-model change). If RTC access is
flaky under QEMU and the worker proposes faking the value → stop.

```text
Packet id: M10A-1-time-authority-v0
Goal: Owner-attested wall-time authority v0: RTC read (rtc_unattested), SET-mode
  attestation bound to TSC (owner_attested_current_boot), read-only time.status,
  typed raios.time_authority.v0 record, fail-closed denial when unattested.
Read first: docs/ROADMAP.md; docs/PROJECT_STATUS.md; this map (Slice 1 + D2);
  seed-kernel/src/console.rs (SET-mode command grammar); seed-kernel/src/event_log.rs
  (typed event constructor pattern, &'static str lattices); raios-core/src (record
  model Value/Field + serializer/hasher); seed-kernel/src/provider_trust.rs (do NOT
  modify).
Allowed write set: seed-kernel/src/time_authority.rs (new); seed-kernel/src/console.rs;
  seed-kernel/src/main.rs (module wiring only); seed-kernel/src/event_log.rs (one
  constructor + vocabulary entries); ONE agent-protocol dispatch site for read-only
  time.status; raios-core/src/** (record-model entry + tests); vm-harness/
  shadow-vm-smoke-profile-quick.ps1 (new needles only); docs/PROJECT_STATUS.md.
Forbidden: provider_trust.rs, openai_trust.rs, openai.rs, provider_config.rs; any
  persistence; any new hand-rolled raios.*.v0 emit/hash code outside the record
  model; any positive capability gated on time; NTP/network time of any kind.
Constraints: RAM-only, label current_boot; attestation accepted at most once per
  boot (re-attest = explicit superseding event, old value retained as evidence);
  ISO-8601 UTC input, reject anything else; TSC offset captured atomically with
  acceptance; no floating point in kernel path; cargo fmt --all -- --check clean.
  If build fails with attested-source mismatch, run the target/descriptor-resign
  flow and say so in the report.
Definition of done: quick profile green including the 4 new time needles (report
  filename cited); host tests green; capability sentence in commit message:
  "The owner can attest wall time and the system reports an honest time authority."
Report format: files touched + line counts; exact new needle strings; quick-profile
  report filename + result; host test count; any deviation from this packet.
```

---

## Slice 2 (M10B-1) — provider descriptors + per-provider trust state

**Capability.** The system can represent N providers as typed descriptors and
track TLS trust per provider (state table, per-provider pins with rotation
slot, per-descriptor verifier metadata) — with OpenAI's serial output
byte-identical to before.

**Files (verify at execution time).** `seed-kernel/src/provider_trust.rs`
(singleton → table keyed by provider_id; metadata derived from descriptor);
new `seed-kernel/src/provider_descriptor.rs` (OpenAI + later Anthropic
descriptors as consts; env-pin names per provider); `raios-core` record-model
entry `raios.provider_descriptor.v0` + host tests; `openai_trust.rs` renamed
conceptually to a parameterized `provider_pin_verifier` (host + pins as
parameters, still P-256-only — algorithm widening is Slice 5's problem if it
bites, see R1). Snapshot render sites that print trust (`system.snapshot.v0`
emitters — grep for `provider_trust::snapshot`).

**Verification.** This touches the provider-trust boundary → focused profile
mandatory: `...\shadow-vm-smoke.ps1 -Profile provider-memory` and
`-Profile quick`. Evidence: ALL existing provider needles pass unchanged
(byte-identical discipline — workers' self-reported diffs are not trusted;
the needles are ground truth), plus new needles: descriptor listing line
`provider descriptor: provider=openai host=api.openai.com pin_policy=...` and
per-provider trust state line for a configured-but-unattempted second
descriptor showing `pin_config_missing` (fail-closed proof that a registered
provider without pins is denied).

**Fail-closed.** A descriptor without a syntactically valid pin is
`pin_config_missing` and can never reach a positive state; the dev bypass
`RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS` stays OpenAI-scoped and MUST NOT
generalize — new providers get NO bypass flag at all.

**STOP-tripwires.** Any change to the meaning of an existing trust state or to
`allows_provider_request` semantics → stop. Any proposal to add a generic
"allow unverified provider" switch → stop.

```text
Packet id: M10B-1-provider-descriptor-trust
Goal: Replace the OpenAI-singular trust singleton with a descriptor-driven
  per-provider trust table; add raios.provider_descriptor.v0 as a record-model
  entry; keep every existing OpenAI serial marker byte-identical.
Read first: this map (Slice 2 + D3); seed-kernel/src/provider_trust.rs (all);
  seed-kernel/src/openai_trust.rs (all); device-protocol/provider-trust-v0.md;
  seed-kernel/src/openai.rs:1-120 and every call into provider_trust; raios-core
  record model.
Allowed write set: seed-kernel/src/provider_trust.rs; seed-kernel/src/
  provider_descriptor.rs (new); seed-kernel/src/openai_trust.rs (parameterize host
  + pins; keep P-256-only verifier logic identical); minimal call-site updates in
  openai.rs (no wire-format changes); snapshot/render sites that print trust;
  raios-core/src/**; vm-harness profile needle additions; device-protocol/
  provider-trust-v0.md (generalize wording, keep every acceptance criterion);
  docs/PROJECT_STATUS.md.
Forbidden: provider_config.rs; any Anthropic network code; any new trust state;
  any change to pin match/CertificateVerify verification logic; any relaxation of
  fail-closed paths; touching wasm_runtime/echo/hello/recovery surfaces.
Constraints: descriptors are &'static consts (no alloc at trust-decision time);
  per-provider rotation pin slot preserved (RAIOS_<PROVIDER>_SPKI_SHA256_NEXT);
  dev bypass stays OpenAI-only; existing openai needles byte-identical; fmt clean;
  re-sign via target/descriptor-resign if attested sources complain.
Definition of done: provider-memory + quick profiles green with zero changed
  legacy needles and the 2 new descriptor needles; host tests green; capability
  sentence: "The system tracks TLS trust per provider from typed descriptors."
Report format: files + line counts; needle diff summary (must be additions only);
  both report filenames + results; any call site you had to touch beyond the list.
```

---

## Slice 3 (M10B-2) — typed completion contract + adapter extraction

**Capability.** The system routes AI requests through one internal typed
completion contract; OpenAI is the first adapter behind it, and the generic
trust-gate/envelope-hash/binding/injection-gate machinery is provider-neutral
code that a second adapter can reuse without copying.

**Files (verify at execution time).** Split `seed-kernel/src/openai.rs`
(1,624 lines) into: `provider_client.rs` (generic: submit/poll state machine,
envelope + binding + export-audit hashing, injection gate emission, TLS/TCP
driving, response buffering) and `openai_adapter.rs` (wire format only:
header build `:1286`, body build `:1296`, `extract_output_text` `:1435`).
`raios-core` gets the completion request/response record-model entries with
host-tested hash vectors. `ask <text>` console path re-pointed at the
contract.

**Verification.** Refactor of the provider boundary → focused
`provider-memory` AND `provider-memory-full` profiles + quick:
`...\shadow-vm-smoke.ps1 -Profile provider-memory-full`. Evidence bar: ZERO
needle changes — this slice is done only when serial output is byte-identical
(M2-collapse discipline; the harness catches dropped fields, worker claims do
not count). Host: raios-core tests for the new records. A local positive smoke
`vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` if a local key
image exists (optional, note in report if skipped).

**Fail-closed.** All existing provider gates (trust gate before key copy,
envelope-hash validation, context-export denial chain, injection-gate blocked
marker) must remain in the generic layer — an adapter physically cannot skip
them because it never sees the key or the socket.

**STOP-tripwires.** Any needle change at all → stop and reconcile before
proceeding (it means the refactor altered behavior). Net line growth > +300
→ stop (the split should be near-neutral).

```text
Packet id: M10B-2-adapter-extraction
Goal: Split openai.rs into provider-neutral provider_client.rs (gating, hashing,
  bindings, transport driving) and openai_adapter.rs (wire format), behind typed
  ProviderCompletionRequest/Response record-model entries. Serial output must be
  byte-identical.
Read first: this map (Slice 3 + D3); seed-kernel/src/openai.rs IN FULL;
  seed-kernel/src/provider_trust.rs + provider_descriptor.rs (post Slice 2);
  seed-kernel/src/net.rs public API; raios-core record model;
  docs/plan-reviews/m2-collapse-map-2026-07-05.md (byte-identical discipline).
Allowed write set: seed-kernel/src/openai.rs (shrink/delete); seed-kernel/src/
  provider_client.rs (new); seed-kernel/src/openai_adapter.rs (new); module wiring
  in main.rs; raios-core/src/**; docs/PROJECT_STATUS.md.
Forbidden: any behavior change; any needle/marker string change; provider_config.rs;
  vm-harness (no needle edits allowed in this slice — that is the point); any
  Anthropic code; net.rs; TLS/verifier code.
Constraints: adapter interface = build_request_head(key_len)->String,
  build_request_body(req)->String, parse_response(bytes)->Result<Response> — the
  adapter never receives the API key bytes or the socket; key copy and HTTPS write
  stay in provider_client.rs AFTER the trust gate exactly as today; KernelRng and
  timeouts stay generic; fmt clean; descriptor-resign if attestation complains.
Definition of done: provider-memory, provider-memory-full and quick profiles green
  with ZERO needle diffs (cite all three report filenames); host tests green;
  capability sentence: "AI requests flow through one typed provider contract with
  the trust gates enforced provider-neutrally."
Report format: before/after line counts of openai.rs vs new files (net delta);
  confirmation string "zero needle diffs" backed by report filenames; host test
  count; anything that resisted the split.
```

---

## Slice 4 (M10B-3) — per-provider RAM key slots

**Capability.** The owner can provision, inspect (set/unset only), and clear
an API key per provider in SET mode; each key lives in its own zeroizable
RAM slot classified `secret`, and no snapshot or export path can ever carry
key material.

**Files (verify at execution time).** `seed-kernel/src/provider_config.rs`
(single slot → fixed table keyed by provider_id, explicit `clear` that
zeroizes); `console.rs` SET grammar (`set key <provider> ...` — keep the old
OpenAI form working as an alias to avoid breaking the owner's muscle memory);
snapshot emitters showing `api_key_set` per provider;
`scripts/package-stage0.ps1` + `scripts/write-stage0-usb.ps1` gain
`-EmbedAnthropicApiKeyFromEnv`/pin flags mirroring the OpenAI ones (env
`RAIOS_DEFAULT_ANTHROPIC_API_KEY`, `ANTHROPIC_SPKI_SHA256`);
`scripts/scan-secrets.ps1` learns Anthropic key patterns (`sk-ant-`);
`docs/SECRETS.md` updated.

**Verification.** Trust/authority-adjacent → focused `provider-memory` +
quick. New needles: `provider key set: provider=anthropic accepted len_ok=true`
(value never echoed), `provider key cleared: provider=anthropic zeroized=true`,
per-provider `api_key_set` in the snapshot. Then
`powershell ...\scripts\scan-secrets.ps1` must be run and pass. Negative
harness check: grep the serial log for the test key value — zero hits.

**Fail-closed.** Key copy still happens only after that provider's trust is
positive (unchanged gate, now per-provider); a key for provider A can never be
used for provider B (slot lookup by descriptor id, no fallback); keys never
reach SEED_DATA or any durable record (assert in review; M7+ rule from D4).

**STOP-tripwires.** Any proposal to persist keys or "remember them across
boots" → stop (needs the sealed-secret ADR). Secret scan failing on tracked
files → stop, do not commit.

```text
Packet id: M10B-3-provider-key-slots
Goal: Per-provider RAM-only key slots with SET-mode set/clear, zeroization,
  per-provider api_key_set snapshot fields, packaging env-embed flags for
  Anthropic, and secret-scanner coverage for Anthropic key shapes.
Read first: this map (Slice 4 + D4); seed-kernel/src/provider_config.rs (all);
  seed-kernel/src/console.rs (set key grammar); docs/SECRETS.md;
  scripts/package-stage0.ps1 (key-embed refusal logic); scripts/scan-secrets.ps1.
Allowed write set: seed-kernel/src/provider_config.rs; seed-kernel/src/console.rs;
  snapshot emit sites for api_key_set; scripts/package-stage0.ps1;
  scripts/write-stage0-usb.ps1; scripts/scan-secrets.ps1; docs/SECRETS.md;
  vm-harness quick/provider-memory needle additions; docs/PROJECT_STATUS.md.
Forbidden: writing key material into any record, event, snapshot, log line, or
  disk path; any persistence of keys; provider_client.rs/adapter logic changes;
  weakening the package-script refusal to embed keys into tracked images.
Constraints: fixed-capacity slots (2 for now), zeroize on clear and on
  re-provision; old `set key <value>` grammar keeps working for openai; the
  packaging scripts must refuse Anthropic key embedding into release\esp or the
  default image exactly like OpenAI; fmt clean; descriptor-resign if needed.
Definition of done: provider-memory + quick green incl. new key-slot needles;
  scan-secrets.ps1 clean run pasted; serial-log grep for the smoke key value = 0
  hits; capability sentence: "The owner can hold and clear a separate RAM-only
  key per provider."
Report format: files + line counts; new needle strings; report filenames; secret
  scan output summary; confirmation of the zero-hit key grep.
```

---

## Slice 5 (M10C-1) — Anthropic adapter, pins, and the provider smoke matrix

**Capability.** The system can hold a pinned-SPKI-verified TLS session to
api.anthropic.com and get a real completion back through the Anthropic
Messages API — the second provider through the same contract, gates, and
evidence chain as OpenAI — and the harness can smoke any provider from one
script.

**Wire facts for the worker (verified against Anthropic docs 2026-06 cache;
re-verify shape at execution time with a curl from the host).**
`POST https://api.anthropic.com/v1/messages`, headers
`content-type: application/json`, `x-api-key: <key>`,
`anthropic-version: 2023-06-01`; minimal body
`{"model":"<from descriptor>","max_tokens":128,"messages":[{"role":"user","content":"<prompt>"}]}`;
response JSON: text at `content[] where type=="text" -> text`, plus
`stop_reason`. Model id from descriptor: default `claude-opus-4-8`;
`claude-haiku-4-5` if OD-1 chose the cheap smoke. Exact ids, no date suffixes.

**Files (verify at execution time).** New
`seed-kernel/src/anthropic_adapter.rs` (head/body/parse against the contract);
`provider_descriptor.rs` Anthropic entry (host `api.anthropic.com:443`, path
`/v1/messages`, auth scheme `x_api_key_version`, pins from
`RAIOS_ANTHROPIC_SPKI_SHA256(_NEXT)`); console `ask` gains a provider selector
(default openai, `ask@anthropic <text>` or similar — match existing console
conventions); harness: new `vm-harness/provider-direct-smoke.ps1` with
`-Provider openai|anthropic` superseding `openai-direct-smoke.ps1` (keep the
old script as a thin forwarding shim for one milestone), plus
`vm-harness/provider-smoke-matrix.ps1` running, per provider:
ExpectSpkiPinnedTrust, ExpectPinMismatch (wrong pin), and (key present)
ExpectProviderResponse.

**Verification.** Focused: NEW named profile `provider-adapter` (added to the
ValidateSet in `shadow-vm-smoke.ps1:12` + profile script
`shadow-vm-smoke-profile-provider-adapter.ps1`) covering keyless fail-closed
behavior for BOTH providers: `...\shadow-vm-smoke.ps1 -Profile provider-adapter`.
Live network smokes (local key images, never CI):
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\provider-direct-smoke.ps1 -Provider anthropic -ExpectSpkiPinnedTrust`
then `-ExpectProviderResponse`, then the matrix script. Needle families:
- F-trust: `anthropic: TLS provider trust verified: pinned_spki sha256:<pin>`
  (and the openai one unchanged);
- F-fail: `ANTHROPIC DIRECT TLS PIN MISMATCH` / `... PIN CONFIG MISSING`
  (fail-closed before key copy or body write — mirror of provider-trust-v0
  acceptance criteria);
- F-binding: `raios.provider_request_binding.v0` and export-audit-binding
  markers emitted with `provider=anthropic` on the positive path only;
- F-adapter: typed completion request/response record markers with
  `format=anthropic_messages_v1`.
Before claiming M10C: full profile once
(`...\shadow-vm-smoke.ps1 -Profile full`).

**Fail-closed.** No Anthropic bypass flag exists, period. No pin → no key
copy, no body write. Context export for Anthropic follows the identical gate
chain as OpenAI (denied until positive trust + request binding + export audit
binding). A provider-B response can never be attributed to provider A
(provider_id is bound into the envelope/binding hashes).

**Risk R1 (honest).** The pin verifier checks CertificateVerify with ECDSA
P-256/SHA-256 only. If api.anthropic.com's leaf key is not P-256, the state
is `pin_verifier_unavailable` (fail-closed, correct) and the positive smoke is
impossible without widening the algorithm set (RSA-PSS or P-384) — that is a
trust-surface extension: STOP, report to the orchestrator, get owner sign-off,
and plan it as its own focused slice. Check the live leaf FIRST (host-side
`openssl s_client`) before writing any code.

**STOP-tripwires.** R1 fires; any TLS1.3 handshake incompatibility requiring
an embedded-tls patch beyond verifier inputs; OD-1 unanswered; any urge to
send system context to the new provider (context export stays gated).

```text
Packet id: M10C-1-anthropic-adapter
Goal: Second provider Anthropic (Messages API) behind the typed contract with
  SPKI-pinned trust, per-provider key slot, and a generalized provider smoke
  matrix. OpenAI behavior byte-identical.
Read first: this map (Slice 5 incl. wire facts + R1); seed-kernel/src/
  provider_client.rs + openai_adapter.rs + provider_descriptor.rs +
  provider_trust.rs (post Slices 2-4); vm-harness/openai-direct-smoke.ps1 (all);
  docs/SECRETS.md; device-protocol/provider-trust-v0.md.
Allowed write set: seed-kernel/src/anthropic_adapter.rs (new);
  provider_descriptor.rs (one entry); console ask-routing site;
  vm-harness/provider-direct-smoke.ps1 (new); vm-harness/provider-smoke-matrix.ps1
  (new); vm-harness/openai-direct-smoke.ps1 (forwarding shim only);
  vm-harness/shadow-vm-smoke.ps1 (ValidateSet + dispatch);
  vm-harness/shadow-vm-smoke-profile-provider-adapter.ps1 (new);
  device-protocol/provider-trust-v0.md; docs/PROJECT_STATUS.md; docs/SECRETS.md.
Forbidden: provider_client.rs gate logic changes; verifier algorithm changes
  (STOP per R1 instead); any bypass flag for anthropic; hardcoding model ids in
  the adapter (descriptor field only); committing any key or real pin value;
  sending snapshots/tool schemas/system context to any provider.
Constraints: BEFORE coding, verify api.anthropic.com leaf key type from the host
  (openssl s_client -connect api.anthropic.com:443) and record it in the report;
  if not P-256 ECDSA, STOP and report — do not widen the verifier yourself.
  Response parse must handle content-length and chunked bodies like the OpenAI
  path; max_tokens 128; anthropic-version header exactly 2023-06-01; prompt
  JSON-escaped with the existing push_json_string; fmt clean; secret scan before
  commit; descriptor-resign if attestation complains.
Definition of done: provider-adapter focused profile green (keyless, both
  providers fail closed); provider-direct-smoke -Provider anthropic
  -ExpectSpkiPinnedTrust green and -ExpectProviderResponse green with a local key
  image; -ExpectPinMismatch green (fails closed before key copy); openai smoke
  still green; ONE full profile green before claiming M10C; capability sentence:
  "The system can get a real completion from a second provider through the same
  pinned-trust evidence chain."
Report format: leaf-key-type finding; all report/smoke filenames + results;
  new needle strings by family (F-trust/F-fail/F-binding/F-adapter); confirmation
  openai needles unchanged; secret scan output.
```

---

## Slice 6 (M10D-1) — WebPKI groundwork: host-proven core + chain evidence

**Capability.** The system records the real presented certificate chain
(hashes, bounded) of every provider handshake as typed evidence with
`validation_authority=none_stage0`, and a HOST-tested chain-validation core
(rustls-webpki + RustCrypto verifiers against captured real chains + pinned
Mozilla anchors) proves the exact validation logic and record schema that
M11's TLS service will run in-guest.

**Files (verify at execution time).** New host-side crate
`provider-chain-core/` (workspace member, std, NOT kernel): vendored
rustls-webpki path validation wired to `p256`/`p384`/`rsa` verifiers,
`webpki-roots` anchors, fixture chains captured from api.openai.com and
api.anthropic.com; emits `raios.provider_chain_validation.v0` via the
raios-core record model. Kernel side: retain per-handshake leaf hash +
intermediate cert hashes as `raios.provider_chain_evidence.v0` — REQUIRES the
embedded-tls verifier-input patch to expose intermediate DERs; verify first
whether it already does; if not, extend the vendored patch minimally
(hash-in-place, bounded count ≤6, never store full DERs in the event ring).
`docs/plan-reviews/` gets a short M11 handoff note (what the TLS service must
import, which record it must emit, where the fixtures live).

**Verification.** Host: `cargo test --locked -p provider-chain-core -p raios-core`
(validation green on fresh fixtures; red on a deliberately broken chain, an
expired fixture with a fixed test time, and a wrong-hostname fixture — the
fail-closed cases are the point). Kernel evidence capture touches the TLS
boundary → focused `provider-adapter` profile + quick; new needle:
`provider chain evidence: provider=<id> leaf_sha256=<h> intermediates=<n> validation_authority=none_stage0`.
Fixtures carry a captured-at date; the host test uses a FIXED verification
time (from the fixture metadata), not `SystemTime::now`, so CI stays green
when certs age — honest, since v0 time authority is owner-attested anyway.

**Fail-closed.** Kernel trust decisions are COMPLETELY unchanged — evidence
capture must not feed the trust state machine; `webpki_verified` remains
unreachable; the host core is proof, not authority.

**STOP-tripwires.** Any temptation to call the host-proven core from the
kernel (that is Order A through the back door) → stop. embedded-tls patch
needing to touch handshake/policy logic rather than read-only input exposure
→ stop. Unparking anything (ota/registry/fake-cloud) → forbidden here, needs
a new ADR.

```text
Packet id: M10D-1-webpki-host-core
Goal: Host-tested WebPKI chain-validation core over captured real provider
  chains + in-kernel chain-evidence capture (hashes only, authority=none), plus
  a written M11 handoff. Kernel trust decisions unchanged.
Read first: this map (Slice 6 + D1); device-protocol/provider-trust-v0.md;
  vendor/embedded-tls-0.17.0 verifier-input patch (what it exposes today);
  seed-kernel/src/openai_trust.rs / provider pin verifier (post Slice 2);
  raios-core record model; docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md
  (map style for the handoff note).
Allowed write set: provider-chain-core/** (new host crate + fixtures);
  workspace Cargo.toml member list; raios-core/src/** (2 record-model entries);
  vendor/embedded-tls-0.17.0/** (minimal read-only intermediate-DER exposure ONLY
  if not already present); kernel verifier call site to hash intermediates into
  the evidence record; event_log.rs (one constructor + vocabulary);
  vm-harness/shadow-vm-smoke-profile-provider-adapter.ps1 (one needle);
  docs/plan-reviews/m11-tls-service-handoff-2026-XX-XX.md (new);
  docs/PROJECT_STATUS.md.
Forbidden: calling chain validation from kernel code; changing any trust state
  transition; vendoring webpki/rsa/p384 into seed-kernel; storing full cert DERs
  in the event ring; network fetches at test time (fixtures are checked in);
  fixture files containing private keys (public chains only — they are public,
  but run scan-secrets anyway).
Constraints: host tests must include negative cases (broken chain, expired-at-
  fixed-time, wrong hostname) that FAIL validation; fixed verification time from
  fixture metadata, never now(); fixtures captured via documented openssl
  commands recorded in the crate README; bounded intermediate count (≤6) in the
  kernel record; fmt clean; descriptor-resign if attestation complains.
Definition of done: host tests green incl. all negative cases (count them);
  provider-adapter + quick profiles green with the new evidence needle; handoff
  doc exists and names: import surface, record schemas, fixture path, open
  algorithm questions; capability sentence: "The system records real provider
  certificate chains as evidence and the chain-validation logic for M11 is
  proven on the host."
Report format: host test counts (positive/negative); whether the embedded-tls
  patch needed extending (diff size if so); report filenames; handoff doc path.
```

---

## Global STOP-tripwires (orchestrator: stop and ask the owner)

1. Anything requiring a new ADR: sealed/persistent secrets, unparking
   ota/registry/fake-cloud, network time, kernel WebPKI (Order A override).
2. Trust-model changes: new trust states, verifier algorithm widening (R1),
   bypass flags, any relaxation of a `capability_denied`.
3. Destructive disk operations of any kind, or anything that would overwrite
   `release/raios-stage0.img`.
4. OD-1 (Anthropic key/model/cost) before M10C starts.
5. Any needle regression in Slices 2-3 (byte-identical refactors) that a
   worker proposes to "fix" by editing the needle instead of the code.
6. Secret scan hits, or a real key/pin appearing in any tracked file.
7. Full-profile red at any point → Red Gate Rule, repair only.

## Verdict and budget

M10 is medium-sized and unusually parallelizable: M10A and M10D-host are
independent lanes; the M10B chain (2→3→4) is the critical path; M10C is the
proof slice, like echo was for M5. Honest estimate: 6 implementation slices +
Slice 0, ~4 focused VM runs, live smokes only with local key images, and ONE
full profile before claiming the milestone. The riskiest unknowns are R1
(Anthropic leaf algorithm) and the intermediate-DER exposure in the vendored
embedded-tls patch — both are checked before code is written, both have STOP
paths. The deliberately unbought item is `webpki_verified` itself: M10 makes
it provable; M11 makes it true, outside the kernel, where it belongs.
