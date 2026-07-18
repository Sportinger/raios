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
- [ ] Engine cage carries threads: shared memory + atomics + wasi thread-spawn
      in the vendored wasmi, deterministic round-robin (host-testable, T1/T2)
- [ ] Bauplatz guest class: hundreds-of-MB linear memory from a memmap-backed
      kernel heap; patience budgets; QEMU + Surface profiles
- [ ] WASI preview1 subset shim behind the import-grant gate (fd/path, args/env,
      clock, random, proc_exit — deterministic, double-build stays byte-equal)
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
