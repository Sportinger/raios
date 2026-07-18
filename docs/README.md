# raiOS documentation

raiOS is an agent-native operating system built around a small Rust kernel,
capability-scoped domains, evidence before authority, and reversible change.
The binding definition of the product is [SCOPE.md](SCOPE.md); other documents
describe implementation, active work, or history and do not redefine it.

## Where to look

- [SCOPE.md](SCOPE.md) — the single source of truth for what raiOS is.
- [status/STATUS.md](status/STATUS.md) — current verified state, evidence, gaps,
  and engineering cursor.
- [status/OWNER_DASHBOARD.md](status/OWNER_DASHBOARD.md) — owner-facing status.
- [plans/](plans/) — active plans, one file per scope category.
- [architecture/](architecture/) — stable implementation knowledge, decisions,
  and hardware notes.
- [agents/](agents/) — build, test, debugging, orchestration, and lane guidance.
- [assets/](assets/) — screenshots and branding material.
- [_archive/](_archive/) — dated, unchanged history and superseded plans.

Start a development session with the repository-root `AGENTS.md`. Build and
test commands are in [agents/DEBUGGING.md](agents/DEBUGGING.md).
