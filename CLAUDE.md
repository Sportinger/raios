# raiOS Claude Agent Instructions

This repository is the local raiOS workspace. Start with `AGENTS.md`, then read
the project status, roadmap, debugging guide, and architecture decisions before
making changes.

## Required Context

Read at minimum:

1. `README.md`
2. `AGENTS.md`
3. `docs/PROJECT_STATUS.md`
4. `docs/ROADMAP.md`
5. `docs/DEBUGGING.md`
6. `docs/architecture-decisions/0001-raios-agent-protocol.md`
7. `docs/architecture-decisions/0004-system-memory-and-agent-context.md`

Then run:

```powershell
git status --short
```

Preserve unrelated user changes.

## Memory Rule

Build toward the ADR 0004 model: raiOS itself is the memory. Memory is typed
system state with provenance, not a raw chat log, fake persistence layer, or
large prompt dump.

When adding or changing features, prefer durable facts that can later feed the
context broker:

- stable service, problem, capability, event, and evidence IDs
- structured snapshots over prose-only status
- `public`, `local_only`, and `secret` classification before provider export
- explicit `capability_denied` when evidence is missing
- summaries and RAG hits as locators only, never as authority
- RAM-only early memory labeled as `current_boot`

Near-term implementation order remains raiOS first: provider trust and
redaction, typed snapshot/service/problem/capability facts, then read-only
`memory.context`. Do not add fake long-term memory before persistence, audit,
and rollback exist.
