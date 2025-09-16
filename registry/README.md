# Content-Addressed Registry

Seed OS artifacts are published into a deterministic, content-addressed registry backed by simple filesystem layout and reproducible metadata records.

## Layout

```
registry/
  blobs/<BLAKE3 hash>
  manifests/<BLAKE3 hash>.json            # signed manifest emitted by ota/mod signers
  index/<namespace>/<name>/<tag>.json     # index record pointing at blob + manifest
```

Index records capture the payload hash, byte length, signer key id, and the logical name/version extracted from the manifest metadata. Filenames are sanitised to ensure portability.

## CLI

The `registry-tools` binary wraps common operations:

- `cargo run -p registry-tools -- init --path registry/local`
- `cargo run -p registry-tools -- publish --registry registry/local --blob <file> --manifest <manifest.json> --root-pub keys/dev/root.pub`
- `cargo run -p registry-tools -- list --registry registry/local [--namespace modules] [--name hello-ui]`

Publishing verifies the manifest against the offline root key, copies the blob and manifest into the CAS layout, and generates the index record.

## Tests

```
cargo test -p registry-core
```

The unit test exercises full publish/list round-trips using deterministically generated key material.
