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
- [ ] Test harness (QEMU profiles + bare metal) startable and readable by agents (W6)
- [ ] Predicate results land as structured reports the orchestrator can gate on

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
- [ ] HANDOFF ~2 KB with displacement rule; size limit enforced by predicate
- [ ] Every agent reads HANDOFF at session start, overwrites its block at end
- [ ] STATUS ≤ ~30 KB, state only — history lives in git + reports
