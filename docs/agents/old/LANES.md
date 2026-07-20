# raiOS AI-Parallel Goal Prompt

Use this prompt when starting a fresh OS-wide raiOS builder thread:

```text
Pivot the whole raiOS plan to an AI-parallel OS build.

Keep the full raiOS vision in scope: a tiny permanent core, a protected
recovery lifeline, a live replaceable service graph, an agent workspace,
Shadow VM acceptance, typed system memory, and later persistence/rollback.

Do not build throwaway MVPs, mocks, fake security, fake loaders, fake memory, or
silent fallbacks. Move fast through real vertical slices on the final
architecture path: bootable, observable, capability-gated, fail-closed,
evidence-bound, and VM-tested.

Work in parallel by ownership boundary, not by traditional long sequential
phases. Split agents across core/loader, service registry, capability/audit,
provider trust/context, UI/input, VM harness, and docs/status. Each agent must
return one integrated slice with verification and doc updates, not scaffolding
for someone else to make real later.

The current first runtime wave is: finish the RAM-only Hello service identity
chain, improve provider trust/UI/harness in independent tracks, then consider
signed artifact bytes only after trust, audit, rollback, and recovery evidence
exists.
```

## Operating Rules

- Start every instance with `AGENTS.md`, `README.md`, `docs/SCOPE.md`,
  `docs/status/STATUS.md`, the active files in `docs/plans/`,
  `docs/agents/DEBUGGING.md`, ADR 0001, ADR 0004, and
  `git status --short`.
- Preserve the bootable Stage-0 image and keep unrelated user changes.
- Prefer the smallest real slice that advances the final OS shape.
- Keep missing capabilities explicit as `capability_denied`, blocked, or known
  gaps.
- Use quick VM smoke for focused runtime slices and full VM smoke when touching
  trust, loader, recovery, persistence, or shared protocol behavior.
- Update `docs/status/STATUS.md` and the relevant file in `docs/plans/` when the active cursor
  or verified state changes.

## Parallel Track Queue

Track A, current runtime identity:

- Add signed built-in artifact identity for `svc.demo.hello`.
- Expose artifact identity/trust ids and hashes in load, inventory, health, and
  RAM audit.
- Keep arbitrary artifact bytes, executable mapping, persistence, durable audit,
  rollback, provider auto-load, and broad mutation denied.

Track B, provider trust and context:

- Harden the direct provider path toward SPKI/WebPKI trust.
- Keep provider context injection gated by typed request/export/authorization
  evidence.
- Never attach raw snapshots, logs, secrets, or local-only fields.

Track C, UI and interaction:

- Improve response wrapping, scrolling, and settings controls.
- Keep UI state derived from the typed system model rather than duplicated
  strings.

Track D, VM harness and evidence:

- Keep focused smoke profiles fast.
- Add predicates only when they prove a real positive behavior or necessary
  fail-closed denial.
- Make reports authoritative from observed execution, not hand-copied summaries.

Track E, recovery and persistence:

- Keep recovery lifeline, durable audit, rollback, and persistence designed from
  the final trust model.
- Do not implement fake persistent memory or rollback before the evidence chain
  exists.
