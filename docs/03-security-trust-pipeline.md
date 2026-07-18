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
- [ ] Every domain version is kept and restorable
- [ ] Rollback of one domain never touches other domains' state
- [ ] Negative test: rollback to a version with fewer grants revokes the delta

## Day 1 — report pipeline (ARTSTOR)
- [ ] Every build and every test run emits a structured report
- [ ] Reports carry identity: what ran, on what hardware/commit, verdict, evidence
- [ ] Reports are the checkbox authority: no green report chain → no checked box

## Distribution phase — signed & reproducible
- [ ] Builds signed; signature checked before a module runs
- [ ] Double build (two independent environments, identical hashes) for releases
- [ ] Audit log: every capability grant queryable (who, what, when, why)
- [ ] Trigger to start this phase: first machine not owned by the owner — noted as ADR
