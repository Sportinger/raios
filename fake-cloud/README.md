# Fake Cloud

A deterministic WebSocket control plane stub that brokers device traffic during tests. It validates signed module/OTA payloads against the offline root key and can push the verified artifacts into the local registry.

## Server

```
cargo run -p fake-cloud-server -- --bind 127.0.0.1:9001 \
    --root-pub keys/dev/root.pub \
    --registry registry/local
```

Supported message types (JSON envelope `{v,t,id,ts,body}`):

- `hello` → responds with `hello_ack`.
- `ota_begin` → body must include `manifest` (signed blob), `blob_b64`, and optional `namespace`/`name`/`version`. The server verifies the manifest, persists into the registry when configured, and replies with `ota_ack`.

## Tests

```
cargo test -p fake-cloud-server
```

The integration test streams a dummy signed module to the server, waits for verification, and asserts that the registry receives the published entry.
