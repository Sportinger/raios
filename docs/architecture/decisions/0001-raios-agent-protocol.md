# ADR 0001: raiOS Agent Protocol Instead Of Porting Codex CLI

## Status
Accepted for the first VM MVP.

## Context
The product idea is a tiny bootable OS that can connect to AI providers and later let AI-generated, signed modules extend the system. A full CLI tool such as Codex assumes a normal host OS: terminal, process model, filesystem, home directory, auth storage, shell tools, Git, sandboxing, and usually an existing package/runtime environment.

Stage-0 raiOS intentionally has far less: boot, framebuffer, serial logs, input, one network path, trust anchors, and a small event loop.

## Decision
Do not port or run the Codex CLI inside Stage-0.

Build a small raiOS-native agent protocol instead. The device exposes explicit, capability-gated tools such as:

- `get_device_inventory`
- `read_boot_log`
- `draw_text`
- `probe_device`
- `download_signed_module`
- `run_module_test`
- `apply_config`

The AI provider is reached through HTTPS/JSON first. Early MVPs can use direct OpenAI API calls with a user-supplied API key. Later versions may add provider adapters, pairing flows, a local module runtime, and a control plane, but the Stage-0 boundary remains small.

## Consequences
This keeps the seed OS small and avoids rebuilding Linux just to host an existing CLI. It also makes every AI action visible as a narrow tool call that can be logged, denied, replayed in a VM harness, and eventually signed or attached to module capabilities.

The first practical milestone is therefore:

```text
Boot in VM -> framebuffer + serial log -> network device visible -> minimal agent/status screen
```

Then:

```text
DHCP/TLS/HTTPS -> provider config -> one prompt/response -> render answer
```

