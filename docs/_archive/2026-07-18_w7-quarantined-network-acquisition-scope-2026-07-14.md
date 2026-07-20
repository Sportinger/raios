# W7 Scope: Quarantined Network Acquisition (2026-07-14)

Packet: `W7-SCOPE`
Status: read-only recon and design; no implementation authority is granted by
this document.

## Capability sentence

After W7, a user or agent can ask raiOS to fetch one exact, owner-approved
content hash from a pinned HTTPS source over raiOS's own QEMU e1000/DHCP/TCP
stack and retain the verified bytes as a `local_only`, `current_boot`, inert
quarantined candidate for inspection or discard, without building, loading,
executing, installing, or persisting them.

## Recon findings and constraints

W7 is the network-transport sibling of existing machinery, not a new artifact
pipeline.

- `raios-core/src/distribution_registry.rs` already owns bounded registry
  entries, a four-chunk delivery, per-chunk SHA-256 checks, ordered
  reassembly, whole-object SHA-256 verification, provenance verification and
  explicit no-install/no-load/no-execute/no-persist results.
- `seed-kernel/src/agent_protocol_registry.rs` already owns the current-boot
  catalog, receiver-identity evidence, guest P-256 verification, catalog begin,
  serial chunk/finalize transport and the receiver-identity load preflight.
  Its successful finalize is the only path W7 should use to reach
  `module_candidate_intake::intake_and_retain_external_wasm_candidate`.
- The current serial implementation couples parsing and transport state in
  `PendingSerialDistribution`, but `finalize_pending_distribution` already
  delegates the security checks to the shared raios-core delivery type. W7
  should extract only the small transport-neutral byte-accept/finalize seam;
  it must not clone the verifier or stage directly into
  `module_candidate_intake`.
- The candidate cap is 256 KiB and the current M12+ logical chunk cap is four.
  W7 should reuse both limits. A canonical 64-KiB network chunking rule yields
  at most four chunks and avoids buffering a complete HTTP response before
  verification.
- `seed-kernel/src/net.rs` already proves e1000, DHCP, DNS, one TCP socket,
  bounded TCP send/receive and abort. `seed-kernel/src/tls_io.rs` supplies the
  blocking embedded-io stream, while `seed-kernel/src/openai.rs` proves TLS
  1.3, pinned P-256 SPKI/certificate verification, bounded HTTP response
  handling and explicit timeouts. The TCP socket is singleton state, so W7 and
  OpenAI must have explicit mutual exclusion rather than inspecting each
  other's status by convention.
- The W1 project store is a durable content-addressed blob/revision store with
  manifest-last visibility and verified replay. The W6 project-install store
  is a separate signed, physical-approval, probation/autoload/rollback chain.
  Neither is needed to prove W7 and neither may be called by W7.
- The structured store can safely persist bounded records, but using it would
  combine the first network intake with a new durable-write policy decision.
  W7 should therefore retain bytes in the existing RAM candidate/catalog only.
  A later reviewed-import action may reuse W1's blob/revision commit, but that
  is explicitly not automatic download behavior.
- `fake-cloud-server` is a WebSocket publisher that verifies an OTA envelope
  and writes to the parked host registry. It is not a static HTTPS CAS reader,
  uses a different Ed25519/BLAKE3 envelope and would add protocol work in the
  guest. W7 does not need it and should not modify or unpark `ota/`,
  `registry/` or `fake-cloud/`.
- The current QEMU runner uses `-netdev user` plus e1000. The guest can address
  a host-only fixture through the slirp host alias (`10.0.2.2`), subject to a
  focused probe on this Windows/QEMU build.
- ADR 0008 accepts exact per-service Wasm import grants and ultimately wants
  TLS verification in a scoped service, but the required `net.*`/crypto/time
  imports remain closed. A native Stage-0 W7 adapter can reuse the network
  stack now only if the owner explicitly accepts that placement as a bounded
  integration of the transport-neutral acquisition contract. Otherwise W7
  must stop until the service/import work is authorized; it must not add a
  decorative Wasm wrapper around a kernel HTTPS fetch.

## Threat and trust analysis

The serial path accepts bytes only from the local console/harness. Network
acquisition adds an unauthenticated peer, remote timing and framing, endpoint
selection, TLS state, response parsing, resource-exhaustion and concurrency
surface before the existing artifact checks. TLS establishes transport peer
evidence; it does not grant artifact load-worthiness. The distribution
signature establishes provenance; it still does not grant load-worthiness.

### Required trust boundaries

1. **Request authority:** the caller selects a compiled or provisioned
   `source_id`, exact SHA-256, exact byte length and ordered chunk hashes. The
   caller never supplies a URL, host, IP, port, request header or redirect
   target. Unknown sources deny before DNS/TCP.
2. **Transport trust:** the selected source maps to one exact SNI host, port,
   path template and SPKI pin policy. W7 accepts only a successful TLS 1.3
   P-256 CertificateVerify plus matching SPKI pin. The development TLS bypass
   is never accepted. The honest trust label remains
   `pin_only_no_webpki_chain_validation` with `not_validated_stage0`; W7 does
   not claim WebPKI or trusted time.
3. **Artifact provenance:** the active local catalog must already bind the same
   whole SHA-256, length, chunk count, distribution signature and complete
   guest-verified receiver identity. The distribution signature is evidence
   only.
4. **Candidate integrity:** each canonical chunk is checked against the
   request's ordered expected hash through the existing
   `ChunkedDistributionDelivery::accept_chunk`; finalize recomputes the whole
   hash and reruns the existing registry-selection/provenance path.
5. **Load authority:** the existing receiver preflight remains the next
   boundary and continues to name the missing M6/M7/provider/owner evidence.
   W7 must not satisfy, bypass or reinterpret those gates.
6. **Memory/export:** acquisition facts and bytes are `local_only`; unknown
   classification is also treated as `local_only`. State is `current_boot`.
   No bytes, endpoint details or raw response data enter provider context,
   summaries, logs or durable memory.

### Mandatory denials and failure behavior

| Threat/failure | Required behavior |
| --- | --- |
| Empty, malformed or extra command arguments | `capability_denied`; no DNS/TCP; no candidate change |
| Caller-supplied URL/host/port/path or unknown `source_id` | deny before network; no generic fetch/SSRF surface |
| Catalog absent, wrong whole hash/length/count, receiver identity incomplete or receiver evidence invalid | deny before network |
| Missing/invalid acquisition pin, wrong SNI, wrong SPKI, TLS handshake or CertificateVerify failure | abort TCP, clear only the pending W7 session, preserve the prior valid candidate |
| DNS/TCP unavailable, connect timeout, read timeout or peer close | explicit transport denial; abort socket; discard incomplete W7 bytes |
| Redirect, non-200 status, missing/duplicate/invalid `Content-Length`, chunked transfer encoding, content encoding, oversized headers or unexpected content type | fail closed; no redirect following or decoder fallback |
| Declared body over 256 KiB, more than four logical chunks, any chunk over 64 KiB, arithmetic overflow or allocation failure | deny before body growth; abort socket |
| Body shorter/longer than the exact request and catalog length | discard pending W7 session; no finalize |
| Per-chunk SHA mismatch, duplicate logical chunk with different bytes, missing chunk or out-of-range index | use the existing chunk denial; discard pending W7 session |
| Whole-object SHA mismatch | existing finalize denial; no retained candidate replacement |
| Mid-transfer abort | no partial catalog visibility, no retained-candidate replacement, socket released for recovery/provider use |
| Replay of identical content | idempotent content-addressed result is allowed; it grants nothing new |
| Concurrent OpenAI/provider and W7 TCP use | second claimant gets `network_transport_busy`; no shared-socket cross-talk |
| Server sends executable Wasm, source, archive, `build.rs` or malformed bytes | bytes remain inert; no build, archive extraction, parse-as-project, load or execution |
| Any attempt to call W1/W6/structured-store/install/rollback writers | design violation and stop condition; W7 result must report `durable_write_attempted:false` |
| Provider/API-key/Vault request | design violation; W7 has no Authorization header, secret lease, provider context or provider export |
| SAFE/recovery use | recovery never downloads; network acquisition remains unavailable there unless separately authorized |

Every failure must leave the previous successfully retained candidate intact.
Only a fully verified replacement may atomically replace it. This is the same
failure-preservation rule W1 `/revise` uses for the prior valid revision.

## Design

### Transport choice

Use one bounded HTTPS `GET` to a fixed, pin-bound QEMU host fixture. The guest
requests raw `application/octet-stream` bytes at a path derived solely from the
approved whole SHA-256, requires an exact `Content-Length`, rejects redirects,
chunked transfer and compression, and splits the body into canonical 64-KiB
logical chunks while streaming. This reuses the existing TLS/TCP substrate and
HTTP header parser without introducing WebSocket, registry-index JSON, archive
or command parsing in the guest.

The QEMU source policy is test-only configuration:

```text
source_id: local.qemu.w7
tcp_peer: 10.0.2.2:8443
sni_host: w7.test.raios
path: /raios/cas/sha256/<64-lower-hex>
method: GET
accepted_status: 200 only
accepted_content_type: application/octet-stream
tls: TLS 1.3 + exact P-256 SPKI pin + CertificateVerify
```

The focused harness starts a small PowerShell/.NET `SslStream` server before
packaging the temporary image. It generates an ephemeral P-256 test
certificate in memory, publishes only its SPKI SHA-256 through a temporary
ready file, and the profile embeds that pin only in its temporary image. The
default release image contains no W7 source pin, so the method fails closed as
`acquisition_pin_missing`. No test private key is committed. The server exposes
one raw artifact and a host-only control channel for the next-response negative
mode; it does not expose a public listener or a general filesystem root.

`fake-cloud-server` remains parked. Replacing the fixture with it would require
an explicit owner decision to unpark the crate and a separate decision about
why WebSocket/Ed25519/BLAKE3 should enter the W7 path.

### Typed request

The command is:

```text
module.acquire_network_candidate \
  local.qemu.w7 \
  sha256:<whole> \
  <total_length> \
  sha256:<chunk_0> [sha256:<chunk_1> ... sha256:<chunk_3>]
```

The number of chunk hashes is the declared chunk count. It must equal
`ceil(total_length / 65536)`, be between one and four, and match the active
catalog's whole hash, length and count. The source policy derives every network
field; the command cannot add headers or change the destination.

This method is an effectful, typed acquisition command, not a read method. Its
record-model response should have a stable current-boot id and include:

- `scope: current_boot`, `classification: local_only`;
- source-policy id, SNI host id and non-secret pin id (not raw pin bytes);
- requested and observed whole SHA-256/length;
- ordered expected/observed per-chunk SHA-256 and lengths;
- TLS verifier result and honest chain/time posture;
- catalog entry id, receiver-identity evidence locator and existing preflight
  locator;
- network attempted/bytes read/finalize reached status;
- retained-candidate hash and `retained_in_ram`;
- all install/load/execute/build/persist/provider/secret/owner-seal authority
  fields false.

No raw downloaded bytes, response headers, certificate DER, endpoint IP, URL,
signature bytes or source content are logged or exported. The exact local
source locator may be retained only in the `local_only` response.

### Exact bounded flow

1. The owner/harness submits the existing catalog entry, receiver-identity
   metadata and six raw receiver evidence parts. The guest performs the
   existing P-256 checks and marks receiver identity complete.
2. The caller sends `module.acquire_network_candidate` with the exact source,
   whole hash, total length and canonical chunk hashes.
3. A pure raios-core evaluator checks grammar, source policy, quotas, canonical
   chunk geometry and equality with the active catalog/receiver evidence. A
   denial performs no network I/O.
4. W7 obtains the singleton TCP lease. If OpenAI or another acquisition owns
   it, W7 denies without aborting the other owner.
5. raiOS connects through its existing e1000/smoltcp path, performs pinned TLS
   without provider credentials and writes the fixed GET request.
6. The bounded HTTP reader accepts only the fixed response shape. It streams
   body bytes into one at-most-64-KiB buffer. On each logical boundary, it
   passes raw bytes plus the caller's expected hash to the same
   transport-neutral acceptance function used by serial delivery.
7. After exact EOF/length, W7 invokes the same existing finalize function as
   serial delivery. That function rechecks every chunk and the whole hash,
   verifies provenance honesty, performs registry selection and only then
   atomically replaces the RAM candidate.
8. For the network catalog source, successful finalize records the same
   `finalized_candidate_sha256` used by
   `module.distribution_receiver_identity_load_preflight`. The preflight still
   returns denied with the existing missing M6/M7/provider/owner gates.
9. W7 emits the typed local-only current-boot receipt and releases/aborts the
   TCP lease on every exit path.

The serial and network wrappers may decode or read bytes differently, but they
must converge before chunk acceptance. There must be one shared byte-accept
function and one shared finalize/stage function.

### Where bytes land

Downloaded bytes live only in the existing RAM-backed pending delivery and
`RETAINED_CANDIDATE`, with catalog/receipt facts labeled `current_boot` and
`local_only`. They disappear on reboot/F12 reset and never enter
`raios-core::structured_store`, `project_store`, `project_install_store`, M7
ARTSTOR or the W6 autoload chain.

This is deliberate, not fake persistence. Network acquisition is unreviewed
input; keeping it volatile prevents a download from becoming a durable write
by implication. If the owner later wants reboot-surviving quarantine, the next
slice must define an explicit reviewed commit into a dedicated namespace or
reuse W1 blob/revision records, run a storage-focused multi-boot profile, and
still keep W6 install authority separate.

### Review and discard

The successful acquisition response plus the existing receiver preflight are
the review locators. A small follow-up command should let the user remove the
exact current-boot candidate:

```text
module.discard_network_candidate sha256:<whole>
```

It clears only when the hash matches the retained network candidate; a wrong
hash denies. It does not uninstall, mutate W1, touch a W6 installed app or
write an audit/store record. Reading the receipt or discarding bytes grants no
authority.

## Verification plan

### Focused VM profile

Add `network-acquisition`, always run with `-Network`. It packages a temporary
pin-bearing image, starts the host-only TLS fixture, boots QEMU with e1000 user
networking, waits for DHCP, drives the typed commands over the existing serial
control path and stops the fixture in `finally`.

Positive needles/predicates:

1. e1000 initialized and DHCP lease acquired;
2. catalog plus all six receiver evidence parts are guest-verified;
3. exact source policy selected, TCP lease acquired and pinned TLS 1.3
   CertificateVerify/SPKI match observed;
4. exact HTTP status/content type/content length accepted;
5. every canonical chunk length/hash accepted by the shared M12+ verifier;
6. whole hash and provenance verified by the existing finalize path;
7. candidate retained as `local_only`/`current_boot`, RAM-only and inert;
8. receiver preflight binds the network-finalized candidate hash and still
   names the same four missing gates;
9. load/build/execute/install/persist/provider/secret/owner-seal flags all
   remain false;
10. exact-hash discard succeeds and a second inspect reports no candidate.

Negative cases, all followed by proof that no partial candidate replaced the
prior valid candidate and the TCP lease was released:

- malformed request, unknown source, wrong length/count and missing catalog;
- incomplete or hash-mismatched receiver evidence;
- missing pin and wrong certificate/SPKI;
- non-200, redirect, malformed/oversized headers, missing/duplicate/wrong
  `Content-Length`, chunked transfer, compression and wrong content type;
- declared/body oversize, per-chunk hash mismatch, whole-hash mismatch, short
  body, extra body and mid-transfer close;
- connect/read timeout;
- provider/acquisition TCP ownership conflict;
- retry after each transport failure, proving the next valid acquisition can
  succeed without reboot;
- wrong-hash discard denial;
- recovery/SAFE request denial and absence of any durable-store or W6 install
  action.

The fixture's host-only control channel selects the next response behavior, so
the guest never receives a test-mode URL or header. The report records the
fixture artifact SHA-256 and public SPKI pin id, never the ephemeral private
key.

### Host tests

- Request evaluator: exact valid request; empty/malformed/extra fields;
  unknown source; catalog/receiver mismatch; zero/over-limit length; wrong
  canonical chunk count; duplicate/missing/bad hashes; classification forced
  to `local_only`.
- Bounded HTTP reader: 200/exact content length; header cap; duplicate or
  malformed content length; redirects/non-200; transfer/content encoding;
  early EOF; extra bytes; timeout classification.
- Shared intake seam: serial bytes and simulated network bytes produce the
  same selection/finalize outcome and record hashes; every existing
  `distribution_registry` negative remains green.
- TCP ownership: OpenAI and W7 cannot own the singleton socket concurrently;
  only the owner can release/abort its lease.
- Receipt projection: stable ids, source/evidence locators, classification and
  every authority field false.

Workers write these host tests but do not build `seed-kernel`. The orchestrator
owns the kernel compile/package loop and every QEMU run.

### Regression profiles

Per implementation slice, run only the new focused `network-acquisition`
profile after the orchestrator's compile loop. At W7 block close also run:

- `quick -Network` for e1000/DHCP and general protocol behavior;
- `m12-distribution-provenance` unchanged, proving the serial sibling remains
  byte-for-byte/semantically intact;
- `m11-7-httphead` if the shared HTTP parser or its adapter changes;
- `full` and `recovery` once at W7 block close, per owner cadence;
- source-size, formatting and secret scans.

Any failed VM run must be classified in `docs/PROJECT_STATUS.md` before retry,
per the repository failure-classification rule.

## Slice and worker packet breakdown

The packets are sequential because both touch the acquisition/catalog
boundary. A worker writes code and host tests; the orchestrator compiles the
kernel, fixes integration fallout, runs the named focused profile and performs
the pre-commit diff read.

### W7-1: exact pinned-HTTPS fetch into the existing inert candidate path

Capability sentence: a user or agent can request one exact catalog-bound hash
from the fixed pinned QEMU HTTPS source and raiOS retains it as an inert
`current_boot` candidate after shared chunk, whole-hash, provenance and
receiver checks.

Worker write set:

- `raios-core/src/network_acquisition.rs` (new): pure request/source-policy
  evaluator, bounded HTTP response state and typed receipt projection with
  host tests;
- `raios-core/src/lib.rs`: export the module;
- `seed-kernel/src/agent_protocol_registry.rs`: rename/extract the minimal
  transport-neutral pending-delivery byte seam, add the network source id and
  route network success through the existing finalize/preflight state;
- `seed-kernel/src/network_acquisition.rs` (new): fixed-source TCP/TLS/GET
  adapter and acquisition-pin verifier; no provider/Vault dependency;
- `seed-kernel/src/net.rs`: explicit singleton TCP owner claim/release/abort;
- `seed-kernel/src/openai.rs`: use the same TCP ownership primitive so the two
  clients cannot overlap;
- `seed-kernel/src/tls_io.rs`: only if needed, add a bounded constructor for
  W7's shorter read/write timeouts;
- `seed-kernel/src/agent_protocol.rs`: register the effectful command;
- `seed-kernel/src/main.rs`: initialize/poll only if the adapter is asynchronous;
- `scripts/build-seed-kernel.ps1`: pass the test-only W7 SPKI/source values
  from the environment without changing default release configuration;
- `vm-harness/w7-artifact-server.ps1` (new): host-only bounded TLS fixture;
- `vm-harness/shadow-vm-smoke.ps1`: add the profile, fixture lifecycle,
  temporary pin packaging and `-Network` requirement;
- `vm-harness/shadow-vm-smoke-profile-network-acquisition.ps1` (new): positive
  path and all malformed/oversize/hash/pin/abort/busy negatives above.

Do not touch `project_store.rs`, `project_install_store.rs`, structured-store
code, W6 runtime/install code, `ota/`, `registry/` or `fake-cloud/`.

Worker host checks: `raios-core` tests for the new evaluator/reader plus the
existing distribution-registry tests. Orchestrator close: kernel compile and
one `network-acquisition -Network` report.

### W7-2: exact inspect/discard control and failure-preserving retry proof

Capability sentence: a user can inspect the typed receipt for an exact
network-retained candidate, discard only that hash, and retry after a failed
transfer without losing or confusing a prior valid candidate.

Worker write set:

- `raios-core/src/network_acquisition.rs`: inspect/discard decision and tests;
- `seed-kernel/src/network_acquisition.rs`: receipt retention and exact-hash
  clear operation;
- `seed-kernel/src/agent_protocol_registry.rs`: expose existing catalog and
  preflight locators without duplicating them;
- `seed-kernel/src/agent_protocol.rs`: register inspect/discard methods;
- `vm-harness/shadow-vm-smoke-profile-network-acquisition.ps1`: inspect,
  wrong-hash discard, valid discard, prior-candidate preservation and
  fail-then-valid retry sequence.

No other source file is in scope. Orchestrator close: compile and one focused
`network-acquisition -Network` run.

### W7 close packet (orchestrator-owned evidence/docs only)

After both slices are green, the orchestrator runs the regression set, secret
scan and end-of-session checks, then updates only the normal status surfaces:
`docs/PROJECT_STATUS.md`, `docs/OWNER_DASHBOARD.md`, and `docs/ROADMAP.md` if the
cursor changes. This packet adds no capability and must not be mislabeled as a
feature slice.

### Explicitly parked follow-up: durable reviewed quarantine

Not part of W7-1/W7-2. If requested later, design a separate storage-boundary
packet that commits already downloaded and explicitly reviewed bytes as a
content-addressed record, replays them after reboot and still has no W6 install
edge. It must have its own focused structured-store/project profile and owner
approval for namespace, quota, deletion/GC and review authority.

## Open owner decisions

1. **Native Stage-0 adapter or wait for Wasm network imports?** Recommendation:
   explicitly authorize the narrow Stage-0 adapter for this QEMU W7 slice only
   while preserving the transport-neutral contract and no-secret boundary.
   If the owner requires ADR 0008 Option 2 placement immediately, stop W7 and
   first scope the real `net.*`/crypto/entropy/time imports and a TLS-capable
   acquisition service. Do not fake service isolation with a one-call
   `https_get` import.
2. **Is `current_boot` retention sufficient for W7?** Recommendation: yes.
   This proves the new network capability without coupling it to durable-write
   authority. Reboot-surviving quarantine should be a later reviewed-store
   packet.
3. **May the QEMU proof use a W7-specific SPKI pin and fixed source id without
   WebPKI/trusted time?** Recommendation: yes, with the exact honest labels
   above and no development bypass. Production public sources remain gated.
4. **May W7 metadata/receiver evidence continue to arrive through the existing
   serial catalog while only artifact bytes come over HTTPS?** Recommendation:
   yes for the first network acquisition. Fetching/parsing a remote manifest
   is a separate attack surface and not needed to prove autonomous byte
   download.
5. **Should any parked host crate be used?** Recommendation: no. If the owner
   prefers `fake-cloud-server` or `registry-tools` as the live fixture, record
   an explicit unpark decision and constrain it to test publication/serving;
   do not import its Ed25519/BLAKE3 authority into guest load decisions.

## Risks

The largest security risk is creating a second network-specific staging or
finalize path that bypasses the proven M12+ catalog/chunk/provenance/receiver
chain. The design prevents that by requiring serial and network transports to
converge on one raw-byte acceptance function and the existing single finalize
function before any retained candidate changes.

The largest architectural risk is growing another native TLS/HTTP client while
M11 intends to move internet parsing into Wasm. The source-policy/evaluator,
chunking, receipt and M12+ integration must therefore live outside provider
code and remain transport-neutral; owner rejection of the narrow Stage-0
adapter is a stop condition, not permission to add a fake wrapper or fallback.

Other material risks are the singleton TCP socket, blocking TLS reads delaying
recovery, Windows/QEMU reachability of `10.0.2.2`, and test-certificate leakage.
TCP ownership, short hard timeouts, a focused reachability probe and an
ephemeral in-memory fixture key address them without adding a new dependency.

## Stop conditions

Stop and return to the owner/orchestrator if any of these is true:

- ADR 0009 Option B/native Stage-0 network pull has not been explicitly opened;
- the implementation needs the OpenAI key, Vault lease, provider context,
  provider export or unverified TLS bypass;
- a caller-controlled URL, redirect, host, port or header becomes necessary;
- the path cannot use the shared M12+ byte acceptance and finalize functions;
- success would write structured-store, project, ARTSTOR, W6 install or durable
  memory state;
- the artifact requires more than 256 KiB/four chunks; do not silently widen
  quotas;
- `fake-cloud`, `registry` or `ota` must be modified without an owner unpark
  decision;
- QEMU cannot prove the positive e1000/DHCP/TCP/TLS path without a development
  trust bypass;
- the current full-profile Red Gate is red for a guest regression;
- any result would be described as WebPKI, trusted-time, owner-sealed, physical
  hardware, WiFi, install, execution or persistence evidence.
