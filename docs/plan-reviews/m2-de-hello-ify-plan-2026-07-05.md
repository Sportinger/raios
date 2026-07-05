# M2 De-hello-ify Plan (2026-07-05)

Read-only scoping analysis (packet M2-18) for splitting
`hello_service.rs` (22,705 lines) and its signed source snapshot chain.
Cardinal risks: the attestation must keep covering moved code (ordered
source-set hashing in build.rs), and key=value hash inputs stay
byte-identical through every emitter port.

**Section Map**
Primary file: [hello_service.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service.rs:1), 22,705 lines.

| Lines | Contents | De-hello-ify read |
|---:|---|---|
| 1-24 | Imports | Mixed; imports generic rollback/storage modules. |
| 26-365 | Service/schema/gate constants | Mostly Hello hardcodes: `svc.demo.hello`, `ram_only_hello_*`, service slot ids. |
| 367-1225 | Record structs | `LoadDescriptor` is generic; many rollback/preflight structs are generic but Hello-named. |
| 1226-1302 | `LOAD_DESCRIPTOR` and descriptor/artifact hash helpers | Generic built-in service descriptor path. |
| 1303-1547 | Hello state hash, migration, hot-swap probation | Service-state specific, but the migration/probation shape is generic. |
| 1548-8524 | Rollback/apply/append/policy/target-region constructors and hashers | Generic rollback transaction machinery trapped in Hello names. |
| 8525-12127 | Huge `hello_rollback_transaction_writer_storage_authority_gate_hash` | Generic durable writer/storage authority gate. Should move out first. |
| 12129-12534 | Artifact load preflight, slot activation, preflight selftests | Generic load-plan and slot activation with hardcoded service ids. |
| 12536-12644 | Local `sha256_bytes`, line hash, sector image writers | `sha256_bytes` duplicates [raios-core/src/lib.rs](C:/Users/admin/Documents/raios2/raios-core/src/lib.rs:9). |
| 12646-13264 | Signature check, globals, method predicates, selftest emitters | Dispatch and selftest shell; should be table/descriptor driven. |
| 13265-13675 | Load/start/restart/hot-swap/stop/drop/health state machine | Hello behavior: RAM-only counter, v1/v2 swap, reset-state denial. |
| 13677-13834 | Rollback apply denial binding setup | Generic gate binding over Hello state/probation. |
| 13836-16707 | Huge writer/storage binding populator | Generic event binding projection. High-value split target. |
| 16709-16868 | Command target parsing and aliases | Generic resolver with Hello aliases (`hello`, `hello.v2`, reset state). |
| 16869-18099 | Huge `HelloServiceLifecycleBinding` initializer | Generic service lifecycle binding plus Hello state fields. |
| 18101-22705 | Manual JSON emitters | Port to `raios_core::record`; keep line-hash hashers byte-identical first. |

Other de-hello-ify surfaces: [agent_protocol.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol.rs:1137) dispatches service commands directly to `hello_service`; [agent_protocol_system.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_system.rs:620) appends Hello to inventory manually; [event_log.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:4427) hardcodes `record_hello_*`; [event_log_types.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:143) has a 1,934-line `HelloServiceLifecycleBinding`.

**Signing Chain**
`build.rs` currently hashes one file’s bytes, not a source tree: it reads `src/hello_service.rs` at [build.rs:33](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:33), hashes those bytes at [build.rs:84](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:84), and emits `source_locator=seed-kernel/src/hello_service.rs` in the content binding at [build.rs:86](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:86).

Build-time chain:
- Watches descriptor/pub/sig files plus `src/hello_service.rs` and artifact bytes at [build.rs:11](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:11).
- Reads current descriptor, Hello source, artifact bytes, public keys, signatures at [build.rs:31](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:31).
- Verifies P-256 signatures for current descriptor, identity v1, identity v2 at [build.rs:62](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:62).
- Recomputes content binding and artifact reference hashes at [build.rs:86](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:86) and [build.rs:103](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:103).
- Asserts both identity desc files match those hashes at [build.rs:126](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:126).
- Writes generated constants to `OUT_DIR/hello_host_bound_descriptor_source.rs` at [build.rs:274](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:274).

Runtime chain:
- Generated constants are included by [descriptor_sources.rs:5](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:5).
- Descriptor and artifact identity records are assembled at [descriptor_sources.rs:217](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:217).
- Runtime validates identity fields including `artifact_content_source_sha256` at [descriptor_sources.rs:375](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:375) and [descriptor_sources.rs:452](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:452).
- Runtime re-verifies descriptor and identity signatures at [descriptor_sources.rs:865](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:865).

Signature files live in `seed-kernel/descriptors/`: current descriptor pub/sig, artifact identity v1 pub/sig, artifact identity v2 pub/sig. The checked-in identity descs pin the current source hash at [v1 desc:11](C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.desc:11) and [v2 desc:12](C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc:12).

Original signing tooling found: only local ignored helper [target/descriptor-resign/src/main.rs](C:/Users/admin/Documents/raios2/target/descriptor-resign/src/main.rs:14). It generates a fresh random P-256 key with `OsRng` at line 16 and writes public key plus DER signature at lines 19-20. No persistent private key file was found; the repo-local dev chain stores public keys and signatures, not signing private keys. The helper only signs two payloads per run, so it covers v1+v2 identities after a Hello source edit, but not all three descriptor payloads in one invocation.

Re-sign after a normal `hello_service` edit:
1. Recompute the source hash, content-binding hash, and artifact-reference hash exactly as `build.rs` does.
2. Update both identity descs: `artifact_content_source_sha256`, `artifact_content_binding_sha256`, `artifact_reference_sha256`, `artifact_reference_content_binding_sha256`.
3. Re-sign v1 and v2 identity descs with `target\descriptor-resign\target\debug\descriptor-resign.exe ...v1 desc/pub/sig ...v2 desc/pub/sig`.
4. If `svc.demo.hello.current_image.desc` changes too, use/extend a signer to sign that descriptor as well.

**Split Feasibility**
Yes, but not with current semantics unchanged. Today the attestation hash is one file’s bytes. If `hello_service.rs` becomes a thin `mod` wrapper and `build.rs` still hashes only that wrapper, the attestation silently stops covering the moved code.

To split safely: change `build.rs` to hash an ordered source-set snapshot, with path framing or a manifest, and add `cargo:rerun-if-changed` for every included file. Then update generated content-binding text, both identity descs, and both identity signatures. Any moved code that came from `hello_service.rs` must remain in that source-set until there is a separate attestation story for shared service runtime code.

**Slice Plan**
1. Snapshot tooling + hash dedupe: add ordered source-set hashing in `build.rs` while still covering the current single file; replace local `sha256_bytes` with `raios_core::sha256_bytes`. Re-sign v1/v2 if source bytes change. Verify: build + quick profile.

2. Mechanical split below 5k/file: split constants/types/hashers/state/emitters into `hello_service/` modules, source-set covers every moved file. Re-sign v1/v2. Verify: `cargo fmt --all -- --check`, build, quick, then `hello-rollback-dry-run`.

3. Move rollback writer/storage gate code out of Hello names: keep output/hash bytes identical, but isolate generic rollback constructors, the 8.5k-12.1k hash block, and the 13.8k-16.7k binding block. Re-sign included source-set. Verify: quick + `hello-rollback-dry-run`.

4. Port Hello emitters to `raios_core::record`: start with leaf emitters, then response emitters. Do not convert key=value hash inputs to JSON hashes in this slice. Re-sign after each batch that changes covered files. Verify: quick for each batch, `hello-rollback-dry-run` after the response batch.

5. Introduce minimal `ServiceDescriptor`: parameterize ids, aliases, capabilities, slot ids, event-log resource/capability fields, and inventory append logic. Keep emitted schema names stable for M2 unless explicitly changing contracts. Re-sign v1/v2. Verify: quick + `hello-rollback-dry-run`, then full profile as M2 closure evidence.

**Risks**
- Biggest silent weakening: hashing only the wrapper or omitting moved shared modules from the source-set.
- Fresh random signing keys prove integrity of checked-in payloads but not key continuity; acceptable for dev chain, weak for real trust.
- No `.gitattributes` exists; signed snapshots are byte-sensitive, so CRLF conversion can break builds again.
- Port emitters first, hashers later. Changing `hash_line_*` inputs will change evidence hashes even if JSON output stays identical.
- Full profile is needed after descriptor/source-set semantics and after event-log/dispatch parameterization; quick-only is too weak for the final M2 claim.