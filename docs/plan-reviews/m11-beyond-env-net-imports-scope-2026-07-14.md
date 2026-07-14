# M11 Scope: Beyond-Env Wasm Network Imports (2026-07-14)

Packet: `M11-NETIMPORTS-SCOPE`

Status: read-only recon and design. This document grants no import, does not
change `policy_allows_beyond_env`, and does not authorize W7 implementation.
It follows owner decision `e696a62`: W7 must run as a scoped Wasm acquisition
service, not as another native Stage-0 HTTPS adapter.

## Capability sentence

After this lane and the separately owner-gated arming slice, a user or agent
can request one exact catalog-bound hash from the fixed, pin-bound W7 HTTPS
source; a signed, fuel-metered Wasm service performs TLS 1.3 and HTTP parsing,
and raiOS retains the bytes only after the permanent core independently
authorizes the transport evidence and routes every chunk through the existing
M12+ verifier into an inert `local_only`, `current_boot` quarantine.

## Recommendation

Use one versioned host-import ABI with these families:

1. existing `env.input_len` and `env.input_read` for the bounded invocation
   request;
2. `net.*` for one pre-bound, generation-checked TCP lease over the kernel's
   singleton socket;
3. `crypto.*` for fixed TLS 1.3/P-256/SHA-256/AES-128-GCM primitives with
   opaque key/session handles;
4. `acquire.*` for the only allowed convergence into the shared M12+ chunk and
   finalize path;
5. `time.monotonic_ms` as an optional untrusted timeout clock, not granted to
   W7 because the core enforces its deadlines;
6. `secret_lease.openai_authorization_send` as a separate future provider-only
   Broker output, never granted to W7; and
7. no general entropy import: `crypto.tls13_session_open` consumes core entropy
   internally and returns only the public ClientHello random and P-256 public
   key.

Recommend host crypto, not pure-Wasm crypto. The service should own TLS record,
handshake, certificate, HTTP and response state machines, while existing pinned
native primitives keep private ECDHE material, traffic keys and nonce counters
out of guest memory. Pure-Wasm crypto would enlarge the artifact, consume much
more interpreter fuel, weaken timing confidence and put key material in guest
memory without moving additional attacker-controlled parsing out of the core.

The W7 service's exact requested list is 16 imports, equal to today's
`MAX_GRANTED_IMPORTS` and requiring no cap increase:

```text
env.input_len
env.input_read
net.tcp_open
net.tcp_send
net.tcp_recv
net.tcp_close
crypto.tls13_session_open
crypto.sha256
crypto.p256_verify
crypto.tls13_handshake_keys
crypto.tls13_application_keys
crypto.tls13_finished
crypto.tls13_aead_seal
crypto.tls13_aead_open
acquire.chunk_accept
acquire.finalize
```

It gets no `env.output_write`, `time.*`, `secret_lease.*`, DNS, generic URL,
filesystem, durable-store, provider-context, install, load or execution import.

## Recon: current enforcement and substrate

- `raios-core/src/scoped_wasm_import_grant.rs` has a known-import allowlist,
  a 16-import cap, duplicate rejection and the forward
  `import_beyond_env_not_owner_authorized` denial. Its ordinary evaluator still
  constructs `policy_allows_beyond_env:false`; a true value currently reaches
  `evidence_bound_import_grant_required`, not authorization.
- The personal-shell evaluator already demonstrates the required stronger
  shape: signed descriptor/artifact/computed-grant evidence binds one exact
  ordered import-list hash and the implemented linker surface.
- `seed-kernel/src/wasm_runtime.rs` builds a fresh `wasmi::Linker` per instance,
  defines only evaluator-authorized imports, checks every `module.imports()`
  pair is a subset before instantiation, meters fuel and memory, and captures
  the existing 4-KiB byte-buffer channel. The current `EnvelopeState` does not
  yet carry service generation, invocation identity or owned resource handles;
  live imports need that foundation before they can exist safely.
- Current parser guests use `env.input_len`, `env.input_read` and
  `env.output_write`. Those functions bounds-check guest memory and charge
  fuel, but the output channel is capped at 4 KiB and only surfaces a length
  and hash. It is not a substitute for W7's 64-KiB chunk acceptance or M12+
  finalize path.
- `seed-kernel/src/net.rs` owns one smoltcp TCP socket and exposes global
  connect/send/receive/abort functions without an owner token. An active
  socket is therefore a singleton resource, not a pool that can honestly be
  represented as many Wasm connections.
- `seed-kernel/src/tls_io.rs` pumps `net::poll()` in bounded blocking loops,
  with 60-second reads and 15-second writes. `openai.rs` uses that stream for
  TLS 1.3, checks trust before copying an API key, and aborts the global TCP
  socket on failure.
- `openai_trust.rs` performs exact-host, leaf/SPKI pin and P-256
  CertificateVerify checks. Its honest permanent labels remain pin-only and
  `not_validated_stage0`; WebPKI chain and trusted-time validation do not
  exist.
- The Secret Vault already implements the ADR 0012 Broker ordering: exact
  provider trust creates a metadata-only durable pre-use audit, the append is
  write/readback/reparse/rescan verified, current state is rechecked, and only
  then a move-only lease can write the exact OpenAI Authorization header. No
  plaintext getter exists.
- `agent_protocol_registry.rs` still couples serial parsing to
  `PendingSerialDistribution`, but `finalize_pending_distribution` already
  converges on `ChunkedDistributionDelivery::accept_chunk`, whole-object
  verification, registry selection, provenance verification and only then
  `intake_and_retain_external_wasm_candidate`. W7 must extract and reuse that
  seam, not reproduce it in the service or a network adapter.

## Host-import ABI v1

The manifest should bind both the exact ordered `(module,name)` list and
`host_import_abi = "raios.host_imports.v1"` into the computed grant. Keeping
the existing short module names avoids renaming current imports; the separate
ABI id prevents a signature or semantic change from silently reinterpreting an
old grant.

All pointers and lengths are Wasm `i32`. The host checks non-negative values,
checked addition, exported-memory presence and family-specific caps before any
copy or effect. Handles are positive `i32` values containing a slot plus a
generation; zero is never valid. Byte-returning calls return a non-negative
length. Other successful commands return `0`, except verification returns `1`
for valid and `0` for invalid. Negative results are stable ABI errors:

```text
-1 invalid_argument       -6 would_block
-2 capability_denied      -7 limit_exceeded
-3 resource_busy          -8 invalid_state
-4 timed_out              -9 transport_error
-5 closed                -10 killed
```

The protocol response and local audit retain the more precise typed denial;
the guest receives only this bounded error vocabulary. A trap is reserved for
memory violations, impossible linker drift and fuel exhaustion, not ordinary
peer or transport failure.

### Invocation authority and handles

Every live-import store carries an unforgeable core-created invocation record:

```text
service_id
artifact_sha256
service_generation
instance_generation
invocation_id
authorized_import_list_sha256
source_policy_id
boot_posture
kill_generation
```

The guest cannot supply or rewrite it. Every network, crypto, acquisition and
secret call checks the caller's invocation against the resource owner. A stale,
foreign or already-closed handle denies without touching another owner's
resource. All resources are fixed-capacity; W7 gets at most one TCP handle, one
TLS session and one pending acquisition.

The W7 store permits one instance, one memory capped at 2 MiB, at most one
table with 64 elements, and a slice-pinned fuel budget measured from the final
artifact plus negative fixtures. A module declaring broader memory/table
limits denies before instantiation; the arming slice may not “fix” that by
removing limits.

On normal return, trap, `OutOfFuel`, F12, service crash or un-promotion, one
core teardown path aborts the owned TCP socket, zeroizes and drops TLS keys,
invalidates handle generations, discards only the incomplete W7 delivery,
preserves the previous valid candidate, and releases the transport lease.

### `net.*`

| Exact import and Wasm signature | Core implementation | Refusals and evidence |
| --- | --- | --- |
| `net.tcp_open() -> i32` | Claims the one global transport lease for this invocation and connects to the IP/port already selected by the core's source policy. It returns one generation-checked connection handle. The guest supplies no address. | Denies missing source policy/pin, SAFE or recovery posture, wrong service/grant, unconfigured network, existing provider/acquisition owner, timeout or killed invocation. Emits local-only invocation/grant/source-policy ids, lease generation, result and timing bounds; no raw pin or private endpoint configuration. |
| `net.tcp_send(conn:i32, ptr:i32, len:i32) -> i32` | Bounds-checks memory, caps one call at 4096 bytes, pumps the existing TCP sender and charges fuel plus a byte quota. | Denies foreign/stale handle, write after close, cumulative transmit cap, absolute deadline or kill. Records operation, byte count and result, never payload bytes. |
| `net.tcp_recv(conn:i32, ptr:i32, cap:i32) -> i32` | Caps one call at 4096 bytes, pumps the existing receiver and copies only received bytes into guest memory. `0` is orderly EOF. | Denies foreign/stale handle, cumulative receive cap, idle/absolute deadline or kill. Records operation, byte count and result, never peer bytes. |
| `net.tcp_close(conn:i32) -> i32` | Aborts/closes only the caller-owned singleton socket, drops associated crypto state and releases the lease. It is idempotent for the current owner. | A foreign owner cannot close or abort the socket. Evidence records the close cause and cleanup completion. |

For W7, `tcp_open` is pre-bound to `local.qemu.w7` -> `10.0.2.2:8443`.
There is no v1 `net.resolve`, UDP, listen, accept, arbitrary connect, socket
option or caller-selected timeout. A later production source may let the core
resolve a policy-owned hostname before instantiation, but must not add a guest
DNS/SSRF surface merely to change the fixture.

Recommended source-policy ceilings are 5 seconds to connect, 15 seconds idle
per I/O direction, 90 seconds total lease time, 32 KiB cumulative transmit and
320 KiB cumulative receive. They are core policy, not import arguments. The
receive allowance covers the 256-KiB candidate plus a bounded TLS handshake and
HTTP header. Final values should be pinned by the owner before arming.

The native OpenAI path must claim the same lease. A second claimant receives
`resource_busy`; it must not inspect, reuse or abort the current owner's
socket. This virtualizes the singleton honestly as one exclusive capability,
not as fictional multiple sockets.

### `crypto.*`

The v1 suite is intentionally one profile: TLS 1.3,
`TLS_AES_128_GCM_SHA256`, P-256 ECDHE and
`ecdsa_secp256r1_sha256`. The service owns protocol parsing and state-machine
choices; the core owns fixed-shape primitive execution, key custody, sequence
counters and the authority to turn captured evidence into a trust label.

| Exact import and Wasm signature | Core implementation | Refusals and evidence |
| --- | --- | --- |
| `crypto.tls13_session_open(conn:i32, public_ptr:i32, public_len:i32) -> i32` | Requires `public_len == 97`; consumes ready core entropy, stores the private P-256 key in an opaque session, and writes only `client_random[32] || uncompressed_public_key[65]`. Returns the session handle. | Denies foreign TCP ownership, entropy not ready, duplicate session, wrong output size or memory fault. Records entropy-ready/source markers and public-output hash, never RNG state or private material. |
| `crypto.sha256(session:i32, ptr:i32, len:i32, out32:i32) -> i32` | Hashes at most 16 KiB of public handshake/transcript material with the existing pinned SHA-256 implementation. The session argument binds the operation to captured TLS evidence. | Denies foreign session, oversize input or bad memory. Records input length/hash domain and output hash only when needed for evidence. It grants no trust by itself. |
| `crypto.p256_verify(session:i32, spki_ptr:i32, spki_len:i32, msg_ptr:i32, msg_len:i32, sig_ptr:i32, sig_len:i32) -> i32` | Verifies one bounded P-256 ECDSA/SHA-256 signature. The core captures SPKI, message and signature hashes and compares the SPKI hash with the source-policy pin; the service still parses the certificate and constructs the TLS CertificateVerify message. | Denies unsupported encoding/algorithm, foreign session, bad bounds or absent source pin. Math-valid and pin-match are separate captured facts. Only the scoped core evaluator may mark the transport session verified. |
| `crypto.tls13_handshake_keys(session:i32, peer_pub_ptr:i32, peer_pub_len:i32, hello_hash_ptr:i32) -> i32` | Performs P-256 ECDH and the fixed TLS 1.3 HKDF handshake schedule. Shared secret and handshake keys remain in the opaque session. | Denies wrong curve/length, repeated/out-of-order transition, foreign handle or zero/invalid transcript hash. Records transition and public-input hashes, never keys. |
| `crypto.tls13_application_keys(session:i32, handshake_hash_ptr:i32) -> i32` | Derives fixed client/server application traffic keys only after pinned CertificateVerify and server Finished evidence are positive in the core session. | Denies self-asserted guest trust, missing pin/signature/Finished evidence, repeated transition or foreign handle. Records the authorized transition only. |
| `crypto.tls13_finished(session:i32, mode:i32, transcript_hash_ptr:i32, proof_ptr:i32, proof_len:i32) -> i32` | With `mode=0`, writes the 32-byte client Finished proof; with `mode=1`, verifies the 32-byte server proof. This is one fixed two-mode primitive, not a generic dispatcher. | Denies any other mode, non-32-byte proof, wrong state or foreign handle. Server verification result is captured core evidence. |
| `crypto.tls13_aead_seal(session:i32, header_ptr:i32, plain_ptr:i32, plain_len:i32, out_ptr:i32, out_cap:i32) -> i32` | Validates the fixed five-byte TLS record header, seals one bounded record and increments the core-owned send sequence only on success. | Denies invalid header/length, record over 16 KiB plus TLS overhead, wrong state, nonce/sequence exhaustion, foreign handle or small output. Records lengths/result only. |
| `crypto.tls13_aead_open(session:i32, header_ptr:i32, cipher_ptr:i32, cipher_len:i32, out_ptr:i32, out_cap:i32) -> i32` | Validates the fixed header, opens one bounded record and increments the core-owned receive sequence only after tag success. Plaintext enters guest memory for TLS/HTTP parsing. | Denies tag/header/length/state errors, sequence exhaustion, foreign handle or small output. Records lengths/result only. |

`net.tcp_close`, teardown or revocation closes the associated TLS session, so
no extra crypto-close import is needed. No import returns a private key,
shared secret, HKDF secret, traffic key, IV, AEAD nonce or Vault material. No
RSA, arbitrary curve, arbitrary AEAD, raw HKDF or generic sign operation is in
v1. If the fixed W7 TLS peer cannot negotiate this profile, stop rather than
silently widen it.

### Entropy family

No standalone entropy import is proposed for W7. The only genuine TLS need is
fresh ClientHello randomness and an ephemeral P-256 private key;
`crypto.tls13_session_open` produces both atomically, returns only public
bytes, and fails if core entropy is not ready. A future
`entropy.public_fill` must have a concrete non-cryptographic consumer and a
separate owner review; it must never become a way to export secret RNG or key
state.

### `time.*`

| Exact import and Wasm signature | Core implementation | Refusals and evidence |
| --- | --- | --- |
| `time.monotonic_ms() -> i64` | Returns boot-relative monotonic milliseconds from the same calibrated TSC basis used by `tls_io`. | It is always labeled `boot_relative`, `local_only`, `trusted:false`, and cannot support certificate validity or a wall-clock claim. |

W7 should not request it: the core transport lease enforces connect, idle and
total deadlines, and the service gains no authority by duplicating them.
There is no `time.now`, Unix time or trusted-time import. Certificate-window
parsing remains guest evidence; current CMOS comparison remains explicitly
unverified in the core. A positive time-valid trust state waits for the M10
owner-gated trusted-time input.

### `secret_lease.*`

| Exact import and Wasm signature | Core implementation | Refusals and evidence |
| --- | --- | --- |
| `secret_lease.openai_authorization_send(session:i32) -> i32` | For the exact future OpenAI provider service only, consumes positive core-owned provider trust and request/export authorization, appends and verifies the ADR 0012 durable pre-use audit, rechecks Vault/store/policy/service generation, obtains one move-only Broker lease, formats the exact header in core memory, seals/sends it on the bound TLS session, advances the core send sequence, then zeroizes it. | Denies W7 and every other service, wrong host/source/session, missing trust/export authority, SAFE without explicit action, stale generation, locked Vault, audit failure, repeated use or transport failure. Evidence contains the existing metadata-only durable receipt plus service/artifact/grant/session ids and send outcome; never secret bytes, length or a secret-derived hash. |

The import does not return the header or plaintext to Wasm and accepts no
header name, host, target or body argument. It is not part of W7's grant.
ADR 0012 currently says V1 has no Wasm interface and names
`svc.provider.openai_direct` as the exact consumer. Therefore this import must
remain unimplemented or unarmable until the owner explicitly amends/reconciles
ADR 0012 and pins the future provider service identity. The present design is
the required Broker shape, not that approval.

### `acquire.*`

These two calls are necessary. The existing 4-KiB `env.output_write` channel
cannot safely carry a 256-KiB candidate, and letting the service stage directly
into `module_candidate_intake` would bypass W7's central security requirement.

| Exact import and Wasm signature | Core implementation | Refusals and evidence |
| --- | --- | --- |
| `acquire.chunk_accept(index:i32, ptr:i32, len:i32) -> i32` | Reads one canonical at-most-64-KiB chunk from guest memory and passes it to the single transport-neutral acceptance seam extracted from the current serial path. Expected index, length and SHA-256 come from the pre-authorized request/catalog, not guest arguments. | Denies before effect unless transport trust is core-authorized and the exact acquisition session is current. Existing M12+ index/hash/count/length denials remain authoritative. Records observed index/length/hash and the shared acceptance result, never raw bytes. |
| `acquire.finalize() -> i32` | Treats the call only as a request. The core rechecks exact EOF/length, all chunk outcomes, source/TLS evidence, catalog and receiver identity, then invokes the one shared M12+ finalize/selection/provenance/stage function. Only its success atomically replaces the prior RAM candidate. | Denies missing/extra bytes, incomplete chunks, trust/evidence mismatch, wrong owner/generation or any attempt to persist/load/execute/install. Emits the W7 `local_only`, `current_boot` receipt and keeps every authority flag false. |

Neither call writes W1, W6, structured-store, ARTSTOR, rollback or durable
memory. An incomplete acquisition is dropped on every exit; the previous valid
candidate is untouched.

## Authority split and W7 flow

### Permanent core only

- parse and authorize the typed acquisition command before network I/O;
- own the fixed source-policy table, endpoint, SNI/path template, SPKI pin and
  pin identifier;
- bind source policy, catalog, receiver evidence, artifact hash, service
  artifact/generation and exact import grant;
- own the singleton transport lease, absolute byte/time quotas and cleanup;
- hold entropy, ECDHE private keys, TLS traffic keys, AEAD sequence counters,
  pins, provider keys and Vault leases;
- capture crypto-call evidence, compare the source pin and run the scoped
  transport-trust evaluator;
- assign honest trust labels and keep chain/time fields
  `pin_only_no_webpki_chain_validation` / `not_validated_stage0`;
- authorize provider request/export and exact secret use; and
- accept chunks and finalize quarantine only through the existing M12+ path.

### Wasm service only

- build and parse TLS 1.3 records and handshake messages;
- parse the certificate and SPKI, construct the CertificateVerify message and
  drive fixed crypto calls;
- build the fixed SNI ClientHello and fixed HTTP GET from the core-derived
  invocation request;
- parse and enforce the W7 HTTP shape: 200 only, exact Content-Length and
  content type, no redirect/chunking/compression, bounded headers/body;
- split the exact body into canonical 64-KiB chunks and request their
  acceptance; and
- produce typed verifier/protocol evidence. It cannot assign trust, choose a
  destination, reveal a pin/key, finalize by assertion or make bytes loadable.

### End-to-end sequence

1. The existing serial path installs the catalog and complete receiver
   evidence. The core command evaluator checks the W7 source id, whole hash,
   total length, canonical chunk hashes/count and 256-KiB/four-chunk limits.
2. Before instantiation, the core verifies the signed service/artifact/grant
   chain, exact ABI/list hash and linker implementations. Missing or broader
   evidence denies with no network effect.
3. The core creates one invocation authority and stages a bounded canonical
   input frame through `env.input_*`. It contains the request hashes and
   core-derived SNI/path values, not an authority to change them.
4. `net.tcp_open()` claims the singleton lease and connects only to the
   source-policy endpoint. The service cannot name another endpoint.
5. The service drives TLS. Host crypto retains keys and captures primitive
   results. The service's parser output is evidence; the scoped core evaluator
   checks exact source/SNI/pin, host-captured P-256 verification, Finished and
   no-development-bypass facts before marking only this transport session
   usable.
6. The service sends the fixed unauthenticated GET, parses the bounded response
   and calls `acquire.chunk_accept` at each canonical boundary. A provider or
   Vault secret is never requested.
7. After exact body completion, `acquire.finalize` reruns the core checks and
   the shared M12+ finalize path. Only then may the prior RAM candidate be
   atomically replaced.
8. The core emits the W7 receipt and existing receiver preflight. Load,
   execute, build, install, persist, provider, secret and owner-seal authority
   remain false. Teardown closes every handle on every outcome.

This is ADR 0008 Option A plus trust shape Option 2: protocol/verifier code is
in Wasm, but the service never blesses itself.

## Grant, arming and revocation ladder

### Declaration and evaluation

1. The signed descriptor declares service id, artifact hash,
   `raios.host_imports.v1` and the ordered exact import pairs.
2. Descriptor-source signature, artifact signature/attestation, computed grant
   and observed `module.imports()` bind the same artifact and ordered-list hash.
3. The existing scoped evaluator is extended using the personal-shell evidence
   pattern: unknown ABI/import, duplicate, missing evidence, list/hash drift,
   missing implementation or policy false denies before instantiation.
4. The evaluator's authorized list is passed directly to the per-instance
   linker. No second hand-maintained broader bundle exists.
5. The existing durable import-grant audit records the exact service,
   artifact, generation, ABI, list hash, evidence locators and honest
   `dev_key_not_owner_sealed` tier. Call-level W7 facts are `local_only` and
   `current_boot`; only a future secret use has ADR 0012's mandatory durable
   pre-use audit.

### Grants-nothing phase

Adding ABI constants, known imports, host implementations, handle tables,
cleanup, tests or a signed service artifact must not change the production
call site: every `WasmImportGrantInput` continues to carry
`policy_allows_beyond_env:false`. The VM must prove a module requesting each
new family is denied before instantiation and no host implementation is
called. Existing env/ui services remain byte-identical.

### Explicit owner-gated arming slice

The first positive grant is its own packet and commit. It must name the exact
`svc.net.acquire.w7` service id, artifact SHA-256, service generation,
`raios.host_imports.v1` list hash, source policy and focused report. Only that
runtime path may construct `policy_allows_beyond_env:true`; all other paths
remain false. The owner must explicitly approve the arming diff after the
grants-nothing profiles are green.

Because owner sealing is not complete, the first grant remains honestly
`dev_key_not_owner_sealed`, current-boot and limited to the QEMU W7 policy. A
true boolean is not owner-seal evidence. The default image still has no W7
test pin, so even an import-capable artifact cannot connect there.

### Revocation and mid-connection failure

- Un-promotion/revocation prevents the next instantiation and invalidates the
  current service generation.
- F12 increments the core kill generation. Every blocking/poll loop and host
  call checks it and returns `killed`; teardown aborts only the owned socket.
- Fuel exhaustion or guest trap is caught as today, then runs the same teardown
  before the response returns.
- A service crash closes the connection, zeroizes crypto state, discards the
  incomplete pending delivery and keeps the prior valid candidate.
- A stale or foreign handle can neither send nor close another owner's socket.
- No restart or reboot automatically rearms or resumes an acquisition without
  re-verifying the signed grant and receiving a new typed request.

The current one-shot wasmi call and blocking network loops make F12 latency the
critical proof. A live slice must show that F12 is observed while the peer is
silent, not merely after a timeout. If interrupt/input plumbing cannot set and
observe the kill generation while a host import is active, stop and recut the
network ABI as short non-blocking polls or a resumable service invocation; do
not ship an unkillable 60-second kernel stall.

## Hostile-service and failure analysis

| Threat | Required bound or denial |
| --- | --- |
| SSRF / arbitrary destination | `net.tcp_open` has no host/IP/port argument and uses only the pre-authorized source policy. No DNS, redirect following or generic connect import exists. |
| Exfiltration | W7 receives only its bounded request frame and has no secret, memory-query, filesystem or provider-context import. It can transmit only to the fixed W7 peer under TX quota. `secret_lease.*` is absent from its linker. |
| Service self-blesses TLS | Guest claims are evidence only. Core-captured pin, P-256, Finished, source/SNI, service/artifact/grant and no-bypass facts feed the scoped evaluator; only core sets the session trust state. |
| Raw key extraction | All ECDHE/HKDF/traffic/Vault keys remain in opaque core slots. No generic crypto export or plaintext secret import exists. |
| AEAD nonce reuse | Send/receive sequence counters are core-owned and advance only on successful operations; stale/repeated state transitions deny. |
| Memory corruption | Every pointer/length is checked against Wasm memory with checked arithmetic and fixed caps before copy/effect; wasmi memory limits remain active. |
| CPU exhaustion | Existing fuel metering remains mandatory; host calls charge fixed plus per-byte fuel. Out-of-fuel triggers shared teardown. |
| Network/resource exhaustion | One socket/session/acquisition, 4-KiB I/O calls, 16-KiB TLS records/hash inputs, 32-KiB TX, 320-KiB RX, idle and absolute deadlines. No allocation grows from peer-declared lengths. |
| Singleton socket starvation | One explicit lease shared with native OpenAI. A second claimant gets `resource_busy`; it cannot abort the owner. Deadline, F12 and teardown guarantee release. |
| Slow-loris peer | Core idle and absolute deadlines apply regardless of guest loops or a peer sending one byte at a time. |
| M12+ bypass | Only `acquire.chunk_accept/finalize` may move bytes toward quarantine; finalize calls the shared existing function and never stages directly. |
| Partial/failing replacement | Pending bytes are separate from the retained candidate. Any failure drops only pending state; atomic replacement occurs after full shared finalize. |
| Durable-write escalation | W7 has no store/project/install/rollback import; every receipt says durable write attempted false. |
| Secret Broker replay | The future secret call is one-use, session/service/generation/target bound, durably audited before use and absent from W7. Repeated invocation denies. |
| SAFE/recovery misuse | Beyond-env acquisition policy is false in SAFE/recovery. The recovery lifeline has no dependency on the service and never downloads. |
| Crash while holding locks | Host functions must not call Wasm while holding `NET_STATE`, Vault or acquisition locks; cleanup uses bounded lock ordering and is idempotent. |

The complete committed W7 denial matrix remains mandatory and maps to one
owner rather than being reimplemented in the service:

| W7 denial group | Enforcing owner |
| --- | --- |
| Empty/malformed/extra command fields, caller URL/host/port/path/header, unknown source | Core request/source evaluator, before instantiation or network |
| Missing catalog, whole-hash/length/count mismatch, incomplete/invalid receiver evidence | Core request/catalog evaluator, before network |
| Missing/invalid pin, wrong SNI/SPKI, CertificateVerify/Finished failure, development bypass | Core source policy plus captured-crypto trust evaluator; socket aborted |
| Network absent/unconfigured/busy, connect/read/total timeout, peer close, F12 | Core singleton lease/deadline/kill path; pending state discarded |
| Redirect/non-200, missing/duplicate/invalid Content-Length, chunked transfer, content encoding, oversized header, wrong content type | Wasm parser evidence; `acquire.*` remains effect-denied unless the exact accepted response state is core-bound |
| Declared/body over 256 KiB, more than four chunks, chunk over 64 KiB, allocation/arithmetic overflow | Core request/acquisition caps plus bounded Wasm memory |
| Short/long/extra body | Service observes framing; core total-byte and exact-finalize checks deny and discard pending state |
| Per-chunk hash/index/duplicate/missing error | Existing shared M12+ `accept_chunk` denial |
| Whole-object hash/provenance/selection error | Existing shared M12+ finalize denial |
| Mid-transfer abort or any failure after a prior success | Shared teardown drops only pending bytes; prior retained candidate remains |
| Identical replay | Existing content-addressed result may be idempotent and grants nothing new |
| Concurrent provider/acquisition use | Core lease returns `resource_busy`; neither claimant touches the other's socket |
| Executable/source/archive/malformed content | Bytes remain inert RAM data; no build/parse-as-project/load/execute/install import exists |
| W1/W6/structured-store/ARTSTOR/rollback/durable-memory attempt | Physically absent import; receipt reports `durable_write_attempted:false` |
| Provider/API-key/Vault use by W7 | `secret_lease.*` absent from both grant and linker |
| SAFE/recovery acquisition | Beyond-env policy false for that posture before instantiation |

The largest architectural risk is not TLS parsing: it is fitting a live,
potentially blocking network session into today's one-shot cooperative Wasm
invocation without delaying F12/recovery or leaking the singleton socket after
a trap. The owner should not arm the grant until silent-peer F12 and retry
evidence prove that lifecycle.

## Slice and packet breakdown

Estimate: nine implementation slices plus one orchestrator-owned close packet.
The first seven keep `policy_allows_beyond_env:false`; slice 8 is the explicit
owner-gated authority flip.

Workers may edit code and host tests within a packet but do not build the
kernel. The orchestrator performs the full diff read, kernel compile/package,
focused QEMU profile, secret scan, commit and status/dashboard updates. Routine
multi-agent adversarial review remains omitted per owner cadence.

### 1. ABI and evidence-bound evaluator, grants nothing

Add the v1 import constants/signatures, ABI/list hashing and generalized
evidence-bound evaluator path, reusing the personal-shell evidence shape. Add
all names to the known set only with `policy_allows_beyond_env:false` tests.
No linker implementation yet.

Verification: host evaluator truth table; orchestrator runs
`m11-wasm-import-grant`. Regress existing env/ui exact lists and
`m11-4-buffer-channel` at block close.

### 2. Invocation identity, handle generations and teardown, grants nothing

Extend live-import store state with core-created invocation authority, fixed
handle slots, kill generation and one idempotent teardown path. Exercise normal
return, trap, OutOfFuel, stale/foreign handles and F12 cleanup without network
authority.

Verification: new focused `m11-beyond-env-lifecycle`; include the existing
`m8-lifeline` trap/fuel/recovery predicates in that profile. Existing
`m8-lifeline` regresses at block close.

### 3. Singleton transport ownership, grants nothing

Add one owner/generation lease around the existing TCP socket and move the
native OpenAI path onto it. Prove owner-only close/abort, busy denial, timeout
release and no cross-talk. No Wasm net import is linkable.

Verification: focused `m11-net-imports` native-owner selftest with `-Network`;
`quick -Network` regresses at block close.

### 4. `net.*` linker implementations, still ungrantable

Implement the four bounded host shims, quotas, kill checks and call evidence.
Keep every production evaluator input false. A signed probe importing one or
all functions must fail before instantiation; direct host tests exercise the
shim state machine without granting a service.

Verification: focused `m11-net-imports`; `m11-wasm-import-grant` negative
subset/linker drift assertions included in the same profile.

### 5. Opaque TLS crypto imports, still ungrantable

Implement the eight fixed crypto functions over already pinned SHA-256, P-256,
HKDF and AES-GCM dependencies, opaque keys, state transitions and sequence
counters. Add RFC 8446/8448-style primitive/record vectors and all
foreign/stale/state/size negatives. No guest receives the grant.

Verification: focused `m11-crypto-imports`; ungranted-instantiation proof plus
host vectors. Stop if implementation requires raw key export or a new
hand-written crypto primitive.

### 6. Transport-neutral acquisition seam and `acquire.*`, grants nothing

Extract the smallest shared pending-byte acceptance/finalize seam from
`PendingSerialDistribution`; keep serial output semantically identical. Add the
two host shims behind the false policy and prove they cannot stage directly.

Verification: focused `m12-distribution-provenance` augmented with serial vs
simulated-service convergence, prior-candidate preservation and exact receipt
hash equivalence. This is the risky M12 boundary and gets its existing focused
profile even though no network grant exists.

### 7. Signed acquisition/TLS service artifact, grant still denied

Build one merged `svc.net.acquire.w7` artifact: TLS handshake/record parser,
SPKI extraction, fixed GET, bounded HTTP parser and chunk driver. Reuse the
relocated X.509 SPKI/HTTP parser crates where applicable. Host-test it against a
mock ABI and fixture vectors. In VM, its exact beyond-env grant must still deny
before instantiation because policy remains false.

Verification: focused `m11-acquisition-service` with vector positives,
malformed TLS/HTTP negatives, fuel/trap cleanup and the explicit
`import_beyond_env_not_owner_authorized` result. No live network success yet.

### 8. OWNER-GATED first beyond-env arming and positive W7 path

After explicit owner approval, bind the exact signed artifact, generation, ABI,
16-import list and `local.qemu.w7` policy, and set
`policy_allows_beyond_env:true` only for that evaluated path. Run the real
e1000/DHCP/TCP/TLS/HTTP/M12+ acquisition against the ephemeral pinned fixture.
All trust labels remain pin-only/time-unvalidated and all candidate authority
flags remain false.

Verification: new focused `network-acquisition -Network`, including positive
fetch, shared chunk/finalize, retained-candidate preflight denial, F12 during a
silent peer, provider/acquisition busy in both directions, cleanup and retry.
This slice must not include secret imports.

### 9. W7 denial matrix, inspect/discard and failure preservation

Complete the committed W7 malformed request, missing/wrong pin, TLS/HTTP,
length/hash, timeout/close, busy, SAFE/recovery, prior-candidate preservation,
inspect, exact-hash discard and fail-then-valid retry matrix. Do not widen the
source policy, candidate cap or import surface.

Verification: focused `network-acquisition -Network`.

### Close packet

Orchestrator only: `quick -Network`, `m11-wasm-import-grant`,
`m11-4-buffer-channel`, `m11-6-certwindow`, `m11-7-httphead`,
`m11-8-certspki`, `m12-distribution-provenance`, `m8-lifeline`, then one
`full` and byte-identical `recovery` at block close, plus source-size,
formatting and secret scans. Any failed VM report is classified before retry.
The close packet adds no capability.

## What this lane does not deliver

- no WebPKI chain building, CA-root store, hostname-general trust or positive
  chain-valid label;
- no cryptographically trusted wall time or positive certificate-validity
  label;
- no RSA, Ed25519, post-quantum, arbitrary TLS suite or TLS 1.2 support;
- no owner-sealed import grant, provider key, service artifact or source pin;
  the first proof remains `dev_key_not_owner_sealed`;
- no durable secret provisioning on physical media, TPM auto-unlock or
  physical internal-drive Vault proof;
- no raw secret, raw TLS private key or generic entropy access in Wasm;
- no durable quarantine, project/store commit, W6 install, load, execute,
  autoload, rollback application or broad mutation;
- no public arbitrary URL fetch, redirect, DNS import, remote manifest fetch,
  archive extraction or generic HTTP client;
- no real-hardware WiFi acquisition proof; W7 is QEMU e1000 with the fixed
  host-only fixture;
- no native Stage-0 W7 fallback if the service is absent, revoked, crashed or
  denied; failure remains explicit; and
- no claim that the whole provider path has moved out of the kernel until the
  later provider service actually runs and the kernel TLS/HTTP code is deleted.

## Open owner decisions

1. Approve `raios.host_imports.v1` as a separately grant-hashed ABI id while
   retaining exact `(module,name)` pairs. Recommendation: yes.
2. Approve the first source as QEMU-only `local.qemu.w7`, fixed
   `10.0.2.2:8443`, exact SNI/path and ephemeral SPKI pin with honest pin-only,
   time-unvalidated labels. Recommendation: yes; no production public source.
3. Approve the proposed lease ceilings: 5-second connect, 15-second idle,
   90-second total, 32-KiB TX and 320-KiB RX. Recommendation: yes, tune only
   from focused evidence before arming.
4. Confirm W7 metadata and receiver evidence continue to arrive through the
   existing serial catalog while only artifact bytes use HTTPS.
   Recommendation: yes; remote-manifest parsing is a separate capability.
5. Choose the F12 execution shape after slice-2 evidence: bounded blocking
   loops with interrupt-visible kill generation, or short non-blocking/resumable
   imports. Recommendation: keep bounded blocking only if silent-peer F12 is
   proven promptly; otherwise stop and recut before net imports.
6. Explicitly approve the slice-8 artifact hash/import-list/source-policy
   arming diff. Until that approval, `policy_allows_beyond_env` stays false
   everywhere.
7. Before any provider service receives `secret_lease.*`, reconcile ADR 0012's
   current “no Wasm interface” rule and pin whether the authorized identity
   remains `svc.provider.openai_direct` or becomes a promoted replacement.
   Recommendation: amend ADR 0012 only when that provider consumer is ready;
   it does not block W7.

## Stop conditions

Stop and return to the owner if any implementation:

- flips `policy_allows_beyond_env` before the explicit arming slice or for
  more than the exact W7 service path;
- requires a caller-controlled URL, host, IP, port, SNI, path, redirect,
  header, timeout or socket option;
- requires raw ECDHE/traffic/Vault keys, a plaintext secret, or a generic
  crypto/secret dispatcher in guest memory;
- cannot prove F12/recovery kill and singleton lease release while the peer is
  silent;
- allows a second claimant to reuse or abort the active TCP socket;
- cannot converge on the existing M12+ acceptance and finalize functions, or
  stages directly into `module_candidate_intake`;
- writes any structured-store, project, ARTSTOR, W6 install, rollback or
  durable-memory state for W7;
- needs more than 256 KiB, four 64-KiB chunks, the 16-import grant, or the fixed
  TLS profile without a new owner decision;
- needs development TLS bypass, WebPKI/trusted-time overclaim, provider key,
  provider context/export or `secret_lease.*` for W7;
- touches recovery-lifeline authority or makes recovery depend on the service;
- cannot keep existing env/ui services and serial distribution behavior
  unchanged; or
- begins while the applicable full-profile Red Gate is red for guest behavior.
