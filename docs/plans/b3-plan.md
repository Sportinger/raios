# 1. Decision

**GO for B3, but NO-GO for starting rustc-as-Wasm or a rustc-sized `Bauplatz` guest now.**
Start with a small signed assembler from bounded raiOS Wasm IR to final `.wasm`;
reuse its encoder for a restricted compiler, and keep rustc a later milestone.

Only an on-device compiler closes station 4, and only the complete QEMU loop closes
the base (`docs/VISION_PLAN.md:38-47`, `docs/VISION_PLAN.md:135-137`). W4 remains
scaffolding (`docs/ROADMAP.md:373-384`); NO-GO with alternatives is valid
(`docs/VISION_PLAN.md:163-169`).

Labels: **LOCAL FACT** is inspected here with file:line; **ASSUMPTION-TO-VERIFY**
has an exact host experiment; **ESTIMATE** is never acceptance evidence.

# 2. Fixed local substrate

- **LOCAL FACT:** the kernel uses `wasmi = 0.31.2` with default features off
  (`seed-kernel/Cargo.toml:23`). The vendored crate calls itself a bytecode
  interpreter (`vendor/wasmi-0.31.2/src/engine/mod.rs:97-102`); raiOS has no
  kernel JIT by decision (`docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md:42-60`).
  The separate experimental `wasmi_wasi` crate is not vendored; raiOS links
  explicit per-guest functions, not a filesystem (`vendor/wasmi-0.31.2/README.md:35-63`).
- **LOCAL FACT:** buffer guests get 2 MiB; the workspace guest gets 4 MiB and
  250,000 fuel (`seed-kernel/src/wasm_runtime/envelope.rs:4-9`,
  `raios-core/src/project_runtime.rs:9-15`). Existing fuel ranges from 10,000
  to 1,000,000 (`seed-kernel/src/wasm_runtime/artifacts.rs:17-67`).
- **LOCAL FACT:** the buffer ABI accepts at most 4 KiB input and 4 KiB output
  (`seed-kernel/src/wasm_runtime/envelope.rs:4-7`,
  `seed-kernel/src/wasm_runtime/envelope.rs:850-927`).
- **LOCAL FACT:** checked-in guests are 605-30,426 bytes by file metadata; external
  intake is capped at 262,144 bytes (`seed-kernel/src/module_candidate_intake.rs:7-10`).
  This is a small-artifact/MiB-memory runtime, not a compiler-artifact class.
- **LOCAL FACT:** the kernel heap is 64 MiB (`seed-kernel/src/main.rs:181`), so
  it is the immediate allocation ceiling even though the normal QEMU launcher
  gives the whole OS 512 MiB (`scripts/run-stage0-qemu.ps1:62-67`).
- **LOCAL FACT:** B2 supplies live-provider content-addressed source revisions and
  scoped feedback; compiler/test evidence is missing (`docs/ROADMAP.md:193-224`,
  `docs/PROJECT_STATUS.md:3-16`).
- **LOCAL FACT:** produced artifacts must reuse W5 physical approval and W6
  signed install/reboot/rollback, including ARTSTOR; no second path is needed
  (`docs/PROJECT_STATUS.md:2082-2117`). Results still require double-build,
  byte identity, fingerprints, and rollback (`docs/VISION_PLAN.md:21-30`).

# 3. rustc-as-Wasm assessment

## 3.1 What exists

- **ASSUMPTION-TO-VERIFY (upstream evidence):** `rustc_codegen_cranelift` is a
  maintained nightly-preview backend and a near drop-in native rustc backend,
  with stated SIMD and unwind gaps. Its published platform table is native
  x86-64/AArch64/RISC-V/s390x, not Wasm. Experiment: install a pinned nightly
  preview in an isolated host directory, build one no-dependency crate for every
  claimed target, and retain command, versions, outputs, and hashes.
  Upstream: <https://github.com/rust-lang/rustc_codegen_cranelift>.
- **ASSUMPTION-TO-VERIFY (critical blocker):** Cranelift does not currently emit
  WebAssembly. The upstream request for a Wasm backend remains open with no
  implementation plan. Therefore cg_clif cannot presently be the backend that
  makes raiOS `.wasm` output. Experiment: run pinned cg_clif against
  `--target wasm32-wasip1`; retain the exact unsupported-target result, then
  inspect its supported-ISA table. Upstream:
  <https://github.com/bytecodealliance/wasmtime/issues/2566>.
- **ASSUMPTION-TO-VERIFY:** Rust's `wasm32-wasip1` is a Tier-2 compilation
  target **without host tools**. It supplies target `std`, but this does not mean
  official `rustc` runs on WASI. Threads/process spawn fail; file operations use
  WASI imports; building the target from Rust source needs LLD/`wasm-ld`.
  Experiment: inspect a pinned Rust distribution manifest for host tools, then
  attempt a host-tools bootstrap for `wasm32-wasip1` and retain the first real
  failure. Upstream: <https://doc.rust-lang.org/rustc/platform-support.html> and
  <https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html>.
- **ASSUMPTION-TO-VERIFY (community existence proof):** `rubrc` ports rustc
  with the **LLVM** backend to browser Wasm/WASI and claims `wasm32-wasip1` and
  `x86_64-unknown-linux-musl` output. It is explicitly WIP, slow without threads,
  dependent on a virtual WASI/filesystem layer, and not recommended for
  production. It publishes no raiOS-relevant wasmi-0.31 size/RSS/time result.
  Experiment: pin a rubrc revision/release, hash every compiler/sysroot file,
  compile hello in its documented browser environment, then run the same
  compiler artifact under host wasmi 0.31.2 with a logged import shim.
  Upstream: <https://github.com/oligamiq/rubrc>.

## 3.2 Pieces hidden by the phrase "rustc-as-Wasm"

1. The compiler host needs rustc driver/frontend, a codegen backend, its host
   `std`, allocator, and panic runtime in one runnable Wasm environment.
2. The output target needs a pinned sysroot: at least `core`/`alloc`, and `std`
   plus their `.rlib`/metadata when allowed. These are build inputs in ARTSTOR,
   not trusted ambient files.
3. Final `.wasm` needs a linker. LLVM rustc normally relies on LLD/`wasm-ld`;
   process spawn is unavailable, so the linker must be linked in-process or
   mediated by an explicit bounded host tool.
4. Cargo build scripts and proc macros execute host code. Their process,
   dynamic-library, and file authority is incompatible with the first
   `Bauplatz`; exclude both rather than fake support.
5. A narrow raiOS handle API must replace a general filesystem: immutable source
   and sysroot manifests, bounded reads, isolated scratch writes, final-output
   commit, no path traversal, symlinks, network, secrets, or ambient directories.

Every item is an **ASSUMPTION-TO-VERIFY**: print the artifact's imports, run with
all denied while logging requests, implement only observed imports over a frozen
manifest, and repeat until hello builds or reaches a named blocker.

## 3.3 Size, memory, and time

| Quantity | Honest assessment | Verification experiment |
|---|---|---|
| rustc + LLVM Wasm artifact | **ESTIMATE: 200-800 MiB**; no local artifact exists | Build/pin rubrc, strip nothing first, record each byte length and SHA-256; repeat stripped |
| hypothetical rustc + cg_clif artifact | **ESTIMATE: 100-300 MiB**, but impossible until a Wasm output backend exists | Only measure after cg_clif can emit a hello `.wasm`; never extrapolate from native archives |
| target sysroot/linker inputs | **ESTIMATE: 100-500 MiB** | Enumerate exact files opened by one no-std and one std hello build; total unique content-addressed bytes |
| compile working set | **ESTIMATE: 512 MiB-2 GiB** for LLVM-hosted rustc; wasmi translation adds host-side state | Run under host wasmi 0.31.2; sample peak process/private bytes and guest `memory.size` |
| interpreter slowdown | **ESTIMATE: 10^1-10^2 versus native on the same host**, with high uncertainty for v0.31 | Run identical compiler/hello under native, Wasmtime, and wasmi 0.31.2; record warm/cold medians and fuel |
| hello compile in raiOS QEMU | **ESTIMATE: 10^3 seconds per build; plausible range 10^2-10^4 seconds** | First host wasmi, then a dedicated QEMU build profile; record module-load, compile, link, and fingerprint phases separately |

The time estimate is subordinate to memory: **under today's 64 MiB kernel heap
and 512 MiB whole-VM profile, rustc-as-Wasm is expected not to fit at all.** A
failed allocation has no meaningful compile-time estimate. Wasmi 0.32's
maintainer reports up to 5x over 0.31 and recommends JIT engines for compute-heavy
workloads; that is upstream guidance, not a raiOS measurement. Experiment: keep
0.31.2 as the baseline, then repeat the same artifact on a separately reviewed
wasmi upgrade. Upstream: <https://wasmi-labs.github.io/blog/posts/wasmi-v0.32/>.

# 4. `Bauplatz` candidate classes

All table numbers are **initial ESTIMATES/experiment caps**, not grants. Set the
smallest measured limit; each class stays single-instance, grants-nothing, signed,
and unable to authorize its output.

| Candidate | Artifact / guest memory / fuel | Imports | 512 MiB acceptance VM? |
|---|---|---|---|
| A. bounded IR -> Wasm assembler | <256 KiB / 4-16 MiB / 10^7-10^8 | existing 4 KiB input/output first; later frozen-source read + output commit only if measured necessary | **Yes**, after a small measured guest limit; preferred first proof |
| B. tiny RUIP-extension or Rust-subset compiler | <1 MiB / 16-64 MiB / 10^8-10^10 | immutable source reads, fixed target description, bounded scratch/output; no Cargo/proc macros/build scripts | **Probably**, but may require growing the 64 MiB kernel heap within the existing 512 MiB total |
| C. hybrid on-device verifier | <256 KiB / 4-16 MiB / 10^6-10^8 | source/receipt/candidate reads; no output authority | **Yes**, largely already covered by W4/W5 |
| D. rustc + LLVM Wasm | 200-800 MiB / 512 MiB-2 GiB / 10^11-10^13 | substantial measured WASI subset, sysroot, linker, scratch; proc macros/build scripts initially denied | **No expected fit**; research only in a dedicated 2 GiB-or-larger profile |

A larger `-m` profile is legitimate research evidence but changes the substrate
contract. It must be named `build-acceptance`, must not replace the ordinary
512 MiB run, and cannot close B4 unless the owner explicitly changes the memory
budget and the complete loop passes under that declared budget. Do not widen the
normal guest class or artifact cap merely to admit a speculative rustc bundle.

# 5. Ranked alternatives

## 1. Staged on-device assembler -- GO now

Define the smallest `RAIOS_WASM_IR_V1` needed for one zero-import function,
integer constants, return, exports, and bounded memory/data. A no-dependency,
PC-testable crate parses it and emits canonical Wasm sections and LEB128 directly.
The same crate is compiled into a signed guest. This is a real compiler stage:
the final runnable `.wasm` bytes are produced on-device, not selected from a
fixture. Later candidate B calls the same encoder.

Smallest proof: exact B2 source revision -> assembler twice in fresh stores ->
byte-identical output/fingerprint -> independent kernel `wasmi::Module::new` ->
inert W5 preview -> physical click -> produced `raios_service_main` returns 42.
Stop before W6; a later durable slice uses existing W6 unchanged.

Focused profile sketch: extend the existing project-workspace/W5 profile; do not
create a parallel install harness. Include malformed IR, non-canonical integer,
duplicate section/export, excessive output, fuel exhaustion, stale source, and
different-second-build denials.

## 2. Own minimal compiler -- GO after the assembler

Compile a restricted RUIP extension or tiny Rust-like language directly to Wasm;
small RUIP programs already build fully in-system (`docs/VISION_PLAN.md:130-132`).
Use bounded integer types/control flow and explicit imports. No LLVM, Cranelift,
or build scripts. Implement it as a no-dependency host-testable crate following
the existing parser-relocation pattern, then run that crate as a signed guest.

Smallest proof: `fn main() -> i32 { 42 }` in a B2 revision -> two fresh builds ->
identical Wasm -> kernel validation -> W5 physical run. Reuse the assembler
encoder and the same focused predicates; add type error, overflow policy,
unbounded-control-flow, forbidden import, and source/output quota cases.

## 3. Honest hybrid -- useful stage, NO-GO as the endpoint

Keep W4 clearly labeled `builder_attested_not_local_rebuild`, then recompute
source/dependency/candidate bindings, Wasm validity, import set, and fingerprint
on-device. This improves evidence but does **not** fulfill on-device compilation
(`docs/VISION_PLAN.md:38-46`).

Smallest proof/harness: no new compiler and preferably no new profile. Reuse the
existing W4/W5 focused path (`docs/ROADMAP.md:373-394`); add a predicate only if
one of those exact recomputations is demonstrably absent. Do not rebuild existing
verification under a new B3 name.

## 4. rustc-as-Wasm -- PARK, then reconsider with prerequisites

Reopen only when a pinned compiler artifact demonstrably runs under host wasmi
0.31.2 (or a separately approved successor), emits Wasm, publishes its complete
import/file trace, fits a declared memory ceiling, and completes two identical
hello builds within a declared patience/fuel budget. First scope excludes Cargo,
dependencies, build scripts, proc macros, threads, network, and `std` output.

Its first harness is host-only measurement. Only after that passes should B3.1
add a large `Bauplatz` envelope and a dedicated QEMU `build-acceptance` profile.

# 6. Recommended first worker-sized slice

**B3A-1 capability sentence:** raiOS can turn one bounded, inspectable source IR
revision into a deterministic, validated, runnable Wasm artifact entirely inside
the guest, while the output remains inert until the existing physical W5 gate.

Ownership packet:

- A Codex worker writes one no-dependency assembler crate, one signed guest
  wrapper, the narrow kernel bridge, and focused harness predicates. It does not
  need to build the kernel or claim VM evidence.
- The orchestrator compiles/signs/packages, runs the focused profile, reads the
  full diff, performs the secret scan, and records evidence. This follows the
  established worker/orchestrator boundary from `docs/plans/b2-plan.md`.
- Reuse B2 source revisions, current wasmi validation/import gates, the typed W4
  receipt boundary with an honest on-device builder tier, W5 physical approval,
  M12 signing, and ARTSTOR/W6 only when persistence is later requested.

Focused predicates:

1. exact source revision/tree/input bytes are bound before assembly;
2. assembler descriptor, artifact hash, signature, import list, and limits pass;
3. unknown imports deny before instantiation;
4. two fresh-store builds consume identical input and emit byte-identical output;
5. guest and kernel SHA-256 agree with host recomputation;
6. output parses through the existing wasmi validator and has the exact export;
7. output is inert before approval; serial/API start cannot substitute;
8. a fresh Genesis pointer click is one-shot and stale-checked;
9. the produced function returns 42 within measured memory/fuel;
10. malformed/oversize/out-of-fuel/second-build-mismatch preserve prior state;
11. no network, secret, install, RECLOG, ARTSTOR, autoload, or rollback effect;
12. any later persistence uses the existing separate W6 click and rollback path.

# 7. Risks and cheapest resolving experiments

| Risk / open question | Cheapest honest experiment |
|---|---|
| rubrc is a browser demo, not a raiOS candidate | Pin it; record compiler/sysroot sizes, hashes, imports, one hello result; then run unchanged under host wasmi 0.31.2 |
| cg_clif cannot emit Wasm | Attempt one pinned `wasm32-wasip1` build and retain the exact failure plus supported-ISA evidence |
| compiler module exceeds intake/ARTSTOR or wasmi translation memory | Load only on host wasmi first; record module bytes, translated-store peak, linear-memory peak; no QEMU work before this |
| 512 MiB total / 64 MiB heap is insufficient | Run assembler at 4/8/16 MiB caps; size a heap increase from the first passing peak; rustc uses a separate >=2 GiB research profile |
| fuel maps poorly to patience | Run fixed input at increasing fuel caps, record fuel used and wall time, set the ceiling immediately above observed need |
| file ABI accidentally becomes a filesystem | Trace actual reads/writes; expose immutable content handles and one scratch/output namespace only; fuzz paths, quotas, ordering, and stale handles |
| output is deterministic only accidentally | Fresh-store double-build plus byte diff; vary ambient clock/order and prove neither is observable |
| same-compiler double-build misses correlated miscompilation | For the assembler, independently encode/recompute expected bytes in host harness; later add a second implementation before broad language claims |
| proc macros/build scripts smuggle authority | Keep them syntactically/manifest-denied; later test one of each only under a separately designed executable-build-input policy |
| Wasm32's address ceiling is exceeded | Measure guest `memory.size`; if a hello requires near 4 GiB, declare this rustc route NO-GO rather than adding memory64 silently |
| a larger QEMU profile weakens the base claim | Keep 512 MiB ordinary acceptance green; label larger evidence research-only until an explicit owner budget decision |

# 7.1 MEASURED (orchestrator host experiment 2026-07-18)

The rubrc/rust_wasm route was measured per the section-7 experiments:

- `oligamiq/rust_wasm` v0.3.0-release: 78 assets, **1,928.5 MiB total**; the
  compiler itself is `rustc_opt.wasm.tar.gz` **28.6 MiB compressed / 91.0 MiB
  uncompressed** (sha256 c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024
  cd00687a5791); per-target sysroots are ~33-36 MiB compressed each. The
  artifact-size ESTIMATE row (200-800 MiB) is therefore corrected downward for
  this LLVM-hosted build: ~91 MiB module + >=1 sysroot.
- **Categorical blocker measured:** host `wasmi 0.31.2` `Module::new` rejects
  the module immediately (0.00 s, 43 MiB peak) with
  `threads must be enabled for shared memories (at offset 0x176b)` — the only
  existing rustc-as-Wasm artifact is built against the Wasm threads/shared-
  memory proposal (rubrc's browser_wasi_shim-threads environment). wasmi 0.31
  does not support threads, and raiOS deliberately has no thread/shared-memory
  substrate. No memory/fuel budget changes this: the artifact cannot load at
  all. A threads-free rustc-as-Wasm build does not currently exist upstream;
  producing one is upstream toolchain work, exactly matching the section-5.4
  PARK criteria. The staged ladder (assembler -> restricted compiler) remains
  the only live path.

# 7.2 OWNER GOAL SHARPENING (2026-07-18, binding)

The owner declared the end goal explicitly: agents build AND test large
software (game-scale, an NLE) ON raiOS via Genesis jobs, with NO external
workshop PC. Consequences for this plan:

- rustc-class on-device tooling moves from PARKED-indefinitely to
  REQUIRED-LATER with an ACTIVE bootstrap lane. Bootstrap principle: every
  self-hosting system in history was compiled once from outside; using the
  workshop ONE more time to produce the on-device toolchain artifact is
  consistent with the goal — afterwards raiOS never needs it again.
- Next bootstrap experiment (workshop-side, cheap relative to the goal):
  **ASSUMPTION-TO-VERIFY — a threads-free rustc wasm32 build is producible.**
  rustc runs single-threaded normally; rubrc chose the threads/shared-memory
  environment for speed. Attempt a single-threaded rustc-as-wasm build (or a
  rebuild of rubrc's pipeline without the threads feature) and re-run the
  section-7.1 wasmi Module::new probe on the result. Success turns the
  categorical blocker into a size/speed problem; failure names the exact
  upstream gap.
- The physics stages to the full goal (each owner-visible, same proof
  discipline): rlang ladder (proves the job machinery) -> Bauplatz big-memory
  guest class + storage budgets -> bootstrap toolchain artifact -> FAST
  execution tier (verified AOT/native; a deliberate later evolution of ADR
  0005's interpreter-only stance, needed for both build speed and big-app
  runtime) -> GPU driver via the loop (VISION_PLAN section 6).

# 8. Exit criteria

B3A-1 is GO only on observed focused evidence. rustc remains PARKED until its
host measurement gate passes. B3 overall is NO-GO only if the bounded assembler
cannot produce and independently validate final Wasm inside this architecture;
failure of rustc alone is not failure of on-device build.
