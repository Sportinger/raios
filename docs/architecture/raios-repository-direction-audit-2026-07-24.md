# raiOS Repository Direction Audit

Date: 2026-07-24<br>
Purpose: decision support for a subsequent AI and the owner<br>
Status: analysis, not an ADR and not a new product decision

Snapshot boundary: this report preserves observations from the repository
state inspected on 2026-07-24. It is non-normative, supplies no current
implementation or hardware evidence, accepts no change, establishes no ADR
status, and closes no checkbox. Uses of current below mean current within
that inspected snapshot unless the text explicitly says otherwise.

## 0. How to read this report

This report separates five things that are often mixed together in the
repository:

1. **normative target** — what raiOS is supposed to become;
2. **accepted implementation** — what is present at a committed source state;
3. **test evidence** — what one particular host/QEMU/hardware run demonstrated;
4. **work in progress** — dirty or branch-only code that has not been accepted;
5. **inference** — a conclusion from several sources, not an owner decision.

The labels `FACT`, `INFERENCE`, `OPEN QUESTION`, and `RECOMMENDATION` below are
intentional. A following AI should preserve them instead of turning every
observation into a claim.

Snapshot warning:

- the inspected folder is detached at `11bcdeb`;
- the locally known `origin/main` is `f1cb333`;
- the merge base is `09751a7`;
- the inspected folder has 2 commits not in `origin/main`, while
  `origin/main` has 92 commits not in this folder;
- no fetch was performed, so all remote statements refer to locally stored refs;
- the dirty files listed in section 11 were foreign work and were not changed.

The most important practical consequence is:

> This folder is not the canonical current product worktree. It is an old,
> detached audit root containing several preserved WIP strands.

The newer `origin/main` HANDOFF explicitly calls this folder the “detached old
root” and names a separate local path, represented here as
`<canonical-main-worktree>`, as the canonical main worktree. Any AI that starts
implementation here without first resolving the canonical ref risks building
on the wrong system state.

## 1. Executive conclusion

### 1.1 What has actually happened

raiOS did not merely accumulate random experiments. A coherent technical core
emerged:

- a custom `no_std` Rust kernel boots through Limine;
- it owns memory, framebuffer, input, storage and network hardware;
- Wasmi executes bounded Wasm guests;
- explicit host imports and fuel/memory limits form real isolation seams;
- local records distinguish observations, evidence, decisions and authority;
- several install, revoke, reboot, recovery and rollback paths exist;
- an unmodified threaded rustc-as-Wasm artifact has compiled a real
  `hello.rs` inside raiOS under QEMU;
- the repository has unusually extensive host/QEMU negative-test machinery.

That core is valuable and should not be discarded merely because the repository
is disordered.

### 1.2 Why the repository nevertheless feels “built crosswise”

At least three products are being pursued at once:

1. **Capability research OS**<br>
   A custom kernel, Wasm isolation, typed grants, recovery, audit and hardware
   authority.

2. **Personal AI appliance**<br>
   One owner, one Surface, direct provider chat, Genesis UI, local memory,
   physical approval and a system that can explain why it allowed or denied an
   action.

3. **On-device software factory**<br>
   An agent submits source, raiOS compiles and tests it on the device, the owner
   approves the exact artifact, and the result is installed and rolled back
   without a permanent external workshop.

All three can fit one long-term vision, but they do not imply the same next
milestone. Hardware Wi-Fi, evidence vocabulary, a general loader, a public
website and a working compiler can each consume months while leaving the other
two product stories incomplete.

### 1.3 Best reconstruction of the current north star

**INFERENCE, not a settled owner decision:** the strongest common center is:

> A personal, custom-kernel OS whose distinguishing product loop is an
> evidence-bound on-device factory: agent-authored source becomes executable
> authority only through bounded Wasm execution, tests and negative tests,
> exact artifact identity, owner approval, revocable grants and rollback.

This combines the binding factory plan, the README’s core thesis, ADRs
0001/0004/0005/0015 and the strongest implemented spike.

The other strands then become subordinate:

- hardware is an enabling substrate;
- the native agent protocol is the control interface, not the product itself;
- the website is a presentation surface;
- distribution is later-phase trust for non-owner machines;
- “self-explaining system” is a useful formulation of the authority plane, not
  a separate product;
- broad app examples such as games or video editing are acceptance targets,
  not work that should widen the current kernel.

### 1.4 The two decisions that must be settled first

1. **Where do drivers ultimately run?**<br>
   Current Scope sections 1–3 and ADR 0005 say native/in-kernel. Scope section 5
   and its breakdown say userspace domains and “no driver code in the kernel,
   ever.” Both cannot be the current normative target.

2. **What exact user journey is the next product proof?**<br>
   The repository needs one falsifiable path, for example:

   ```text
   owner request
   -> agent-produced multi-file source
   -> bounded compile
   -> machine-readable diagnostics
   -> corrected second revision
   -> deterministic rebuild
   -> negative tests
   -> exact owner approval
   -> run
   -> reboot
   -> rollback
   ```

   Until that path is chosen, subsystem checkboxes can continue to grow without
   proving a coherent product.

## 2. Authority order for a following AI

When sources disagree, use this order and record the conflict:

1. explicit new owner decision;
2. `docs/SCOPE.md`, but only after identifying internal contradictions;
3. active/superseding ADR;
4. current `docs/status/HANDOFF.md` on the canonical ref;
5. relevant `docs/scope/*.md`;
6. active `docs/plans/*.md`;
7. accepted code plus evidence bound to that exact code;
8. README and UI story;
9. `STATUS.md` / `OWNER_DASHBOARD.md` as potentially stale derived views;
10. archive, attic, branches, reports and untracked files as historical clues.

Important exceptions:

- A checkbox is not a proof by itself.
- A commit message is not a proof by itself.
- A report file is not a proof for a different source/image/hardware identity.
- An ADR with a stale footer or a superseding ADR must be interpreted
  chronologically.
- The current `docs/SCOPE.md` is internally contradictory, so “Scope wins”
  cannot resolve the driver question without owner input.

## 3. Repository state and topology

### 3.1 Git reality

| Item | Observed state |
|---|---:|
| Current checkout | detached `11bcdeb` |
| Locally known `origin/main` | `f1cb333` |
| Divergence `HEAD...origin/main` | 2 / 92 commits |
| Reachable commits over all refs | 1,086 |
| Commits reachable from current HEAD | 966 |
| Commits reachable from `origin/main` | 1,056 |
| Local branches | 28 |
| Remote refs | 16 |
| Registered worktrees | 30 |
| Stashes | 1 |

Many temporary worktrees are marked `prunable`, but they still explain why the
repository contains lane refs, patch-equivalent branch tips and multiple
“current” stories. No cleanup was performed.

ADR 0019 defines branchless main and one Git writer as a target practice. The
actual history contains merge commits, lane branches, many worktrees and merge
objects even after ADR 0019. Therefore:

- treat ADR 0019 as a governance norm;
- do not repeat its claim that the whole history already followed that norm;
- do not infer missing work solely from an unmerged lane ref, because several
  lane tips are patch-equivalent to commits already on main.

### 3.2 Canonical-main discontinuity

`HEAD..origin/main` changes 54 files by roughly 17,666 additions and 1,116
deletions. The newer main concentrates on:

- Surface fact capture;
- PCI BAR safety;
- Marvell DMA and firmware diagnostics;
- machine-bound hardware evidence;
- ADRs 0033–0045;
- a blocked H26 Wi-Fi replacement race.

The current folder’s two unique commits instead record A69/A70 audits and a
bare-metal isolation handoff. They were not carried into the newer main line.

This is not a small stale checkout; it is a diverged control-state problem.

### 3.3 Website/product split incident

The strongest single example of repository-direction discontinuity is:

- `fc05ed5`: reduced main to the website/UI builder, deleting roughly 1,327
  files and 506k lines;
- `4557600`: immediately restored the product tree, adding roughly 1,335 files
  and 510k lines;
- later commits documented a separate website/product branch split.

Do not interpret `fc05ed5` as a real historical decision to abandon the OS.
It was reverted by restoration almost immediately. It does demonstrate that
branch and deployment operations were able to make main briefly tell a
completely different product story.

## 4. Direction history

### Phase A — SeedOS substrate (September 2025)

The first commits already contained:

- a Rust seed kernel;
- Limine/UEFI image material;
- VirtIO work;
- OTA/registry tooling;
- a fake cloud;
- a VM harness.

This means distribution and host test infrastructure are not late additions.
They are ancestral directions that survived later pivots in reduced form.

### Phase B — bootable personal AI Stage-0 (May 2026)

Representative changes:

- verified Stage-0 snapshot (`af4034a`);
- framebuffer status UI, serial console, DHCP and input;
- direct OpenAI host bridge and then direct HTTPS in Stage-0;
- xHCI/HID work and a bare-metal USB path;
- early module, evidence and recovery vocabulary;
- rename from SeedOS through “raisOS” to raiOS.

Product picture: a small OS that boots, talks to a provider and exposes local
machine state.

Durable decision: ADR 0001 rejected porting the Codex CLI into Stage-0 and chose
a small native agent protocol.

### Phase C — permanent core and self-describing live world (May 2026)

ADRs 0002–0004 added:

- manifest-described, live-built modules;
- a permanent core plus replaceable services;
- system state as typed local memory;
- task-scoped agent context;
- facts/evidence/decisions with different authority levels.

This is the origin of both the “Lisp machine for an AI” story and the later
untracked “self-explaining system” analysis.

The implementation at that time was much less modular than the model.
Vocabulary and diagnostic surfaces grew before a general runtime existed.

### Phase D — mechanism before vocabulary; bare metal plus Wasm (July 4–6)

A deep review diagnosed “ceremony before mechanism”: extensive schemas and
authority prose without a closed physical isolation or product loop.

ADR 0005 then chose:

- bare metal, not Linux;
- a custom Rust kernel;
- Wasm as the first practical service isolation boundary;
- physical mechanisms and negative tests before more vocabulary;
- distribution temporarily parked.

Wasmi was vendored, the first signed guest ran, then a second service proved the
architecture was not hello-only.

### Phase E — grants, trust, storage, recovery and distribution slices (July 7–10)

Major strands:

- persistent memory records and recovery;
- per-service Wasm import grants;
- provider trust and time/certificate parsing;
- signed local distribution/provenance;
- IOMMU detection and Marvell work;
- Genesis and personal-shell boundaries;
- structured storage and secret vault.

ADR 0008 aimed to move Internet parsing into Wasm while keeping trust and
secrets under kernel authority. ADR 0009 re-enabled a narrow local signed
registry but not general network OTA.

### Phase F — apps, workspace and mass refactor (July 10–14)

The system gained several partially overlapping demonstrations:

- current-boot personal shell;
- calculator/editor-style bounded UI programs;
- project source workspace, dependencies, build and install;
- signed candidate delivery and rollback;
- parser guests for certificate window, SPKI, HTTP and DNS;
- network acquisition guest/import work;
- a large protocol/evidence vocabulary refactor.

Commit `3b70b9b` removed about 45.8k lines of legacy recovery protocol code.
This is important counterevidence to the idea that the project only grows; it
has also performed substantial consolidation.

### Phase G — closed self-build loop becomes the product (July 15–17)

The archived Vision Plan reframed the goal:

> Build the perfect closed self-build loop; do not build every future desktop
> or media application in the external workshop.

Provider answers became inert source revisions, bounded feedback could be sent
back, and project builds/installs were connected. UI programs and project Wasm
remained separate routes.

### Phase H — compiler route thrash, then real rustc (July 17–19)

This is the clearest evidence-based park/reactivation sequence:

1. Full rustc-as-Wasm was initially judged NO-GO because of threads, memory and
   interpreter limits.
2. `raios-wasm-ir` and a small `raios-lang`/assembler route became the staged
   alternative.
3. The owner re-centered the goal on a real on-device factory “without
   workshop”; `raios-lang` was parked as a spare tool.
4. A Wasmtime workshop probe showed an existing 91 MB threaded rustc artifact
   was viable.
5. Instead of forking the compiler, raiOS implemented shared memory, atomics,
   wait/notify, deterministic green threads, WASI preview1 contracts, a large
   guest memory class and BuildFS.
6. The 95,427,808-byte compiler was loaded and eventually compiled a real
   `hello.rs` inside QEMU.

This was not random oscillation: measurements changed the feasible option.
But it also shows how quickly a strategically central route can reverse within
days, leaving active plans and Scope wording behind.

### Phase I — docs/governance reframe (July 18–20)

Changes included:

- current `SCOPE.md` plus breakdown structure;
- archive/attic cleanup;
- custom kernel retained over seL4 (ADR 0015);
- single-writer governance (ADR 0019);
- build storage, suspension, grant and rollback ADRs;
- AGENTS.md as the only Codex control plane (ADR 0025);
- RAM-only boot recovery instead of recovery re-persisting installs (ADR 0029);
- serial RECLOG transport;
- crash-supervision and rollback-image work that later parked.

### Phase J — hardware fast track and H26 stop (July 20–22)

After the website/product split was repaired, main moved sharply toward the
Surface:

- Marvell DMA safety models;
- physical fact capture and RECLOG wire formats;
- PCI BAR restore/sizing rules;
- owner-custodied hardware evidence;
- repeated narrow Wi-Fi probes;
- governance for one-shot hardware attempts.

The newest locally known main parks H26 because a Ready-replacement race could
allow a loser to quiesce or erase the winner. Green local tests were rejected
by independent review. This is a good example of the project’s review culture
working, but it also means the current hardware main line is blocked.

### Phase K — unaccepted narrative experiments (July 24)

The current worktree contains untracked:

- `docs/architecture/self-explaining-system-report-2026-07-24.md`;
- `raios-system-comparison.html`.

They explore “evidence-bound OS control plane” positioning and system
comparisons. They have no commit provenance and are not ADRs. Treat them as a
possible new narrative, not an accepted direction.

## 5. Direction register

| Direction | Current classification | Preserve / question |
|---|---|---|
| Native agent protocol instead of Codex CLI in OS | Accepted, active | Preserve. It keeps provider/tool authority smaller than a full host CLI. |
| One owner, one bonded machine | Accepted product constraint | Preserve unless owner changes the product. |
| Custom Rust kernel | Owner-approved in ADR 0015 | Preserve as current path; do not silently restart seL4. |
| seL4 as substrate | Rejected as current parallel path | Only revisit through owner/ADR; keep floor contract kernel-agnostic. |
| Wasm linear-memory and import isolation | Implemented and central | Preserve; strengthen exact runtime/hardware claims. |
| Typed evidence/authority/memory | Implemented in parts and central | Preserve, but reduce vocabulary that does not drive a live gate. |
| Direct OpenAI in kernel | Implemented bootstrap path | Transitional. Conflicts with “kernel does not parse the Internet” and provider neutrality. |
| Provider-agnostic adapters | Target only | Decide later; current implementation is OpenAI-centric. |
| Replaceable live service graph | Long-term accepted idea | Transition from current monolith is undefined. |
| Drivers in kernel | Newer Scope §§1–3 / ADR 0005 | Probably current bring-up model, but conflicts with Scope §5. |
| Driver domains / “no driver code in kernel ever” | Simultaneously normative in Scope §5 | Owner decision required; not currently implementable as claimed without isolation machinery. |
| IOMMU/VT-d confinement | Probe and branch-only work | Translation is not enabled; never report DMA isolation as present. |
| Bounded RUIP UI programs | Implemented product/demo path | Useful trust/UI proof; avoid confusing it with general app compilation. |
| Project workspace and host-built Wasm | Implemented in bounded forms | Transitional workshop route. |
| On-device rustc factory | Real, highly specialized QEMU proof | Strongest differentiator; not yet a general platform. |
| `raios-lang` / mini-language | Implemented library, parked | Keep as spare encoder; not current critical path. |
| General games/video editor factory | Long-term scale goal | Not current capability and should not widen near-term Scope. |
| Local registry/OTA/fake cloud | Host tools exist, runtime partial | Distribution phase; do not treat as installed product. |
| Secret vault / structured store | Implemented, QEMU-coupled in places | Needs one storage/recovery authority story. |
| Crash-loop supervision | ADR exists; implementation is foreign WIP | Parked/blocked, not accepted. |
| Rollback image security | Large foreign WIP | Parked pending authority/contract decision. |
| NET8/Schannel fixture | Large foreign WIP | Infrastructure strand, currently blocked/unfinished. |
| Website/UI dream | Separate active presentation branch | Keep separate from product truth and release claims. |
| Self-explaining system | Untracked analysis/narrative | Potentially useful framing; not a fourth implementation program. |

## 6. Actual implementation architecture

### 6.1 Runtime shape

At the inspected source state, raiOS is a single `no_std` kernel image with a
cooperative event loop.

```text
Limine / UEFI
  -> seed-kernel::_start
  -> early_main
     -> heap + memory + serial
     -> framebuffer + ShellHost
     -> USB / PCI / AHCI / E1000 / Marvell probes
     -> IOMMU detect-only probe
     -> core policy + grant projection
     -> stores and autoload paths
     -> entropy + network
     -> PeriodicTasks::run forever
```

`seed-kernel/src/main.rs` directly includes 117 top-level modules. The live
kernel scheduler is a small TSC-based periodic polling helper. The more
elaborate `JobThreadScheduler` belongs to deterministic Wasm/WASI jobs; it is
not a kernel process scheduler.

There are no separate driver processes, general per-service address spaces or
preemptive native user processes in the inspected implementation.

### 6.2 Main layers

| Layer | Location | Actual role |
|---|---|---|
| Boot/runtime/TCB | `seed-kernel/` | One kernel image: boot, devices, UI, protocol, stores, provider, Wasm and recovery glue. |
| Shared policy/types | `crates/raios-core/` | Host-testable formats, evaluators, projections, crypto and authority contracts. A type here is not automatically a live enforcement gate. |
| Protocol/parser crates | other `crates/raios-*` | DNS, HTTP, X.509, WASI, Wasm IR, acquisition logic and conformance tests. |
| Wasm guest fixtures/services | `wasm-guests/` | Echo, buffer, parser, build assembler, W7 acquisition and personal-shell proof. Mostly embedded and specifically routed, not general deployable processes. |
| Host tools | `tools/` | Descriptor/core-policy signing, import inventory and BuildFS packing. |
| Host distribution | `distribution/` | File/CAS registry, signing CLIs and a fake WebSocket cloud. |
| Build/package control | `scripts/` | Build, image, USB, manifests, source-size, secrets, hardware-lane and website tooling. |
| QEMU/evidence control | `vm-harness/` | Host-side “Shadow VM” profiles, serial commands, mutations, reboots and JSON report generation. |
| Hardware facts | `hardware/` | Machine manifests, schemas and register maps for agents; not runtime-consumed configuration. |
| Public UI/story | `ui-lab/`, `raios-ui-lab.html`, `cloudflare/` | Website/film/design laboratory, separate from the kernel framebuffer implementation. |

### 6.3 Wasm reality

The Wasm layer is real:

- local Wasmi fork;
- fuel and memory limits;
- import-grant tables;
- OOB and malformed-import negatives;
- deterministic suspension/green-thread work;
- concrete signed/embedded guests;
- personal-shell and build paths.

Its limitation is generality:

- most guests are embedded at build time;
- positive execution routes are specific;
- the large generic module-loader vocabulary is not itself a universal loader;
- owner sealing is not complete;
- driver isolation is not provided by Wasm;
- a guest cannot become a general hardware domain with current DMA/MMIO
  enforcement.

### 6.4 Persistence reality

Two persistent architectures coexist:

1. **SEED_DATA / legacy path**
   - BOOTCTL;
   - RECLOG;
   - ARTSTOR;
   - grants, memory records, artifacts, install and recovery records.

2. **C1 structured store**
   - transactional typed store;
   - vault and project source/dependency paths;
   - in several kernel integrations bound to a specific disposable QEMU AHCI
     device/port.

Project installation can use SEED_DATA while project source/dependencies use
the QEMU-specific C1 route. There is no single, clearly dominant persistence
and recovery authority across all subsystems.

### 6.5 Parallel stacks

The code contains several deliberate or accidental double paths:

- native OpenAI TLS vs W7/Wasm opaque TLS session;
- host filesystem registry vs kernel RAM serial catalog;
- periodic kernel poller vs Wasm job scheduler;
- RUIP interpreter vs `raios-wasm-ir` assembler vs rustc/WASI build;
- concrete working service loaders vs large generic loader evidence/projection;
- build-time embedded API key/pins vs vault/lease-based provider credentials;
- legacy SEED_DATA vs C1 structured store.

Each duplicate can be a migration strategy. Without an explicit “old path,
new path, switch criterion, deletion criterion” record, it becomes a source of
architectural ambiguity.

## 7. File and data map

### 7.1 Tracked repository

The inspected checkout has 1,378 tracked files.

| Top-level area | Tracked files | Interpretation |
|---|---:|---|
| `vendor/` | 552 | Pinned/forked dependencies; not raiOS product logic, but local patches matter. |
| `seed-kernel/` | 275 | Main product/TCB concentration. |
| `docs/` | 183 | Scope, ADRs, status, plans, protocols, archive and assets. |
| `crates/` | 133 | Shared policy, parsers, WASI, Wasm IR and tests. |
| `vm-harness/` | 54 | QEMU/evidence orchestration. |
| `ui-lab/` | 44 | Public/product design lab. |
| `scripts/` | 39 | Build, packaging, checks and host orchestration. |
| `distribution/` | 25 | Host signing, registry and fake cloud. |
| `wasm-guests/` | 19 | Nine embedded/specific guest crates. |
| `tools/` | 16 | Host tooling. |
| `_attic/` | 13 | Quarantined, non-active history. |
| `hardware/` | 8 | Agent-facing machine and register data. |
| `release/` | 6 | Only small tracked boot/release inputs; most reports/images are ignored. |

The Cargo workspace contains 28 members:

- 1 kernel;
- 10 `crates/raios-*` libraries/test crates;
- 9 Wasm guests;
- 4 host tools;
- 4 distribution components.

### 7.2 First-party scale

Approximate text-line inventory at the inspected worktree:

| Area | Selected text files | Approx. lines |
|---|---:|---:|
| `seed-kernel/` | 191 | 192k+ |
| `crates/` | 132 | 69k+ |
| `vm-harness/` | 59 including local generated/WIP view | 28k+ |
| `scripts/` | 35 selected source files | 10k+ |
| `docs/` | 173 selected text files | 52k+ |

The permanent kernel is therefore not currently “tiny” in any ordinary source
or TCB sense.

Representative large kernel files include:

- `agent_protocol_memory.rs`;
- `event_log.rs`;
- `agent_protocol_module_load_gate_render.rs`;
- `usb.rs`;
- `durable_store.rs`;
- rollback authority/emitter modules;
- `module_evidence.rs`;
- WASI build-job glue.

The repository’s own source-size check currently fails because
`seed-kernel/src/usb.rs` exceeds the hard line cap. Several other oversized
files pass only through exact no-growth exemptions. This directly contradicts
README language that every source file is below the readability thresholds.

### 7.3 Generated and ignored bulk

The physical folder is much larger than the tracked source:

| Directory | Files observed | Approx. size |
|---|---:|---:|
| `.cargo-home/` | 17,601 | 311 MiB |
| `target/` | 1,419 | 322 MiB |
| `target-tools/` | 1,144 | 386 MiB |
| `release/` | 1,570 | 415 MiB |
| `docs/` | 184 | 132 MiB, largely assets |

This bulk is primarily caches, binaries, VM reports, images and assets. It
explains filesystem clutter but not product architecture. A following AI must
not treat generated `release/` or `target/` contents as accepted source state.

### 7.4 Documentation map

| Purpose | Location | Caution |
|---|---|---|
| Product target | `docs/SCOPE.md` | Binding by rule, but internally contradictory. |
| Checkbox detail | `docs/scope/` | Must not redefine Scope; currently does for drivers. |
| Durable decisions | `docs/architecture/decisions/` | Read status, supersession and stale footers. |
| Active sequencing | `docs/plans/` | Plans can lag completed work or retain abandoned wording. |
| Current cursor | `docs/status/HANDOFF.md` | Use from canonical ref, not this detached root. |
| Derived status | `STATUS.md`, `OWNER_DASHBOARD.md` | Valuable history, but stale and no longer sole authority. |
| Historical plans/reviews | `docs/_archive/` | Evidence of direction changes, not active instructions. |
| Quarantined code/docs | `_attic/` | Not built/referenced. |
| Runtime/protocol specs | `docs/architecture/device-protocol/` | Several “Current V0” or “spec only” labels are stale. |

## 8. Capability reality matrix

This matrix intentionally avoids one “percent complete” score.

| Capability | Host/unit | QEMU | Surface/bare metal | Durable/reboot | Current verdict |
|---|---|---|---|---|---|
| Limine boot + idle loop | buildable | repeatedly exercised | partial/owner evidence exists | n/a | Real, hardware closure still incomplete in Scope. |
| Framebuffer/serial/input | tests + code | real | Surface bring-up partial | current boot | Real but device-specific. |
| USB/xHCI | models/tests | real QEMU + fixes | real-hardware fixes exist | n/a | Real in-kernel driver, not isolated domain. |
| E1000/network | code/tests | real | not Surface path | current boot | Real QEMU path. |
| Marvell Wi-Fi | extensive models/code | limited structural proof | active main strand, H26 blocked | current boot | Not production-ready; DMA not IOMMU-confined. |
| Wasm OOB/import isolation | host tests | strong focused reports | required bare-metal pair still open | n/a | Best-established security mechanism, claims must stay environment-specific. |
| Fuel/F12/green threads | host conformance | focused QEMU proof | not general hardware proof | current boot | Real for Wasm jobs, not a general OS scheduler. |
| Typed grant/revoke | host policy tests | specific live paths | no broad bare-metal proof | durable for selected surfaces | Real but not every host surface. |
| Crash-loop supervisor | ADR/model | foreign WIP tests | none | current boot design | Not accepted. |
| RECLOG | formats/parser | many records | serial transport/bare-metal closure partial | persistent log paths | Real record format; full crash contract still open. |
| General external module loading | many types/selftests | specific candidate paths | no | partial | No universal loader despite large vocabulary. |
| Bounded UI programs | host + core tests | calculator/editor/current-boot and install paths | no broad proof | some program persistence | Real narrow application path. |
| Project source/build | host and QEMU paths | real bounded workflows | no | sources/install split across stores | Partial platform. |
| rustc inside raiOS | host probe + contracts | real `hello.rs` build, 507/507 referenced | no | compiler/sysroot host-seeded | Substantial spike, not general factory. |
| Structured store/vault | core tests | QEMU-specific integration | no general device binding | reboot profiles exist | Real format, hardware/generalization incomplete. |
| Rollback | specific positive paths | strong domain/program reports | not general hardware | selected reboot paths | Specific implementations, not “every domain.” |
| TPM owner sealing | probe/types | honesty state | hardware presence probe | no sealed production authority | Not implemented. |
| IOMMU/driver confinement | models/branch work | detect/structure | not enabled | n/a | Not implemented. |
| Signed/reproducible distribution | host tools | provenance profiles | no release path | n/a | Later-phase, not product-ready. |
| Unattended hardware loop | scripts/plans | no equivalent | power-cycle/watchdog/ramoops open | intended | Not implemented. |
| Provider-neutral runtime | abstractions exist | OpenAI direct works | connectivity incomplete | secrets mixed | Product target, not current reality. |

Scope counts at both the detached snapshot and locally known `origin/main` are
unchanged:

- top-level `SCOPE.md`: 7 checked, 38 open;
- breakdowns combined: 61 checked, 62 open;
- driver/hardware breakdown: 0 checked, 15 open.

These are inventory numbers, not progress percentages.

## 9. Evidence and test reality

### 9.1 What is strong

The harness does more than compile:

- boots QEMU;
- drives serial and virtual physical input;
- mutates images;
- performs multi-boot tests;
- checks negative boundaries;
- records hashes and predicate results;
- keeps failed reports;
- has caught real bugs and stale tests;
- has rejected green local work after independent review.

That culture is a genuine project asset.

### 9.2 What the on-disk report set actually says

At the inspected folder:

- 509 `shadow-*.json` files exist;
- 508 parse as JSON;
- 1 is malformed/truncated;
- 289 carry scalar result `passed`;
- 217 carry scalar result `failed`;
- 2 have a non-enum command/object string in the `result` field;
- among the latest report per 39 profiles, 34 are green and 5 are red;
- the newest report is red:
  `shadow-rollback-grant-delta-20260720-113115-7712.json`;
- the latest `full` report is red because image packaging exited 101;
- the latest build-assemble, persistence-reboot and one rollback-isolation
  profile are also red for different reasons.

The failed report history is not itself bad. Keeping failures is useful. The
problem is that Scope text uses the number of report files as if every file
were a current, fully identity-bound closure proof.

### 9.3 Missing source identity

Every parsed `raios.vm_test_report.v0` top-level object has fields for image,
candidate, QEMU arguments, hardware profile, logs and predicates. None of the
508 parsed reports has a top-level `commit`, `git` or `head` field.

Therefore the Scope claim that reports carry “what hardware/commit” is not
supported by the current report schema.

Image hashes and commit messages that mention report IDs provide partial
provenance, but they are not equivalent to embedding and validating an exact
source-tree identity in the report.

### 9.4 Predicate count is not independent coverage

The project’s own historical status documents describe cases where:

- many supposed security checks matched one repeated byte pattern;
- a test reached the wrong response;
- stale expectations failed after a representation change;
- infrastructure failures produced red reports with zero failed predicates.

The repairs demonstrate good honesty, but also show that a count such as
“8,205 predicates” cannot be treated as 8,205 independent security properties.

A strong future audit unit is:

```text
claim
-> exact mechanism
-> exact trigger
-> exact observed effect
-> exact negative mutant
-> exact source/image/hardware identity
```

### 9.5 Checkers can be green while semantics are red

Observed in this audit:

- `scripts/check-docs-hygiene.ps1` is green: 12 checks, 0 violations;
- the Scope still contains a direct kernel-driver vs driver-domain conflict;
- `scripts/check-source-size.ps1` is red because `usb.rs` exceeds hard cap.

This illustrates the main meta-risk: mechanical governance validates what it
models, not the whole meaning of the project.

## 10. Contradictions and likely misdirections

The following are not all “bugs.” They are decision points where continued
implementation without clarification is likely to waste work.

### P0 — current folder vs canonical main

**FACT:** this worktree is detached and 92 main commits behind.<br>
**Risk:** any new implementation or status update can land on the wrong
architecture and combine unrelated old WIP.<br>
**Required action:** resolve canonical ref/worktree before any product lane.

### P0 — driver end state

**Kernel model:**

- `docs/SCOPE.md` §§1–3;
- ADR 0005;
- current code;
- performance and early bring-up practicality.

**Domain model:**

- `docs/SCOPE.md` §5;
- `docs/scope/05-drivers-hardware.md`;
- driver plans;
- small-TCB and fault-isolation product promise.

**Blocking fact:** VT-d translation is not enabled and live driver-capability
checks are not called from drivers. Full DMA-safe driver domains are not
present.

**OPEN QUESTION:** Is in-kernel a temporary bring-up stage with an explicit
migration criterion, or the permanent architecture?

### P1 — “tiny immutable core” vs actual TCB

README describes a tiny, immutable, write-protected permanent core. Actual
`seed-kernel` has roughly 192k text lines, 117 top-level modules and directly
contains:

- Agent Protocol;
- memory/event vocabulary;
- USB/AHCI/E1000/Marvell;
- storage and recovery;
- TLS/OpenAI;
- UI and workspaces;
- Wasm and WASI build glue.

There is no demonstrated write-protected core partition against everything
above it. “Tiny immutable core” is a target narrative, not current reality.

**RECOMMENDATION:** create a falsifiable TCB map and budget:

```text
module -> authority -> unsafe/MMIO/DMA -> secret access
       -> crash effect -> update path -> intended final layer
```

### P1 — factory proof vs factory platform

The rustc proof is real and should be preserved. It is also specialized:

- one pinned ~95 MB compiler;
- host/offline-seeded compiler and sysroot images;
- one built-in/simple source fixture;
- large QEMU memory profile;
- interpreter execution, not fast AOT;
- general JSON diagnostics, templates, crash behavior and promotion remain
  open.

Do not call the external workshop eliminated yet. The proof shows that the
compiler can run inside raiOS; it does not show that raiOS can autonomously
accept and iterate on arbitrary projects.

### P1 — workshop route vs “without workshop”

The owner’s factory plan explicitly says the host workshop should become
temporary bootstrap. Current reality still relies heavily on:

- PowerShell build/package scripts;
- QEMU orchestration;
- host signing and image mutation;
- host-seeded compiler/sysroot;
- host-side provider and test fixtures;
- 28k+ lines of VM harness.

This is acceptable for bootstrap, but only if every host dependency has one of:

- permanent trusted build input;
- migration plan into raiOS;
- explicit decision to remain external.

### P1 — evidence vocabulary vs live authority

There are hundreds of `raios.*.vN` schema literals and large protocol/render
modules. Some shared-core evaluators, driver-capability types and loader
projections have no live caller outside tests or evidence paths.

The project has already diagnosed this failure mode once (“ceremony before
mechanism”). It remains a risk whenever a new schema, report field or denial
surface is counted as product progress without a live effect boundary.

### P1 — “scoped gate” naming vs operation order

Static inspection found legacy append/write functions that:

1. plan a frame;
2. perform AHCI write/readback;
3. call a `evaluate_scoped_*` function containing actual write/readback fields.

Examples exist in durable records, boot-success audit, repromotion audit and
ARTSTOR paths. These evaluators may intentionally be postcondition/evidence
checks, and earlier call layers may already hold authority.

**Do not report this as a confirmed vulnerability without a focused review.**
It is a high-priority naming and authority-order audit:

- which gate authorizes before I/O?
- which evaluator only verifies after I/O?
- can any caller reach the write with merely well-shaped data?
- does a failed post-write decision leave a durable side effect?

The newer build-output path does evaluate before I/O, showing that the codebase
already distinguishes the safer order in at least one subsystem.

### P1 — hardware fast track vs uncontained DMA

The recent main history spends about 92 commits beyond this snapshot largely on
one Surface/Marvell strand.

Arguments for that focus:

- the product is bonded to one Surface;
- the device lacks convenient serial/wired-Ethernet development paths;
- connectivity is required for the resident agent story;
- real hardware bugs cannot be solved in QEMU.

Arguments against continuing unchanged:

- IOMMU remapping is not active;
- the Marvell path is high-trust, in-kernel and DMA-capable;
- the newest attempt is blocked by a concurrency race;
- hardware runs require owner custody;
- unattended recovery/power-cycle/ramoops are still open;
- the factory’s coherent user path remains incomplete.

**OPEN QUESTION:** Is Wi-Fi the current critical dependency, or should a bounded
alternative connectivity path unblock the factory while hardware isolation and
observability are built?

### P1 — direct OpenAI vs provider-neutral architecture

Current kernel code has a real direct OpenAI path and build-time trust/key
options. README simultaneously says:

- provider agnostic;
- the kernel does not parse the Internet;
- network/TLS belongs in replaceable services.

The README’s “current reality” partially admits the mismatch, but its main
architecture prose reads as if the migration had already happened.

### P2 — status authority drift

Several sources compete:

- user-supplied/current AGENTS rules say HANDOFF is the current memory window;
- README calls `STATUS.md` authoritative;
- docs hygiene still checks `STATUS.md` and `OWNER_DASHBOARD.md`;
- current folder HANDOFF is July 21;
- current folder STATUS/dashboard are older;
- origin/main HANDOFF is July 22 and describes a different worktree/WIP set.

Choose one current-state authority and label the others derived/history.

### P2 — ADR status drift

Examples:

- ADR 0008/0009 say accepted at the top and proposal at the bottom;
- ADR 0010 remains proposed while plans assume its driver model;
- ADR 0026 is partly superseded by 0029;
- old invariant sheets still claim no Wasm host/persistence;
- old protocol docs describe planned/denied loaders as “Current V0.”

This makes “read all ADRs” insufficient; a following AI needs a supersession
map.

### P2 — Cranelift wording vs interpreter route

Top-level Scope still says “rustc with Cranelift backend.” The breakdown itself
flags that wording as stale. The implemented route is an unmodified threaded
rustc Wasm artifact running under the caged interpreter, with any fast AOT stage
deferred to a future ADR.

### P2 — Shadow VM terminology

README describes a sealed parallel environment that evaluates candidate
behavior. Current implementation is primarily a host-side QEMU harness that
boots images, drives serial/input and writes reports. That is useful and real,
but it is not obviously an in-OS always-available shadow execution domain.

Public and internal claims should distinguish:

- host QEMU test harness;
- guest Wasm sandbox;
- future on-device shadow environment.

### P2 — public UI and release claims

The website/UI lab contains downloadable/installer-looking actions that are
mockups. `release/README.md` describes releases with images, logs, checksums and
SBOMs while only small boot inputs are tracked and current images/reports are
ignored local artifacts.

Presentation is a separate valid strand, but it must not become product-state
authority.

### P3 — stale branches, worktrees and generated bulk

Thirty registered worktrees, 28 local branches, a stash, hundreds of ignored
reports and more than a gigabyte of caches make state discovery harder.

No cleanup should occur until owners and exact recoverability are established.
After that, a dedicated governance task can distinguish:

- canonical source;
- recoverable lane history;
- patch-equivalent stale refs;
- archived reports to retain;
- reproducible cache/build output.

## 11. Current dirty WIP in this folder

The initial and final pre-report worktree had 11 modified tracked files plus
untracked material. These files belong to several independent strands.

### 11.1 Rollback image / grant-delta security

Files:

- `scripts/make-gpt-persist-image.py`;
- `scripts/rollback_image_security.py`;
- `scripts/tests/rollback-image-security/test_contract.py`;
- `vm-harness/shadow-vm-persistence-reboot.ps1`.

Direction:

- signed predecessor fixtures;
- grant-fold and rollback target reconstruction;
- hostile mutations/resealing/duplicate installs;
- cut-point recovery and multi-boot verification.

Status from HANDOFF/history: parked pending authority/contract decisions. This
large WIP must not be treated as accepted rollback closure.

### 11.2 Crash-loop supervision

Files:

- `seed-kernel/src/echo_service.rs`;
- `seed-kernel/src/recovery_lifeline.rs`;
- `seed-kernel/src/wasm_runtime.rs`;
- untracked `seed-kernel/src/wasm_runtime/crash_loop_supervisor.rs`;
- `vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1`;
- `seed-kernel/src/main.rs` includes a small overlapping/formatting change.

Direction:

- bounded crash counts;
- replace once, then park;
- exact recovery/unpark;
- current-boot authority.

Status: ADR 0031 exists, but the implementation is not in HEAD or
`origin/main`; earlier review parked the strand and requested wider file
allocation. Not accepted.

### 11.3 NET8 / Schannel fixture

Files:

- `vm-harness/net8-w7-tls-fixture.ps1`;
- `vm-harness/net8-w7-tls-fixture/Program.cs`;
- related persistence harness changes.

Direction:

- process-local TLS 1.3 fixture;
- SNI/SPKI binding;
- readiness and child-process isolation.

Status: unfinished infrastructure WIP; several latest reports fail because the
fixture did not publish its ready file.

### 11.4 Untracked narrative/diagnostics

- self-explaining-system report;
- system-comparison HTML;
- `release/diagnostics/`;
- experiment `Cargo.lock`.

These are analyses/generated evidence, not accepted product code.

## 12. What should be preserved

Even a major direction correction should retain:

1. real Wasm memory/import/fuel boundaries and their negative tests;
2. the separation of fact, evidence, decision and actual authority;
3. revocation, re-verification and exact artifact identity;
4. recovery/rollback as a product invariant rather than a backup afterthought;
5. the real rustc-in-raiOS proof and the measurements that led to it;
6. real Surface/USB work and honest hardware uncertainty;
7. explicit `dev_key_not_owner_sealed` / non-authorizing labels;
8. the practice of preserving failed reports and rejecting green WIP after
   independent review;
9. the custom-kernel-agnostic Genesis import/service contract;
10. archive/history rather than silent deletion.

## 13. Decision agenda for a subsequent AI

The following AI should not start by proposing code. It should produce neutral,
evidence-backed options for the owner in this order.

### Decision 1 — canonical source state

Questions:

- Is `origin/main/f1cb333` the intended base?
- What should happen to the two unique detached audit commits?
- Which WIP belongs to this old root vs the canonical main worktree?
- Are any local refs not pushed/recoverable?

Output:

- exact canonical SHA;
- preserved foreign WIP manifest;
- no cleanup or merge without owner/orchestrator authority.

### Decision 2 — one next product proof

Candidate A: on-device factory user journey.<br>
Candidate B: operable reference hardware with connectivity/recovery.<br>
Candidate C: end-to-end evidence-bound promotion/rollback.<br>
Candidate D: self-explaining authority query plane.

These overlap, but only one should be the acceptance headline for the next
phase. The others become prerequisites or parked follow-ups.

### Decision 3 — driver timeline

Options to compare neutrally:

- permanently in-kernel;
- in-kernel bring-up, then native isolated domains;
- Wasm “brain” plus kernel actuator;
- mixed per-device model.

Required facts:

- IOMMU availability and activation plan;
- MMIO/IRQ/DMA authority;
- restart semantics;
- latency needs;
- failure effect;
- update/signing model;
- migration seam from current code.

This decision requires owner approval and an ADR because it changes the
architecture/security model.

### Decision 4 — TCB budget and boundaries

Define:

- permanent core modules;
- permitted Internet parsing;
- permitted storage formats;
- driver trust tier;
- secrets/keys;
- maximum source/unsafe/authority surface;
- what can be restarted or replaced;
- what “immutable” technically means.

### Decision 5 — factory generalization gate

Before more compiler breadth, require one non-builtin small project:

- multiple files;
- compile error and machine-readable diagnosis;
- corrected revision;
- deterministic double build;
- measured time and RAM;
- owner approval;
- execution and rollback;
- no special source constant embedded in the kernel.

### Decision 6 — evidence identity and audit

Revise or version report contracts to bind:

- Git/source-tree identity;
- exact kernel/image hash;
- exact candidate;
- toolchain;
- hardware/machine manifest digest;
- test profile version;
- negative mutant identity;
- result enum validation.

Then audit ten high-risk claims rather than counting all predicates.

### Decision 7 — persistence convergence

Choose and document:

- which data belongs in RECLOG/ARTSTOR;
- which data belongs in C1 structured store;
- which store is available on the Surface;
- authority before write;
- atomicity and power-loss model;
- recovery ownership;
- migration/deletion criteria for duplicate paths.

### Decision 8 — provider and networking boundary

Decide the bootstrap/final boundary:

- direct OpenAI in kernel;
- Wasm network/TLS service;
- provider adapter service;
- external bridge during bootstrap.

The public provider-neutral claim should not outrun the chosen live boundary.

### Decision 9 — hardware operating loop

Before more risky physical driver attempts, consider requiring:

- crash-visible framebuffer/RECLOG facts;
- machine-manifest binding;
- safe image/USB provenance;
- watchdog;
- power-cycle path;
- retained pre-network crash evidence;
- a bounded fallback connectivity route.

### Decision 10 — documentation truth pass

After owner decisions, synchronize:

- Scope §§1, 5 and 6;
- relevant breakdowns;
- ADR status/supersession;
- README current vs target wording;
- HANDOFF as current authority;
- STATUS/dashboard as derived or archived views;
- release and website claims;
- stale protocol “Current V0” labels.

## 14. Recommended sequence

```text
P0  establish canonical SHA and preserve all foreign WIP
P0  resolve driver end-state/time-line with owner
P1  choose one next product proof
P1  build TCB/authority map
P1  audit ten critical claims with exact source-bound mutants
P2  generalize the on-device factory by one non-builtin project
P2  establish safe, observable reference-hardware operation
P3  converge persistence/provider paths
P3  only then expand distribution and public installer/UI claims
P4  prune stale refs, worktrees, reports and caches in a separate authorized task
```

This ordering deliberately does not throw away hardware or security work. It
forces each to support one selected product loop.

## 15. Questions a following AI must not silently answer

- Is the factory, hardware appliance or research kernel the next primary
  product?
- Are drivers permanently in-kernel?
- Does “immutable core” require physical/write-protected image separation?
- Is external workshop bootstrap permanently allowed?
- Is direct OpenAI a supported product path or a temporary bootstrap?
- When does distribution begin: second owner machine, first stranger, or now?
- Is owner sealing mandatory before any durable install claim?
- Does a report without source commit identity close a checkbox?
- Is QEMU “Shadow VM” the final shadow architecture?
- Should the old service-graph vision remain active?
- What is the fate of `raios-lang`, driver-domain plans and the two unique
  detached audit commits?

These are owner/ADR questions, not implementation-lane discretion.

## 16. Machine-oriented handoff

```yaml
snapshot:
  date: 2026-07-24
  inspected_head: 11bcdeb
  checkout: detached
  locally_known_origin_main: f1cb333
  head_only_commits: 2
  origin_main_only_commits: 92
  foreign_dirty_tracked_files: 11
  canonical_worktree_claimed_by_newer_handoff: '<canonical-main-worktree> (local absolute path redacted)'

strongest_current_inference:
  product: evidence-bound on-device software factory on a personal custom-kernel OS
  isolation: bounded Wasm linear memory plus explicit host imports
  authority: local typed evidence plus owner approval, revocable grants and rollback
  hardware: enabling substrate, current drivers actually kernel-resident

blocking_decisions:
  - canonical source ref and worktree
  - driver kernel-vs-domain end state
  - next product acceptance journey
  - permanent TCB budget
  - workshop bootstrap boundary
  - report/source identity contract

do_not_claim:
  - tiny immutable core is already realized
  - drivers are isolated domains
  - VT-d translation or DMA confinement is active
  - provider neutrality is implemented
  - the external workshop is no longer required
  - every VM report is green or commit-bound
  - general module loading or general app factory is complete
  - crash-loop supervision WIP is accepted
  - untracked self-explaining narrative is an ADR

preserve:
  - Wasm OOB/import/fuel negatives
  - typed fact/evidence/decision/authority separation
  - revoke/reverify/rollback work
  - real rustc-in-QEMU proof
  - real Surface/USB observations
  - honest non-authorizing/dev-key labels
  - failed-report history and independent review culture
```

## 17. Method and coverage

The audit used five independent read-only Codex lanes:

1. documentation/ADR/scope direction history;
2. implementation and component map;
3. build/harness/evidence/release reality;
4. full reachable Git archaeology;
5. neutral product/security/architecture critic.

The orchestrator independently checked:

- Git status, refs, branches, worktrees, history and divergence;
- Scope and breakdown checkbox inventories;
- Cargo workspace and entry points;
- top-level file/size distribution;
- source-size and docs-hygiene checkers;
- current dirty diff ownership;
- 509 local VM report files and latest-per-profile state;
- current and `origin/main` HANDOFF differences;
- representative runtime, storage, Wasm, hardware and authority call paths.

Coverage statement:

- all tracked and visible untracked paths were inventoried;
- first-party docs, code, plans, branches and evidence flows were semantically
  sampled by dedicated lanes;
- vendored dependencies, caches, binaries and generated reports were
  inventoried, not interpreted line-by-line as raiOS design;
- no network, cloud, hardware, QEMU, build, commit, push or destructive cleanup
  was performed;
- foreign worktree changes were preserved.

Limits:

- no fresh runtime or hardware result was generated;
- no complete literature, patent or market review was performed; comparisons
  are limited to the repository material and named sources inspected for this
  snapshot;
- locally missing/deleted refs and unfetched remote history cannot be audited;
- static code findings such as post-write evaluators require focused
  independent review before being called vulnerabilities;
- this report identifies decisions but does not make owner-only Scope or
  security decisions.

## 18. Key local sources

Start here:

- `AGENTS.md`
- `docs/SCOPE.md`
- `docs/status/HANDOFF.md`
- `docs/scope/01-rust-kernel.md`
- `docs/scope/05-drivers-hardware.md`
- `docs/scope/06-personal-rust-playground.md`
- `docs/plans/plan-personal-rust-playground.md`
- `docs/architecture/decisions/0001-raios-agent-protocol.md`
- `docs/architecture/decisions/0003-always-on-core-and-live-rebuildable-world.md`
- `docs/architecture/decisions/0004-system-memory-and-agent-context.md`
- `docs/architecture/decisions/0005-bare-metal-substrate-and-wasm-isolation.md`
- `docs/architecture/decisions/0008-per-service-wasm-import-grants.md`
- `docs/architecture/decisions/0010-shareable-driver-modules.md`
- `docs/architecture/decisions/0015-custom-kernel-statt-sel4.md`
- `docs/architecture/decisions/0019-branchless-main-and-single-git-writer.md`
- `seed-kernel/src/main.rs`
- `seed-kernel/src/wasm_runtime.rs`
- `crates/raios-core/src/lib.rs`
- `vm-harness/shadow-vm-smoke.ps1`
- `vm-harness/shadow-vm-smoke-support.ps1`
- `scripts/check-source-size.ps1`
- `scripts/check-docs-hygiene.ps1`

Key historical commits:

- `af4034a` — verified Stage-0 restart;
- `4611359` — direct OpenAI HTTPS;
- `9df2044` — ADR 0005, mechanism before vocabulary;
- `c8d6f74`, `3b057a9`, `a10f209` — Wasmi and first/second guests;
- `26b11c2` — owner acceptance of import/distribution ADRs;
- `3b70b9b` — remove ~45.8k legacy recovery lines;
- `443c739`, `3d164ca` — rustc route parked by measurements;
- `2890b63`, `d27a52e` — `raios-lang` then factory re-centering;
- `37929ba` through `b8c573c` — threaded rustc route and real in-system compile;
- `9cf73ee` — Scope §§1–3 reframed to current Wasm/kernel-driver model;
- `fc05ed5`, `4557600` — website-only main then product restoration;
- `f1cb333` — latest locally known main, H26 parked after race rejection.

## Final assessment

raiOS is not failing because it has no direction. It is failing to distinguish
the hierarchy among several directions.

The repository already contains the seed of a distinctive product:
agent-authored software does not become authority merely because an AI wrote it;
it must pass through a local, typed, evidence-bound, revocable and recoverable
OS transaction. The on-device rustc experiment makes that more than a slogan.

The immediate danger is that this core becomes buried under:

- a monolithic expanding TCB;
- contradictory driver targets;
- hardware work without active DMA isolation;
- duplicated transitional stacks;
- status/ref divergence;
- report quantity mistaken for source-bound proof;
- public future-state language read as current capability.

The next AI should therefore optimize for **convergence and falsifiability**,
not for another subsystem. Resolve the canonical source state, settle the
driver timeline, choose one end-to-end product proof, and make every remaining
strand explicitly support or wait behind that proof.
