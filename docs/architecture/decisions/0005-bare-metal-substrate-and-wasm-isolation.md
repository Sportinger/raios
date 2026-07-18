# ADR 0005: Bare-Metal Substrate Confirmed, Wasm Service Isolation, Mechanism Before Vocabulary

## Status

Accepted. Owner decision recorded 2026-07-04, following
`docs/_archive/2026-07-18_review-4-deep-scope-code-and-process-2026-07.md`.

## Context

Review 4 identified three unresolved strategic questions that most of the
existing 165k-line codebase implicitly depends on:

1. **Substrate.** Should the agent-protocol / typed-memory / evidence-gated
   promotion layer be proven first as a Linux userspace supervisor (two-track
   option), or does raiOS stay bare-metal only?
2. **Isolation model.** The docs forked three ways:
   `docs/architecture/decisions/invariant-choices.md` (line 62) commits to a single active Wasm
   module; ADR 0002 (line 461) leaves isolation an open question; ADR 0003
   requires a native multi-service graph with versioned state migrators.
   Meanwhile no isolation mechanism exists in code at all — everything runs
   statically linked in one ring-0 address space, so capability grants are
   descriptive, not enforced.
3. **Sequencing.** ~105k lines of capability/evidence vocabulary were built
   before the mechanisms they govern (durable writes, module execution,
   rollback application) exist. Whatever isolation and persistence answers
   land will reshape that vocabulary.

## Decision

### 1. Bare metal only

raiOS remains a bare-metal operating system. The Linux-userspace two-track
option from review 4 (option B) is **rejected** by owner decision. There will
be no Linux system underneath raiOS. The Lisp-machine/Tamagotchi identity —
one bonded machine, a surface small enough for an agent to fully model, an
immutable recovery core — is the product, and it is kept whole.

Consequence accepted knowingly: the evidence-gated promotion loop arrives
later than it would on Linux primitives. To compensate, the milestone order
below pulls mechanism work forward as aggressively as possible.

### 2. Wasm is the first real isolation boundary

The first enforceable capability boundary in raiOS is an **in-kernel
WebAssembly interpreter** (wasmi-class, `no_std`-compatible, interpreter
only — no JIT in the kernel).

- Replaceable services compile to `wasm32-unknown-unknown` and are loaded
  as Wasm modules into interpreter instances owned by the kernel.
- The capability envelope of a service becomes its **host-function import
  surface**: the interpreter only links the imports that the service's
  computed grant includes. A service physically cannot call an authority it
  was not granted — the boundary is enforced by construction, not by policy
  prose.
- Hot-swap = instantiate v2 next to v1, migrate versioned state, switch
  handles; rollback = drop the instance and reinstantiate the previous
  artifact. This makes the existing hot-swap/rollback vocabulary real.
- Interpreter-speed execution is explicitly acceptable for v0 services
  (tools, UI panels, diagnostics, provider adapters). Drivers and
  performance-critical paths stay native and kernel-resident for now.

This supersedes the three-way fork: `invariant-choices.md`'s Wasm choice is
confirmed and generalized to multiple module instances; ADR 0003's native
multi-service graph with separate address spaces remains the **long-term
evolution** (entered only after the Wasm service world demonstrably works),
not the first implementation.

### 3. Mechanism before vocabulary

The sequencing error diagnosed in review 4 is inverted as a standing rule:

- **No new `raios.*.v0` evidence schema may be added** until the milestone
  gates in `docs/_archive/2026-07-18_ROADMAP.md` say otherwise. Denial-gate and schema-only
  slices no longer count as progress.
- The near-term milestone order is: stabilize the red full profile →
  host-testable core library → ceremony collapse onto a typed record model
  → **first durable write** (the `RAIOS_AUDITRB_V0` LBA1 audit/rollback
  transaction append; AHCI write/readback already works, so this is
  authority policy, not driver work) → **first Wasm-isolated service** →
  second-service generality proof → first external artifact through the
  full promotion loop.
- One real transaction append plus one real enforced capability boundary
  convert the existing evidence edifice from description into a functioning
  transaction system, retroactively justifying it.

### 4. Orphaned host-side signing lane is parked

The `distribution/ota/`, `distribution/registry/`, and `distribution/fake-cloud/` host crates (~3,700 lines, frozen
since 2026-05, never connected to the kernel) are **parked, not deleted**.
ADR 0002's local-attestation model remains the trust root for the MVP. A
future slice may revive the lane when external artifact distribution becomes
real; until then no work lands there.

### 5. Truth-in-documentation follow-up

Until milestone M4 (Wasm isolation) lands, README claims of the form
"impossible to exceed capabilities" must be downgraded to design-intent
language ("designed so that…"), per the review-4 credibility finding. The
4-label status rule from `docs/architecture/decisions/invariant-choices.md` applies to README
marquee sections.

## Consequences

- The capability-envelope vocabulary gets a concrete enforcement target (the
  Wasm import surface); envelope schemas can be validated against a real
  boundary instead of speculation.
- The Shadow VM promotion loop becomes achievable on bare metal: candidate
  Wasm artifacts can be executed under the existing QEMU harness, evidence
  recorded, capabilities granted, and the artifact promoted into the live
  system — the project's actual first product milestone.
- The kernel gains one significant new dependency (a Wasm interpreter
  crate). It must be vendored and pinned like embedded-tls, and it lives
  above the permanent core, not inside it.
- ADR 0001's `download_signed_module` tool vocabulary and ADR 0002's
  local-attestation flow are unchanged; this ADR decides *where the module
  runs*, not how it is trusted.

## Non-Goals

- No JIT or native code generation in the kernel.
- No attempt to run drivers in Wasm.
- No removal of the long-term native service graph (ADR 0003) — it is
  deferred, not rejected.
- No resumption of the `distribution/{ota,registry,fake-cloud}` lane without a new ADR.
- No claim that Wasm isolation equals full security review — memory-safety
  of the interpreter boundary still needs its own evidence chain.
