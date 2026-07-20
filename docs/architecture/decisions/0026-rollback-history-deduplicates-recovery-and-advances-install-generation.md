# 0026 — Rollback history deduplicates recovery and advances install generation

Date: 2026-07-20 · Status: superseded in part by ADR 0029

ADR 0029 supersedes this ADR's recovery re-persist deduplication decision after
review proved that legitimate recovery and an authentic replay are
cryptographically indistinguishable in the current durable format. The strict
install ordering and new-physical-install generation decisions remain active.

## Context

The strengthened grant-delta rollback run reached boot-2 autoload but denied
the first rollback command with `rollback_target_authenticated_order_duplicate`.
The disk contains a signed log-only predecessor and a distinct signed source
install. Normal boot recovery then re-persists the source with the same signed
install action, so a raw persist-record scan mistakes one recovered install for
two installations. The predecessor and source also both carry generation 1,
although their signed log sequences are 1 and 6.

Two fresh read-only Codex reviews agreed on the replay cause and on preserving
the duplicate/tamper boundary. They disagreed on repair scope. The inventory
review recommended changing only `artifact_store.rs` first and stopping if the
generation ordering then failed. The causal review recommended changing both
`artifact_store.rs` and `granted_candidate_service.rs`, because the existing
strict Core order necessarily rejects equal generations with different log
sequences after replay collapse.

## Decision

1. Durable install history is canonicalized by verified signed install-action
   identity, not by the number of artifact-persist frames. A kernel recovery
   re-persist of the same authorization is one installation only when its
   artifact, grant-target snapshot, signed order, and linked authorization all
   agree exactly.
2. Repeated signed payloads, a repeated linked authorization/promote/persist
   triple, or any disagreement within one claimed identity remains a fail-closed
   duplicate or ambiguous-order error. Recovery deduplication must not make the
   adversarial duplicate fixture acceptable.
3. A new physical install derives its generation as one greater than the
   maximum generation in the fully verified durable install history. Recovery
   autoload restores the already signed generation unchanged and never consumes
   a new generation.
4. Core predecessor selection remains strict: both generation and signed log
   sequence must advance monotonically. The repair must not relax
   `select_authenticated_predecessor` or use physical frame order as authority.
5. The focused rollback profile must prove intent → exact-parent revoke →
   commit → RAM install → zero-effect counter denial before rebuild, and must
   then execute every existing tamper, duplicate, recovery, retry, and peer
   isolation boundary.

## Alternatives & second opinions

- Repair only the artifact-store replay scan: rejected as the final design.
  It is a useful diagnostic boundary, but the already observed `{generation=1,
  log=1}` and `{generation=1, log=6}` pair would immediately fail the unchanged
  Core monotonic-order contract and has no predecessor under its strict `<`
  relation.
- Accept equal generations when signed log sequence advances: rejected because
  it weakens the independent version/order cross-check and would turn a known
  fixture defect into accepted authority history.
- Stop re-persisting during normal recovery: rejected for this repair because
  it changes the established repromotion durability model and would not define
  the correct generation for the next physical install.
- Treat every repeated install-action identity as benign recovery: rejected
  because the authenticated duplicate negative deliberately repeats genuine
  signed payloads with new outer framing and must remain denied without effect.

## Consequences

The repair is limited to the artifact-history reader and physical-install
generation producer. Existing Core ordering, signed fixture construction,
image mutators, grant-delta journal, and harness expectations remain unchanged.
The implementation must distinguish a legitimate kernel recovery replay from
an adversarial duplicate using fully verified linked context; this adds scan
work but preserves fail-closed behavior and produces a real ordered predecessor.
