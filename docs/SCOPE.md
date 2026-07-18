# raiOS — Scope & Vision

> **Status:** Target picture, written as if raiOS were finished. Every checkbox
> is a capability the finished system must have. Nothing here is checked —
> a box gets checked only when predicates prove it.
>
> **Two levels:** Each category links to a breakdown file in `docs/scope/`.
> A top-level box may be checked only when every box in its breakdown is green.
> Breakdown files may evolve (lanes propose, orchestrator commits); THIS list
> changes only with owner approval.

---

## What raiOS is (finished)

raiOS is an agent-native operating system. A minimal capability-based Rust
kernel (Genesis Layer) safely multiplexes bare hardware. Everything above it —
drivers, network stack, graphics, applications — is built by agents in isolated
domains with explicitly granted hardware capabilities. A failed agent attempt
never tears down the system, only its own domain.

raiOS deliberately replaces the missing mathematical proof with a trust
pipeline: reports, rollback, and escape negative tests from day 1; signed and
reproducible builds once strangers use it. A documented trade, not an accident.

**Reference hardware:** x86_64, bare metal on Surface. **Language:** Rust.
**Developers:** agents (10 lanes, 1 orchestrator). **Human:** sets goals, grants rights.

---

## 1. Rust Kernel (serial core) → `docs/scope/01-rust-kernel.md`

- [ ] Boots on bare metal (UEFI) to a stable idle state
- [ ] MMU/paging: strictly separated address spaces per domain
- [ ] Scheduler: preemptive, domain-fair, kill-capable
- [ ] Syscall interface: minimal, stable, versioned
- [ ] Interrupt routing to userspace domains (drivers do NOT live in the kernel)
- [ ] IOMMU mandatory for everything DMA-capable
- [ ] `unsafe` inventory: every unsafe site documented + predicate-covered
- [ ] Serial/debug output machine-readable (RECLOG frames)

## 2. Genesis Layer (capability floor) → `docs/scope/02-genesis-layer.md`

- [ ] Primitives: `create_domain`, `grant_capability`, `revoke_capability`, `kill_domain`
- [ ] Storage primitive (persistent block access as a capability)
- [ ] Fine-grained capabilities: exactly one PCIe BAR, one IRQ line, one DMA region
- [ ] Kill + restart of a domain without system reboot, in < 1 s
- [ ] **Floor interface narrow & kernel-agnostic** — seL4 stays substitutable
      beneath it (documented contract, no kernel internals leak through)

## 3. Security & Trust Pipeline → `docs/scope/03-security-trust-pipeline.md`

**Day 1 — proving the foundation** (measuring device while building, non-negotiable):

- [ ] Escape negative tests as predicates:
  - [ ] Domain touches another domain's memory → denied + logged
  - [ ] Domain touches kernel memory → denied + logged
  - [ ] Domain uses a foreign DMA region → blocked by IOMMU + logged
- [ ] Rollback: every domain version can be rolled back
- [ ] Report pipeline: every build/test emits a structured report (ARTSTOR)

**Distribution phase — trust for strangers** (only once others run the system):

- [ ] Signed builds + double build (reproducibility)
- [ ] Audit log: every capability grant traceable (who, what, when, why)

## 4. Agent Fabric (orchestration) → `docs/scope/04-agent-fabric.md`

- [ ] 10 parallel agent lanes + 1 orchestrator in one workspace
- [ ] Machine-readable hardware introspection: PCI enumeration, register maps,
      device info as structured data (not PDFs)
- [ ] Compiler diagnostics as JSON → direct agent feedback loop
      (generate → compile → read errors → fix)
- [ ] Test harness (QEMU + bare metal) drivable by agents themselves (W6 machinery)
- [ ] Lane rules documented: what parallelizes (drivers, predicates, pipeline),
      what stays serial (MMU, scheduler, syscalls — max 2 lanes)
- [ ] Log domain: minimal early-boot domain streaming RECLOG frames via UDP
      to the orchestrator — survives the crash of every other domain
- [ ] Ramoops region: reserved RAM survives warm reboot; pre-network crash logs
      are read back and forwarded after reboot
- [ ] Remote power-cycle (smart plug/relay) + hardware watchdog: the bare-metal
      loop (deploy → boot → stream → hang → cycle → read crash log → retry)
      runs unattended, including overnight
- [ ] Doc discipline: HANDOFF (~2 KB, displacement rule) read by every agent at
      session start, overwritten at session end

## 5. Drivers & Hardware (agent-built, in domains) → `docs/scope/05-drivers-hardware.md`

- [ ] Wi-Fi (Marvell port) runs as an isolated domain with its own DMA region
- [ ] USB stack as a domain
- [ ] Network stack as a domain
- [ ] Storage driver as a domain
- [ ] GPU: framebuffer access as a capability (long-term: 3D/rendering straight on hardware)

## 6. Personal Rust Playground → `docs/scope/06-personal-rust-playground.md`

- [ ] A human or agent can request an empty domain and build in it without
      being able to endanger the system
- [ ] Rust toolchain inside the OS (rustc with Cranelift backend) for self-compilation
- [ ] Template domains ("Hello Hardware"): minimal start with serial-out + 1 capability
- [ ] Crash of a playground domain = log + restart offer, nothing else
- [ ] Playground results can be promoted to "real" domains
      (through the trust pipeline, never around it)

## 7. Docs & Project Hygiene → `docs/scope/07-docs-hygiene.md`

- [ ] This scope document is the single source for "what raiOS is"
- [ ] Docs structure: `SCOPE.md` + `scope/` (breakdowns), `architecture/`
      (+ `decisions/`), `agents/`, `plans/`, `status/`, `assets/`, `_archive/` — nothing else
- [ ] Every architecture decision is an ADR (incl. the seL4 decision, dated)
- [ ] Outdated plans are archived (`docs/_archive/`), never silently deleted

---

## Deliberate non-goals

- **No formal proof.** Substitute: predicates + negative tests (§3). Documented trade.
- **No POSIX, no Linux compatibility.** raiOS is not a Unix.
- **No multi-user desktop OS.** One machine, one owner, many domains.
- **No legacy hardware.** Reference hardware + QEMU only.

---

*Definition of done per checkbox: the capability exists + at least one predicate
proves it + one negative test proves its boundary.*
