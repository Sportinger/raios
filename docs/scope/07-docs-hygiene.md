# 07 — Docs & Project Hygiene

> Breakdown of `docs/SCOPE.md` §7. Docs here are agent infrastructure, not
> literature: short is performance, structure is navigation, history writes
> itself (git + reports).

## Single source
- [ ] `docs/SCOPE.md` is the only definition of "what raiOS is"; conflicts
      resolve in its favor (README states this explicitly)
- [ ] Breakdown files (`docs/scope/0N-*.md`) detail but never contradict it;
      top-level list changes need owner approval, breakdowns evolve by
      orchestrator commit

## Structure (the whole map, nothing else)
- [x] `docs/`: `SCOPE.md`, `README.md`, `scope/`, `architecture/`
      (+ `decisions/`), `agents/`, `plans/`, `status/`, `assets/`, `_archive/`
      — enforced by `scripts/check-docs-hygiene.ps1` rule 1 (green 2026-07-18,
      negative via `-SelfTest` planted foreign file)
- [ ] Root instruction files consistent with docs paths (CLAUDE.md, AGENTS.md,
      `.claude/` skill) — one truth, checked after every restructure
      (manually re-aligned 2026-07-18: stale VISION_PLAN-era skill and
      AGENTS.md commit step rewritten; still needs an automated predicate)
- [x] No loose files in `docs/` root except `SCOPE.md` and `README.md`
      (HANDOFF lives in `status/` — layout decided 2026-07-18); enforced by
      the same rule 1 + self-test

## Living state
- [x] HANDOFF ~2 KB (displacement rule) — predicate warns > 2560 B, fails
      > 4096 B (`check-docs-hygiene.ps1` rule 2; negative via `-SelfTest`
      planted 5000-byte HANDOFF)
- [ ] STATUS ≤ ~30 KB, current state only; no diaries, no done-lists
      (size rule exists in the script, but its red path is not yet
      self-tested; "state only" needs review discipline)
- [ ] Plans: exactly one active plan file per scope category in `docs/plans/`
      (script checks `plan-*` naming only; category mapping still manual)

## Decisions & history
- [ ] Every architecture decision is an ADR (numbered, dated) — including the
      seL4/custom-kernel decision and the branchless-main convention
- [ ] Second-opinion dissent recorded in the ADR (both positions)
- [ ] Outdated material goes to `docs/_archive/` date-prefixed — never silently
      deleted, never retro-edited
