# 1. Decision

**GO for B3A-2 as `RAIOS_LANG_V1`, called `rlang`: a bounded, deterministic, loop-free expression
language compiled in-system to wasm32 through the existing `raios-wasm-ir` encoder.** Do not start a
Rust subset, general RUIP compiler, filesystem ABI, linker, optimizer, or larger compiler guest.

Capability sentence: an owner can place one inspectable `main.rl` revision in raiOS, have a signed
compiler guest turn it into canonical Wasm twice, and run it after the existing physical W5 click.

The assembler foundation is VERIFIED-CLOSED: encoder, literal goldens, signed guest, same-boot double
build, kernel recompute, W5 run, negatives, and zero drift are recorded in `docs/plans/b3-impl-notes.md`.
The substrate remains wasmi 0.31.2, the 4 KiB buffer ABI, 2-4 MiB guest classes, and
250,000-1,000,000 fuel (`docs/plans/b3-plan.md`, `seed-kernel/src/wasm_runtime/envelope.rs`, and
`raios-core/src/project_runtime.rs`).

**No loops in V1.** Fuel limits damage but does not make loop intent, compiler semantics, or build cost
easy to recompute. Loop-free V1 already proves locals, arithmetic, types, and control flow. Loops wait
for a real need, a static iteration cap, and a measured fuel rule.
This is the controlled language-growth path in `docs/VISION_PLAN.md` B3.3, not a replacement toolchain.

# 2. `RAIOS_LANG_V1` source contract

The source path is exactly `main.rl`; its media type is `text/raios-lang`; the first line is exactly
`RAIOS_LANG_V1`. `raios-core/src/project_workspace.rs::agent_source_media_type` gains only this exact
root-path mapping, following the existing `main.rwir` precedent.

Example:

```text
RAIOS_LANG_V1
fn raios_service_main() -> i32 {
  let answer = 6 * 7;
  if answer == 42 { 31415 } else { 0 }
}
```

Exact lexical rules:

- The complete source is 1-4096 bytes, ASCII only, and ends with one LF.
- The header occupies the first line with no surrounding whitespace.
- After the header, only ASCII space and LF are whitespace; CR, tab, comments, strings, escapes, and
  every other control byte are rejected.
- Adjacent word-like tokens require whitespace; punctuation delimits tokens; `->`, `==`, `!=`, `<=`,
  and `>=` use maximal munch.
- An identifier is `[a-z][a-z0-9_]{0,31}`; keywords are reserved.
- An integer is canonical signed decimal: `0`, `[1-9][0-9]*`, or `-[1-9][0-9]*`, within the i32
  range. `+1`, `01`, and `-0` are invalid.
- `-` plus adjacent digits is a signed literal only where a primary is expected; otherwise it is subtraction.
- The closing function brace is followed immediately by the final LF and EOF.

Exact grammar, from lowest to highest precedence where shown:

```text
source       := "RAIOS_LANG_V1" LF function LF EOF
function     := "fn" WS "raios_service_main" "(" ")" WS "->" WS "i32" WS function_body
function_body:= "{" WS* let_stmt* expr WS* "}"
let_stmt     := "let" WS identifier WS* "=" WS* expr WS* ";" WS*
expr         := if_expr | equality
if_expr      := "if" WS expr WS* branch WS "else" WS branch
branch       := "{" WS* expr WS* "}"
equality     := relation (("==" | "!=") relation)?
relation     := additive (("<" | "<=" | ">" | ">=") additive)?
additive     := multiply (("+" | "-") multiply)*
multiply     := primary (("*" | "/") primary)*
primary      := integer | identifier | "(" WS* expr WS* ")"
WS           := one or more ASCII space or LF bytes
```

Comparison chaining is invalid. There is one zero-parameter function, one fixed export, at most 32
immutable `let` bindings, 512 tokens, 256 expression nodes, and depth 16. Bindings are visible only
after declaration and cannot shadow; branches contain one expression and no declarations.

V1 has no parameters, extra functions, calls, imports, memory, data, strings, arrays, structs,
mutation, early return, loop, recursion, or I/O.

# 3. Types and arithmetic policy

The source has two static types: `i32` and `bool`. Both lower to Wasm i32; there is no conversion.

- Integer literals and `+ - * /` require and produce `i32`.
- `== != < <= > >=` require two `i32` operands and produce `bool`.
- `if` requires `bool`; its branches must have the same type.
- A `let` binding infers and retains its initializer type.
- The function's final expression must be `i32`.
- Evaluation is left-to-right. V1 has no optimization; it emits the accepted AST as written.
- `+`, `-`, and `*` wrap modulo 2^32, exactly matching Wasm i32 operations.
- `/` is signed division toward zero. Closed-program static evaluation rejects zero and
  `i32::MIN / -1` divisors in every branch, including dead branches, so accepted V1 cannot trap.

Checks run in this order: size, ASCII, terminator/header, lexing, parse/quotas, binding/types,
arithmetic safety, then backend/output.

| Reason | Exact condition |
|---|---|
| `oversize_source` | input exceeds 4096 bytes |
| `non_ascii_source` | any byte has its high bit set |
| `missing_terminator` | final LF is absent; missing `;`, `)`, or `}` is `syntax_error` |
| `wrong_version_line` | first line is not exactly `RAIOS_LANG_V1` |
| `invalid_character` | forbidden ASCII whitespace/control/punctuation occurs |
| `non_canonical_integer_text` | spelling or i32 range is invalid |
| `token_quota_exceeded` | more than 512 tokens are required |
| `syntax_error` | tokens do not match the grammar or comparison is chained |
| `quota_exceeded` | let, AST-node, or depth cap is exceeded |
| `duplicate_binding` | a name is declared twice |
| `unknown_binding` | a name is read before a matching declaration |
| `type_mismatch` | an operator, condition, branch, or final result has the wrong type |
| `division_by_zero` | a divisor statically evaluates to zero |
| `division_overflow` | a division statically evaluates `i32::MIN / -1` |
| `oversize_output` | canonical Wasm would exceed 4096 bytes |
| `backend_error` | the typed encoder rejects the compiler-generated sequence |

Every reason gets a negative host vector; overlaps pin the precedence above. Guest errors are
`error:<reason>`; `error:input_abi` covers negative/mismatched reads. Errors contain no source text.

# 4. Compiler and encoder architecture

Add `raios-lang` (`raios_lang`), a PC-testable `no_std`, no-allocator crate with only the workspace
`raios-wasm-ir` dependency. Fixed arrays hold tokens, AST, bindings, and instructions. Its operation is
`compile(&[u8]) -> Result<WasmModuleBytes, &'static str>`: parse -> bind/typecheck/static arithmetic
validation -> emit. The kernel calls this crate rather than copying the parser.

Extend `raios-wasm-ir` with one bounded typed function emitter reusing its buffer, shortest LEB writers,
and sections (`raios-wasm-ir/src/lib.rs`). Add no text version: `RAIOS_WASM_IR_V1`, `assemble`, reasons,
and literal goldens remain frozen. The emitter is backend surface, not `RAIOS_WASM_IR_V2` or assembler input.

The typed body needs exactly these instructions and canonical encodings:

| Typed instruction | Wasm bytes after operands |
|---|---|
| `I32Const(v)` | `41` + shortest signed i32 LEB128 |
| `LocalGet(i)` / `LocalSet(i)` | `20` / `21` + shortest unsigned index LEB128 |
| `I32Eq` / `I32Ne` | `46` / `47` |
| `I32LtS` / `I32GtS` | `48` / `4a` |
| `I32LeS` / `I32GeS` | `4c` / `4e` |
| `I32Add` / `I32Sub` / `I32Mul` / `I32DivS` | `6a` / `6b` / `6c` / `6d` |
| `IfI32` / `Else` / `End` | `04 7f` / `05` / `0b` |

There are no `block`, `loop`, `br`, call, memory, or import instructions. Section order is
magic/version, type, function, export, code; type is `() -> i32`, export is `raios_service_main`, and
locals are `00` or `01 <count:u32leb> 7f`. The encoder appends return/end `0f 0b`, uses shortest
section/body lengths, and emits no custom, name, import, memory, table, start, element, or data section.

At 256 nodes and 32 locals, six bytes per node plus two per `local.set` keeps the designed worst case
below 1800 bytes. The 4096-byte cap remains authoritative; a worst-shape host test pins the bound.

Host tests contain three independently hand-written Wasm arrays and results: precedence returning 42;
the `let`/comparison/if demo returning 31415; and wrapping signed arithmetic plus division. They compare
bytes, validate/run with wasmi 0.31.2, cover every reason, and pin local/body/LEB boundaries. The
compiler and emitter may not generate their own goldens.

# 5. Guest and kernel integration

Create a separate signed `svc-build-compiler` guest (`svc.build.compiler`); do not re-sign the proven
assembler. It uses the same wrapper, links only `env.input_len`, `env.input_read`, and
`env.output_write`, calls `raios_lang::compile`, and returns at most 4096 bytes.

Use the 2 MiB buffer class and 1,000,000 fuel as caps to verify, not a new Bauplatz class. Give the
artifact its own descriptor, signature, attestation, and hashes. `build.compile_probe` proves signed
execution, guest/kernel byte identity, Wasm validity, zero imports, exact export, and inertness.

Add parallel `build.compile_revision`; do not mode-switch `build.assemble_revision`. It requires the
current B2 revision's one `main.rl` with exact media/classification/hash bindings, runs two fresh
same-boot compiler instances, and calls `raios_lang::compile` in the kernel. Success replaces the
shared current-boot output; denial preserves it.

Generalize the existing RAM output slot instead of copying it; `build.run_prepare` stays the W5 route
(`seed-kernel/src/agent_protocol_build_assemble.rs`, `seed-kernel/src/workspace_candidate_service.rs`).
Its binding gains a source-kind discriminator: assembler keeps its exact challenge bytes and
`main.rwir` recheck; compiler uses `raios.workspace_compiled_run_challenge.v1`, binds `main.rl` plus
`text/raios-lang`, and rechecks at the click. Cross-kind replay denies.

The route uses existing `raios.agent.v0` `body.result`, not a new schema. It reports exact source,
compiler, output, double-build/fuel, recompute, validation, export/import, and inert facts. Denials cover
revision or `main.rl` binding, compiler reason, guest failure, mismatches, invalid/importing output,
wrong export, and second-build mismatch.

# 6. Proof shape and independent check

The proof mirrors B3A-1c: B2 fixture -> two independent instances -> byte identity -> kernel recompute
-> literal host golden -> wasmi validation -> inert W5 preview -> serial denial -> one physical click
-> expected return -> stale denial -> negative table -> zero executable/durable drift.
That keeps the proof-before-claim, physical-approval, recomputation, and named-gap rules in
`docs/VISION_PLAN.md` section 2.

The same-library kernel recompute detects corruption, ABI errors, nondeterminism, and transport drift,
not compiler logic errors. The independent check is three hand-written host arrays plus expected
results; it covers only those programs and encoding boundaries, not every source or a shared compiler
bug. Require a differential compiler or independent semantics checker before loops, imports, state,
or broader types; do not build it in B3A-2.

No W6 preview, install, ARTSTOR/RECLOG write, autoload, network, secret, provider,
rollback, new UI, or alternate loader path is added. The honest double build is
two fresh instances in one boot, not a cross-reboot fresh-store proof.

# 7. Worker-sized slices

1. **B3A-2a — encoder + language crate.** Add the additive typed emitter and
   `raios-lang` together; separating them would leave an unused backend slice.
   Predicates: old IR goldens unchanged; three rlang literal goldens exact and
   runnable; every static reason covered; quota/LEB/worst-output bounds pinned.
2. **B3A-2b — signed compiler guest + inert probe.** Add the wrapper, artifact,
   descriptor/attestation, runtime entry, and `build.compile_probe`.
   Predicates: exact three imports; guest/kernel/golden bytes agree; valid
   zero-import module; compiler error survives ABI; zero executable effect.
3. **B3A-2c — revision and W5 route.** Add `main.rl` mapping/fixture,
   `build.compile_revision`, shared produced-output kind, compiled challenge,
   and reuse `build.run_prepare` plus the existing pointer handler.
   Predicates: exact source binding; two builds agree; old assembler challenge
   and responses do not drift; serial/API cannot approve; click is stale-checked.
4. **B3A-2d — focused `build-compile` profile.** Predicates:
   `fixture-revision-committed`, `revision-compiled-deterministic`,
   `host-golden-match`, `w5-preview-bound`, `serial-approve-denied`,
   `physical-click-runs-31415`, `second-activation-stale-denied`,
   `negative-table`, and `zero-executable-drift`.

Workers write their packet and host tests; the orchestrator builds/signs/packages
and supplies observed profile evidence. No worker claims VM proof it did not run.

# 8. Risks and cheapest resolving experiments

| Risk | Cheapest honest experiment |
|---|---|
| New instruction encoding is non-canonical | Compare three literal byte arrays, including local/body lengths across 127/128, then parse and execute with wasmi |
| Refactoring the encoder changes B3A-1 | Run every existing `RAIOS_WASM_IR_V1` literal golden byte-for-byte; stop on one changed byte |
| Compiler guest exceeds the existing class | **ASSUMPTION-TO-VERIFY:** record artifact size, peak guest memory, and fuel for the three goldens plus a max-quota source; change no cap before measurement |
| 4 KiB source/output is too small | Generate the max-node/max-literal valid program and pin encoded length; deny oversize rather than truncate |
| Arithmetic checks disagree with Wasm | Run boundary vectors for wrapping, signed compare, zero division, and `i32::MIN / -1` against wasmi |
| Same compiler miscompiles in guest and kernel | Keep literal host goldens as the independent leg; do not generalize the correctness claim beyond them |
| Source-kind challenge replay | Attempt `main.rwir`/`main.rl` hash/path/media swaps and require stale-binding denial with no run |
| Loops make fuel the language semantics | Keep loops syntactically denied; later measure one statically bounded construct before defining a new version |

The buffer ABI remains correct for V1 because both source and output are capped
at 4096 bytes. ARTSTOR/immutable-handle input replaces it only when a measured
real program needs multi-file input or output beyond that cap; raising the
buffer or exposing a filesystem is not a B3A-2 fallback.

# 9. First owner-visible demo and exit

The fixture program computes `6 * 7`, checks that it equals 42, and returns
31415 on success or 0 on failure. The owner sees the existing W5 preview,
clicks the existing Genesis approval target, and sees the existing run outcome
and `return_value_i32_bits=31415`. There is no new screen or control.

B3A-2 closes only when the focused report observes all nine predicates and the
B3A-1 assembler path remains byte-identical. Until then the honest label is
implemented or partial, never proven. V2 loops and broader language features
remain absent by design, not silently emulated.
