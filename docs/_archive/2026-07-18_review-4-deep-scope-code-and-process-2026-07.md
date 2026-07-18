# Review 4: Deep Scope, Code, and Build-Process Review (2026-07-04)

## How This Review Was Produced

This review was produced by a multi-agent analysis (11 agents): eight
independent readers over the docs, code, git history, and infrastructure,
followed by three senior critics (scope/feasibility, code health, build
process). Every load-bearing claim below was verified directly against files
in this repository — file paths and line numbers are cited so future agents
can re-check them. It follows the same tradition as the May 2026 plan reviews
(`review-1` through `review-3`), which this repo's own process rules say
should be repeated periodically.

Audience: the project owner (a non-programmer directing a coding AI) and the
coding agents themselves.

---

## 1. Executive Summary

**The idea is good. The workmanship is good. The aim is currently wrong.**

raiOS's thesis — *"software generation is no longer scarce; safe system
change is"* — is a genuinely interesting and coherent research direction. The
~17,000 lines of real operating system underneath (Limine/UEFI boot, e1000
networking, USB, AHCI, framebuffer UI, fail-closed pinned TLS to OpenAI) are
legitimate and unusually well-made for AI-generated bare-metal Rust: exactly
**1 `unwrap()` in 165,187 lines**, `unsafe` confined to drivers, honest
failure records, real QEMU boot evidence with matching SHA-256 receipts.

But the project has inverted its own premise. A system pitched as *"small
enough for an agent to fully model"* is now **165k lines, of which ~90% is
governance ceremony** describing authority that has never once been granted:

- 82 of 119 kernel source files are `agent_protocol_*` (~105k lines, 64%).
- The flagship demo service `hello_service.rs` is **22,705 lines and has no
  behavior at all** — no code ever emits a greeting; it is lifecycle and
  rollback *evidence about itself*.
- The kernel defines **535 distinct versioned schema record types**
  (`raios.*.v0`), ~30 of them chained gates that each conclude "all write
  authority still denied."
- The flagship loop of the entire pitch (AI authors a service → Shadow VM
  verifies → capabilities granted → promotion → rollback) **has never
  executed end-to-end for any artifact.**
- **No isolation mechanism exists.** Everything — core, agent host, demo
  service, TLS stack — runs statically linked in one ring-0 address space.
  The capability model is currently string constants and hash chains
  describing a boundary that physically does not exist.

The highest death risk is not technical. It is the **economics of the build
loop itself**: each ceremony stratum makes the next slice slower (compile
time, AI context window, 10-minute QEMU verification), which selects for more
copy-paste ceremony, while owner-visible capability (boot, chat, a demo
counter) has been static for months. The only progress signal is predicate
counts (6789/6789), which measure verification *volume*, not capability.

**The recommended response is not to abandon anything.** It is: freeze
ceremony, collapse it ~10x with a data-model refactor, decide the isolation
model, land the first real durable write, and change the agents'
definition-of-done from "evidence emitted" to "capability delta." Section 5
also presents a strategic option (proving the agent-protocol layer on Linux
in parallel) the owner should explicitly decide on and record as an ADR.

---

## 2. What Is Genuinely Good (Preserve Through Any Pivot)

These are the transferable assets. Do not lose them.

1. **The platform layer (~17k lines).** Limine/UEFI higher-half boot,
   e1000 + smoltcp + DHCP, xHCI USB HID, AHCI with confined `unsafe`, RDRAND
   entropy, double-buffered framebuffer UI, and a genuinely fail-closed TLS
   gate (defaults to `pin_config_missing`; embedded-tls with P-256
   CertificateVerify), honestly documented as pin-only rather than WebPKI.
2. **Micro-level code hygiene.** 1 `unwrap()`, 1 `panic!`, 2 TODOs in 165k
   lines; minimal pinned dependencies; `--locked` builds; vendored TLS;
   stdlib-only Python FAT32 imager.
3. **The receipts culture.** 588 VM reports with per-predicate
   expected/actual records; spot-checked report SHA-256 hashes match the
   claims in `docs/PROJECT_STATUS.md` exactly; failures are recorded *as
   failures* (the rustfmt stack overflow, serial flakes, a 7005/7006
   near-miss are all logged honestly).
4. **The honesty machinery.** The 4-label status rule
   (implemented/verified, partial, planned, denied), the identical deny-lists
   in README/AGENTS.md, and "do not claim WebPKI until actually present"
   mostly worked: the project never faked persistence or trust.
5. **Hard-won systems knowledge.** `docs/DEBUGGING.md` Known Failure Modes
   (limine.conf vs .cfg, higher-half PHDR linking, SSE enable before
   allocator code, the smoltcp `pshufb` #UD from SSSE3 in the custom target)
   is genuinely valuable and correct.
6. **The vision documents.** ADRs 0001–0004 are coherent and mutually
   consistent on the big picture; ADR 0004 (system-as-memory, typed
   provenance-bound facts, token-budgeted context packets) is the most
   original and defensible part of the whole project.

---

## 3. The Core Diagnosis: Governance Built Before the Governed Mechanism

Everything below is one sequencing error viewed from different angles.

### 3.1 The ceremony cathedral

The denied-by-default rule from ADR 0002 ("structured `capability_denied`
naming missing evidence") mutated into a code generator. Each recent commit
(`2b2294f` "Add hello rollback append intent gate", `1a3d361`, `aef5f21` …)
adds another gate proving a nonexistent feature is still nonexistent. A
single "latest slice" bullet in `docs/ROADMAP.md` (lines 49–164) runs 100+
lines chaining ~20 evidence records that bind hashes of hashes of hashes —
for a rollback that is deliberately never applied.

Every gate stratum costs the same hand-stamped quartet: ~187 constant
declarations (`hello_service.rs:26-1226`), a 25-field struct, a field-by-field
JSON emitter, a field-by-field SHA-256 chain (2,090 `hash_line_*` calls in
`hello_service.rs` alone; ~29,600 raw emit call sites codebase-wide), an
in-kernel selftest, and PowerShell substring needles. The same field name
exists in four hand-synchronized copies.

### 3.2 The two unanswered questions all of it depends on

1. **The isolation model is forked three ways in the docs and answered
   nowhere in code**: `docs/invariant-choices.md:62` commits to a single
   active Wasm module; ADR 0002:461 leaves isolation open; ADR 0003 requires
   a native multi-service graph with versioned state migrators. The kernel
   has zero Wasm support and no second address space. Whatever answer
   eventually lands will invalidate large fractions of the 105k-line envelope
   vocabulary, because capability envelopes shaped for an unknown execution
   model are speculation, not architecture.
2. **The first durable write has never happened.** Every rollback chain
   terminates in `persistence_device_write_path_missing` — yet AHCI
   write/readback already works (`ahci.rs:750-771`), and the
   `RAIOS_AUDITRB_V0` LBA1 target region is already discovered and verified
   read-only. The remaining work is authority policy, not driver work. One
   real transaction append would convert the entire denial edifice from
   theater into a functioning transaction system.

### 3.3 The safety claims have no enforcement mechanism

README lines 182–208 claim recovery "is impossible to break from above" and
"nothing can exceed its declared capabilities at runtime." Reality: one flat
statically-linked ring-0 binary; the 10,412-line module loader runtime never
transfers control to anything; TLS/HTTPS/DNS run kernel-resident,
contradicting the README's own principle "the kernel does not parse the
internet" (lines 253–254). This is not a gap to fill later — it is the
load-bearing claim of the pitch.

### 3.4 Aspiration stated as fact — a credibility risk for a trust product

- README "Building with the AI" and "The Recovery Lifeline" (166–208)
  narrate the Shadow-VM promotion loop, signed recordings, recovery shell,
  and rollback replay in present tense, while "Still intentionally missing"
  (289–299) admits none of it exists.
- README 214–220 claims "no embedded credentials … keys never appear on
  disk," but `scripts/build-seed-kernel.ps1:41-90` embeds the OpenAI API key
  and TLS pins into the kernel binary at compile time from env vars, and an
  `-AllowUnverifiedOpenAiTls` switch exists (fail-open TLS). Guardrails
  against *committing* key-bearing images are real; the on-disk-binary claim
  is false as written.
- "Provider-agnostic by design" describes an OpenAI-only transport.

For a project whose product is trust, one falsifiable doc claim costs more
than all 588 VM reports buy.

---

## 4. Code Health Verdict

**Will not collapse, but on trajectory to become economically unextendable
well before six more months at current growth.** (+87,678 lines landed in one
six-day window in May; 104 "Add" commits vs 3 "Fix" and 1 "Remove"; nothing
meaningfully deleted since 2026-05-11; rustfmt already stack-overflows on the
oversized sources; the flat binary crate has no `lib.rs` and no workspace
split.)

The saving grace: the duplication is *stamped, not organic* — extremely
regular — and the 6,789-predicate golden-string harness makes byte-identical
refactoring machine-checkable. The collapse is still cheap today. The window
closes as each new stratum (18,300 uncommitted lines added to
`hello_service.rs` in ~2 days) pushes the refactor toward a rewrite.

Specific structural defects (each with a mechanical, AI-executable fix):

| # | Defect | Evidence | Fix |
|---|--------|----------|-----|
| 1 | No data model: JSON streamed to serial as a side effect via a 221-line support module; emitter and hasher are two independent field lists that can silently diverge | `agent_protocol_support.rs:109-111`; ~29,600 emit sites; 2,440 hash calls | One typed `Value` enum + one serializer over a `ByteSink` trait (serial impl + `Vec<u8>` test impl) + one canonical hasher over the same structure. Port gate-by-gate, diffing serial output against harness needles. Est. collapse: 75–85% of the 148k-line agent layer |
| 2 | 22,705-line single-file monolith, 80% uncommitted | `hello_service.rs` (4,400 at HEAD) | Commit now; gate-freeze; split by concern into `hello/` module dir; then absorb into the Value model |
| 3 | Near-identical 25-field Input/ReferenceCheck structs cloned per pipeline stage; positional literal-bool soup | `agent_protocol_recovery_command_effect_types.rs:4-101`; `..._loader_runtime.rs:5016-5096` | Shared `CommandBindings<'a>` + `BoundaryFlags` struct with field-init syntax |
| 4 | 21-positional-token command grammar; arity validated by a 22-term `&&` chain; insertion-brittle | `agent_protocol_recovery_command_effect_reference_eval.rs:14-100` | Named `key=value` args parsed by one generic routine |
| 5 | Dispatch god-file: linear if-chain of ~200 blocks + 215 string-predicate functions | `agent_protocol.rs:292+` | Static `MethodEntry` table; makes `system.describe` generatable from the same data |
| 6 | "Generic" infrastructure hardcodes the demo service | `event_log.rs:4419-4526` (`record_hello_service_*`); 6 files hardcode `svc.demo.hello` | Parameterize by a `ServiceDescriptor`; acceptance test: a second trivial service (`svc.demo.echo`) must cost only a descriptor + state machine |
| 7 | Inverted test pyramid: **zero** `#[test]` in the kernel; everything through a 9-minute serial-substring harness that is currently red | grep-verified; `ci/` is a 127-byte README | Extract a no_std library crate (types + eval logic) that also compiles for the host; ordinary `cargo test` in seconds; keep QEMU for boot/driver/integration |

---

## 5. The Scope Decision the Owner Must Make

The genuinely novel content of raiOS — typed agent protocol, capability
grants computed from local policy + evidence (never trusted from manifests),
evidence-gated promotion, typed provenance-bound memory with classification
and budgeted context packets, transactional rollback — **contains no
bare-metal dependency.** On Linux, every mechanism this project has spent
months describing-but-denying exists today: seccomp/Landlock/namespaces for
real capability confinement, btrfs/overlayfs snapshots for real rollback,
microVMs for the Shadow VM, persistence on day one. The promotion loop that
has never run in raiOS could run end-to-end for a real AI-authored service
within weeks — validating or falsifying the core bet.

The honest counterargument is identity: the Lisp-machine/Tamagotchi framing
and the fully-modelable world are *why the project exists*, and the kernel
work is real. But an OS pitch about safe self-modification is not validated
by drivers; it is validated by the promotion loop, which bare metal has
pushed years out.

Three options — the choice should be recorded as **ADR 0005**:

- **Option A — status quo, bare-metal-first.** Highest identity purity;
  promotion loop remains years away; the thesis stays unfalsified.
- **Option B — two-track (reviewer recommendation).** Prove the
  agent-protocol / typed-memory / promotion layer as a Linux userspace
  supervisor now; keep Stage-0 as the long-term substrate track; design the
  protocol/evidence crate to be shared between both (the no_std library
  crate from Section 4 fix #7 is exactly that crate). This converts 105k
  lines of aspiration into a falsifiable product experiment without
  abandoning the kernel.
- **Option C — bare metal + Wasm now.** Keep one track, but adopt an
  in-kernel Wasm interpreter (wasmi-class, no_std-compatible) immediately so
  capability envelopes become enforceable syscall surfaces and isolation is
  real. Middle cost, keeps identity, still delays real persistence/recovery.

Also resolve the isolation three-way fork (Section 3.2) in the same ADR —
whichever option is chosen.

---

## 6. Build-Process Verdict: How the Agents Are Building

**This is one of the more rigorous AI-driven build processes reviewed** —
verification is real (actual QEMU boots of the real image; failures recorded
as failures) — **but it is a completely closed loop.** The same agent writes
the kernel's marker strings, the harness needles that match them, and the
docs describing both. A bug implemented consistently across all three passes
100% of predicates. There is no CI, no second machine, no host unit tests, no
review, no branch workflow, and no mechanical gate stopping a commit.

### 6.1 Urgent, this week

1. **The project's own pre-commit gate is red while development continues.**
   `DEBUGGING.md:205` names `-Profile full` "the pre-commit/release evidence
   path." Every full-profile run since 2026-07-02's 6789/6789 green has
   failed — including **two real predicate failures**
   (shadow-20260703-183727: 7005/7006 `module_manifest_audit_source`;
   shadow-20260703-190659: 7380/7381 entrypoint-boundary), not just serial
   flakes. Meanwhile ~20,500 lines sit uncommitted.
   **Rule to add to AGENTS.md: "While the full profile is red, the ONLY
   permitted work is fixing it."**
2. **Commit the working tree.** The +18.3k-line uncommitted delta in
   `hello_service.rs` repeats the June pattern where 5 weeks of work landed
   as one mislabeled 22,547-insertion commit (`c596f20`). Add a rule: no
   slice ends with uncommitted source; >~2,000 uncommitted lines means the
   next action is a commit, not a feature.
3. **Classify failures before retrying.** The documented flake protocol
   (rerun with smaller serial chunks until green) is currently blending real
   guest bugs with host transport noise. Require: every failed run gets the
   failing predicate name + a one-line "host-transport vs guest-behavior"
   verdict in the status file before any retry; fail-then-pass-without-code-
   change is logged as a suspected intermittent guest bug.

### 6.2 The incentive fix: change the definition-of-done

The cheapest way for the agent to satisfy "complete a real, evidence-bound
vertical slice" is another deny-by-default gate schema — and that is exactly
what happens. The project has already diagnosed this itself (ROADMAP.md:26
"stop adding schema-only boundaries"; AGENTS.md:91-92 warns against
"denial-only predicate counts") but nothing enforces it.

Add to `AGENTS.md` and `AI_Builder_Initial_Prompt.md`:

1. Every slice must state, in one sentence, **what a user or agent can now
   DO that it could not do before.** "A new denial schema exists" does not
   qualify.
2. No new `raios.*.v0` schema may be added as hand-rolled emit/hash code —
   new schemas must be entries in the (to-be-built) Value/record model.
3. At most one denial-gate slice per N capability slices.
4. Deletion budget during the refactor: each slice that ports a gate to the
   new model must delete more lines than it adds (the golden-string harness
   verifies equivalence).
5. Status reports to the owner lead with the capability sentence, never the
   predicate count.

### 6.3 Break the self-referential oracle (owner can do all of this)

1. **CI on a second machine.** Have the agent build a GitHub Actions
   workflow in one session: pinned-toolchain build, package image, headless
   QEMU quick profile under TCG, upload the report artifact. This gives
   reproducibility proof and a gate the agent cannot skip. (`ci/` currently
   contains a 3-line aspirational README while `release/README.md` claims a
   "reproducible pipeline" with SBOMs that do not exist.)
2. **Host unit tests.** Definition-of-done addition: any new
   gate/eval/hash logic ships with a host-runnable `cargo test` (enabled by
   the library-crate split, Section 4 #7).
3. **Periodic adversarial review by a different model.** Once a month,
   paste the week's diff summary and PROJECT_STATUS.md into a different
   model with: "find what is wrong with this plan and this code." The May
   plan-review process was exactly this and it worked; it predates the July
   pivot and was never refreshed.
4. **Convert prose rules to mechanical checks.** Every current rule is
   being violated (file-size rule vs 22.7k-line file; "stop schema-only
   boundaries" vs five consecutive schema-only commits; "full smoke on
   rollback changes" vs a red full profile during a rollback-gate spree).
   Required end-of-session block: a line-count check script, a
   newest-full-report-is-green-and-newer-than-last-commit check, and
   `cargo fmt --check`.

### 6.4 Fix the owner-visibility channel

`PROJECT_STATUS.md` (4,118 lines, ~75-80% ceremony, rolling-rewrite with
muddled chronology) works as agent-to-agent memory but has failed as owner
oversight. Add a mandatory **OWNER-DASHBOARD** section (or separate 1-page
file), capped at ~30 lines, rewritten every session, in plain language:
current capability, gate status (full profile green/red + date of last
green), top risk, next task. Adopt an explicit **owner-visible capability
per month** metric; when the owner can no longer perceive progress, the
project ends.

---

## 7. Cut / Freeze / Park List

1. **Freeze all `agent_protocol_recovery_*` growth** (46 files, the largest
   single code area) until a recovery mechanism — any recovery mechanism —
   exists. Purest case of vocabulary-before-mechanism.
2. **Formally retire or re-scope the orphaned host-side signing lane**
   (`ota/`, `registry/`, `fake-cloud/` — ~3,700 lines, frozen since May,
   architecturally orphaned by ADR 0002's local-attestation model, never
   connected to the kernel). Write a short ADR either killing it or naming
   the future slice that revives it.
3. **Roadmap/status debt.** Collapse ROADMAP.md to: current cursor, next 3
   capability milestones (durable write, isolation model, first external
   service through the promotion loop), and the gate list. Move verification
   history into `docs/archive/` or the vm-reports themselves. Drop the
   "parallel tracks" framing until more than one agent actually works
   concurrently (history shows execution is de facto serial).
4. **README truth pass.** Move the promotion-loop and recovery-lifeline
   narratives to future/conditional tense or a `VISION.md`; apply the
   4-label rule to every claim; fix the key-embedding description; delete
   `-AllowUnverifiedOpenAiTls` or force "UNSAFE" into the output filename.
5. **Repo hygiene (one session).** `.git/objects` is 682 MB against a
   1.71 MB source pack (the 67 MB image committed 24×, the 18 MB ELF 45×);
   tracked binaries stopped matching tracked source after 2026-05-21;
   `vendor/limine` is a dangling gitlink with no `.gitmodules` — the
   trust-critical bootloader has no buildable provenance; six diskpart
   transcripts litter the root. Actions: `.gitignore` + `git rm --cached`
   the binaries, publish images via tagged releases, run `git-filter-repo`
   once **now** (it only gets more disruptive later), pin Limine via proper
   submodule or hash-verified fetch script, delete the debris.

---

## 8. Recommended Order of Work

Each step fits the project's existing small-slice cadence and is verifiable
with existing profiles.

0. **Stabilize:** commit the working tree; fix the full-profile serial
   transport (root-cause the `audit.events 256` hang — it may be a guest
   bug); get the golden master green. *Nothing else until this is done.*
1. **Decide:** ADR 0005 — isolation model + substrate strategy (Option
   A/B/C from Section 5). One decision, supersedes the three-way fork.
2. **Extract:** the no_std library crate (types + eval + Value model +
   ByteSink) with host `cargo test`; stand up minimal CI.
3. **Collapse:** port gates to the Value/record model one slice at a time,
   deleting old emitters; byte-identical serial output verified by the
   harness. Target: agent layer under ~20k lines.
4. **Mechanism #1 — first durable write:** the `RAIOS_AUDITRB_V0` LBA1
   audit/rollback transaction append (driver work already done; this is
   authority policy). This single slice retroactively justifies the entire
   evidence chain.
5. **Mechanism #2 — isolation:** per ADR 0005 (Wasm interpreter, second
   native service in its own address space, or Linux-track supervisor).
6. **Generality proof:** `svc.demo.echo` as a second service; must cost a
   descriptor + state machine only.
7. **The loop:** one external, AI-authored artifact through the full
   promotion path — authored, shadow-verified, capability-granted, promoted,
   rolled back. This is the project's actual first milestone as a product.

---

## 9. Bottom Line for the Owner

You have proven something real and rare: a disciplined, honest, AI-built
bare-metal OS foundation with a receipts culture most human teams lack. You
have not yet proven the thing raiOS is *about* — that an AI can safely
change a running system through evidence-gated, reversible transactions —
because the machinery for actually changing anything (isolation, persistence,
promotion) was deferred while its paperwork was built to extraordinary depth.

The approach and angle are right. The scope inside them needs to be
re-pointed from *describing authority* to *building mechanism*, the
verification loop needs one independent leg (CI + host tests + periodic
outside review), and the agents need a definition-of-done that counts
capability, not evidence volume. All of it is incremental, all of it is
executable by the coding AI you already use, and the golden-string harness
you already built is precisely the safety net that makes the cleanup cheap —
if it starts now.
