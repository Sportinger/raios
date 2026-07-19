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
kernel (Genesis Layer) safely multiplexes bare hardware. Drivers and other
performance-critical hardware paths deliberately remain in-kernel; agent-built
replaceable code runs as Wasm services whose bounds-checked linear memory and
explicit capability imports form the isolation boundary. A guest cannot name
kernel or another guest's memory, and a failed agent attempt stops only its own
service rather than tearing down the system.

Sections 1–3 were reframed to this architecture under ADR 0005 and ADR 0015
with owner approval on 2026-07-19; rewording does not make any checkbox green.

raiOS deliberately replaces the missing mathematical proof with a trust
pipeline: reports, rollback, and escape negative tests from day 1; signed and
reproducible builds once strangers use it. A documented trade, not an accident.

**Reference hardware:** x86_64, bare metal on Surface. **Language:** Rust.
**Developers:** agents (10 lanes, 1 orchestrator). **Human:** sets goals, grants rights.

---

## 1. Rust Kernel (serial core) → `docs/scope/01-rust-kernel.md`

- [ ] Boots on bare metal (UEFI) to a stable idle state
- [ ] Panic → machine-readable RECLOG crash frame before halt; an induced panic
      must produce the frame rather than fail silently
- [ ] Early serial/debug is live before higher initialization; an early-init
      failure must still leave a machine-readable diagnostic
- [ ] Boot time + memory footprint recorded per build; a missing or regressing
      measurement fails its report predicate
- [ ] Agent-built code runs as Wasm guests, isolated by bounds-checked linear
      memory + capability import grants: no shared address space, and a guest
      cannot name kernel or another guest's memory; OOB access traps + logs with
      zero host effect
      <!-- evidence (OOB boundary): release/vm-reports/shadow-20260719-084519-8004.json,
      predicate command:isolation.selftest; orchestrator verification required. -->
- [ ] Wasm-runtime scheduling: fuel-metered guests, fair budgets, and F12
      kill/suspend of a running guest; a hostile busy loop exhausts only its
      budget while the kernel and peer guests remain responsive
- [ ] Wasm import/service interface: minimal, stable, versioned; malformed or
      unavailable imports fail closed + log without host effect
- [ ] Drivers run in-kernel (deliberate, ADR 0005): the kernel keeps hardware
      authority and guests receive only explicit capability imports; a guest
      without an import cannot reach the corresponding host surface
      <!-- evidence (default-deny import boundary):
      release/vm-reports/shadow-20260714-114527-24812.json,
      predicates m11-import-grant:unauthorized-import-refused and
      m11-import-grant:forbidden-import-link-failure-preserved; orchestrator
      verification required. -->
- [ ] `unsafe` inventory: every unsafe site documented + predicate-covered
- [ ] Serial/debug output machine-readable (RECLOG frames)

**Explicit future hardware hardening (not a current checkbox):** enable VT-d
translation and enforce IOMMU isolation for every DMA-capable device. The
existing `iommu_vtd.rs` probe is structure-only; translation is not enabled.
The future predicate must prove translation active, and its negative test must
show a foreign/out-of-range DMA attempt blocked + logged with zero host effect.

## 2. Genesis Layer (capability floor) → `docs/scope/02-genesis-layer.md`

- [ ] Create a fresh Wasm service/guest with zero grants by default; its attempt
      to use any ungranted host import is denied + logged with zero host effect
- [ ] Persistent storage is a range/quota-scoped ARTSTOR/structured-store
      capability; absent or out-of-scope writes are denied + logged
- [ ] Typed import grants: explicit grant/revoke + durable audit record; one
      import grant conveys no authority to another host surface
      <!-- evidence (default-deny/import-scope boundary only):
      release/vm-reports/shadow-20260714-114527-24812.json, passed
      m11-wasm-import-grant profile; grant/revoke verification still required. -->
- [ ] Kill + restart a Wasm service in < 1 s without reboot; a crash loop parks
      + reports the service, and restart leaks no grants or state from its prior life
- [ ] **Floor interface narrow & kernel-agnostic:** ADR 0015 chooses the custom
      kernel; the substitutable floor is the documented Wasm-import + service-
      capability contract, with no kernel-internal types. A contract test rejects
      any service that depends on kernel internals
      <!-- docs/architecture/genesis-layer.md exists (2026-07-19, cited from
      code); the breakdown's conformance-test boxes remain open. -->

## 3. Security & Trust Pipeline → `docs/scope/03-security-trust-pipeline.md`

**Day 1 — proving the foundation** (measuring device while building, non-negotiable):

- [ ] Escape negative tests as predicates:
  - [ ] A Wasm guest reads/writes outside its linear memory → traps + logged,
        zero host or peer-guest effect
        <!-- evidence (OOB boundary): release/vm-reports/shadow-20260719-084519-8004.json,
        predicate command:isolation.selftest (OOB store/load/offset trapped,
        logged=1, host_exposed=0); orchestrator verification required. -->
  - [ ] A guest without a capability import cannot reach that host surface →
        denied + logged, zero host effect
        <!-- evidence (default-deny import boundary):
        release/vm-reports/shadow-20260714-114527-24812.json,
        predicates m11-import-grant:unauthorized-import-refused and
        m11-import-grant:forbidden-import-link-failure-preserved; orchestrator
        verification required. -->
- [ ] Rollback: every domain version can be rolled back
- [x] Report pipeline: every build/test emits a structured report (ARTSTOR)
      — breakdown group green: 427 `release/vm-reports/shadow-*.json`, each
      carrying identity + verdict, gating every closure (see `scope/03`)

**Explicit future hardware hardening (not a current checkbox):** after VT-d
translation is enabled, add the DMA escape predicate: a foreign/out-of-range
DMA attempt is blocked by the IOMMU + logged with zero host/peer effect. The
current VT-d probe is structure-only and does not establish this property.

**Distribution phase — trust for strangers** (only once others run the system):

- [ ] Signed builds + double build (reproducibility)
- [ ] Audit log: every capability grant traceable (who, what, when, why)

## 4. Agent Fabric (orchestration) → `docs/scope/04-agent-fabric.md`

- [ ] 10 parallel agent lanes + 1 orchestrator in one workspace
- [ ] Machine-readable hardware introspection: PCI enumeration, register maps,
      device info as structured data (not PDFs)
- [x] Compiler diagnostics as JSON → direct agent feedback loop
      (generate → compile → read errors → fix)
      <!-- breakdown feedback-loop group 3/3 green 2026-07-19:
      scripts/cargo-json-diag.ps1 (raios.cargo_diag.v0, negative proven) +
      agent-drivable harness + gated structured reports. -->

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

- [x] This scope document is the single source for "what raiOS is"
      <!-- breakdown group "Single source" green 2026-07-19: single-source
      rule + breakdown-consistency rule 12 (red paths self-tested, 488f2df). -->
- [x] Docs structure: `SCOPE.md` + `scope/` (breakdowns), `architecture/`
      (+ `decisions/`), `agents/`, `plans/`, `status/`, `assets/`, `_archive/` — nothing else
      <!-- breakdown group "Structure" green: check-docs-hygiene rule 1 +
      root-instructions rule, negatives self-tested (2259b95). -->
- [x] Every architecture decision is an ADR (incl. the seL4 decision, dated)
      <!-- breakdown group "Decisions & history" green: adr-form rule gapless
      numbering + dissent demonstrated (ADR 0020), red paths (abe403c). -->
- [x] Outdated plans are archived (`docs/_archive/`), never silently deleted
      <!-- archive-dated rule with red path (abe403c); no-silent-delete
      guarded by single-writer git history (ADR 0019). -->

---

## Deliberate non-goals

- **No formal proof.** Substitute: predicates + negative tests (§3). Documented trade.
- **No POSIX, no Linux compatibility.** raiOS is not a Unix.
- **No multi-user desktop OS.** One machine, one owner, many domains.
- **No legacy hardware.** Reference hardware + QEMU only.

---

*Definition of done per checkbox: the capability exists + at least one predicate
proves it + one negative test proves its boundary.*
