# 02 — Genesis Layer (capability floor)

> Breakdown of `docs/SCOPE.md` §2. The floor is THE product decision: narrow,
> documented, kernel-agnostic. Everything above it is replaceable; the floor
> contract is not casually changeable (changes = ADR + owner).

## Primitives
- [ ] `create_domain` — new empty domain, zero capabilities by default
- [ ] `grant_capability` / `revoke_capability` — explicit, typed, logged
- [ ] `kill_domain` — immediate, reclaims all resources, cannot be blocked by the domain
- [ ] Negative tests: a domain cannot invoke any primitive on another domain

## Capability granularity
- [ ] One PCIe BAR, one IRQ line, one DMA region, one framebuffer region —
      each an individual, revocable grant
- [ ] Grants are typed records (who, what, scope), not flags
- [ ] Revocation takes effect immediately; in-flight DMA fenced via IOMMU
- [ ] Negative test: capability for device A grants nothing on device B

## Storage primitive
- [ ] Persistent block access as a capability (range-scoped, not whole-disk)
- [ ] A domain without a storage grant cannot persist anything
- [ ] Negative test: write outside the granted block range → denied + logged

## Domain lifecycle
- [ ] Kill + restart of any domain in < 1 s, without system reboot
- [ ] Restart restores a declared clean state (no leaked grants from the previous life)
- [ ] Crash loop detection: N rapid crashes → domain parked + reported, not respawned forever

## Floor contract
- [ ] The full floor interface fits in one document (`docs/architecture/genesis-layer.md`)
- [ ] No kernel-internal types leak through the interface
- [ ] seL4 substitutability argued in writing: every primitive mapped to a
      plausible seL4 realization (paper exercise, kept current)
