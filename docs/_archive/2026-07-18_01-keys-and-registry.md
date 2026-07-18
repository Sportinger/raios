# Section Report 1 – Keys & Registry

## Scope
Build runbook section 1 deliverables: deterministic key toolchain, signing/verification CLIs, content-addressed registry, and an initial fake cloud service able to accept and validate signed artifacts.

## Artifacts
- `ota/cli` Rust crate with binaries: `ota-keygen`, `ota-sign`, `ota-verify`, `mod-sign`, `mod-verify`.
- `registry/core` library + `registry/cli` binary implementing filesystem-backed CAS registry with publish/list flows.
- `fake-cloud/server` async WebSocket stub that verifies signed blobs and optionally publishes them into the registry.
- Generated dev keys in `keys/dev/` (deterministic seed derivation).

## Exit criteria
- Dummy signed blob verified through WebSocket control plane and persisted in registry (see `cargo test -p fake-cloud-server`).
- Registry initialised with CAS layout and index records (`cargo test -p registry-core`).
- Signing toolchain exercised via unit tests (`cargo test -p ota-tools`).

## Validation
- `cargo test -p ota-tools`
- `cargo test -p registry-core`
- `cargo test -p fake-cloud-server`
