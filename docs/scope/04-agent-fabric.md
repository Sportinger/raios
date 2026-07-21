# 04 — Agent Fabric (orchestration)

> Breakdown of `docs/SCOPE.md` §4. This category is the meta-machine: it doesn't
> ship in the OS image, it is what BUILDS the OS. Its DoD is measured in loop
> throughput and unattended nights survived.

## Lanes & orchestration
- [x] 10 lanes + 1 orchestrator, one workspace, main-only (per `AGENTS.md`)
      <!-- ADR 0025 makes AGENTS.md the only live control plane. Its role
      selection keeps the root as orchestrator and every bounded codex exec as
      a non-committing worker; up to ten disjoint workers may run. ADR 0019
      mechanically limits main history to the single orchestrator writer.
      Negative boundary: a worker touching a file outside its exact order is
      rejected at acceptance and cannot be staged by the orchestrator. -->
- [x] Lane rules written and enforced: serial core max 2 lanes, disjoint file sets
      <!-- AGENTS.md requires max two conservative overlapping core/security
      lanes, up to ten other workers, and a pre-dispatch file-set disjointness
      check. Worker file sets are absolute taboos; the sole writer stages exact
      accepted files only (never git add -A). -->
- [x] Exclusive-lane mode for repo-wide mechanical changes
      <!-- Repo-wide work necessarily owns overlapping file sets and therefore
      runs exclusively under AGENTS.md's disjointness rule. Exercised by the
      ADR-0023 78-site func_wrap→gate migration, which was deferred rather
      than run beside conflicting lanes. -->

## Machine-readable introspection
- [x] PCI enumeration, device IDs, BARs, IRQs exported as structured data
      <!-- device.graph pci rows + pci_functions walk (734e612): BDF, vendor/
      device ids, class triple, interrupt line/pin, typed BARs with size.
      Evidence shadow-20260719-151342-1916.json (504/504): xHCI positive with
      IRQ 11/pin 1 + BAR64; negative = absent wifi AND e1000 must carry
      pci:null — fabricated PCI data fails (b6d4681). -->
- [x] Register maps for owned devices as data files (agents never parse PDFs)
      <!-- hardware/register-maps/*.v1.json covers the four owned MMIO paths:
      e1000, AHCI, xHCI, and Marvell 88W8897 (71 source-backed registers and
      constants) under register-map.v1 schema. `check-register-maps.ps1`
      validates all 4 maps; its in-memory mutation selftest rejects 10/10
      malformed boundaries (duplicate/range/width/access/mask/provenance/
      binding/schema). Orchestrator re-run green 2026-07-19; commit 2d5c17f. -->
- [ ] Hardware manifest per machine: CPU features, memory topology, devices —
      the curated-context source for lane system prompts

## Feedback loop
- [x] Compiler diagnostics consumed as JSON in the lane cycle —
      `scripts/cargo-json-diag.ps1` (raios.cargo_diag.v0: level/message/code/
      file/line/column/rendered, written on success AND failure). Negative
      proven: planted fixture error → exit 1, structured E0308 @
      src\lib.rs:3:24, re-verified 2026-07-21 against
      `scripts/experiments/json-diag-fixture` (positive exit 0; negative exit
      1 with exact span; hostile inherited CARGO_HOME/CARGO_TARGET_DIR
      overridden by the script's pins). Honest limits (audit A69): the env
      pin covers only CARGO_HOME and CARGO_TARGET_DIR — RUSTC*/RUSTFLAGS/PATH
      are not locked; no self-test wrapper or CI caller exists yet, so the
      negative-path re-run stays orchestrator discipline. Host workspace
      crates; the kernel's custom-target build still reports via the harness.
      Verified 2026-07-21.
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
      — mandated by AGENTS.md and demonstrated every session, but a
      process invariant, not a mechanical predicate (stays open honestly).
- [x] STATUS ≤ ~30 KB, state only — history lives in git + reports — size
      predicate is `check-docs-hygiene.ps1` rule 3 (fail >30720 B) with a
      self-tested red path (2259b95); "state only" remains review discipline.
