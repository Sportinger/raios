# 07 — Docs & Project Hygiene

> Breakdown of `docs/SCOPE.md` §7. Docs here are agent infrastructure, not
> literature: short is performance, structure is navigation, history writes
> itself (git + reports).

## Single source
- [x] `docs/SCOPE.md` is the only definition of "what raiOS is"; conflicts
      resolve in its favor — README states it explicitly and the
      `single-source` rule asserts the phrase (red path self-tested,
      2259b95, 2026-07-18)
- [ ] Breakdown files (`docs/scope/0N-*.md`) detail but never contradict it;
      top-level list changes need owner approval, breakdowns evolve by
      orchestrator commit

## Structure (the whole map, nothing else)
- [x] `docs/`: `SCOPE.md`, `README.md`, `scope/`, `architecture/`
      (+ `decisions/`), `agents/`, `plans/`, `status/`, `assets/`, `_archive/`
      — enforced by `scripts/check-docs-hygiene.ps1` rule 1 (green 2026-07-18,
      negative via `-SelfTest` planted foreign file)
- [x] Root instruction files consistent with docs paths (CLAUDE.md, AGENTS.md,
      `.claude/` skill) — `root-instructions` rule fails on any referenced
      `docs/` path that does not exist (red path self-tested with a planted
      bogus reference, 2259b95, 2026-07-18)
- [x] No loose files in `docs/` root except `SCOPE.md` and `README.md`
      (HANDOFF lives in `status/` — layout decided 2026-07-18); enforced by
      the same rule 1 + self-test

## Living state
- [x] HANDOFF ~2 KB (displacement rule) — predicate warns > 2560 B, fails
      > 4096 B (`check-docs-hygiene.ps1` rule 2; negative via `-SelfTest`
      planted 5000-byte HANDOFF)
- [x] STATUS ≤ ~30 KB, current state only; no diaries, no done-lists —
      size rule red path self-tested with a planted oversized STATUS
      (2259b95, 2026-07-18); "state only" remains review discipline at
      every overwrite
- [x] Plans: at most one active plan file per scope category in `docs/plans/`
      — `plan-category` rule maps every `plan-<slug>.md` onto exactly one
      `docs/scope/NN-<slug>.md` and rejects duplicates and orphans (red path
      self-tested, 2259b95, 2026-07-18)

## Decisions & history
- [x] Every architecture decision is an ADR (numbered, dated) — seL4 = 0015,
      branchless-main = 0019; `adr-form` rule enforces gapless numbering +
      machine-readable Date:/Status: with red paths (abe403c, 2026-07-18);
      "every decision gets one" stays CLAUDE.md step-7 discipline
- [ ] Second-opinion dissent recorded in the ADR (both positions)
- [x] Outdated material goes to `docs/_archive/` date-prefixed — `archive-dated`
      rule enforces the prefix with a red path (abe403c, 2026-07-18);
      no-silent-delete/no-retro-edit is guarded by single-writer git history
      (ADR 0019)
