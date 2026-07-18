# Session Retrospective Audit — 2026-07-08

**Scope:** independent, read-only retrospective audit of everything built in
`git log 9cee0c8..HEAD` (47 commits, M6D-2 → M7 → M8 → M9A → M9B → M9C-1a).
**Audit target:** committed state at **HEAD = `209c450`** (M9C-1a). Uncommitted
working-tree changes to `durable_store.rs` / `memory_store.rs` /
`agent_protocol*.rs` / `ui.rs` / `PROJECT_STATUS.md` / the memory-durable profile
(two live Codex workers) were deliberately ignored; every file cited below was read
via `git show 209c450:<path>` when the working copy was dirty.
**Context:** the ~20 h under review ran with a mis-configured orchestrator
(implementation workers were Claude subagents rather than OpenAI Codex CLI;
planners ran as Opus rather than Fable). This audit asks the only question that
matters for that: *is the resulting code correct, consistent, and honestly
documented?*

---

## 1. Executive verdict

**SOUND-WITH-FINDINGS** — every security invariant holds end-to-end at HEAD and
every load-bearing milestone claim is backed by real code; all findings are
documentation-consistency / claim-precision / dead-input issues, **zero** are
code-security defects.

Findings by severity: **CRITICAL 0 · HIGH 0 · MEDIUM 3 · LOW 5 · NIT 3.**

**The wrong-orchestrator period did not corrupt the code.** See §5.

---

## 2. Security-invariant table

| # | Invariant | Verdict | Evidence (file:line @ 209c450) |
|---|-----------|---------|--------------------------------|
| 1 | **Own-scoped-evaluator discipline** — every durable write path is gated by its OWN pinned evaluator, no shared write-boundary flip | **HOLDS** | 10 evaluators, pairwise-distinct method+target pins: `scoped_seed_data_append.rs:7-10`, `scoped_promotion_transaction_append.rs:8-11`, `scoped_recovery_action_append.rs:8-11`, `scoped_recovery_load_append.rs:23-26`, `scoped_artifact_persist_append.rs:8-11`, `scoped_artifact_store_blob.rs:8-11`, `scoped_boot_control_replace.rs:13-16`, `scoped_memory_record_append.rs:30-33`, `scoped_repromotion_append.rs:8-11`. Kernel callers each use exactly one (`durable_store.rs:238,573,1327,1628,2540`; `artifact_store.rs:298,376,1084,1159`; `boot_control.rs:346,548`; `repromotion.rs:816`). No `write_boundary_flip` boolean exists in the range (grep empty). |
| 2 | **No shared write-boundary flip introduced/widened** | **HOLDS** | The old `agent_protocol_module_write_boundary_*` booleans are untouched by any M7–M9 durable writer; every new writer routes through a fresh scoped evaluator, not a shared boolean. |
| 3 | **Provider export fail-closed** | **HOLDS** | `memory.context` emits `provider_export:"disabled"` (`agent_protocol_memory.rs:64,384`); provider gate state `"disabled"`/`automatic_context_injection:"disabled"` (`agent_protocol_provider.rs:653-654,778-779,869-870`); no M9 durable-record content is routed to any provider path. |
| 4 | **Secrets never durable (3 layers)** | **HOLDS** | Type has no `Secret` variant (`memory_record.rs:159-162`); constructor `Classification::parse("secret") → Err(SecretNeverDurable)` (`:168-175`); write-boundary evaluator `Some("secret") → denied("classification_secret_never_durable")` (`scoped_memory_record_append.rs:269`); reparser `Classification::parse` maps secret → `SecretClassification` drop (`memory_record.rs:875-876`). |
| 5 | **Honest trust labels — nothing claims owner_sealed** | **HOLDS** | No `owner_sealed:true` literal anywhere in `seed-kernel/src`+`raios-core/src` (grep empty). Every memory write forces `owner_sealed:false, persistence_claimed:false, trust_tier:"dev_key_not_owner_sealed"` (`durable_store.rs:2569-2571,2698-2700`; evaluator denials `scoped_memory_record_append.rs:324-332`). |
| 6 | **Agent confinement (M9B)** | **HOLDS** | Kernel hardcodes `kind:"observation", classification:"local_only", authority:"agent", supersedes:[]`, kernel-assigned id (`memory_store.rs:1025-1041`); evaluator backstop re-denies non-observation / any supersede / non-local_only when `agent_authored` (`scoped_memory_record_append.rs:312-322`); the parser is bounded+fail-closed (base64 decode, 4-field count, per-field byte caps 64/32/96/64, locator charset `[A-Za-z0-9 ._:/-]`) (`memory_store.rs:927-1008`). Broad `memory.record_observation`/`propose_policy`/`supersede_fact`/`redact`/`compact` stay `MethodAction::DeniedMemoryMutation` (`agent_protocol.rs:546-550`). |
| 7 | **Audit immutability — write side** | **HOLDS** | Audit kinds may never be authored superseding: constructor `kind.is_audit() && !supersedes.is_empty() → Err(AuditKindMayNotSupersede)` (`memory_record.rs:286-288`) AND evaluator `audit_kind_may_not_supersede` (`scoped_memory_record_append.rs:290-302`). |
| 8 | **Audit immutability — read side (R1 + id-shadow)** | **HOLDS** | Resolver ignores supersede links targeting audit kinds and records them (`memory_record_resolve.rs:132-135`); a later same-id non-audit record can never displace an audit record — reuse is flagged, audit stays visible (`:73-95`), proven both orderings (`:296-318`). |
| 9 | **Signed set untouched** | **HOLDS** | `git log 9cee0c8..209c450 --` for every `build.rs:7-29` `HELLO_ARTIFACT_SOURCE_SET` member (`hello_service.rs`, submodules, `current_boot_service.rs`) returns empty. The one commit that names `current_boot_service.rs` (`2655474`) touches only `.gitattributes`. |
| 10 | **M8 lifeline separation + pinned vocabulary hash** | **HOLDS** | `LIFELINE_METHODS` (6 methods) fingerprint `7488a1abb0791a9e278d6883d6b45d993001148c7e0ffc03a50347399af3cc56` pinned identically in all 3 locations: golden test `recovery_lifeline_table.rs:201`, `shadow-vm-smoke-profile-m8-lifeline.ps1:16`, `shadow-vm-smoke-profile-quick.ps1:3214`. Dispatch is a separate path checked BEFORE the general table (`agent_protocol.rs:807-812`) and only accepts the 6 pinned names (`recovery_lifeline.rs:43-82`). |

---

## 3. Per-milestone claim-vs-reality spot checks

### M7 — Persistence + two-boot proof
- **"Boot 2 re-verifies the signature itself, never trusting a stored boolean"** — **VERIFIED.** The persisted blob's sha256 is recomputed over the actual disk bytes (`artifact_store.rs:708-738`, `repromotion_reverify.rs:45`); the p256 dev-key signature is re-verified cryptographically against the pinned SEC1 key over the *recomputed* attestation hash (`repromotion_reverify.rs:84-97` → `promotion_attestation.rs:38-51`); the stored `signature_verified` bool is never read on the decision path, and the test `positive_reverifies_signature_even_when_stored_boolean_is_false` (`repromotion_reverify.rs:196-204`) proves it.
- **Authority flip goes through the UNMODIFIED M6 gate** — **VERIFIED.** Repromotion reaches execution via the pre-existing `granted_candidate_service::emit_load`/`emit_start` (`repromotion.rs:426,449`), which existed at `447428c~1`; no parallel grant path.
- **Boot-control ping-pong writes only the loser slot; SAFE disables appends** — **VERIFIED.** `plan_boot_success_mark` writes `loser_slot(...)` with `seq+1` and asserts the authoritative slot/seq first (`boot_control.rs:268-407`); SAFE posture gate present at every durable-append site.
- **Artifact store hash-verified on readback + full-pin auth** — **VERIFIED.** write→readback→sha256 compare (`artifact_store.rs:280-315`), authorized by `evaluate_scoped_artifact_store_blob` + `evaluate_scoped_artifact_persist_append` full pin battery, not a boolean.

### M8 — Recovery lifeline
- **Crash survival is a REAL Wasm trap** — **VERIFIED.** `run_echo_fuel_starved` runs the real echo artifact with `fuel=1`, producing a genuine wasmi `OutOfFuel` trap caught as `Err` (never unwrapped) (`wasm_runtime.rs:159+`, `:597-617`; `echo_service.rs:311-356`); the single-boot m8-lifeline profile drives crash → lifeline_table (unchanged hash) → snapshot (echo listed crashed) → restart, all after the trap.
- **disable_module / restart_last_good: durable-audit-before-mutate, denial blocks mutation** — **VERIFIED.** Append first, then `if evidence.performed` disable/restart; the denied branch mutates nothing (`recovery_lifeline.rs:243-284,428-475`).
- **load_artifact_by_hash full re-verify** — **VERIFIED** (sha256-over-disk-blob + parse + signature re-verify + reconstructed-wasm validity + M6 gate). One claim-wording caveat: see **L-1** below.

### M9A — Durable memory
- **"payload_sha256 == record_sha256 by construction"** — **VERIFIED.** Payload = `write_json(record.to_record_value(), indent 0)` (`durable_store.rs:2704-2711`); `record_sha256()` = `sha256_of_json(to_record_value())` = sha256 over `write_json(...,0)` (`memory_record.rs:353-354`, `record.rs:109-116`). Same renderer, same indent → identical bytes.
- **supersede-not-overwrite** — **VERIFIED.** Records A/P/B each appended (never mutated), B carries `supersedes:[A.id]` (`memory_store.rs:298-330,361-364`); resolution is read-side (`memory_record_resolve.rs`).
- **decision/problem required-field rules** — **VERIFIED** in constructor (`memory_record.rs:268-284`), evaluator, and reparser (`memory_record.rs:937-952`).

### M9B — Agent observation
- **"the parser never decodes the value sub-tree" / confinement** — **VERIFIED.** Kernel + evaluator double-force the observation confinement (see invariant 6); the reparser structurally skips `value` (`memory_record.rs:869-871`, `skip_value` `:633-720`) and `MemoryRecordView` has no `value` field (`:413-427`).
- **quota reserve/release balanced** — **VERIFIED.** Reserve before plan/write (`durable_store.rs:2466`); release on every post-reserve denial (`:2475,2502,2517,2577,2601`); no release on success (`:2615`); SAFE check precedes reserve (`:2459-2464`) so no leak.

### M9C-1a — Reparser / walker / resolver (core-only, grants nothing)
- **"grants nothing / nothing wires it yet"** — **VERIFIED.** `memory_record::parse`, `resolve_durable_memory`, `scan_reclog_payloads` have zero callers in `seed-kernel/src` (grep empty). The reparser's `CANONICAL_FIELDS[15]` (`memory_record.rs:389-405`) is a byte-exact twin of the writer's `fields()` order (`:315-345`).

---

## 4. Ranked findings

### MEDIUM

**M-1 — Committed docs cite a stale full-profile count (8168) that no longer matches reality (7834).**
`docs/PROJECT_STATUS.md:3421` (M9A-2b) says `` `full` (8168) all green``; `docs/OWNER_DASHBOARD.md:109` says `8,168/8,168`; ROADMAP uses 8168 for M8/M9A. But the M9-era commit bodies and the doc's own M9B block-close (`PROJECT_STATUS.md:3582`) say `full 7834/7834 PASSED`, and **all three surviving on-disk full reports** (`release/vm-reports/shadow-20260707-201746-23964.json`, `-212506-20936.json`, `-231216-4252.json`) are **7834/7834 passed, 0 failed**. The current full profile is genuinely 7834 and genuinely green; the `8168` figures are a stale carry-over from the M6–M8 era.
*Impact:* an owner/agent comparing a fresh `full` run against the docs would see a 334-predicate "shortfall" that is not real. *Fix:* update the 8168 references to 7834 and add a one-line note that the full profile was re-pinned between M8 and M9B (coverage stayed green). Likely already in-flight in the uncommitted `PROJECT_STATUS.md` edit.

**M-2 — Multiple stale "current milestone / next task" cursors point back to M6/M6A.**
`docs/ROADMAP.md:20` (`Current milestone: **M6 Promotion Loop v0**`, `Last updated: 2026-07-06`); `docs/PROJECT_STATUS.md:2537` (`Current exact next task … sub-milestone M6A … DONE (2026-07-06)`, and it is the doc's *designated* cursor per `:4021-4022`); `docs/OWNER_DASHBOARD.md:159/172/203` (`Now active: M6` / `M7 … now active` / `M7 is complete … Next: M8`). The bodies of all three docs correctly reach M9B; reality at HEAD is M9C-1a landed. Four different "current" signals disagree within the committed tree.
*Impact:* a future agent trusting the cursor could redo M6–M8 work or mis-sequence M9C. *Fix:* repoint the three cursors to "M9C-1a landed (core-only); next M9C-1b kernel wiring." (The M9C-1a write-up itself is the pending uncommitted `PROJECT_STATUS.md` diff, so the docs trail HEAD by exactly one slice — an *under*-claim, not an overclaim.)

**M-3 — Stale Known-Gap: "No signed module runtime exists yet."**
`docs/PROJECT_STATUS.md:5364`. Contradicted by M6C-1 (`fc28e18`, a granted external Wasm candidate loads and RUNS under the M4 envelope) and M7D-2 (`8965301`, a signed promoted module runs live across a reboot via the M6 gate). A dev-key signed module runtime demonstrably exists.
*Impact:* understates shipped capability. *Fix:* revise the gap line to "signed module runtime is dev-key only; owner-sealed trust not yet implemented."

### LOW

**L-1 — M8D claim wording overstates what the recovery-load evaluator gates.**
The milestone framing "authority flip gated by `evaluate_scoped_recovery_load_append` pins" is imprecise: the actual RAM load authority is the **M6 gate** (`granted_candidate_service`), which runs and commits the load *before and independently of* the scoped evaluator; `evaluate_scoped_recovery_load_append` gates only the durable **audit** record appended afterward (`recovery_lifeline.rs:681-704`, comment `:687-688`). The code is correct and the comments are accurate — only the milestone/commit *language* overstates. *Fix:* say "the load runs through the unmodified M6 gate; the scoped evaluator gates the durable audit so it can't claim authority the gate didn't grant." No code change.

**L-2 — `repromotion_reverify.rs:19` carries an unconsulted `signature_verified` input.**
`RepromotionTransactionFields.signature_verified` is parsed (`repromotion.rs:781`) and reported but never read by the decision (which re-verifies crypto instead — the *correct* behavior). It is a dead decision input. *Fix:* comment it "evidence-only, never trusted" or drop it from the decision struct.

**L-3 — Partly-dead status branches in the recovery-load evaluator.**
`scoped_recovery_load_append.rs:263-279` documents/handles `reverified` and `reinstate_denied` audit statuses, but the only production caller hard-codes `decision_status:reinstated, would_reinstate:true` (`durable_store.rs:1655-1656`); the other branches are unit-test-only. Defensive, not wrong, but the doc comment implies audit records HEAD never emits. *Fix:* note the non-`reinstated` branches are test-only today.

**L-4 — `recovery_lifeline_table.rs:88-90` stale NOTE.**
The M8A-2 note says `snapshot`/`rollback`/`load_artifact_by_hash` "must move to head-token matching once implemented," but `load_artifact_by_hash` already is head-token matched (`recovery_lifeline.rs:63`). Reads as an open TODO whose actionable part is done. *Fix:* trim the note.

**L-5 — AGENTS.md end-of-session check not reconciled with the new cadence.**
`AGENTS.md:192-193` still requires "the newest full-profile report … is newer than the last commit," which cannot hold under the 2026-07-07 aggressive-fast rule (`AGENTS.md:118-140`) that runs `full` only at block close. Internal tension inside one file. *Fix:* qualify check #2 with "at a durable-milestone / block-close commit."

### NIT

**N-1 — Two artifact evaluators share the schema id `raios.artifact_persist.v0`.**
`scoped_artifact_persist_append.rs:10` and `scoped_artifact_store_blob.rs:10` share `EXPECTED_RECORD_SCHEMA`, but are still pairwise-distinguished by method + target_id + region_marker (`RECLOG` vs `ARTSTOR`), so neither evaluator can accept the other's input. No widening; noted only because every *other* evaluator has a unique schema id too.

**N-2 — Reparser is intentionally stricter than the writer on metadata escapes.**
`parse_string` rejects any `\` in a metadata string (`memory_record.rs:592`), while `write_json_str` *can* emit escapes (`record.rs:200-214`). Safe today because both confined writers (fixed system strings; agent charset `[A-Za-z0-9 ._:/-]`) never put an escapable char in a metadata field, so no legit frame is ever dropped. This is a deliberate borrow-raw-slice optimization but an undocumented forward constraint: a future writer that allows richer metadata must keep metadata escape-free or the reparser will silently drop the frame. *Fix:* one comment on `CANONICAL_FIELDS` noting metadata fields must stay escape-free.

**N-3 — Old cited VM report files are absent from disk (expected cleanup, flagged for the record).**
Every pre-M9 report filename cited by commits/docs (the 8168 fulls, the 85/85 two-boot reboot proof, the 43/43 memory-durable) is missing from `release/vm-reports/` — consistent with the owner's documented practice of cleaning old `vm-reports` to reclaim disk. The 16 surviving reports are all M9-era and all corroborate the headline M9 numbers (77/77, 105/105, 7834/7834, 3644/3644) plus four honestly-documented host-transport flakes (`shadow-20260707-223139-27740` memory-durable UART-RX flake; `-225515`/`-225907` quick serial flake; `-230851` image-packaging exit-1). Not a dishonesty finding — recorded so the owner knows the old evidence files were cleaned, not that the runs never happened.

---

## 5. What the wrong-orchestrator period cost us

**The code is fine.** Across 47 commits produced under the mis-configured setup
(Claude subagents instead of Codex CLI; Opus instead of Fable planners), every
one of the ten security invariants holds end-to-end at HEAD, every load-bearing
milestone claim (boot-2 crypto re-verify, M6-gate authority flip, ping-pong
crash-safety, hash-verified artifact persist, real Wasm-trap crash survival,
payload==record hash by construction, balanced quota, three-layer secret refusal,
two-sided audit immutability, agent confinement) is backed by the actual source,
and the M9-era verification numbers are corroborated by real passing VM reports.
The scoped-evaluator discipline — the spine of the whole security model — was
maintained faithfully: ten independent evaluators, each with its own pins and a
genuine (non-tautological, pairwise-unique) denial truth-table test, and **no**
reintroduction of the old shared write-boundary flip. The signed source set was
not touched. Nothing overclaims trust: everything is honestly `dev_key_not_owner_sealed`.

**What it did cost is documentation coherence, not correctness.** The findings are
concentrated in the docs (M-1/M-2/M-3): stale milestone cursors pointing back as
far as M6, and a full-profile predicate count (8168) that the docs never updated
to the current 7834 even though the current profile is green. These are exactly
the kind of bookkeeping drift you would expect when the narrative-writing cadence
outran the doc-cursor updates — annoying and mildly misleading to a future agent,
but they do not touch a single security boundary or a single durable-write path.
The remaining LOW/NIT items (a dead decision input, a couple of test-only
branches, one over-precise milestone sentence about M8D) are ordinary hygiene.

Net: an owner worried that a wrong-model worker "quietly broke something the tests
wouldn't catch" can stand down on the code. The right follow-up is a short docs
pass — repoint the three cursors, fix 8168→7834, refresh the "no signed module
runtime" gap — most of which appears to already be in flight in the uncommitted
`PROJECT_STATUS.md` edit.

---

*Auditor note: read-only for all source/docs; this file is the sole write. No
QEMU/build was run (workers hold the build lock); VM numbers were checked against
existing `release/vm-reports/*.json` only.*
