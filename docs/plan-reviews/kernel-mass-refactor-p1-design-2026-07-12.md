# Kernel Mass Refactor P1 Implementation Design (2026-07-12)

Packet: `REFACTOR-P1-DESIGN`. This design is limited to P1 readability work on
inventory rows routed `RELOCATE`. It assigns no implementation work to a
`RETIRE` row. P1 is behavior-neutral: serial bytes, gate decisions, authority,
and fail-closed behavior must not change.

## 1. Loader-runtime split

Keep `seed-kernel/src/agent_protocol_module_loader_runtime.rs` as the public
facade so its two callers and `main.rs` remain unchanged. Move whole functions,
never partial function bodies, into six private child modules. The ranges below
are planning anchors from the 10,156-line / 487,849-byte baseline; an
implementer should move at the nearest function boundary if surrounding
attributes or comments require it.

| Result file | Baseline range | Approx. size | Ownership | P2 destination |
| --- | ---: | ---: | --- | --- |
| `seed-kernel/src/agent_protocol_module_loader_runtime.rs` | 1-1,127 | 1,127 lines / 53,597 B before extraction; expected smaller afterward | Facade; reads current-boot event-log facts, calls pure assembly/evaluation, and exposes only `emit_module_loader_runtime` and `emit_module_loader_runtime_selftest` | Keep as the thin kernel RAM-ring adapter; move its record construction later to `raios_core::module_loader_runtime::render` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/render.rs` | 1,128-2,897 | 1,770 / 76,031 B | Emit-only field builders: retained evidence, header/policy fields, blocked-by gates, live-load boundary projections, and runtime-fact rendering | `raios_core::module_loader_runtime::render` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/evidence_core.rs` | 2,898-4,540 | 1,643 / 72,731 B | Typed source-fact map, selftest-value rendering, execution/descriptor/artifact/authorization source-evidence construction | `raios_core::module_loader_runtime::evidence` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/evidence_live_load.rs` | 4,541-6,279 | 1,739 / 87,126 B | Remaining live-load source-evidence constructors, prior-evidence carry-forward, completeness check, and source-evidence record assembly | `raios_core::module_loader_runtime::evidence` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/snapshot.rs` | 6,280-8,011 | 1,732 / 96,860 B | Runtime candidate snapshots plus missing/available fact and boundary constructors; no emission | `raios_core::module_loader_runtime::model` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/eval.rs` | 8,012-9,177 | 1,166 / 54,564 B | Pure gate evaluation, reasons/statuses, and completeness predicates | `raios_core::module_loader_runtime::eval` |
| `seed-kernel/src/agent_protocol_module_loader_runtime/selftest.rs` | 9,178-10,156 | 979 / 46,940 B | Reference selftest case table and case evaluation | `raios_core::module_loader_runtime` host tests; retain only one thin in-VM integration sanity case in P2 |

Every result is below 3,000 lines and 120 KiB with at least 19% byte headroom.
The split adds six files; it does not add a parallel facade or compatibility
shim.

The facade should declare the children after its local imports/macros:

```rust
mod eval;
mod evidence_core;
mod evidence_live_load;
mod render;
mod selftest;
mod snapshot;
```

`main.rs` change: **none**. Preserve its exact existing declaration:

```rust
mod agent_protocol_module_loader_runtime;
```

Child-to-child APIs are `pub(super)` only. Imports should be explicit. Move
`push_status_reason!` and `bf!` into `render.rs`, where all current uses live,
instead of exporting macros across module boundaries. Preserve the
`#[allow(clippy::too_many_arguments)]` attributes with their two functions.
The two existing `pub(crate)` emit entry points stay in the facade, so
`agent_protocol.rs` needs no edit.

## 2. Wide-line analysis and proof classes

The measured threshold is greater than 400 characters. Sampling found only
two of the five named files have such lines.

| File | >400-char lines | Cause and classification | Safe transform | Cheapest sufficient proof |
| --- | ---: | --- | --- | --- |
| `agent_protocol_memory.rs` | 108; max 23,859 | 51 tuple rows around 1,018-1,702 and 57 invocations of `define_direct_binding_fields!` around 2,099-2,880. They are code/tables containing string literals, but the width comes from many macro tokens on one physical line, not from one indivisible value. | Insert newlines/indentation only between existing tokens and tuple arms. Do not split identifiers or literal contents and do not reorder arms. | `git diff -w --exit-code -- seed-kernel/src/agent_protocol_memory.rs` must be empty, then release build. This proves the source token stream is unchanged. |
| `agent_protocol_module_load_gate_render.rs` | 17; max 2,994 | One macro table row, one long `required` JSON literal, seven long retained-reference JSON literals, one long descriptor-boundary literal, and seven long requested-capability/boundary literals. Most width is serialized string data passed to `raw`. | Reflow the macro row with whitespace only. For strings, use `raw(concat!("fragment", "fragment"))` to preserve one call, or consecutive `raw("fragment")` calls only after checking `raw` is append-only. Split only between complete escaped JSON tokens; never introduce/remove spaces. | Token changes are not established by `diff -w`: run release build and the `full-module-load-gate` focused profile, and compare the affected command serial bytes with the pinned pre-P1 baseline. Treat consecutive `raw` calls as the same expensive class. |
| `event_log.rs` | 0; max line <=400 | No wide-line defect in the sampled 7,141-line file. Its problem is total size (282,628 B), not long literals or tables. | No P1 rewrap. Leave on the explicit cap exemption; P2 relocates pure event logic while keeping the RAM-ring adapter. | No transform, hence no proof run attributable to this file. |
| `event_log_types.rs` | 0; max line <=400 | No wide-line defect in the sampled 3,918-line file. It exceeds 200 KiB through accumulated type vocabulary. | No P1 rewrap. Leave on the explicit cap exemption for P2 relocation. | No transform. |
| `agent_protocol_module_types.rs` | 0; max line <=400 | No wide-line defect in the sampled 3,144-line file. It exceeds warning thresholds through accumulated typed vocabulary. | No P1 rewrap; P2 relocates the vocabulary. | No transform. |

Use two proof batches: (A) whitespace-only `agent_protocol_memory.rs`; (B)
value-preserving token changes in `agent_protocol_module_load_gate_render.rs`.
Do not mix them. Although `git diff -w` plus a release build is sufficient to
classify batch A, the P1 phase contract independently requires a focused VM
profile for every implementation slice; run it after the cheap proof rather
than using it to compensate for a non-empty whitespace diff. Batch B always
requires byte-identical focused serial evidence. `cargo fmt` may be used only
after inspecting that it did not alter literal contents or macro-arm order.

## 3. `scripts/check-source-size.ps1` specification

The script uses repository-native PowerShell and no dependency. From the repo
root it unions tracked and non-ignored untracked `*.rs` paths, excludes
`.git/**`, `.cargo-home/**`, `target/**`, and vendored sources under `vendor/**`,
then reports deterministic path-sorted rows. Count physical lines with
`[IO.File]::ReadLines()` and bytes with `Get-Item.Length` (1 KiB = 1,024 B).

- Warn when either dimension is at least 3,000 lines or 122,880 bytes.
- Fail with exit code 1 when either dimension is at least 5,000 lines or
  204,800 bytes.
- Print both dimensions and which thresholds fired. Exit 0 when there are only
  warnings.
- An exemption stores the adoption line and byte baselines. It may shrink, but
  fails if either dimension grows above its own baseline. Once both hard caps
  are cleared, a stale exemption is itself an error so the entry is removed.
- The exemption is temporary and exact; no glob, family, generated, or
  "documented plan" bypass is accepted.

Adoption exemptions (the RETIRE rows are listed only because the checker must
adopt against the real tree; P1 assigns them zero design or refactor work):

| File | Baseline lines | Baseline bytes | Route |
| --- | ---: | ---: | --- |
| `seed-kernel/src/agent_protocol_memory.rs` | 3,263 | 611,483 | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_render.rs` | 6,768 | 322,324 | RELOCATE |
| `seed-kernel/src/agent_protocol_module_loader_runtime.rs` | 10,156 | 487,849 | RELOCATE; exemption should disappear in the loader split packet |
| `seed-kernel/src/agent_protocol_recovery.rs` | 6,167 | 296,022 | RETIRE |
| `seed-kernel/src/event_log.rs` | 7,141 | 282,628 | RELOCATE |
| `seed-kernel/src/event_log_types.rs` | 3,918 | 216,113 | RELOCATE |
| `seed-kernel/src/hello_service/emitters.rs` | 5,086 | 265,421 | RELOCATE |

The exact replacement paragraph for AGENTS.md end-of-session check 1 is:

> 1. Source-size check: run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-source-size.ps1`. It warns at 3,000 lines or 120 KiB and fails at 5,000 lines or 200 KiB per owned `.rs` file. Temporary adoption exemptions are exact no-growth baselines and must be removed as P1-P3 shrink the files; do not add or widen an exemption instead of splitting or relocating a file.

## 4. P1 implementation packets

These worker write sets do not overlap. Timestamped VM reports are generated
verification evidence, not hand-edited packet files. The orchestrator owns the
shared phase-close `PROJECT_STATUS`, `OWNER_DASHBOARD`, and (only if the cursor
changes) `ROADMAP` updates after all packets, avoiding concurrent shared-doc
writes.

### P1-A — loader ownership split

Exact write set:

- `seed-kernel/src/agent_protocol_module_loader_runtime.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/render.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/evidence_core.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/evidence_live_load.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/snapshot.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/eval.rs`
- `seed-kernel/src/agent_protocol_module_loader_runtime/selftest.rs`

Checks: confirm every result is below both warning thresholds; `cargo fmt --all
-- --check`; release kernel build; focused `full-module-load-gate` profile with
affected serial output byte-identical to baseline; secret scan. Verification
tier: focused VM because this moves trust/gate and boot-compiled module code.

### P1-B — whitespace-only memory reflow and size-rule adoption

Exact write set:

- `seed-kernel/src/agent_protocol_memory.rs`
- `scripts/check-source-size.ps1`
- `AGENTS.md`

Checks: `git diff -w --exit-code -- seed-kernel/src/agent_protocol_memory.rs`;
run the new size checker and confirm only named exemptions can exceed hard caps;
`cargo fmt --all -- --check`; release kernel build; focused `memory-durable`
profile; secret scan. Verification tier: cheap whitespace/build proof first,
then focused VM solely because the P1 contract requires one per slice.

### P1-C — load-gate serialized-literal reflow

Exact write set:

- `seed-kernel/src/agent_protocol_module_load_gate_render.rs`

Checks: `cargo fmt --all -- --check`; release kernel build; focused
`full-module-load-gate` profile with byte-identical affected serial output;
rerun the size checker; secret scan. Verification tier: focused VM because the
transform changes Rust tokens on a trust/gate serialization path.

At phase close, the orchestrator carefully reads the joined diff, runs the
`full` profile (the P1 contract requires full at phase close), runs the secret
scan, updates shared status/dashboard docs, and performs the repository
end-of-session checks. The existing owner cadence reserves `recovery` for a
sub-milestone close or changed recovery boundary; P1 does not change recovery.

## 5. Behavior-change risks

- **Macro scope:** `push_status_reason!` and `bf!` are textually scoped today.
  Moving their callers into child modules without moving the macros will fail
  or invite export widening. Keep both private in `render.rs`. Memory's two
  macro definitions must retain arm order and token content during reflow.
- **Visibility:** sibling helpers that were private in one file need only
  `pub(super)`, never `pub(crate)`. The facade remains the sole crate-visible
  API. An accidental broader export is a design failure even if it compiles.
- **Attributes:** the two `#[allow(clippy::too_many_arguments)]` attributes at
  baseline lines 5,733 and 6,031 must move with their functions. There are no
  sampled `cfg` attributes in the loader file; if implementation finds one at
  a moved boundary, move it with the item.
- **Literal bytes:** `concat!` preserves the compile-time `&str` value only if
  every escaped byte, punctuation mark, and intentional space is retained.
  Consecutive `raw` calls are assumed equivalent only after confirming `raw`
  is append-only; otherwise use `concat!`. Uncertainty is assigned to the
  expensive focused byte-comparison class.
- **Ordering:** field arrays, macro arms, blocked-by lists, and selftest cases
  are serialized in source order. Reflow may not sort or regroup them.
- **`include!`:** none was found in the six scoped source files. Do not
  introduce it as a shortcut; it obscures module ownership and changes macro
  and diagnostic context.
- **Line-dependent output:** no `line!`, `file!`, `panic!`, `assert!`, or
  `debug_assert!` path was found in the loader target. Compiler diagnostics will
  change file/line locations, but known serial schemas should not. Any newly
  discovered line/file string in an emitted or panic path forces focused
  byte-identical proof and blocks a whitespace-only classification.
- **Concurrent lane:** `main.rs`, `agent_protocol.rs`, and
  `agent_protocol_system.rs` are being changed by W6. This design deliberately
  requires no edit to them. Any implementation that needs one must stop and be
  rescheduled after W6 rather than broadening its write set.

## Out of scope observed

The five sampled secondary targets are not uniformly wide-line files:
`event_log.rs`, `event_log_types.rs`, and `agent_protocol_module_types.rs` have
zero lines above 400 characters. P1 should not manufacture churn in them.
`event_log.rs` and `event_log_types.rs` still exceed the new hard byte/line caps
and therefore remain exact no-growth exemptions until their routed P2
relocation. `agent_protocol_module_types.rs` is warning-only. RETIRE-routed
`agent_protocol_recovery.rs` is also over the hard caps, but its only P1 work is
the checker exemption entry; no split or relocation is proposed.
