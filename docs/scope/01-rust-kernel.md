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
- [x] Every guest execution is fuel-metered; exhausted fuel stops/traps that
      guest instead of wedging the kernel
      <!-- fuel enabled across all execution worlds (envelope.rs:578/194,
      personal_shell.rs:310, invocation.rs:1113/425, wasi_build_job.rs:755/
      1060). m11-beyond-env-lifecycle HEAD re-run shadow-20260719-195334
      (183/183): max_fuel_busy_loop_bound (hostile 1e6-fuel loop → OutOfFuel
      ≤250ms) + m8_wedge_out_of_fuel_crashed (real echo traps, loop stays
      usable). Cross-guest FAIR budgets are the separate open box below. -->
- [ ] Fair per-guest fuel budgets prevent one guest from consuming every service turn
- [x] F12 can kill/suspend a running guest and return control to the core
      <!-- F12 is core-only secure attention advancing the kill generation
      (input.rs:293); the invocation checks it at each pump boundary
      (beyond_env_invocation.rs:201); main loop keeps polling+pumping
      (main.rs:408). m11-beyond-env-lifecycle HEAD re-run
      shadow-20260719-195334 (183/183): physical_f12_monitor_path,
      f12_host_bound, killed_cleanup_guest_bound (resources reclaimed),
      second_run_after_kill (control returns, fresh run). -->
- [ ] Kernel watchdog integration: hung kernel/runtime state triggers the
      hardware watchdog
- [ ] Negative test: hostile busy-loop guest exhausts its budget while the
      system and a peer guest stay responsive, and F12 remains effective

## Wasm host-import interface
- [ ] Minimal host surface, every import documented; versioned ABI (`v0`,
      additive evolution)
- [x] Invalid/malformed import or call → typed denial/trap + log, never undefined
      behavior or partial host effect
      <!-- host_import.selftest (shadow-20260719-203005, 509/509): six
      malformed classes each fail closed with a DISTINCT typed reason +
      machine-readable denial record, host/peer/persistent/partial effect all
      0, RECLOG+ARTSTOR hashes unchanged (disk=pass). See the negative-matrix
      box below for the case list. -->
- [x] Import/capability catalog exported as structured data (agents read it,
      not kernel headers)
      <!-- structured, not headers: known_host_imports + count exported
      (agent_protocol_honesty.rs:285), capability catalog with
      id/risk/status/scope/summary (agent_protocol_system.rs:16,776).
      Predicates protocol:system_honesty_report_standing_posture (exact
      import list+count) and protocol:capability_catalog_status_observed
      (structured entries, non-authorizing read) green in current-session
      shadow-20260719-192220-32592 (507/507). Signatures/import→cap mapping
      are the separate ABI box (open). -->
- [x] Negative tests: missing imports, wrong signatures, bad guest-memory
      offsets/lengths, bad handles, and out-of-range indices fail closed
      <!-- host_import.selftest (shadow-20260719-203005, 509/509), six cases
      pairwise-distinct reasons: missing=module_import_not_authorized (before
      instantiation), wrong_sig=signature_mismatch (wasmi FuncTypeMismatch on
      a wrong-arity env.log import, before instantiation), bad_offset=trapped
      (env.log negative/overflow ptr, envelope.rs:812), bad_length=denied
      (output_write >4096, envelope.rs:907), bad_handle=denied
      (crypto_foreign_session), bad_index=denied (acquire_chunk_index_
      mismatch); logged=1, all effect surfaces 0, prior state preserved. -->

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
