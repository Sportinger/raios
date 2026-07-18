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
- [ ] rustc with Cranelift backend runs inside raiOS (self-compilation)
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
