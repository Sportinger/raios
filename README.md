# SeedOS / RaiOS2

SeedOS/RaiOS2 is an ultra-small bootable OS experiment whose first useful screen
is a minimal AI agent host, not a normal desktop or Linux distribution.

The first MVP goal is:

```text
Boot in VM -> framebuffer + serial log -> network device visible -> minimal agent/status screen
```

The larger product idea is a small OS that can connect to known AI providers
without requiring a custom dedicated cloud server. The OS should eventually expose
small capability-gated tools to an AI agent, instead of trying to run a full host
CLI such as Codex inside the kernel.

## Start Here

For humans, start here. Codex instances should already receive `AGENTS.md` as
project memory, then read the rest in this order:

1. `AGENTS.md` - working memory for Codex sessions.
2. `README.md` - repo overview.
3. `docs/PROJECT_STATUS.md` - current verified state and exact next task.
4. `docs/ROADMAP.md` - overall plan and phase boundaries.
5. `docs/DEBUGGING.md` - how to build, run, inspect, and debug the VM.
6. `docs/architecture-decisions/0001-seedos-agent-protocol.md` - core AI agent
   architecture decision.

## Current State

The current bootable MVP artifact is:

```text
release/seedos-stage0.img
```

It has been visually verified in QEMU on Windows. It boots through Limine, reaches
the Rust kernel, negotiates a framebuffer, draws a small Stage-0 overlay, and
detects virtio-rng.

Expected first screen:

```text
SEEDOS STAGE-0
AGENT HOST: STUB
VM MVP: BOOT + FRAME + DEVICE POLL
```

## Windows Quick Commands

Build the kernel:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
```

Run the VM:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

Run workspace tests:

```powershell
cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server
```

Format check:

```powershell
cargo fmt --all -- --check
```

## Important Boundaries

- Keep Limine for the MVP. It is the boot handoff layer, not the OS runtime.
- Do not port the Codex CLI into Stage-0.
- Build a native SeedOS agent protocol with explicit capability-gated tools.
- Keep kernel changes small and boot-testable.
- Preserve `release/seedos-stage0.img` as the known bootable image until a new
  image has been tested visually and via serial logs.

## Local Convenience

There is a Desktop shortcut on this machine:

```text
C:\Users\admin\Desktop\SeedOS Codex Bypass.lnk
```

It launches Codex in this repo with approvals and sandbox disabled. Use it only
when that level of local access is intended.
