# M10 Provider Trust Hardening & Adapters — Design Map (2026-07-06)

**Header.** Authored 2026-07-06 AHEAD of execution as pre-planning; file:line
claims verified against HEAD (329e78b) on 2026-07-06 and will drift.
Execution preconditions: M6 CLOSED (`docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`),
M7/M8/M9 maps closed per roadmap order, full Shadow profile green (Red Gate).
Note: M10 has no hard technical dependency on M7-M9 (keys stay RAM-only,
nothing touches disk); default is strict order, resequencing is an owner call.
**MANDATORY Slice 0 = map revalidation**: re-check every file:line claim
against HEAD, update this map first where reality diverged, commit the map
update BEFORE any implementation slice.

**Milestone capability sentence.** After M10 the system can talk to two AI
providers (OpenAI + Anthropic) through one typed, descriptor-driven provider
contract with per-provider SPKI-pinned trust, per-provider RAM-only keys, an
honest owner-attested time authority recorded as evidence, and a host-proven
WebPKI chain-validation core staged for M11's TLS-service extraction — without
weakening a single existing denial.

## Baseline (verified 2026-07-06)

| Area | Exists today | Gap |
|---|---|---|
| Trust states | `seed-kernel/src/provider_trust.rs:28` TrustState (9 states; `WebPkiVerified` defined but unreachable — no code path sets it) | All OpenAI-singular: one `RuntimeTrust` static (`:10`), pins from `RAIOS_OPENAI_*` env (`:3-6`), `OPENAI_PINNED_TLS_VERIFIER_METADATA` (`:88`) hardcodes `api.openai.com`, `chain_policy="pin_only_no_webpki_chain_validation"` (`:97`), `time_policy="not_validated_stage0"` (`:98`) |
| Pin verifier | `seed-kernel/src/openai_trust.rs:14` `OpenAiPinnedCertVerifier`, embedded-tls `TlsVerifier<Aes128GcmSha256>` (`:40`); SPKI/leaf pin match + TLS1.3 CertificateVerify **ECDSA P-256/SHA-256 only** (`:213`); other algs → `pin_verifier_unavailable` | Host-fixed, algorithm-narrow; second provider needs a parameterized verifier, maybe wider algs (risk R1) |
| Provider client | `seed-kernel/src/openai.rs` (1,624 lines): consts `:21-24` (path `/v1/responses`, model `gpt-5.4`), `submit_request` `:185`, envelope+binding+export-audit hashing `:417/:814/:884`, HTTP build `:1286/:1296`, JSON extract `:1435`, `KernelRng` `:1599` | Generic gate/hash/binding machinery interleaved with OpenAI wire format; no internal completion contract |
| Keys | `seed-kernel/src/provider_config.rs` (102 lines): ONE slot, `provider_name="OPENAI"` (`:36`), `copy_api_key` `:50`; SET-mode intake + `-EmbedOpenAiApiKeyFromEnv` (`docs/SECRETS.md`) | No per-provider slots, no explicit zeroizing clear, scanner knows only OpenAI key shapes |
| Time | Nothing; `now_ms()` (openai.rs `:412`) is boot-relative | No wall time; no honest time authority for evidence or (later) cert validity |
| Harness | `vm-harness/openai-direct-smoke.ps1` modes `-ExpectProviderResponse/-ExpectPinnedTrust/-ExpectSpkiPinnedTrust/-ExpectPinMismatch` (lines 6-9); `shadow-vm-smoke.ps1:12` ValidateSet incl. `provider-memory`, `provider-memory-full` | OpenAI-only; positive marker `openai: TLS provider trust verified: pinned_spki sha256:<pin-id>` is already provider-prefixed (generalizes cleanly) |
| TLS vendoring | `vendor/embedded-tls-0.17.0` narrow verifier-input patch: leaf DER + CertificateVerify bytes exposed (`device-protocol/provider-trust-v0.md`) | Intermediates presumably NOT exposed — verify at execution time |
| Contract doc | `device-protocol/provider-trust-v0.md`: trust-state table, acceptance criteria, rule "WebPKI only after anchors, time, hostname, chain handling are specified and tested" | OpenAI-worded; needs provider-generic revision |

## Decision D1 — where WebPKI chain validation lands

- **Order A: in-kernel now (M10), extract in M11.** Feasible: rustls-webpki
  ≥0.103 takes pluggable `SignatureVerificationAlgorithm` impls, so ring is
  avoidable via RustCrypto (`p256`/`p384`/`rsa`) and `webpki-roots` is plain
  data. But it vendors an X.509 path builder + RSA verify into no_std ring 0
  on pinned nightly-2024-10-15 — exactly the internet-facing parsing the
  standing "kernel does not parse the internet" note says must leave the
  kernel — and the work is redone when M11 moves TLS into a Wasm service.
- **Order B: M10 does groundwork (host-proven validation core, real chain
  evidence capture, time authority); in-path chain validation ships WITH the
  M11 TLS-service extraction**, where a chain-parser bug is contained by
  fuel-metered Wasm instead of being ring-0 RCE.

**RECOMMENDATION: Order B — firm, not marked OWNER DECISION** (not close:
A duplicates work and grows ring-0 attack surface against ADR 0005 for an
earlier `webpki_verified` label only). Honest cost of B: `webpki_verified`
stays unreachable until M11; pin rotation stays an operational burden
(mitigated by the existing `_NEXT` rotation-window slot, generalized per
provider). Owner may override; then M10D becomes a kernel vendoring slice —
stop and re-plan.

## Decision D2 — trusted time v0

Options: (a) owner-attested time via SET + TSC drift bounds; (b) authenticated
network time (NTS/Roughtime) — new protocol+crypto in kernel, circular with
TLS, too heavy; (c) TLS/HTTP-derived bounds (leaf notBefore, Date header) —
cross-check only, circular as authority. **DECIDED: (a)** with two honest
supporting inputs:
1. CMOS RTC read (ports 0x70/0x71) → `time_authority="rtc_unattested"` (real
   input, NOT authority: owner-settable, unauthenticated).
2. Owner attests wall time in SET mode (same trusted intake as API key);
   bound to current TSC; authority `owner_attested_current_boot`. RAM-only,
   `current_boot`, lost on reboot — no fake persistence.
3. Date header from a pin-verified response recorded as
   `provider_response_header_cross_check` evidence, never authority.
Fail-closed: unattested → `capability_denied` reason `wall_time_unattested`;
verifier `time_policy` STAYS `not_validated_stage0` — M10 does not claim cert
validity checks. Authenticated network time = later work, post-M11, in the
network service.

## Decision D3 — adapter and registry shape

- Internal typed contract, host-testable: `ProviderCompletionRequest
  { provider_id, model, prompt, max_output_tokens }` →
  `ProviderCompletionResponse { output_text, finish, http_status }`. All new
  schemas (`raios.provider_descriptor.v0`, `raios.provider_completion_request
  /.response.v0`, `raios.time_authority.v0`, `raios.provider_chain_evidence
  .v0`, `raios.provider_chain_validation.v0`) are **record-model entries in
  raios-core only** — mechanism-before-vocabulary.
- Registry is descriptor-driven, not hardcoded: `ProviderDescriptor` binds
  provider_id, host, port, path, transport, pin-env names, auth scheme id
  (`bearer` vs `x_api_key_version`), wire-format id (`openai_responses_v1` /
  `anthropic_messages_v1`). Wire builders/parsers remain code selected by
  format id — descriptors configure identity/endpoint/trust, honestly.
- Trust becomes per-provider: singleton → fixed table keyed by provider_id;
  verifier metadata derives from descriptor; OpenAI markers stay
  BYTE-IDENTICAL (needle regression guard), Anthropic gets
  `anthropic: TLS provider trust verified: pinned_spki sha256:<id>`.

## Decision D4 — key lifecycle

RAM-only, SET-provisioned (or env-embedded into git-ignored local images per
`docs/SECRETS.md`). M10 adds per-provider slots, zeroizing clear, per-provider
`api_key_set` in snapshots (never material). **Sealed durable secrets OUT of
scope**: even after M7, keys MUST NOT touch SEED_DATA — blocked on a future
hardware-backed sealing design (Surface Pro 4 TPM unexplored) needing its own
ADR. Classification `secret`, provider export denied, always.

## OWNER DECISION OD-1 — second-provider positive smoke

Live Anthropic smoke needs a real key and spends tokens. Options: (a) owner
provides a key; positive smoke runs locally like the OpenAI one; model from
descriptor, default `claude-opus-4-8`, or `claude-haiku-4-5` for the cheapest
smoke ($1/$5 per MTok; a smoke needs 128 output tokens); (b) fail-closed-only
verification (pins + denials, no live response) — weaker, adapter unproven
end-to-end; (c) different second provider. **Recommendation: (a) with
`claude-haiku-4-5` for routine smokes.** Exact ids, no date suffixes. Ask
before starting M10C.

## Sub-milestones

- **M10A Trusted Time v0** — Slice 1 (independent lane).
- **M10B Provider-Agnostic Core** — Slices 2→3→4 (critical path).
- **M10C Second Provider: Anthropic** — Slice 5 (needs M10B + OD-1).
- **M10D Chain-Validation Groundwork** — Slice 6 (host-side; parallel to
  M10C after Slice 2; ends with a written M11 handoff).

## Slice 0 — map revalidation (MANDATORY, first)

Re-verify against HEAD: every file:line in the Baseline table; the profile
ValidateSet; whether M7-M9 execution moved provider files; whether the
embedded-tls verifier-input patch is unchanged; whether any provider source is
in `seed-kernel/build.rs`'s attested source SET (if yes, every kernel slice
needs the `target/descriptor-resign` step). Update map, commit
(`M10-0: map revalidation`), then start.

---

## Slice 1 (M10A-1) — owner-attested time authority v0

**Capability.** The owner can attest wall time once per boot in SET mode; the
system then answers "what time is it and why do you believe that" with a typed
`raios.time_authority.v0` record; every wall-time-dependent claim stays denied
until attestation.
**Files (verify at execution).** New `seed-kernel/src/time_authority.rs`;
`console.rs` SET command (verify grammar first); read-only `time.status`;
raios-core record entry + host tests; one typed event constructor.
**Verification.** Host: `cargo test --locked -p raios-core`. VM:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick`
with 4 new needles: `time: rtc_unattested`, `time: wall time unattested - denied`,
`time: owner attested wall time accepted` + `time_authority=owner_attested_current_boot`
(harness sends SET over serial), typed `time.status` output. QEMU `-rtc base=utc`.
**Fail-closed.** Unattested → denial `wall_time_unattested`; attestation never
upgrades any trust state; `time_policy` stays `not_validated_stage0`; value is
`current_boot`, never persisted.
**STOP-tripwires.** Touching trust states/verifier metadata; any proposal to
fake the RTC value if QEMU is flaky.

```text
Packet id: M10A-1-time-authority-v0
Goal: Owner-attested wall-time authority v0: RTC read (rtc_unattested), SET-mode
  attestation bound to TSC (owner_attested_current_boot), read-only time.status,
  typed raios.time_authority.v0 record, fail-closed denial when unattested.
Read first: docs/ROADMAP.md; docs/PROJECT_STATUS.md; this map Slice 1 + D2;
  seed-kernel/src/console.rs (SET grammar); seed-kernel/src/event_log.rs (typed
  event constructors, &'static str lattices); raios-core record model;
  seed-kernel/src/provider_trust.rs (context only — do NOT modify).
Allowed write set: seed-kernel/src/time_authority.rs (new); console.rs; main.rs
  (wiring only); event_log.rs (one constructor + vocabulary); ONE agent-protocol
  dispatch site for time.status; raios-core/src/**; vm-harness/
  shadow-vm-smoke-profile-quick.ps1 (needle additions only); docs/PROJECT_STATUS.md.
Forbidden: provider_trust.rs/openai_trust.rs/openai.rs/provider_config.rs; any
  persistence; hand-rolled raios.*.v0 emit/hash outside the record model; any
  positive capability gated on time; network time of any kind.
Constraints: RAM-only, current_boot labeled; attestation once per boot (re-attest
  = explicit superseding event, old value retained); ISO-8601 UTC input only; TSC
  offset captured atomically with acceptance; no float in kernel path;
  cargo fmt --all -- --check clean; run target/descriptor-resign if the build
  fails on attested-source mismatch and say so.
Definition of done: quick profile green incl. the 4 new time needles (cite report
  filename); host tests green; commit capability sentence: "The owner can attest
  wall time and the system reports an honest time authority."
Report format: files + line counts; exact new needle strings; quick report
  filename + result; host test count; deviations.
```

---

## Slice 2 (M10B-1) — provider descriptors + per-provider trust state

**Capability.** The system represents N providers as typed descriptors and
tracks TLS trust per provider (state table, per-provider pins with rotation
slot, per-descriptor verifier metadata) — OpenAI serial output byte-identical.
**Files (verify at execution).** `provider_trust.rs` (singleton → table keyed
by provider_id; metadata derived from descriptor); new `provider_descriptor.rs`
(OpenAI const now, Anthropic in Slice 5); `openai_trust.rs` parameterized into
a generic pin verifier (host+pins as parameters, logic identical, still
P-256-only); raios-core `raios.provider_descriptor.v0` + tests; snapshot
render sites (grep `provider_trust::snapshot`).
**Verification.** Trust boundary → focused mandatory:
`...\shadow-vm-smoke.ps1 -Profile provider-memory` and `-Profile quick`. ALL
existing provider needles unchanged (needles are ground truth, not worker
claims — M2 Batch 4 lesson); new needles: `provider descriptor:
provider=openai host=api.openai.com ...` and a second registered descriptor
showing `pin_config_missing` (fail-closed proof for a pinless provider).
**Fail-closed.** No valid pin = `pin_config_missing`, never positive. Dev
bypass `RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS` stays OpenAI-scoped; new providers
get NO bypass flag.
**STOP-tripwires.** Changing any existing trust-state meaning or
`allows_provider_request` semantics; any generic unverified-provider switch.

```text
Packet id: M10B-1-provider-descriptor-trust
Goal: Descriptor-driven per-provider trust table replacing the OpenAI singleton;
  raios.provider_descriptor.v0 record-model entry; every existing OpenAI serial
  marker byte-identical.
Read first: this map Slice 2 + D3; seed-kernel/src/provider_trust.rs (all);
  seed-kernel/src/openai_trust.rs (all); device-protocol/provider-trust-v0.md;
  every provider_trust call site in openai.rs; raios-core record model.
Allowed write set: provider_trust.rs; provider_descriptor.rs (new);
  openai_trust.rs (parameterize host+pins only; verifier logic identical);
  minimal call-site updates in openai.rs (no wire changes); trust snapshot render
  sites; raios-core/src/**; vm-harness needle additions;
  device-protocol/provider-trust-v0.md (generalize wording, keep all acceptance
  criteria); docs/PROJECT_STATUS.md.
Forbidden: provider_config.rs; Anthropic network code; new trust states; changes
  to pin-match/CertificateVerify logic; relaxing any fail-closed path; wasm/echo/
  hello/recovery surfaces.
Constraints: descriptors are &'static consts (no alloc at trust-decision time);
  keep the RAIOS_<PROVIDER>_SPKI_SHA256_NEXT rotation slot per provider; dev
  bypass stays OpenAI-only; fmt clean; descriptor-resign if attestation complains.
Definition of done: provider-memory + quick green, legacy needles unchanged
  (additions only), 2 new descriptor needles pass; host tests green; capability
  sentence: "The system tracks TLS trust per provider from typed descriptors."
Report format: files + line counts; needle diff summary (must be additions only);
  both report filenames + results; extra call sites touched beyond the list.
```

---

## Slice 3 (M10B-2) — typed completion contract + adapter extraction

**Capability.** AI requests flow through one internal typed completion
contract; OpenAI becomes the first adapter behind it; the trust-gate/
envelope-hash/binding/injection-gate machinery is provider-neutral code a
second adapter reuses without copying.

**Files (verify at execution).** Split `openai.rs` (1,624 lines) into
`provider_client.rs` (generic: submit/poll state machine, envelope + binding +
export-audit hashing, injection-gate emission, TLS/TCP driving, response
buffering, KernelRng, timeouts) and `openai_adapter.rs` (wire only: header
build `:1286`, body build `:1296`, `extract_output_text` `:1435`). raios-core
gets completion request/response record entries with hash-vector tests. `ask`
console path re-pointed at the contract.

**Verification.** Boundary refactor → `-Profile provider-memory`,
`-Profile provider-memory-full`, `-Profile quick`. Bar: **ZERO needle
changes** — done only when serial output is byte-identical (M2-collapse
discipline). Host: raios-core tests. Optional local
`openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` if a key image exists (note
if skipped).

**Fail-closed.** All existing gates stay in the generic layer; an adapter
physically cannot skip them — it never sees the key bytes or the socket.

**STOP-tripwires.** ANY needle change (reconcile before proceeding — it means
behavior changed); net line growth > +300 (split should be near-neutral).

```text
Packet id: M10B-2-adapter-extraction
Goal: Split openai.rs into provider-neutral provider_client.rs (gates, hashing,
  bindings, transport) and openai_adapter.rs (wire format) behind typed
  ProviderCompletionRequest/Response record entries. Serial output byte-identical.
Read first: this map Slice 3 + D3; seed-kernel/src/openai.rs IN FULL;
  provider_trust.rs + provider_descriptor.rs (post Slice 2); net.rs public API;
  raios-core record model; docs/plan-reviews/m2-collapse-map-2026-07-05.md
  (byte-identical discipline).
Allowed write set: openai.rs (shrink/delete); provider_client.rs (new);
  openai_adapter.rs (new); main.rs wiring; raios-core/src/**;
  docs/PROJECT_STATUS.md.
Forbidden: any behavior or marker-string change; vm-harness edits (no needle
  edits — that is the point); provider_config.rs; Anthropic code; net.rs;
  TLS/verifier code.
Constraints: adapter interface = build_request_head(key_len)->String,
  build_request_body(req)->String, parse_response(bytes)->Result<Response>; the
  adapter never receives key bytes or the socket; key copy + HTTPS write stay in
  provider_client.rs AFTER the trust gate exactly as today; fmt clean;
  descriptor-resign if attestation complains.
Definition of done: provider-memory, provider-memory-full, quick all green with
  ZERO needle diffs (cite all three report filenames); host tests green;
  capability sentence: "AI requests flow through one typed provider contract with
  trust gates enforced provider-neutrally."
Report format: before/after line counts (net delta); "zero needle diffs" backed
  by report filenames; host test count; anything that resisted the split.
```

---

## Slice 4 (M10B-3) — per-provider RAM key slots

**Capability.** The owner can provision, inspect (set/unset only), and clear
an API key per provider in SET mode; each key is a zeroizable RAM slot
classified `secret`; no snapshot/export path can carry key material.

**Files (verify at execution).** `provider_config.rs` (single slot → fixed
table keyed by provider_id; zeroizing `clear`); `console.rs` grammar
`set key <provider> ...` (old OpenAI form kept as alias); snapshot emitters
(`api_key_set` per provider); `scripts/package-stage0.ps1` +
`write-stage0-usb.ps1` gain `-EmbedAnthropicApiKeyFromEnv` + pin flags
(mirroring OpenAI; env `RAIOS_DEFAULT_ANTHROPIC_API_KEY`,
`ANTHROPIC_SPKI_SHA256`); `scripts/scan-secrets.ps1` learns `sk-ant-`
patterns; `docs/SECRETS.md` updated.

**Verification.** Focused `-Profile provider-memory` + `-Profile quick`. New
needles: `provider key set: provider=anthropic accepted len_ok=true` (value
never echoed), `provider key cleared: provider=anthropic zeroized=true`,
per-provider `api_key_set` snapshot fields. Then run
`powershell ...\scripts\scan-secrets.ps1` (must pass) and grep the serial log
for the test key value — zero hits.

**Fail-closed.** Key copy only after THAT provider's trust is positive
(unchanged gate, now per-provider); provider A's key never usable for B (slot
lookup by descriptor id, no fallback); keys never reach SEED_DATA or any
durable record (D4 rule).

**STOP-tripwires.** Any proposal to persist/remember keys across boots (needs
the sealed-secret ADR); secret scan failing on tracked files.

```text
Packet id: M10B-3-provider-key-slots
Goal: Per-provider RAM-only key slots with SET set/clear, zeroization,
  per-provider api_key_set snapshot fields, Anthropic env-embed packaging flags,
  and scanner coverage for Anthropic key shapes.
Read first: this map Slice 4 + D4; seed-kernel/src/provider_config.rs (all);
  console.rs (set key grammar); docs/SECRETS.md; scripts/package-stage0.ps1
  (key-embed refusal logic); scripts/scan-secrets.ps1.
Allowed write set: provider_config.rs; console.rs; api_key_set snapshot emit
  sites; scripts/package-stage0.ps1; scripts/write-stage0-usb.ps1;
  scripts/scan-secrets.ps1; docs/SECRETS.md; vm-harness needle additions;
  docs/PROJECT_STATUS.md.
Forbidden: key material in any record/event/snapshot/log/disk path; any key
  persistence; provider_client/adapter logic changes; weakening the packaging
  refusal to embed keys into tracked images.
Constraints: fixed-capacity slots (2 for now); zeroize on clear and re-provision;
  old `set key <value>` grammar keeps working for openai; packaging refuses
  Anthropic keys into release\esp or the default image exactly like OpenAI; fmt
  clean; descriptor-resign if needed.
Definition of done: provider-memory + quick green incl. new key-slot needles;
  scan-secrets.ps1 clean output pasted; serial-log grep for the smoke key = 0
  hits; capability sentence: "The owner can hold and clear a separate RAM-only
  key per provider."
Report format: files + line counts; new needle strings; report filenames; secret
  scan summary; zero-hit key grep confirmation.
```

---

## Slice 5 (M10C-1) — Anthropic adapter, pins, provider smoke matrix

**Capability.** The system can hold a pinned-SPKI-verified TLS session to
api.anthropic.com and get a real completion back via the Anthropic Messages
API — a second provider through the same contract, gates, and evidence chain
— and the harness smokes any provider from one script.

**Wire facts (verified against Anthropic docs cache 2026-06; re-verify shape
at execution with a host-side curl).** `POST https://api.anthropic.com/v1/messages`;
headers `content-type: application/json`, `x-api-key: <key>`,
`anthropic-version: 2023-06-01`; minimal body
`{"model":"<descriptor>","max_tokens":128,"messages":[{"role":"user","content":"<prompt>"}]}`;
response text at `content[] where type=="text" -> text`, plus `stop_reason`.
Model per OD-1 (`claude-opus-4-8` default / `claude-haiku-4-5` cheap smoke).

**Files (verify at execution).** New `anthropic_adapter.rs`;
`provider_descriptor.rs` Anthropic entry (host `api.anthropic.com:443`, path
`/v1/messages`, auth `x_api_key_version`, pins `RAIOS_ANTHROPIC_SPKI_SHA256(_NEXT)`);
console `ask` provider selector (default openai; e.g. `ask@anthropic <text>`,
match console conventions); harness: new `vm-harness/provider-direct-smoke.ps1`
with `-Provider openai|anthropic` (old `openai-direct-smoke.ps1` becomes a
forwarding shim for one milestone) + `vm-harness/provider-smoke-matrix.ps1`
running per provider: ExpectSpkiPinnedTrust, ExpectPinMismatch, and (key
present) ExpectProviderResponse.

**Verification.** NEW named focused profile `provider-adapter` (ValidateSet in
`shadow-vm-smoke.ps1:12` + `shadow-vm-smoke-profile-provider-adapter.ps1`)
covering keyless fail-closed behavior for BOTH providers:
`...\shadow-vm-smoke.ps1 -Profile provider-adapter`. Live smokes (local key
images, never CI):
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\provider-direct-smoke.ps1 -Provider anthropic -ExpectSpkiPinnedTrust`
then `-ExpectProviderResponse`, then the matrix script. Needle families:
- F-trust: `anthropic: TLS provider trust verified: pinned_spki sha256:<pin>`
  (openai one unchanged);
- F-fail: `ANTHROPIC DIRECT TLS PIN MISMATCH` / `... PIN CONFIG MISSING`
  before key copy or body write (mirror of provider-trust-v0 criteria);
- F-binding: `raios.provider_request_binding.v0` + export-audit-binding
  markers with `provider=anthropic` on the positive path only;
- F-adapter: typed completion records with `format=anthropic_messages_v1`.
Before claiming M10C: one full profile (`-Profile full`).

**Fail-closed.** No Anthropic bypass flag exists, period. No pin → no key
copy, no body write. Context export for Anthropic uses the identical denial
chain as OpenAI. provider_id is bound into envelope/binding hashes so a
provider-B response can never be attributed to A.

**Risk R1 (honest).** The verifier checks CertificateVerify with ECDSA
P-256/SHA-256 only. If api.anthropic.com's leaf key is not P-256, the state
is `pin_verifier_unavailable` (fail-closed, correct) and the positive smoke is
impossible without widening the algorithm set (RSA-PSS / P-384) — a
trust-surface extension: STOP, report, get owner sign-off, plan as its own
focused slice. Check the live leaf FIRST (`openssl s_client` from the host)
before writing code.

**STOP-tripwires.** R1 fires; TLS1.3 handshake incompatibility needing an
embedded-tls patch beyond verifier inputs; OD-1 unanswered; any urge to attach
system context to the new provider.

```text
Packet id: M10C-1-anthropic-adapter
Goal: Second provider Anthropic (Messages API) behind the typed contract with
  SPKI-pinned trust and a generalized provider smoke matrix. OpenAI behavior
  byte-identical.
Read first: this map Slice 5 (incl. wire facts + R1); provider_client.rs,
  openai_adapter.rs, provider_descriptor.rs, provider_trust.rs (post Slices 2-4);
  vm-harness/openai-direct-smoke.ps1 (all); docs/SECRETS.md;
  device-protocol/provider-trust-v0.md.
Allowed write set: anthropic_adapter.rs (new); provider_descriptor.rs (one
  entry); console ask-routing site; vm-harness/provider-direct-smoke.ps1 (new);
  vm-harness/provider-smoke-matrix.ps1 (new); vm-harness/openai-direct-smoke.ps1
  (forwarding shim only); vm-harness/shadow-vm-smoke.ps1 (ValidateSet+dispatch);
  vm-harness/shadow-vm-smoke-profile-provider-adapter.ps1 (new);
  device-protocol/provider-trust-v0.md; docs/SECRETS.md; docs/PROJECT_STATUS.md.
Forbidden: provider_client.rs gate changes; verifier algorithm changes (STOP per
  R1 instead); any bypass flag for anthropic; hardcoded model ids in the adapter
  (descriptor field only); committing any key or real pin; sending snapshots/
  tool schemas/system context to any provider.
Constraints: BEFORE coding, check api.anthropic.com leaf key type from the host
  (openssl s_client -connect api.anthropic.com:443) and record the finding; if
  not P-256 ECDSA, STOP and report. Parse must handle content-length and chunked
  bodies like the OpenAI path; max_tokens 128; anthropic-version exactly
  2023-06-01; prompt escaped via existing push_json_string; fmt clean; secret
  scan before commit; descriptor-resign if attestation complains.
Definition of done: provider-adapter profile green (keyless, both providers fail
  closed); provider-direct-smoke -Provider anthropic -ExpectSpkiPinnedTrust and
  -ExpectProviderResponse green with a local key image; -ExpectPinMismatch fails
  closed before key copy; openai smoke still green; ONE full profile green before
  claiming M10C; capability sentence: "The system can get a real completion from
  a second provider through the same pinned-trust evidence chain."
Report format: leaf-key-type finding; all report/smoke filenames + results; new
  needle strings by family; confirmation openai needles unchanged; secret scan
  output.
```

---

## Slice 6 (M10D-1) — WebPKI groundwork: host-proven core + chain evidence

**Capability.** The system records the real presented certificate chain
(hashes, bounded) of every provider handshake as typed evidence labeled
`validation_authority=none_stage0`, and a HOST-tested chain-validation core
(rustls-webpki + RustCrypto verifiers over captured real chains + pinned
Mozilla anchors) proves the validation logic and record schema M11's TLS
service will run in-guest.

**Files (verify at execution).** New host crate `provider-chain-core/`
(workspace member, std, NOT kernel): rustls-webpki path validation wired to
`p256`/`p384`/`rsa` verifiers, `webpki-roots` anchors, fixture chains captured
from api.openai.com and api.anthropic.com; emits
`raios.provider_chain_validation.v0` via the record model. Kernel: retain
per-handshake leaf hash + intermediate cert hashes as
`raios.provider_chain_evidence.v0` — requires the embedded-tls patch to expose
intermediate DERs; verify first whether it already does; if not, extend the
vendored patch minimally (hash-in-place, count ≤6, never store full DERs in
the event ring). Plus a short M11 handoff doc in `docs/plan-reviews/`.

**Verification.** Host:
`cargo test --locked -p provider-chain-core -p raios-core` — validation green
on fresh fixtures AND red on negative cases (broken chain, expired-at-fixed-
time, wrong hostname) — the fail-closed cases are the point. Fixtures carry a
captured-at date; tests use a FIXED verification time from fixture metadata
(never now()) so CI stays green as certs age — honest, since v0 time authority
is owner-attested anyway. Kernel capture touches the TLS boundary → focused
`-Profile provider-adapter` + `-Profile quick`; new needle:
`provider chain evidence: provider=<id> leaf_sha256=<h> intermediates=<n> validation_authority=none_stage0`.

**Fail-closed.** Kernel trust decisions completely unchanged — evidence
capture must not feed the trust state machine; `webpki_verified` remains
unreachable; the host core is proof, not authority.

**STOP-tripwires.** Any call from kernel into the host-proven core (Order A
via the back door); embedded-tls patch touching handshake/policy logic instead
of read-only input exposure; unparking ota/registry/fake-cloud (needs new ADR).

```text
Packet id: M10D-1-webpki-host-core
Goal: Host-tested WebPKI chain-validation core over captured real provider
  chains + in-kernel chain-evidence capture (hashes only, authority=none) + a
  written M11 handoff. Kernel trust decisions unchanged.
Read first: this map Slice 6 + D1; device-protocol/provider-trust-v0.md;
  vendor/embedded-tls-0.17.0 verifier-input patch (what it exposes today); the
  generic pin verifier (post Slice 2); raios-core record model;
  docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md (handoff-doc style).
Allowed write set: provider-chain-core/** (new host crate + fixtures + README);
  workspace Cargo.toml member list; raios-core/src/** (2 record entries);
  vendor/embedded-tls-0.17.0/** (minimal read-only intermediate-DER exposure
  ONLY if not already present); kernel verifier call site to hash intermediates
  into the evidence record; event_log.rs (one constructor + vocabulary);
  vm-harness/shadow-vm-smoke-profile-provider-adapter.ps1 (one needle);
  docs/plan-reviews/m11-tls-service-handoff-2026-XX-XX.md (new);
  docs/PROJECT_STATUS.md.
Forbidden: calling chain validation from kernel code; changing any trust state
  transition; vendoring webpki/rsa/p384 into seed-kernel; full cert DERs in the
  event ring; network fetches at test time (fixtures checked in); fixtures with
  private keys (public chains only — run scan-secrets anyway).
Constraints: host tests include the 3 negative cases and they must FAIL
  validation; fixed verification time from fixture metadata, never now();
  capture commands (openssl) documented in the crate README; bounded
  intermediate count (≤6) in the kernel record; fmt clean; descriptor-resign if
  attestation complains.
Definition of done: host tests green incl. negative cases (count them);
  provider-adapter + quick green with the new evidence needle; handoff doc names
  import surface, record schemas, fixture path, open algorithm questions;
  capability sentence: "The system records real provider certificate chains as
  evidence and the M11 chain-validation logic is proven on the host."
Report format: host test counts (positive/negative); whether the embedded-tls
  patch needed extending (diff size if so); report filenames; handoff doc path.
```

---

## Global STOP-tripwires (orchestrator: stop and ask the owner)

1. Anything needing a new ADR: sealed/persistent secrets, unparking
   ota/registry/fake-cloud, network time, kernel WebPKI (Order A override).
2. Trust-model changes: new trust states, verifier algorithm widening (R1),
   bypass flags, any relaxation of a `capability_denied`.
3. Destructive disk operations, or anything overwriting
   `release/raios-stage0.img`.
4. OD-1 (Anthropic key/model/cost) before M10C starts.
5. Needle regressions in Slices 2-3 that a worker proposes to "fix" by editing
   the needle instead of the code.
6. Secret-scan hits, or a real key/pin in any tracked file.
7. Full profile red at any point → Red Gate Rule, repair only.

## Verdict and budget

M10 is medium-sized and unusually parallelizable: M10A and M10D-host are
independent lanes; M10B (2→3→4) is the critical path; M10C is the proof slice,
as echo was for M5. Estimate: 6 implementation slices + Slice 0, ~4 focused VM
runs, live smokes only with local key images, ONE full profile before claiming
the milestone. Riskiest unknowns: R1 (Anthropic leaf algorithm) and
intermediate-DER exposure in the vendored embedded-tls patch — both checked
before code is written, both with STOP paths. The deliberately unbought item
is `webpki_verified` itself: M10 makes it provable; M11 makes it true, outside
the kernel, where it belongs.
