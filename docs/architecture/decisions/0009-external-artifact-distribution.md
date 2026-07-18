# ADR 0009: External Artifact Distribution

## Status

Status: ACCEPTED by the raiOS owner (2026-07-08). Owner decision: **Option A** —
a LOCAL signed registry / fake-cloud feeding external artifacts into the EXISTING
serial candidate-intake channel (no network), re-verified through the unchanged
M6 gate + M7 persistence re-verify-from-disk + signed artifact-identity chain,
audited as M9 durable memory. Hard invariant preserved: download = candidate
intake, NEVER install; a distribution signature is provenance, not
load-worthiness. Real network OTA (Option B) stays deferred/gated. Implementation
proceeds slice by slice, grants-nothing-until-proven, honest dev_key_not_owner_
sealed.

(Historical: this ADR was PROPOSED — pending owner decision — until the owner
accepted Option A on 2026-07-08.)

## Context

M12+ explicitly names external artifact distribution as future direction, not a
slice plan: "external artifact distribution (unparking `ota/`/`registry/`/
`fake-cloud/` - requires a new ADR)" (`docs/_archive/2026-07-18_ROADMAP.md:980-985`). The same
roadmap forbids work in those directories without that ADR
(`docs/_archive/2026-07-18_ROADMAP.md:1094-1095`).

The M12+ direction document says what this item means today. Current artifacts
are repo-local or delivered through the serial harness; the external
distribution item would let one user's promoted, evidence-carrying module be
published and another raiOS receive it as a candidate
(`docs/_archive/2026-07-18_m12-plus-direction-2026-07-06.md:111-123`). The same
document sets the hard framing: "download = candidate intake, NEVER install";
network bytes enter the M6 pipeline as inert bytes, and a distribution
signature is provenance, not load-worthiness
(`docs/_archive/2026-07-18_m12-plus-direction-2026-07-06.md:130-135`).

What exists at HEAD:

- `ota/` is a host-side OTA/module signing tool. Its README describes
  deterministic key generation, sign/verify commands, Ed25519 signatures, and
  BLAKE3 payload hashes (`ota/README.md:3-13`). Its tests exercise generated
  keypairs and a sign/verify round trip (`ota/README.md:17-21`).
- `registry/` is a host-side content-addressed registry. It stores blobs,
  manifests, optional evidence, and index records
  (`registry/README.md:1-15`). Its own README says host evidence is not kernel
  load approval and Stage-0 still denies module loading until guest policy,
  grants, audit records, and rollback checks exist
  (`registry/README.md:17-29`). Its grant and audit/rollback diagnostics remain
  non-authorizing (`registry/README.md:32-46`).
- `fake-cloud/` is a deterministic WebSocket test control-plane stub. It
  validates signed module/OTA payloads against an offline root key and can push
  verified artifacts into the local registry (`fake-cloud/README.md:3-16`).
  Its integration test streams a dummy signed module and checks publication
  into the registry (`fake-cloud/README.md:21-24`).
- ADR 0005 already classified the lane as parked, not deleted: the host crates
  were frozen since 2026-05 and never connected to the kernel
  (`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:86-91`).
  It also says there is no resumption of `ota/registry/fake-cloud` without a
  new ADR (`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:118-124`).

The device already has one real external-candidate intake surface, but it is
not a network fetcher. `module_candidate_channel.rs` accepts base64 chunks,
clears pending state on malformed chunks, caps delivery at
`MAX_EXTERNAL_WASM_CANDIDATE_BYTES`, and sends finalized bytes only to
`intake_external_wasm_candidate` (`seed-kernel/src/module_candidate_channel.rs:67-102`,
`seed-kernel/src/module_candidate_channel.rs:113-134`). The intake path is
bounded at 256 KiB, labels the channel `serial_console_base64_chunks_v0`, and
returns an inert current-boot candidate with load, execution, and persistence
all false (`seed-kernel/src/module_candidate_intake.rs:9-14`,
`seed-kernel/src/module_candidate_intake.rs:52-67`,
`seed-kernel/src/module_candidate_intake.rs:118-126`). Project status confirms
that this serial path has no reachable load, grant, instantiate, execute, or
persist sink (`docs/_archive/2026-07-18_PROJECT_STATUS_history.md:2688-2706`).

Any external distribution design must preserve the existing safety invariants:

- **M6 promotion gate.** The live grant path flips only when the evidence check
  is valid, the local attestation is signature-verified, and the attestation's
  computed-grant, manifest, artifact, VM-report, and local-attestation hashes
  match the grant reference (`seed-kernel/src/agent_protocol_module_grant.rs:433-471`).
  The trust tier is honestly `dev_key_not_owner_sealed`, not owner-sealed
  (`seed-kernel/src/agent_protocol_module_grant.rs:449-455`;
  `docs/_archive/2026-07-18_PROJECT_STATUS_history.md:2747-2756`).
- **Signed artifact identity chain.** Current guest artifacts are build-bound
  through P-256 descriptor signatures: build.rs reads the artifact descriptor,
  public key, and DER signature, verifies the descriptor and load descriptor,
  and embeds the Wasm artifact bytes
  (`seed-kernel/build.rs:405-431`, `seed-kernel/build.rs:594-599`,
  `seed-kernel/build.rs:673-676`). The artifact identity and load descriptor
  envelopes explicitly do not authorize external load or persistent install
  (`seed-kernel/build.rs:488-501`, `seed-kernel/build.rs:574-587`). External
  distribution metadata may not replace this raiOS artifact-identity check.
- **M7 persistence and boot-2 re-verification.** The persistent artifact store
  writes a `raios.artifact_persist.v0` record only after a verified promotion
  transaction, and the stored blob remains inert until re-verified
  (`docs/_archive/2026-07-18_ROADMAP.md:276-286`). The two-boot proof recomputes the blob hash,
  recomputes the attestation hash, re-runs signature verification, and reaches
  execution only through the same M6 load/start gate, never because a stored
  boolean said "OK" (`docs/_archive/2026-07-18_ROADMAP.md:287-295`). The code path also denies
  missing retained bytes, invalid Wasm, hash mismatch, missing readback, and
  scoped append failures before reporting persistence
  (`seed-kernel/src/artifact_store.rs:207-246`,
  `seed-kernel/src/artifact_store.rs:376-408`,
  `seed-kernel/src/artifact_store.rs:708-737`).
- **M8 recovery restore-only behavior.** Recovery load by hash selects an
  artifact from the local M7D store, re-verifies the full M6 chain, and never
  fetches or accepts new bytes or a URL (`docs/_archive/2026-07-18_ROADMAP.md:140-153`;
  `docs/_archive/2026-07-18_PROJECT_STATUS_history.md:3441-3453`). External distribution must not turn
  recovery into a network intake path.
- **M9 memory and provider export.** ADR 0004 says authoritative memory is
  typed, evidence-bound, and capability-gated
  (`docs/architecture/decisions/0004-system-memory-and-agent-context.md:37-41`);
  public/local_only/secret classes gate provider context, and secrets never
  appear in provider context or durable plaintext
  (`docs/architecture/decisions/0004-system-memory-and-agent-context.md:143-151`).
  Records that affect capabilities, provider export, persistence, rollback, or
  trust must be evidence-bound
  (`docs/architecture/decisions/0004-system-memory-and-agent-context.md:288-300`).
  M9C-2's real provider export path is still disabled for ordinary dispatch:
  real `provider.context_export` remains `capability_denied`, provider writes
  are `not_attempted`, and the positive authority flip is selftest-only with
  no transmission (`docs/_archive/2026-07-18_PROJECT_STATUS_history.md:3755-3766`,
  `docs/_archive/2026-07-18_PROJECT_STATUS_history.md:3792-3813`; `docs/_archive/2026-07-18_ROADMAP.md:54-63`).
- **M11 import grants.** If any distribution client runs inside raiOS, it must
  run under the per-service Wasm import-grant boundary. ADR 0008 recommends an
  exact import list, authorized by a scoped evaluator and enforced by building
  each wasmi `Linker` from only that list
  (`docs/architecture/decisions/0008-per-service-wasm-import-grants.md:179-190`).
  Current M11 progress keeps non-`env` imports and per-service secret custody
  owner-gated (`docs/_archive/2026-07-18_ROADMAP.md:28-38`).

This ADR is required because external distribution introduces an external trust
surface and, for any real OTA fetch, a network fetch surface. The direction doc
requires the ADR to decide transport, signing authority, candidate staging,
revive-vs-rewrite of the parked lane, and fail-closed rules
(`docs/_archive/2026-07-18_m12-plus-direction-2026-07-06.md:144-159`). It also names
the conceptual risk: a shortcut that lets network bytes skip a gate
(`docs/_archive/2026-07-18_m12-plus-direction-2026-07-06.md:161-165`).

## Decision Drivers

- Fail closed: missing manifest request, missing artifact identity, unknown
  source, malformed package metadata, quota overflow, hash mismatch, failed
  distribution signature, missing local evidence, or M6/M7 re-verification
  mismatch denies before load or durable authority.
- Honest labeling: distribution can produce only provenance evidence until the
  local chain grants something. Current positive local grants remain
  `dev_key_not_owner_sealed`, never `owner_sealed`
  (`seed-kernel/src/agent_protocol_module_grant.rs:449-455`;
  `docs/_archive/2026-07-18_ROADMAP.md:61-63`).
- Re-verify from source evidence: never trust a registry index, distribution
  manifest, or stored "already verified" flag as load authority. M7 already
  requires re-running hashes and signature verification from disk on boot 2
  (`docs/_archive/2026-07-18_ROADMAP.md:287-295`).
- Preserve the signed identity chain: parked Ed25519/BLAKE3 metadata may be
  provenance, but the raiOS P-256 descriptor/artifact identity chain and M6
  local evidence chain remain the load boundary
  (`seed-kernel/build.rs:488-501`; `registry/README.md:24-37`).
- Auditability: fetch, verify, publish, denial, and promotion events should
  become typed, evidence-bound memory facts using the M9 model, with
  public/local_only/secret classification and no secret-durable payloads
  (`docs/architecture/decisions/0004-system-memory-and-agent-context.md:37-41`,
  `docs/architecture/decisions/0004-system-memory-and-agent-context.md:143-151`).
- Minimal permanent-core growth: ADR 0005 says service capability is the Wasm
  host-function import surface, and drivers/performance paths stay native for
  now (`docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md:44-60`).
  A future network distribution client should be a scoped service, not another
  broad kernel parser, once M10/M11 prerequisites are owner-approved.
- Provider/export firewall: if a design sends artifact bytes or metadata
  off-machine, it must go through the M9C provider/export gate. Current real
  export remains denied; the only positive export proof is test-only and does
  not transmit (`docs/_archive/2026-07-18_PROJECT_STATUS_history.md:3792-3813`).
- Owner-key sealing stays final: the roadmap carries real provider
  transmission and the owner-key sealing ceremony as production/final M12+
  work (`docs/_archive/2026-07-18_ROADMAP.md:61-63`).

## Considered Options

### Option A: Local signed registry/fake-cloud feeding the existing serial candidate channel

The first distribution source is local to the developer or owner workstation.
`ota-tools` signs module/package metadata, `registry-tools` stores the blob and
evidence in the CAS layout, and `fake-cloud` may be used only as a local test
publisher. raiOS receives the selected artifact through the existing
`module.submit_candidate_chunk` / `module.submit_candidate_finalize` serial
delivery surface.

Trust model: the distribution signature and registry index prove provenance and
content addressing only. The receiving machine still treats the bytes as inert
candidate bytes, recomputes hashes in guest, requires the raiOS
artifact-identity descriptor/signature chain, runs Shadow VM evidence, requires
local attestation, applies the M6 grant gate, and persists through M7 only after
the existing promotion transaction and artifact-store checks pass.

What is buildable now: the host signing/registry/fake-cloud tools already
exist (`ota/README.md:3-13`; `registry/README.md:1-15`;
`fake-cloud/README.md:3-16`), and the serial candidate channel is the existing
bounded intake path (`seed-kernel/src/module_candidate_channel.rs:67-102`).
This adds no network fetcher, no new import grant, and no new authority by
itself.

Tradeoff: this is not autonomous OTA. A host/operator still selects and sends
the artifact. That is acceptable for the first honest step because it exercises
distribution provenance and candidate intake without adding network trust.

### Option B: Real network OTA pull from a static content-addressed registry

raiOS fetches a manifest and artifact blob over HTTPS from a static
content-addressed registry. The registry may be the owner's local LAN host
first, later a public endpoint. The fetcher is a scoped Wasm service once the
owner accepts the required M10/M11 trust and import decisions.

Trust model: HTTPS protects transport only after real provider/network trust is
accepted; registry signatures are provenance only; local P-256 artifact
identity plus the M6/M7 chain still decide load and persistence. The fetcher
gets only the imports the owner grants under ADR 0008.

What is buildable now: not as a positive network feature. The direction doc
requires M10 closed for WebPKI/trusted time and prefers M11 so the download
client is a Wasm service, not more kernel-resident internet parsing
(`docs/_archive/2026-07-18_m12-plus-direction-2026-07-06.md:137-142`). The current
roadmap still carries M10 production trust and M11 beyond-`env` imports as
owner/production-gated (`docs/_archive/2026-07-18_ROADMAP.md:28-38`, `docs/_archive/2026-07-18_ROADMAP.md:61-63`).

Tradeoff: this is the real product shape for module sharing, but it is the
largest attack-surface expansion and should not be the first implementation
while network trust and service imports are still owner-gated.

### Option C: Source-agnostic content-addressed artifact acquisition

The architecture defines a single artifact-acquisition contract: an owner-
approved request names an expected artifact identity, size bound, and content
hash. Bytes may arrive from serial, USB, LAN registry, HTTPS registry, or a
future re-binding bundle. The source is recorded as provenance, but source type
never grants load authority.

Trust model: content address and distribution signature are evidence fields.
The local M6/M7 chain is the authority. A registry index is a locator; a
downloaded blob is candidate input; the local artifact identity and evidence
chain decide whether anything can run.

What is buildable now: the shape can be adopted as the invariant behind Option
A. The current registry is already content-addressed by BLAKE3
(`registry/README.md:7-15`), while raiOS's local gates use SHA-256 evidence and
P-256 identity. A future design map must reconcile those hashes explicitly
instead of letting one replace the other.

Tradeoff: this keeps transport from becoming authority and lets local serial
delivery, static registry pull, and future bundle import share one trust model.
It does not by itself answer discovery, publishing-key custody, rate limits, or
network fetch scheduling.

## Decision

Author recommendation for the owner to accept or reject: choose **Option A as
the first M12+ external-distribution step**, with **Option C as the durable
contract**. Defer Option B until the owner explicitly opens the network fetch
work and accepts the M10/M11 trust/import prerequisites.

Concretely:

- The first accepted source should be a local signed registry/fake-cloud lane
  that emits provenance and content-addressed metadata, then feeds bytes into
  the existing serial candidate channel. It grants no load authority and adds
  no network fetcher.
- Distribution signatures remain provenance only. They may never substitute
  for raiOS artifact identity, local attestation, the M6 grant gate, M7
  persistence re-verification, or recovery's local-store restore invariant.
- The receiving machine must re-run the same local evidence path: candidate
  bytes -> hash -> signed artifact identity -> Shadow VM report -> local
  attestation -> computed grant -> M6 promotion gate -> M7 artifact store and
  boot-2 re-verification.
- Any later network pull must be owner/production-gated. The fetcher should be
  a Wasm service with per-service import grants, and real transmission/export
  must pass the M9 provider/export firewall.
- The owner-key sealing ceremony is untouched. Until that ceremony happens,
  successful local grants remain honestly labeled `dev_key_not_owner_sealed`.

This proposal does not authorize a fetcher, a registry client, a new signing
authority, a new host import, a new provider export path, external unsigned
artifact intake, persistent install by distribution metadata, or owner-sealed
promotion.

## Consequences

If the owner accepts this ADR, M12+ design-map work can become concrete without
granting new authority:

- inventory the parked `ota/`, `registry/`, and `fake-cloud` formats against
  the current typed record model;
- decide which host-side metadata is retained as provenance and which stale
  evidence formats are retired;
- define a local signed registry packet that includes the raiOS artifact
  identity evidence required by the receiver;
- build a local host-to-serial distribution harness that uses the existing
  candidate chunk/finalize path;
- emit typed, classified, evidence-bound memory/audit facts for publish,
  receive, verify, deny, and promote outcomes;
- prove malformed package metadata, bad distribution signatures, hash
  mismatch, missing raiOS artifact identity, quota overflow, M6 mismatch, SAFE
  posture, and tampered M7 records all fail closed.

What stays owner/production-gated:

- real network OTA fetch;
- any public registry endpoint;
- publishing-key custody and rotation policy;
- any non-`env` host import needed by a download service;
- any raw secret exposure or provider/API-key use by a distribution service;
- any real off-machine export or transmission of local artifacts/evidence;
- any owner-sealed trust label or owner-key ceremony.

## Coupled Sub-Decisions

1. **Source and transport.** The first source should be local signed registry
   over serial, static HTTPS registry pull, fake-cloud WebSocket push, or a
   source-agnostic CAS contract with one local implementation.
2. **Distribution signing authority.** Decide whether parked Ed25519/BLAKE3
   keys remain provenance-only, whether raiOS artifact identity requires a
   separate P-256 descriptor signature, and who owns publishing-key custody and
   rotation.
3. **Candidate staging.** Decide where inert received bytes stage before M6:
   current-boot RAM only, M7D ARTSTOR as non-authorizing candidate material, or
   both with separate quotas.
4. **Request policy.** Decide whether a download requires a prior
   owner-approved manifest request, exact content hash, size bound, namespace,
   and rate/quota limits.
5. **Fetcher ownership.** Decide whether the first fetcher is host-side test
   infrastructure over serial or an in-guest Wasm service once M10/M11 are
   accepted.
6. **Audit/memory shape.** Decide which existing M9 record kinds represent
   receive/verify/deny/promotion facts, and whether any new typed record-model
   entries are needed after the design map.
7. **Provider/export interaction.** Decide whether publishing artifacts or
   evidence off-machine is routed through the M9C provider/export gate, and
   what destination policies are allowed.
8. **Sealing ceremony relationship.** Decide which, if any, distribution
   events are prerequisites for the final owner-key sealing ceremony. This ADR
   recommends none.

## Open Questions

1. Does the owner accept local signed registry/fake-cloud over the existing
   serial channel as the first external-distribution step?
2. Should fake-cloud be revived at all, or should the first source be a static
   filesystem/HTTP content-addressed registry?
3. Which signing keys are distribution provenance keys, and which keys are
   raiOS artifact-identity or promotion-authority keys?
4. Does every external artifact need a raiOS P-256 artifact-identity descriptor
   before the receiver accepts it as an M6 candidate?
5. Is real network fetch in scope now, or deferred until M10 production trust
   and M11 beyond-`env` imports are owner-accepted?
6. Where should inert received bytes live before promotion: RAM only, M7D
   ARTSTOR as non-authorizing candidate material, or another bounded staging
   area?
7. What exact owner approval is required before a requested artifact can be
   fetched or accepted?
8. Which typed M9 memory facts must be written for receive, verify, deny,
   publish, promote, rollback, and re-verify outcomes?
9. If artifact or evidence bytes are published off-machine, what export gate,
   redaction, classification, and audit records are required?
10. What relationship should distribution have to the final owner-key sealing
    ceremony? This proposal recommends that it has none beyond producing
    auditable provenance and local evidence.

This ADR is a PROPOSAL. No code has been changed. The owner decides; until then
M12+ external distribution remains a labeled TODO. The owner-key sealing
ceremony remains the final step.
