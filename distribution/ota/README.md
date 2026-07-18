# OTA Tooling

Command-line helpers and libraries for generating deterministic signing keys, producing signed manifests, and verifying OTA/module payloads.

## Commands

- `cargo run -p ota-tools --bin ota-keygen -- --output keys/dev`
- `cargo run -p ota-tools --bin ota-sign -- --key keys/dev/online-key.json --cert keys/dev/online.cert.json --input <blob> --output <manifest.json>`
- `cargo run -p ota-tools --bin ota-verify -- --input <blob> --signature <manifest.json> --root-pub keys/dev/root.pub`
- `cargo run -p ota-tools --bin mod-sign -- --key ... --cert ... --input module.wasm --module-id hello-ui --module-version 0.1.0 --output module.manifest.json`
- `cargo run -p ota-tools --bin mod-verify -- --input module.wasm --signature module.manifest.json --root-pub keys/dev/root.pub`

All manifests embed Ed25519 signatures and BLAKE3 payload hashes.

## Tests

```
cargo test -p ota-tools
```

Unit tests exercise deterministically generated keypairs and a sign/verify round-trip.
