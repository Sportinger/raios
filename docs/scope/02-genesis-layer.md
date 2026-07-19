# 02 — Genesis Layer (capability floor)

> Breakdown of `docs/SCOPE.md` §2. The floor is THE product decision: narrow,
> documented, kernel-agnostic. Everything above it is replaceable; the floor
> contract is not casually changeable (changes = ADR + owner).
> Reframed per ADR 0005/0015 with owner approval on 2026-07-19; no box was
> checked by the reframe.

## Service primitives
- [ ] Create a fresh Wasm service/guest with zero capability imports by default
- [ ] Grant/revoke a typed capability import explicitly, with each transition logged
- [x] Kill a service immediately, reclaim its guest resources, and prevent the
      guest from blocking teardown — m11-beyond-env-lifecycle
      (shadow-20260714-123624, 183/183): F12 kill of a running/spinning guest
      with killed_cleanup_guest_bound (resources reclaimed) and
      terminal_matrix_exactly_once (teardown once, unblockable). Verified
      2026-07-19.
- [ ] Negative tests: a guest cannot manage another service, and a zero-grant
      guest's host call is denied + logged with zero host effect

## Capability granularity
- [ ] Each host import/service surface is an individual, revocable grant; the
      kernel retains direct hardware authority
- [ ] Grants are typed records (who, what, import/service scope), not ambient flags
- [ ] Revocation prevents the next host call and is durably logged; no stale
      instance retains the revoked authority
- [ ] Negative test: a grant for import/service A grants nothing on host surface B
      <!-- evidence (default-deny/import-scope boundary only):
      release/vm-reports/shadow-20260714-114527-24812.json, passed
      m11-wasm-import-grant profile, including unauthorized import refusal
      before instantiation; grant/revoke verification still required. -->

## Storage primitive
- [ ] Persistent ARTSTOR/structured-store access is a range/quota-scoped
      capability, never ambient whole-store authority
- [x] A guest without a storage capability cannot persist anything
      <!-- guests have no storage import at all; the only persistent egress
      is the unforgeable BuildStorageAuthority handle, and Absent denies
      first: storage_capability_absent before lock/evaluator/controller
      (d18bcc0). Evidence shadow-20260719-154053-30128 (507/507). -->
- [x] Negative test: absent grant, out-of-range write, or quota overflow →
      denied + logged with no partial persistent effect
      <!-- storage.selftest live: absent_grant=storage_capability_absent,
      out_of_range=output_span_out_of_artstor, quota_overflow=
      output_span_length_exceeds_lease, ram_quota=nospc, serial-logged,
      persistent_effect=0 + full RECLOG/ARTSTOR SHA-256 equality
      (disk=pass reclog_unchanged=1 artstor_unchanged=1). Scope honesty:
      proves the build-output egress boundary (the guest-facing path);
      older internal post-write-evaluator orderings are a separate open
      hardening item (scout 2026-07-19). -->

## Service lifecycle
- [ ] Kill + restart of any Wasm service in < 1 s, without system reboot
- [ ] Restart restores a declared clean state (no leaked imports, handles, or
      mutable guest state from the previous life)
- [ ] Crash loop detection: N rapid crashes → service parked + reported, not
      respawned forever
- [x] Negative test: after kill/restart, the old instance cannot run or write,
      and an ungranted authority from its prior life remains denied —
      m11-beyond-env-lifecycle (shadow-20260714-123624, 183/183):
      second_run_after_kill starts a fresh instance while the killed one is
      gone (exactly-once terminal), and the beyond-env import stays denied.
      Verified 2026-07-19.

## Floor contract
- [x] The full Wasm import + service-capability floor fits in one document
      (`docs/architecture/genesis-layer.md`) — 242 lines, written 2026-07-19
      from code with file:line citations (5 env.* imports, the frozen 30-import
      build surface incl. digest 4145184d…, grant/lifecycle authority,
      non-guarantees). Orchestrator spot-checked 7 citations incl. the exact
      digest. Verified 2026-07-19.
- [ ] No kernel-internal types leak through the import/service interface
- [ ] Contract conformance: services depend only on the documented Wasm import
      + service-capability floor; a fixture that depends on a kernel-internal
      type or undeclared import is rejected

ADR 0015 chooses the custom Rust kernel as the development and product path.
Substitutability attaches to this narrow contract; maintaining a fictional
primitive-by-primitive seL4 mapping is not a current requirement. The floor
document exists since 2026-07-19; the no-internal-types and conformance boxes
above still need their mechanical predicate/test.
