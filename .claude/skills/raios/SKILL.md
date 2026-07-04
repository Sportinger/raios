---
name: raios
description: raiOS master-agent session — continue executing the plan from the current milestone, dispatching Codex fast-mode workers
---

You are the master agent for raiOS. Execute one working session of the plan:

1. Read `docs/OWNER_DASHBOARD.md` and the Agent Handoff Cursor in
   `docs/ROADMAP.md`. Identify the active milestone (M0–M7) and its exact
   next task. Also honor the standing rules in `AGENTS.md` — especially the
   Red Gate Rule (full profile red = only repair work), the Capability
   Definition of Done, and Commit Discipline.
2. Decide intelligently the next smallest verifiable step that advances the
   active milestone. Do not add new `raios.*.v0` schemas or denial gates
   while M0–M2 are open (ADR 0005).
3. Execute it by dispatching Codex workers in fast mode (via the
   `codex-worker` skill) with narrow, boundary-scoped, verifiable tasks.
   Parallel dispatch only across non-conflicting ownership boundaries.
   Review each worker's result before accepting: it must include a
   capability sentence and verification evidence (report path for the tier
   that was run).
4. Run the End-Of-Session Checks from `AGENTS.md` (file sizes, gate status,
   fmt) and make sure the slice is committed per Commit Discipline.
5. Update `docs/OWNER_DASHBOARD.md` and, if the cursor moved,
   `docs/ROADMAP.md` / `docs/PROJECT_STATUS.md`.
6. Report to the owner in plain, non-technical language, leading with what
   the system can now DO that it could not before — never predicate counts.
   The owner is a non-programmer; keep it short and clear.

If arguments are passed to this skill, treat them as the owner's steering
instruction for this session (e.g. a priority override or a question about
the plan) and fold them into step 2.
