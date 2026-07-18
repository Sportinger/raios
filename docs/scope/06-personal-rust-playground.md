# 06 — Personal Rust Playground

> Breakdown of `docs/SCOPE.md` §6. The playground is the product promise made
> personal: anyone (human or agent) can build against real hardware without
> being able to hurt the system. It is also the proving ground for the floor —
> if the playground is safe, the floor works.

## Requesting & isolation
- [ ] One command/request → fresh empty domain, zero grants beyond the asked-for set
- [ ] Playground domains are marked as such; they can never receive
      security-critical grants (kernel regions, foreign DMA) — denied by policy
- [ ] Negative test: hostile playground code exhausts CPU/memory → its domain
      throttled/killed, system and other domains unaffected

## Toolchain in the OS
> Route B (owner 2026-07-18, binding): the public `wasm32-wasip1-threads`
> rustc artifact runs UNMODIFIED inside the caged interpreter (green threads,
> T1/T2). A fast execution stage (AOT) is a later, deliberate ADR — the
> top-level SCOPE.md still says "Cranelift backend"; flagged to owner.
- [x] T1 — engine threads instruction surface: shared memories, all 64 atomic
      operators, wait/notify as typed suspensions, opt-in fuel-quantum yield in
      the vendored wasmi. Negative boundary: default config rejects shared
      memories and stays byte-identical; non-resumable calls hitting a
      suspension end in `AtomicSuspendNotResumable`, never hang. Evidence:
      32-test conformance suite (8a58f5f, 863ea4d, 323f974, fab0dab, 2c59257,
      9137872).
- [x] T2 — deterministic green-thread scheduling end to end: policy
      host-proven with replay-equal traces (97d163c); kernel pump runs a
      fixed-thread job in QEMU with double-run trace equality; wasi
      thread-spawn (deferred materialization), proc_exit whole-job end and
      cap-48 denial live (8321953). Negative boundary: futex deadlock →
      deterministic JobDeadlocked, digest-equal double run, bounded rounds —
      never an endless pump (3f2a64a, QEMU quick 499/499
      shadow-20260718-214754-2760).
- [ ] Bauplatz guest memories: the 1-GiB shared-memory window (399/16384
      pages) instantiable in the kernel with QEMU and Surface RAM profiles.
      Negative boundary: a job exceeding its class limits is denied before
      instantiation.
- [x] Bauplatz substrate contracts: memmap-backed kernel heap (291 MiB in the
      512M VM; negative: too-small memmap → proven static fallback with boot
      continuing, b93a743) and the canonically hashed BuildGuestClassV1 limits
      (negative: 11 typed validation rejections; cross-checked against
      inventory, scheduler and shim constants, e51dc2a).
- [x] WASI measured import contract: the 30-import surface is measured
      (b3c2df4) and frozen behind `raios.wasi_build_imports.v1`. Negative
      boundary: extra/missing/reordered import or signature drift → typed
      denial, never a partial grant (2f29e96).
- [x] WASI file world: chunk-CAS read-only `/sysroot`+`/src`, quota-atomic RAM
      arenas, `/out` freeze, double-build egress gate. Negative boundary: ROFS
      on read-only mounts, atomic quota rejections, unshadowable reserved
      names, XDEV across arenas, one differing output byte → no egress plan
      (2698a70, f8f5804).
- [x] WASI process world: manifest-bound args/env, fuel-derived clock, seeded
      pinned PRNG, typed proc_exit/yield. Negative boundary: post-exit calls
      fail closed with the original code; PRNG errors are transactional; fd
      poll subscriptions reject whole-call (83c8631).
- [ ] WASI kernel glue: the shim linked behind the grant gate with validated
      guest pointers and store adapters (slice 6). Negative boundary: a module
      with any undeclared import never instantiates.
- [ ] rustc-as-Wasm compiles a real program inside raiOS (W5-proven, slow is fine)
- [ ] Compile diagnostics available as JSON (same feedback loop as the fabric)
- [ ] Build artifacts land only in the domain's own granted storage range

## Templates ("Hello Hardware")
- [ ] Minimal template: serial-out + exactly one capability, builds and runs
      in under a minute from request
- [ ] Templates document their grant set in-file (readable contract)

## Crash behavior
- [ ] Crash = RECLOG entry + restart offer, nothing else — no system effect
- [ ] Crash loop → domain parked with report (same rule as Genesis lifecycle)

## Promotion path
- [ ] A playground result can be promoted to a "real" domain only through the
      trust pipeline (§3): predicates + negative tests + report chain
- [ ] Negative test: promotion attempt without green evidence → denied + logged
