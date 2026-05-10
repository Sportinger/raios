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
the Rust kernel, negotiates a framebuffer, draws a live Stage-0 status UI, uses
virtio-rng to seed entropy, configures virtio-net through DHCP, and accepts
console commands from serial and the QEMU virtio keyboard. It also has a first
serial host-bridge path: `ask <text>` emits a protocol request to a host script,
and the response is rendered back into the Stage-0 console. The VM console also
has a `setup` menu for choosing the current provider and entering an API key into
RAM without echoing the key back to the serial log.

Expected first screen:

```text
SEEDOS STAGE-0
AGENT HOST: LIVE STATUS
FRAMEBUFFER  READY
ENTROPY      READY
VIRTIO-RNG   READY
VIRTIO-NET   CONFIGURED
INPUT        READY
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

Rebuild and repackage the boot image on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release
```

Build a local OpenAI-default image from `OPENAI_API_KEY` without touching the
tracked ESP staging directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\seedos-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv
```

Run with an interactive serial console on TCP port 4555:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

Run the development host bridge against that serial port:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\host-bridge.ps1 -Port 4555
```

Run the host bridge against OpenAI from the host environment:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\host-bridge.ps1 -Port 4555 -Provider openai
```

This requires `OPENAI_API_KEY` in the host PowerShell environment. The current
bridge does not read the VM-stored key back out of the guest.

Inside the VM, type `setup` to open the provider/API-key menu. Keys are stored in
guest RAM only for now and are cleared by reboot or the menu's clear command.

Run the headless host-bridge smoke test:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\host-bridge-smoke.ps1
```

Run the OpenAI host-bridge smoke test:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-bridge-smoke.ps1
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
