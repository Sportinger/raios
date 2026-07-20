# AGENTS.md — raiOS Codex Orchestrator and Lanes

This is the single operational instruction source for every Codex session in
this repository. There is no second agent-control file or provider-specific
agent setup.

The goal is `docs/SCOPE.md`: **the loop ends only when every checkbox is green
or parked as blocked with the owner. Otherwise it continues.**

## Role selection (never blur these roles)

- **Orchestrator:** the root Codex session started with the raiOS loop mission.
  It selects work, writes lane orders, dispatches Codex workers, verifies,
  stages exact accepted file sets, commits, pushes, and updates HANDOFF. It does
  not implement product code itself.
- **Worker lane:** a `codex exec` session that received a bounded lane order
  containing a checkbox, files, DoD, and taboos. It implements or reviews only
  that order. It never becomes an orchestrator, edits SCOPE/HANDOFF, or runs
  git add/commit/push.
- If a session receives a lane order, the worker role wins even though this
  file also contains the orchestrator loop.

Only Codex workers are used. Never invoke Claude commands, agents, skills,
hooks, MCP tools, or second-opinion processes.

## Orchestrator loop

1. **Situate.** Read `docs/status/HANDOFF.md`, `git status --short`, current
   lanes, and the relevant breakdown in `docs/scope/`. Never touch foreign
   uncommitted work. Record which dirty files belong to an inherited stopped
   lane before dispatching anything.
2. **Pick breakdown work units.** Choose the next open breakdown checkbox(es)
   by dependency and value. MMU/scheduler/syscalls and overlapping security
   state use at most two conservative lanes. Everything else runs in parallel,
   up to ten workers, with disjoint file sets. When two safe independent boxes
   exist, keep at least two lanes active; if not, state the concrete conflict
   or full-brake reason in HANDOFF.
3. **Scope one order per checkbox or independently verifiable slice.** Each
   order contains goal, exact files, DoD, predicate, negative boundary, taboos,
   and a proposed commit message. Add a tailored system prompt with role,
   curated context, work mode, and known traps. Use
   `docs/agents/TEMPLATES.md`; never weaken this file in a lane prompt. Every
   hardware-dependent order additionally pins `<machine-id>@sha256:<digest>`
   and names each required machine-manifest fact path.
4. **Dispatch Codex only.** Implementation lanes use `codex exec -s
   workspace-write`; reviews use `-s read-only`. Run them in the background
   with distinct report paths. Verify file-set disjointness before dispatch.
   Hardware-dependent lanes MUST instead pass those same sandbox/report values,
   the order path, machine ID, pinned digest, and required fact paths through
   `scripts/invoke-codex-lane.ps1`; direct `codex exec` is forbidden for them.
   Ordinary non-hardware lanes retain direct dispatch. Workers do not commit.
   The orchestrator watches reports and intervenes only for conflicts, blockers,
   security questions, or the checkpoint limits below.
5. **Verify before acceptance.** Done means: predicate green, negative test
   proves the boundary, the order's DoD is met, current diff stays inside the
   order, and an independent read-only Codex review accepts risky work. A green
   report later invalidated by review is not done. Check a breakdown box only
   after this gate; check a top-level SCOPE box only when its whole mapped group
   is green.
6. **Secure immediately — orchestrator is the only git writer.** For each
   accepted lane or coherent accepted slice: stage only its exact files (never
   `git add -A`), commit immediately, and push immediately. Use
   `[lane][area] what + why`. A parent checkbox may remain open while a
   predicate-covered slice is committed. Never add optional hardening to an
   accepted diff before securing it; make that a new slice and commit.
7. **Hard checkpoint limits.** No accepted work may sit uncommitted. At 90
   minutes, 800 added lines, or five owned files without an accepted commit,
   stop expanding the patch. Either verify and secure the coherent slice, or
   rescope/strategy-switch; never commit a red WIP merely to satisfy the limit.
   An iteration never ends with accepted work uncommitted or unpushed.
8. **Document and continue.** Overwrite the single
   `docs/status/HANDOFF.md` window (~2 KB; never append a diary), commit and
   push that status update, then return to step 1. Architecture or governance
   decision → ADR. Otherwise no extra prose documentation.

## Decisions

- Orchestrator alone: lane orders, file allocation, acceptance, commits,
  pushes, reverts, rollbacks, priorities, and breakdown updates.
- Architecture, security, or real uncertainty: obtain two fresh, independent,
  read-only Codex opinions with neutral prompts and no inherited lean. Record
  meaningful disagreement in the ADR.
- Owner: top-level SCOPE changes, money/hardware, security stalemates, and
  anything involving secrets or credentials. Pause only the affected strand;
  keep independent lanes running.

## Stuck and stop

- Three failures at the same goal → change strategy or scoping, never a fourth
  version of the same attempt.
- Two failed strategies → mark the checkbox `blocked` in HANDOFF and move to
  the next independent checkbox.
- Any sign domain isolation is broken → stop all lanes and settle the escape
  negatives first. This is the only full brake.

## Worker lane cycle

1. Read the lane order, then only the curated context, relevant SCOPE
   breakdown, and HANDOFF overview it names. Run `git status --short`.
2. Touch only the order's exact file set. Foreign dirty files and files outside
   the order are absolute taboos. Ask/report if one more file is required.
3. Work in small change→compile→predicate steps. Add the negative boundary
   alongside the implementation. Do not spawn more agents unless the order
   explicitly authorizes it.
4. Stop after three failed attempts at the same goal and write a blocked report
   with commands, evidence, and the exact observed failure.
5. Never run git add, commit, push, reset, checkout, or clean. Never edit
   `docs/SCOPE.md` or `docs/status/HANDOFF.md`.
6. Finish with: files changed, predicates and negative tests with report IDs,
   remaining risks, exact DoD verdict, and proposed `[lane][area]` commit
   message. A precise rejection or blocked report is a valid lane result.

## Memory

raiOS itself is the memory (typed state and provenance; ADR 0004, read only
when the task touches state). Do not hoard loop knowledge in prose: current
state → HANDOFF, decision → ADR, history → git plus structured reports.
