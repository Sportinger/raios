# Roadmap

This file holds direction, the capability milestones, and the compact active
cursor. It must stay under ~250 lines. Verification history, report hashes,
and per-slice evidence live in `docs/PROJECT_STATUS.md` and
`release/vm-reports/`; the full pre-restructure roadmap (1,947 lines,
including all phase definitions and archived evidence blocks) is preserved
verbatim at `docs/archive/roadmap-2026-07-04-pre-restructure.md`.

Restructured 2026-07-04 per
`docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md` and
`docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md`
(owner decisions: bare metal only, Wasm isolation first, mechanism before
vocabulary).

## Agent Handoff Cursor

Last updated: 2026-07-06.

Current milestone: **M6 Promotion Loop v0** (see Capability Milestones).
**M6A (external candidate identity) COMPLETE** (M6A-1 intake + M6A-2a real
serial delivery + M6A-2b real identity). **M6B (verified grant) done:**
M6B-1 verifies a pinned dev-key P-256 promotion signature; M6B-2 is the
FIRST authority flip — `grants_capability=true` (labeled
`trust_tier=dev_key_not_owner_sealed`) when evidence is valid AND the
attestation is signature_verified AND bound to this grant; load stays
denied. Owner decision (ADR 0007): the dev key gets full grant function;
owner key K is the later sealing ceremony. **M6C-1/M6C-2 done:** a
granted, dev-key-signed external Wasm candidate — delivered over serial —
now actually LOADS and RUNS as a live current-boot RAM service inside the
UNCHANGED M4 envelope (`granted_candidate_service.rs`), and the live run is
truthfully visible in `service.inventory`, `module.service_slot_diagnostic`,
and one additive `module.loader_runtime` projection. Native page-mapping
loader readiness, persistence/durable writes, owner-seal, and native guest load
stay false. **M6D-1 done:** that RAM-only dev-key service can now be
UN-PROMOTED through `service.rollback_apply svc.dev.granted_candidate`, which
uses a recorded RAM rollback plan, clears the retained bytes, removes the RAM
slot from `service.inventory`, and verifies the projected inventory hash is
back to the pre-load baseline. Persistence/durable writes, owner-seal, and the
generic durable load gate still stay denied. **M6 COMPLETE** (dev-tier RAM loop:
delivered → identity → grant → load → RUN → rolled back).

**M7 Persistence Foundation COMPLETE (2026-07-07).** All of M7A–M7D are done and
verified green: GPT layout, the SEED_DATA RECLOG durable store, boot control +
A/B/SAFE, the durable promotion transaction (M6D-2), the persistent artifact store
(M7D-1), and — the product moment — boot-time re-promotion proven across a REAL
reboot (M7D-2, two-boot proof 85/85). A promoted service now survives a restart and
comes back to life through the same governed M6 gate, still dev-tier and never
owner-sealed. **Now active: M8 Recovery Agent Lifeline** (map
`docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md`). **M8A-1 done
(2026-07-07, evidence-only, grants nothing):** a frozen pinned lifeline command
table on a SEPARATE dispatch path checked before the general method table, with a
`vocabulary_sha256` fence; only `recovery.lifeline_table` reads, the five spec
endpoints all return typed `capability_denied` and mutate nothing; imports no
wasm/provider/net/tls. Verified quick 583/583 (7 needles), FULL 8168/8168 (frozen
recovery byte-identical), raios-core 2/2, max adversarial review clean. **M8A-2 done
(2026-07-07, read-only):** `recovery.snapshot` renders `raios.recovery_snapshot.v0`
from live boot posture + service inventory/health for diagnosis — mutates nothing,
promotes nothing, no provider call, secret-leakage structurally impossible (fixed
strings only, `last_error` dropped). Verified quick 583/583, FULL 8168/8168,
raios-core 2/2, max adversarial review (no BLOCKER/HIGH/MEDIUM). **M8A-3 done
(2026-07-07) — M8's KEY RISK GATE PASSED:** a real wasmi `OutOfFuel` wedge of
svc.demo.echo is caught as a value (no panic), echo is marked `crashed`, and the
lifeline table + snapshot STILL answer while it is wedged (new `m8-lifeline` profile,
191/191). Survives because Wasm is fuel-metered + cooperative, NOT via hardware
isolation (post-M11). Review found + fixed one HIGH honesty bug (crashed latch never
reset → healthy restarted service falsely crashed). Verified m8-lifeline 191/191,
recovery byte-identical, quick 583/583, FULL 8168/8168, raios-core 111. **M8B-1 done
(2026-07-07) — the lifeline's FIRST mutating action + FIRST durable write:
recovery.disable_module.** Disables an exact non-core current-boot module (svc.demo.echo)
by writing a durable recovery_action.v0 record FIRST (through its OWN new scoped
evaluator scoped_recovery_action_append — split 1a grants-nothing → 1b executor), THEN
stopping it; core/lifeline/unknown/SAFE targets denied BEFORE any write. Avoided the two
map traps (shared write-boundary chain; editing the signed current_boot_service.rs).
Vocab hash re-pinned 523b719b→03d3985c. Restore-only, grants nothing, dev_key_not_owner_sealed.
Max adversarial review: no BLOCKER/HIGH, deny-before-mutate fail-closed. Verified
m8-lifeline 225/225 (durable disable proven live on a valid-a persist disk), recovery
byte-identical, quick 580/580, FULL 8168/8168, raios-core 115. **M8B-2 done (2026-07-07)
— the second restore action, recovery.restart_last_good:** brings a disabled/crashed
current-boot module back to known-good by writing a durable recovery_action.v0 record
FIRST (same evaluator, pinned 2nd action_kind restart_last_good — split 2a grants-nothing
→ 2b executor), then clearing the RAM latches and re-running the EXISTING verified start
path (re-hashes the built-in echo bytes against the pinned hash every call → can only run
the attested built-in; false-healthy impossible). Deny core/lifeline/unknown/SAFE/not-
restartable before any write. Vocab hash re-pinned 03d3985c→4a2c52a5. Restore-only,
grants nothing. Max review: nothing above LOW. Verified m8-lifeline 265/265 (restart of a
disabled AND a crashed echo proven live), recovery 3833/3833 byte-identical, quick
580/580, FULL 8168/8168, raios-core 116. **M8B complete (disable + restart).** Next:
**M8C-1 done (2026-07-07, read-only):** recovery.snapshot now surfaces the durable M7C
boot-control state (durable_last_good: last-good A/B slot / seq / boot_success_mark /
safe_mode, source bootctl_slot_pointer, honest missing-evidence) + a read-only
rollback_preview (mutates_nothing, mutating_rollback_available_via_lifeline:false) from
ONE bootctl read — no durable write, no new method, no vocab re-pin (recovery.rollback
stays denied). Verified m8-lifeline 266/266, quick 581/581, recovery 3833/3833
byte-identical, FULL 8168/8168, raios-core 118, max review nothing above LOW. Next:
**M8D-1 done (2026-07-07, grants nothing):** recovery.load_artifact_by_hash <sha256> re-instates a
persisted artifact from the LOCAL M7D store only — M8D-1 parses the hash, selects the matching
artifact_persist record, re-verifies the FULL M6 chain from scratch (reuses reverify_persisted_artifact
unchanged), and REPORTS ONLY (no load, no durable write; authorizes_load always false). Never fetches /
accepts new bytes / a URL. Repromotion reverify extracted to a shared pub(crate) fn with
emit_repromotion_run byte-for-byte unchanged (m6c-promotion 180/180). Vocab re-pinned 4a2c52a5→7488a1ab.
Max review nothing above LOW. Verified m8-lifeline 270/270, m6c-promotion 180/180, quick 584/584,
recovery 3833/3833 byte-identical, FULL 8168/8168, raios-core 119. **M8D-2 done (2026-07-07) — M8
COMPLETE:** the authority flip re-instates a persisted artifact by hash through the FULL
reverify_record path (reconstructed-wasm-validity + the UNMODIFIED M6 gate) + a durable audit via a
NEW own scoped_recovery_load_append evaluator; load authorized ONLY by a genuine reinstated
(reverify + wasm-valid + load + start); grants nothing new, never fetches, deny-before-append. Proven
by the two-boot harness (110/110): boot-2 load-by-hash re-instates the boot-1-persisted artifact and
the service ANSWERS LIVE; wrong-hash + tampered-record deny with no load. Max review nothing above
LOW. Verified two-boot 110/110, m6c-promotion (repromotion intact), recovery byte-identical, quick,
FULL 8168/8168, raios-core 125. **M8 Recovery Agent Lifeline COMPLETE** (diagnose incl. durable
last-good/SAFE, survive a Wasm crash, disable, restart, re-instate-by-hash — all restore-only,
re-verified, dev-tier). **M9 Durable Memory & Context Broker v1 now active** (ADR 0004 Phase D):
raiOS itself is the memory — typed facts with provenance + classification, never a chat log / fake
persistence. **M9A-1 done (2026-07-07, grants nothing):** raios-core/src/memory_record.rs — the typed
raios.memory_record.v0 with a fail-closed constructor (Classification has NO Secret variant, so a
secret plaintext is un-constructable and can never become durable or reach a provider; unknown kind →
Err; unknown classification → local_only; observation needs entity+source; supersede-not-overwrite).
Host-only, no kernel change, no vocab change. Verified cargo test -p raios-core 133 (8 new), rustfmt
clean, kernel builds. The read-only agent_context broker + fail-closed provider export already exist.
**M9A-2 done (2026-07-07, first durable memory write, single-boot):** M9A-2a added
`raios-core/src/scoped_memory_record_append.rs` — a dedicated OWN-pinned write-boundary evaluator (cloned
from scoped_recovery_load_append; 41 pairwise-unique denial reasons; grants nothing). M9A-2b then wrote the
first real durable memory fact: `seed-kernel/src/durable_store.rs::append_memory_record` (structural clone of
append_recovery_load + a RAM-only per-boot write quota) drives the shared reclog gauntlet authorized ONLY by
that evaluator, and `seed-kernel/src/memory_store.rs`'s ONE Read0 driver builds a system-authored
`capability_denial` of the permanently-denied durable module-load gate via the fail-closed `MemoryRecord::new`.
A new single-boot `memory-durable` VM profile is green (43/43): a real durable append whose `payload_sha256`
matches the golden `record_sha256` computed in raios-core (the EXACT record landed, not just a frame), the
live RAM quota driven to exhaustion + refund, RAM-only fail-closed selftests (secret/unknown-kind/quota, no
disk write), and parsed guard needles (every `memory.*` mutation still denied, provider export still
fail-closed). Grants nothing new; honestly dev_key_not_owner_sealed / current_boot. Max-effort adversarial
review: SHIP. Regression green (quick/recovery byte-identical/m6c-promotion/full 8168).
**M9A-3 done (2026-07-07 — CLOSES the M9A block):** M9A-3a added the write-side supersede confinement
(audit kinds can never be authored as superseding records; supersedes ≤8; no self-supersede; decision needs
entity+source, problem needs entity+status) with the read-side R1 rule explicitly deferred to M9C. M9A-3b then
durably wrote THREE truthful system-authored facts — a general `decision` (module sharing is owner-confirmed
vision), a `problem` (memory.* mutations still denied), and a refined `decision` that SUPERSEDES the first
(sharing = candidate intake, NEVER install) — proving supersede-not-overwrite: `memory-durable` 77/77 with all
three records' pinned golden hashes matched (exact bytes on disk, including B's `supersedes:[A.id]`). An
adversarial review caught + fixed a top-level over-claim (the trio response now derives success from the real
per-record evidence). Grants nothing new; system-authored only (agent write is M9B); dev_key_not_owner_sealed
/ current_boot. Regression green (quick/recovery byte-identical/m6c-promotion/full). Next: **M9B-1**
(agent-authored observation, scoped — the first NON-system durable memory write). (M7 map
`docs/plan-reviews/m7-persistence-map-2026-07-06.md`,
revalidated M7-0). Sequencing per the M7-0 note: M7A + M7B build GPT + the
SEED_DATA RECLOG durable store; then M6D-2 records its durable promotion
transaction into SEED_DATA; then M7D re-verifies it after reboot. **M7A-1 done:**
a harness run attaches a real GPT persist disk (SEED_ESP_A/B 128 MiB + SEED_DATA
raw region map with `RAIOS_DATA_SB_V0` superblock: BOOTCTL/RECLOG/ARTSTOR) as a
4th QEMU drive (`bus=ide.3`) with hard release/ refusal — no production image
touched; host-side GPT/superblock validation green. **M7A-2 done:** the kernel
now READ-ONLY parses + validates the GPT (protective MBR, header/entry-array
CRC32, type-GUID/name match) and the `RAIOS_DATA_SB_V0` superblock and reports
typed `persist.layout` evidence (present/absent/invalid) — pure parsers in
raios-core (`gpt_layout.rs`/`seed_data_layout.rs`, 42 host tests incl. every
corruption fixture), read via the existing AHCI `READ_DMA_EXT` (no write path,
no driver rework); corruption/absent → fail-closed, kernel continues without
persistence; on-demand only (not at boot). **M7B-1 done:** the kernel SCANS the
SEED_DATA RECLOG region and validates the full RAIOSRC0 hash chain frame by
frame, reporting typed head/tail/count + torn-tail evidence via
`durable.record_log_scan` — still READ-ONLY (appends stay capability_denied);
pure frame codec + scan in raios-core (`durable_record_frame.rs`, 51 host tests
incl. bad-magic/hash/seq/torn/multi-sector), bounded region read (sector<4096),
fail-closed at the first invalid frame; a `--seed-reclog-fixture` builder flag +
child-VM torn/chain/empty fixtures prove it in-guest. **M7B-2 done (2026-07-06):
the FIRST REAL persistence WRITE.** raiOS builds a `raios.durable_record.v0`
boot-lifecycle frame chained to the scanned tail (seq=tail+1, prev=tail hash),
proves the target span lies fully inside the pinned RECLOG bounds, writes the
multi-sector span, READS IT BACK from disk, verifies it byte-identical
(readback sha256 == planned frame sha256) and re-parses it as a valid chained
frame, then reports `appended` — via `durable.record_log_append`. A NEW pinned
evaluator `raios-core/src/scoped_seed_data_append.rs` gates it (own EXPECTED_*
method/target/schema/RECLOG-marker pins + range/chain/write-readback-reparse
gauntlet, 32 distinct denials); the AHCI writer loops the existing
`issue_write_sector` over the frame sectors, validating every LBA in bounds
BEFORE any write (no partial-write escape), `issue_dma_command` untouched.
Store-full → deny (no rotation); torn tail → deny (no overwrite); within-boot
only (`persistence_claimed:false`, dev-tier, RAM ring still authoritative).
IMPORTANT correction: the max-effort scope caught that the map's older
"generalize the write-boundary chain" wording was STALE — that chain's booleans
are shared cross-target and flipping them would grant generic write to every
module; the real write went through a separate scoped evaluator (mirroring M3),
ZERO write-boundary edits.

**M7C-1 done (2026-07-06): boot control READ + state machine + SAFE posture
(read-only).** The kernel reads the BOOTCTL region's two 2048-byte ping-pong
slots (`RAIOSBC0` envelope wrapping a fixed binary boot-control payload — raiOS
has no kernel JSON reader, so on-disk is binary like the superblock/RECLOG),
picks the highest-valid-seq slot, and runs a pure fail-closed state machine that
selects the boot slot + posture (Normal/Probation/Safe/PersistenceUnavailable),
reported via `boot.control_read`. SAFE is entered on both-slots-invalid,
ambiguous equal-seq, or `safe_mode`; nothing is consumed/marked-good (that is
M7C-2). Writes NOTHING (WRITE_DMA_EXT uncalled). `MAX_PENDING_BOOT_ATTEMPTS=3`
is a v0-provisional, owner-overridable threshold. `current_boot_posture()` is
exposed for the next slice to consume.

**M7C-2 done → M7C COMPLETE (2026-07-06).** Three sub-slices: **2a** wired
`current_boot_posture()` into the durable append so SAFE/PersistenceUnavailable
deny it (`boot_control_safe_mode`) — the posture flag now does real work, not
paperwork; **2b** the FIRST BOOTCTL write — a booted kernel that passes the
map-3.4 success criteria (evaluated once) ping-pong-writes a `winner.seq+1`
record into the LOSER BOOTCTL slot via the NEW scoped target
`replace.boot_control.seed_data` (validate-all → write-one-slot → readback →
reparse → re-read-assert), advances `last_good` only on a genuine Probation
success, and appends a RECLOG audit record — crash-safe (a torn write damages
only the loser; the winner stays authoritative); **2c** offline owner tooling
(`scripts/switch-boot-slot.ps1`, dry-run by default, refuses `release/`+non-GPT)
+ a `--stage-slot`/`--set-pending` Python codec subcommand + a non-gating OVMF
observation. raiOS now has TWO of the three M7 scoped write targets live
(`append.record_log.seed_data`, `replace.boot_control.seed_data`); everything
else stays `capability_denied`. Still within-boot dev-tier
(`persistence_claimed:false`); deterministic firmware slot boot NOT claimed.
**M6D-2 done (2026-07-07).** The bridge from M6's RAM promotion loop to M7
persistence: on a verified dev-key promote (and un-promote), raiOS durably appends
a self-contained `raios.promotion_transaction.v0` RECLOG record (the full M6
evidence chain + the retained dev-key signature) via a NEW sibling scoped evaluator
`scoped_promotion_transaction_append` — a complete re-verification input so M7D can
recompute the attestation hash and re-verify the signature after reboot; dev-tier
throughout (M2a retained the signature DER in RAM; 2b writes the record, SAFE-gated,
complete-or-absent, nested-only best-effort). **M7D-1 done (2026-07-07): persistent artifact store.** On a verified M6 promotion
whose durable promotion transaction verified this boot, raiOS writes the promoted
module's wasm bytes as a content-addressed `RAIOSAR0` blob into ARTSTOR (the THIRD
scoped write target `blob.artifact_store.seed_data`) + chains a
`raios.artifact_persist.v0` RECLOG record binding blob offset/len/sha + the M6
hashes + the M6D-2 promotion-transaction hash (the RECLOG record is the single
commit point; a blob without its record is garbage). The code IS on disk, yet the
stored blob is completely INERT — zero load authority — until re-verified. Persist
denied in SAFE / when ARTSTOR is full / without a verified promotion transaction;
zero write-boundary or shared-evaluator edits. Split 1a (raios-core codec + two
scoped evaluators) → 1b (kernel writer + persist hook). **M7D-2 done (2026-07-07)
→ M7D COMPLETE → M7 PERSISTENCE FOUNDATION COMPLETE — THE PRODUCT MOMENT.** A
two-boot proof (`shadow-vm-persistence-reboot.ps1`, 85/85 predicates, 0 failures)
where a promoted service SURVIVES A REAL REBOOT and answers live on boot 2:
boot 1 persists a real P-256 dev-key-signed promotion; boot 2 re-verifies the whole
persisted chain — recompute blob sha, recompute the attestation hash, RE-RUN the
signature verify (never trusting the stored boolean) — then reaches execution only
through the SAME UNMODIFIED M6 `emit_load`/`emit_start` gate (no bypass, no "trusted
because stored"), granting `cross_reboot_proven:true` only on the repromoted record.
A corrupt blob, a tampered record, and SAFE posture each stay inert (denied /
skipped). Still honestly `dev_key_not_owner_sealed`, owner_sealed false. Verified:
two-boot 85/85, host 109/109 + 4/4, FULL 8168/8168, max adversarial review (two real
read-path defects — an `extract_sha256` `sha256:`-prefix bug and a matching fixture
inspector bug — found and fixed). This ends raiOS's current-boot-only era. Next:
**M8 — Recovery Agent Lifeline.**

**M5 Second Service Proof closed 2026-07-06.** Capability sentence
verified TRUE: adding svc.demo.echo cost only a descriptor + a small
state machine (net +1,064 kernel lines) — no new emitters, hash chains,
or harness profiles beyond generated needles; a hello copy would have
been ~19k. Echo loads/starts (runs its wasm under the M4 envelope)/
health/inventory/stop/drop through the shared shell, fail-closed to its
two imports. Evidence: quick `shadow-20260706-073224-7536.json` 486/486
(67 echo needles) + FULL `shadow-20260706-073633-23460.json` 7825/7825.
Slices M5-1..M5-5 (commits db52116..a10f209). The M2 record model, M3
durable-write posture, and M4 wasm envelope generalized.

**M4 Wasm Isolation closed 2026-07-06.** Capability sentence verified
TRUE: a service runs inside the in-kernel wasmi interpreter and cannot
call any authority outside its granted host-function imports — the
capability envelope IS the linker import set. Evidence: quick profile
`shadow-20260706-054603-21952.json` (465/465, 49 wasm needles): attested
echo guest runs under env.log+env.counter_get with fuel metering; a
forbidden-import module fails AT INSTANTIATION (link error); 4/4 trap
hardening cases (malformed / over-memory / fuel exhaustion / guest trap)
end as evidence, never a kernel panic. No-regression full profile
`shadow-20260706-055027-25628.json` 7825/7825. Slices M4-1..M4-7
(commits 27bfb56..328c90e); wasmi =0.31.2 vendored/pinned; wasm32 guest
attested via the P-256 chain.

**M3 First Durable Write closed 2026-07-06.** Capability sentence
verified TRUE: raiOS performs its first real, policy-authorized, durable
mutation — a rollback-transaction append to the RAIOS_AUDITRB_V0 LBA1
region with readback + hash inspection — and the hello hot-swap rollback
actually APPLIES using it (the transaction is the authority record).
Closure evidence: FULL profile `shadow-20260706-035553-13924.json`,
7825/7825 predicates (the count grew with the applied-path needles).
Slices M3-1..M3-6 (commits 8f1aa71..81fb374); fail-closed posture
preserved everywhere else (generic module authority denied, scratch
never durable authority, LBA0/boot metadata unwritable).

**M2 Ceremony Collapse closed 2026-07-06** under the re-scoped capability
sentence of ADR 0006 (byte-identical collapse floor accepted; vocabulary
compaction is an optional later slice, owner-overridable). Evidence: the
Batches 1–5 program (commits ad694f2..3a108c3), nine green FULL profiles
(7,814/7,814), agent layer 138k→126.5k lines at proven identical
behavior, zero-warning build, every file below size thresholds.

**M1 Testable Core closed 2026-07-05.** Capability sentence verified:
`raios-core` host tests pass in <1s (`cargo test --locked -p raios-core`,
9/9), and a second machine (GitHub Actions) builds AND smokes every
commit — run 28734873106 all green: host tests 15s, pinned kernel build
1m11s, headless QEMU quick profile 5m39s with report artifact. Slices:
M1-1 (`772003b`), M1-2 (`836d622`), M1-3 (`d57243b`), M1-3b
(`9db5321` + CRLF fix `943a9a0` — Windows checkout CRLF conversion broke
the signed source snapshots; forced LF).

**M0 Stabilize closed 2026-07-05.** Evidence: honest committed tree
(`0ee066e`, `9df2044`, `a6a8f56`, `e3984fb`); full profile green
(`shadow-20260704-184615-9224.json`, 7814/7814 predicates, SHA-256
`68c8e160849ca9333867ea6007013b2e49d6f39e4e7e4930b761944967ba96ee`); all
recent failures classified (failure classification log in
`docs/PROJECT_STATUS.md`); harness transport instrumentation landed —
every report now carries a `serial_transport_failure` classification
(qemu_exited / listener_missing_process_alive /
connect_timeout_listener_present), `qemu_process` teardown snapshots, and
a structured `stderr_log` block; a dead VM aborts the run immediately
instead of burning the timeout. Verified: quick profile
`shadow-20260705-094659-19752.json`, 417/417 predicates.

Done in M1 so far (2026-07-05): slice M1-1 — `raios-core` `no_std`
workspace crate with `sha256_bytes`/`sha256_hex`/`ByteSink`;
`descriptor_sources.rs` deduplicated; quick profile
`shadow-20260705-100850-5584.json` 417/417. Slice M1-2 — the pure protocol
parsers (`method_eq`, `method_head_eq`, `parse_sha256_ref`,
`parse_current_boot_event_sequence`) moved into `raios-core` with
truth-table host tests (`cargo test --locked -p raios-core`, 9/9); kernel
keeps thin wrappers/re-exports; quick profile
`shadow-20260705-101746-21240.json` 417/417. Note: the `hello_service.rs`
sha256 duplicate stays until M2 — replacing it invalidates the signed
Hello source snapshot (`artifact_content_source_sha256`); that dedup
belongs to the M2 de-hello-ify slice.

Done in M2 so far (2026-07-05): slice M2-1 — `raios-core::record` exists:
`Value` enum (Null/Bool/U64/Str/Sha256/EventSequence/Array/ordered
Object), ONE serializer `write_json` reproducing the kernel's exact JSON
style (CRLF, two-space indent, kernel escaping table), and
`sha256_of_json` implemented through a hashing `ByteSink` so serializer
and hasher cannot diverge; 14/14 host tests. Kernel untouched. Key
finding for all ports: today's kernel hashers hash `key=value` LINES,
not JSON bytes (`module_evidence.rs:4538-4592`) — every ported gate must
consciously map its old line-hash convention.

Slice M2-2 done (2026-07-05): first emitter port —
`agent_protocol_recovery_artifact_selftest_emit.rs` now builds
`raios_core::record::Value` and serializes through the new `SerialSink`
adapter (`agent_protocol_support.rs`); byte-identical output proven by
the recovery profile (`shadow-20260705-105605-12660.json`, 3644/3644,
859 selftest predicates). Net line delta -1 (the one-time SerialSink
adapter cost is now paid; subsequent ports should delete heavily).

Slice M2-3 done (2026-07-05): batch port of
`agent_protocol_recovery_command_handler_emit.rs`,
`agent_protocol_recovery_status_handler_emit.rs`,
`agent_protocol_recovery_rollback_preview_emit.rs` (net -25 lines);
recovery profile byte-identical (`shadow-20260705-111122-15364.json`,
3644/3644).

Slice M2-4 done (2026-07-05): batch port of command_dispatch_emit,
command_admission_emit, command_effect_emit, load_binding_emit (net -230
kernel lines; raios-core gained Value::InlineObject +
write_json_fields); recovery profile byte-identical
(`shadow-20260705-114327-7224.json`, 3644/3644). One classified
intermittent failure on the first verify attempt (`qemu_exited` in 0.5s
— the M0-2 instrumentation working as designed; see failure log:
suspected timing-dependent guest reset after `memory.recent_events`,
pre-dates M2).

Slice M2-5 done (2026-07-05): batch port of memory_provenance_emit,
persistence_emit, command_envelope_emit, loader_runtime_emit,
rollback_transaction_emit; recovery profile byte-identical
(`shadow-20260705-120458-16280.json`, 3644/3644). 13 recovery emit
modules now render through the record model.

Slices M2-6/M2-7 done (2026-07-05): the silent guest crash is root-caused
and FIXED — a 3.78 MB `EventSnapshot` stack copy on every
`memory.recent_events` corrupted return frames (~50% crash rate).
Checkpoint bisection localized it; the ring is now iterated one event at
a time. Proof: 5/5 recovery runs green + final clean run
`shadow-20260705-125828-3624.json` (3644/3644). Details in the failure
classification log (PROJECT_STATUS).

Slice M2-8 done (2026-07-05): batch port of command_body_emit,
rollback_apply_emit, memory_write_emit, durable_write_emit,
service_inventory_effect_emit (net -124 lines) + dead EventSnapshot
struct deleted; recovery profile byte-identical
(`shadow-20260705-131513-9748.json`, 3644/3644). 18 recovery emit
modules now render through the record model.

Slice M2-9 done (2026-07-05): final recovery batch —
lifeline_command_vocabulary_emit, lifeline_protocol_emit,
target_binding_emit, artifact_reference_emit ported (net -309 lines).
**The entire recovery emit surface (22 modules) now renders through the
single record model.** Recovery profile byte-identical
(`shadow-20260705-133645-13328.json`, 3644/3644).

Slice M2-10 done (2026-07-05): module-boundary porting map produced and
saved at `docs/plan-reviews/m2-module-boundary-porting-map-2026-07-05.md`
— 32 files classified SAFE/COUPLED/NOT-emitter with hash sites, profile
coverage, batch order, and the hard rule that key=value line-hash inputs
must never become JSON.

Slice M2-11 done (2026-07-05): map Batch 1 — five SAFE write-boundary
emitters ported (emit, availability, write_policy, append_engine,
append_intent; net -59 lines + adapter); module-audit-rollback profile
byte-identical (`shadow-20260705-140011-21652.json`, 1626/1626).

Slice M2-12 done (2026-07-05): map Batch 2 — storage_layout,
append_contract, write_boundary_boundary ported (net -290 lines;
raios-core gained Value::HexBytes + TrimmedAsciiBytes, 17 host tests);
verified byte-identical by module-audit-rollback
(`shadow-20260705-142306-4516.json`, 1626/1626) AND quick
(`shadow-20260705-142625-22816.json`, 417/417).

Slice M2-13 done (2026-07-05): map Batch 3 — loader_identity,
loader_artifact_hash_binding, loader_fact, service_slot_allocator ported
(net -126 lines; raios-core gained Value::InlineArray, 18 host tests);
module-audit-rollback byte-identical
(`shadow-20260705-144753-8004.json`, 1626/1626).

Slices M2-14/M2-15 done (2026-07-05): map Batch 4 — loader_runtime fully
ported (-260 lines), load_gate_selftest_emit fully ported (-41),
load_gate_render largely ported (-167; a few heavily interleaved
streaming sections honestly left as-is). **FULL profile green over the
entire M2 state**: `shadow-20260705-152745-17896.json`, 7814/7814
predicates, 334 commands, 17.1 min.

Slice M2-16 done (2026-07-05): coupled Batch 5 first packet — JSON
rendering of grant, service_slot, append_payload_hash, audit ported with
all hash-input sites documented and untouched; verified by
module-audit-rollback (`shadow-20260705-160504-8620.json`, 1626/1626)
AND full (`shadow-20260705-160858-23032.json`, 7814/7814).

Slice M2-17 done (2026-07-05): coupled Batch 5 second packet —
attestation, approval, reference JSON rendering ported, all hash sites
untouched (worker diff check: zero changed hash-input lines); verified
by module-audit-rollback (`shadow-20260705-164930-25652.json`,
1626/1626) AND full (`shadow-20260705-165249-26488.json`, 7814/7814).
**The module-boundary porting map is fully executed** — all SAFE and
COUPLED emit surfaces render through the record model.

Slice M2-18 done (2026-07-05): de-hello-ify plan produced and saved at
`docs/plan-reviews/m2-de-hello-ify-plan-2026-07-05.md` — full section
map of hello_service.rs (22,705 lines), the signed source snapshot chain
explained end-to-end (build.rs hashes ONE file's bytes; re-signing via
target/descriptor-resign), split feasibility (needs ordered source-set
hashing in build.rs first), 5-slice plan, risks (attestation must keep
covering moved code; no .gitattributes = CRLF hazard for signed bytes).

De-hello-ify Slice 1 done (2026-07-05): build.rs hashes an ordered
source SET with length-framed entries (currently exactly
hello_service.rs); the hello_service.rs sha256_bytes duplicate is gone
(raios_core::sha256_bytes); .gitattributes protects signed bytes from
EOL conversion; v1/v2 identity descs re-signed. Verified: quick
(`shadow-20260705-172534-6696.json`, 417/417) + hello-rollback-dry-run
(`shadow-20260705-172834-16852.json`, 203/203) — the guest re-validates
the new signatures at runtime.

De-hello-ify Slice 2 done (2026-07-05): hello_service.rs (22,705 lines)
mechanically split into 16 modules under `hello_service/`, largest
4,557 lines — all below the AGENTS.md thresholds. Every module is in the
build.rs attestation source set (root first, then declaration order);
v1/v2 re-signed. Verified: quick (`shadow-20260705-173919-20792.json`,
417/417) + hello-rollback-dry-run (`shadow-20260705-174304-16956.json`,
203/203).

De-hello-ify Slice 4 done (2026-07-05, pulled ahead of Slice 3): Hello
emitters ported to the record model (-233 lines; only the hash-adjacent
staged evidence writer stays manual; hash-module diff verified empty);
re-signed; verified quick (`shadow-20260705-181004-22820.json`, 417/417)
+ hello-rollback-dry-run (`shadow-20260705-181324-8968.json`, 203/203).

De-hello-ify Slice 3 done (2026-07-05): generic rollback machinery
extracted into generically-named modules (rollback_writer_gate,
rollback_authority_gates, storage_authority_gate,
rollback_writer_bindings; old files are shims), hello_ prefixes dropped
from the generic gate-hash symbols with quoted-literal sequence proofs
(all emitted strings byte-identical); source set extended, re-signed.
Verified: quick (`shadow-20260705-182727-22700.json`, 417/417) +
hello-rollback-dry-run (`shadow-20260705-183114-27284.json`, 203/203).

De-hello-ify Slice 5 done (2026-07-05): ServiceDescriptor introduced and
threaded (ids, aliases, slots, inventory, small event-log constructors);
HelloServiceLifecycleBinding + global capability table recorded as
M5-prep. Verified quick 417/417 + hello-rollback-dry-run 203/203.

**M2 midpoint evaluation (2026-07-05, honest verdict): M2 stays OPEN.**
Full profile green over the final state
(`shadow-20260705-184946-16520.json`, 7814/7814 — third full green of
the day). Achieved: single record model everywhere (emitter/hasher
divergence structurally impossible), all files below size thresholds
(largest 4,557 lines), attested source-set chain, generic rollback
modules, ServiceDescriptor. NOT achieved: the ~10x size target — the
agent layer measures ~138k lines (agent_protocol* 101,763 +
hello_service/ 22,306 + event_log* 14,296) vs the ~20k target, because
byte-identical ports delete little by design. The remaining M2 items
(table-driven dispatch, named key=value command arguments, shared
CommandBindings, selftest scaffolding collapse) are where the mass
deletion happens — the record model was the prerequisite, not the
collapse itself.

Collapse map produced (2026-07-05):
`docs/plan-reviews/m2-collapse-map-2026-07-05.md` — census (recovery
40.7k is the heaviest family; top pattern: the 7.1k-line flattened event
binding emitter), four design sketches, 6-batch plan. Reality check:
byte-identical collapse bottoms out ~55-75k lines; ~20k requires
output-shape/vocabulary changes (batch 6 = OWNER DECISION + needle
updates + likely an ADR).

Collapse Batch 1 done (2026-07-05): MethodEntry dispatch table replaces
the 168-branch chain, console routing, and envelope target enumeration;
146 dead *_method helpers deleted; net -1,653 lines. Verified by FULL
profile (`shadow-20260705-194232-27296.json`, 7814/7814 — fourth full
green of the day).

Collapse Batch 2 part 1 done (2026-07-05): shared
CommandBindings/StageBinding replace 15 cloned input/check struct pairs
(authorization, effects, dispatch); one shared positional reference
parser replaces 15; net -1,159 lines. Recovery profile byte-identical
(`shadow-20260705-202307-20408.json`, 3644/3644). Deferred by the
worker: admission family + execution structs.

Collapse Batch 2 complete (2026-07-05): part 2 converted the admission
family and execution stages onto CommandBindings and routed the last 7
positional parsers through the shared parser (only the shared parser
still splits input); recovery profile byte-identical
(`shadow-20260705-203821-18288.json`, 3644/3644).

Collapse Batch 3 family 1 done (2026-07-05): generic selftest runner
(SelftestCase/CaseSpec + run_selftest_cases + shared record-model case
emitter in agent_protocol_support.rs); all 13 recovery command-reference
selftest families converted to const case tables (the selftest file
alone dropped ~1,950 lines of hand-written factories); valid base hashes
now come from the production evaluator instead of duplicated
construction. Recovery profile byte-identical
(`shadow-20260705-210517-7660.json`, 3644/3644).

Collapse Batch 3 complete (2026-07-05): ALL selftest families (recovery,
module load-gate, write-boundary, grant/audit/attestation/approval/
service_slot) run on the shared runner as const case tables; only the
service_slot_allocator's 29 multi-source cases stay special (reported).
HONEST accounting: the module-family conversion was net +1,015 lines
(their factories were thinner than recovery's; the map's -12-16k
estimate was recovery-shaped). Value delivered: uniformity — one runner,
one emitter, production-evaluator-derived base hashes — which is the
prerequisite for the batch-6 host-test migration that WOULD delete the
tables wholesale. Verified: module-audit-rollback
(`shadow-20260705-221938-18704.json`, 1626/1626) + FULL
(`shadow-20260705-222328-20516.json`, 7814/7814 — fifth full green of
the day). Ops note: the harness never cleans its temp dirs — 356
raios-shadow-* dirs (~23 GB) filled the disk mid-batch; cleaned
manually; a teardown-cleanup slice is queued.

Both maintenance slices done (harness temp cleanup `9b18a88`;
zero-warning build `c490f53`). Collapse Batch 4 (emitter half) done
(2026-07-05/06): emit_event_bindings' Hello branch (~4.6k lines of flat
emission) collapsed to a 1,120-entry const field-descriptor table +
one generic loop — net ~-3,365 lines in agent_protocol_memory.rs. The
golden needles EARNED THEIR KEEP: the worker's own field comparison
claimed 1120/1120 identical, but the quick profile caught 10 genuinely
dropped fields (scratch no_metadata_overlap + 9 append-record/sector
fields); orchestrator restored them at exact old positions with a
scripted old-vs-new key-order proof (968/968, order identical).
Verified: quick 417/417, hello-rollback-dry-run 203/203, FULL
`shadow-20260705-235005-11568.json` 7814/7814 (sixth full green).

Collapse Batch 4 fully done (2026-07-06): all 86 remaining
emit_event_bindings variants converted to descriptor tables WITH
per-variant scripted key-diff proofs (0 missing, order identical
everywhere — the M2-32 lesson institutionalized); net -1,703 more lines;
agent_protocol_memory.rs is now 3,229 lines (was 8,275 at M2 start).
Verified: quick 417/417 + FULL `shadow-20260706-002904-24144.json`
7814/7814 (seventh full green). Only ModuleLoadGate/
RecoveryArtifactLoadDenied render elsewhere (out of that packet's
scope).

Collapse Batch 5 done (2026-07-06): the Hello hash-input construction
collapsed to tables — 11 hash blocks in storage_authority_gate.rs and 32
binder blocks in rollback_writer_bindings.rs, each with scripted
byte/order-identity proofs (target/m2-34-debug/); net -4,450 lines;
re-signed; emitters.rs residuals honestly left (irregular interleaving).
Verified: hello-rollback-dry-run 203/203 + FULL
`shadow-20260706-010427-24984.json` 7814/7814 (eighth full green).

**The byte-identical collapse program (Batches 1-5) is complete.**
Cumulative M2 deletion: ~-18,500 kernel lines at byte-identical
behavior. Remaining to reach the ~20k target: Batch 6 (vocabulary
compaction / host-test migration — changes output shape, needs OWNER
DECISION + needle updates + an ADR).

Checkpoint resolved via ADR 0006 (M2 closed re-scoped; batch-6
vocabulary compaction optional/deferred, owner may still choose it).

M3-1 done (2026-07-06): design map saved at
`docs/plan-reviews/m3-durable-write-map-2026-07-06.md` (region contract,
10-gate denial chain, minimal single-use hello-scoped grant, transaction
flow, 5-slice plan, needle-flip list, authority-leak risks).
M3-2 done (2026-07-06): the scoped-grant evaluator exists as PURE
host-tested logic (`raios-core/src/scoped_rollback_apply.rs`) and its
decision is emitted as a NEW standalone `RAIOS_ROLLBACK_APPLY_SCOPE_DECISION`
record after the rollback-apply response — additive only, all existing
needles green (`shadow-20260706-022318-7556.json`, 203/203); re-signed.

M3-3 done (2026-07-06): **the first policy-authorized durable write in
raiOS history.** A positive scope decision routes into the existing AHCI
LBA1 target-region path via thin authorized-append wrappers (no driver
behavior change); the new `raios.scoped_rollback_authorized_append.v0`
evidence records write + readback + inspection hashes; rollback state
application stays denied (slice 3). Verified: hello-rollback-dry-run
207/207 incl. 4 new authorized-append needles
(`shadow-20260706-024250-3116.json`) + quick regression 417/417.

M3 slices 3-4 done (2026-07-06): **the hello rollback ACTUALLY APPLIES**
— after the verified chain (scope decision → authorized append →
readback match → inspection match) the state machine transitions v2→v1
citing the transaction/write-readback/inspection hashes as its authority
record (raios.ram_only_hello_service.rollback_apply_applied_binding.v0);
post-apply recovery.rollback_inspect retains the applied transaction
evidence (rollback_applied_transaction_inspected). Deny paths unchanged.
Quick/dry-run profile needles updated to the applied reality (two
worker-invented needle fields corrected by the orchestrator against the
actual serial output). Verified: hello-rollback-dry-run 214/214, quick
416/416, module-audit-rollback 1626/1626 (generic module authority
still denied).

M3 closed (see above).

M4-1 done: design map at
`docs/plan-reviews/m4-wasm-isolation-map-2026-07-06.md`. M4-2 done
(2026-07-06): wasmi =0.31.2 vendored and pinned (permissive licenses
verified; wasmparser-nostd is Apache-2.0-with-LLVM-exception), wired via
[patch.crates-io], compiled INTO the no_std kernel with a
wasmi::Module::new compile-proof (seed-kernel/src/wasm_runtime.rs);
zero-warning build; quick profile green
(`shadow-20260706-044149-25532.json`, 416/416).

M4 slice 2 done (2026-07-06): the first Wasm guest exists —
wasm-guests/svc-demo-echo (55-line no_std cdylib, exports
raios_service_main, imports exactly env.log + env.counter_get), built
via scripts/build-wasm-guest.ps1 to an ATTESTED artifact
(seed-kernel/artifacts/svc.demo.echo.wasm, descriptor + P-256 signature,
fail-closed build verification at build.rs:414/420); the kernel
heap-validates the embedded bytes via wasmi::Module::new (no execution
yet). Verified: quick `shadow-20260706-050625-19120.json` 416/416.
(Worker was sandbox-blocked on rustup; orchestrator installed the
wasm32-unknown-unknown target for nightly-2024-10-15.)

M4 slices 3-4 done (2026-07-06): **the isolation boundary is real and
VM-proven.** The attested echo module executes inside the wasmi envelope
(Linker defines exactly env.log + env.counter_get; fuel-metered;
heap-only; non-panicking hosts); `wasm.echo_probe` surfaces typed
evidence; the negative proof — a module importing env.forbidden_write —
fails AT INSTANTIATION with a link error. Verified in the VM: quick
profile `shadow-20260706-052847-5000.json`, 441/441 incl. 25 new wasm
needles (guest log line observed on serial; exceed-capability link
failure evidenced).

M4 closed (see above).

M5-1 done: design map at
`docs/plan-reviews/m5-second-service-map-2026-07-06.md`. Recommended
path: extract a shared `current_boot_service` shell driven by
ServiceDescriptor (NOT a hello copy = 19k+ lines = fail). Estimate
~900-1,800 net lines for the minimal echo surface. Honest verdict: the
live positive service path is still hello-hardcoded at dispatch,
inventory, event-log binding, descriptor lookup, capability table, and
audit schemas — M5 legitimately includes generalizing those.

M5 slice 1 done (2026-07-06): shared `current_boot_service.rs` extracted
(ServiceDescriptor + generic ServiceState + descriptor/alias target
matching + health/activation helpers); hello uses it byte-identically
(scripted proof: 0 changed output sources, 168 route cases 0 mismatches);
added to the attested source set + re-signed. Verified: hello-rollback
214/214 + FULL `shadow-20260706-063034-21036.json` 7825/7825.

M5 slice 2 done (2026-07-06): descriptor-driven
`record_service_lifecycle`/`record_service_health` in event_log.rs;
hello wrappers delegate through them (binding type + emitter untouched,
so no re-sign needed; byte-identity proof 1130/1130 keys, order
identical). Verified: hello-rollback-dry-run
`shadow-20260706-065903-26760.json` 214/214.

M5 slice 3 done (2026-07-06): echo has a service identity —
ECHO_SERVICE_DESCRIPTOR (echo-valued ids/aliases/caps/slots/inventory)
in echo_service.rs + a signed current-boot load descriptor
(svc.demo.echo.current_boot_load.*) binding the echo wasm artifact and
authorizing current-boot wasm execution under env.log+env.counter_get;
build.rs verifies it fail-closed. Verified: quick
`shadow-20260706-071031-25824.json` 465/465.

M5 slices 4-5 done (2026-07-06): **echo is a real second current-boot
service.** module.load_ephemeral / service.start (executes the wasm
under the M4 envelope) / service.health / service.inventory /
service.stop / service.drop all drive svc.demo.echo through the shared
current_boot_service shell + descriptor-driven event records + echo
dispatch rows; echo stays fail-closed (only its two imports; no durable
write/rollback/broad mutation). Verified: quick
`shadow-20260706-073224-7536.json` 486/486 incl. 67 echo needles.

**M5 VERDICT — PASS.** Total second-service kernel cost (db52116..HEAD,
seed-kernel/src): net +1,064 lines (M5-2 +124, M5-3 +49, M5-4 +91,
M5-5 +800) — a descriptor + a small state machine reusing the M2 record
model, M3 durable-write posture, and M4 wasm envelope. A hello copy
would have been ~19k. The architecture generalized.

M5 closed (see the cursor top). 

Exact next task:

```text
M6-1 done: design map at
`docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`. M6 splits into
M6A external candidate identity → M6B verified grant → M6C promotion →
M6D rollback (6-10 slices). The denial edifice was built for exactly
this loop; M6 turns hash-reference denials into real authorities.

M6A-1 done (2026-07-06): the runtime candidate-intake MECHANISM exists.
`module_candidate_intake.rs` accepts arbitrary bounded bytes
(`intake_external_wasm_candidate`), computes the SHA-256 in-guest,
validates via `wasm_runtime::validate_module_bytes` (wasmi::Module::new),
and returns an inert `ExternalWasmCandidate` — `load_attempted`,
`authorizes_load`, `execution_attempted`, `authorizes_execution`,
`writes_persistent_state` hard-false on EVERY path. Probe covers three
real cases (echo bytes valid+retained, malformed retained-invalid,
oversize>256KiB rejected-not-retained); evidence emitted through the
record model on the existing `wasm.echo_probe` response. LOAD STAYS
DENIED — no grant, no promotion, no authority file touched. Per the v0
scope the byte SOURCE is still a fixed test vector labeled
`pending_m6a_slice2`; real external delivery arrives with slice 2.
Verified: quick `shadow-20260706-093418-2968.json` 562/562 incl. 8
`quick:wasm_echo_probe_candidate_*` needles (one flaked-then-green
host-transport run classified in PROJECT_STATUS).

M6A-2a done (2026-07-06): REAL external delivery over the serial console.
New `seed-kernel/src/module_candidate_channel.rs` reassembles a real
external `.wasm` sent as base64 chunks (bounded RAM buffer capped at
MAX_EXTERNAL_WASM_CANDIDATE_BYTES, local base64 decoder, fail-closed
discard on malformed/overflow/empty); finalize's ONLY sink is
`intake_external_wasm_candidate`. Two registered read-methods
`module.submit_candidate_chunk` / `module.submit_candidate_finalize` (no
new MethodAction, no dispatch-arm behavior change). The delivery label is
now the real `serial_console_base64_chunks_v0`; `pending_m6a_slice2` is
retired. Verified: focused `shadow-20260706-102027-16828.json` 176/176
(real 4205-byte echo wasm delivered, exact SHA f81f9442…abd2, retained
inert, all denials false, malformed-discard + VM-responsive negative) +
quick regression `shadow-20260706-102839-18048.json` 562/562.
Adversarially reviewed: no reachable load/grant/instantiate/execute/persist
sink, no panic/OOB/bound-bypass/lock/state-leak. Known residual: wasm
validation (`wasmi::Module::new`) runs on attacker bytes, bounded but not
time/fuel-bounded — a later hardening candidate (see PROJECT_STATUS).

M6A-2b done (2026-07-06): the module-evidence cross-check now evaluates
the REAL delivered-candidate artifact identity. In
`shadow-vm-smoke-profile-full-module-evidence.ps1` the synthetic `2222…`
candidate artifact hash is replaced by the on-disk echo wasm SHA
(computed via `Get-FileSha256OrNull`, anchored to the known ECHO hash
`f81f9442…abd2` == the intake `artifact_sha256`), flowing through the
grant + artifact-reference canonicals and their echo assertions; a new
`protocol:module_evidence_real_candidate_sha_matches_echo` predicate and
the existing `can_load_now: false` assertions prove real identity + load
denied. Zero kernel change. Honest gap: vm_test_report/local_attestation
identities remain synthetic (report-file hash is post-run; a real binding
is a later land-if-cheap step). Verified: FULL profile
`shadow-20260706-104758-19976.json` 8160/8160.

Exact next task:

```text
M6B slice 1 (verified grant) — FIRST AUTHORITY STEP, owner go-ahead
required: turn the computed capability grant for the delivered candidate
from a retained hash-reference diagnostic into an authorizing decision
(bind manifest + real artifact SHA + vm_test_report + local attestation),
while still denying LOAD until audit/rollback/slot exist. This is the
first slice that moves a `Denied*`/hash-reference toward a real authority
— design it fail-closed, evidence-bound, one narrow capability. See the
m6 map: M6B (verified grant) → M6C (promotion) → M6D (rollback).
```
```

## Capability Milestones

The roadmap's backbone is now capability milestones, not schema phases. A
milestone is done when its capability sentence is true and verified, not when
its evidence is described. Denial-gate and schema-only slices do not advance
milestones (ADR 0005 §3).

### M0 Stabilize (active)

Capability sentence: "The project's own pre-commit gate is green and the
repository history is truthful again."

- Working tree committed in honest, boundary-scoped commits.
- Full Shadow VM profile green; the `audit.events 256` serial failure
  root-caused and classified (guest bug vs host transport).
- Recent real predicate failures (7005/7006, 7380/7381) explained.

### M1 Testable Core

Capability sentence: "Kernel gate/evidence logic runs and passes as ordinary
host `cargo test` in seconds, and a second machine (CI) builds and smokes
every commit."

- Extract a `no_std` library crate from the flat binary crate: types, eval
  logic, hash chains, descriptor parsing, behind a `ByteSink` trait (serial
  impl in the kernel, `Vec<u8>` impl in tests).
- Host `cargo test` coverage for gate truth tables, parse round-trips, and
  hash-chain vectors.
- Minimal CI (GitHub Actions): pinned-toolchain build, image packaging,
  headless QEMU quick profile under TCG, report uploaded as artifact.

### M2 Ceremony Collapse

Capability sentence: "The agent layer is small enough for an agent to fully
model again (~10x smaller), with byte-identical serial output proven by the
existing golden-string harness."

- One typed `Value`/record model + one JSON serializer + one canonical
  hasher over the same structure (emitter/hasher divergence becomes
  impossible).
- Port gates slice-by-slice; every porting slice deletes more lines than it
  adds; harness needles prove byte-identical output.
- Table-driven method dispatch; named `key=value` command arguments;
  shared `CommandBindings` struct replacing per-stage 25-field clones.
- De-hello-ify: `event_log.rs` and gate constants parameterized by a
  `ServiceDescriptor`; `hello_service.rs` split below the AGENTS.md size
  thresholds.
- Target: agent layer under ~20k lines; rustfmt runs clean on all sources.

### M3 First Durable Write

Capability sentence: "raiOS performs its first real, policy-authorized,
durable mutation: an audit/rollback transaction append to the
`RAIOS_AUDITRB_V0` LBA1 region — and a hello hot-swap rollback actually
applies using it."

- Grant the first narrow write authority through the existing gate chain
  (AHCI write/readback already verified; this is authority policy, not
  driver work).
- Real transaction append + readback + hash verification; rollback apply
  transitions from `capability_denied` to a real, evidenced state change
  with the transaction as its record.
- The existing denial edifice becomes a functioning transaction system.

### M4 Wasm Isolation

Capability sentence: "A service runs inside an in-kernel Wasm interpreter
and physically cannot call an authority outside its granted host-function
imports."

- Vendored, pinned `no_std` Wasm interpreter (wasmi-class, no JIT).
- Hello (or echo) compiled to wasm32, loaded as a real module artifact
  through the existing descriptor/attestation chain.
- Capability envelope = linked import surface; a deliberate
  exceed-capability test fails at the boundary, not at a policy string.

### M5 Second Service Proof

Capability sentence: "Adding `svc.demo.echo` costs only a descriptor and a
state machine — no new emitters, hash chains, or harness profiles beyond
generated needles."

This is the acceptance test that M2's refactor and M4's runtime actually
generalize. If a second service still costs tens of thousands of lines, the
architecture is not what the ADRs claim.

### M6 Promotion Loop v0

Capability sentence: "One external, AI-authored artifact travels the full
loop: authored, Shadow-VM verified, capability-granted, promoted into the
live system, and rolled back — with evidence at every step."

This is the project's first true product milestone; everything before it is
substrate.

### M7 Persistence Foundation (pre-planned)

Capability sentence: "A promoted artifact and its evidence survive a
reboot, and a bad boot automatically falls back to last-good." GPT
`SEED_ESP_A/B` + `SEED_DATA` per `docs/image-layout-v0.md`; sub-milestones
M7A layout detection (read-only) → M7B durable record store → M7C boot
control/A-B/SAFE → M7D persistent artifact store + evidence-gated boot-time
re-promotion. Map: `docs/plan-reviews/m7-persistence-map-2026-07-06.md`.

### M8 Recovery Agent Lifeline (pre-planned)

Capability sentence: "When the world above breaks, a minimal pinned
serial-first path still diagnoses and restores last-good — restoring
known-good state only, never promoting anything new." (ADR 0003 /
archived Phase 8.) Map:
`docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md`.

### M9 Durable Memory & Context Broker v1 (pre-planned)

Capability sentence: "Typed, classified, provenance-bound memory records
survive reboot with append/readback evidence, and budgeted
agent_context.v0 packets draw on them with explicit omissions — provider
export stays fail-closed end-to-end." (ADR 0004 Phase D.) Map:
`docs/plan-reviews/m9-durable-memory-map-2026-07-06.md`.

**R1 — HARD M9C broker precondition (from M9A-3a, commit 2bef6bf):** supersede-not-overwrite
is split across the read/write boundary. M9A-3a closed the WRITE side (audit kinds —
capability_grant/denial, promotion_tx_ref, rollback_tx_ref, export_audit — can never be
AUTHORED as superseding records). The READ side is R1: **when the M9C broker resolves
supersession, it MUST IGNORE any supersede link whose TARGET record is an audit kind** (a
non-audit `decision` naming a `capability_denial` id cannot be denied at write time — that
needs the target's kind = reparsing the log = building the broker early). **No reader that
resolves supersession may ship before R1**, or an audit trail could be silently hidden.
Dangling-supersede existence is likewise a broker concern (harmless under append-only +
read-time resolution), NOT a write gate.

### M10 Provider Trust Hardening & Adapters (pre-planned)

Capability sentence: "Provider connections validate real certificate
chains under an honest time authority, through a provider-agnostic
adapter proven by a second provider." Map:
`docs/plan-reviews/m10-provider-trust-map-2026-07-06.md`.

### M11 Kernel Slimming / Services-out-of-kernel (pre-planned)

Capability sentence: "The kernel does not parse the internet — TLS/HTTP
protocol logic runs as a replaceable, capability-scoped Wasm service, and
the kernel measurably shrinks." The concrete path toward the end vision:
a slim permanent core (boot + network bring-up + ledger + recovery) with
everything else loaded as replaceable services. Map:
`docs/plan-reviews/m11-kernel-slimming-map-2026-07-06.md`.

### M12+ (direction, not slice-planned)

Bare-metal Wi-Fi vs USB-Ethernet, external artifact distribution
(unparking `ota/`/`registry/`/`fake-cloud/` — requires a new ADR),
re-binding to new hardware, core-generation handoff. Direction doc:
`docs/plan-reviews/m12-plus-direction-2026-07-06.md`.

All M7-M11 maps were pre-planned 2026-07-06 (before M6 closed) and carry
a MANDATORY Slice 0 that revalidates every file:line claim against HEAD
before implementation. Execution procedure for orchestrators (including
cheap/mid-tier models): `docs/ORCHESTRATOR_PLAYBOOK.md`.

## Active Execution Rules

Standing rules live in `AGENTS.md` (Definition of Done, Red Gate Rule,
Commit Discipline, Failure Classification, End-of-Session Checks). Compact
reminders:

- Every slice states what the system can now DO that it could not before.
- No new `raios.*.v0` schemas as hand-rolled emit/hash code; after M2, new
  schemas are record-model entries only.
- Match verification cost to slice risk exactly as before (quick often,
  full rarely, focused when the touched boundary is risky); never skip VM
  evidence for trust/storage/rollback/recovery/authority/descriptor/boot
  changes.
- Execution model: a master agent plans from this roadmap and dispatches
  worker agents with narrow, verifiable tasks split by ownership boundary
  (runtime/loader, provider trust, UI/input, VM harness, docs). Workers
  return one integrated slice with a capability sentence plus verification
  evidence. Parallel dispatch only across non-conflicting boundaries.

## Product Thesis

raiOS is a personal, self-modifying, bare-metal operating system where AI
can change the machine only through evidence-gated, capability-scoped system
transactions that can be rolled back. It is bonded to one machine and one
user, small enough for an agent to fully model, and anchored in an immutable
recovery core. It is not a Linux distribution, does not run on a Linux host
(ADR 0005), and does not port the Codex CLI into the kernel (ADR 0001).

## North Star Architecture

```text
permanent core -> recovery agent lifeline -> live service graph
-> agent workspace -> shadow VM/test world -> persistence/rollback
```

The permanent core holds only survival mechanisms. Everything else —
UI, console, input, USB, networking, provider adapters, diagnostics, agent
tools, builder — becomes a replaceable service, first as Wasm modules
(ADR 0005), long-term as a native service graph with versioned state and
migrators (ADR 0003). The provider/OpenAI path is a service, not the core
identity. The system itself is the memory: typed, classified,
provenance-bound facts feeding budgeted `agent_context.v0` packets
(ADR 0004).

## Planning Gates

Unchanged from the May 2026 plan-review consensus, now with the milestone
overlay:

```text
fail-closed TLS/provider trust        (implemented, pin-only)
-> read-only agent protocol           (implemented)
-> typed system.snapshot.v0           (implemented)
-> static service.inventory.v0        (implemented)
-> capability policy v0               (implemented)
-> read-only memory.context           (implemented)
-> RAM-only event.log.v0              (implemented)
-> module_manifest.v0                 (implemented, non-authorizing)
-> vm_test_report.v0                  (implemented)
-> raios.local_attestation.v0         (implemented, non-authorizing)
-> live loading denied until evidence matches   (M3/M4 make this real)
```

The direct OpenAI path remains a normal provider-service candidate, never
the recovery lifeline.

## Phase Map (legacy)

The former Phase 0–10 structure is retired as the planning backbone (phases
had become taxonomy, not gates — Phase 7 scope shipped inside Phase 6 while
Phase 6's own definition of done was unmet). Full phase definitions remain in
`docs/archive/roadmap-2026-07-04-pre-restructure.md`. Rough mapping: Phases
0–5.14 are the implemented substrate above; Phase 6/7 work continues inside
M3–M5; Phase 8 (recovery lifeline) and Phase 10 (persistence/core handoff)
live in M7+; Phase 9 (Shadow VM acceptance) is realized by M6.

## Documentation Ownership

- `README.md`: product thesis, quickstart, concise current reality only.
- `AGENTS.md`: stable startup checklist, standing engineering rules, durable
  current facts only.
- `docs/ROADMAP.md`: this file — direction, milestones, compact cursor.
- `docs/PROJECT_STATUS.md`: authoritative detailed status, exact next task,
  verification evidence; entries older than two weeks move to
  `docs/archive/`.
- `docs/OWNER_DASHBOARD.md`: one page, plain language, updated every
  session — current capability, gate status, top risk, next task.
- `docs/DEBUGGING.md`: commands, smoke profiles, protocol probes, failure
  modes.

## Blockers And Non-Goals

- Do not add fake persistent memory. Memory stays `current_boot` and
  read-only until M3+ persistence exists.
- Do not send raw `system.snapshot` or boot logs to a provider.
- Do not grant module/service/config mutation beyond the explicit milestone
  gates above.
- Do not add non-authorizing loader boundaries or new denial gates while
  M0–M2 are open.
- Do not treat the direct OpenAI provider path as the recovery lifeline.
- Do not overwrite `release/raios-stage0.img` unless the replacement has
  booted in QEMU.
- No work in `ota/`, `registry/`, `fake-cloud/` without a new ADR
  (ADR 0005 §4).
