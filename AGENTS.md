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
7. `docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md`
8. `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`

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

## Vertical Slice And Multi-Agent Rule

Keep the full raiOS vision, but move it through observable vertical slices.
The default next step should be the smallest real capability on the final
architecture path, not another schema-only boundary.
This is an OS-wide AI-parallel build, not a traditional serial big-team plan:
independent agents may work at the same time when their ownership boundaries do
not conflict, but every merged result must be a real, tested slice.

- A non-authorizing schema or diagnostic slice is allowed only when it directly
  unblocks the next positive behavior, closes a concrete trust gap, or repairs
  verification. Do not chain schema-only loader boundaries while a runtime slice
  can be built instead.
- Do not copy the active next slice into this file. The current engineering
  cursor lives in `docs/ROADMAP.md` and the detailed exact task lives in
  `docs/PROJECT_STATUS.md`; use this file for durable rules only.
- Work is selected from the capability milestones M0-M7 in `docs/ROADMAP.md`
  (the legacy Phase 0-10 structure is retired as the planning backbone; see
  ADR 0005). Keep persistence, external unsigned artifact intake, durable
  writes, rollback application, and broad mutation denied until the milestone
  gates say the evidence chain is ready.
- A built-in hello artifact is acceptable only as labeled test infrastructure
  for the real path. It must not fake success, bypass the service registry, or
  imply that arbitrary external modules are supported.
- Multi-agent work should split by ownership boundary: loader/runtime,
  service registry, audit/capability gate, VM harness, and docs. Each agent
  should return one integrated slice with verification, not expand scaffolding
  for later.
- VM reports should prefer proving positive behavior plus the necessary
  fail-closed denials over growing denial-only predicate counts.

## Verification Budget Rule

Do not run the heaviest verification after every tiny evidence hop. Keep real
verification, but match the check to the risk of the slice:

- For docs-only changes, run a targeted diff/whitespace check.
- For local UI, formatting, or refactor-only changes, run `cargo fmt --all
  -- --check` plus the smallest relevant build or test.
- For trust, storage, rollback, recovery, authority, descriptor, or harness
  changes, run a focused or quick VM smoke that exercises the changed path.
- Batch 3-5 small same-boundary, non-authorizing evidence hops before running
  the next focused VM smoke when the prior focused/quick smoke stays green.
  Do not batch changes that cross storage, rollback, recovery, authority,
  provider-trust, descriptor-signing, harness, or boot-risk boundaries.
- Before committing, handing off a release image, or claiming a durable
  security/recovery milestone, run the relevant full/focused VM profile plus
  secret scan.

Quick often, full rarely, focused when the touched boundary is risky. Never
skip VM evidence for changes that affect storage, rollback, recovery,
capability authority, provider trust, descriptor signing, or boot behavior.

### Owner cadence decision (2026-07-07): aggressive-fast

The owner weighed speed/cost over per-sub-slice safety margin and chose the
aggressive-fast cadence. This is a standing decision that tunes the budget above:

- **Per sub-slice:** run ONLY that slice's own focused VM profile (e.g.
  `memory-durable`), then commit. That is sufficient evidence to close a
  sub-slice.
- **Adversarial (max-effort) review:** DROPPED from the routine loop by owner
  decision (2026-07-08) — do NOT run the multi-agent adversarial review workflow
  per slice. The relocation/grants-nothing slices consistently returned SHIP with
  zero defects, so the owner traded it away for speed/cost. Replace it with: the
  orchestrator's OWN careful read of the full diff before commit, the host DoD,
  the focused VM profile, and a secret scan. If a step is genuinely dangerous — a
  real authority flip, secret custody, or anything that could grant more than
  claimed — surface it to the owner rather than silently shipping; do not treat
  "review dropped" as license to skip judgment.
- **Worker model = CODEX, not Claude (owner 2026-07-08):** dispatch a Codex
  worker for BOTH implementation AND scoping/design/recon — Codex is trusted,
  fast, and far cheaper than a multi-agent Claude Workflow. Do NOT spin up
  5-Opus-agent Claude scoping workflows anymore; use a read-only Codex pass that
  writes the recon/design/packet instead. Claude (the orchestrator) stays for
  planning, dispatching, the pre-commit diff read, VM profiles/regressions,
  judgment, commits, and dashboards. Don't interrupt already-running agents when
  switching approaches — let them finish and apply the policy to the next
  dispatch.
- **Full profile (`full`) + `recovery` byte-identical:** only at a BLOCK /
  sub-milestone close (e.g. when M9A / M9B / M9C finishes), NOT on every
  sub-slice — plus the pre-existing rule to run them before handing off a release
  image or claiming a durable security/recovery milestone.

Rationale: the VM tests dominate wall-clock (each profile recompiles the kernel +
boots QEMU), and `full`/`recovery` have been byte-identical-green on every slice
because each slice is cleanly isolated behind its own scoped evaluator — so
full-per-sub-slice is low-yield. The structural fix for test time as the system
grows is M11 (services out of kernel → a change tests only its own service). The
Red Gate Rule still holds at every tier actually run.

### Owner verification preference (2026-07-10): batch ordinary checks

Prefer one well-targeted verification pass after a coherent set of independent,
non-authorizing foundation changes over repeating the same build or VM check after
every individual module. Keep the existing mandatory focused check at each real
storage, recovery, authority, provider-trust, descriptor-signing, or boot boundary;
never batch across those boundaries. Reuse still-current evidence for unchanged
surfaces, then run the combined regression/profile when the joined behavior is ready.

## Capability Definition Of Done

Adopted 2026-07-04 after
`docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md` and
ADR 0005. These rules override older slice habits:

- Every slice must state, in one sentence in its commit message and status
  entry, what a user or agent can now DO that it could not do before. "A new
  denial schema exists" does not qualify as a capability.
- No new `raios.*.v0` schema may be added as hand-rolled emit/hash code.
  While milestones M0-M2 (`docs/ROADMAP.md`) are open, no new schemas at
  all; after M2, new schemas are typed record-model entries only.
- At most one denial-gate or evidence-only slice per five capability slices.
- During the M2 ceremony collapse, every slice that ports a gate must delete
  more lines than it adds; the smoke harness proves byte-identical serial
  output.
- Progress reports to the owner lead with the capability sentence, never
  predicate counts. Update `docs/OWNER_DASHBOARD.md` every session.

## Red Gate Rule

While the full Shadow VM profile is red, the only permitted work is fixing
it: no new slices, no new gates, no new schemas. Every commit message must
name the passing report file for the verification tier that was run. The
final commit of a session requires a green full-profile report filename or
an explicit Red Gate note naming the repair work done.

## Commit Discipline

- No slice ends with uncommitted source changes.
- If more than ~2,000 lines are uncommitted, the next action is a commit,
  not a feature.
- Never label a catch-up commit as a single small feature; describe what it
  actually contains, split by ownership boundary where practical.

## Failure Classification Rule

Every failed VM run must be classified in `docs/PROJECT_STATUS.md` with the
failing predicate name and a one-line `host-transport` vs `guest-behavior`
verdict before any retry. A predicate that fails and then passes on retry
without a code change is logged as a suspected intermittent guest bug, not
closed as a host flake.

## End-Of-Session Checks

Run and paste output from all three before ending a session:

1. Source-size check: run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-source-size.ps1`. It warns at 3,000 lines or 120 KiB and fails at 5,000 lines or 200 KiB per owned `.rs` file. Temporary adoption exemptions are exact no-growth baselines and must be removed as P1-P3 shrink the files; do not add or widen an exemption instead of splitting or relocating a file.
2. Gate check: the newest full-profile report says `result: passed` and is
   newer than the last commit, or the session was Red Gate repair work.
3. `cargo fmt --all -- --check` (scoped to crates rustfmt can process until
   M2 fixes the oversized sources).

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

## Development Architecture Hygiene

During development, do not pretend that every normal code change already runs
through the final raiOS memory/recovery model. Build in the repository with real
code, real tests, VM reports, and docs; the finished product will later store
its own typed memory through the OS architecture.

Still shape every durable slice so it can become raiOS memory later:

- split a file or document as soon as a stable boundary is visible; do not wait
  for oversized protocol files or handoff docs to become expensive to untangle
- treat cleanup as part of normal building when it is low-risk and useful:
  extracting a clear runtime/diagnostic/selftest/emit/harness boundary early is
  preferred over letting a temporary monolith grow
- avoid speculative cleanup while the behavior, trust boundary, or protocol
  shape is still unclear; finish the real slice first, then cut along the
  stable boundary that emerged
- use size as an early warning: around 1k-2k LOC, look for ownership boundaries;
  around 3k-5k LOC, actively split if a stable boundary exists; above 10k LOC
  should be exceptional and documented; 20k+ LOC requires a deliberate split
  plan before adding more behavior
- separate runtime paths, diagnostics/selftests, harness logic, and handoff
  prose instead of mixing them into one growing surface
- keep evidence authoritative: derive reports from observed execution, not
  manually duplicated command inventories or stale summaries
- record project knowledge for future agents in stable repo docs with source
  pointers, not by prompt stuffing or fake in-guest persistence
- use fast real slices for normal iteration and run recovery/full VM smokes when
  touching those trust, recovery, or persistence surfaces

## Current Verified State

This file intentionally keeps only durable startup facts. Do not mirror the
active engineering cursor or per-slice implementation history here. Read
`docs/PROJECT_STATUS.md` for the authoritative detailed current state, latest
VM reports, known gaps, and exact next task; read `docs/ROADMAP.md` for the
compact phase cursor and parallel work lanes.

Stable facts for new agents:

- Repo path: `C:\Users\admin\Documents\raios2`
- Bootloader: Limine 10 UEFI binary in `release/esp/EFI/BOOT/BOOTX64.EFI`
- Limine config uses `limine.conf`, not `limine.cfg`
- Bootable image: `release/raios-stage0.img`
- QEMU visual boot is verified on Windows with GTK display
- The kernel draws the double-buffered framebuffer UI with `AI`, `CONSOLE`,
  and `SET` modes plus compact device/provider status
- Serial command input exists through the documented QEMU TCP serial path
- `ask <text>` uses the in-guest OpenAI direct transport; the TLS path is
  pin/SPKI based and still does not provide full WebPKI chain or trusted-time
  validation
- `svc.demo.hello` is the real current-boot built-in service test path. It
  exercises signed descriptor/artifact evidence, lifecycle/inventory,
  hot-swap/state-migration, rollback preview/apply denial, test-media
  write/readback evidence, and recovery-lifeline bindings as recorded in
  `docs/PROJECT_STATUS.md`.
- Persistence, external unsigned artifact intake, executable candidate-byte
  mapping, provider auto-load, broad mutation, durable audit writes, rollback
  store writes, real transaction append, rollback application, and installed
  rollback state remain denied unless `docs/PROJECT_STATUS.md` and
  `docs/ROADMAP.md` say otherwise.
- Latest verification evidence should be read from
  `release/vm-reports/shadow-*.json`, not copied into this file.

Routine slice progress updates should touch `docs/PROJECT_STATUS.md` and, when
the cursor changes, `docs/ROADMAP.md`. Update `README.md` or `AGENTS.md` only
when durable project reality, startup instructions, or standing rules change.

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

### Local Cargo cache on this machine

The inherited `CARGO_HOME` and `CARGO_TARGET_DIR` may point to the unavailable
`F:\scorefollower-build\cargo` path. Before any Cargo command or PowerShell build/
package script, use the repository-local ignored cache and target directory instead:

```powershell
$env:CARGO_HOME = (Resolve-Path '.cargo-home').Path
$env:CARGO_TARGET_DIR = Join-Path (Resolve-Path '.').Path 'target'
```

For concurrent agent lanes, keep the same local `CARGO_HOME` but give each lane its
own target subdirectory, for example
`$env:CARGO_TARGET_DIR = Join-Path (Resolve-Path '.').Path 'target\lanes\<lane>'`.

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

## Engineering Cursor

Do not maintain the exact next task here. Read:

- `docs/ROADMAP.md` for the compact active cursor and parallel work lanes.
- `docs/PROJECT_STATUS.md` for the detailed exact next task and latest
  verification evidence.
- `docs/DEBUGGING.md` for the cheapest relevant build, smoke, and diagnostic
  command.

Stable work lanes are runtime/recovery, provider trust, UI/input, VM harness,
and bare-metal bring-up. Pick the smallest lane-local slice that advances the
roadmap without duplicating the current cursor.

## Working Rules

- Do not revert unrelated user changes.
- Keep changes narrow and boot-testable.
- Prefer Windows PowerShell scripts for this local machine; Bash scripts are for
  WSL/Linux environments.
- Preserve `release/raios-stage0.img` as the currently bootable MVP artifact.
