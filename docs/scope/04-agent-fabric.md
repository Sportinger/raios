# 04 — Agent Fabric (orchestration)

> Breakdown of `docs/SCOPE.md` §4. This category is the meta-machine: it doesn't
> ship in the OS image, it is what BUILDS the OS. Its DoD is measured in loop
> throughput and unattended nights survived.

## Lanes & orchestration
- [ ] 10 lanes + 1 orchestrator, one workspace, main-only (per CLAUDE.md/AGENTS.md)
- [ ] Lane rules written and enforced: serial core max 2 lanes, disjoint file sets
- [ ] Exclusive-lane mode for repo-wide mechanical changes

## Machine-readable introspection
- [ ] PCI enumeration, device IDs, BARs, IRQs exported as structured data
- [ ] Register maps for owned devices as data files (agents never parse PDFs)
- [ ] Hardware manifest per machine: CPU features, memory topology, devices —
      the curated-context source for lane system prompts

## Feedback loop
- [ ] Compiler diagnostics consumed as JSON in the lane cycle
- [x] Test harness (QEMU profiles + bare metal) startable and readable by
      agents (W6) — `vm-harness/shadow-vm-smoke.ps1` + ~45 profiles emit
      machine-readable JSON; agent-driven hundreds of times this session.
      Bare-metal packaging path exists (`run-stage0-baremetal-vm.ps1`, Surface
      boots) but is the experimental half; QEMU is the proven daily driver.
- [x] Predicate results land as structured reports the orchestrator can gate
      on — every run writes `release/vm-reports/shadow-*.json` with
      per-predicate pass/fail; the loop gates every closure on a named id
      (427 reports on disk).

## Unattended bare-metal loop
- [ ] Log domain: minimal, early-boot, streams RECLOG frames via UDP; survives
      the crash of any other domain
- [ ] Ramoops region: reserved RAM survives warm reboot; pre-network crash logs
      read back and forwarded after reboot
- [ ] Remote power-cycle (smart plug/relay) driven by the orchestrator
- [ ] Hardware watchdog armed by the kernel; hang → automatic cycle
- [ ] Full circle proven: deploy → boot → stream → forced hang → auto-cycle →
      crash log recovered → next attempt, zero human touches

## Doc discipline (agent-facing)
- [x] HANDOFF ~2 KB with displacement rule; size limit enforced by predicate —
      `check-docs-hygiene.ps1` rule 2 (warn >2560 B, fail >4096 B) with a
      `-SelfTest` planting an oversized HANDOFF (red path proven, 2259b95).
- [ ] Every agent reads HANDOFF at session start, overwrites its block at end
      — mandated by CLAUDE.md/AGENTS.md and demonstrated every session, but a
      process invariant, not a mechanical predicate (stays open honestly).
- [x] STATUS ≤ ~30 KB, state only — history lives in git + reports — size
      predicate is `check-docs-hygiene.ps1` rule 3 (fail >30720 B) with a
      self-tested red path (2259b95); "state only" remains review discipline.
