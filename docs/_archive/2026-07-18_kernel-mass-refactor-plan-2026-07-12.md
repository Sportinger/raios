# Kernel Mass Refactor Program — Plan (2026-07-12)

Owner decision 2026-07-12: execute a real refactor of the accumulated agent
protocol / evidence mass, cost accepted ("egal wie teuer"). This plan defines
the program. It supersedes the "optional, later" parking of vocabulary
compaction in ADR 0006 by scheduling it as the LAST phase of a sequenced
program, not the first.

## Measured Problem (2026-07-12)

- `seed-kernel/src`: 206,481 lines across 162 files, one `no_std` binary,
  no host tests — every check needs a QEMU boot.
- `raios-core`: 35,875 lines, 415 host tests, runs in under a second.
- `agent_protocol*` family: 106,711 lines / 95 files (~52% of the kernel),
  of which `agent_protocol_module_*` 50,611 / 29 files and
  `agent_protocol_recovery*` 39,193 / 47 files.
- `hello_service/`: 19,156 lines / 20 files of labeled test infrastructure.
  M6 promotion loop and M7 persistence are COMPLETE; a substantial share of
  hello's evidence role is now supersedable by the real loop.
- `event_log*`: 15,064 lines / 6 files (`event_log.rs` 282 KB,
  `event_log_types.rs` 216 KB).
- Byte-width loophole in the size rule: `agent_protocol_memory.rs` is 611 KB
  in only 3,263 lines (108 lines exceed 400 chars) — largest file by bytes,
  invisible to the line-count rule. `agent_protocol_module_loader_runtime.rs`
  is 10,156 lines / 487 KB — above the "exceptional and documented" 10k
  threshold with no documented split plan found. README's claim that every
  file is below the readability thresholds has drifted from reality.
- ADR 0006 accepted the current agent layer as the "byte-identical collapse
  floor" and parked Batch 6 (vocabulary compaction, est. −30k+ lines) as an
  optional owner decision. The mass is live: all four largest files are
  declared in `main.rs` and dispatched through the method table. They are
  read-only evidence emitters, not boot-path logic.

## Why Now

1. W4 just closed with a green full baseline (7870/7870) plus focused
   reports; block close is exactly when the cadence already demands
   full/recovery evidence, so the program inherits a fresh baseline for free.
2. Program start gates on the product cursor: at plan time the W5 slice
   (current-boot app run under computed Wasm imports) is IN FLIGHT with an
   active worker and an uncommitted tree. P0 (read-only inventory) may run
   during W5 — it writes only a plan appendix and cannot conflict. P1 and
   everything after start only on a committed, green, post-W5 tree; the
   program's full+recovery baseline is taken at W5 close.
3. Every future slice (W5+, M11 relocations, drivers) builds on this
   surface; deferral makes the same refactor strictly more expensive.
4. The two required tools are already proven in-repo: the byte-identical
   golden-output harness (M2 collapse program) and the relocation crate
   pattern (M11-6 X.509, M11-7 HTTP parse).

## End State (measurable exit criteria)

1. All pure decision/emit/parse logic is host-testable (in `raios-core` or
   carved no-dep crates); in-guest selftest reference-case tables are ported
   to `cargo test`; VM predicates remain only for hardware, boot, authority,
   and integration behavior.
2. File caps enforced by script: warn at 3,000 lines or 120 KB, hard fail at
   5,000 lines or 200 KB per `.rs` file (bytes cap closes the wide-line
   loophole). The script joins the end-of-session checks in `AGENTS.md`.
3. Superseded evidence surfaces are deleted; `hello_service` is reduced to
   the slices the real M6/M7 loop does not already prove.
4. Remaining in-kernel emit vocabulary is table-driven over the typed record
   model (Batch 6 executed; ADR 0006 formally reopened and closed).
5. Target: `seed-kernel` at or below ~120k lines (from 206k) with zero
   capability regression. Every currently green predicate is either
   preserved byte-identically, ported to a host test, or retired with a
   named justification in the commit that removes it.

P0 replaces these estimates with firm per-family numbers before any code
moves.

## Phases

### P0 — Inventory and baseline (read-only, grants nothing)

One Codex recon slice. For all 95 `agent_protocol*` files plus
`hello_service` and `event_log*`: which dispatch entries are live, which
harness profiles consume them, what is pure vs hardware-touching, what is
superseded by the real M6/M7 loop. Output: an appendix table (file, lines,
bytes, dispatch methods, harness consumers, classification) that routes
every family to exactly ONE of three fates — RETIRE (P3), RELOCATE (P2), or
KEEP-IN-KERNEL (P4 candidate) — plus firmed targets. Record the fresh
full+recovery baseline report names.

Routing rule: a family routed to RETIRE is never relocated first; P2 spends
effort only on survivors. P3 may therefore run before or interleaved with
P2, family by family, as the map dictates. This program preserves old code
only while it serves as the verification oracle for its replacement or is
genuinely still doing its job — preservation is a means, never the goal.

### P1 — Emergency readability splits (byte-identical)

Split `agent_protocol_module_loader_runtime.rs` along its ownership
boundaries; rewrap the wide-line files (`agent_protocol_memory.rs`,
`agent_protocol_module_load_gate_render.rs`, `event_log.rs`,
`event_log_types.rs`) to sane line widths; land the size-check script and
the bytes cap in `AGENTS.md`. 2–4 slices. Serial output must stay
byte-identical (focused profile per slice, full at phase close). No
behavior change of any kind.

### P2 — Host relocation of pure logic (byte-identical; the core phase)

Family by family (`module_*`, `recovery*`, `event_log`, `provider`,
`memory`): carve pure evaluators and emit-builders into host-testable crate
modules using the proven relocation pattern (no-dep crate or `raios-core`
module, `pub use` re-export, kernel byte-identical); port the in-guest
selftest reference-case tables to `cargo test`; keep one thin in-VM sanity
predicate per family. Est. 12–18 slices, batched per the 2026-07-10
verification preference: focused profile per batch, full at each family
close. This is what converts minutes-per-check QEMU time into
seconds-per-check host tests.

### P3 — Retire superseded surfaces (deletion-dominant)

Using the P0 map, per family and with owner confirmation: delete denial-only
emitters whose real capability now exists, collapse `hello_service` to its
non-superseded slices, remove duplicate selftest emit surfaces now covered by
host tests. Every deletion updates the harness predicate lists in the same
slice. Est. 5–8 slices. The full-profile predicate count will DROP by
design; each drop is named and justified in its commit — retirement, not
lost coverage (the Capability DoD already forbids predicate-count progress
reporting).

### P4 — Vocabulary compaction (behavior-changing; formal Batch 6)

Reopens ADR 0006 explicitly. Replace per-method hand-built field lists with
table-driven emission over the typed record model for what remains
in-kernel. Golden outputs change: regenerate per family, full+recovery at
close. Est. 8–12 slices. Deliberately LAST: compaction is cheapest on a
surface that is already host-tested (P2) and smaller (P3).

P4 opens with an explicit owner choice of ambition level: (a) minimal
compaction — same vocabulary, table-driven emission, lowest golden churn;
or (b) evidence vocabulary v1 — a from-scratch redesign of the emitted
schema set on the typed record model, larger reduction, every golden
regenerated. By P4 the surviving logic is host-tested, so equivalence of
MEANING (not bytes) is provable by cargo tests either way; (b) is the
"new and better" option and is legitimate here, where it is cheap — not at
program start, where it would destroy the only oracle.

## Rules and Invariants

- Red Gate rule, cadence rules, secret scan, and commit discipline are
  unchanged. No slice ends with uncommitted source.
- P1–P2 must keep serial output byte-identical; P3 changes it only by named
  predicate retirements; only P4 changes emitted shapes.
- Fail-closed posture is untouchable: no gate, denial, or authority check
  may be weakened by relocation, deletion, or compaction.
- One QEMU suite at a time. Parallel lanes stay available: G7 read-only
  stick preflight, UI polish. W5 may proceed as a disjoint parallel lane if
  the owner wants product motion — write sets barely overlap (new
  `project_*`/Wasm-engine files vs. old families; `agent_protocol.rs`
  dispatch table is the only shared file, merges are additive).
- ADR 0015 records this program's decision; ADR 0006 is formally reopened at
  P4 start, not before.

## Cost (honest ballpark)

30–40 Codex worker slices plus orchestration and QEMU wall-clock. At the
demonstrated W1–W4 pace (four verified blocks in one day), P0–P1 fit in one
session, P2–P3 in a few focused sessions, P4 in a comparable block. VM
wall-time dominates and shrinks structurally as P2 proceeds.

## Risks and Mitigations

- Golden churn in P4 → it runs last, family-wise, on a host-tested surface.
- Hidden consumers of retired surfaces → P0 maps harness consumers first;
  deletions fail closed in the full profile and CI.
- Merge friction with parallel lanes → disjoint write sets, additive
  dispatch-table merges, one QEMU suite rule.
- Optics of a dropping predicate count → dashboard and commits explain
  retirement per the Capability DoD; coverage moves to host tests, it does
  not vanish.
