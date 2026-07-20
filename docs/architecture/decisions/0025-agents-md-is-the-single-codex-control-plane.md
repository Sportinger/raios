# 0025 — AGENTS.md is the single Codex control plane

Date: 2026-07-20 · Status: active

## Context

raiOS had three overlapping operational instruction surfaces: `CLAUDE.md`, a
repo-local `.claude/` skill/hook bundle, and `AGENTS.md`. During the overnight
loop, `AGENTS.md` was changed from worker rules into orchestrator rules while
the resumed root session still named `CLAUDE.md` as its loop source. Workers
therefore received ambiguous roles, and provider-specific second-opinion rules
remained active even though that provider was unavailable by owner decision.

The same run demonstrated the operational cost: many correctly dispatched
Codex reviews found real rollback defects, but one checkbox accumulated a large
uncommitted patch because ownership, acceptance, and checkpoint rules were
split across instruction files and a stale resumed session.

## Decision

1. Root `AGENTS.md` is the only live agent-control document in this repository.
   `CLAUDE.md` and the repo-local `.claude/` integration are retired.
2. `AGENTS.md` selects roles explicitly: the root loop session orchestrates;
   a bounded `codex exec` order is always a worker and never commits.
3. Only Codex workers and independent read-only Codex reviews are dispatched.
   No Claude command, agent, hook, skill, MCP, or second opinion is part of the
   raiOS loop.
4. The orchestrator remains the sole git writer and immediately commits and
   pushes each accepted, predicate-covered slice before expanding it.
5. A 90-minute / 800-added-line / five-owned-file checkpoint forces
   verification-and-secure or rescoping. It never authorizes committing red
   work.
6. A loop restart is a fresh Codex session. It must not resume the retired
   transcript, because resume preserves stale prompt and session state.

## Alternatives

- Keep `CLAUDE.md` as a compatibility shim: rejected because it would remain a
  second live instruction surface and could drift again.
- Put orchestration only in the start prompt: rejected because repo-durable
  rules belong in automatically loaded `AGENTS.md` and must also constrain
  worker sessions.
- Let workers commit their own output: rejected by ADR 0019's single-writer
  invariant and the shared-main worktree model.

## Consequences

Fresh Codex roots and workers load the same rules but select different roles
deterministically. Historical ADRs and archives may still mention earlier
tools; they remain immutable history, not live instructions. The docs-hygiene
predicate now requires `AGENTS.md` and rejects reintroduction of `CLAUDE.md` or
`.claude/` as operational control surfaces.
