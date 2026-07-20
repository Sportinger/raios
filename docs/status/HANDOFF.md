# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~13:30, root orchestrator active)

HEAD/origin is `5f6bb10`. Core signed predecessor selection, deterministic
rollback fixture, machine-context injection, ADR 0029, and the independently
reviewed image-security helper are committed and pushed. Foreign diagnostics,
the experiment lockfile, and permission-denied temp directories remain taboo.

W10's five-file RAM-only recovery repair is locally green but provisional.
W13B owns its three-file runtime/grant-table dependency: exact empty import
targets, no durable fallback, and quarantine-safe projection replacement. W15
owns only the dirty image builder, deleting duplicate verifier code and wiring
the committed helper below the 800-line checkpoint. Neither lane is accepted.

The NET8 repair is blocked after UserKeySet and direct-certificate strategies
both reached host Schannel `SEC_E_NO_CREDENTIALS`; its partial Program/wrapper
diff is rejected and must not be staged. QEMU rollback evidence therefore needs
an owner-provided Windows host with a usable TLS credential provider or an
owner-authorized alternative test credential strategy. Surface manifest closure
also needs owner access to capture actual CPU, memory, and device facts.

## Next step

Poll W13B/W15. Review W10 plus its runtime dependency as one frozen coherent
security slice; separately review the wired image builder. Commit and push each
accepted exact slice immediately, then repair/review the rollback QEMU harness.
Without a NET8-capable host, park only live QEMU evidence with the owner and move
to the independent unsafe-inventory build gate.

## Recently (exactly 3, newest first)

### 2026-07-20 — image authority extracted below checkpoint
The pure fixture/P-256/GPT/recovery verifier passed 8/8 tests, 154 hostile
variants, and two independent reviews; commit `5f6bb10` is pushed.

### 2026-07-20 — RAM-only recovery architecture chosen
ADR 0029 rejects replay-shaped re-persistence without a protected device key
and monotonic anchor; implementation review remains open.

### 2026-07-20 — rollback prerequisites secured
Strict signed order, semantic fixture, and bounded QEMU machine context were
reviewed and pushed as `23a88b4`, `d9e054a`, and `58cb935`.
