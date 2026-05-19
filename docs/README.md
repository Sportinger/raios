# Documentation

Start with the root `README.md`, then:

- `PROJECT_STATUS.md` - current verified boot state and exact next task.
- `ROADMAP.md` - overall phase plan.
- `DEBUGGING.md` - build, run, test, and boot debugging notes.
- `architecture-decisions/` - durable architecture decisions.
  - `0004-system-memory-and-agent-context.md` defines the rule that raiOS
    itself is the memory and agents receive bounded context packets, not raw
    memory dumps.
- `sections/` - older section reports from the original runbook.
