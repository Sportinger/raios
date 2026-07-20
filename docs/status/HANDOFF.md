# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~12:50, root orchestrator active)

HEAD/origin is `264ac24`. Core signed predecessor selection, the deterministic
signed rollback fixture, validated machine-context injection, and ADR 0029 are
accepted, committed, and pushed. The inherited rollback integration remains
dirty only in its allocated Rust/runtime/image/harness files; foreign generated
diagnostics and the experiment lockfile remain untouched.

Two conservative lanes are active. W10 replaces same-action re-persist
canonicalization with one physical chain, RAM restore, hard duplicate denial,
and zero-grant targets. W11 was stopped unaccepted at +903 after crossing the
hard checkpoint. W12 extracts a sub-700-line independent security helper before
any smaller main-script wiring attempt. Neither active slice is accepted.

The NET8 repair is blocked after UserKeySet and direct-certificate strategies
both reached host Schannel `SEC_E_NO_CREDENTIALS`; its partial Program/wrapper
diff is rejected and must not be staged. QEMU rollback evidence therefore needs
an owner-provided Windows host with a usable TLS credential provider or an
owner-authorized alternative test credential strategy. Surface manifest closure
also needs owner access to capture actual CPU, memory, and device facts.

## Next step

Poll W10/W12, verify exact diffs and focused negatives, and obtain fresh
read-only Codex acceptance reviews for any green security slice. Commit and push
each accepted exact file set immediately; otherwise rescope or park its strand.
Then reconcile the rollback breakdown checkbox, secure this HANDOFF update, and
select the next independent non-hardware checkbox while owner-blocked hardware
and NET8 strands remain parked.

## Recently (exactly 3, newest first)

### 2026-07-20 — RAM-only recovery architecture chosen
ADR 0029 records two neutral opinions and rejects replay-shaped recovery
re-persistence without a protected device key and monotonic external anchor.

### 2026-07-20 — machine facts bound before lane dispatch
QEMU context now validates and injects bounded provenance; Surface remains
not-ready until real owner hardware capture. Commit `58cb935` is pushed.

### 2026-07-20 — signed rollback order and fixture secured
Core strict generation/log ordering and the independent semantic fixture were
reviewed and pushed as `23a88b4` and `d9e054a`.
