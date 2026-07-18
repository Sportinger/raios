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
- [ ] `docs/`: `SCOPE.md`, `scope/`, `architecture/` (+ `decisions/`),
      `agents/`, `plans/`, `status/`, `assets/`, `_archive/`
- [ ] Root instruction files consistent with docs paths (CLAUDE.md, AGENTS.md,
      `.claude/` skill) — one truth, checked after every restructure
- [ ] No loose files in `docs/` root except `SCOPE.md` and `HANDOFF.md`

## Living state
- [ ] HANDOFF ~2 KB (displacement rule) — limit enforced by predicate
- [ ] STATUS ≤ ~30 KB, current state only; no diaries, no done-lists
- [ ] Plans: exactly one active plan file per scope category in `docs/plans/`

## Decisions & history
- [ ] Every architecture decision is an ADR (numbered, dated) — including the
      seL4/custom-kernel decision and the branchless-main convention
- [ ] Second-opinion dissent recorded in the ADR (both positions)
- [ ] Outdated material goes to `docs/_archive/` date-prefixed — never silently
      deleted, never retro-edited
