# 0019 — All work lands on branchless main through a single git writer

Date: 2026-07-18 · Status: active

## Context

Ten parallel lanes plus one orchestrator share one repository. The
conventional answer (feature branches + merges) was never adopted here, but
the decision existed only as practice inside CLAUDE.md/AGENTS.md, not as a
recorded architecture decision — flagged by scope §7 ("every architecture
decision is an ADR, including the branchless-main convention").

## Decision

- Everyone works on `main` in one working tree. There are no feature
  branches, no merge commits, no rebases of published history.
- Isolation is not branch isolation but **disjoint file sets**: the
  orchestrator allocates files per lane order and must verify no two live
  orders share a file before dispatch. Repo-wide mechanical changes run as
  an exclusive lane with everything else paused.
- The **orchestrator is the only git writer**. Lanes propose commit
  messages; the orchestrator reviews gates, stages exactly the order's file
  set (`git add <files>`, never `-A`), commits, and pushes immediately —
  uncommitted work counts as unsaved work.
- Repair happens forward only: `git revert`, never `git reset` on published
  history. The remote `main` is the only copy assumed to survive a machine
  failure.

## Alternatives & second opinions

- **Feature branches per lane:** rejected. The lanes are short-lived and
  file-disjoint by construction; branch/merge machinery adds conflict
  resolution work and hides integration risk until merge time, while the
  actual risk this project manages (two agents touching one file) is solved
  by allocation, not by merging.
- **Multiple worktrees per lane:** rejected for routine work — same
  isolation already guaranteed by file-set allocation; worktrees would slow
  the collect-immediately cycle. (Throwaway read-only worktrees for
  verification remain permissible; they never publish.)
- No advisor dissent to record: this codifies long-standing proven practice
  (the whole commit history is single-writer, branchless).

## Consequences

- Easier: instant collection and push after every lane; linear, auditable
  history where commits ARE the project log; no merge conflicts by design.
- Harder: the orchestrator is a serialization point and must be disciplined
  about file-set disjointness; a mistake there surfaces as overlapping edits
  in the working tree instead of a merge conflict.
- The trade is deliberate: allocation discipline over merge machinery.
