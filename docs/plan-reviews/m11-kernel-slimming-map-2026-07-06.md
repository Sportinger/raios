# M11 Kernel Slimming / Services-out-of-kernel — Design Map (2026-07-06)

**Authored 2026-07-06 AHEAD of execution as pre-planning.** Execution
preconditions: **M6 (Promotion Loop) CLOSED, M7A-D (Persistence) CLOSED,
M10 (Provider Trust Hardening) CLOSED.** M8/M9 are expected closed by
numbering; slices M11-2/-3 strictly need only M6+M7D and may be pulled
earlier only by explicit owner instruction. Every file:line below is
HEAD as of 2026-07-06 (before M6-M10 execution); M6-M10 WILL rewrite the
openai/net/wasm surfaces, and all M6/M7 interface names used here
(`module.submit_candidate`, promotion transaction, persistent artifact
store) are the PLANNED names from those maps, not verified code.
**MANDATORY Slice 0 = map revalidation:** re-check every file:line claim
against HEAD, update this map first if reality diverged, commit the map
update BEFORE any implementation slice.

## 1. What M11 Makes True

Milestone capability sentence: "No application-layer internet content —
TLS records, TLS handshake messages, HTTP responses, provider JSON — is
parsed by kernel-resident code; it is parsed inside fuel-metered Wasm
services with a narrow granted import surface, and the provider path is
a promoted, persistent, replaceable service."

Honest scoping of "the kernel does not parse the internet": it means
application-layer, attacker-influenced content. It does NOT mean L2-L4:
smoltcp keeps parsing Ethernet/IP/TCP headers in the kernel (fixed-shape
binary headers, driver-adjacent, ADR 0005 keeps drivers native). X.509
chain parsing stays kernel-side in M11 (see D2). DNS message parsing
(`net.rs:579-735`) stays kernel in M11 and is next-candidate #1.

This is the concrete path to the owner's end vision: a slim permanent
core that boots and brings up network, everything else replaceable
loaded services. M11 is also the first time the M6 promotion loop and
M7 persistence are consumed by INFRASTRUCTURE itself — the product loop
closing on its own plumbing.

## 2. Feasibility: Interpreted Crypto vs Crypto-as-Host-Imports

wasmi =0.31.2 is an interpreter (no JIT, ADR 0005). Estimated overhead
for compute-heavy code: ~30-150x native. Unmeasured — M11-1 measures.

| Workload (per provider request) | Native est. | Interpreted est. |
|---|---|---|
| P-256 ECDHE + ECDSA verify (handshake, once) | 1-10 ms | 0.3-1.5 s |
| HKDF/SHA-256 key schedule (once) | <1 ms | <100 ms |
| AES-128-GCM open/seal, ~32 KiB response | 1-5 ms (soft AES) | 0.1-2 s |

Estimates are wide and possibly wrong; that is why M11-1 exists. But
the recommendation does NOT hinge on performance:

1. **Attack surface lives in parsing, not primitives.** Heartbleed-class
   bugs are length/state confusion in record/message/cert/JSON parsing.
   Crypto primitives are fixed-shape constant-time code.
2. **Timing.** An interpreted crypto implementation's constant-time
   properties are unauditable; fuel metering adds data-dependent timing.
3. **Zero new kernel code.** SHA-256 (raios-core), P-256 (descriptor
   verification), AES-GCM (embedded-tls's crypto internals) already
   exist kernel-side. Crypto-in-Wasm would ADD a whole crypto stack to
   audit; crypto-as-host-imports only re-exposes what exists.

**RECOMMENDATION (firm): crypto primitives stay narrow HOST FUNCTIONS;
all protocol/parsing logic moves to Wasm.** Key material NEVER enters
guest memory (opaque session handles, §4). M11-1's measurement can only
strengthen this (if interpreted crypto is slow) or leave it unchanged
(the attack-surface argument stands even if it is fast).

## 3. Host-Import Surface for a Network-Parser Service

Each import is one capability in the service's computed grant (the M6
grant format decides exact naming at execution time; keep the existing
`env.`-module linker convention from `wasm_runtime.rs:521`). Drivers
stay native. All ptr/len pairs are bounds-checked against guest memory;
per-call caps (32 KiB); invalid handle or ungrated import → typed
`capability_denied` evidence, never a panic.

| Import | Shape | Capability meaning |
|---|---|---|
| `env.log` | (ptr,len) | exists today (`wasm_runtime.rs:528`) |
| `env.net_resolve` | (host ptr,len, out_ip) -> code | kernel DNS lookup |
| `env.net_tcp_connect` | (ip, port) -> conn handle/err | open one TCP conn |
| `env.net_tcp_send` | (conn, ptr, len) -> code | send bytes |
| `env.net_tcp_recv` | (conn, ptr, len) -> n/code | recv bytes |
| `env.net_tcp_close` | (conn) | close |
| `env.sys_entropy` | (ptr, len) -> code | KernelRng fill (ClientHello random only; ECDHE keys stay kernel) |
| `env.sys_monotonic_ms` | () -> u64 | timeouts only, not trusted wall time |
| `env.crypto_sha256` | (ptr,len,out32) | hash PUBLIC bytes (transcript) |
| `env.tls_session_open` | (profile) -> session/err | allocate session slot |
| `env.tls_ecdhe_start` | (session, out_pub) | kernel keygen; returns public only |
| `env.tls_ecdhe_finish` | (session, peer_pub ptr,len) | shared secret stays kernel |
| `env.tls_keyschedule` | (session, stage, transcript ptr) | HKDF stages kernel-side |
| `env.tls_aead_seal` | (session, aad, in, out) -> n/err | encrypt record payload |
| `env.tls_aead_open` | (session, aad, in, out) -> n/err | decrypt record payload |
| `env.tls_finished` | (session, transcript, out_mac / verify) -> code | Finished MACs |
| `env.tls_verify_cert_chain` | (session, msg ptr,len) -> verdict | kernel verifier (pinned / M10 WebPKI) decides; guest never decides trust |
| `env.tls_session_close` | (session) | free slot |
| `env.provider_write_secret_header` | (session) -> code | kernel seals+sends `Authorization: Bearer <key>` itself (§4) |

The tcp_send/recv imports internally pump `net::poll()` with the same
timeout discipline as `tls_io.rs:50-69` (KernelTcpStream::wait_for), so
the guest sees blocking-with-timeout semantics. Same blocking model as
today's `perform_https_request`; async service scheduling is OUT of M11
scope.

## 4. Opaque Session Handles and Secret Splicing

- `SessionHandle = u32`: upper 8 bits generation counter, lower 24 bits
  index into a fixed kernel table (`MAX_TLS_SESSIONS = 2`). Entry:
  state enum, negotiated AEAD keys/IVs both directions, sequence
  counters, owning service instance id. TCP conn handles use the same
  scheme in a separate table (`MAX_GUEST_TCP = 2`).
- Every handle op checks ownership (handle bound to calling instance);
  mismatch/stale generation → typed error evidence, fail-closed.
- Keys, shared secrets, HKDF stages: derived and stored kernel-side,
  NEVER written to guest memory. No import returns key bytes.
- API key: today the kernel splices `Authorization: Bearer <key>` into
  the TLS stream itself (`openai.rs:1235-1253`). That stays kernel.
  With the record layer in the guest, plaintext HTTP flows through
  guest memory — so the secret header must be sealed and transmitted by
  the KERNEL mid-stream: guest sends request-line + headers part 1,
  calls `env.provider_write_secret_header(session)`, kernel seals its
  own record(s) carrying the header, guest continues with remaining
  headers/body. TLS records are stream chunks, so this is legal.
  STOP-tripwire if this proves unworkable at execution time.
- Received response plaintext IS guest-visible — that is the point
  (parsing moves to the guest). Requests contain no secret except the
  header handled above.

## 5. Staging and the Kernel-Shrink Ledger

Stages (each = net kernel deletion + byte-identical provider needles):
(1) HTTP/JSON response parsing → Wasm (pure bytes-in/bytes-out, zero
new capability imports, immediate attack-surface win); (2) TLS record
layer → Wasm (crypto/session imports land); (3) handshake logic → Wasm
(embedded-tls leaves the kernel); (4) provider path consolidated as one
promoted persistent replaceable service.

Baseline ledger (verify in Slice 0; M6-M10 will have changed these):
`openai.rs` 1,624 lines (pure parse block ~1322-1597), `tls_io.rs` 105,
`openai_trust.rs` 342, `net.rs` 756, `wasm_runtime.rs` 578; plus the
vendored `embedded-tls` 0.17 dependency (kernel-resident attack surface
even though not kernel LOC).

Standing measurement rule: every extraction slice's report MUST include
the `seed-kernel/src` net line delta, the list of deleted functions,
and needle evidence of byte-identical provider-path behavior. M11-1 and
M11-4 are the only allowed net-additive slices (benchmark + import
surface); their additions must be paid back by M11-3/-5/-6 deletions.
Golden needles are ground truth, not worker self-reports (M2 Batch 4
lesson: worker claimed 1120/1120 identical, needles caught 10 dropped
fields). New needles are derived from OBSERVED serial output, never
invented (M3 lesson).

## 6. Slices

### M11-0 — Map revalidation (MANDATORY, first)

Capability: the orchestrator has a map whose every claim is true at
HEAD. Files: this map only. Verification: none (docs-only diff check).
Fail-closed: no implementation before the updated map is committed.

```text
Packet id: M11-0-map-revalidation
Goal: Re-verify every file:line, LOC count, interface name, and profile
  name in docs/plan-reviews/m11-kernel-slimming-map-2026-07-06.md
  against current HEAD. M6-M10 executed after this map was authored.
Read first: the map; docs/ROADMAP.md cursor; docs/PROJECT_STATUS.md;
  seed-kernel/src/openai.rs, net.rs, tls_io.rs, wasm_runtime.rs,
  openai_trust.rs; the M6/M7/M10 maps in docs/plan-reviews/ and their
  closure entries; seed-kernel/build.rs attested source set.
Allowed write set: docs/plan-reviews/m11-kernel-slimming-map-*.md only.
Forbidden: any code change; renumbering slices; changing verdicts
  without listing the diverged evidence.
Constraints: for each corrected claim, cite old claim -> new reality
  with file:line. If the M6 candidate-intake method or M7D persistent
  artifact store differ from this map's assumptions, update §6 packets.
Definition of done: map committed with a "revalidated at <commit>"
  header line; divergence list in the commit message.
Report format: table of claim | old | new | slice impact; list of
  slices whose packets changed.
```

STOP-tripwires: any M6-M10 outcome that contradicts a design constant
here (e.g. promotion loop cannot grant net imports; M7D store absent);
STOP and re-plan with the owner rather than patching around it.

### M11-1 — Interpreted-crypto measurement guest

Capability: raiOS can execute a computational Wasm workload under the
existing envelope and report measured cost (fuel + wall ms) as typed
evidence — replacing §2's estimates with numbers before any TLS design
freezes.

Files: new `wasm-guests/bench-crypto/` (SHA-256 of 32 KiB, software
AES-GCM open of one 16 KiB record, one P-256 scalar-mult, pure Rust
no_std, vendored primitive crates in the GUEST only); a temporary
labeled agent method `wasm.bench_probe` behind the existing wasm
runtime (pattern: `wasm_runtime.rs:157` echo probe); needles for the
evidence record. Verify at execution time how M6 changed guest intake —
prefer submitting the bench artifact through the M6 candidate path.

Verification: quick profile —
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick`
New needles: bench evidence record present with nonzero fuel + ms per
workload. Fail-closed: bench guest gets `env.log` +
`env.sys_monotonic_ms` at most; no net/crypto/session imports exist yet.

```text
Packet id: M11-1-crypto-bench-guest
Goal: Measure interpreted crypto cost in-kernel: a wasm32 guest running
  SHA-256/32KiB, AES-128-GCM-open/16KiB, one P-256 scalar mult, each
  reported as (fuel consumed, wall ms) in a typed evidence record.
Read first: docs/plan-reviews/m11-kernel-slimming-map-2026-07-06.md §2;
  seed-kernel/src/wasm_runtime.rs (echo probe + envelope + fuel);
  wasm-guests/svc-demo-echo/ + scripts/build-wasm-guest.ps1; the M6
  candidate intake path as it exists at HEAD.
Allowed write set: wasm-guests/bench-crypto/**; seed-kernel/src/
  wasm_runtime.rs (new probe fn only); the agent method table entry;
  vm-harness quick profile needle file(s); scripts/build-wasm-guest.ps1
  (parameterization only).
Forbidden: adding crypto host imports; touching openai.rs/net.rs/
  tls_io.rs; new raios.*.v0 schemas outside the record model; changing
  the echo/hello paths.
Constraints: guest crates vendored+pinned; fuel budget explicit; trap/
  exhaustion ends as evidence, not panic; label the probe as test
  infrastructure. If any attested source-set file is touched (check
  build.rs list), run the descriptor re-sign flow (target/
  descriptor-resign) and commit updated signatures; LF-only.
Definition of done: quick profile green with the new bench needles;
  measured numbers pasted in the report; net kernel delta stated.
Report format: capability sentence; table workload|fuel|ms|est-native-x;
  quick report filename; kernel LOC delta.
```

STOP-tripwires: measured overhead so extreme (>1000x) that even
parsing-only-in-Wasm looks unviable for the provider timeout budget —
STOP, owner decision before M11-2.

### M11-2 — HTTP/JSON parser service through the promotion loop (parallel proof)

Capability: an external Wasm artifact (`svc.net.httpparse`) travels the
REAL M6 loop (submit → Shadow-VM verify → grant → promote) and is
persisted/re-promoted via M7D, and the kernel runs it in parallel with
the old parser on fixture vectors, emitting byte-identity comparison
evidence — the first infrastructure service consumed by the loop.

Design: pure bytes-in/bytes-out. Kernel copies response bytes into
guest memory; guest export parses (status line, headers,
content-length, chunked decode, `output_text` JSON extraction incl.
unicode escapes — the logic of `openai.rs:1322-1597`) and writes a
typed result region; kernel compares against the old functions' output.
Import surface: `env.log` ONLY — zero new capabilities. Guest logic
lives in a host-testable no_std lib crate (`net-parser-core`) wrapped
by the guest cdylib, with fixture vectors covering: 200 + chunked, 200
+ content-length, non-200 with error message, truncated body, malformed
chunk sizes, oversized input.

OWNER DECISION D3 — who authors the guest: (a) worker-authored in-repo
source built by script, submitted at RUNTIME through the M6 external
path (RECOMMENDED — trust comes from the promotion evidence chain, not
source location); (b) fully external AI-authored artifact (the purist
M6 story; slower, no auditability gain for infrastructure); (c)
build-embedded like echo (REJECTED: bypasses the loop M11 exists to
consume). Proceed with (a) unless the owner objects at kickoff.

Verification: NEW focused profile `net-parser` (new file
`vm-harness/shadow-vm-smoke-profile-net-parser.ps1` following the
existing profile pattern) —
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile net-parser -TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10`
plus quick. New needles: promotion-loop evidence for svc.net.httpparse;
per-fixture comparison records old_hash==new_hash; malformed fixtures
end as typed parse-error evidence. Host tests:
`cargo test --locked -p net-parser-core`.
Fail-closed: old kernel parser remains AUTHORITATIVE this slice; the
parallel run is labeled test infrastructure; guest parse failure never
affects the live provider answer yet; no net/crypto imports granted.

```text
Packet id: M11-2-httpparse-service-parallel
Goal: svc.net.httpparse exists as a promoted, persisted external Wasm
  service (M6 loop + M7D re-promotion) whose parse output is proven
  byte-identical to openai.rs's parse functions on fixture vectors via
  in-guest comparison evidence. Old parser stays authoritative.
Read first: map §3/§5/§6.2; openai.rs:1322-1597 (parse functions — line
  numbers from 2026-07-06, re-locate at HEAD); the M6 submit/verify/
  grant/promote methods at HEAD; M7D re-promotion path at HEAD;
  wasm-guests/svc-demo-echo/; vm-harness/shadow-vm-smoke-profile-
  hello-rollback-dry-run.ps1 as profile template.
Allowed write set: wasm-guests/svc-net-parser/** (guest + net-parser-
  core lib with host tests); seed-kernel/src/ (comparison harness fn +
  method table entry + fixture wiring only); vm-harness/shadow-vm-
  smoke-profile-net-parser.ps1 + needle updates; workspace Cargo files.
Forbidden: deleting or bypassing the old kernel parse functions;
  granting any import beyond env.log; real provider traffic in the
  profile; touching tls_io.rs/net.rs/openai_trust.rs; secrets in
  fixtures.
Constraints: fixtures are SYNTHETIC provider-shaped bytes (no recorded
  real responses); comparison = sha256 of a canonical result encoding
  on both sides; record-model entries only for new evidence; re-sign if
  attested sources touched (check build.rs set); LF-only fixtures.
Definition of done: cargo test --locked -p net-parser-core green;
  net-parser profile green incl. promotion + comparison needles; quick
  green; reboot re-promotion needle green (M7D); kernel LOC delta
  reported (expected small +; deletion lands in M11-3).
Report format: capability sentence; fixture list with old/new hashes;
  profile report filenames; promotion evidence ids; LOC delta.
```

STOP-tripwires: M6 grant vocabulary cannot express "env.log only";
persistence re-promotion fails evidence gates; any fixture requires
real provider content — STOP, ask owner.

### M11-3 — Cutover: kernel stops parsing HTTP/JSON

Capability: the live provider answer is produced by svc.net.httpparse;
the kernel HTTP/JSON parse functions are DELETED (first real kernel
shrink), and a parser failure fails the request closed with typed
evidence — there is no silent kernel fallback because the fallback no
longer exists.

Files: `openai.rs` — `perform_https_request` routes response bytes
through the promoted service; delete `read_http_response` tail parsing,
`http_response_complete`, `parse_status`, `parse_content_length`,
`header_contains`, `decoded_body`, `decode_chunked`,
`extract_output_text`, `extract_json_string_after`,
`push_json_unicode_escape`, `read_json_u16`, `split_header` +
helpers (~275 lines; re-locate at HEAD). Comparison harness from M11-2
removed. Expected net kernel deletion: ≥250 lines.

Verification: net-parser focused profile + quick + (owner-run, key
required, optional) `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1`
+ FULL checkpoint:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile full`
New needles: "parse routed via svc.net.httpparse" evidence on the ask
path; parser-failure fixture → typed provider error, VM stays alive.
Fail-closed: if the parser service is not promoted/healthy, provider
requests fail closed with evidence (no request sent is acceptable; no
un-parsed fallback).

```text
Packet id: M11-3-httpparse-cutover
Goal: Route the live provider response through svc.net.httpparse and
  DELETE the kernel HTTP/JSON parse functions; provider behavior byte-
  identical per needles; parser absence/failure = typed fail-closed.
Read first: map §6.3; M11-2 report; openai.rs perform_https_request +
  parse block at HEAD; net-parser profile.
Allowed write set: seed-kernel/src/openai.rs (+ the routing shim
  module); vm-harness net-parser/quick/full needle updates.
Forbidden: keeping dead parse functions "just in case"; any fallback
  path that parses in-kernel; changing request construction, trust
  checks, or key splicing; touching embedded-tls usage.
Constraints: needle updates derived from observed serial output only;
  re-sign if attested sources touched; report the exact deleted
  function list + net LOC delta.
Definition of done: net-parser + quick green; FULL profile green
  (checkpoint); net kernel deletion >= ~250 lines; failure-path needle
  green.
Report format: capability sentence; deleted functions; LOC delta;
  report filenames (incl. full).
```

STOP-tripwires: byte-identity cannot be reached for some response class
(would need output-shape change → needle rewrites beyond mechanical) —
STOP, owner decision.

### M11-4 — Crypto/session host-import surface + guest TLS record layer (vectors only)

Capability: the §3/§4 import surface and opaque session/conn handle
tables exist and are grantable, and a host-tested guest record-layer
library (`tls-record-core`) frames/deframes real TLS 1.3 records
against those imports on test vectors — no live traffic yet.

Files: new `seed-kernel/src/wasm_net_imports.rs` (linker definitions,
handle tables, ownership checks, bounds caps) reusing kernel crypto
already present (raios-core sha256, p256, embedded-tls internals or a
pinned aes-gcm crate — verify what M10 left available); new
`wasm-guests/svc-net-tls/` with `tls-record-core` no_std lib (record
framing, content types, length checks, key-update handling) host-tested
against a mock of the import surface with RFC 8448-style vectors.
This is the second allowed net-additive slice.

**STOP-TRIPWIRE (blocking, before implementation): OWNER DECISION D1 —
the session-handle / crypto-as-host-imports / secrets-never-in-guest
design is a trust-model addition and should be recorded as a short new
ADR.** Options: (a) new ADR before M11-4 (RECOMMENDED); (b) amendment
section appended to ADR 0005; (c) map-only. Orchestrator drafts, owner
approves; do not implement without one of (a)/(b) explicitly chosen.

Verification: `cargo test --locked -p tls-record-core` (vectors);
quick profile (import surface present but UNGRANTED to all existing
services — needles prove echo/hello/httpparse still instantiate with
their old envelopes and a probe module importing `env.tls_aead_open`
without grant fails at instantiation, same pattern as M4's negative
proof). Fail-closed: no service is granted the new imports this slice;
handle ops with stale/foreign handles produce typed errors in host
tests; key bytes provably never cross into guest memory (no import
signature returns them).

```text
Packet id: M11-4-crypto-session-imports
Goal: Implement the env.net_*/env.sys_*/env.crypto_sha256/env.tls_*/
  env.provider_write_secret_header host imports with opaque generation-
  checked handle tables (map §3/§4), granted to NOBODY yet; plus
  tls-record-core (guest TLS 1.3 record framing lib) green on host
  vectors against a mocked import surface.
Read first: map §2-§4 + the approved ADR/amendment (D1); wasm_runtime.rs
  envelope/linker at HEAD; tls_io.rs; net.rs tcp_* fns; openai.rs
  KernelRng; M6 grant vocabulary at HEAD; RFC 8446 §5 + RFC 8448
  vectors.
Allowed write set: seed-kernel/src/wasm_net_imports.rs (new) + minimal
  wasm_runtime.rs linker hook; wasm-guests/svc-net-tls/** incl.
  tls-record-core; workspace Cargo files; quick-profile negative-proof
  needles.
Forbidden: granting any new import to any service; any import that
  returns key/secret bytes; live TLS traffic through the guest;
  touching openai.rs request flow; new vendored kernel crypto beyond
  what already exists (guest-side vendoring is fine).
Constraints: per-call 32 KiB caps; ownership + generation checks on
  every handle op; imports pump net::poll with tls_io.rs-style
  timeouts; record-model evidence only; re-sign if attested sources
  touched.
Definition of done: cargo test --locked -p tls-record-core green
  (vector list in report); quick green incl. ungranted-import
  instantiation-failure needle; kernel LOC delta reported (additive,
  budget stated).
Report format: capability sentence; import list as implemented; vector
  coverage table; report filenames; LOC delta + payback plan reference.
```

STOP-tripwires: D1 unresolved; the secret-header mid-stream splice (§4)
proves unworkable against real TLS 1.3 record sequencing; any design
pressure to hand key bytes to the guest — STOP.

### M11-5 — Live provider record layer through the guest

Capability: the live provider connection's TLS record layer
(deframing/framing, AEAD via host imports) runs inside svc.net.tls
(promoted through the loop, granted exactly its §3 import set);
embedded-tls's record path is no longer on the live path; provider
behavior byte-identical per needles.

Handshake stays kernel-side (embedded-tls) this slice; the seam:
kernel completes the handshake, installs negotiated keys into the
session table, hands the guest the session handle + the open TCP conn
handle for application-data phase. Verify at execution time that
embedded-tls 0.17 exposes or can be patched (vendored) to export
negotiated traffic secrets to the session table — if not cleanly
possible, M11-5 and M11-6 merge into one slice (note in revalidation).

Verification: NEW focused profile `provider-service` (fixture TLS peer
is NOT feasible in-VM without a test server; instead the profile
proves: svc.net.tls promotion + grant needles, session-handle
lifecycle evidence, and negative proofs) —
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile provider-service -TimeoutSeconds 360 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10`
plus owner-run `vm-harness\openai-direct-smoke.ps1` with a key image
for the REAL end-to-end proof (this slice cannot be called done
without one green live smoke — honest requirement, needs owner).
Fail-closed: guest record-layer failure aborts the request with typed
evidence; kernel never falls back to embedded-tls record path (delete
the routing, keep handshake use only); foreign-handle probes denied.

```text
Packet id: M11-5-record-layer-live
Goal: Provider application-data records are framed/deframed by
  svc.net.tls via env.tls_aead_* host imports; kernel handshake
  installs keys into the session table; embedded-tls record path off
  the live path; one green live openai-direct smoke.
Read first: map §4/§6.5; M11-4 report + tls-record-core; openai.rs
  perform_https_request at HEAD; vendored embedded-tls record/split
  internals; M6 grant flow for svc.net.tls.
Allowed write set: seed-kernel/src/openai.rs + wasm_net_imports.rs +
  vendored embedded-tls patch (key export seam only, clearly marked);
  wasm-guests/svc-net-tls/**; vm-harness/shadow-vm-smoke-profile-
  provider-service.ps1 + needles.
Forbidden: secrets in guest memory; keeping a silent embedded-tls
  record fallback; touching openai_trust.rs verdict logic; weakening
  any trust check; committing key-bearing images.
Constraints: secret header via env.provider_write_secret_header only;
  needles from observed serial; re-sign if attested sources touched;
  scripts\scan-secrets.ps1 before commit.
Definition of done: provider-service profile green; quick green; ONE
  green live openai-direct smoke (owner-run, report filename cited);
  net kernel LOC delta reported.
Report format: capability sentence; seam description as built; report
  filenames incl. live smoke; LOC delta.
```

STOP-tripwires: embedded-tls cannot export traffic secrets without
invasive vendored surgery (>~200 changed vendored lines) — STOP, owner
chooses merge-with-M11-6 vs alternative; live smoke unavailable (no
key) — STOP, do not close on fixtures alone.

### M11-6 — Handshake in the guest; embedded-tls leaves the kernel

Capability: the full TLS 1.3 client (ClientHello build, handshake
message parsing, transcript, key-schedule driving via host imports)
runs in svc.net.tls; the embedded-tls dependency is REMOVED from the
kernel; certificate chain trust verdicts remain kernel-side
(`env.tls_verify_cert_chain` → pinned/M10-WebPKI verifier in
openai_trust.rs lineage); "the kernel does not parse the internet" is
now true for the provider path.

OWNER DECISION D2 — cert-verdict authority long-term: (a) keep the
verifier kernel-resident (RECOMMENDED for M11; small, trust-critical,
verdict must not come from the thing being sandboxed); (b) move
X.509/DER parsing to Wasm with kernel-side signature+pin math over
re-hashed extracted SPKI (attack-surface win but a verdict-authority
redesign — M12+ candidate, REQUIRES a new ADR); (c) dual-run both.
Proceed with (a).

Verification: host `cargo test --locked -p tls-record-core` (handshake
vectors added, RFC 8448 full-handshake transcript); provider-service
profile (handshake-phase evidence needles, malformed-handshake fixture
→ typed failure); quick; one green live openai-direct smoke (owner);
FULL checkpoint. Expected kernel shrink: embedded-tls out of
Cargo.lock/vendor for seed-kernel, tls_io.rs deleted or reduced to the
conn-handle pump, openai.rs handshake glue deleted — report exact
numbers. Fail-closed: handshake failure classes (bad cert verdict,
bad Finished, downgrade, timeout) each end as typed evidence + closed
connection; NoVerify development bypass either preserved EXACTLY as
the existing owner-gated path or removed — no new bypass.

```text
Packet id: M11-6-handshake-guest-embedded-tls-out
Goal: svc.net.tls performs the full TLS 1.3 client handshake via host
  crypto imports; env.tls_verify_cert_chain keeps trust verdicts in
  the kernel verifier; embedded-tls removed from the seed-kernel
  dependency graph; provider path byte-identical per needles + live
  smoke.
Read first: map §6.6 + D2 decision; M11-5 seam code; tls-record-core;
  openai_trust.rs at HEAD (M10 will have rewritten it); RFC 8446/8448.
Allowed write set: wasm-guests/svc-net-tls/**; seed-kernel/src/
  openai.rs, tls_io.rs (delete/shrink), wasm_net_imports.rs; seed-
  kernel Cargo.toml/lock + vendor removal; provider-service/quick/full
  needles.
Forbidden: moving pin/WebPKI verdict logic into the guest; new trust
  states; weakening the fail-closed handshake outcomes; adding a
  kernel TLS fallback.
Constraints: every handshake failure class has a needle; scan-secrets
  before commit; re-sign if attested sources touched; needles from
  observed serial only.
Definition of done: host handshake vectors green; provider-service +
  quick green; live smoke green (owner-run); FULL profile green
  (checkpoint); embedded-tls absent from seed-kernel deps; LOC ledger
  updated (expect the largest net deletion of M11).
Report format: capability sentence; deleted deps/files/functions; LOC
  ledger table; report filenames incl. live smoke + full.
```

STOP-tripwires: anything requiring D2 option (b) to make progress
(needs new ADR); any change to pin format/trust states (M10 territory);
temptation to keep embedded-tls "for recovery" (the recovery lifeline
must NOT depend on the rich provider path — ADR 0003; if someone
proposes TLS in the lifeline, STOP).

### M11-7 — Provider path as one promoted, persistent, replaceable service

Capability: `svc.provider.openai` — HTTP request construction (minus
secret header), TLS driving, response parsing composed — is a single
promoted, persisted, re-promotable, ROLLBACK-ABLE service; replacing
the provider adapter is now a promotion transaction, not a kernel
rebuild; the kernel keeps only: sockets/DNS, crypto/session imports,
secret splice, trust verdicts, evidence emission, UI glue.

Files: compose svc.net.httpparse + svc.net.tls + request-builder into
the provider service (verify at execution time whether one merged
artifact or a service-calls-service composition is possible — if M6/M7
have no inter-service call mechanism, MERGE into one artifact and note
that service composition is an M12+ direction item); openai.rs shrinks
to submit/poll/evidence/secret/trust glue. M10's second provider
adapter should be re-pointed at this shape if trivially possible;
otherwise note as follow-up, do not scope-creep.

Verification: provider-service + net-parser + quick green; live smoke
green (owner); FULL checkpoint before closing M11; rollback needle: a
deliberate un-promotion of svc.provider.openai returns the system to
fail-closed provider-absent state with typed evidence (nothing
auto-reloads without re-verified evidence per M7D).

```text
Packet id: M11-7-provider-as-service
Goal: One promoted persistent service owns the provider protocol path;
  kernel keeps sockets, crypto imports, secret splice, trust verdicts,
  evidence, UI; un-promote -> fail-closed provider-absent with
  evidence; re-promote after reboot works under M7D gates.
Read first: map §6.7; M11-2..-6 reports; openai.rs residual at HEAD;
  M6 promotion/rollback + M7D re-promotion flows at HEAD.
Allowed write set: wasm-guests/svc-provider-openai/** (or merged
  artifact per constraint); seed-kernel/src/openai.rs (+ method table);
  profile needles (provider-service, net-parser, quick, full).
Forbidden: secret bytes in guest; auto-load without re-verified
  evidence; deleting the fail-closed provider-absent behavior;
  touching recovery-lifeline code.
Constraints: if no inter-service call mechanism exists, build ONE
  merged artifact and record composition as an M12+ direction note;
  needles from observed serial; scan-secrets; re-sign if attested
  sources touched.
Definition of done: all four profiles green incl. FULL; live smoke
  green (owner); rollback/un-promote needle green; reboot re-promotion
  needle green; final M11 LOC ledger in the report.
Report format: capability sentence; final kernel-keeps list as built;
  LOC ledger (per-slice + total); all report filenames.
```

STOP-tripwires: inter-service composition needs new kernel mechanism
design (defer, merge artifacts instead); any pressure to auto-load the
provider service without evidence — STOP.

## 7. Next Extraction Candidates (direction only, NO slice plans)

1. **DNS message parsing** (`net.rs:579-735`) — small, bytes-in/
   address-out, same pattern as M11-2. Likely first post-M11 slice.
2. **X.509/DER + WebPKI chain parsing** — D2 option (b); real
   attack-surface win but verdict-authority redesign; REQUIRES an ADR.
3. **Console/UI text rendering** — the framebuffer console's text/
   layout logic as a service; the framebuffer DRIVER stays native.
4. **USB HID report parsing** — only when USB exists (verify: today
   input is the QEMU/PS2-style path; Surface Pro 4 hardware will force
   USB work in M12+ bring-up).
5. **Agent-protocol command parsing** — input from the trusted serial
   operator, lowest risk, lowest priority.
6. **smoltcp L2-L4** — recommended to STAY native (driver-adjacent,
   performance, fixed-shape headers); revisit only in the ADR 0003
   native-service-graph era.

## 8. What NEVER Leaves the Core

Boot/Limine handoff; physical memory management + allocator;
interrupts/exceptions; ALL drivers (e1000, AHCI, framebuffer, serial,
input) per ADR 0005; the wasmi interpreter itself + fuel/StoreLimits
(the isolation boundary cannot live inside what it isolates); the
capability ledger, grant evaluation, promotion/rollback transaction
authority; the durable write path (RAIOS_AUDITRB_V0, SEED_DATA);
attestation/signature verification + trust verdicts; the recovery
agent lifeline (ADR 0003 — must work when the service world is
broken; it never depends on any M11 service); secret storage and
splicing; entropy + time sources; the event log core.

## 9. Owner Decisions (recorded here so the orchestrator never guesses)

- **D1** (blocks M11-4): record session-handle/crypto-import/secrets
  design as (a) new short ADR [RECOMMENDED] / (b) ADR 0005 amendment /
  (c) map-only. Implementation may not start on (c) silently.
- **D2** (M11-6): cert-verdict authority — (a) kernel verifier
  [RECOMMENDED for M11] / (b) X.509-parse-in-Wasm + kernel signature
  math (M12+, needs ADR) / (c) dual-run.
- **D3** (M11-2): guest authorship — (a) in-repo source submitted
  through the runtime M6 path [RECOMMENDED] / (b) fully external
  AI-authored / (c) build-embedded [REJECTED].

## 10. Global STOP-Tripwires

Stop and ask the owner when: any slice needs a new ADR or trust-model
change (D1/D2 included); any design would put key/secret bytes in
guest memory; any destructive disk operation or any write to
`release/raios-stage0.img`; the recovery lifeline would gain a
dependency on any M11 service; a slice cannot show net kernel deletion
where this map promises one; the FULL profile goes red (Red Gate Rule:
repair only); live-smoke evidence is unavailable for M11-5/-6/-7
closure; M6/M7 interfaces at HEAD contradict this map's assumptions.

## 11. Verdict

M11 is 7 implementation slices plus mandatory revalidation. The easy,
high-value half is M11-1..-3 (parser out, first real kernel deletion,
loop consumed by infrastructure) — low risk, no new trust surface. The
hard half is M11-4..-6: a guest TLS 1.3 client against a host crypto
seam, with the embedded-tls key-export seam (M11-5) the most likely
place reality diverges from this map — the merge fallback is
pre-authorized there. Honest uncertainties: §2 performance numbers are
estimates until M11-1; the secret-header mid-stream splice (§4) is
designed but unproven; M6/M7/M10 outcomes will rewrite the exact
surfaces this map cites, which is exactly what Slice 0 is for. End
state: a kernel that boots, drives hardware, holds secrets and trust
verdicts, meters services — and does not parse the internet.
