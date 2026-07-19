# 01 — Rust Kernel (serial core)

> Breakdown of `docs/SCOPE.md` §1. A top-level box there gets checked only when
> its group here is fully green. Serial-core work: max 2 lanes, conservative mode.
> Reframed per ADR 0005/0015 with owner approval on 2026-07-19; no box was
> checked by the reframe.

## Boot & runtime
- [ ] UEFI → kernel handoff reaches stable idle on Surface and in QEMU
- [ ] Panic handler emits a machine-readable crash frame (RECLOG) before halt
- [ ] Early serial/debug channel live before higher initialization
      (first-instruction visibility)
- [ ] Boot time and memory footprint recorded per build (trend visible in reports)
- [ ] Negative tests: induced panic and early-init failure still emit the
      expected RECLOG/diagnostic; missing boot measurements fail the report

## Wasm guest isolation
- [ ] Agent-built code executes as Wasm guests with bounds-checked linear memory
      and per-instance capability imports; guests share no address space
- [ ] A guest cannot name kernel memory or another guest's linear memory
- [ ] Guest memory faults trap, stop/report only that guest, and leave the kernel
      and peer guests running
- [x] Negative tests: OOB read, OOB write, and overflowed effective address →
      trap + logged, with zero host or peer-guest effect — isolation.selftest
      (shadow-20260719-084519, quick 502/502): three hostile guests
      (oob_store/oob_load/oob_offset) each trap MemoryOutOfBounds, the write
      never lands, the host guard is unchanged, host_exposed=0, logged=1;
      permanent quick needle. Verified 2026-07-19.

## Wasm-runtime scheduling
- [ ] Every guest execution is fuel-metered; exhausted fuel stops/traps that
      guest instead of wedging the kernel
- [ ] Fair per-guest fuel budgets prevent one guest from consuming every service turn
- [ ] F12 can kill/suspend a running guest and return control to the core
- [ ] Kernel watchdog integration: hung kernel/runtime state triggers the
      hardware watchdog
- [ ] Negative test: hostile busy-loop guest exhausts its budget while the
      system and a peer guest stay responsive, and F12 remains effective

## Wasm host-import interface
- [ ] Minimal host surface, every import documented; versioned ABI (`v0`,
      additive evolution)
- [ ] Invalid/malformed import or call → typed denial/trap + log, never undefined
      behavior or partial host effect
- [ ] Import/capability catalog exported as structured data (agents read it,
      not kernel headers)
- [ ] Negative tests: missing imports, wrong signatures, bad guest-memory
      offsets/lengths, bad handles, and out-of-range indices fail closed
      <!-- evidence (default-deny import boundary):
      release/vm-reports/shadow-20260714-114527-24812.json,
      predicates m11-import-grant:unauthorized-import-refused and
      m11-import-grant:forbidden-import-link-failure-preserved; orchestrator
      verification required. -->

## In-kernel drivers & hardware authority
- [ ] Drivers remain native and in-kernel by deliberate ADR 0005 decision;
      guests receive service capabilities, not direct device ownership
- [ ] The kernel owns interrupt routing and hardware access; no guest can reach
      a device without the corresponding typed host import
- [ ] IRQ storm from one device cannot lock the system (rate limit / mask + report)
- [ ] Negative test: a guest without the device/service import is denied +
      logged and causes no device or host-state change

## Explicit future hardware hardening (not current checkboxes)

- Enable VT-d translation and require IOMMU isolation for every DMA-capable
  device. The existing `iommu_vtd.rs` probe is structure-only; translation is
  not enabled.
- Predicate: reports prove translation active with no identity-mapped escape
  hatch. Negative test: foreign/out-of-range DMA is blocked + logged with zero
  host or peer effect, including on the reference Surface.

## unsafe inventory
- [ ] Every `unsafe` block tagged with reason + invariant it relies on
- [ ] Inventory generated from source, count tracked per build
- [ ] Each inventoried site covered by at least one predicate exercising it

## RECLOG serial/debug
- [ ] Serial/debug output is emitted as parseable RECLOG frames
- [ ] Negative test: malformed/truncated output is rejected by the parser and
      fails the report rather than being accepted as evidence
