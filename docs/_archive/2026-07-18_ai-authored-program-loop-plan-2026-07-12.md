# AI-authored program loop plan (2026-07-12)

## Outcome

Capability sentence: a user can ask raiOS for a small interactive program,
inspect and test the exact candidate, approve it physically, and run or roll it
back without giving the provider authority over the machine.

The first acceptance program is a complete offline signed-integer calculator:
digits, clear, `+`, `-`, `*`, `/`, equals, keyboard, and pointer. The mechanism
must then accept another bounded UI program without adding calculator code to
the evaluator.

## Current implementation truth

- The signed `svc.user.shell` engine already runs through a fresh, metered
  Wasmi instance for each invocation. Its six `ui.*` imports, RCTX/RINP input,
  RFRM output validation, secure-attention exit, and core-owned focus remain the
  execution boundary.
- `raios-core::ui_program` is the new host-testable program-data contract. It
  defines widgets, key/pointer events, bounded numeric state, guarded rules,
  generic arithmetic actions, canonical `RUIP` V1 bytes, and SHA-256 identity.
  It grants no capability and owns no persistence.
- The calculator is data built with that generic contract; the evaluator has
  no calculator branch.
- Provider/serial workspace integration, VM evidence, physical approval,
  activation, and rollback are not yet proven merely because the contract
  exists.

## Final architecture

```text
user request
  -> provider or serial workspace (candidate author only)
  -> bounded RAIOS_UI_SPEC_V1 authoring data (provider) or exact RUIP_BASE64 (serial)
  -> typed local compile/parse + Program::new + canonical RUIP/hash
  -> inert candidate workspace
  -> Shadow/VM tests and human-readable evidence
  -> physical owner approval of exact hash
  -> existing signed svc.user.shell engine + core-owned ProgramState
  -> fresh Wasmi invocation for each event, validated RFRM only
  -> evidence-bound promotion transaction
  -> previous approved hash remains the rollback target
```

`RUIP` is data interpreted by the trusted bounded engine, not new native code,
Wasm, a loader extension, or authority. `ProgramState` stays in the core across
fresh Wasmi instances. Program identity is SHA-256 over the exact canonical
RUIP bytes. Any parse, limit, reference, arithmetic, hash, evidence, approval,
or frame failure leaves both active program and state unchanged.

The workspace accepts two deliberately distinct authoring forms that converge
before identity or authority:

```text
provider: RAIOS_UI_SPEC_V1 followed by the bounded typed line grammar
serial/test: RUIP_BASE64:<canonical-RUIP-bytes-as-base64>
```

No surrounding prose is part of either candidate. The text form is parsed into
the existing typed `Program`; the binary form must canonical-round-trip exactly.
Both produce the same canonical RUIP identity boundary. The
workspace records the request, candidate bytes/hash, validation result, test
report IDs, requested UI-only surface, approval, active hash, and rollback
hash. Secrets and raw provider context are never fields in RUIP.

## Frozen contract and limits

- ABI: `RUIP` V1, little-endian, canonical re-encode must equal input.
- Maximum encoded program: 16 KiB.
- Maximums: 16 state slots, 64 widgets, 64 key bindings, 128 rules,
  2 conditions per rule, 8 actions per rule, 64-byte widget text,
  2 KiB total text, event IDs 1 through 64.
- Widgets: static text, numeric state display, button rectangle.
- Input: exact key/modifier binding or pointer click in one non-overlapping
  button.
- Conditions: numeric equality or inequality.
- Actions: set, copy, checked add/subtract/multiply/divide, and checked
  multiply-add. Dispatch is atomic; overflow, divide-by-zero, ambiguous input,
  or ambiguous rule commits nothing.
- No RUIP field can name an import, capability, address, file, network target,
  provider, secret, recovery action, persistence target, or raw bytecode.

## Trust boundaries

1. The provider and serial user may propose bytes; neither authorizes them.
2. `ui_program_spec::parse` and `Program::parse` are the two untrusted-input
   boundaries. Both end in `Program::new`, which rejects bad references,
   overlaps, duplicate bindings and every exceeded limit; binary RUIP also
   rejects unknown opcodes, reserved/padding bytes and trailing bytes.
3. Canonical RUIP SHA-256 is the identity used by evidence, approval,
   activation, and rollback. A hash over decoded fields or provider text is not
   sufficient.
4. Core-owned state and event routing are not writable by provider output or
   guest pointers. Rule evaluation is checked and atomic.
5. The existing signed shell engine remains the only executable artifact. A
   RUIP candidate cannot add an import or select another loader.
6. RFRM remains the final display boundary. The existing validator must accept
   the whole frame before anything is drawn.
7. Promotion requires evidence for the exact RUIP hash and an on-device
   physical approval. Provider text, a passing parser, or a rendered preview is
   never approval.
8. Durable activation occurs only through the existing promotion/audit and
   rollback transaction architecture. Until those exact gates are wired, the
   program is honestly `current_boot` and return-to-Genesis is not claimed as
   durable rollback.

## Agent-native disjoint lanes

Lanes may run together only while they own disjoint files:

| Lane | Ownership | Deliverable |
| --- | --- | --- |
| Core contract | `raios-core/src/ui_program.rs`, `ui_program_spec.rs`, minimal `lib.rs` exports | Canonical parser/evaluator, bounded text compiler, calculator instance, host tests |
| App engine | signed-shell runtime/invocation files only | Core state retained across fresh Wasmi events; unchanged six imports |
| Workspace ingress | provider/serial workspace files only | Bounded provider text or exact serial RUIP, shared canonical identity, inert candidate/hash |
| Evidence | VM profile/harness files only | Positive calculator plus malformed, oversized, ambiguity, hash and state-atomicity negatives |
| Documentation | status, roadmap, dashboard, this plan | Honest cursor, evidence filenames, remaining denials |

Each implementation lane returns one integrated capability slice and its
smallest relevant check. The orchestrator alone joins lanes, reads the full
diff, runs the combined focused profile, updates the dashboard, and commits.

## Execution gates

### G1 - contract

- Host tests pin canonical round-trip and deterministic SHA-256 identity.
- Calculator proves all four operations, multi-digit entry, clear, keyboard,
  pointer, divide-by-zero atomicity, malformed/oversized/unknown-op rejection,
  and exact limits.
- `cargo fmt --all -- --check`, focused `raios-core` tests, line-size check,
  diff check, and secret scan pass.

### G2 - current-boot calculator

- A checked-in calculator RUIP is parsed and hash-bound before activation.
- Keyboard and pointer drive the same typed events; core state survives fresh
  Wasmi invocations; F12 still exits through core secure attention.
- The invocation exposes exactly the existing six imports. No new import,
  loader path, or authority appears.
- A focused VM profile captures calculator input/result, canonical hash,
  fresh-instance evidence, RFRM acceptance, and return to Genesis.

### G3 - shared authoring workspace

- Serial paste accepts only exact `RUIP_BASE64:`; provider responses accept only
  the typed `RAIOS_UI_SPEC_V1` grammar. Both produce canonical RUIP and one hash.
- Invalid base64, prose, wrong ABI, noncanonical encoding, limit excess,
  unknown opcode, and hash substitution remain inert with no state change.
- Provider output is candidate data only; no automatic activation follows a
  successful response.

### G4 - evidence and approval

- The Shadow/VM report binds candidate hash, tested event vectors, observed
  state/frame results, engine artifact/descriptor identity, unchanged imports,
  and all fail-closed negatives.
- Genesis shows request summary, program hash, UI-only scope, evidence status,
  and previous rollback hash before approval.
- Only a physical on-device confirmation approves the exact hash. Cancel,
  timeout, reboot, different bytes, or different hash denies activation.

### G5 - promotion and rollback

- Activation uses the existing evidence-gated promotion transaction; it does
  not create a parallel program store or audit log.
- The previous approved program remains addressable by exact hash. A rollback
  transaction restores it and VM evidence proves the calculator replacement
  and restoration paths.
- Recovery can ignore all personal programs and reach Genesis. Program bytes
  never become part of the immutable recovery core.
- Full/recovery profiles and secret scan pass before release or USB handoff.

## Explicit denials and non-goals

- No new Wasm import, syscall, loader, native executable format, compiler,
  package manager, network capability, filesystem capability, provider
  capability, private signing key, embedded secret, or recovery authority.
- No provider auto-load, provider auto-approval, unsigned engine replacement,
  broad mutation, or silent fallback from invalid RUIP.
- No new persistence mechanism in the contract/current-boot slices. Durable
  program installation waits for the existing audited promotion/rollback path.
- No arbitrary source language or Turing-complete bytecode in RUIP V1. Add an
  operation only when a real second program needs it and the same limits,
  parser, atomicity, and evidence model still hold.
- No claim that an AI can yet build arbitrary software inside raiOS. This plan
  proves bounded interactive UI programs first.

## Honest provider limitation

The first live attempt demonstrated that asking a model to hand-encode binary
RUIP is unreliable and was correctly denied as malformed. The durable repair is
the bounded, typed `RAIOS_UI_SPEC_V1` authoring form compiled locally into the
unchanged canonical RUIP model. It is intentionally not general source code or
Turing-complete: current programs are UI widgets plus checked numeric rules and
have no file, network, secret, persistence or arbitrary import surface.

## Completion evidence

G1-G4 current-boot completion is proven by 415/415 core tests, live same-boot
pinned-provider authoring plus physical exact-hash activation, and focused report
`shadow-20260712-025218-6208.json` (252/252). That report also proves calculator
behavior, malformed preservation, fresh Wasmi events, unchanged imports,
core-owned state, validated frames, F12 recovery access, secure-strip clipping,
and trap/fuel fallback. Temporary key artifacts and the live log were deleted
after hashing. G5 durable install/promotion/previous-hash rollback remains open
and unclaimed; current-boot return to Genesis is not durable rollback.
