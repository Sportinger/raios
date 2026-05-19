# raiOS Codex Memory

This repository is the local raiOS workspace.

## Start Every New Instance Here

Read these files before making changes:

1. `README.md`
2. `docs/PROJECT_STATUS.md`
3. `docs/ROADMAP.md`
4. `docs/DEBUGGING.md`
5. `docs/architecture-decisions/0001-raios-agent-protocol.md`
6. `docs/architecture-decisions/0004-system-memory-and-agent-context.md`

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

## Full-Vision Engineering Rule

Do not deliberately build throwaway MVPs, mocks, fake services, fake security, or
silent fallback paths. The project can move fast through agents, so default to
keeping the full raiOS vision in scope instead of traditional staged-down
prototypes.

When a narrow step is needed, make it a real vertical slice on the final
architecture path:

- real boot/test behavior, not mocked success
- real protocol/schema boundaries, not ad-hoc placeholders
- fail-closed or explicit `capability_denied` when evidence is missing
- no fake provider, driver, sandbox, module loader, trust, or persistence layer
  that pretends to be complete
- temporary harnesses are allowed only when they test the real path and are
  clearly labeled as test infrastructure

If the full feature cannot be completed in one pass, implement the durable
foundation first and expose unfinished parts as explicit denials, TODO status, or
known gaps. Do not hide missing functionality behind a fallback that could later
be mistaken for the intended system.

Agents should approach new problems and features from the final system shape
first. Start by identifying the full target architecture, invariants, protocols,
trust boundaries, and evidence needed for the real raiOS design. Then implement
the smallest durable slice that moves that architecture forward. Avoid spending
time optimizing intermediate product shapes, demo-only flows, compatibility
shims, or "good enough for now" branches unless they are explicitly part of the
final architecture or test the real path.

## System Memory Architecture Rule

Future agents must build toward the ADR 0004 model: raiOS itself is the memory.
Do not treat memory as a large chat transcript, prompt stuffing, or a generic
RAG database. Important system knowledge should become typed, classified,
evidence-bound records or source facts that can later feed
`raios.agent_context.v0`.

When changing status, logs, provider context, service inventory, problems,
capabilities, test reports, recovery behavior, or persistence, preserve these
rules:

- expose stable IDs and typed facts before prose-only strings
- attach provenance or evidence for facts that may guide agent action
- classify fields as `public`, `local_only`, or `secret` before provider export
- make summaries and semantic/RAG results locators only, never authority
- enforce token budgets through a context broker instead of sending whole memory
- keep memory writes denied or explicitly scoped until audit, policy, and
  persistence exist
- label early memory as `current_boot` when it is RAM-only or non-persistent

The near-term path is still raiOS first: harden provider trust and redaction,
stabilize `system.snapshot.v0`, `service.inventory`, `problem.list`, and
capability policy, then add read-only `memory.context` over those real facts.
Do not build fake persistent memory ahead of the real persistence/rollback
architecture.

## Current Verified State

- Repo path: `C:\Users\admin\Documents\raios2`
- Bootloader: Limine 10 UEFI binary in `release/esp/EFI/BOOT/BOOTX64.EFI`
- Limine config uses `limine.conf`, not `limine.cfg`
- Bootable image: `release/raios-stage0.img`
- QEMU visual boot has been verified on Windows with GTK display
- Kernel currently draws a double-buffered framebuffer UI:
  - chat-first `AI` mode
  - `CONSOLE` mode for debug output
  - `SET` mode for provider/API-key setup
  - compact status strip for network, input, USB-xHCI, and entropy
- Serial command input exists when QEMU is run with `-SerialMode tcp`:
  - `help`
  - `status`
  - `devices`
  - `log`
  - `provider`
  - `openai`
  - `setup`
  - `ask <text>`
- Serial log confirms:
  - Limine loaded base revision 3
  - framebuffer response revision 1
  - USB-HID keyboard and mouse enumerate through xHCI in the bare-metal VM profile
  - Intel e1000 DHCP configures in QEMU without Virtio devices
- `ask <text>` uses the in-guest OpenAI direct transport. The old host-side
  serial relay is no longer part of the runtime path; DNS, TCP 443, TLS 1.3,
  HTTPS, and first `output_text` parsing work in the bare-metal VM profile.
- The current TLS path is MVP-only: certificate verification is bypassed and
  must be hardened with verification or provider pinning before serious use.
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
- The framebuffer renderer draws into a heap backbuffer and presents to the
  Limine framebuffer, avoiding visible clear/redraw flicker during mouse moves.
- The visible QEMU GTK profile uses `grab-on-hover=on,show-cursor=off`; raiOS
  draws its own cursor and the host pointer should not escape the VM as easily.

## Secret Handling Rule

- Never commit OpenAI/provider keys or key-bearing boot artifacts.
- Provider keys may enter a VM image or boot USB only from the local process
  environment, through the documented `-EmbedOpenAiApiKeyFromEnv` path.
- Key embedding must use a temporary ESP staging tree and must not write into
  tracked `release\esp` or the default `release\raios-stage0.img`.
- Local provider images such as `release\raios-stage0-local-openai.img` are
  ignored artifacts and should be deleted after testing when not needed.
- Before committing or pushing, run `scripts\scan-secrets.ps1`; when checking
  GitHub/remote safety, fetch remote refs and run
  `scripts\scan-secrets.ps1 -GitHistory`.
- If a real provider key was ever pushed or shared, rotate it. Removing it from
  the current tree is not enough.

## Useful Commands

Build the release kernel on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
```

Run the current stage-0 VM on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

Run the bare-metal-style VM profile on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Run workspace tests:

```powershell
cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server
```

Format check:

```powershell
cargo fmt --all -- --check
```

Run the direct OpenAI VM smoke after packaging a local key image:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1
```

Debugging and failure modes are documented in `docs/DEBUGGING.md`.

## Next Engineering Steps

1. Harden the direct OpenAI TLS path with certificate verification or provider
   pinning.
2. Improve response wrapping, scrolling, and clickable settings controls in the
   framebuffer UI.
3. Define the first native agent protocol messages outside the kernel boundary.
4. Continue bare-metal input/network bring-up while preserving the VM test path.

The current exact next task is maintained in `docs/PROJECT_STATUS.md`.

## Working Rules

- Do not revert unrelated user changes.
- Keep changes narrow and boot-testable.
- Prefer Windows PowerShell scripts for this local machine; Bash scripts are for
  WSL/Linux environments.
- Preserve `release/raios-stage0.img` as the currently bootable MVP artifact.
