HELLO-ATTEST-RECON is complete. The chain is fully regenerable: the repository intentionally retains no Hello signing private keys; `descriptor-resign` creates fresh ephemeral development keypairs and replaces each affected public-key/signature tuple.

## 1. Chain anatomy

### A. Ordered source-set snapshot

`HELLO_ARTIFACT_SOURCE_SET` currently contains 22 files, in declaration order: the root `hello_service.rs`, `current_boot_service.rs`, and 20 Hello modules ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:8)).

For every entry, `build.rs` appends exactly:

```text
decimal(path byte length) + "\n"
path bytes + "\n"
decimal(file byte length) + "\n"
raw file bytes + "\n"
```

Entries are concatenated in manifest order ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:1761)). SHA-256 of that complete byte stream becomes `artifact_content_source_sha256` ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:198)).

Current value:

```text
sha256:18a6ec6e0f77c212c92691f87f9359d3cd553203a23d38f3a6e3089c1bb6d5b7
```

It is pinned independently in:

- v1 identity, line 12: [svc.demo.hello.builtin_artifact_identity.desc](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.desc:12)
- v2 identity, line 13: [svc.demo.hello.builtin_artifact_identity.v2.desc](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc:13)

### B. Content binding

`build.rs` constructs the exact newline-delimited, no-final-newline content-binding payload shown at [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:200), inserting the source-set hash. SHA-256 of those exact bytes becomes `artifact_content_binding_sha256` ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:213)).

Current value:

```text
sha256:b32d27c98399af6edd803d51568e8c55fabe889a317c2d94cd64372e53c7d521
```

It appears twice in each identity descriptor:

- binding pin and reference backlink: [v1 descriptor](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.desc:13)
- binding pin and reference backlink: [v2 descriptor](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc:14)

### C. Artifact byte reference

The artifact bytes are read from `seed-kernel/artifacts/svc.demo.hello.builtin.artifact` ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:147)). Their SHA-256 is inserted into the exact artifact-reference text at [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:217); that text is then SHA-256 hashed ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:232)).

Current values:

```text
artifact bytes:
sha256:ecd0ddf0607cb8898d92597d60e4946548ae5d0a40b0186cc6f69ecf37287528

artifact reference:
sha256:6f839045f3c231c7bd0de59afcfd4babcfd91caf663068924064df0d90698db2
```

Both identities pin both values ([v1](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.desc:22), [v2](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc:23)).

Removing source-set files changes the source hash, binding hash, and reference hash. It does not change the artifact-bytes hash unless the `.artifact` file also changes.

### D. Identity v1/v2 and signatures

`build.rs` asserts that both identity descriptors contain all five computed pins:

- source-set hash
- content-binding hash
- artifact-reference hash
- artifact-bytes hash
- reference-to-content-binding hash

The v1 assertions are at [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:240); v2 additionally must have the exact v2 ID and matching pins ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:263)).

Each signature is P-256 ECDSA over the descriptor’s exact raw file bytes—not its SHA-256 text, an envelope, or a canonicalized reconstruction:

- current-image descriptor: [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:176)
- identity v1: [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:177)
- identity v2: [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:182)

The verifier parses uncompressed SEC1 public-key bytes and ASN.1 DER signature bytes, then calls `p256`’s `Verifier::verify(payload)` ([build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:1787)).

The verifying keys are the adjacent files:

- current image: [current_image.p256.pub.hex](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.current_image.p256.pub.hex:1)
- identity v1: [builtin_artifact_identity.p256.pub.hex](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.p256.pub.hex:1)
- identity v2: [builtin_artifact_identity.v2.p256.pub.hex](/C:/Users/admin/Documents/raios2/seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.pub.hex:1)

`build.rs` embeds those exact public keys, signatures, payloads, and derived envelope hashes into generated kernel constants. Runtime records consume the generated keys at [descriptor_sources.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:110), and runtime re-verifies the signatures at [descriptor_sources.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:884).

There is no independent in-kernel Hello public-key allowlist. The checked-in `.pub.hex` files are the pins.

## 2. Key custody

The old private keys do not exist. This is intentional.

`descriptor-resign`:

- reads exact raw descriptor bytes;
- generates a fresh random P-256 signing key using `OsRng`;
- signs those bytes;
- writes only the uncompressed SEC1 public key and DER signature ([main.rs](/C:/Users/admin/Documents/raios2/tools/descriptor-resign/src/main.rs:19), [main.rs](/C:/Users/admin/Documents/raios2/tools/descriptor-resign/src/main.rs:50)).

ADR 0013 explicitly says the private key exists only in process memory and is neither persisted nor printed ([ADR 0013](/C:/Users/admin/Documents/raios2/docs/architecture-decisions/0013-reproducible-local-descriptor-resigning.md:13)). Outputs are only `dev_key_not_owner_sealed` provenance and grant no owner, OTA, promotion, loader, or runtime authority ([ADR 0013](/C:/Users/admin/Documents/raios2/docs/architecture-decisions/0013-reproducible-local-descriptor-resigning.md:26)).

Repository searches found:

- no Hello `.priv`, `.key`, private `.pem`, or private scalar matching these public keys;
- `dev-promotion-signer` uses a separate scalar-1 promotion test/dev key and does not sign Hello descriptors;
- `core-policy-sign` manages the separate owner core-policy key;
- OTA/Wasm private scalars are separate dev authorities and are explicitly forbidden as substitutes.

The tracked signer superseded the lost ignored helper in commit `1a0b8ad`. Commit `488cea5` records that v1/v2 were regenerated through the former equivalent helper. The current workflow is documented at [ORCHESTRATOR_PLAYBOOK.md](/C:/Users/admin/Documents/raios2/docs/ORCHESTRATOR_PLAYBOOK.md:330).

## 3. Exact regeneration procedure

For a changed `HELLO_ARTIFACT_SOURCE_SET`:

1. Edit the ordered source-set list. Preserve the intended order and LF/raw byte handling; `.gitattributes` must continue marking every covered path `-text` ([DEBUGGING.md](/C:/Users/admin/Documents/raios2/docs/DEBUGGING.md:2005)).

2. Recompute the ordered framed snapshot exactly as [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:1761) specifies. SHA-256 it.

3. Reconstruct the exact content-binding payload from [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:200), substituting the new source hash. SHA-256 that payload.

4. Reconstruct the exact artifact-reference payload from [build.rs](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:217), retaining the artifact-byte hash and substituting the new content-binding hash. SHA-256 it.

5. Update both identity descriptors with:

```text
artifact_content_source_sha256=<new source-set hash>
artifact_content_binding_sha256=<new binding hash>
artifact_reference_sha256=<new reference hash>
artifact_reference_content_binding_sha256=<new binding hash>
```

Do not change `artifact_reference_bytes_sha256` unless the artifact bytes changed.

6. Build the tracked signer:

```powershell
$env:CARGO_HOME = (Resolve-Path '.cargo-home').Path
$env:CARGO_TARGET_DIR = Join-Path (Resolve-Path '.').Path 'target'
cargo build --locked -p descriptor-resign
```

7. Explicitly generate fresh v1 and v2 tuples:

```powershell
cargo run --locked -p descriptor-resign -- sign `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.desc `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.p256.pub.hex `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.p256.sig.der.hex

cargo run --locked -p descriptor-resign -- sign `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.pub.hex `
  seed-kernel/descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.sig.der.hex
```

8. Verify both tuples using identical paths and `verify` instead of `sign`. The command contract is defined at [main.rs](/C:/Users/admin/Documents/raios2/tools/descriptor-resign/src/main.rs:14).

9. Build the kernel. `build.rs` then independently checks all pins and signatures.

The current-image descriptor tuple does not need regeneration for a source-set-only change. If its raw descriptor changes, sign and verify it with the same tool using its `.desc`, `.pub.hex`, and `.sig.der.hex` paths.

No old private key is needed. Fresh public keys are correct because the public-key files are themselves the development pins and `build.rs` regenerates the in-kernel constants from them.

## 4. Blast radius

Searching all current four hash literals across `vm-harness/` and `docs/` found no hard-coded copies. Only the two identity descriptors carry those literal pins.

The quick harness derives hashes from the guest response and checks internal equality and successful verification:

- descriptor envelope algorithm, payload hash, key/signature hashes, verification result: [quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:1732)
- v1 identity ID, identity hash, and signature verification: [quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:1766)
- content-binding locator and hashes: [quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:1805)
- propagation through load request/service/loader: [quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:1901)
- health response equality and verified trust: [quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:2030)

Therefore fresh keys, signatures, and hashes require no harness golden edits provided schemas, IDs, locators, denial flags, and cross-record equality remain unchanged.

The one semantic golden relevant to shrinking is the locator assertion: the content binding must still report `seed-kernel/src/hello_service.rs` ([quick profile](/C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:1819)). Removing subordinate files from the attested set does not require changing that locator.

## 5. Recommended slice plan

1. **P3-7 source-set shrink:** remove only files whose behavior has genuinely moved out of the Hello artifact boundary. Recompute four descriptor fields, re-sign v1/v2, run signer verification, kernel build, `cargo fmt --all -- --check`, then the focused quick/Hello profile. This is one trust-boundary slice.

2. **P3-1 write-boundary separation:** Hello currently imports all three write-boundary modules directly at [hello_service.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service.rs:6). Those imports feed one central adapter, `rollback_writer_storage_foundation()`, which obtains and evaluates storage-layout, append-engine, and append-contract snapshots ([rollback_authority_gates.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:4354)). The rest of Hello consumes the resulting `RollbackWriterStorageFoundation`, notably [rollback_writer_gate.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_gate.rs:283) and [rollback_writer_bindings.rs](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_bindings.rs:140).

   Minimal rewiring: move that single foundation-construction adapter to the generic write-boundary side and expose one generic snapshot/result function. Hello then imports that one function/type and deletes its three evaluator-module imports. Do not rewrite downstream Hello hashes, bindings, or emitters in this slice.

3. Once that adapter and any genuinely generic evaluator files no longer belong to the Hello artifact implementation, remove them from `HELLO_ARTIFACT_SOURCE_SET`, regenerate v1/v2 again, and run the focused write-boundary/rollback VM profile. Because this crosses the write/authority boundary, do not combine it with unrelated P2 waves.

Bottom line: regeneration is possible today without recovering any old secret. The smallest safe path is descriptor-field recomputation plus two explicit fresh-key signatures; no kernel key constants or harness hash goldens need manual updates.