# 01 — Rust Kernel (serial core)

> Breakdown of `docs/SCOPE.md` §1. A top-level box there gets checked only when
> its group here is fully green. Serial-core work: max 2 lanes, conservative mode.

## Boot & runtime
- [ ] UEFI → kernel handoff reaches stable idle on Surface and in QEMU
- [ ] Panic handler emits a machine-readable crash frame (RECLOG) before halt
- [ ] Early serial/debug channel live before MMU init (first-instruction visibility)
- [ ] Boot time and memory footprint recorded per build (trend visible in reports)

## Memory (MMU/paging)
- [ ] Per-domain page tables; no shared writable mappings between domains
- [ ] Kernel mappings never reachable from domain context (SMEP/SMAP-style checks)
- [ ] Page-fault path: domain fault → domain kill/report, never kernel panic
- [ ] Negative tests: cross-domain read, cross-domain write, kernel read/write from domain

## Scheduler
- [ ] Preemptive, timer-driven; a spinning domain cannot starve the system
- [ ] Domain-fair budgets; kill-capable at any point in a domain's execution
- [ ] Kernel watchdog integration: hung scheduler state triggers hardware watchdog
- [ ] Negative test: hostile busy-loop domain — system stays responsive, domain killable

## Syscalls
- [ ] Minimal surface, every call documented; versioned ABI (`v0`, additive evolution)
- [ ] Invalid/malformed syscall → typed error to domain, never undefined behavior
- [ ] Syscall table exported as structured data (agents read it, not headers)
- [ ] Negative tests: bad pointers, bad handles, out-of-range indices from a domain

## Interrupts & DMA
- [ ] Interrupt routing to userspace driver domains; kernel keeps only the trampoline
- [ ] IOMMU on for every DMA-capable device; no identity-mapped escape hatches
- [ ] IRQ storm from one device cannot lock the system (rate limit / mask + report)

## unsafe inventory
- [ ] Every `unsafe` block tagged with reason + invariant it relies on
- [ ] Inventory generated from source, count tracked per build
- [ ] Each inventoried site covered by at least one predicate exercising it
