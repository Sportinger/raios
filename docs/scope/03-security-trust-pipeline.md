# 03 — Security & Trust Pipeline

> Breakdown of `docs/SCOPE.md` §3. Day-1 items are the measuring device for the
> foundation — the substitute for the formal proof. Distribution items start
> only when strangers run the system.
> Reframed per ADR 0005/0015 with owner approval on 2026-07-19; no box was
> checked by the reframe.

## Day 1 — escape negative tests (the foundation of the foundation)
- [x] Wasm guest OOB linear-memory read and write → trap + logged, with zero
      host or peer-guest effect — isolation.selftest (shadow-20260719-084519,
      quick 502/502): OOB store/load/offset each trap MemoryOutOfBounds,
      logged=1, host_exposed=0; permanent quick needle. Verified 2026-07-19.
- [x] Guest requests an ungranted capability import/host surface → denied +
      logged before effect — m11-wasm-import-grant (shadow-20260714-114527,
      159/159): `unauthorized-import-refused` (refused before instantiation)
      and `forbidden-import-link-failure-preserved` (link fails, no host
      effect). Verified 2026-07-19.
- [ ] Both Wasm-boundary tests run in QEMU AND on bare metal
- [x] Both run on every kernel/Wasm-runtime-touching change (cheap enough to
      be non-optional) — both are permanent quick-profile needles since
      2026-07-19: OOB selftest + ungranted-import denial
      (`RAIOS_ISOLATION importdeny=refused logged=1 host_effect=0`), quick
      503/503 shadow-20260719-104006. Negative boundary shown live: the
      first fixture emitted importdeny=failed and the whole report went red
      (shadow-20260719-102907). Verified 2026-07-19.
- [ ] Isolation-suspicion protocol: any unexplained cross-guest or guest-to-host
      effect halts
      all lanes until these tests settle it (mirrors AGENTS.md full brake)

## Explicit future hardware hardening (not current checkboxes)

- Enable VT-d translation and enforce IOMMU isolation for DMA-capable devices;
  the existing `iommu_vtd.rs` probe is structure-only and does not enable
  translation.
- Future predicate: translation is reported active. Negative test: a foreign
  or out-of-range DMA attempt is blocked + logged with zero host/peer effect,
  including on real reference hardware.

## Day 1 — rollback
- [x] Every domain version is kept and restorable — m6d-rollback
      (shadow-20260715-121316-27156, 271/271): two consumable approvals, exact
      inventory restore, linked unpromote, tombstone honored; survives three
      boots (persistence-reboot 198/198). Scope caveat: proven for the
      Wasm-service domain model (ADR 0005), the architecture actually built.
- [x] Rollback of one domain never touches other domains' state —
      rollback-isolation profile (shadow-20260719-183417, 280/280): two
      REAL foreign-family durable records (memory.observation_log_append)
      seeded pre-install, independently re-observed via the bounded
      per-record durable scan (4e72924) — both bit-identical through A's
      full W6-install→rollback cycle; RECLOG advanced by exactly A's
      unpromote; the rollback chain names only A (B-hash presence,
      degenerate identifiers, and count violations each fail closed).
      Chain: 87cd503→53b6a31→530b7b→B8 (see git).
- [ ] Negative test: rollback to a version with fewer grants revokes the delta

## Day 1 — report pipeline (ARTSTOR)
- [x] Every build and every test run emits a structured report — every
      vm-harness profile writes one `release/vm-reports/shadow-*.json`
      (+`.sha256`); 427 on disk. Negative boundary: a failed predicate makes
      the whole report `result=failed` (seen repeatedly this session, e.g.
      the sysimport/rustcrun red runs).
- [x] Reports carry identity: what ran, on what hardware/commit, verdict,
      evidence — each report carries profile, per-predicate expected/actual,
      pass/fail counts, verdict, and a sha256 sidecar; run ids embed the
      timestamp+PID.
- [x] Reports are the checkbox authority: no green report chain → no checked
      box — the orchestrator loop gates every closure on a named report id
      (this file's checks cite them); nothing is checked without one.

## Distribution phase — signed & reproducible
- [ ] Builds signed; signature checked before a module runs
- [ ] Double build (two independent environments, identical hashes) for releases
- [ ] Audit log: every capability grant queryable (who, what, when, why)
- [ ] Trigger to start this phase: first machine not owned by the owner — noted as ADR
