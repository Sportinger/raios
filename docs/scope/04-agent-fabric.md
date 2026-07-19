# 04 — Agent Fabric (orchestration)

> Breakdown of `docs/SCOPE.md` §4. This category is the meta-machine: it doesn't
> ship in the OS image, it is what BUILDS the OS. Its DoD is measured in loop
> throughput and unattended nights survived.

## Lanes & orchestration
- [x] 10 lanes + 1 orchestrator, one workspace, main-only (per CLAUDE.md/AGENTS.md)
      <!-- written: CLAUDE.md loop (up to 10 parallel lanes, 1 orchestrator,
      "Everyone works on main, one worktree, no branches") + AGENTS.md:3.
      MECHANICALLY enforced: single git writer (ADR 0019) — workers dispatch
      -s workspace-write and never git add/commit/push (AGENTS.md:19-21);
      .claude/settings.json enforce-bg-dispatch hook blocks foreground
      dispatch. Negative boundary: a lane touching a file outside its order
      is "absolutely taboo" (AGENTS.md:22-23) and cannot reach main at all
      (no git access). This session's git history is the running proof. -->
- [x] Lane rules written and enforced: serial core max 2 lanes, disjoint file sets
      <!-- CLAUDE.md loop: "Serial core (MMU/scheduler/syscalls): max 2 lanes;
      rest parallel up to 10 … isolation = disjoint file sets, so verify no
      two live orders share a file"; AGENTS.md:22 "Your order's file set IS
      your isolation." Enforced by the orchestrator's pre-dispatch file-set
      disjointness check (done every dispatch this session) + single-writer
      staging (git add <files>, never -A). -->
- [x] Exclusive-lane mode for repo-wide mechanical changes
      <!-- CLAUDE.md loop: "Repo-wide mechanical changes run as an exclusive
      lane (all others paused until gates are green)." Exercised this session:
      the ADR-0023 78-site func_wrap→gate migration was explicitly scoped and
      DEFERRED as an exclusive lane rather than run alongside parallel lanes. -->

## Machine-readable introspection
- [x] PCI enumeration, device IDs, BARs, IRQs exported as structured data
      <!-- device.graph pci rows + pci_functions walk (734e612): BDF, vendor/
      device ids, class triple, interrupt line/pin, typed BARs with size.
      Evidence shadow-20260719-151342-1916.json (504/504): xHCI positive with
      IRQ 11/pin 1 + BAR64; negative = absent wifi AND e1000 must carry
      pci:null — fabricated PCI data fails (b6d4681). -->
- [ ] Register maps for owned devices as data files (agents never parse PDFs)
- [ ] Hardware manifest per machine: CPU features, memory topology, devices —
      the curated-context source for lane system prompts

## Feedback loop
- [x] Compiler diagnostics consumed as JSON in the lane cycle —
      `scripts/cargo-json-diag.ps1` (raios.cargo_diag.v0: level/message/code/
      file/line/column/rendered, written on success AND failure, hijack-safe
      env). Negative proven: planted fixture error → exit 1, structured E0308
      @ src\lib.rs:3:24 (orchestrator-rerun 2026-07-19). Honest limit: host
      workspace crates; the kernel's custom-target build still reports via
      the harness. Verified 2026-07-19.
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
