# M11-8-SCOPE: Next Parser Ownership Move (2026-07-14)

## Recon verdict and numbering collision

Current HEAD already contains an M11-8 relocation that the roadmap/status cursor
does not record:

- `fdb1ce2` extracted the live P-256 SPKI parser into the no-dependency crate
  `raios-x509-spki`.
- `802ee6f` added the signed `svc.demo.certspki` guest and the independent core
  cross-check.
- `vm-harness/shadow-vm-smoke-profile-m11-8-certspki.ps1` exists, and
  `release/vm-reports/shadow-20260708-223318-28048.json` records a passing
  `m11-8-certspki` run.

Therefore the next implementation must not reuse M11-8. This document keeps the
requested filename and packet id, but calls the proposed implementation **M11-9
DNSPARSE**. The orchestrator must resolve the stale docs and approve that number
before dispatch. Reusing `m11-8-*` is a stop condition.

Capability sentence: **a signed, fuel- and memory-bounded current-boot Wasm
service can parse a bounded DNS A response through only the three byte-buffer
imports, while the core independently parses the same bytes and retains all
authority; the live DNS path remains behaviorally identical and fail-closed.**

## 1. Remaining pure internet-facing parser inventory

Line spans are current HEAD, not the 2026-07-08 planning baseline. Approximate
LOC counts include directly supporting private helpers but not tests.

| Surface | Current owner and span | Approx. LOC | Untrusted bytes and result | Current call sites | Existing coverage |
| --- | --- | ---: | --- | --- | --- |
| DNS query codec and A-response parser | `seed-kernel/src/net.rs:691-852` (`build_dns_query`, `parse_dns_response`, `read_dns_name`, result type) | 162 | Raw UDP DNS payload plus expected transaction id/hostname -> IPv4 + effective TTL | `NetState::poll_dns` at `net.rs:438-472`; query construction at `net.rs:474-507`; ultimately used by `openai.rs:278` | Only incidental live `openai-direct-smoke.ps1`; no deterministic VM DNS fixture and no host tests |
| HTTP chunked-body decoder | `seed-kernel/src/openai.rs:1466-1490` | 25 | TLS-authenticated HTTP body -> de-chunked body bytes | `perform_https_request` at `openai.rs:1368` | Live OpenAI smoke only; M11-7 `httphead` covers headers/completion, not body decoding |
| Provider JSON string extraction | `seed-kernel/src/openai.rs:1492-1590` | 99 | Provider JSON response/error body -> status, error text, answer text | `openai.rs:1356`, `1369`, `1374` | Live OpenAI smoke only; no malformed/surrogate/truncation fixture profile or host crate |
| TLS record and handshake parsing | Vendored `embedded-tls`: `parse_buffer.rs`, `record.rs`, `record_reader.rs`, `alert.rs`, `handshake/*.rs`, `extensions/**/*.rs` | about 1,300 parser/state LOC within 2,814 total lines in those files | Raw TCP TLS records -> handshake/application records, certificate messages, extensions, alerts | `openai.rs:1237-1258` and TLS reads at `1348`; `tls_io.rs` supplies transport only | Vendored unit tests plus live pinned-cert/SPKI OpenAI smoke; no narrow raiOS Wasm cross-check |
| Marvell legacy scan-response parser | Already outside `seed-kernel/src`: `raios-core/src/marvell_wifi_cmd.rs:546-651`; kernel DMA wrapper `marvell_wifi_pcie.rs:3010-3035` | 106 pure core LOC; 26 kernel wrapper LOC | Firmware mailbox response -> BSS descriptors | `parse_scan_dma_response`, then synthetic 802.11 frames are passed to `wifi::ingest_scan_frame` | Strong `raios-core` host tests; no real-radio VM fixture |
| 802.11 beacon/probe-response parser | Already outside `seed-kernel/src`: `raios-core/src/dot11_scan.rs:95-204`; kernel consumer `wifi.rs:326-350` | 110 | Radio scan frame -> SSID/channel/security | Live Marvell scan ingestion and self-test scan fixtures | `raios-core` host fixtures cover open/WPA/WPA2/WPA3/truncation; not a signed Wasm service |

Not candidates for this ownership move:

- `marvell_wifi_pcie.rs:3038-3157` reads event/RX DMA buffers through unsafe
  driver-owned pointers. The byte interpretation could later be split from DMA
  access, but the current functions are not pure bytes-in/values-out seams.
- smoltcp owns Ethernet/IPv4/UDP/TCP/DHCP parsing below the application layer.
  The original M11 map and ADR 0005 deliberately keep this fixed-shape,
  driver-adjacent path native.
- `openai_trust.rs` no longer owns local X.509 validity or SPKI parsing. Those
  are the completed `raios-x509-time` and `raios-x509-spki` moves. P-256
  signature DER decoding is provided by the pinned crypto dependency, not a
  seed-kernel parser block.
- `read_http_response` at `openai.rs:1431-1464` is an I/O/timeout loop, not a
  pure parser. Its completion predicate already lives in `raios-http-parse`.

## 2. Chosen target

**Choose DNS query/response parsing.** It removes about 150 net lines from
`seed-kernel/src` after the small smoltcp adapter, is the largest remaining
self-contained pure parser still physically owned by a seed-kernel source file,
and handles unauthenticated raw UDP bytes before TLS establishes provider
identity. Its interface is small (bounded payload + transaction id + hostname
in, optional four-byte address + TTL out), it needs no host import beyond the
proven three-buffer envelope, and the 2026-07-06 and 2026-07-08 M11 maps both
name DNS as the first post-M11 candidate (`net.rs:579-735` in the old layout;
that span resolves to current `net.rs:691-852`). Risk is moderate and localized:
DNS compression pointers and truncated counts need adversarial fixtures, but no
trust, secret, persistence, network-import, or provider-authority design changes.

**Runner-up: HTTP chunked-body plus provider JSON extraction**
(`openai.rs:1466-1590`, about 125 LOC). It has high semantic exposure and the
same three-import feasibility, but its output is variable-length user-visible
text, its current ad-hoc key search is not a general JSON parser, and combining
chunk framing with provider-specific extraction creates a less crisp first
ABI. Move it next as one provider-body parser crate, not as two tiny services.
TLS is higher value by total attack surface but is not the next slice: it needs
the beyond-`env` crypto/network/time/secret imports and secret-custody work that
this packet explicitly keeps denied.

## 3. Exact M11-9 DNSPARSE slice design

### 3.1 Standalone parser crate and byte-identical live path

Create `raios-dns-parse`, edition 2021, with
`#![cfg_attr(not(test), no_std)]`, no normal dependencies, and `sha2` only as a
dev-dependency for pinned fixture provenance. `raios-core` depends on it and
re-exports it exactly as:

```rust
pub use raios_dns_parse as dns_parse;
```

Move these behaviors from `net.rs`:

- `DNS_DEFAULT_TTL_SECS`;
- `build_dns_query` without semantic edits;
- `parse_dns_response` and DNS-name compression traversal;
- the result model, represented dependency-free as
  `DnsARecord { address: [u8; 4], ttl: u32 }`;
- fixed guest input/output record encoders and fail-closed decoders described
  below.

The crate must not depend on smoltcp or `alloc`. Replace the private
`read_dns_name -> String` allocation with a streaming comparison against the
expected hostname while preserving the existing bounds checks and 16-pointer
jump cap. This removes allocation without changing accepted live fixtures.

`net.rs` imports `build_dns_query` from `raios_core::dns_parse`, so the existing
query call remains byte-identical. Keep a tiny kernel-local adapter named
`parse_dns_response` that calls `raios_core::dns_parse::parse_dns_response` and
converts `[u8; 4]` to smoltcp `Ipv4Address`. Consequently the live call remains
exactly:

```rust
parse_dns_response(payload, query.tx_id, &query.hostname)
```

and the cache/logging call sites remain unchanged. The adapter is necessary:
putting smoltcp in the new crate would violate the no-dependency/Wasm rule.
Expected seed-kernel deletion is at least 145 net lines. If equivalence needs a
larger kernel compatibility layer, stop rather than claim a measured move.

### 3.2 Signed guest and byte ABI

Add `wasm-guests/svc-demo-dnsparse`, depending directly on
`raios-dns-parse`. Service id: `svc.demo.dnsparse`. Entrypoint:
`raios_service_main`. Authorized imports, in this exact order, are only:

```text
env.input_len
env.input_read
env.output_write
```

`policy_allows_beyond_env` stays `false`.

The host-to-guest input is one exact-length record:

| Offset | Bytes | Meaning |
| ---: | ---: | --- |
| 0 | 4 | ASCII `DNSQ` |
| 4 | 1 | version `1` |
| 5 | 1 | reserved, must be zero |
| 6 | 2 | expected DNS transaction id, big-endian |
| 8 | 1 | hostname byte length (0-253) |
| 9 | 2 | DNS payload length, big-endian |
| 11 | variable | hostname UTF-8 bytes, followed immediately by DNS payload |

The decoder requires exact total length, valid UTF-8, valid DNS label lengths,
and a payload no larger than the live `UDP_BUFFER_LEN` (512). Invalid framing
produces the canonical no-answer record; it never traps and never grants.

The guest-to-host output is a fixed 16-byte record:

| Offset | Bytes | Meaning |
| ---: | ---: | --- |
| 0 | 4 | ASCII `DNSR` |
| 4 | 1 | version `1` |
| 5 | 1 | status: `0` no matching A answer, `1` matching A answer |
| 6 | 4 | IPv4 octets; all zero when status is `0` |
| 10 | 4 | effective TTL, big-endian; zero when status is `0` |
| 14 | 2 | reserved, must be zero |

The kernel decoder rejects any wrong length, magic, version, status, reserved
byte, inconsistent zero fields, or an answer with zero effective TTL. A hostile
guest output therefore cannot panic the kernel or become an address.

### 3.3 Core cross-check and authority shape

Add the read-only method `wasm.dnsparse_probe`, rendered through the existing
typed record model as `raios.wasm_dnsparse_probe.v0`. It is additive test
infrastructure, not a live DNS routing change.

For every case, the host stages the same encoded input for the guest, then the
core independently calls `raios_core::dns_parse::parse_dns_response` over the
original payload/transaction-id/hostname and encodes its own 16-byte result.
Comparison requires both decoded values and exact encoded output SHA-256 to
match.

Required in-guest cases:

1. Happy path: a pinned response for `api.openai.com` with a compressed answer
   name, matching transaction id, one IPv4 address, and nonzero TTL. Guest and
   core must match address, TTL, record bytes, and output hash.
2. Truncated path: the same response cut inside the answer/RDATA. Guest and core
   must both return canonical no-answer, and the VM stays alive.
3. Compression-loop path: a name pointer cycle. Guest and core must both stop at
   the existing 16-jump ceiling and return canonical no-answer.
4. Import denial: run the real artifact with only `env.input_len` authorized.
   It must fail before instantiation with zero output.

The response must state and the profile must assert:

```text
guest_output_is_evidence_only=true
core_is_authority=true
policy_allows_beyond_env=false
owner_sealed=false
trust_tier=dev_key_not_owner_sealed
authorizes_provider_request=false
authorizes_provider_export=false
authorizes_dns_cache_update=false
durable_write=false
capability_granted=false
```

The live `poll_dns` path continues using the independently compiled core parser.
Guest absence, invalid output, fuel exhaustion, trap, or mismatch yields only
probe failure evidence; it never updates the DNS cache. No fallback is needed
because the guest is evidence, not authority.

### 3.4 Signing and build gates

Copy the certwindow/httphead/certspki descriptor shape exactly:

- built-in artifact identity and current-boot load descriptors;
- separate P-256 signatures over both descriptor sources;
- `trust_tier=dev_key_not_owner_sealed`, `scope=current_boot`,
  `classification=local_only`, `persistence=none`;
- exact authorized import list/count of three;
- external artifact intake/load, executable-page mapping, durable writes,
  persistent install, and rollback install all false.

`seed-kernel/build.rs` must verify both signatures and hard-assert every field,
including the dev-key tier and all false authority/durability fields, before it
generates Rust constants. It must also bind the descriptor to the actual Wasm
SHA-256. Add the guest to the cargo-build branch of
`scripts/build-wasm-guest.ps1`; do not compile `raios-core` for wasm32.

### 3.5 Fuel and memory bounds

- Fuel: `DNSPARSE_WASM_FUEL_BUDGET = 1_000_000`, matching the three completed
  parser guests. The profile requires `0 < fuel_used < fuel_budget`.
- Staged input/output: existing host caps remain 4,096 bytes; the guest uses
  fixed `IN: [u8; 4096]` and `OUT: [u8; 16]`. Live DNS payloads remain capped at
  512 bytes by `net.rs`.
- Linear memory: cap buffer-service stores at 2 MiB, one instance, one memory,
  zero tables, and attach the store limiter before instantiation.

Recon found that `buffer_state` currently builds default `StoreLimits` and
`execute_validated_module_bytes` does not attach `store.limiter`; wasmi's
default linear-memory limit is explicitly unlimited. The smallest honest fix is
one shared runner hardening change: configure the 2 MiB/1/1/0 limits in
`buffer_state` and call `store.limiter` immediately after creating the store.
This narrows authority but touches bufecho/certwindow/httphead/certspki, so their
focused regressions are mandatory. If any embedded guest declares an initial
memory above 2 MiB, stop and measure it; do not silently raise the cap.

## 4. Verification plan

No cargo/build command was run during this read-only recon.

### Host checks

The new crate gets direct tests for:

- pinned query bytes for `api.openai.com` and a fixed transaction id;
- compressed and uncompressed matching A answers;
- zero TTL mapping to `DNS_DEFAULT_TTL_SECS`;
- wrong transaction id, response bit absent, nonzero RCODE, wrong question
  hostname, wrong type/class, and no A answer;
- truncation at every header/question/answer boundary;
- compression pointer out of range and a pointer cycle exceeding 16 jumps;
- maximum label length, empty/oversize labels, output-buffer exhaustion;
- input-record and output-record round trips plus bad magic/version/length/
  status/reserved/inconsistent fields;
- pinned fixture SHA-256 using dev-only `sha2`.

The orchestrator runs:

```powershell
cargo test --locked -p raios-dns-parse -p raios-core
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
```

Workers write code and tests but do not run or repair the seed-kernel compile
loop in their sandbox.

### Focused VM profile and needles

Add the exact profile name `m11-9-dnsparse`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile m11-9-dnsparse -TimeoutSeconds 180 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

Required predicates:

- `m11-9-dnsparse:positive-crosscheck` — service id, three exact imports,
  success, pinned input hash, fixed output length, guest/core address+TTL match,
  encoded bytes/hash match, fuel bounded;
- `m11-9-dnsparse:truncated-agree` — both return canonical no-answer;
- `m11-9-dnsparse:pointer-loop-agree` — both hit the bounded fail-closed result;
- `m11-9-dnsparse:refused-unless-granted` — partial import list fails before
  instantiation with zero output;
- `m11-9-dnsparse:grants-nothing` — every authority/trust/durable flag above is
  false and the dev-key label is exact.

Because the explicit memory limiter changes the shared byte-buffer runner, run
`m11-buffer-channel`, `m11-6-certwindow`, `m11-7-httphead`, and
`m11-8-certspki` once after the combined compile loop. Run `full` plus
`recovery` only if the orchestrator declares this the M11 parser-relocation
block close, consistent with the aggressive-fast cadence. Recovery output and
predicate behavior must remain byte-identical.

Before commit: source-size check, secret scan, careful full diff read, and the
focused report filename in the commit message. No live provider key is needed.

### Byte-identical invariants

- The bytes emitted by `build_dns_query` for every host test fixture are
  identical before/after extraction.
- Live `poll_dns` accepts/rejects the same fixture corpus and emits the same
  serial lines/cache values; only its parser ownership changes.
- Existing `svc.demo.bufecho`, `certwindow`, `httphead`, and `certspki`
  descriptors, Wasm artifacts, output records, hashes, and profile needles do
  not change.
- Existing full/recovery harness needles do not move for the extraction. The
  new method/profile is additive only.
- `policy_allows_beyond_env` and every provider/trust/durable authorization
  remain false.

## 5. Implementation packets and exact write sets

### Packet A — `M11-9A-dnsparse-extract` (worker writes; orchestrator compiles)

Goal: extract the pure codec/parser, preserve live DNS behavior, and leave host
tests that prove the move.

Exact write set:

```text
Cargo.toml
Cargo.lock
raios-core/Cargo.toml
raios-core/src/lib.rs
raios-dns-parse/Cargo.toml                 (new)
raios-dns-parse/src/lib.rs                 (new)
seed-kernel/src/net.rs
```

Worker does not run cargo or build the kernel. Orchestrator runs the host tests,
fmt, kernel compile loop, and checks the net seed-kernel LOC delta. Do not begin
Packet B until query bytes and parser outcomes are equivalent.

### Packet B — `M11-9B-dnsparse-signed-guest` (worker writes; orchestrator builds/signs)

Goal: add the signed guest, bounded shared runner, independent core cross-check,
and exact grants-nothing record.

Exact worker write set:

```text
Cargo.toml
Cargo.lock
wasm-guests/svc-demo-dnsparse/Cargo.toml                    (new)
wasm-guests/svc-demo-dnsparse/src/lib.rs                    (new)
scripts/build-wasm-guest.ps1
seed-kernel/build.rs
seed-kernel/src/wasm_runtime.rs
seed-kernel/src/agent_protocol.rs
seed-kernel/src/agent_protocol_wasm.rs
seed-kernel/descriptors/svc.demo.dnsparse.wasm_artifact_identity.desc (new)
seed-kernel/descriptors/svc.demo.dnsparse.current_boot_load.desc       (new)
```

Exact orchestrator-owned generated/signing write set after reviewing the
sources:

```text
seed-kernel/artifacts/svc.demo.dnsparse.wasm                            (new)
seed-kernel/descriptors/svc.demo.dnsparse.wasm_artifact_identity.p256.pub.hex (new)
seed-kernel/descriptors/svc.demo.dnsparse.wasm_artifact_identity.p256.sig.der.hex (new)
seed-kernel/descriptors/svc.demo.dnsparse.current_boot_load.p256.pub.hex (new)
seed-kernel/descriptors/svc.demo.dnsparse.current_boot_load.p256.sig.der.hex (new)
```

The orchestrator uses the existing guest build/signing flow, then owns every
seed-kernel compile-error iteration. Any required source-attestation hash
regeneration stays orchestrator-owned and must be derived from the final source,
never guessed by a worker.

### Packet C — `M11-9C-dnsparse-profile-close` (worker writes; orchestrator runs)

Goal: add only the focused harness plumbing and close on observed output.

Exact write set:

```text
vm-harness/shadow-vm-smoke.ps1
vm-harness/shadow-vm-smoke-profile-m11-9-dnsparse.ps1       (new)
```

The worker derives predicates from the typed probe contract, not from invented
serial output. The orchestrator runs all profiles named in section 4, classifies
any failure before retry, runs the secret/size checks, then updates
`docs/PROJECT_STATUS.md`, `docs/ROADMAP.md`, and `docs/OWNER_DASHBOARD.md` in a
separate orchestrator-owned docs close. Those status files are intentionally not
part of a worker implementation packet.

## 6. Risks and stop conditions

1. **Number collision:** M11-8 already means certspki. Stop until the
   orchestrator records M11-8 and approves M11-9 naming.
2. **Red Gate:** if the newest full profile is red, only repair work is allowed;
   do not begin this slice.
3. **Behavior drift:** any query byte, accepted fixture, TTL rule, cache value,
   or serial behavior differs after extraction. Stop and repair the shared
   parser; do not edit needles to bless drift.
4. **No-dependency failure:** if the crate needs smoltcp, `raios-core`, `alloc`,
   or wasm32-incompatible crypto, stop and keep the small kernel adapter. The
   parser crate itself must stay dependency-free.
5. **Memory bound conflict:** any existing buffer guest needs more than the
   proposed 2 MiB. Measure and ask before changing the bound; never leave the
   limiter unattached or silently widen it.
6. **Authority creep:** any proposal to let the guest update the DNS cache,
   open sockets, select the DNS server, authorize a provider request/export,
   receive a secret, or set `policy_allows_beyond_env=true`. Stop; that is a new
   trust/import slice.
7. **Mismatch handling:** guest/core mismatch, malformed guest output, trap,
   fuel exhaustion, or missing service influences live DNS. Stop; the guest must
   remain evidence-only and the core remains authority.
8. **Signing weakness:** missing signature, field/hash mismatch, wrong trust
   tier, external intake, persistence, owner sealing, or any true authorization
   flag must fail the build.
9. **Shared-runner regression:** any bufecho/certwindow/httphead/certspki focused
   profile changes after the memory limiter. Repair before commit; do not exempt
   the old guests.
10. **Scope expansion:** TLS host imports, live DNS routing through Wasm,
    provider JSON relocation, WiFi service work, persistence, or new schemas
    beyond the one typed probe record are follow-ups, not part of this slice.
