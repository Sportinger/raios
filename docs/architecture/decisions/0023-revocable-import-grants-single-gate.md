# 0023 — Wasm import grants are individually revocable through one enforcement gate over a folded audit chain

Datum: 2026-07-19 · Status: aktiv

## Kontext

A Wasm guest's only authority is the host imports granted to it. Today that
authority is frozen when the guest's wasmi `Linker` is constructed: the
granted host functions are closed into the instance, and there is no revoke
path (SCOPE §2 "Grant/revoke a typed capability import explicitly" and
"Revocation prevents the next host call … no stale instance retains the
revoked authority" are open; §3 "rollback to a version with fewer grants
revokes the delta" depends on it). Two repo facts constrain any design:
registration is decentralized (`Linker::func_wrap` is called from ~7 shim
files, both the envelope path `envelope.rs:648` and the beyond-env path
`invocation.rs:426`), and execution is fuel-metered and **resumable** — a
guest can be suspended mid host-call-stack and resumed thousands of pump
turns later, possibly after a revoke landed. The durable record model is
append-only immutable audit: a later record can never supersede an earlier
one (`memory_record_resolve.rs:11`), so revoke must be a *new* event chained
to the grant, not a mutation. There are no kernel host tests; every property
is proven by QEMU serial predicates.

Both second opinions (Codex gpt-5.6 xhigh, read-only; Claude Fable 5 max)
were asked the enforcement question fresh and neutral, without our lean.

## Entscheidung

Adopt **revocable indirection through a single gate** (advisors' "Option 3"):

1. **One grant per surface, not per import-list.** Each `(grantee service,
   artifact/service binding, host import id, typed scope)` is its own durable
   grant with its own identity. The existing list-digest evaluator stays as
   evidence only.
2. **Authority lives in a kernel-owned per-domain grant table, keyed by
   `(domain-instance, surface)` — never surface alone** (surface-only keying
   over-revokes peers or under-enforces). What is closed into the linker is
   an inert slot id, never authority. The table is the *materialized fold of
   the append-only grant/revoke chain*, rebuilt from the chain at every boot.
3. **One enforcement gate is the sole legal registration path.** A single
   `RevocableLinker`/gate module replaces every scattered `func_wrap`; the
   raw `Linker` never leaves it. Enforced as a greppable invariant: `func_wrap`
   appears nowhere in `seed-kernel/src` outside the gate. Every host call
   passes `gate.enter(instance, slot)` exactly once before any effect;
   denial returns a typed error (not a trap — a trap is indistinguishable
   from a crash). The same table generates the per-surface deny fixtures
   (same-source discipline). Service/instance/kill-generation checks move
   into the gate; resource-object checks (socket/TLS/storage handle ownership)
   stay in handlers — they are not grant checks.
4. **Revoke is a new immutable event chained 1:1 to its grant** (parent
   record id + parent hash carried and verified in the typed payload), never
   via `supersedes`. Outer kinds: `CapabilityGrant` for issue,
   `CapabilityDenial` with predicate `wasm_import.revoked.v1` for revoke.
   Current authority ≡ grants with no chained revoke — a pure fold. Recovery
   uses a dedicated typed grant-event parser folding every valid frame in
   sequence order; a malformed link, fork, missing parent, repeated epoch, or
   ambiguous history resolves to **denied**.
5. **Ordering rule (kills the crash window): durably append the revoke
   first, then flip the slot, in the same pump turn, before any guest
   resumes.** Flip-before-append is forbidden (would enforce an event absent
   from history). Crash between the two → boot re-fold enforces the revoke
   anyway. Emit `cap.projection sha256:…` at boot so a predicate verifies
   chain↔table agreement across reboot.
6. **No slot/ticket reuse within an instance lifetime; regrant uses a new
   record and strictly larger generation.** This is the one place a
   generation counter is load-bearing — it defeats ABA so an old instance's
   old binding can never regain authority when a surface is later re-granted.
7. **Rollback synthesizes one revoke per delta grant** (grants live now minus
   grants live at target version), each a one-frame record chained to its
   grant, then rebuilds the domain's instances. Teardown is rollback's
   mechanism, not the revocation mechanism.

Decisive negative test (keeps the SAME instance alive across revoke): grant
surfaces A and B as separate durable events; call A, suspend before the next
call; revoke A (both events present, hash-chained); resume the *same*
instance; its next A call is denied with `host_effect_delta=0` while B still
succeeds (`host_effect_delta=1`); a different service's A is unaffected;
reboot replay still denies A; regrant A at a larger generation and the old
instance's old binding stays denied while only a freshly instantiated
binding succeeds.

## Alternativen & Zweitmeinungen

Both advisors independently rejected the alternatives for the same reasons,
recorded here so the rejection is durable:

- **Naked per-call epoch/generation check.** Check→effect TOCTOU: the effect
  can occur after revoke returns; also reboot-reset and ABA exposure. The
  generation is necessary for slot-reuse safety but is *not* the validity
  mechanism — the live table lookup is.
- **Instance-teardown as the authority mechanism.** Cannot stop a
  suspended-mid-execution instance from completing one more call before the
  interrupt lands; coarse (revoking A destroys unrelated B state); degenerates
  into deny-stubs (i.e. into Option 3) because a module's declared imports
  must still link *something*. Kept only as rollback's rebuild step and as
  defense-in-depth after revocation.
- **Bare capability-handle indirection.** Right for socket/TLS/storage
  objects but insufficient alone: a guest handle must still be checked against
  a live owner/import/generation table every call, or ABA and cross-service
  leakage return.

**Recorded difference (concurrence, one sub-decision).** On the resumable-call
gap — a shim that gate-checks, starts an effect, suspends, and completes the
effect after resume (post-revoke):
- *Fable:* keep the table lookup as the whole validity mechanism, drop the
  epoch as a validity device, and require the invariant "no suspension point
  between gate and effect; a shim that must suspend re-enters the gate at its
  post-resume effect boundary." Simpler; fits raiOS's few suspending shims.
- *Codex:* additionally carry a permit into the pending operation and have
  revoke actively drain/cancel in-flight permits before finalizing (a
  linearizable close→drain→append→revoked protocol). Stronger; heavier.

Reconciliation: adopt Fable's re-enter-the-gate-at-the-effect-boundary
invariant as the baseline (it subsumes the common case and is cheap to audit
across our small set of suspending shims: `net_shims`, the thread pump), and
escalate to Codex's permit-drain only for a specific shim that starts an
irreversible async effect before it can re-check. The one-turn append→flip
ordering plus single-threaded run-to-completion between yields makes
revoke-commit and host-call totally ordered for every non-suspending surface.

## Folgen

Leichter: individual revoke, durable grant/revoke audit, and the §3
fewer-grants-on-rollback delta all become one mechanism; the "next call is
denied" property is a theorem about pump ordering, not a convention across
seven files; the black-box serial regime can falsify authority surface by
surface from one source-of-truth table. Schwerer: every instance builder
(env, beyond-env, UI, WASI, thread, fixtures) must migrate to the gate façade
before the invariant holds — a wrapping that misses one path leaves a bypass,
so the migration is an **exclusive-lane** refactor gated on the full suite
staying green, sliced small: (1) façade as a pass-through routing both
registration paths + the `func_wrap`-only-in-gate invariant, no behavior
change; (2) live per-domain table + one revocable surface + the decisive
negative test; (3) durable chain + boot re-fold + projection predicate; (4)
rollback delta synthesis; (5) per-surface migration of the remaining shims.
Bewusster Trade: physical truncation/rollback of the entire RECLOG can
resurrect a pre-revoke state — no append-only format detects that alone; if
it enters threat scope, anchor the durable head sequence/hash in TPM/NVRAM or
a remote witness (future hardening, not this ADR).
