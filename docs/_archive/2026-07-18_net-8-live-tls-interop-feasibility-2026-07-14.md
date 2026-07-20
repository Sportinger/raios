# NET-8 live TLS interoperability feasibility

Date: 2026-07-14

Packet: `NET-8-RECON`

Scope: read-only recon and implementation packet; no source, build artifact, or VM change was made.

## Executive verdict

The fixed raiOS `OpaqueSession` can complete a standards-conforming TLS 1.3 client handshake as-is against a controlled host server using TLS 1.3, `TLS_AES_128_GCM_SHA256`, P-256 ECDHE, and `ecdsa_secp256r1_sha256`; no defect was found in its labels, transcript boundaries, key schedule, Finished calculation, AEAD record construction, nonce sequence, or P-256 verification. The interoperability constraint is above that core: the W7 guest deliberately accepts only one narrow server flight, and NET-8 still needs a real live invocation/linker, core-held pin/source staging, and an evidence bridge into `acquire.finalize`.

Recommendation: use a small .NET 8 console fixture whose TLS implementation is `SslStream`/Windows Schannel, not a hand-written TLS server. The client offers only the approved suite, group, and signature scheme, so the server cannot silently select another profile; the fixture must also assert the negotiated protocol and cipher after the handshake.

## 1. Interoperability verdict, detail by detail

### 1.1 `OpaqueSession` is RFC 8446-correct for the fixed profile

The implementation is intentionally narrow, not a general TLS stack. Its declared profile is TLS 1.3, AES-128-GCM/SHA-256, P-256 ECDHE, and P-256 ECDSA (`raios-core/src/tls13_session.rs:1-33`). Within that profile:

| Detail | Finding |
|---|---|
| HKDF-Expand-Label | Correct encoding: output length, length-prefixed `"tls13 " + label`, and length-prefixed context (`raios-core/src/tls13_session.rs:786-812`). The fixed labels are spelled correctly: `derived`, `c hs traffic`, `s hs traffic`, `c ap traffic`, `s ap traffic`, `key`, `iv`, and `finished` (`raios-core/src/tls13_session.rs:693-766`). |
| Early/handshake/master secrets | Correct TLS 1.3 schedule: zero-PSK early extract, derived secret, ECDHE handshake extract, a second derived secret, and zero-IKM master extract (`raios-core/src/tls13_session.rs:693-766`). No PSK/resumption branch is claimed. |
| Handshake transcript boundary | Handshake traffic secrets use `Hash(ClientHello ... ServerHello)` (`raios-core/src/tls13_session.rs:340-369`). The guest supplies exactly the handshake messages without record headers (`wasm-guests/svc-net-acquire-w7/src/lib.rs:330-364`). |
| Application transcript boundary | Application traffic secrets use the transcript through the server Finished, as RFC 8446 requires (`raios-core/src/tls13_session.rs:372-407`; guest call at `wasm-guests/svc-net-acquire-w7/src/lib.rs:470-494`). The client Finished is deliberately not included in that derivation. |
| Finished | The finished key is HKDF-expanded with `finished`; verify data is HMAC-SHA256 over the supplied transcript hash (`raios-core/src/tls13_session.rs:410-457,814-835`). Server Finished is checked over the transcript through CertificateVerify, and client Finished is produced over the transcript through server Finished (`wasm-guests/svc-net-acquire-w7/src/lib.rs:446-500`). |
| CertificateVerify message | The guest constructs 64 spaces, `TLS 1.3, server CertificateVerify`, a zero separator, and the transcript hash through Certificate (`raios-w7-acquire-logic/src/lib.rs:389-399`). That is the exact server CertificateVerify input. |
| Signature scheme | CertificateVerify must encode scheme `0x0403`; its signature is parsed as DER ECDSA and verified by P-256/SHA-256 (`raios-w7-acquire-logic/src/lib.rs:374-383`; `raios-core/src/tls13_session.rs:305-337`). The SHA-256 pin is over the exact leaf SubjectPublicKeyInfo DER, not the full certificate. |
| Record header/AAD | The outer record must be application-data type 23, legacy version `0x0303`, with a self-consistent bounded length (`raios-core/src/tls13_session.rs:671-685`). The five-byte header is passed as AEAD AAD (`raios-core/src/tls13_session.rs:460-543`). |
| Record plaintext | The caller supplies TLSInnerPlaintext (`content || content_type || optional zero padding`). The guest emits content type 22 for client Finished and 23 for HTTP; its receive path removes zero padding and accepts inner types 21/22/23 (`wasm-guests/svc-net-acquire-w7/src/lib.rs:487-518`; `raios-w7-acquire-logic/src/lib.rs:401-411`). |
| Nonce and sequence | The 64-bit record sequence is big-endian, left-padded to 12 bytes, and XORed into the static IV (`raios-core/src/tls13_session.rs:837-843`). Send/receive counters advance only after successful cryptographic operations; both directions reset when application traffic keys take over (`raios-core/src/tls13_session.rs:372-407,460-543`). |
| Client Finished key epoch | The client derives application keys after validating server Finished but seals client Finished with the client handshake traffic key. The seal transition then resets the application send sequence (`raios-core/src/tls13_session.rs:460-511`). This subtle ordering is correct. |
| Vector evidence | The module contains an RFC 8448 key-schedule/record vector (`raios-core/src/tls13_session.rs:1062-1195`), so the conclusion is not based only on visual similarity to the RFC. |

No core deviation was found that requires a pre-NET-8 crypto repair slice.

### 1.2 Exact guest offer and accepted server behavior

The W7 ClientHello is deterministic apart from its random and P-256 key share (`raios-w7-acquire-logic/src/lib.rs:162-222`):

- legacy record version `0x0301` and ClientHello legacy version `0x0303`;
- empty legacy session ID;
- exactly one cipher suite, `0x1301` (`TLS_AES_128_GCM_SHA256`);
- null legacy compression;
- SNI `w7.test.raios`;
- supported version TLS 1.3 only;
- supported group P-256 only;
- signature algorithm `ecdsa_secp256r1_sha256` only;
- one uncompressed P-256 key share.

The server parser requires legacy version `0x0303`, an empty session-ID echo, suite `0x1301`, null compression, selected version TLS 1.3, and a P-256 key share (`raios-w7-acquire-logic/src/lib.rs:266-314`). It permits additional well-formed ServerHello extensions and arbitrary well-formed EncryptedExtensions (`raios-w7-acquire-logic/src/lib.rs:300-345`).

The encrypted handshake state machine requires exactly EncryptedExtensions, Certificate, CertificateVerify, and Finished; it does not implement HelloRetryRequest, client-authentication CertificateRequest, PSK, or an alternate certificate/signature path (`wasm-guests/svc-net-acquire-w7/src/lib.rs:366-504`). It does handle:

- the optional TLS 1.3 compatibility ChangeCipherSpec (`wasm-guests/svc-net-acquire-w7/src/lib.rs:370-374`);
- handshake messages fragmented across records or coalesced in one record;
- extra certificates after the first leaf certificate;
- post-handshake NewSessionTicket messages, which it parses and ignores (`wasm-guests/svc-net-acquire-w7/src/lib.rs:596-605`).

Therefore a standards server interops if it accepts the offered P-256 share directly, uses the ECDSA P-256 leaf key for CertificateVerify, does not request a client certificate, and does not require ALPN. A server that sends HelloRetryRequest, selects X25519/RSA/a different cipher, requires ALPN, or requests client authentication is correctly outside the fixed profile rather than evidence of a broken crypto core.

The server certificate must keep the entire encrypted handshake transcript under the core's 16 KiB bound. A one-certificate, self-signed P-256 chain is comfortably below it (`raios-core/src/tls13_session.rs:305-337`). The local X.509 extractor accepts the normal v3 certificate structure and requires the standard EC public-key, P-256, and uncompressed-point SPKI encodings (`raios-x509-spki/src/lib.rs:81-141`).

### 1.3 HTTP contract

The request is exactly (`raios-w7-acquire-logic/src/lib.rs:413-435`):

```text
GET /raios/cas/sha256/<lowercase-whole-sha256> HTTP/1.1\r\n
Host: w7.test.raios\r\n
Accept: application/octet-stream\r\n
Connection: close\r\n
\r\n
```

The response must be `HTTP/1.1 200`, contain exactly one decimal `Content-Length` matching the staged artifact length, and use `Content-Type: application/octet-stream`. Transfer-Encoding, Content-Encoding, a conflicting/duplicate Content-Length, excess bytes in the parsed body, or a non-200 status is rejected (`raios-w7-acquire-logic/src/lib.rs:450-501`; receive path at `wasm-guests/svc-net-acquire-w7/src/lib.rs:521-594`).

## 2. Fixture choice and exact design

### 2.1 Choose .NET 8 `SslStream`

Use a minimal application fixture atop `SslStream`; do not implement a minimal TLS protocol server. The purpose of NET-8 is standards interoperability. A hand-written peer tailored to the guest could accidentally reproduce a guest mistake and turn the proof into a closed-system vector.

This machine has .NET 8 SDK/runtime support. Repository searches found no existing `SslStream`, `X509Certificate`, or `CertificateRequest` fixture in `vm-harness` or `scripts`; the nearby harness code provides TCP listener/client and QEMU-monitor patterns, but not a reusable TLS server. The fixture must be a `net8.0` console project, not Windows PowerShell 5.1 `Add-Type`: the latter runs on .NET Framework and does not expose `SslServerAuthenticationOptions`.

On Windows, .NET 8's `CipherSuitesPolicy` is not a portable way to force TLS 1.3 suite `0x1301`; its reference contract documents platform limitations. That does not weaken this fixture because the ClientHello offers only `0x1301`. Configure TLS 1.3 only, allow the handshake, then fail the fixture unless both `SslStream.SslProtocol == Tls13` and `SslStream.NegotiatedCipherSuite == TLS_AES_128_GCM_SHA256`. There is likewise no Windows `SslStream` option that directly chooses the ECDHE curve; the client offers only P-256 and supplies a P-256 share, so negotiation either uses P-256 or fails. The guest's ServerHello parser is the independent assertion of that group.

### 2.2 Certificate and server settings

At fixture startup:

1. Create `ECDsa.Create(ECCurve.NamedCurves.nistP256)`.
2. Create a short-lived self-signed certificate with `CertificateRequest`, subject `CN=w7.test.raios`, SHA-256, a SAN DNS entry `w7.test.raios`, digital-signature key usage, and server-auth EKU.
3. Make it valid from approximately five minutes before startup until one hour after startup. No trust-store install is needed: raiOS authenticates the exact SPKI pin, not a CA chain or host wall clock.
4. Export `SubjectPublicKeyInfo` from that same ECDSA key and compute `SHA-256(SPKI DER)`. Keep the private key in process memory only.
5. Listen for one bounded run on the host fixture port and wrap the accepted `NetworkStream` in `SslStream`.
6. Authenticate with `EnabledSslProtocols = Tls13`, `ServerCertificate` set, `ClientCertificateRequired = false`, `CertificateRevocationCheckMode = NoCheck`, no ALPN protocols, and `AllowTlsResume = false`. Do not set `CipherSuitesPolicy` on Windows.
7. Assert the negotiated protocol and cipher, read and compare the exact bounded HTTP request, serve only the one configured artifact, then close the TLS connection.

The planned guest endpoint remains `10.0.2.2:8443`, SNI `w7.test.raios`, and the fixed content-addressed path (`docs/plan-reviews/w7-quarantined-network-acquisition-scope-2026-07-14.md:138-170`). QEMU user networking already uses `-netdev user` plus e1000 (`scripts/run-stage0-qemu.ps1:113-117`). The fixture should first bind loopback and let the focused profile prove the slirp route; if this Windows/QEMU combination cannot reach a loopback listener as `10.0.2.2`, classify that as `host-transport` and stop to choose an explicit host-interface binding/firewall rule. Do not silently broaden the bind to all interfaces.

### 2.3 Artifact, ready file, and cleanup

Serve the existing inert echo artifact so the live path converges with the already-built shared acquisition finalizer:

- file: `seed-kernel/artifacts/svc.demo.echo.wasm`;
- length: `4205` bytes;
- whole SHA-256: `f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2`;
- one chunk, with chunk 0 equal to the whole file and the remaining three chunk slots zero;
- path: `/raios/cas/sha256/f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2`.

The fixture writes an atomic, per-run ready JSON only after the listener is active. It contains schema/version, SNI, host port, path, artifact length/hash, lowercase SPKI SHA-256, process ID, and a random run ID. It must not contain the private key, a provider secret, or a reusable certificate bundle. The profile waits for and validates this ready file, uses its pin only to build a temporary ignored test image/configuration, and removes the ready file, process, and temporary image in `finally` cleanup.

## 3. Request stager

### 3.1 What crosses into the guest

`RW7REQ01` is already the fixed guest-input ABI (`raios-w7-acquire-logic/src/lib.rs:12-20,66-115`). For this artifact the core must stage exactly 271 bytes:

| Offset | Value |
|---|---|
| `0..8` | ASCII `RW7REQ01` |
| `8` | SNI length `13` |
| `9..11` | big-endian path length `82` (`0x0052`) |
| `11..15` | big-endian total length `4205` (`0x0000106d`) |
| `15` | chunk count `1` |
| `16..48` | whole SHA-256 `f81f...abd2` |
| `48..80` | chunk 0 SHA-256, also `f81f...abd2` |
| `80..176` | three zero chunk-hash slots |
| `176..189` | ASCII `w7.test.raios` |
| `189..271` | ASCII fixed CAS path above |

The guest validates that SNI is exactly `w7.test.raios`, the path is exactly the lowercase whole-hash-derived path, chunk geometry is nonzero and bounded, used hashes are nonzero, and unused hash slots are zero (`raios-w7-acquire-logic/src/lib.rs:66-159`).

### 3.2 What must remain core-held

The source policy and SPKI pin must **not** be added to `RW7REQ01` and must never be guest-readable:

- source policy ID: `local.qemu.w7`;
- endpoint: `10.0.2.2:8443`;
- SPKI pin: the per-run 32-byte value from the fixture ready file;
- approved W7 service artifact hash, import-list hash, ABI, and exact 16-import set;
- expected artifact/chunk metadata duplicated only as core authority/evidence, not trusted because the guest supplied it.

`CryptoInvocationState` already holds `expected_pin_sha256` and passes it to `OpaqueSession` without exposing it through the ABI (`seed-kernel/src/wasm_runtime/crypto_shims.rs:122-158,426-505`). NET-8 should construct that state from the approved staged source object. The live net state should likewise be constructed from the fixed endpoint rather than accepting a guest-selected address.

Put the authoritative `W7SourcePolicy`, `StagedW7Request`, approved-grant evaluation, and request encoder in `seed-kernel/src/wasm_runtime/acquisition_service.rs`. `seed-kernel/src/agent_protocol_wasm.rs` may expose the one typed start/report operation, but it must hand only an already validated request to the acquisition service; it must not create a second authorization decision. `seed-kernel/src/wasm_runtime/invocation.rs` should consume the approved object to create one instance, link exactly the 16 authorized imports, and own start/resume/teardown.

Two existing fixture-only state shapes must be made real rather than reused deceptively:

- `NetInvocationState` currently models DNS/silent test scenarios, not a live fixed source (`seed-kernel/src/wasm_runtime/net_shims.rs:30-35`). Add a distinct exact-policy live source variant.
- `AcquireInvocationState` currently has a fixture service ID, three chunk expectations, and a preset `source_tls_evidence_valid` boolean (`seed-kernel/src/wasm_runtime/acquire_shims.rs:12,155-208,517-559`). W7 supports four chunks. Its live state needs four slots plus a count, the real service identity, and TLS evidence derived from the successful core crypto session—not a boolean initialized true.

The latter is the most important trust-boundary change: `acquire.finalize` may become eligible only after the same opaque session has proved the expected SPKI, CertificateVerify math, server Finished, and application-key transition. A profile assertion or guest return code is not authority for that fact.

## 4. The tightly bounded arming flip

The one current construction site is `seed-kernel/src/wasm_runtime/acquisition_service.rs:97-123`; the false bit is line 122. The approved literal identities are:

- W7 service artifact SHA-256: `32a018b0c730a4f85210ca820483ca68f8a4d0715021a1dda97951fe305e9e54`;
- ordered ABI-v1 import-list SHA-256: `eb390ec5c2dfde5ac632b127515c5101c812ed6ca209191846bc762409bf4345`;
- source policy ID: `local.qemu.w7`.

Add separate literal `[u8; 32]` approval constants for the first two values. Do not define the approval hash by aliasing the build-generated artifact hash or by hashing the current list and calling the result approved; the comparison must be capable of failing after artifact/import drift.

At that site, and nowhere else, construct:

```rust
let policy_allows_beyond_env =
    artifact_sha256 == NET_8_APPROVED_W7_ARTIFACT_SHA256
        && import_list_sha256 == NET_8_APPROVED_W7_IMPORT_LIST_SHA256
        && source_policy_id == "local.qemu.w7";
```

Pass that single value to `evaluate_evidence_bound_wasm_import_grant` and to the live `InvocationAuthority`; do not independently change `seed-kernel/src/wasm_runtime/invocation.rs:309` or the fixture denials in `acquire_shims.rs:290`. The construction cannot authorize another service because it sits in the W7-only acquisition service, whose evaluator input fixes `service_id` to `svc.net.acquire.w7`, artifact bytes to the signed W7 artifact, and linker implementations to the exact W7 import set (`seed-kernel/src/wasm_runtime/acquisition_service.rs:12-31,97-123`). Preserve that structural isolation rather than generalizing the function to accept an arbitrary service ID.

`seed-kernel/src/agent_protocol_wasm.rs:1293-1338` currently reports the W7 probe with a hard-coded false. It should report the result returned by this one decision, not construct another true value. A source grep for `policy_allows_beyond_env =` (or a deliberately unique approval helper name) must find exactly one true-producing W7 site; all unrelated services remain false. The focused negative predicate below is still required because grep is structural evidence, not runtime proof.

## 5. NET-8 implementation slice

### 5.1 Expected write set

Keep `raios-core/src/tls13_session.rs` and the W7 guest unchanged unless live standards interop produces a reproducible core/guest defect. Expected files are:

- `seed-kernel/src/wasm_runtime/acquisition_service.rs`: approval constants/conjunction, source policy, input encoder, live state/evidence ownership;
- `seed-kernel/src/wasm_runtime/invocation.rs`: exact W7 production instance, 16-import linker, lifecycle and cleanup;
- `seed-kernel/src/wasm_runtime/net_shims.rs`: exact live source variant and lease-bound socket behavior;
- `seed-kernel/src/wasm_runtime/crypto_shims.rs`: only the narrow live-session evidence export to core state; no algorithm/key-schedule change;
- `seed-kernel/src/wasm_runtime/acquire_shims.rs`: four-chunk live state, real service authority, shared finalizer convergence, crypto-evidence binding;
- `seed-kernel/src/wasm_runtime.rs`, `seed-kernel/src/agent_protocol_wasm.rs`, and, only if dispatch requires it, `seed-kernel/src/main.rs`: exports and one typed start/status path;
- the existing build/package path, minimally extended to accept the per-run W7 SPKI pin only for a temporary test image, with the same secret-safe staging discipline as provider-key images;
- a small `vm-harness` .NET 8 fixture project and PowerShell process wrapper;
- `vm-harness/shadow-vm-smoke.ps1` plus one focused network-acquisition profile wrapper/report schema;
- `docs/PROJECT_STATUS.md`, `docs/ROADMAP.md`, and `docs/OWNER_DASHBOARD.md` after evidence exists.

`shadow-vm-smoke.ps1` must add the profile to its profile set, require `-Network`, and allocate a monitor TCP port in the same way as existing monitor-driven profiles (`vm-harness/shadow-vm-smoke.ps1:9-13,72-73,171-184,211-216`).

### 5.2 Focused profile predicates

The NET-8 focused profile should require all of the following in one report:

1. **Positive live fetch:** e1000 initialization and DHCP succeed; the guest connects to `10.0.2.2:8443`; the fixture observes the exact GET; raiOS proves TLS 1.3, suite `0x1301`, P-256 CertificateVerify math, matching ephemeral SPKI pin, server Finished, HTTP 200/type/length, every chunk hash, and the whole hash.
2. **Shared-finalize convergence:** live W7 bytes enter the existing acquire finalizer and produce the same inert current-boot retained candidate/receipt semantics as the already-tested native acquisition route. No W7-private success store is allowed.
3. **Retained-candidate preflight denial:** after successful retrieval, load/install/execute remain denied and the receipt names the still-missing activation evidence; no candidate execution, install, durable write, rollback mutation, or provider auto-load occurs.
4. **F12 during a silent peer:** the fixture accepts TCP and then remains silent at a deterministic phase. A physical F12 injected through the QEMU monitor cancels promptly, does not resume the invocation, zeroizes/removes crypto session state, closes TCP, releases the transport lease, discards incomplete bytes, and preserves any previously retained candidate.
5. **Provider/acquisition busy, both directions:** an active provider lease makes W7 acquisition return the typed busy denial; an active W7 acquisition makes provider transport return the same shared-lease busy class. Neither path steals or duplicates the lease.
6. **Cleanup and retry:** after peer silence, guest trap, out-of-fuel, malformed response, and user cancellation as applicable to this slice, teardown happens exactly once and a valid live request succeeds in the same boot.
7. **Different service still denied:** invoke a different signed service (or the same beyond-env import shape under a different service identity) and prove denial before instantiation with zero network, crypto, acquisition, candidate, and durable effects. Also prove artifact-hash, import-list-hash, and source-policy mismatches independently keep the W7 bit false.

Do not turn NET-8 into the full NET-9 negative matrix. These negatives are necessary to prove the arming boundary, lease/cancel behavior, and retryability of the newly live path.

### 5.3 Orchestrator versus worker ownership

The implementation worker writes the bounded kernel, fixture, harness, and tests and returns the exact diff plus targeted host-test instructions. The orchestrator owns the careful full-diff read, reproducible build, any artifact-generation/signing step, temporary pin-bearing image packaging, focused VM run, failure classification, secret scan, report selection, commit, and dashboard/status update.

The committed W7 guest artifact is already signed and pinned. Because this recon found no guest or crypto-core change necessary, NET-8 should not rebuild/re-sign it merely to exercise the path; the orchestrator should verify the existing bytes/hash and fail on drift. If a guest change becomes unavoidable, stop this slice, rebuild through the canonical pipeline, update all signed descriptors and explicit owner-approved pins, and re-establish NET-7 evidence before arming.

## 6. Risks and stop conditions

### Primary risks

1. **Live evidence binding, not primitive crypto:** the current acquire fixture initializes `source_tls_evidence_valid` as a scenario boolean. Carrying that shortcut into production would allow the finalizer to trust staged intent rather than the observed opaque TLS session. NET-8 must make this an unforgeable core-state transition.
2. **Unobserved Windows Schannel flight:** the fixed guest should accept standard Schannel output, including compatibility CCS and tickets, but no live flight has yet been observed on this machine. The focused run is the necessary moment-of-truth proof. The guest also has a transport-edge risk: its receive pump waits when fewer than two bytes are available (`seed-kernel/src/wasm_runtime/net_shims.rs:480-488`), so a one-byte TCP delivery can stall even though TLS is correct.
3. **Platform configuration limits:** Windows does not permit relying on `CipherSuitesPolicy` to force `0x1301`, nor does `SslStream` expose a direct curve selector. The one-suite/one-group ClientHello plus post-handshake assertions are the honest constraint mechanism.
4. **Ephemeral pin custody:** the pin is not secret, but it is authority-bearing for this run. It must enter only a temporary test build/image and remain absent from the normal release image. The fixture private key must never leave process memory or appear in the ready file.
5. **Host routing/firewall:** `10.0.2.2` to a Windows loopback listener is a host-transport assumption to prove, not silently reinterpret. A failure here does not justify changing TLS or broadening the listener.

### Stop conditions

- If a reproducible live trace shows a wrong HKDF label, transcript boundary, Finished value, nonce/sequence, AEAD AAD, or CertificateVerify calculation in `OpaqueSession`, stop before arming and open a separate crypto-interop repair slice with an external standards vector. Do not make the fixture mimic the defect.
- If Schannel cannot negotiate the exact offered standard profile, stop and diagnose the Windows TLS provider/certificate construction. A second standards implementation may be used as an independent oracle, but a non-standard guest-shaped TLS server is not acceptable proof.
- If the live acquire path cannot derive source TLS evidence from the opaque core session, stop rather than preset or guest-assert the evidence bit.
- If the three-part approval conjunction is generalized to arbitrary services, duplicated at another true-producing site, or based on self-moving generated identities, stop the arming change.
- If the test requires committing a pin-bearing image, certificate private key, provider secret, broad host listener, fake success response, or durable/install authority, stop and reduce it to the approved ephemeral current-boot boundary.

## Decision packet

- **Interop:** SHIP to live NET-8 testing with the crypto core unchanged. The fixed guest is narrow but standards-conforming for the approved server profile.
- **Fixture:** .NET 8 `SslStream`/Schannel with an in-memory self-signed P-256 certificate, one offered/negotiated TLS suite, post-handshake assertions, one exact CAS path, and an atomic pin ready file.
- **Arming site:** replace the false at `seed-kernel/src/wasm_runtime/acquisition_service.rs:122` with the single literal artifact-hash + literal import-list-hash + exact `local.qemu.w7` conjunction, then pass that value into the W7-only evaluator/runtime.
- **Largest risk:** accidentally treating staged fixture intent as TLS evidence at `acquire.finalize`; the live core session must be the only source of the pin/math/Finished evidence bit.
