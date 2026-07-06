# M7 Persistence Foundation — Design Map (2026-07-06)

**Header / execution preconditions**

- Authored 2026-07-06 AHEAD of execution as pre-planning. No M7 code exists yet.
- Intended home: `docs/plan-reviews/m7-persistence-map-2026-07-06.md`.
- Execution preconditions: M6 Promotion Loop v0 CLOSED (all of M6A-D per
  `docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md`, final full Shadow VM
  profile green). M7 must not start while M6 is open or the full profile is red.
- MANDATORY Slice 0 = map revalidation: re-check every file:line claim in this map
  against HEAD, update the map first if reality diverged, commit the map update
  before any implementation slice. M6 will have moved files; assume drift.
- Downstream: M9 (durable memory) and the M7D-dependent parts of M8 (recovery
  restart-last-good) block on M7. Wi-Fi stays deliberately late (M12+).

## 1. What M7 is

After M7, a service promoted through the M6 evidence chain survives a reboot: its
artifact, its promotion transaction, and the boot-control state live on a real
GPT `SEED_DATA` partition, and on the next boot the kernel re-verifies the SAME
evidence chain before the service is allowed to run again. This is the moment M6
promotions stop being current_boot-only and the "no fake persistence" rule is
retired by building the real thing. Everything that fails re-verification stays
inert with typed evidence.

Sub-milestones (numbering FINAL):

- **M7A** GPT layout + read-only partition/layout detection
- **M7B** SEED_DATA append-only durable record store (generalizes M3 discipline)
- **M7C** boot control (`raios.boot_control.v0`, A/B state, success marker,
  last-good fallback, SAFE mode)
- **M7D** persistent artifact store + boot-time re-promotion under the SAME gates

## 2. Verified baseline this map builds on (re-check in Slice 0)

- Kernel block IO: `seed-kernel/src/ahci.rs` — READ_DMA_EXT/WRITE_DMA_EXT consts
  at `ahci.rs:44-45`; markers `RAIOS_SCRATCH_V0` / `RAIOS_AUDITRB_V0` and the
  LBA0-marker / LBA1-region contract at `ahci.rs:52-58`; MBR partition inventory
  evidence (`AhciPartitionEntryEvidence`, `AhciPartitionInventoryEvidence`) at
  `ahci.rs:162`, `:185`; target write/readback/inspection paths near `:771`,
  `:810`, `:2221` (M3 map values — re-verify, file has grown).
- M3 denial chain to generalize: the `agent_protocol_module_write_boundary_*`
  files (availability, write_policy, storage_layout, append_engine,
  append_contract, append_payload_hash, append_intent, boundary) per the table in
  `docs/plan-reviews/m3-durable-write-map-2026-07-06.md`.
- Harness disk wiring: `scripts/run-stage0-qemu.ps1:55-70` — boot image on
  `if=ide`, scratch on `bus=ide.1,unit=0`, audit target on `bus=ide.2,unit=0`.
- Profile dispatch: `vm-harness/shadow-vm-smoke.ps1:12` ValidateSet
  (full/quick/recovery/hello-rollback-dry-run/module-audit-rollback/
  provider-memory/provider-memory-full).
- Image build: `scripts/package-stage0.ps1:88` calls
  `scripts/make-fat32-image.py` (superfloppy FAT32, no partition table, fixed
  67108864 bytes). `release/raios-stage0.img` is the protected MVP artifact.
- Normative draft spec: `docs/image-layout-v0.md` (GPT, SEED_ESP_A/B, SEED_DATA,
  control.json state model, SAFE mode, atomic replace, fail-closed).
- Record model: `raios-core` (Value/Field, single JSON serializer+hasher, host
  tests). Mechanism-before-vocabulary: ALL new `raios.*.v0` schemas in M7 are
  record-model entries, never hand-rolled emit/hash code.
- M6 output (expected at M7 start, verify in Slice 0): promotion transaction
  appended to `RAIOS_AUDITRB_V0` binding candidate hash, manifest hash, VM report
  hash, attestation, grant, slot; rollback/un-promote transaction.

## 3. Design decisions

### 3.1 Test topology: dedicated GPT persist disk, production image untouched

M7 does NOT convert `release/raios-stage0.img` to GPT. QEMU keeps booting the
single-FAT stage0 image; M7 adds a fourth, harness-provisioned raw disk
(`raios-persist-gpt.img`) carrying the full image-layout-v0 GPT: protective MBR,
GPT header+entries+backup, partitions `SEED_ESP_A` (FAT32, standard ESP type
GUID), `SEED_ESP_B` (FAT32), `SEED_DATA` (raw raiOS layout, new type GUID
`5EEDDA7A-C0DE-4A55-9A15-000000000001` — add to image-layout-v0.md). The disk is
created fresh per harness run in the run temp dir; a `-PersistDiskPath` parameter
keeps/reuses one only for the two-boot re-promotion test. Migrating the
production boot image to GPT is OUT of M7 scope and is a STOP-tripwire (owner
sign-off; only after M7C is QEMU-proven on the test disk).

### 3.2 SEED_DATA v0: raw region map, no filesystem (OWNER DECISION, recommended)

The kernel has no FAT32 write driver; writing FAT correctly (FATs, directories,
rename ordering) is a large new corruption surface. image-layout-v0.md lists
file paths but leaves the DATA filesystem as an open question. Decision needed:

- (a) RECOMMENDED: raw region map on SEED_DATA. Superblock at partition LBA0
  (copy at LBA1), magic `RAIOS_DATA_SB_V0`, version, region table
  {tag,start_lba,lba_count} x3, sha256 over the header. Regions: `BOOTCTL`
  (LBA 2..9: two 4-sector ping-pong slots), `RECLOG` (LBA 16..4111, 2 MiB
  append-only record log), `ARTSTOR` (LBA 8192..end, artifact blobs).
  Superblock is written by the host image builder; the kernel validates it
  read-only and NEVER writes SEED_DATA LBA0/1. Exact numbers become normative
  in one Rust const module mirrored by the Python builder.
  This implements the image-layout-v0.md STATE MODEL exactly while replacing
  file rename semantics with slot/append semantics; document as an addendum.
- (b) Minimal FAT32 write driver first: weeks of work, worst crash semantics of
  any option, delays M7D. Not recommended.
- (c) Host-side writes only: fake persistence, violates AGENTS.md. Rejected.

### 3.3 M7B record store format (generalized M3 discipline)

Fixed length-framed records, sector-aligned, sha256-chained, in `RECLOG`:
frame = magic `RAIOSRC0` (8B) | u32 frame_len (multiple of 512) | u32
payload_len | u64 seq (starts at 1) | prev_frame_sha256 (32B, zeros for seq 1) |
payload_sha256 (32B) | payload (record-model JSON) | zero pad. Chain hash =
sha256 over the full frame. Boot recovery scan: walk from region start, a record
is valid iff magic/lengths sane, payload hash matches, seq = prev+1, prev hash
matches; the log ends at the first invalid frame; a torn tail is reported as
typed evidence and overwritten by the next authorized append. Capacity posture
(OWNER DECISION): (a) RECOMMENDED deny-when-full with explicit
`durable_store_full` denial — simplest honest v0; (b) rotation/compaction —
deferred, needs supersede rules from ADR 0004 first. Every append follows the M3
transaction: build frame → verify target region → authorized write → readback →
inspect → only then report appended. The RAM event ring (256 entries) STAYS;
durable records are a NEW, higher authority level per ADR 0004 (evidence/ledger
track with provenance + `public|local_only|secret` classification fields);
secrets are never persisted. Gate flips are scoped to new target ids
`append.record_log.seed_data` / `raios.durable_record.v0`; the generic module
write boundary and all other targets stay `capability_denied`.

### 3.4 M7C boot control semantics

`raios.boot_control.v0` record (fields exactly per image-layout-v0.md: active,
last_good, pending, safe_mode, boot_attempt{slot,generation,attempt_id,
started,success_marked}, slots{A,B}{generation,state,failure_count,...}) stored
as record-model JSON in a BOOTCTL slot frame: magic `RAIOSBC0` | u32 payload_len
| u64 seq | payload_sha256 | payload | zero pad to 2048B. Two slots; the valid
slot with highest seq is authoritative; a write goes to the LOSER slot, is read
back and verified, and wins by seq. This gives .prev fallback and atomic replace
without rename: power loss mid-write can only corrupt the slot being written.
Both slots invalid ⇒ SAFE posture: boot continues from the loaded image,
persistence disabled, nothing marked good, typed evidence emitted (spec rule).

Boot-success criteria v0 (machine-checkable from what the kernel already knows):
framebuffer OR serial active, heap allocator initialized, AHCI probe complete,
SEED_DATA superblock + boot control valid, event log ready, agent protocol
handler registered plus one successful internal self-dispatch of
`system.snapshot`, and no panic before the mark is written. Network/provider
are explicitly NOT success criteria. "Agent protocol answering an external
client" cannot be required (no peer may exist on bare metal) — the internal
self-dispatch is the honest v0 stand-in; note this in the addendum.

A/B switching is v0-manual: the kernel maintains the full state machine
(pending → probation → success_marked → last_good advance; failure_count;
pending-not-consumed-in-SAFE) and writes markers; a host script
`scripts/switch-boot-slot.ps1` reads control state from the persist disk image,
stages a slot payload into SEED_ESP_A/B, and sets pending. HONEST LIMITS
(execution-time verification items): Limine does not read control.json to pick
a slot; OVMF's choice between two ESPs on one disk is not proven deterministic;
seeding UEFI boot variables via the VARS pflash file is unproven here. If
deterministic slot boot cannot be shown in QEMU, M7C still closes with the state
machine + markers real and firmware-level slot selection moves to an owner
decision (tiny boot manager vs boot variables vs Limine config) — do not fake it.

### 3.5 M7D artifact store + boot-time re-promotion

Persist path (runs only after a successful M6 promotion): artifact blob written
to `ARTSTOR` (frame: magic `RAIOSAR0` | u32 frame_len | u32 payload_len |
payload_sha256 | wasm bytes | pad), then a `raios.artifact_persist.v0` record
appended to RECLOG binding blob offset/len, artifact sha256, manifest hash, VM
report hash, grant hash, promotion transaction hash, service id, import set.
A blob without its chained RECLOG record is garbage, never authority. Allocation
is a bump allocator whose state is rebuilt by scanning RECLOG at boot.

Boot-time re-promotion: after boot control is read and NOT in SAFE mode, scan
RECLOG for persist records, for each: recompute blob sha256 from disk, verify
every referenced hash and the promotion transaction readback, then feed the
candidate through the SAME M6 gate chain (grant, slot allocator, promotion
authority) — no bypass path, no "trusted because stored" shortcut. Any mismatch
⇒ artifact stays inert with `repromotion_denied` + reason evidence. Nothing
auto-loads without re-verified evidence. Honest note: no shadow-VM runs at
boot; the authority is the durable promotion transaction that already bound the
VM report hash — re-promotion re-verifies the recorded chain, it does not re-run
tests. A later revocation/undo path reuses the M6D un-promote transaction.

## 4. Verification strategy, profiles, disk safety

New focused profiles (extend ValidateSet at `shadow-vm-smoke.ps1:12`; profile
files follow the existing `shadow-vm-smoke-profile-*.ps1` pattern):

- `persistence` — single boot with the GPT persist disk: GPT/superblock
  detection, scoped durable append/readback/chain, boot-control read/write,
  boot-success mark, SAFE posture on corrupted-control fixture, torn-tail
  detection on pre-corrupted RECLOG fixture. Run:
  `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile persistence`
- `persistence-reboot` — NEW wrapper `vm-harness\shadow-vm-persistence-reboot.ps1`
  (reuses support functions; safer than teaching the single-boot runner to
  reboot): boot 1 promotes + persists a real external candidate (M6 flow),
  shuts down; boot 2 with the SAME kept persist disk asserts re-promotion and
  a live service answer; one merged `raios.vm_test_report.v0` with two phases.
  Run: `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-persistence-reboot.ps1`

Needle families (golden needles are ground truth — M2 Batch 4 proved worker
self-reports lie): `gpt-header-valid`, `gpt-crc-checked`, `gpt-seed-data-found`,
`gpt-absent-fail-closed`, `data-superblock-valid`, `durable-append-authorized`,
`durable-readback-hash`, `durable-chain-head`, `durable-store-full-denied`,
`torn-tail-detected`, `boot-control-read`, `boot-control-write-pingpong`,
`boot-success-marked`, `safe-posture-both-slots-invalid`,
`pending-not-consumed-in-safe`, `artifact-persisted`, `repromotion-granted`,
`repromotion-denied-hash-mismatch`, `service-answers-after-reboot`, plus
fail-closed needles proving generic write targets STAY denied.

Disk safety: the persist disk is always harness-created in the run temp dir or
an explicitly passed `-PersistDiskPath`; the scripts refuse to write GPT images
into `release/`; `release/raios-stage0.img` is never attached as the persist
disk and never rebuilt by M7 slices; kernel-side, SEED_DATA LBA0/1 and all GPT
metadata sectors are validated-read-only, and the boot disk gets no write path
at all. QEMU wiring adds `id=raiospersist0` on `bus=ide.3,unit=0`
(execution-time verify: the q35/AHCI port exists; ide.2 works today).

Budget: quick after low-risk slices; `persistence` focused after every M7 slice
(storage/rollback/boot boundaries are ALWAYS focused-evidence per AGENTS.md);
full profile at M7-CLOSE and before any release-image handoff.

## 5. Slice plan

### Slice M7-0 (MANDATORY): map revalidation

Capability: the orchestrator can trust every file:line in this map against HEAD.
Verification: docs-only — diff review; no VM run. Fail-closed: no code changes.

```text
Packet id: M7-0-revalidate
Goal: Re-verify every file:line and every "expected M6 output" claim in
  docs/plan-reviews/m7-persistence-map-2026-07-06.md against current HEAD.
Read first: the map itself; docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md;
  seed-kernel/src/ahci.rs; scripts/run-stage0-qemu.ps1; vm-harness/shadow-vm-smoke.ps1;
  scripts/package-stage0.ps1; docs/image-layout-v0.md; docs/PROJECT_STATUS.md.
Allowed write set: docs/plan-reviews/m7-persistence-map-2026-07-06.md only.
Forbidden: any source, script, harness, or release file.
Constraints: update stale line numbers and stale M6-shape claims in place; if a
  DESIGN assumption broke (not just a line number), STOP and report instead of
  redesigning.
Definition of done: map matches HEAD; commit "M7-0: map revalidated against HEAD".
Report format: list of corrected claims (old -> new), list of broken design
  assumptions (must be empty to proceed), commit hash.
```

STOP-tripwires: any M6 output shape differs from section 2's last bullet; the
write-boundary denial chain was restructured beyond line-number drift.

### Slice M7A-1: GPT persist test disk + harness wiring

Capability: a harness run can attach a real GPT disk with SEED_ESP_A/B +
SEED_DATA (superblock + empty regions) without touching production images.
Files: NEW `scripts/make-gpt-persist-image.py` (may import Fat32Builder from
`scripts/make-fat32-image.py` for the ESP slots); `scripts/run-stage0-qemu.ps1`
(fourth drive); `vm-harness/shadow-vm-smoke.ps1` + support (provision disk,
`-PersistDiskPath`); NEW `vm-harness/shadow-vm-smoke-profile-persistence.ps1`
(host-side-only needles this slice: disk exists, GPT/CRC/superblock verified by
a script check; kernel needles come in M7A-2).
Verification: `quick` profile stays green + new persistence profile runs its
host-side checks: `...shadow-vm-smoke.ps1 -Profile persistence`.
Fail-closed: builder refuses output paths under `release/`; no kernel changes.

```text
Packet id: M7A-1-gpt-test-disk
Goal: Build a GPT persist test disk (protective MBR, GPT header+entries+backup,
  SEED_ESP_A/SEED_ESP_B FAT32 64 MiB each, SEED_DATA raw with the section-3.2
  superblock/region layout, type GUID 5EEDDA7A-C0DE-4A55-9A15-000000000001) and
  wire it into QEMU + harness as drive id raiospersist0 bus=ide.3,unit=0.
Read first: docs/plan-reviews/m7-persistence-map-2026-07-06.md sections 3.1-3.3;
  docs/image-layout-v0.md; scripts/make-fat32-image.py; scripts/run-stage0-qemu.ps1;
  vm-harness/shadow-vm-smoke.ps1 and shadow-vm-smoke-support.ps1.
Allowed write set: scripts/make-gpt-persist-image.py (new),
  scripts/run-stage0-qemu.ps1, vm-harness/shadow-vm-smoke.ps1,
  vm-harness/shadow-vm-smoke-support.ps1,
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 (new).
Forbidden: release/**, seed-kernel/**, raios-core/**, any existing profile's
  needle expectations.
Constraints: GPT header + partition-entry-array CRC32s must be correct (verify
  with a python self-check in the builder); superblock sha256 per map 3.2; disk
  created in the harness run dir by default; builder must hard-fail on any
  output path under release/. If bus=ide.3 is rejected by QEMU, report the
  error and the -device list instead of improvising a different bus.
Definition of done: persistence profile passes its host-side needles; quick
  profile green; commit with capability sentence + report filename.
Report format: builder output listing (partition table + superblock hex head),
  passing report JSON path, needle count added.
```

STOP-tripwires: QEMU cannot expose a fourth IDE/AHCI disk (topology decision
needed); any temptation to rebuild release/raios-stage0.img.

### Slice M7A-2: kernel GPT + SEED_DATA read-only detection

Capability: the kernel can find and validate SEED_DATA on a real GPT disk and
report typed layout evidence (present/absent/invalid) — still zero writes.
Files: `seed-kernel/src/ahci.rs` or NEW `seed-kernel/src/gpt.rs` +
`seed-kernel/src/seed_data_layout.rs` (parse GPT header/entries with CRC32,
match ESP + SEED_DATA name/GUID, validate superblock); raios-core record-model
entries for `raios.gpt_layout.v0` / `raios.data_layout.v0` + host tests over
fixture byte arrays; agent protocol read-only query + event needles.
Verify-at-execution: how the AHCI probe enumerates the fourth port/device.
Verification: focused `persistence` profile with new kernel needles
(gpt-header-valid, gpt-crc-checked, gpt-seed-data-found, data-superblock-valid,
gpt-absent-fail-closed via a run without the disk); host tests `cargo test -p raios-core`.
Fail-closed: no write authority anywhere; corrupt GPT/superblock ⇒ explicit
invalid-layout evidence, kernel continues without persistence.

```text
Packet id: M7A-2-gpt-detect
Goal: Read-only GPT parsing + SEED_DATA superblock validation with typed
  evidence records, per map sections 3.2; no write path.
Read first: map sections 2, 3.1-3.2, 4; seed-kernel/src/ahci.rs (probe,
  partition inventory evidence, sector read paths); raios-core/src (record
  model + hasher); vm-harness/shadow-vm-smoke-profile-persistence.ps1.
Allowed write set: seed-kernel/src/gpt.rs (new), seed-kernel/src/seed_data_layout.rs
  (new), seed-kernel/src/ahci.rs (probe hookup only), raios-core/src/** (record
  entries + tests), one agent-protocol read-only query file, event needle
  additions in vm-harness/shadow-vm-smoke-profile-persistence.ps1.
Forbidden: WRITE_DMA_EXT call sites; any write-boundary gate file; attested
  hello/echo/descriptor sources (if a change there proves unavoidable, STOP —
  otherwise no re-sign is expected this slice); release/**.
Constraints: parsing/validation logic lives in raios-core or a host-testable
  unit with byte-fixture tests (GPT header CRC good/bad, truncated table,
  duplicate SEED_DATA names must yield invalid, superblock hash mismatch);
  schemas as record-model entries only; all evidence labeled current_boot.
Definition of done: raios-core host tests green; persistence profile green with
  the new kernel needles including the no-disk fail-closed run; quick green.
Report format: needle names added, report JSON paths (with/without disk),
  host-test count.
```

STOP-tripwires: detection requires touching attested descriptor sources; AHCI
multi-port support turns out to need driver rework (report scope, do not hack).

### Slice M7B-1: record store read/scan (recovery scan, still read-only)

Capability: the kernel can scan RECLOG, validate the full hash chain, report
head/tail/count and torn-tail evidence — before any kernel write exists.
Files: raios-core NEW `durable_record_frame.rs` (frame encode/decode/validate +
chain scan, fully host-tested incl. torn/corrupt fixtures — the framing brain
lives host-testable, the kernel only does sector IO); NEW
`seed-kernel/src/durable_store.rs` (scan over AHCI reads); harness gains a
fixture writer that pre-seeds RECLOG with valid records + one torn tail.
Verification: focused `persistence` (durable-chain-head, torn-tail-detected,
empty-log-valid); `cargo test -p raios-core`.
Fail-closed: appends still `capability_denied` end to end.

```text
Packet id: M7B-1-reclog-scan
Goal: RECLOG frame codec + chain-validating recovery scan with typed evidence;
  read-only. Frame format exactly per map section 3.3.
Read first: map 3.3; raios-core/src (sha256, record model);
  docs/plan-reviews/m3-durable-write-map-2026-07-06.md (discipline);
  seed-kernel/src/durable_store.rs predecessor files if Slice M7A-2 created any.
Allowed write set: raios-core/src/durable_record_frame.rs (new) + tests,
  seed-kernel/src/durable_store.rs (new), minimal ahci.rs read hookup,
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 + support (fixture seeder),
  scripts/make-gpt-persist-image.py (optional --seed-reclog-fixture flag).
Forbidden: any write authority flip; write-boundary gate files; attested
  sources; release/**.
Constraints: host tests must cover: empty log, N valid records, bad magic, bad
  payload hash, bad prev hash, bad seq, torn tail (partial last frame), record
  spanning multiple sectors. Scan stops at first invalid frame and reports it
  as evidence, never as authority.
Definition of done: raios-core tests green; persistence profile green with
  chain + torn-tail needles; quick green; commit with capability sentence.
Report format: fixture description, needle names, report JSON path.
```

STOP-tripwires: none specific beyond globals.

### Slice M7B-2: scoped durable append authority

Capability: the kernel can durably append a typed record to SEED_DATA RECLOG
with append → readback → inspect → only-then-report, under a scoped grant —
the first real persistence write in raiOS.
Files: the M3 write-boundary chain files (scoped positive path for target
`append.record_log.seed_data` only — mirror how M3 scoped
`append.audit_ledger.current_boot`, see the flip list in the M3 map);
`seed-kernel/src/durable_store.rs` (append path over the existing AHCI write,
generalized to multi-sector spans inside the validated RECLOG region —
verify-at-execution how hardcoded the current LBA1 write path is);
raios-core (append planner + tests). First real payloads: boot lifecycle
records mirroring RAM-ring events (ring STAYS authoritative for current_boot
UI; durable records carry the new persistence authority level).
Verification: focused `persistence` (durable-append-authorized,
durable-readback-hash, durable-store-full-denied via a nearly-full fixture,
plus generic-target-still-denied needles); host tests.
Fail-closed: writes outside RECLOG span denied; GPT metadata + superblock +
BOOTCTL + ARTSTOR unwritable this slice; generic module write boundary,
scratch, boot media: all still denied; store-full ⇒ deny, no rotation.

```text
Packet id: M7B-2-durable-append
Goal: Scoped append authority for target append.record_log.seed_data with the
  M3 transaction discipline (build frame -> verify region -> write -> readback
  -> inspect -> report), per map section 3.3. No other target gains authority.
Read first: map 3.3; docs/plan-reviews/m3-durable-write-map-2026-07-06.md
  (Minimal Grant Design + flip list); the eight
  seed-kernel/src/agent_protocol_module_write_boundary_*.rs files;
  seed-kernel/src/durable_store.rs; seed-kernel/src/ahci.rs write/readback path.
Allowed write set: seed-kernel/src/durable_store.rs, the write-boundary files
  (scoped additions only), seed-kernel/src/ahci.rs (multi-sector span write
  within validated region), raios-core/src/** (+tests),
  vm-harness/shadow-vm-smoke-profile-persistence.ps1 + support.
Forbidden: flipping any shared writes_enabled / generic authorizes_append;
  touching RAIOS_AUDITRB_V0 semantics; BOOTCTL/ARTSTOR writes; attested
  sources; release/**.
Constraints: every gate flip must name the exact seed_data target id; add
  explicit needles proving the OLD generic denials still hold; if the existing
  AHCI write path cannot take a span parameter without restructuring, report
  the restructure size before doing it.
Definition of done: persistence profile green including still-denied needles;
  quick green; full-module-audit-rollback profile unchanged-green.
Report format: list of flipped gate fields with file:line, needle diff summary,
  report JSON paths.
```

STOP-tripwires: any change that would widen write authority beyond the RECLOG
span; AHCI restructure exceeding ~500 changed lines (report first).

### Slice M7C-1: boot-control read + state model + SAFE posture

Capability: the kernel reads `raios.boot_control.v0` from BOOTCTL (ping-pong,
highest-valid-seq), evaluates the pending/last_good/failure state machine, and
enters an honest SAFE posture when control is invalid or safe_mode is set.
Files: raios-core NEW `boot_control.rs` (slot codec + state machine, host
tests: fresh disk, valid A, valid A+B pick-higher-seq, both-invalid, pending
without success, failure-count threshold); NEW `seed-kernel/src/boot_control.rs`
(read + posture wiring: SAFE ⇒ persistence-writes denied flag consumed by M7B
gates and later by M7D); builder seeds an initial control record.
Verification: focused `persistence` (boot-control-read,
safe-posture-both-slots-invalid via corrupted fixture,
pending-not-consumed-in-safe); host tests.
Fail-closed: SAFE posture disables ALL durable writes except (later) SAFE-audit;
invalid control never selects a candidate; nothing marked good.

```text
Packet id: M7C-1-boot-control-read
Goal: Boot-control slot codec + state machine + SAFE posture, read-only against
  BOOTCTL, exactly the raios.boot_control.v0 fields from docs/image-layout-v0.md
  with the slot/seq storage from map section 3.4.
Read first: map 3.4; docs/image-layout-v0.md (Boot Control State, SAFE Mode,
  Boot Flow); raios-core record model; seed-kernel/src/durable_store.rs.
Allowed write set: raios-core/src/boot_control.rs (new) + tests,
  seed-kernel/src/boot_control.rs (new), scripts/make-gpt-persist-image.py
  (initial control record), persistence profile + support (fixtures/needles).
Forbidden: writing BOOTCTL from the kernel (next slice); slot switching;
  attested sources; release/**.
Constraints: state machine is pure host-tested logic in raios-core; kernel only
  feeds it sector bytes and posture flags; schema as record-model entry.
Definition of done: host tests cover the six listed scenarios; persistence
  profile green with SAFE-posture needles; quick green.
Report format: state-machine test matrix, needle names, report JSON path.
```

STOP-tripwires: any pressure to auto-recover by rewriting control instead of
entering SAFE posture.

### Slice M7C-2: boot-success marker write + v0-manual slot switch script

Capability: a booted kernel that meets the section-3.4 success criteria durably
marks boot success (ping-pong control write + RECLOG audit record), advances
last_good per spec, and an owner-invoked script can stage/select slots offline.
Files: `seed-kernel/src/boot_control.rs` (success evaluation + verified write
through a BOOTCTL-scoped grant — a SECOND scoped target,
`replace.boot_control.seed_data`, same discipline, still no generic authority);
NEW `scripts/switch-boot-slot.ps1` (host-side: validate GPT image, copy slot
payload into SEED_ESP_A/B, set pending via direct control-slot write with the
same codec — implement by calling a small raios-core host binary or a python
port of the codec; pick whichever exists cheaper at execution time and say so);
harness: probation scenario fixtures.
Verification: focused `persistence` (boot-success-marked,
boot-control-write-pingpong, last-good-advance, failure-count fixture keeps
last_good); plus one honest EXPERIMENT (non-gating): attempt booting the GPT
disk directly in QEMU/OVMF to observe ESP selection — record findings in the
map addendum; deterministic slot boot is NOT claimed in M7.
Fail-closed: success never marked in SAFE mode or when any criterion is unmet;
pending is never consumed in SAFE; the script refuses to touch any image under
`release/` and refuses non-GPT images.

```text
Packet id: M7C-2-boot-success-write
Goal: Kernel writes boot-success into BOOTCTL via scoped
  replace.boot_control.seed_data grant (write loser slot -> readback -> verify
  -> seq wins), appends a RECLOG audit record, advances last_good per
  docs/image-layout-v0.md rules; plus scripts/switch-boot-slot.ps1 for
  owner-invoked v0-manual slot staging/pending.
Read first: map 3.4; docs/image-layout-v0.md (Rules, Boot Flow, Atomic Writes);
  raios-core/src/boot_control.rs; seed-kernel write-boundary files (M7B-2 flip
  pattern); scripts/make-gpt-persist-image.py.
Allowed write set: seed-kernel/src/boot_control.rs, write-boundary files
  (second scoped target only), raios-core/src/** (+tests),
  scripts/switch-boot-slot.ps1 (new), persistence profile + support.
Forbidden: generic write authority; firmware/UEFI boot-variable manipulation;
  touching release/raios-stage0.img or building GPT boot images into release/;
  attested sources.
Constraints: success criteria are exactly the map-3.4 list, evaluated once,
  evidence-logged; the QEMU direct-GPT-boot experiment is observation-only and
  must not become a gating needle; script must print a dry-run plan and require
  an explicit -Apply switch.
Definition of done: persistence profile green with success/pingpong/last-good
  needles; host tests green; experiment findings appended to the map; quick green.
Report format: flipped gate list, success-criteria evidence sample, experiment
  observations (OVMF ESP selection behavior), report JSON paths.
```

STOP-tripwires: anything requiring bootloader replacement, UEFI variable writes,
or real-hardware boot changes ⇒ owner decision first (likely a new ADR).

### Slice M7D-1: persistent artifact store

Capability: a successfully M6-promoted candidate can be durably persisted —
blob in ARTSTOR + chained `raios.artifact_persist.v0` record binding the full
evidence chain — and enumerated after a rescan, still inert without re-verification.
Files: NEW `seed-kernel/src/artifact_store.rs` (blob write via third scoped
target `blob.artifact_store.seed_data`, bump allocation rebuilt from RECLOG);
raios-core (blob frame codec + persist-record entry + tests); M6 promotion path
gains a post-promotion persist step (verify-at-execution where the M6C
transaction code landed; if it lives near attested sources, run the descriptor
re-sign flow via target/descriptor-resign after edits).
Verification: focused `persistence` (artifact-persisted, blob-hash-verified,
blob-without-record-is-garbage fixture, persist-denied-in-safe); host tests.
Fail-closed: persist denied unless the promotion transaction verified this
boot; denied in SAFE; blob region never executable/loadable without M7D-2 gates.

```text
Packet id: M7D-1-artifact-persist
Goal: Persist a promoted candidate: ARTSTOR blob (map 3.5 frame) + chained
  RECLOG artifact_persist record binding artifact/manifest/VM-report/grant/
  promotion-transaction hashes; enumeration by RECLOG scan; no load authority.
Read first: map 3.5; the M6C promotion transaction implementation (locate via
  docs/PROJECT_STATUS.md at execution time); seed-kernel/src/durable_store.rs;
  raios-core durable_record_frame.rs.
Allowed write set: seed-kernel/src/artifact_store.rs (new), write-boundary
  files (third scoped target only), the M6 promotion completion path file,
  raios-core/src/** (+tests), persistence profile + support.
Forbidden: any instantiate/load from stored blobs; generic write authority;
  release/**. If the promotion path file is attestation-covered, complete the
  descriptor re-sign flow (target/descriptor-resign) and prove build.rs
  verification passes — do NOT hand-edit hashes.
Constraints: allocation state must be derived only from the RECLOG scan (host
  test: rebuild after simulated reboot); a blob whose RECLOG record is missing
  or unchained is reported as garbage evidence.
Definition of done: persistence profile green incl. safe-denied and
  garbage-blob needles; host tests green; quick green.
Report format: persist record sample (hashes redacted to prefixes), flipped
  gate list, re-sign evidence if applicable, report JSON paths.
```

STOP-tripwires: persist step requires weakening any M6 gate; attested-source
edits ballooning beyond the promotion completion hook.

### Slice M7D-2: boot-time re-promotion + two-boot proof

Capability: THE PRODUCT MOMENT — a promoted service survives reboot: on boot 2
the stored artifact re-verifies through the SAME evidence chain and answers
live; anything failing re-verification stays inert with typed evidence.
Files: NEW `seed-kernel/src/repromotion.rs` (post-boot-control, non-SAFE scan →
re-verify per map 3.5 → feed the normal M6 gate chain → instantiate);
`raios.repromotion.v0` record entries + RECLOG audit of each grant/denial; NEW
`vm-harness/shadow-vm-persistence-reboot.ps1` two-boot wrapper (+ profile
needles); corrupted-blob and tampered-record fixtures for the denial path.
Verification: NEW `persistence-reboot` profile (service-answers-after-reboot,
repromotion-granted, repromotion-denied-hash-mismatch,
repromotion-skipped-in-safe) + focused `persistence` regression. Command:
`powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-persistence-reboot.ps1`.
Fail-closed: SAFE ⇒ zero re-promotion; any hash/chain/transaction mismatch ⇒
inert + evidence; re-promotion uses no bypass entry point into wasm_runtime —
same slot allocator, same grant checks, same inventory path as M6.

```text
Packet id: M7D-2-repromotion-reboot
Goal: Boot-time re-promotion per map 3.5 through the unmodified M6 gate chain,
  plus the two-boot harness wrapper proving promote -> persist -> reboot ->
  re-verify -> service answers, and proving corrupted artifacts stay inert.
Read first: map 3.5 and 4; seed-kernel/src/artifact_store.rs, repromotion
  predecessors, the M6 gate chain files (locate via PROJECT_STATUS);
  vm-harness/shadow-vm-smoke.ps1 + support (reuse, do not fork logic).
Allowed write set: seed-kernel/src/repromotion.rs (new), minimal boot-sequence
  hookup, raios-core/src/** (+tests),
  vm-harness/shadow-vm-persistence-reboot.ps1 (new), persistence +
  persistence-reboot profile files, shadow-vm-smoke-support.ps1 (shared helpers
  only), scripts/run-stage0-qemu.ps1 only if a keep-disk flag is missing.
Forbidden: new load entry points bypassing M6 gates; auto-load in SAFE mode;
  weakening any denial needle; release/**; attested sources (STOP if needed).
Constraints: boot 2 must run from the SAME persist disk file (KeepImage-style);
  the merged report stays schema raios.vm_test_report.v0 with per-phase
  command/predicate attribution; denial fixtures corrupt (a) blob bytes and
  (b) the persist record hash binding — both must yield repromotion_denied.
Definition of done: persistence-reboot profile green incl. both denial
  fixtures; persistence + quick green; commit with the capability sentence
  "a promoted service now survives reboot under re-verified evidence".
Report format: two-phase report JSON path, boot-2 service answer transcript
  excerpt, denial-fixture evidence lines.
```

STOP-tripwires: re-promotion cannot reuse the M6 chain without modifying it
(design divergence — report, do not fork a parallel trust path).

### Slice M7-CLOSE: full-profile closure + docs

Capability: M7 is claimable — full Shadow VM profile green with all persistence
needles folded in; docs and owner dashboard reflect real persistence.
Files: fold stable persistence needles into
`vm-harness/shadow-vm-smoke-profile-full-*.ps1` (new
`shadow-vm-smoke-profile-full-persistence.ps1` if cleaner); update
`docs/image-layout-v0.md` (addendum: raw region map realization, superblock,
type GUID, OVMF findings, honest slot-boot status); `docs/ROADMAP.md` cursor,
`docs/PROJECT_STATUS.md`, `docs/OWNER_DASHBOARD.md` (plain language: "things
the system learned and services it was granted now survive switching it off").
Verification: `powershell -NoProfile -ExecutionPolicy Bypass -File
vm-harness\shadow-vm-smoke.ps1 -Profile full` green + `scripts\scan-secrets.ps1`
clean + both focused persistence profiles green in the same session.
Fail-closed: closure blocked while ANY profile is red (Red Gate Rule).

```text
Packet id: M7-CLOSE
Goal: Fold persistence needles into the full profile, update
  image-layout-v0.md addendum + ROADMAP/PROJECT_STATUS/OWNER_DASHBOARD, run
  full profile + secret scan, close M7.
Read first: map section 4; all persistence profile files; docs conventions in
  docs/ROADMAP.md and docs/OWNER_DASHBOARD.md.
Allowed write set: vm-harness/shadow-vm-smoke-profile-full-persistence.ps1
  (new) or additions to existing full profile files, vm-harness/shadow-vm-smoke.ps1
  (full-profile dispatch only), docs/image-layout-v0.md, docs/ROADMAP.md,
  docs/PROJECT_STATUS.md, docs/OWNER_DASHBOARD.md.
Forbidden: kernel/source changes (repair-only if full goes red — then this
  packet pauses and Red Gate rules apply); release image rebuilds.
Definition of done: full profile report result: passed, newer than last commit;
  secret scan clean; owner dashboard updated in plain language; commit names
  the passing report file.
Report format: full report JSON path + sha256, list of docs updated, the
  capability sentence for the milestone.
```

STOP-tripwires: full profile red for non-persistence reasons (Red Gate — stop
feature work); any doc claim the needles do not back.

## 6. Global STOP-tripwires (every slice, orchestrator must halt and ask owner)

- Anything requiring a NEW ADR: unparking ota/registry/fake-cloud, any external
  artifact download path, trust-model or attestation-key changes, bootloader
  replacement, UEFI variable manipulation, real-hardware boot changes.
- Any write path that could touch: the boot disk, GPT metadata sectors,
  SEED_DATA LBA0/1, `RAIOS_AUDITRB_V0` LBA0, or anything under `release/`
  (especially overwriting `release/raios-stage0.img`), or any destructive
  operation on a disk not created by the harness in that run.
- Persisting any secret (provider keys, Wi-Fi credentials, tokens) — the
  image-layout-v0.md denial list is binding until a sealed-secret design ADR.
- Converting the production boot image to GPT (explicitly deferred past M7).
- Any generic (non-scoped) durable-write authority flip.
- A worker proposing to weaken/delete existing denial needles.

## 7. Estimate and verdict

Nine implementation slices plus revalidation and closure. The risky boundaries
are M7B-2 (first real persistence write — keep it as scoped as M3 was), M7C-2
(bootloader honesty — do not overclaim slot boot), and M7D-2 (the reboot proof;
it is also the payoff). Three OWNER DECISIONS are marked inline (3.2 storage
mechanism, 3.3 capacity posture, 3.4 firmware slot selection fallback), each
with a recommendation, so a cheap orchestrator only escalates, never designs.
M7 ends the "everything is current_boot" era for exactly three scoped targets
and nothing else; every other write stays capability_denied.
