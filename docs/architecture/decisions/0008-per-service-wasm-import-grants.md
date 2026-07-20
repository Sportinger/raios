# ADR 0008: Per-Service Wasm Import Grants

Date: 2026-07-08

## Status

Status: ACCEPTED by the raiOS owner (2026-07-08). Owner decision: **Option A**
(manifest-declared exact per-service import list, authorized by a pinned
raios-core scoped evaluator, enforced by a per-instance wasmi Linker) **plus the
TLS trust shape Option 2** (the TLS/certificate verifier implementation moves
into the Wasm service, but the permanent core keeps trust-label authority,
provider request/export authorization, and API-key custody — the service may
produce evidence, never bless itself or receive raw secrets). Implementation
proceeds slice by slice; each slice grants nothing until proven, honestly
labeled dev_key_not_owner_sealed, never owner_sealed.

(Historical: this ADR was PROPOSED — pending owner decision — until the owner
accepted A + 2 on 2026-07-08.)

## Context

M11 wants the kernel to stop parsing the internet: TLS/HTTP protocol logic
should move into replaceable, capability-scoped Wasm services, and the kernel
should measurably shrink (`docs/_archive/2026-07-18_ROADMAP.md:959-962`,
`README.md:258-259`, `README.md:315`).

ADR 0005 already decided the core isolation rule: the first real service
boundary is the in-kernel Wasm interpreter, and "the capability envelope of a
service becomes its host-function import surface" (`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:42-54`).
It also says drivers and performance-critical paths stay native for now
(`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:58-60`)
and that the Wasm import surface is the concrete enforcement target
(`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:103-106`).

HEAD does not yet express that rule per service:

- The current module grant authority checks retained evidence and a signed
  attestation against the computed grant, manifest, artifact, VM report, and
  local-attestation hashes. It does not authorize an import list
  (`seed-kernel/src/agent_protocol_module_grant.rs:433-471`).
- The granted candidate service turns that artifact-level grant plus retained
  Wasm validity into `can_execute`, then calls the shared runtime entry point
  (`seed-kernel/src/granted_candidate_service.rs:940-963`,
  `seed-kernel/src/granted_candidate_service.rs:570-598`).
- The runtime validates bytes with `wasmi::Module::new`, creates a metered
  store, constructs a `Linker`, and calls `define_capability_envelope`
  (`seed-kernel/src/wasm_runtime.rs:281-284`,
  `seed-kernel/src/wasm_runtime.rs:321-343`).
- That envelope currently links the same two imports for every module:
  `env.log` and `env.counter_get`
  (`seed-kernel/src/wasm_runtime.rs:680-685`).
- The negative test proves a forbidden import fails when the global linker does
  not define it, but the allowed set is still global, not per-service
  (`seed-kernel/src/wasm_runtime.rs:412-460`).

The M11-1 kernel-surface baseline is pure measurement and grants no authority;
it records the current service-candidate internet parsing surface, including
`openai.rs`, `tls_io.rs`, DNS parsing in `net.rs`, and the vendored TLS tree
(`crates/raios-core/src/kernel_surface.rs:1-4`,
`crates/raios-core/src/kernel_surface.rs:41-84`). That gives M11 a measuring stick,
not an import-grant mechanism.

ADR 0001 requires a small raiOS-native protocol whose actions are explicit,
capability-gated, logged, denied, and replayable (`docs/architecture/decisions/0001-raios-agent-protocol.md:14-27`).
ADR 0004 says authoritative system memory is typed, evidence-bound,
capability-gated, and traceable, including capability grants/denials and test
evidence (`docs/architecture/decisions/0004-system-memory-and-agent-context.md:11-25`,
`docs/architecture/decisions/0004-system-memory-and-agent-context.md:37-40`,
`docs/architecture/decisions/0004-system-memory-and-agent-context.md:82-96`).

Therefore M11 substantive work is blocked until raiOS can authorize and enforce
a specific import surface per running Wasm service. Without that, a TLS parser
service cannot be honestly distinguished from an `env.log`-only parser service
or from a later service that needs `net.*`, `tls_record.*`, `crypto.*`,
`time.*`, or `secret.*` host functions.

## Decision Drivers

- Fail closed: absent or mismatched import-surface evidence grants no imports.
  The default linker for a service is empty, and imports are added only after a
  scoped evaluator authorizes them.
- Honest labeling: raiOS must never report that a service has an import it
  cannot actually call, or that it lacks an import that was actually linked.
  The audited grant and the per-instance linker must be the same list.
- Mechanism before vocabulary: the next slice should make an enforced import
  boundary real, not add another non-authorizing schema-only layer. Future
  records should use the typed record model, not hand-written emit/hash code.
- Auditability: the granted import surface should become durable evidence
  alongside the module/artifact/test evidence chain, consistent with ADR 0004's
  evidence-bound memory model.
- Minimal permanent-core growth: the permanent core should keep Wasm
  instantiation, host-function dispatch, service registry hooks, trust gates,
  and secret custody, but not grow a large TLS/HTTP parser if M11 can move that
  parser into a service.
- Separate scoped evaluator discipline: per-service import grants should be
  evaluated by their own pinned `raios-core` evaluator with pairwise-unique
  denial reasons, like the existing scoped provider export and memory append
  gates (`crates/raios-core/src/scoped_provider_export.rs:1-6`,
  `crates/raios-core/src/scoped_provider_export.rs:64-132`,
  `crates/raios-core/src/scoped_memory_record_append.rs:1-19`).

## Considered Options

### Option A: Manifest-declared import list, authorized by a new scoped evaluator

The service manifest or load descriptor declares an ordered import list such as
`env.log`, later `net.tcp_open`, `tls_record.read`, `crypto.ecdsa_verify`,
`time.now_untrusted`, or `secret.write_authorization_header`. A new
`raios-core` evaluator, tentatively
`scoped_wasm_import_grant`, authorizes the exact list for one service/artifact
grant after checking service id, artifact hash, computed grant hash, VM report
hash, local attestation, trust tier, and allowed import policy. The kernel then
constructs a fresh wasmi `Linker` per instance and defines only the authorized
host functions for that service.

Default-deny story: missing import-surface evidence, an unknown import, a
duplicate import, a mismatch between manifest and computed grant, a missing host
implementation, or an attempt to link a broader list denies the load before
instantiation.

Enforcement point in current code: replace the current global
`define_capability_envelope(&mut linker)` call in the instance path with a
per-service `define_granted_imports(&mut linker, granted_imports)` step
(`seed-kernel/src/wasm_runtime.rs:331-343`,
`seed-kernel/src/wasm_runtime.rs:680-685`). The existing missing-definition
behavior remains the last physical boundary for undeclared imports
(`seed-kernel/src/wasm_runtime.rs:450-460`).

Auditability: high. The same canonical import list can be hashed into the
computed grant and later written as a durable memory/audit fact. The runtime can
emit the actual linked list derived from the same data, so drift is checkable.

Effort: medium. It needs a small record-model type, a host-tested scoped
evaluator, a service descriptor/manifest field, and one runtime threading change.

Risk: medium. The main risk is drift between the evaluator's list and the
linker construction. Make that impossible by passing the evaluator output into
the linker builder directly and denying if any requested import lacks a host
implementation.

### Option B: Capability-tier-derived fixed import bundles

Keep manifests coarse. A service gets a tier such as `parser_log_only`,
`network_client`, or `provider_tls_client`, and the kernel maps that tier to a
fixed import bundle.

Default-deny story: unknown tier or missing evidence denies. Known tiers link a
predefined bundle.

Enforcement point in current code: the same per-instance linker construction
changes, but the input is a tier-to-import mapping rather than a manifest list
(`seed-kernel/src/wasm_runtime.rs:331-343`,
`seed-kernel/src/wasm_runtime.rs:680-685`).

Auditability: medium. The tier is compact, but a later reader must know which
bundle version the tier meant at the time of the grant. Bundle-version drift can
make old evidence ambiguous unless bundle hashes are recorded.

Effort: low to medium. It avoids per-service list validation, but it still needs
bundle hashing, versioning, and linker enforcement.

Risk: medium to high. Bundles tend to grow. A TLS service may need crypto/time
imports that a simpler parser service must never get. Coarse bundles push raiOS
away from ADR 0005's exact import-surface boundary.

### Option C: Host-function capability table requested per call

Link a generic dispatcher import such as `raios.host_call(capability_id, args)`.
The module requests each host function at runtime, and the host checks a
capability table on every call.

Default-deny story: unknown capability id, missing runtime grant, bad arguments,
or denied service id returns a trap or `capability_denied`.

Enforcement point in current code: instead of defining many imports in
`define_capability_envelope`, the runtime would define one dispatcher and move
most enforcement into its handler (`seed-kernel/src/wasm_runtime.rs:680-685`).

Auditability: high for call logs, lower for physical import boundaries. The
service can always import the dispatcher, so the import surface no longer names
real authorities; the per-call table must carry the authority proof.

Effort: high. It needs an ABI, argument decoding, per-call checking, and a
larger permanent dispatcher surface.

Risk: high. This recreates a syscall layer inside one Wasm import and weakens
the ADR 0005 invariant that the import surface is the capability envelope.

## Decision

Author recommendation for the owner to accept or reject: choose **Option A**.

Per-service Wasm imports should be declared as an exact import list, authorized
by a new pinned `raios-core` scoped evaluator, and enforced by constructing each
wasmi instance's `Linker` with only that list. The evaluator output, not a
second hand-written table, should be the input to linker construction.

The initial accepted list can stay tiny: existing demo services should remain
`env.log` plus `env.counter_get`, and a future parser-only service may be
`env.log` only. A later TLS service can request a larger, explicit list such as
`net.*`, `tls_record.*`, `crypto.*`, `time.*`, and opaque `secret.*` imports
only after the owner accepts this ADR and the corresponding service policy.

The fail-closed enforcement point is before instantiation: if the authorized
list is absent, broader than policy, not byte-identical to the grant evidence,
or cannot be fully linked, the kernel must not instantiate the module. If a
module imports something outside the list, wasmi's missing-definition path
should continue to fail at link time.

This proposal does not grant any new import, change the current runtime, or
change the current module grant. It only names the intended architecture.

## Coupled Sub-Decision: TLS Verifier Trust Shape

M11 cannot be decided only as an import-list issue. Moving TLS/HTTP out of the
kernel also forces an owner decision about where certificate verification lives.

Current reality:

- The direct OpenAI path still performs TLS/HTTP in `openai.rs`, using either a
  development bypass or `OpenAiPinnedCertVerifier` during the TLS open
  (`seed-kernel/src/openai.rs:1150-1186`).
- After handshake, the path still denies before API-key copy if trust does not
  authorize provider requests (`seed-kernel/src/openai.rs:1196-1203`).
- The API key is copied into a stack buffer and written into the HTTPS request
  only after positive trust/binding steps, then the buffer contents are written
  to TLS without logging the key (`seed-kernel/src/openai.rs:1228-1253`).
- M10 explicitly says trusted time, live certificate validity parsing, real
  certificate-chain validation, and a live second provider remain
  owner/production-gated; until then trust labels must stay
  `unverified/not_validated` (`docs/_archive/2026-07-18_ROADMAP.md:34-39`).
- The M10 honesty evaluator denies development bypass, chain/time overclaims,
  and WebPKI overclaims, and even its honest pin-only positive result grants no
  provider request/export authority by itself
  (`crates/raios-core/src/scoped_provider_trust_honesty.rs:74-105`).

Two trust-shape options follow.

### TLS Option 1: Kernel-side verifier, Wasm-side TLS/HTTP parser

The Wasm service handles HTTP framing and possibly TLS record flow, but the
kernel keeps certificate verification and trust-label authority. The service
passes certificate/transcript evidence to a host import, and the kernel decides
whether trust is sufficient.

Honesty advantage: the current M10 fail-closed posture is easiest to preserve,
because the kernel keeps the trust label and secret-release gate. API keys stay
host-side and opaque.

Cost: if full WebPKI/X.509 parsing lands in the kernel, M11 only partly
succeeds. The kernel would still parse one of the most complicated internet
formats.

### TLS Option 2: Guest-side verifier implementation, kernel-side trust authority

The Wasm TLS service implements TLS record parsing, certificate parsing, chain
building, and verifier evidence production. The kernel provides only narrow
imports: crypto primitives, trust-anchor lookup, time evidence lookup, network
record I/O, and opaque secret-send operations. The kernel does not accept a
service's self-asserted "verified" label; it authorizes provider request/export
only after a scoped raiOS-core trust evaluator checks the evidence shape.

Honesty advantage: this best satisfies "the kernel does not parse the
internet." No-WebPKI/no-trusted-time honesty survives because the kernel refuses
to convert guest evidence into positive provider authority until the scoped M10
conditions exist.

Cost: the import surface is larger and more sensitive. The service must not get
raw API-key access. It gets an opaque host-side operation such as "append this
Authorization header for provider X to the already-authorized encrypted request"
only after the kernel trust gate authorizes it.

Author recommendation for the owner to accept or reject: choose **TLS Option 2
with kernel-side trust authority and secret custody**.

That means the parser/verifier code that grows large moves into a replaceable
Wasm service, but the permanent core remains the authority for trust labels,
provider request/export authorization, and API-key release. The Wasm service
may produce evidence; it may not bless itself or receive raw secrets.

## Consequences

If the owner accepts Option A, M11-2 through M11-7 can become real runtime work
instead of another measurement-only chain:

- define the import-list record shape in the typed model;
- add a host-tested `raios-core` scoped evaluator for per-service imports;
- thread the evaluator output into per-instance wasmi linker construction;
- prove existing demo services still run with exactly their current imports;
- add a negative service that imports an ungranted function and fails before
  execution;
- split out an `env.log`-only parser service as the first narrow M11 service;
- later grant a TLS service only the explicit network/crypto/time/secret imports
  the owner approves.

What remains owner/production-gated:

- trusted time;
- live X.509 validity parsing;
- real certificate-chain validation with roots/intermediates;
- live second-provider trust evidence;
- any positive WebPKI label;
- any raw secret exposure to a Wasm service;
- any claim that TLS/HTTP relocation is complete before the service both runs
  and the kernel-surface baseline shrinks.

## Open Questions

1. Does the owner accept Option A as the M11 import-grant architecture?
2. Should import grants be named by exact `module.name` pairs only, or by exact
   pairs plus a stable import ABI version?
3. Which imports are allowed for the first non-demo M11 service: `env.log` only,
   or also a read-only byte-buffer/input import?
4. Does the owner accept the TLS trust recommendation: verifier implementation
   in Wasm, trust authority and secret custody in the kernel?
5. If the TLS verifier moves into Wasm, which crypto primitives remain native
   host imports versus compiled into the service?
6. What is the first acceptable time evidence for TLS service work:
   current `cmos_rtc_unverified` evidence only, or no time-bearing positive
   labels until a production trusted-time source exists?
7. How should opaque API-key use be expressed: host-side header append,
   sealed-provider request handle, or another secret-service import?
8. What owner approval is required before granting any import broader than
   `env.log` and `env.counter_get`?

This ADR is a PROPOSAL. No code has been changed. The owner decides; until then
M11 substantive work (relocating TLS/HTTP) remains a labeled TODO.
