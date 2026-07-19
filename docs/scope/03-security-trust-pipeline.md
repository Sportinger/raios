# 03 — Security & Trust Pipeline

> Breakdown of `docs/SCOPE.md` §3. Day-1 items are the measuring device for the
> foundation — the substitute for the formal proof. Distribution items start
> only when strangers run the system.

## Day 1 — escape negative tests (the foundation of the foundation)
- [ ] Domain → foreign domain memory (read and write) → denied + logged
- [ ] Domain → kernel memory (read and write) → denied + logged
- [ ] Domain → foreign DMA region → blocked by IOMMU + logged
- [ ] All three run in QEMU AND on bare metal (IOMMU behavior differs on real silicon)
- [ ] All three run on every kernel-touching change (cheap enough to be non-optional)
- [ ] Isolation-suspicion protocol: any unexplained cross-domain effect halts
      all lanes until these tests settle it (mirrors CLAUDE.md full brake)

## Day 1 — rollback
- [x] Every domain version is kept and restorable — m6d-rollback
      (shadow-20260715-121316-27156, 271/271): two consumable approvals, exact
      inventory restore, linked unpromote, tombstone honored; survives three
      boots (persistence-reboot 198/198). Scope caveat: proven for the
      Wasm-service domain model (ADR 0005), the architecture actually built.
- [ ] Rollback of one domain never touches other domains' state
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
