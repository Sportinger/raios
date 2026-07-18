# ADR 0006: M2 re-scope — accept the byte-identical collapse floor

Date: 2026-07-06. Status: accepted (provisional — the owner may override
by choosing vocabulary compaction, which would reopen this ADR).

## Context

M2 "Ceremony Collapse" originally targeted an agent layer "~10x smaller"
(~20k lines). The byte-identical collapse program (Batches 1–5,
2026-07-05/06) delivered:

- ONE typed record model rendering every response and event binding
  (serializer/hasher derive from the same structure — the divergence bug
  class is structurally impossible);
- ONE MethodEntry dispatch table (was: 168-branch chain + duplicated
  console routing);
- ONE shared CommandBindings/StageBinding representation (was: 48 cloned
  per-stage structs + 30 positional parsers);
- ONE selftest runner over const case tables (was: hand-written case
  factories per family);
- descriptor-table event bindings (all 88 variants) and table-built hash
  inputs in the attested Hello chain, each with scripted byte/order
  identity proofs;
- every source file below the AGENTS.md size thresholds; zero-warning
  build; nine green FULL profiles (7,814/7,814) across the program.

Measured result: agent layer ~126.5k lines (from ~138k at M2 start).
The remaining mass IS the emitted evidence vocabulary itself: reaching
~20k requires changing output shape (compacting bindings, moving negative
selftests to host tests), which changes golden needles and verified
behavior — a heavier change class.

## Decision

M2's capability sentence is re-scoped to what the collapse program
actually proves: "The agent layer renders through one typed record model
with structurally non-divergent hashing, one dispatch table, one command
representation, and one selftest runner; every file is small enough for
an agent to fully read; byte-identical behavior proven by the
golden-string harness." Under this sentence, M2 is CLOSED.

Vocabulary compaction (the former Batch 6, est. -30k+ lines) becomes an
OPTIONAL later milestone-slice (natural home: alongside M5, when the
second service forces vocabulary generalization anyway). Choosing it
reopens nothing else.

## Rationale

The disease review-4 diagnosed — divergence bug class, unmodelable
monolith, ceremony growth — is cured by structure, not by line count.
M3–M6 (durable write, Wasm isolation, second service, promotion loop)
advance the product thesis; further line reduction does not.
