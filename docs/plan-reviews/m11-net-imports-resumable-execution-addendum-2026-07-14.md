# M11 Net Imports: Resumable Execution Addendum (2026-07-14)

Packet: `NET-RECUT-SCOPE`

Status: read-only recon and design. This document amends only the execution
shape and slices 2-4 of
`m11-beyond-env-net-imports-scope-2026-07-14.md`. Everything not explicitly
amended here remains authoritative. This addendum grants no import, does not
change `policy_allows_beyond_env`, and does not authorize the W7 arming slice.

## Decision and capability sentence

Use a main-loop-owned resumable invocation, not a nested blocking pump.
`call_resumable` starts the invocation; when a suspending host import is
reached, the kernel retains the `Store` and `ResumableInvocation` as one active
task and returns to the normal kernel loop. The existing input, serial,
network, Genesis, and recovery tasks therefore remain schedulable while the
peer is silent.

After revised NET-2, a signed built-in test invocation can suspend inside a
real wasmi host boundary, leave the core main loop responsive, and be killed by
physical F12 with exactly-once resource teardown before it can resume.

This resolves parent-scope owner decision 5 in favor of resumable execution.
It does not create a new owner decision.

## Why main-loop integration wins

A nested pump would be a smaller local diff, but it would run from inside the
current synchronous console/agent dispatch stack. Servicing serial commands or
Genesis there would recursively enter code that already owns console, UI, or
protocol state. Deferring those paths would make the recovery lifeline
unanswerable for the duration of a network operation. A separate nested input
poll would also queue physical events without giving their normal owner a safe
place to process them.

Main-loop integration has one intentional execution change: a beyond-env
request becomes an asynchronous current-boot task. Starting it returns a typed
accepted/pending result; completion or failure is emitted later against the
same invocation id. Existing env-only calls keep the current synchronous
`Func::call` path. This avoids changing proven echo/buffer/personal-shell
behavior merely to support the new lane.

Only one beyond-env invocation may be active. That matches the one TCP lease,
one W7 service instance, and one pending acquisition already required by the
parent scope.

## Exact vendored wasmi mechanics

This design uses only vendored wasmi 0.31.2.

### Creating a suspension

A suspending host function returns a custom host error as its `Trap`:

```rust
#[derive(Debug)]
struct HostSuspend {
    invocation_id: u64,
    operation_id: u32,
}

impl core::fmt::Display for HostSuspend { /* fixed non-secret text */ }
impl wasmi::core::HostError for HostSuspend {}

// After validating arguments and recording one PendingHostOperation in Store data:
Err(HostSuspend { invocation_id, operation_id }.into())
```

The marker contains only identifiers; endpoint, payload, pin, key, and peer
bytes do not enter its display text. `HostError` supplies wasmi's downcastable
`TrapReason::Host`. `Trap::from(TrapCode)` is not the suspension mechanism.

In the vendored engine, an error returned by a normally called host function
is tagged with the host function and returned from `Func::call_resumable` as:

```text
Ok(ResumableCall::Resumable(ResumableInvocation))
```

The kernel accepts that result as a planned suspension only when
`invocation.host_error().downcast_ref::<HostSuspend>()` succeeds and its ids
match the pending operation in the Store. Every other host error is terminal.
This check is mandatory because `call_resumable` also exposes ordinary
host-function errors, including `Trap::new(...)` and a host closure's
`TrapCode::OutOfFuel`, as resumable-by-origin. Those errors must never be
mistaken for scheduler yields.

The vendored engine cannot resume a host error reached through the tail-call
edge where no Wasm frame remains; it converts that case to a terminal trap.
Suspending imports therefore use ordinary Wasm `call`, and NET-2's fixture
proves that shape. If a future W7 toolchain emits a tail call to a suspending
import, validation or artifact acceptance must reject it; the runtime must not
pretend that terminal result is resumable.

### Retained state and resume values

`ResumableInvocation` retains:

- the engine handle;
- the root Wasm function;
- the host function that returned the error;
- the host error; and
- the Wasm value and call stacks.

It does **not** own the `Store`, instance, guest memory, invocation authority,
or raiOS resource handles. The kernel-owned active task retains those
separately:

```text
ActiveBeyondEnvInvocation
  Store<BeyondEnvState>
  Instance and exported Memory handles
  ResumableInvocation
  entrypoint output buffer
  invocation authority and captured kill generation
  optional PendingHostOperation
  wall/fuel/step/byte budgets
  teardown state
```

When the pending operation completes, the task writes any receive bytes into
the retained guest memory, then consumes the continuation with
`ResumableInvocation::resume`. The `inputs` slice is the result tuple of the
host function that suspended, not a new entrypoint argument. All v1 network
imports return one `i32`, so their resume value is exactly one
`Value::I32(result)`. Its type and count are checked by wasmi against the
suspending host function. The separate `outputs` buffer still matches the root
entrypoint's results.

`resume` may finish, trap, or return another `ResumableInvocation`. A second
suspension is accepted only after the same custom-error/id checks and only if
the Store contains exactly one matching pending operation.

### Fuel across resumes

Fuel belongs to the retained Store. The total invocation fuel is added once
before the initial call and is consumed cumulatively across every resume; it
is never reset or replenished at suspension boundaries. A Wasm instruction
`OutOfFuel` returned from `call_resumable` or `resume` is terminal. A host
closure's fuel-charge error may arrive wrapped as a resumable host error, so
the pump explicitly classifies `TrapCode::OutOfFuel` as terminal before the
custom suspension-marker check.

wasmi 0.31.2 has no epoch interrupt or resumable fuel-yield API. Therefore F12
cannot preempt an arbitrary guest instruction loop mid-stack. The hard bound
for that case remains the signed artifact's total fuel ceiling, and NET-2 must
measure the maximum-fuel busy-loop wall time. The runtime must not claim a
stronger instruction-level interrupt guarantee.

### Abandonment and Drop

Dropping `ResumableInvocation` recycles only wasmi's interpreter stack. It does
not know about the raiOS TCP lease, TLS keys, acquisition buffers, or handle
generations.

`ActiveBeyondEnvInvocation` therefore owns an idempotent teardown guard. Every
terminal route calls it before dropping the continuation and Store. Its final
`Drop` implementation calls the same teardown as a last-resort invariant, so
an early Rust return cannot leak global resources. Teardown order is:

1. mark the invocation terminal and invalidate all owned handle generations;
2. remove the pending operation so it cannot be resumed;
3. abort only the invocation-owned TCP socket;
4. zeroize and close owned TLS/session/key slots;
5. discard only the incomplete acquisition and preserve the prior candidate;
6. release the singleton transport lease;
7. clear bounded pending send/receive buffers;
8. record exactly one terminal outcome and cleanup receipt; then
9. drop the continuation, instance handles, and Store.

No teardown path calls Wasm, and no network, Vault, acquisition, or Store lock
is held while the continuation or Store is dropped.

## Main-loop execution model

### Start

The beyond-env start path performs the parent scope's evidence-bound grant,
module import, linker-surface, memory/table, service-generation, source-policy,
and posture checks before creating any active task. It then:

1. creates the core-owned invocation authority;
2. records the current secure-attention kill generation;
3. creates the Store, limiter, instance, memory handle, total fuel, and budgets;
4. invokes the entrypoint with `call_resumable`;
5. handles immediate finish/trap through normal teardown, or validates and
   stores the returned continuation; and
6. returns an accepted/pending result to the original serial/agent caller.

The current synchronous `execute_validated_module_bytes` remains the env-only
runner. The beyond-env runner is a separate path because its Store must outlive
the initiating call stack.

### Scheduler ownership and ordering

`PeriodicTasks` (or a directly adjacent main-loop owner) carries
`Option<ActiveBeyondEnvInvocation>`. There is no static mutex around a running
wasmi Store and no recursive execution.

While an invocation is active, the main loop keeps its existing console,
input, entropy, network, provider, and UI owners. The invocation task runs
immediately after the existing 8 ms `input::poll` task and before provider/UI
work. One invocation tick does at most one of:

- observe a terminal generation/budget/deadline condition and tear down;
- make one non-blocking progress attempt for the pending host operation;
- resume once with a completed host result; or
- do nothing while waiting for the next network/input tick.

The 8 ms contract also applies to the loop around those tasks. While a
beyond-env invocation is active, a provider/TLS path that still uses today's
blocking loops is not scheduled; a provider that already owns the singleton
lease prevents the beyond-env start. Other periodic work must remain bounded
or be deferred until after the invocation tick. Otherwise an unrelated task
could delay the next physical-input poll and invalidate the F12 guarantee.

Before every progress attempt and immediately before every resume it checks:

```text
current kill generation == invocation captured generation
service and instance generations still current
boot posture still allowed
absolute and operation deadlines not expired
pump step budget remains
pending operation id/kind/owner matches
```

A failed check terminates; it is never converted into a guest-visible retry.

The parent source-policy deadlines remain 5 seconds to connect, 15 seconds
idle per I/O direction, and 90 seconds total unless owner tuning changes them
before arming. The execution task adds an 8 ms scheduling quantum and at most
11,250 progress ticks (`90_000 / 8`) per invocation. One step is consumed for
each invocation tick that checks or advances an operation and for each resume.
The wall deadline wins if scheduling is slower; the step ceiling wins if a bug
spins faster. Existing fuel, import-call, handle, byte, and memory/table quotas
remain separate and cumulative.

### Terminal routes

The same teardown runs after:

- normal return, including a guest-returned negative application result;
- an entrypoint/start/resume trap;
- Wasm or host-side `OutOfFuel`;
- unrecognized or mismatched host error;
- F12 kill-generation change;
- service revocation or generation change;
- connect/idle/absolute timeout;
- pump-step exhaustion;
- explicit close followed by return;
- source-policy/posture invalidation; and
- abandonment through the active task's `Drop` guard.

## Network, crypto, and acquisition suspension contract

The v1 import signatures in the parent scope do not change. In particular,
`net.tcp_open/send/recv/close` continue to return `i32`.

### Network calls

`net.tcp_open`, `net.tcp_send`, and `net.tcp_recv` **always suspend** after
bounded argument/ownership checks, even if the socket appears immediately
ready. This gives every network effect a main-loop boundary and avoids a
readiness-dependent guest contract.

| Import | Work before suspension | Main-loop completion | Resume value |
| --- | --- | --- | --- |
| `net.tcp_open()` | Validate invocation/source/grant and record one open operation; no caller-selected endpoint. | Claim the lease, start the pre-bound connect, and observe non-blocking socket progress until connected or terminal. | positive generation handle, or existing negative ABI error |
| `net.tcp_send(conn,ptr,len)` | Validate handle/bounds/cap/quota and copy at most 4096 bytes into a kernel-owned pending buffer. | Attempt non-blocking sends until at least one byte is accepted or a terminal condition occurs. | accepted byte count, or negative ABI error |
| `net.tcp_recv(conn,ptr,cap)` | Validate handle/bounds/cap/quota and retain the checked destination/capacity. | Wait through non-blocking socket progress for at least one byte, EOF, or terminal result; copy bytes before resume. | received byte count, `0` for EOF, or negative ABI error |
| `net.tcp_close(conn)` | Validate ownership, run idempotent owner-only close/crypto cleanup/lease release. | No wait is needed. It may complete immediately. | `0` or negative ABI error |

The guest never receives `-6 would_block` from these imports. The value stays
reserved in the v1 error vocabulary, but W7 does not branch or spin on it. A
pending peer operation remains inside the suspended host boundary. Partial
positive sends are allowed; a subsequent guest send is another mandatory
suspension boundary.

No network helper called by the invocation task may contain the current
15/60-second loops from `tls_io`. It performs one socket inspection or one
bounded buffer operation and returns. `net::poll` remains the shared
non-blocking device/socket progress owner.

### Crypto and acquisition calls

Pure fixed-shape crypto calls may complete immediately. They never wait for
network readiness, and their input sizes/state transitions are already bounded
by the parent scope. `crypto.tls13_session_open` also completes immediately or
denies; it does not wait for entropy because entropy readiness is a
precondition. Each synchronous primitive must meet the bounded-host-call wall
test below. A primitive that cannot do so must be converted into a scheduled
operation before arming; it must not grow a private polling loop.

`acquire.chunk_accept` and `acquire.finalize` may complete immediately because
they call the bounded existing M12+ acceptance/finalize seam and do not wait on
an external peer. Their 64 KiB/hash work remains charged to fuel/byte/call
quotas and measured as a bounded host call. `env.input_*` remains immediate.

The future `secret_lease.*` import remains outside W7 and outside this recut.

## Concrete F12 semantics and guarantee

### Kill generation

The secure-attention generation is a core-owned monotonic atomic in the input
boundary, exposed read-only to the invocation scheduler. The F12 make-code
branch in `queue_key_event` increments it **before** queuing
`InputEventKind::SecureAttention`. Both PS/2 and USB already converge on that
branch through the existing main-loop `input::poll`; F12 release does not
increment it.

An invocation captures the generation after its authority is created and
before its initial call. Any inequality means killed. Comparison, not ordering,
is authoritative, so wrapping arithmetic is safe for a current-boot counter;
2^64 F12 presses in one boot are outside the physical threat model.

The queued `SecureAttention` event remains owned by the normal console/Genesis
path. The invocation task sees the generation change in the same scheduler pass
after input polling, tears down, and does not resume. Genesis consumes the
event normally on its next console turn, after the active service resources are
already invalid.

### Guarantee statement

**After the PS/2 or USB decoder observes an F12 make code, a beyond-env
invocation is never resumed again. While it is suspended on a silent peer,
kill and teardown begin in the same main-loop pass: one existing 8 ms input
boundary plus at most one 8 ms bounded transport-progress boundary, for a
16 ms kernel scheduling bound. No network wait may run longer than one 8 ms
progress quantum between suspension boundaries. The focused QEMU acceptance
bound is 250 ms from HMP `sendkey f12` to the serialized
`outcome=killed teardown_complete=true` marker, allowing HMP/HID/serial host
transport overhead.**

If Wasm or a synchronous crypto/acquisition call is currently executing, kill
is observed at its next suspension, return, or trap. NET-2 adds two acceptance
ceilings: a maximum-fuel busy-loop must terminate through `OutOfFuel` within
250 ms in the focused QEMU profile, and every synchronous host primitive must
return within 25 ms. These are honest cooperative-execution bounds, not
instruction-level preemption. Failure to meet them is a stop condition before
net imports or arming.

The 16 ms number starts when the guest input decoder can read the event; QEMU's
250 ms predicate includes host monitor delivery. Neither number weakens the
5/15/90-second operation deadlines, which remain failure ceilings rather than
kill latency.

## Main-loop integration risks and required behavior

### Reentrancy and Genesis

There is no nested console, Genesis, protocol, or Wasm dispatch. The main loop
owns the continuation and resumes at most once per task tick. A normal F12
Genesis action is deferred only until the invocation teardown in that same
scheduler pass. No other Genesis action is fired by the invocation task.

While an invocation is active, a command that would start another Wasm service,
claim the same acquisition session, or claim the singleton TCP lease returns
the existing typed busy/denied result. It does not recursively invoke Wasm.
Read-only UI/status rendering may continue from snapshots; it may not borrow
the active Store.

### Serial commands

Because start returns pending and control reaches the main loop, the existing
serial parser continues to run. Ordinary read-only commands remain answerable.
Commands that touch the active service or its resources must either:

- invalidate its service/kill generation and wait for the normal teardown
  receipt before applying an already-authorized core action; or
- return an explicit `resource_busy_active_invocation` denial.

They must not directly drop the continuation from inside protocol dispatch.
No second command may resume the guest or inject a host result.

### Recovery lifeline

`recovery.snapshot` and `recovery.lifeline_table` remain immediately answerable
while the test invocation is suspended and must report that they route through
neither Wasm nor the provider. An authorized disable/revocation of the active
service changes its generation; the invocation task observes that before any
resume and tears it down. Other recovery actions remain governed by their
existing evaluators.

The recovery core never waits for the service, its peer, the singleton socket,
or the continuation. SAFE/recovery posture continues to deny starting a
beyond-env invocation.

### Singleton TCP interaction

NET-2 contains no network lease. Its always-suspending test import owns only a
test pending-operation token.

NET-3 keeps the parent singleton authority model: one owner/generation lease
shared by native OpenAI and future Wasm acquisition. The execution recut only
changes how a future Wasm owner waits. A second owner still gets
`resource_busy`; it cannot inspect, resume, close, abort, or reuse the active
owner's socket. Lease revocation changes the resource generation and causes
the invocation task to terminate before resume.

## Recut slices 2-4

Slices 1 and 5 onward retain their parent-scope meaning. Revised slices 2-4
replace the parent versions below.

### NET-2: invocation authority, resumable main-loop task, F12 and teardown

Capability sentence: a built-in real-wasmi test invocation can suspend at a
host import, leave serial/recovery/input responsive, and be physically killed
with exactly-once teardown before resume.

Production authority remains false. Add:

- core invocation/service/instance ids and handle-generation state;
- the input-owned secure-attention generation increment/read;
- a main-loop-owned `ActiveBeyondEnvInvocation` lifecycle;
- `call_resumable`, strict custom-host-error classification, typed resume, and
  cumulative fuel handling;
- wall and pump-step budgets;
- one idempotent teardown plus Drop guard; and
- a built-in, clearly labeled test-only Wasm fixture whose `test.suspend_once`
  import always records a pending operation and returns `HostSuspend`. It has
  no network, crypto, secret, acquisition, persistence, or production ABI
  authority.

Likely write set:

```text
raios-core/src/beyond_env_invocation.rs       pure lifecycle/owner/budget decisions + host tests
raios-core/src/lib.rs                         module export only
seed-kernel/src/input.rs                      F12 generation at the existing decode point
seed-kernel/src/wasm_runtime.rs               vendored wasmi continuation/store/teardown path
seed-kernel/src/main.rs                       active-task ownership and scheduler ordering
seed-kernel/src/agent_protocol_wasm.rs        test-only start/status evidence command
vm-harness/shadow-vm-smoke.ps1                focused-profile registration
vm-harness/shadow-vm-smoke-profile-m11-beyond-env-lifecycle.ps1
```

Do not create a guest crate merely for the fixture; a tiny pinned Wasm byte
fixture beside the existing trap/fuel fixtures is sufficient.

Host tests cover start/suspend/resume/finish, custom marker mismatch,
non-suspend host error terminality, normal/trap/OutOfFuel/kill/abandon exits,
generation invalidation before resume, wall/step exhaustion, and exactly-once
teardown with stale handles denied. The exact wasmi stack retention is proved
in the kernel focused profile because the host-testable core model does not
pretend to execute wasmi.

Focused profile: `m11-beyond-env-lifecycle`. Required QEMU predicates:

1. test fixture reaches a real `ResumableCall::Resumable` and emits
   `suspended=true`, with no network/secret/acquisition/durable effect;
2. `agent recovery.snapshot` and `agent recovery.lifeline_table` answer while
   the continuation is retained;
3. the harness injects physical input through the existing QEMU monitor helper,
   `Send-QemuMonitorCommand -Command "sendkey f12 60"`, never through serial;
4. the killed marker arrives within 250 ms of monitor acknowledgement and says
   no resume after kill, teardown complete, handles invalid, no lease held,
   pending acquisition absent, and prior candidate unchanged;
5. a second fixture run succeeds after the kill, proving task/handle release;
6. normal return, guest trap, unrecognized host trap, host fuel error, Wasm
   `OutOfFuel`, and abandoned-task probes all share exactly-once cleanup;
7. the maximum-fuel busy-loop exits within 250 ms; and
8. existing `m8-lifeline` trap/fuel/recovery predicates remain included.

The profile must timestamp guest markers from the same boot-relative basis;
the host also measures HMP-to-marker elapsed time. A late F12 pass after a
timeout is a failure.

### NET-3: singleton transport ownership with non-blocking progress

Capability sentence: native OpenAI and a future resumable Wasm invocation see
one generation-checked TCP lease whose owner alone can progress or release it,
without adding a Wasm-linkable import.

The parent authority, source binding, and busy/owner-only-close semantics are
unchanged. Recut the implementation surface so it exposes one-step,
non-blocking connect/send/receive inspection to the main-loop invocation task;
no lease method contains a sleep or retry loop. Move native OpenAI onto the
same lease as already planned. Native OpenAI may retain its current calling
shape only if its bounded loops release to the main loop; otherwise it is
separately flagged as existing debt and may not be used as proof of the W7
guarantee.

Likely write set:

```text
raios-core/src/transport_lease.rs              owner/generation transition tests if not already present
raios-core/src/lib.rs                          module export only if needed
seed-kernel/src/net.rs                         singleton lease + one-step socket operations
seed-kernel/src/openai.rs / tls_io.rs           native owner token and loop removal where required
vm-harness/shadow-vm-smoke-profile-m11-net-imports.ps1
vm-harness/shadow-vm-smoke.ps1                 profile registration if new
```

Host tests cover exclusive claim, same-owner progress, foreign/stale
send/receive/close/abort denial, timeout/revocation release, generation reuse,
and idempotent owner teardown. Focused `m11-net-imports -Network` proves native
owner success, both-direction busy denial, owner-only abort, timeout release,
and successful retry. No Wasm net import is linked.

### NET-4: `net.*` shims as suspension points, still ungrantable

Capability sentence: direct runtime tests can drive pre-bound TCP open/send/
receive through real wasmi suspension/resume boundaries while every production
beyond-env grant remains denied before instantiation.

Implement the four parent-scope shims with these amendments:

- `tcp_open`, `tcp_send`, and `tcp_recv` always create exactly one pending
  operation and return the typed `HostSuspend` error;
- only the main-loop invocation task may turn that operation into the import's
  `i32` result and call `resume`;
- `tcp_close` is immediate and idempotent for the current owner;
- `would_block` never reaches W7;
- every pending buffer/guest destination is fixed-capacity and owner/id bound;
- kill, quota, deadline, lease generation, posture, and service generation are
  checked before progress and before resume; and
- all production evaluator inputs still carry
  `policy_allows_beyond_env:false`.

Likely write set:

```text
seed-kernel/src/wasm_runtime.rs                net host closures + pending operation/resume mapping
seed-kernel/src/main.rs                        invocation progress dispatch if a net arm is additive
seed-kernel/src/net.rs                         only if NET-3's one-step API needs a narrow consumer hook
seed-kernel/src/agent_protocol_wasm.rs         direct test-only shim evidence
vm-harness/shadow-vm-smoke-profile-m11-net-imports.ps1
vm-harness/shadow-vm-smoke-profile-m11-wasm-import-grant.ps1  negative assertions if housed there
```

Host tests cover pointer/length/handle/quota validation before suspension,
send-buffer copy isolation, receive write-before-resume, partial send, EOF,
timeout, peer close, kill-before-ready, kill-after-ready-before-resume,
foreign/stale close denial, and custom-marker/result-type mismatch. Focused
`m11-net-imports -Network` proves real suspend/resume against a silent and a
responsive fixture, F12 kill before timeout, lease cleanup/retry, and no guest
`would_block` result. The profile also proves a signed module requesting each
`net.*` import remains denied before instantiation and no production shim is
called. Include the NET-1 exact-list/linker-drift negatives from
`m11-wasm-import-grant`.

## What does not change from the parent scope

The following remain authoritative without amendment:

- exact `raios.host_imports.v1` ABI id, ordered import-list hash, and current
  signatures;
- permanent-core versus Wasm authority split;
- evidence-bound evaluator, observed-import equality, and per-instance linker;
- `policy_allows_beyond_env:false` throughout grants-nothing slices;
- exact W7 service/artifact/generation/list/source binding at the later arming
  slice;
- explicit owner approval for the arming diff;
- fixed source policy and no caller-controlled destination/SNI/path/header;
- one TCP handle, one TLS session, one pending acquisition, and one singleton
  transport lease;
- 5-second connect, 15-second idle, 90-second total, 32 KiB TX, 320 KiB RX,
  4 KiB network-call, 16 KiB TLS-record/hash, 256 KiB artifact, four-chunk, and
  2 MiB Wasm-memory ceilings unless the owner changes them before arming;
- opaque crypto-key/session custody and core-owned trust labels;
- M12+ chunk/finalize convergence and prior-candidate preservation;
- SAFE/recovery start denial, no automatic restart/resume, and no native W7
  fallback;
- arming ladder and later slices 5-9; and
- all persistence, install, load, execution, secret, provider, WebPKI, and
  trusted-time denials named by the parent scope.

The reserved `-6 would_block` ABI value remains defined, but the amended W7
network contract never returns it.

## Risks and stop conditions

Primary risks introduced or made explicit by this recut:

- wasmi considers all normal host-call errors resumable by origin; accepting
  anything except the typed marker would turn a trap into forged completion;
- `ResumableInvocation` does not own raiOS resources, so relying on its Drop
  would leak the singleton lease or sensitive state;
- the tail-call host-error edge is terminal;
- the Store and continuation must have exactly one main-loop owner; putting
  either behind reentrant protocol locks risks deadlock or aliasing;
- async start/completion must retain one invocation id so serial consumers do
  not confuse pending acknowledgment with success;
- wasmi fuel is a terminal budget, not a resumable scheduler quantum, so guest
  loops remain cooperatively bounded rather than preemptible; and
- a supposedly non-blocking driver/helper can silently reintroduce the exact
  F12 stall this addendum removes.

In addition to every parent stop condition, stop and return to the owner if:

- the implementation needs a wasmi patch, upgrade, epoch API, or fuel-trap
  resume;
- the Store/continuation cannot be retained in the main-loop owner without
  unsafe global aliasing or a lock held across `resume`;
- an ordinary host trap, host `OutOfFuel`, mismatched marker, or tail-call edge
  is resumed;
- any network import returns guest-visible `would_block` or performs a blocking
  retry loop;
- F12 after a silent-peer suspension misses the 250 ms QEMU bound, is observed
  only after the connect/idle timeout, or permits any later resume;
- the maximum-fuel busy-loop misses its 250 ms terminal bound;
- a synchronous host primitive exceeds 25 ms and is not recut into a scheduled
  bounded operation;
- recovery snapshot/lifeline-table serial requests cannot complete while the
  test continuation is suspended;
- teardown is not exactly once on every terminal route, or retry finds any
  stale task/handle/lease/pending acquisition;
- NET-3 lets native OpenAI and the test/future Wasm owner inspect, abort, or
  reuse each other's socket; or
- any grants-nothing profile observes a production beyond-env linker call.

## Owner decisions

No new owner decision is required. Parent decision 5 is resolved by this
verified enabling fact and addendum: use main-loop-integrated resumable
execution before implementing `net.*`. Parent decisions 6 and 7 remain where
they were: explicit W7 arming approval later, and ADR 0012 reconciliation only
when a provider service is ready.
