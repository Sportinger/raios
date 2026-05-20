# Content-Addressed Registry

Seed OS artifacts are published into a deterministic, content-addressed registry backed by simple filesystem layout and reproducible metadata records.

## Layout

```
registry/
  blobs/<BLAKE3 hash>
  manifests/<BLAKE3 hash>.json            # signed manifest emitted by ota/mod signers
  evidence/<SHA-256 hash>.json            # host evidence records copied by content hash
  index/<namespace>/<name>/<tag>.json     # index record pointing at blob + manifest
```

Index records capture the payload hash, byte length, signer key id, and the logical name/version extracted from the manifest metadata. Filenames are sanitised to ensure portability.

Evidence references are optional and host-side only. A registry entry may bind
`raios.vm_test_report.v0` and `raios.local_attestation.v0` JSON files by
SHA-256 and relative registry path. Publishing copies those evidence files into
`evidence/` and records their schema, kind, result, and hash in the index
record. The publisher reads the SHA-256 from the evidence tool's adjacent
`.sha256` sidecar and records `sha256_source: "sidecar"`, so evidence should
come from `vm-harness\shadow-vm-smoke.ps1` or
`vm-harness\create-local-attestation.ps1`. This is not a kernel load approval:
Stage-0 still denies module loading until guest loader policy, capability
grants, audit records, and rollback checks exist.

Local attestation records must explicitly report `limits.grants_load_now: false`
or publishing fails closed. VM reports and attestations are treated as evidence
inputs for later policy, not as authority by themselves.

`registry-tools grant-diagnostic` computes
`raios.computed_capability_grant.v0` for an exact
manifest/artifact/VM-report/local-attestation tuple. This is still a
non-authorizing host diagnostic: valid evidence produces a stable diagnostic
hash and `computed_candidate_present: true`, while `grants_capability`,
`grants_load_now`, `can_load_now`, and `load_attempted` remain false.

## CLI

The `registry-tools` binary wraps common operations:

- `cargo run -p registry-tools -- init --path registry/local`
- `cargo run -p registry-tools -- publish --registry registry/local --blob <file> --manifest <manifest.json> --root-pub keys/dev/root.pub`
- `cargo run -p registry-tools -- publish --registry registry/local --blob <file> --manifest <manifest.json> --root-pub keys/dev/root.pub --vm-report release/vm-reports/<report>.json --local-attestation release/attestations/<attestation>.json`
- `cargo run -p registry-tools -- grant-diagnostic --manifest <module-manifest.json> --artifact <file> --vm-report release/vm-reports/<report>.json --local-attestation release/attestations/<attestation>.json --approval "APPROVE RAM_ONLY <tuple-prefix>"`
- `cargo run -p registry-tools -- list --registry registry/local [--namespace modules] [--name hello-ui]`

Publishing verifies the manifest against the offline root key, copies the blob and manifest into the CAS layout, and generates the index record.

## Tests

```
cargo test -p registry-core
```

The unit test exercises full publish/list round-trips using deterministically generated key material.
