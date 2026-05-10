# SeedOS / RaiOS2 Codex Memory

This repository is the local RaiOS2 / SeedOS workspace.

## Start Every New Instance Here

Read these files before making changes:

1. `README.md`
2. `docs/PROJECT_STATUS.md`
3. `docs/ROADMAP.md`
4. `docs/DEBUGGING.md`
5. `docs/architecture-decisions/0001-seedos-agent-protocol.md`

Then run `git status --short` and preserve unrelated user changes.

## Project Intent

Build an ultra-small OS MVP that boots directly into a minimal agent host:

- framebuffer/monitor output
- network device bring-up
- AI client/agent interface
- no dedicated custom cloud server requirement for the first milestone
- connect to known providers later, starting with ChatGPT/Codex-style workflows

The core idea is not to port the full Codex CLI into stage-0. The OS should grow a
native, capability-gated agent protocol and UI. CLI tools such as Codex can be a
reference/workstation tool, not the hard dependency inside the kernel.

## Current Verified State

- Repo path: `C:\Users\admin\Documents\raios2`
- Bootloader: Limine 10 UEFI binary in `release/esp/EFI/BOOT/BOOTX64.EFI`
- Limine config uses `limine.conf`, not `limine.cfg`
- Bootable image: `release/seedos-stage0.img`
- QEMU visual boot has been verified on Windows with GTK display
- Kernel currently draws a live framebuffer status UI:
  - `SEEDOS STAGE-0`
  - `AGENT HOST: LIVE STATUS`
  - status rows for framebuffer, entropy, virtio-rng, virtio-net, input
- Serial log confirms:
  - Limine loaded base revision 3
  - framebuffer response revision 1
  - virtio-rng legacy device detected
- virtio-rng entropy delivery currently times out and is shown as degraded.
- Detailed current status is in `docs/PROJECT_STATUS.md`.

## Important Technical Notes

- Keep Limine for the MVP. Replacing it now would waste effort; it only handles
  UEFI-to-kernel handoff and boot protocol requests.
- Building Limine from source is possible later, but this Windows/WSL setup was
  missing build dependencies such as `autoreconf`, `nasm`, and `mtools`.
- The kernel must be linked higher-half at `0xffffffff80000000`; lower-half ELF
  program headers fail under Limine.
- Limine requests need explicit start/end markers:
  - `.limine_requests_start`
  - `.limine_requests`
  - `.limine_requests_end`
- The kernel enables SSE early before Rust/allocator-heavy code paths.
- The framebuffer renderer writes directly to the Limine framebuffer address.

## Useful Commands

Build the release kernel on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
```

Run the current stage-0 VM on Windows:

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

Debugging and failure modes are documented in `docs/DEBUGGING.md`.

## Next Engineering Steps

1. Add a capability-gated command/input path, initially serial or keyboard.
2. Fix or bypass the virtio-rng entropy timeout so net/input can progress.
3. Finish virtio-net/DHCP visibility in the UI.
4. Define the first native agent protocol messages outside the kernel boundary.
5. Add a host-side bridge that can talk to Codex/OpenAI APIs from the VM during
   development, before attempting any direct in-OS provider integration.

The current exact next task is maintained in `docs/PROJECT_STATUS.md`.

## Working Rules

- Do not revert unrelated user changes.
- Keep changes narrow and boot-testable.
- Prefer Windows PowerShell scripts for this local machine; Bash scripts are for
  WSL/Linux environments.
- Preserve `release/seedos-stage0.img` as the currently bootable MVP artifact.
