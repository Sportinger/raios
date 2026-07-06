# M7 Persistence Foundation — Design Map (2026-07-06)

**Header.** Authored 2026-07-06 AHEAD of execution as pre-planning; no M7 code exists yet. Intended home: `docs/plan-reviews/m7-persistence-map-2026-07-06.md`. Execution preconditions: M6 Promotion Loop v0 CLOSED (all of M6A-D per `docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`; final full Shadow VM profile green). MANDATORY Slice 0 = map revalidation: re-check every file:line claim against HEAD, update this map first if reality diverged, commit the map update before any implementation slice — M6 will have moved files, assume drift. Downstream: M9 and the M7D-dependent parts of M8 block on M7.

## 1. What M7 is

After M7, a service promoted through the M6 evidence chain survives a reboot: its artifact, its promotion transaction, and boot-control state live on a real GPT `SEED_DATA` partition, and the next boot re-verifies the SAME evidence chain before the service may run again. Anything failing re-verification stays inert with typed evidence. This ends the "everything is current_boot" era — for exactly three scoped write targets and nothing else.

Sub-milestones (numbering FINAL): **M7A** GPT layout + read-only detection; **M7B** SEED_DATA append-only durable record store (generalizing the M3 discipline); **M7C** boot control (`raios.boot_control.v0`, A/B state, success marker, last-good fallback, SAFE mode); **M7D** persistent artifact store + boot-time re-promotion under the SAME gates.

## 2. Verified baseline (re-check all of this in Slice 0)

- Block IO: `seed-kernel/src/ahci.rs` — READ/WRITE_DMA_EXT consts at `:44-45`; `RAIOS_SCRATCH_V0` / `RAIOS_AUDITRB_V0` marker+LBA1-region contract at `:52-58`; MBR partition inventory evidence impls at `:162`, `:185`; audit/rollback target write/readback wrappers at `:771` and `:798`, inspect wrappers at `:828` and `:869`, low-level inspection helper at `:2130`.
- M3 denial chain to generalize: the write-boundary files listed in `docs/plan-reviews/m3-durable-write-map-2026-07-06.md` (current tree also has base/helper emit files in the same family).
- Harness disks: boot image `if=ide` at `scripts/run-stage0-qemu.ps1:57`, scratch `bus=ide.1,unit=0` at `:62-63`, audit target `bus=ide.2,unit=0` at `:69-70`. Profile ValidateSet at `vm-harness/shadow-vm-smoke.ps1:12`.
- Image build: `scripts/package-stage0.ps1:88` → `scripts/make-fat32-image.py` (superfloppy FAT32, no partition table). `release/raios-stage0.img` is the protected MVP artifact.
- Spec: `docs/image-layout-v0.md` (GPT, SEED_ESP_A/B, SEED_DATA, control.json state model, SAFE mode, atomic replace, fail-closed) — normative for semantics.
- `raios-core`: record model + single JSON serializer/hasher, host tests. Mechanism-before-vocabulary: ALL new `raios.*.v0` schemas in M7 are record-model entries, never hand-rolled emit/hash code.
- Actual M6 output at M7-0: RAM-only dev-tier load/run/un-promote through M6D-1 is present; the durable promotion/rollback transaction originally expected here is M6D-2 and is deferred.

**M7-0 REVALIDATION NOTE (2026-07-06).** Verified against HEAD `c61bf93` plus unrelated local edits in `seed-kernel/src/console.rs` and `seed-kernel/src/openai.rs`; neither affects this map. M6 did not deliver the durable promotion transaction assumed by the original section-2 bullet. It delivered the RAM-only dev-tier loop through M6D-1: runtime candidate intake/delivery in `module_candidate_intake.rs` and `module_candidate_channel.rs`, dev-key grant evidence in `agent_protocol_module_grant.rs`, and live load/run/rollback in `granted_candidate_service.rs`. M6D-2 (durable promotion transaction binding candidate/manifest/vm_report/attestation/grant/slot plus durable un-promote rollback transaction) is DEFERRED and is a prerequisite for M7D re-promotion. Recommended sequencing: M7A-1 is fully unblocked and is the next implementation slice; M7A + M7B build GPT plus the SEED_DATA RECLOG durable store first; M6D-2 should then record its durable promotion/rollback transaction into that SEED_DATA RECLOG store, not the single shared `RAIOS_AUDITRB_V0` sector; M7D then re-verifies the SEED_DATA-persisted transaction after reboot. Additional checked drift: `docs/image-layout-v0.md` recommends 128 MiB ESP slots while the M7A-1 packet below says 64 MiB; implementation should use 128 MiB or record an explicit test-disk deviation before coding.

## 3. Design decisions

### 3.1 Topology: dedicated GPT persist test disk; production image untouched

M7 does NOT convert `release/raios-stage0.img` to GPT. QEMU keeps booting the single-FAT stage0 image; M7 adds a fourth harness-provisioned raw disk (`raios-persist-gpt.img`) carrying the image-layout-v0 GPT: protective MBR, GPT header+entries+backup, `SEED_ESP_A`/`SEED_ESP_B` (FAT32, standard ESP type GUID) and `SEED_DATA` (raw raiOS layout, new type GUID `5EEDDA7A-C0DE-4A55-9A15-000000000001` — add to image-layout-v0.md). Disk is created fresh per run in the harness temp dir; `-PersistDiskPath` keeps/reuses one only for the two-boot test. Migrating the production boot image to GPT is OUT of M7 scope and is a STOP-tripwire.

### 3.2 SEED_DATA v0 = raw region map, no filesystem — OWNER DECISION

The kernel has no FAT32 write driver; correct FAT writing (FATs, directories, rename ordering) is a large corruption surface, and image-layout-v0.md leaves the DATA filesystem question open. Options: **(a) RECOMMENDED** raw region map — superblock at partition LBA0 (copy at LBA1): magic `RAIOS_DATA_SB_V0`, version, region table {tag,start_lba,lba_count}, sha256 over header. Regions: `BOOTCTL` LBA 2..9 (two 4-sector ping-pong slots), `RECLOG` LBA 16..4111 (2 MiB append-only log), `ARTSTOR` LBA 8192..end (artifact blobs). Superblock is written by the host image builder; the kernel validates it read-only and NEVER writes SEED_DATA LBA0/1. Exact numbers become normative in one Rust const module mirrored by the Python builder. This implements the image-layout-v0.md STATE MODEL exactly while replacing file-rename semantics with slot/append semantics; document as a spec addendum. **(b)** minimal FAT32 write driver first — weeks, worst crash semantics, delays M7D. **(c)** host-side writes only — fake persistence, violates AGENTS.md. Recommendation: (a).

### 3.3 M7B record store format (generalized M3 discipline)

RECLOG frame, sector-aligned: magic `RAIOSRC0` (8B) | u32 frame_len (multiple of 512) | u32 payload_len | u64 seq (from 1) | prev_frame_sha256 (32B; zeros at seq 1) | payload_sha256 (32B) | payload (record-model JSON) | zero pad. Chain hash = sha256 over the full frame. Boot recovery scan walks from region start; a frame is valid iff magic/lengths sane, payload hash matches, seq = prev+1, prev hash matches; the log ends at the first invalid frame; a torn tail is typed evidence and is overwritten by the next authorized append. Capacity — OWNER DECISION: **(a) RECOMMENDED** deny-when-full with explicit `durable_store_full` denial (simplest honest v0); **(b)** rotation/compaction — deferred until ADR 0004 supersede rules are designed. Every append follows the M3 transaction: build frame → verify region → write → readback → inspect → only then report appended. The RAM ring (256 events) STAYS; durable records are a NEW higher authority level per ADR 0004 (evidence/ledger track, provenance + `public|local_only|secret` classification; secrets never persisted). Gate flips are scoped to new target ids `append.record_log.seed_data` / `raios.durable_record.v0`; everything else stays `capability_denied`.

### 3.4 M7C boot control semantics

`raios.boot_control.v0` (fields exactly per image-layout-v0.md: active, last_good, pending, safe_mode, boot_attempt{...,success_marked}, slots{A,B}{generation,state,failure_count,...}) stored as record-model JSON in a BOOTCTL slot frame: magic `RAIOSBC0` | u32 payload_len | u64 seq | payload_sha256 | payload | pad to 2048B. Two slots; valid slot with highest seq is authoritative; a write goes to the LOSER slot, is read back and verified, and wins by seq — .prev fallback and atomic replace without rename (power loss corrupts only the slot being written). Both slots invalid ⇒ SAFE posture: boot continues from the loaded image, persistence disabled, nothing marked good, typed evidence (spec rule).

Boot-success criteria v0 (machine-checkable from what the kernel already knows): framebuffer OR serial active; heap allocator initialized; AHCI probe complete; SEED_DATA superblock + boot control valid; event log ready; agent protocol handler registered plus one successful internal self-dispatch of `system.snapshot`; no panic before the mark. Network/provider are NOT success criteria. Honest note: "agent protocol answering" an external client cannot be required (no peer may exist on bare metal); the internal self-dispatch is the v0 stand-in.

A/B switching is v0-manual: the kernel runs the full state machine (pending → probation → success_marked → last_good advance; failure_count; pending never consumed in SAFE) and writes markers; a host script `scripts/switch-boot-slot.ps1` stages slot payloads into SEED_ESP_A/B and sets pending. HONEST LIMITS (execution-time verification): Limine does not read control.json to pick a slot; OVMF's choice between two ESPs on one disk is not proven deterministic; seeding UEFI boot variables via the VARS pflash file is unproven here. If deterministic slot boot cannot be shown in QEMU, M7C still closes with the state machine + markers real, and firmware-level slot selection becomes an OWNER DECISION (tiny boot manager / boot variables / Limine config) — do not fake it.

### 3.5 M7D artifact store + boot-time re-promotion

**M7-0 REVALIDATION NOTE (2026-07-06).** The M7D text below assumes a durable M6 promotion/un-promote transaction. At HEAD, only the RAM-only dev-tier loop exists (`module_candidate_intake.rs`, `module_candidate_channel.rs`, `agent_protocol_module_grant.rs`, `granted_candidate_service.rs`); M6D-2 is deferred. Sequence M7A -> M7B -> M6D-2 into SEED_DATA RECLOG -> M7D re-promotion over that persisted transaction. Do not use `RAIOS_AUDITRB_V0` as the long-term promotion ledger for M7D.

Persist path (only after a successful M6 promotion, never in SAFE): blob to ARTSTOR (frame: magic `RAIOSAR0` | u32 frame_len | u32 payload_len | payload_sha256 | wasm bytes | pad), then a `raios.artifact_persist.v0` RECLOG record binding blob offset/len, artifact sha256, manifest hash, VM report hash, grant hash, promotion transaction hash, service id, import set. A blob without its chained RECLOG record is garbage, never authority. Allocation = bump allocator rebuilt from the RECLOG scan at boot.

Boot-time re-promotion (after boot control read, NOT in SAFE): scan RECLOG persist records; for each, recompute blob sha256 from disk, verify every referenced hash and the promotion transaction readback, then feed the candidate through the SAME M6 gate chain (grant, slot allocator, promotion authority) — no bypass, no "trusted because stored". Any mismatch ⇒ inert + `repromotion_denied` evidence. Honest note: no shadow-VM runs at boot; the authority is the durable promotion transaction that already bound the VM report hash — re-promotion re-verifies the recorded chain, it does not re-run tests. Revocation reuses the M6D un-promote transaction.

## 4. Verification, profiles, disk safety

New focused profiles (extend ValidateSet at `shadow-vm-smoke.ps1:12`; files follow the `shadow-vm-smoke-profile-*.ps1` pattern):

- `persistence` — single boot with the GPT persist disk: GPT/superblock detection, scoped durable append/readback/chain, boot-control read/write, success mark, SAFE posture on corrupted-control fixture, torn-tail detection on pre-corrupted RECLOG fixture.
  `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile persistence`
- `persistence-reboot` — NEW wrapper `vm-harness\shadow-vm-persistence-reboot.ps1` (reuses support functions; safer than teaching the single-boot runner to reboot): boot 1 promotes + persists a real external candidate, shuts down; boot 2 on the SAME kept persist disk asserts re-promotion and a live service answer; one merged two-phase `raios.vm_test_report.v0`.
  `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-persistence-reboot.ps1`

Needle families (golden needles are ground truth — M2 Batch 4 proved worker self-reports lie): `gpt-header-valid`, `gpt-crc-checked`, `gpt-seed-data-found`, `gpt-absent-fail-closed`, `data-superblock-valid`, `durable-append-authorized`, `durable-readback-hash`, `durable-chain-head`, `durable-store-full-denied`, `torn-tail-detected`, `boot-control-read`, `boot-control-write-pingpong`, `boot-success-marked`, `safe-posture-both-slots-invalid`, `pending-not-consumed-in-safe`, `artifact-persisted`, `repromotion-granted`, `repromotion-denied-hash-mismatch`, `service-answers-after-reboot`, plus needles proving generic write targets STAY denied.

Disk safety: the persist disk is always harness-created in the run temp dir or an explicit `-PersistDiskPath`; builder and scripts hard-refuse output paths under `release/`; `release/raios-stage0.img` is never attached as persist disk and never rebuilt by M7; kernel-side, GPT metadata sectors and SEED_DATA LBA0/1 are validated read-only and the boot disk gets no write path. QEMU wiring: `id=raiospersist0`, `bus=ide.3,unit=0` (execution-time verify the q35 port exists; ide.2 works today). Budget: quick after low-risk slices; focused `persistence` after EVERY M7 slice (storage/boot boundaries are always focused-evidence per AGENTS.md); full at M7-CLOSE.

## 5. Slice plan

### Slice M7-0 (MANDATORY): map revalidation

Capability: the orchestrator can trust every file:line in this map against HEAD. Verification: docs-only diff review, no VM run. Fail-closed: no code changes. STOP-tripwires: any M6 output shape differs from section 2's last bullet; the write-boundary chain was restructured beyond line drift.

```text
Packet id: M7-0-revalidate
Goal: Re-verify every file:line and "expected M6 output" claim in
  docs/plan-reviews/m7-persistence-map-2026-07-06.md against HEAD.
Read first: the map; docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md;
  seed-kernel/src/ahci.rs; scripts/run-stage0-qemu.ps1;
  vm-harness/shadow-vm-smoke.ps1; scripts/package-stage0.ps1;
  docs/image-layout-v0.md; docs/PROJECT_STATUS.md.
Allowed write set: docs/plan-reviews/m7-persistence-map-2026-07-06.md only.
Forbidden: any source, script, harness, or release file.
Constraints: fix stale line numbers/claims in place; if a DESIGN assumption
  broke (not just a line number), STOP and report instead of redesigning.
Definition of done: map matches HEAD; commit "M7-0: map revalidated".
Report format: corrected claims (old -> new); broken design assumptions
  (must be empty to proceed); commit hash.
```

### Slice M7A-1: GPT persist test disk + harness wiring

Capability: a harness run can attach a real GPT disk with SEED_ESP_A/B + SEED_DATA (superblock, empty regions) without touching production images. Files: NEW `scripts/make-gpt-persist-image.py` (may import Fat32Builder from `make-fat32-image.py` for ESP slots); `scripts/run-stage0-qemu.ps1` (fourth drive); `vm-harness/shadow-vm-smoke.ps1` + support (provision, `-PersistDiskPath`); NEW `shadow-vm-smoke-profile-persistence.ps1` (host-side needles only this slice). Verification: quick stays green + `-Profile persistence` host-side checks. Fail-closed: builder refuses `release/` paths; no kernel changes. STOP-tripwires: QEMU cannot expose a fourth IDE/AHCI disk; any urge to rebuild `release/raios-stage0.img`.

```text
Packet id: M7A-1-gpt-test-disk
Goal: Build a GPT persist test disk (protective MBR, GPT header+entries+backup,
  SEED_ESP_A/SEED_ESP_B FAT32 64 MiB each, SEED_DATA raw per map 3.2 superblock
  layout, type GUID 5EEDDA7A-C0DE-4A55-9A15-000000000001) and wire it into
  QEMU + harness as drive id raiospersist0, bus=ide.3,unit=0.
Read first: map sections 3.1-3.3 and 4; docs/image-layout-v0.md;
  scripts/make-fat32-image.py; scripts/run-stage0-qemu.ps1;
  vm-harness/shadow-vm-smoke.ps1 and shadow-vm-smoke-support.ps1.
Allowed write set: scripts/make-gpt-persist-image.py (new),
  scripts/run-stage0-qemu.ps1, vm-harness/shadow-vm-smoke.ps1,
  vm-harness/shadow-vm-smoke-support.ps1,
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 (new).
Forbidden: release/**, seed-kernel/**, raios-core/**, existing needle
  expectations in other profiles.
Constraints: GPT header and partition-entry-array CRC32s must be correct
  (python self-check in the builder); superblock sha256 per map 3.2; disk
  created in the harness run dir by default; hard-fail on release/ output
  paths. If bus=ide.3 is rejected, report the QEMU error and device list
  instead of improvising another bus.
Definition of done: persistence profile passes host-side needles; quick green;
  commit with capability sentence + report filename.
Report format: partition table + superblock hex head, report JSON path,
  needle count added.
```

### Slice M7A-2: kernel GPT + SEED_DATA read-only detection

Capability: the kernel finds and validates SEED_DATA on a real GPT disk and reports typed layout evidence (present/absent/invalid) — zero writes. Files: NEW `seed-kernel/src/gpt.rs` + `seed-kernel/src/seed_data_layout.rs` (GPT header/entries with CRC32, ESP + SEED_DATA name/GUID match, superblock validation), probe hookup in `ahci.rs`; raios-core record entries `raios.gpt_layout.v0` / `raios.data_layout.v0` + host fixture tests; a read-only agent-protocol query. Verify-at-execution: how the AHCI probe enumerates the fourth port. Verification: focused `persistence` (gpt-header-valid, gpt-crc-checked, gpt-seed-data-found, data-superblock-valid, gpt-absent-fail-closed via a no-disk run) + `cargo test --locked -p raios-core`. Fail-closed: corrupt GPT/superblock ⇒ invalid-layout evidence, kernel continues without persistence; no write authority anywhere. STOP-tripwires: detection needs attested descriptor-source edits; AHCI multi-port needs driver rework (report scope, do not hack).

```text
Packet id: M7A-2-gpt-detect
Goal: Read-only GPT parsing + SEED_DATA superblock validation with typed
  evidence records per map 3.2; no write path.
Read first: map sections 2, 3.1-3.2, 4; seed-kernel/src/ahci.rs (probe,
  partition inventory evidence, sector reads); raios-core/src record model;
  vm-harness/shadow-vm-smoke-profile-persistence.ps1.
Allowed write set: seed-kernel/src/gpt.rs (new),
  seed-kernel/src/seed_data_layout.rs (new), seed-kernel/src/ahci.rs (probe
  hookup only), raios-core/src/** (+tests), one agent-protocol read-only query
  file, vm-harness/shadow-vm-smoke-profile-persistence.ps1 needles.
Forbidden: WRITE_DMA_EXT call sites; write-boundary gate files; attested
  hello/echo/descriptor sources (STOP if unavoidable); release/**.
Constraints: parsing/validation logic host-testable in raios-core with byte
  fixtures (CRC good/bad, truncated table, duplicate SEED_DATA -> invalid,
  superblock hash mismatch); schemas as record-model entries only; all
  evidence labeled current_boot.
Definition of done: raios-core tests green; persistence profile green incl.
  the no-disk fail-closed run; quick green.
Report format: needle names added; report JSON paths (with/without disk);
  host-test count.
```

### Slice M7B-1: RECLOG read/scan (recovery scan, still read-only)

Capability: the kernel scans RECLOG, validates the full hash chain, reports head/tail/count and torn-tail evidence — before any kernel write exists. Files: raios-core NEW `durable_record_frame.rs` (frame codec + chain scan; the framing brain is host-tested, the kernel only does sector IO); NEW `seed-kernel/src/durable_store.rs` (scan over AHCI reads); harness fixture seeder (valid records + one torn tail). Verification: focused `persistence` (durable-chain-head, torn-tail-detected, empty-log-valid) + raios-core tests. Fail-closed: appends still `capability_denied` end to end. STOP-tripwires: globals only.

```text
Packet id: M7B-1-reclog-scan
Goal: RECLOG frame codec + chain-validating recovery scan with typed evidence;
  read-only. Frame format exactly per map 3.3.
Read first: map 3.3; raios-core/src (sha256, record model);
  docs/plan-reviews/m3-durable-write-map-2026-07-06.md;
  seed-kernel/src/seed_data_layout.rs.
Allowed write set: raios-core/src/durable_record_frame.rs (new) + tests,
  seed-kernel/src/durable_store.rs (new), minimal ahci.rs read hookup,
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 + support (fixture
  seeder), scripts/make-gpt-persist-image.py (--seed-reclog-fixture flag).
Forbidden: any write-authority flip; write-boundary gate files; attested
  sources; release/**.
Constraints: host tests must cover empty log, N valid records, bad magic, bad
  payload hash, bad prev hash, bad seq, torn tail, multi-sector record. Scan
  stops at first invalid frame and reports it as evidence, never authority.
Definition of done: raios-core tests green; persistence profile green with
  chain + torn-tail needles; quick green; capability sentence in commit.
Report format: fixture description, needle names, report JSON path.
```

### Slice M7B-2: scoped durable append authority

Capability: the kernel durably appends a typed record to RECLOG with append → readback → inspect → only-then-report under a scoped grant — the first real persistence write in raiOS. Files: the M3 write-boundary chain files (scoped positive path for `append.record_log.seed_data` ONLY, mirroring how M3 scoped `append.audit_ledger.current_boot` — flip list in the M3 map); `durable_store.rs` append over the existing AHCI write generalized to multi-sector spans inside the validated RECLOG region (verify-at-execution how hardcoded the current LBA1 path is); raios-core append planner + tests. First payloads: boot lifecycle records mirroring RAM-ring events (ring stays authoritative for current_boot UI). Verification: focused `persistence` (durable-append-authorized, durable-readback-hash, durable-store-full-denied via nearly-full fixture, generic-target-still-denied needles); `module-audit-rollback` profile unchanged-green. Fail-closed: writes outside the RECLOG span denied; GPT metadata, superblock, BOOTCTL, ARTSTOR unwritable this slice; generic module write boundary, scratch, boot media all still denied; store-full ⇒ deny, no rotation. STOP-tripwires: any widening beyond the RECLOG span; AHCI restructure over ~500 changed lines (report first).

```text
Packet id: M7B-2-durable-append
Goal: Scoped append authority for target append.record_log.seed_data with the
  M3 transaction discipline (build frame -> verify region -> write -> readback
  -> inspect -> report), per map 3.3. No other target gains authority.
Read first: map 3.3; docs/plan-reviews/m3-durable-write-map-2026-07-06.md
  (Minimal Grant Design + flip list); the eight
  seed-kernel/src/agent_protocol_module_write_boundary_*.rs files;
  seed-kernel/src/durable_store.rs; ahci.rs write/readback path.
Allowed write set: seed-kernel/src/durable_store.rs, write-boundary files
  (scoped additions only), seed-kernel/src/ahci.rs (span write within
  validated region), raios-core/src/** (+tests),
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 + support.
Forbidden: flipping shared writes_enabled / generic authorizes_append;
  changing RAIOS_AUDITRB_V0 semantics; BOOTCTL/ARTSTOR writes; attested
  sources; release/**.
Constraints: every gate flip names the exact seed_data target id; add needles
  proving the OLD generic denials still hold; if the AHCI write path cannot
  take a span parameter without restructuring, report the restructure size
  before doing it.
Definition of done: persistence profile green incl. still-denied needles;
  quick green; module-audit-rollback profile unchanged-green.
Report format: flipped gate fields with file:line, needle diff summary,
  report JSON paths.
```

### Slice M7C-1: boot-control read + state model + SAFE posture

Capability: the kernel reads `raios.boot_control.v0` from BOOTCTL (ping-pong, highest-valid-seq), evaluates the pending/last_good/failure state machine, and enters an honest SAFE posture when control is invalid or safe_mode is set. Files: raios-core NEW `boot_control.rs` (slot codec + pure state machine; host tests: fresh disk, valid A, A+B pick-higher-seq, both-invalid, pending-without-success, failure-count threshold); NEW `seed-kernel/src/boot_control.rs` (read + posture flag consumed by M7B gates and later M7D); builder seeds an initial control record. Verification: focused `persistence` (boot-control-read, safe-posture-both-slots-invalid via corrupted fixture, pending-not-consumed-in-safe) + host tests. Fail-closed: SAFE disables all durable writes (except later SAFE-audit); invalid control never selects a candidate; nothing marked good. STOP-tripwires: any pressure to auto-recover by rewriting control instead of entering SAFE posture.

```text
Packet id: M7C-1-boot-control-read
Goal: Boot-control slot codec + state machine + SAFE posture, read-only
  against BOOTCTL; raios.boot_control.v0 fields exactly per
  docs/image-layout-v0.md with the slot/seq storage from map 3.4.
Read first: map 3.4; docs/image-layout-v0.md (Boot Control State, SAFE Mode,
  Boot Flow); raios-core record model; seed-kernel/src/durable_store.rs.
Allowed write set: raios-core/src/boot_control.rs (new) + tests,
  seed-kernel/src/boot_control.rs (new), scripts/make-gpt-persist-image.py
  (initial control record), persistence profile + support fixtures/needles.
Forbidden: writing BOOTCTL from the kernel (next slice); slot switching;
  attested sources; release/**.
Constraints: state machine is pure host-tested raios-core logic; the kernel
  only feeds sector bytes and consumes posture flags; schema as record-model
  entry only.
Definition of done: host tests cover the six listed scenarios; persistence
  profile green with SAFE-posture needles; quick green.
Report format: state-machine test matrix, needle names, report JSON path.
```

### Slice M7C-2: boot-success marker write + v0-manual slot switch script

Capability: a booted kernel that meets the section-3.4 success criteria durably marks boot success (ping-pong control write + RECLOG audit record) and advances last_good per spec; an owner-invoked script stages/selects slots offline. Files: `seed-kernel/src/boot_control.rs` (success evaluation + verified write through a SECOND scoped target `replace.boot_control.seed_data`, same discipline); NEW `scripts/switch-boot-slot.ps1` (validate GPT image, copy slot payload into SEED_ESP_A/B, set pending using the same codec — via a small raios-core host binary or a python codec port; pick whichever is cheaper at execution time and say so); probation fixtures. Verification: focused `persistence` (boot-success-marked, boot-control-write-pingpong, last-good-advance, failure-count fixture keeps last_good). Plus one non-gating EXPERIMENT: boot the GPT disk directly under OVMF to observe ESP selection; record findings in a map addendum — deterministic slot boot is NOT claimed in M7. Fail-closed: success never marked in SAFE or with any criterion unmet; pending never consumed in SAFE; script refuses `release/` images and non-GPT images. STOP-tripwires: anything requiring bootloader replacement, UEFI variable writes, or real-hardware boot changes ⇒ owner decision (likely a new ADR).

```text
Packet id: M7C-2-boot-success-write
Goal: Kernel writes boot success into BOOTCTL via scoped
  replace.boot_control.seed_data (write loser slot -> readback -> verify ->
  seq wins), appends a RECLOG audit record, advances last_good per
  docs/image-layout-v0.md rules; plus scripts/switch-boot-slot.ps1 for
  owner-invoked v0-manual slot staging/pending.
Read first: map 3.4; docs/image-layout-v0.md (Rules, Boot Flow, Atomic
  Writes); raios-core/src/boot_control.rs; the M7B-2 write-boundary flip
  pattern; scripts/make-gpt-persist-image.py.
Allowed write set: seed-kernel/src/boot_control.rs, write-boundary files
  (second scoped target only), raios-core/src/** (+tests),
  scripts/switch-boot-slot.ps1 (new), persistence profile + support.
Forbidden: generic write authority; UEFI boot-variable manipulation; touching
  release/raios-stage0.img or writing GPT boot images into release/; attested
  sources.
Constraints: success criteria are exactly the map-3.4 list, evaluated once,
  evidence-logged; the direct-GPT-boot OVMF experiment is observation-only,
  never a gating needle; the script prints a dry-run plan and requires an
  explicit -Apply switch.
Definition of done: persistence profile green with success/pingpong/last-good
  needles; host tests green; experiment findings appended to the map;
  quick green.
Report format: flipped gate list, success-criteria evidence sample, OVMF ESP
  selection observations, report JSON paths.
```

### Slice M7D-1: persistent artifact store

Capability: a successfully M6-promoted candidate can be durably persisted — ARTSTOR blob + chained `raios.artifact_persist.v0` record binding the full evidence chain — and enumerated after rescan, still inert without re-verification. Files: NEW `seed-kernel/src/artifact_store.rs` (blob write via THIRD scoped target `blob.artifact_store.seed_data`; bump allocation rebuilt from RECLOG); raios-core blob codec + persist-record entry + tests; the M6 promotion path gains a post-promotion persist step (verify-at-execution where M6C landed; if attestation-covered, run the descriptor re-sign flow via `target/descriptor-resign` — never hand-edit hashes). Verification: focused `persistence` (artifact-persisted, blob-hash-verified, blob-without-record-is-garbage fixture, persist-denied-in-safe) + host tests. Fail-closed: persist denied unless the promotion transaction verified this boot; denied in SAFE; stored blobs gain no load authority. STOP-tripwires: persist step requires weakening any M6 gate; attested-source edits balloon beyond the promotion completion hook.

```text
Packet id: M7D-1-artifact-persist
Goal: Persist a promoted candidate: ARTSTOR blob (map 3.5 frame) + chained
  RECLOG artifact_persist record binding artifact/manifest/VM-report/grant/
  promotion-transaction hashes; enumeration via RECLOG scan; no load authority.
Read first: map 3.5; the M6C promotion transaction implementation (locate via
  docs/PROJECT_STATUS.md at execution time); seed-kernel/src/durable_store.rs;
  raios-core/src/durable_record_frame.rs.
Allowed write set: seed-kernel/src/artifact_store.rs (new), write-boundary
  files (third scoped target only), the M6 promotion completion path file,
  raios-core/src/** (+tests), persistence profile + support.
Forbidden: any instantiate/load from stored blobs; generic write authority;
  release/**. If the promotion path file is attestation-covered, complete the
  descriptor re-sign flow (target/descriptor-resign) and prove build.rs
  verification passes; do NOT hand-edit hashes.
Constraints: allocation state derived only from the RECLOG scan (host test:
  rebuild after simulated reboot); a blob with a missing/unchained RECLOG
  record is reported as garbage evidence.
Definition of done: persistence profile green incl. safe-denied and
  garbage-blob needles; host tests green; quick green.
Report format: persist record sample (hash prefixes only), flipped gate list,
  re-sign evidence if applicable, report JSON paths.
```

### Slice M7D-2: boot-time re-promotion + two-boot proof

Capability: THE PRODUCT MOMENT — a promoted service survives reboot: on boot 2 the stored artifact re-verifies through the SAME evidence chain and answers live; anything failing re-verification stays inert with typed evidence. Files: NEW `seed-kernel/src/repromotion.rs` (post-boot-control, non-SAFE: scan → re-verify per 3.5 → feed the normal M6 gate chain → instantiate); `raios.repromotion.v0` records + RECLOG audit of each grant/denial; NEW `vm-harness/shadow-vm-persistence-reboot.ps1` two-boot wrapper + profile; corrupted-blob and tampered-record fixtures. Verification: NEW `persistence-reboot` profile (service-answers-after-reboot, repromotion-granted, repromotion-denied-hash-mismatch, repromotion-skipped-in-safe) + focused `persistence` regression. Fail-closed: SAFE ⇒ zero re-promotion; any hash/chain/transaction mismatch ⇒ inert + evidence; re-promotion uses no bypass into wasm_runtime — same slot allocator, grant checks, and inventory path as M6. STOP-tripwires: re-promotion cannot reuse the M6 chain without modifying it — report, never fork a parallel trust path.

```text
Packet id: M7D-2-repromotion-reboot
Goal: Boot-time re-promotion per map 3.5 through the unmodified M6 gate chain,
  plus a two-boot harness wrapper proving promote -> persist -> reboot ->
  re-verify -> service answers, and proving corrupted artifacts stay inert.
Read first: map 3.5 and 4; seed-kernel/src/artifact_store.rs; the M6 gate
  chain files (locate via docs/PROJECT_STATUS.md); vm-harness/shadow-vm-smoke.ps1
  + shadow-vm-smoke-support.ps1 (reuse, do not fork logic).
Allowed write set: seed-kernel/src/repromotion.rs (new), minimal boot-sequence
  hookup, raios-core/src/** (+tests),
  vm-harness/shadow-vm-persistence-reboot.ps1 (new), persistence +
  persistence-reboot profile files, shadow-vm-smoke-support.ps1 (shared
  helpers only), scripts/run-stage0-qemu.ps1 only if a keep-disk flag is
  missing.
Forbidden: new load entry points bypassing M6 gates; auto-load in SAFE mode;
  weakening or deleting any denial needle; release/**; attested sources
  (STOP if needed).
Constraints: boot 2 runs from the SAME persist disk file; merged report stays
  schema raios.vm_test_report.v0 with per-phase command/predicate attribution;
  denial fixtures corrupt (a) blob bytes and (b) the persist record hash
  binding — both must yield repromotion_denied.
Definition of done: persistence-reboot profile green incl. both denial
  fixtures; persistence + quick green; commit capability sentence: "a promoted
  service now survives reboot under re-verified evidence".
Report format: two-phase report JSON path, boot-2 service answer transcript
  excerpt, denial-fixture evidence lines.
```

### Slice M7-CLOSE: full-profile closure + docs

Capability: M7 is claimable — full Shadow VM profile green with persistence needles folded in; docs and owner dashboard reflect real persistence. Files: fold stable needles into the full profile (new `shadow-vm-smoke-profile-full-persistence.ps1` if cleaner); update `docs/image-layout-v0.md` (addendum: raw region map realization, superblock, type GUID, OVMF findings, honest slot-boot status), `docs/ROADMAP.md` cursor, `docs/PROJECT_STATUS.md`, `docs/OWNER_DASHBOARD.md` (plain language: "services the system was granted now survive switching it off"). Verification: `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile full` green + `scripts\scan-secrets.ps1` clean + both persistence profiles green in the same session. Fail-closed: closure blocked while ANY profile is red (Red Gate Rule). STOP-tripwires: full profile red for non-persistence reasons (Red Gate — stop feature work); any doc claim the needles do not back.

```text
Packet id: M7-CLOSE
Goal: Fold persistence needles into the full profile, write the
  image-layout-v0.md addendum, update ROADMAP/PROJECT_STATUS/OWNER_DASHBOARD,
  run full profile + secret scan, close M7.
Read first: map section 4; all persistence profile files; docs conventions in
  docs/ROADMAP.md and docs/OWNER_DASHBOARD.md.
Allowed write set: vm-harness/shadow-vm-smoke-profile-full-persistence.ps1
  (new) or additions to existing full profile files,
  vm-harness/shadow-vm-smoke.ps1 (full-profile dispatch only),
  docs/image-layout-v0.md, docs/ROADMAP.md, docs/PROJECT_STATUS.md,
  docs/OWNER_DASHBOARD.md.
Forbidden: kernel/source changes (if full goes red, this packet pauses and
  Red Gate repair rules apply); release image rebuilds.
Constraints: owner dashboard entry in plain non-programmer language leading
  with the capability sentence.
Definition of done: full-profile report result "passed" and newer than last
  commit; secret scan clean; commit names the passing report file.
Report format: full report JSON path + sha256, docs updated, milestone
  capability sentence.
```

## 6. Global STOP-tripwires (every slice — halt and ask the owner)

- Anything requiring a NEW ADR: unparking ota/registry/fake-cloud, any external artifact download path, trust-model or attestation-key changes, bootloader replacement, UEFI variable manipulation, real-hardware boot changes.
- Any write path that could touch: the boot disk, GPT metadata sectors, SEED_DATA LBA0/1, `RAIOS_AUDITRB_V0` LBA0, anything under `release/` (especially overwriting `release/raios-stage0.img`), or any destructive operation on a disk not created by the harness in that run.
- Persisting any secret (provider keys, Wi-Fi credentials, tokens) — the image-layout-v0.md denial list is binding until a sealed-secret ADR exists.
- Converting the production boot image to GPT (explicitly deferred past M7).
- Any generic (non-scoped) durable-write authority flip; any worker proposing to weaken or delete existing denial needles.

## 7. Estimate and verdict

Nine implementation slices plus revalidation and closure. Risky boundaries: M7B-2 (first real persistence write — keep it as scoped as M3 was), M7C-2 (bootloader honesty — do not overclaim slot boot), M7D-2 (the reboot proof; also the payoff). Three OWNER DECISIONS are marked inline with recommendations (3.2 storage mechanism → raw region map; 3.3 capacity → deny-when-full; 3.4 firmware slot selection fallback), so a cheap orchestrator only escalates, never designs. M7 ends the current_boot-only era for exactly three scoped targets — `append.record_log.seed_data`, `replace.boot_control.seed_data`, `blob.artifact_store.seed_data` — and every other write stays `capability_denied`.
