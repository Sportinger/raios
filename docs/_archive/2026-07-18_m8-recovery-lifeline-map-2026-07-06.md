# M8 Recovery Agent Lifeline — Design Map (2026-07-06)

## Header

- **Authored 2026-07-06 AHEAD of execution as pre-planning.** No M8 code exists
  yet. Every file:line claim below was checked against HEAD on 2026-07-06 but
  M6 and M7 will move these files before M8 runs.
- **Execution preconditions:** M6 Promotion Loop v0 CLOSED (all of M6A-D green,
  final full profile passed). M8A and M8B need only M6. M8C additionally needs
  M7C (boot control / last-good / SAFE) CLOSED. M8D additionally needs M7D
  (persistent artifact store) CLOSED. The M7 map
  (`docs/plan-reviews/m7-persistence-map-2026-07-06.md`, if authored) must be
  read before M8C/M8D slices are dispatched.
- **MANDATORY Slice 0 = map revalidation** (below). Re-check every file:line
  claim against HEAD, update this map FIRST if reality diverged, commit the map
  update before any implementation slice.
- This map is written for a cheap orchestrator: no open judgment calls except
  the explicitly marked OWNER DECISION items.

## What M8 Is

ADR 0003 and the archived Phase 8
(`docs/archive/roadmap-2026-07-04-pre-restructure.md:1874`) define a recovery
agent lifeline: a tiny pinned control path in the trusted base, separate from
the rich provider path, that still works when the replaceable world above is
broken. `device-protocol/recovery-v0.md` already specifies the protocol
(`raios.recovery.v0`): five methods (`recovery.snapshot`,
`recovery.restart_last_good`, `recovery.disable_module`, `recovery.rollback`,
`recovery.load_artifact_by_hash`), explicit recovery trust states, and
denied-by-default rules. Today that spec is vocabulary only.

**M8 makes the lifeline real:** over serial, always, the agent can (1) read a
real recovery snapshot, (2) disable a module/service by id for the current
boot, (3) restart the known-good service set, (4) load a recovery artifact by
hash from the local recovery store (M8D, after M7D) — and all of this keeps
working when the Wasm service world is wedged, proven by a real fault-injection
needle.

## Exists / Missing

| Area | Exists today (verified 2026-07-06) | Missing for M8 |
|---|---|---|
| Protocol spec | `device-protocol/recovery-v0.md` full method set, trust states, denial rules | Runtime implementation of any method |
| Recovery vocabulary | `seed-kernel/src/agent_protocol_recovery.rs` (6,167 lines): diagnostics + selftests for identity/trust/vm-test/approval/loader/rollback-evidence, lifeline request/protocol/vocabulary (`:981`), command admission/envelope/dispatch/canonicalization/handler-binding, and per-method target-binding diagnostics: `disable_module` `:3938`, `restart_last_good` `:4087`, `load_artifact_by_hash` `:4242` | All of these are denial/evidence emitters. There is NO real `recovery.snapshot`, no executor, no state mutation. Dispatch entries in `agent_protocol.rs:410-460+` are `*_diagnostic`/`*_selftest` only |
| Lifeline transport | Serial agent protocol works (TCP serial in VM, real serial on hardware) | Nothing distinguishes lifeline dispatch from normal dispatch; no pinned method table; no proof it survives a wedged service world |
| Service health | `current_boot_service.rs:36-38` health values `running`/`stopped`/`missing`; `:149` `health_state()`; `service_inventory.rs:162` `service_health()`; echo lifecycle `echo_service.rs:285/:333/:385/:426` | No `crashed`/`wedged` health value; no crash record; no supervisor that detects a trapped Wasm instance; no restart-from-known-good path |
| Fault path | wasmi fuel metering + StoreLimits in `wasm_runtime.rs` (fuel: verify exact lines at execution) — a fuel-exhausted invoke traps for real | No labeled fault-injection command; no needle proving the lifeline answers while a service is wedged |
| Audit for mutations | M3 durable append/readback/inspect to LBA1 `RAIOS_AUDITRB_V0` (`raios-core/src/scoped_rollback_apply.rs`, hello-scoped constants at `:16`); M6 will generalize it for promotion/rollback transactions | No recovery-action record kind; lifeline mutations must reuse the same append/readback/inspect discipline |
| Last-good / SAFE | `docs/image-layout-v0.md` (DRAFT): control.json, A/B slots, last-good, SAFE mode — no kernel write path | M7C delivers it; M8C binds the lifeline to it |
| Verification | `-Profile recovery` (registered `vm-harness/shadow-vm-smoke.ps1:212`), 3,644/3,644 needles green (latest: `shadow-20260705-131513-9748.json`), plus 5 more recovery-named focused profiles | No positive-behavior lifeline profile; no wedge-survival needles |

## Design Decisions (fixed, not open)

### D1. "Pinned trusted base" — what it concretely means today
There is no separate address space yet; the lifeline lives inside the kernel
image. "Pinned" is therefore enforced by construction and by evidence, not by
hardware isolation:

1. **Separate frozen method table.** Lifeline methods live in a dedicated
   `LIFELINE_METHODS` table (own module, e.g.
   `seed-kernel/src/recovery_lifeline.rs`), not interleaved into the ~180-entry
   `AGENT_METHODS`. The serial dispatcher checks `LIFELINE_METHODS` first and
   that path must not call into: wasm execution, provider/TLS/HTTP code
   (`openai.rs`, `openai_trust.rs`), or service-graph mutation helpers — only
   read-only inventory/health views, the fault/crash records, and the narrow
   recovery executor added by M8B.
2. **No blocking work.** Per ADR 0003, lifeline handlers must not perform
   synchronous network/TLS/provider work. Serial in, serial out, bounded time.
3. **Vocabulary hash needle.** The lifeline method table renders a stable
   `lifeline_vocabulary_sha256`; the focused profile pins it so silent growth
   of lifeline authority fails the gate.
4. **Wedge-survival evidence.** The dedicated focused profile wedges a real
   service and proves the lifeline still answers (Slice M8A-3).

Real isolation (own address space / core-generation update) stays post-M11.

### D2. Transport
Serial is THE lifeline transport for v0 (`trust_state`:
`local_physical_console` per `device-protocol/recovery-v0.md`). A pinned
minimal provider route is explicitly out of M8 — see OWNER DECISION 1.

### D3. Authority model — the lifeline is narrower, never broader
The lifeline restores KNOWN-GOOD state only. It must never:
- promote or accept NEW artifacts, bytes, URLs, code, or provider text
  (`load_artifact_by_hash` takes a hash of bytes ALREADY in the local recovery
  store with a full pre-existing evidence chain — M8D);
- start anything without a hash-verified known-good pointer;
- disable `core_owned` services or the lifeline endpoint itself;
- write persistence in `safe_mode`;
- exceed what the normal M6 promotion path could authorize.
Every mutating lifeline action appends/reads-back/inspects a durable recovery
action record (M3 discipline on `RAIOS_AUDITRB_V0`; M7B store once it exists)
BEFORE mutating live state. Missing evidence = typed `capability_denied`.

### D4. Profile strategy: NEW focused profile, existing one stays frozen
The existing `recovery` profile (3,644 needles) is the byte-exactness fence
over the denial/diagnostic vocabulary from the M2 collapse. M8 does NOT edit
it. M8 adds a NEW focused profile `recovery-lifeline`
(`vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1`, registered in the
profile validation block at `shadow-vm-smoke.ps1:167-222`), covering positive
lifeline behavior + wedge survival + fail-closed denials. New REAL methods get
the spec's names (`recovery.snapshot`, ...), which do not collide with the
existing `*_diagnostic` method names, so the old profile stays green untouched.
If any slice would flip an existing needle in the frozen `recovery` profile,
that is a STOP-tripwire (ask the owner).

Run commands used throughout:
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile recovery-lifeline
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile recovery   # must stay 3644/3644
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick
```

### D5. Schemas via record model only
`raios.recovery_snapshot.v0` and the recovery action record are added as
raios-core record-model entries (mechanism-before-vocabulary, ADR 0005 §3),
rendered through the shared factory the M2 collapse produced — no hand-rolled
emit/hash code. Everything is labeled `current_boot` until M7B/M7C say
otherwise. Snapshot redaction per spec: no keys, no Wi-Fi passphrases, no raw
prompts, no raw crash memory.

### D6. Sub-milestones
- **M8A Lifeline channel + read-only snapshot** (needs only M6): pinned table,
  real `recovery.snapshot`, wedge-survival proof.
- **M8B Bounded current-boot repair** (needs only M6): `disable_module` by id,
  `restart_last_good` v0 over the current-boot verified set (the boot-time
  attested built-in set IS a hash-identified last-good set for this boot —
  honest, labeled `current_boot`).
- **M8C Durable last-good integration** (needs M7C): pointer source becomes
  control.json last-good; SAFE-mode state in snapshot; denials when
  boot-success marker missing.
- **M8D Recovery artifact load by hash** (needs M7D): from local recovery
  store only, full evidence chain, never fetches, never promotes new code.

## OWNER DECISIONS

1. **Second lifeline transport (pinned minimal provider route).** Options:
   (a) serial-only for all of M8, provider lifeline route deferred to its own
   ADR at M10+ — RECOMMENDED (a provider route touches trust, keys, and
   network availability; it must not ride in on a milestone about surviving a
   broken world); (b) design the pinned route inside M8 (requires a new ADR
   NOW + recovery trust gate); (c) local physical link other than serial (no
   hardware story today). Default if unanswered: (a).
2. **`recovery.rollback` (persistent pointer switch) through the lifeline.**
   Options: (a) M8C exposes only a read-only rollback PREVIEW via the
   lifeline; the mutating pointer switch stays where M7C built it —
   RECOMMENDED (one authority path, no duplicate mutation surface);
   (b) full mutating `recovery.rollback` in M8C; (c) omit entirely. Default:
   (a).
3. **Where the wedged health value surfaces.** Options: (a) add `crashed` to
   the health lattice in `current_boot_service.rs` + inventory render —
   RECOMMENDED; (b) keep `stopped` plus a separate crash record. Default: (a).

## Slice Plan

### Slice 0 (MANDATORY): map revalidation
Re-check against HEAD after M6 (and M7 for C/D slices): line numbers in the
Exists/Missing table; whether M6 changed `agent_protocol.rs` dispatch, the
service-slot/inventory files, or `scoped_rollback_apply.rs` generalization;
whether `recovery.snapshot` (real) appeared meanwhile; fuel-metering call
sites in `wasm_runtime.rs`; profile registration block position in
`shadow-vm-smoke.ps1`; whether the `recovery` profile is still 3,644 needles.
Update THIS map first, commit (`M8-0: map revalidation`), then dispatch M8A-1.
STOP if: M6 left the full profile red (Red Gate), or M6's rollback engine
diverged from what M8B assumes (no generic verified-apply available).

---

### M8A-1 — Pinned lifeline method table + dispatch isolation
1. **Capability:** the agent can ask the kernel for its pinned lifeline
   command set and the kernel proves (typed evidence) that lifeline dispatch
   is a separate frozen path that never routes through wasm/provider/TLS code.
2. **Files:** NEW `seed-kernel/src/recovery_lifeline.rs` (LIFELINE_METHODS
   table + dispatch entry); `seed-kernel/src/agent_protocol.rs` (route
   lifeline lookup first; verify exact hook point at execution — M6 will have
   touched this file); raios-core record entry for
   `raios.recovery_lifeline_table.v0` (table render + vocabulary hash).
3. **Verification:** focused, NEW profile created in M8A-3; for THIS slice run
   `-Profile quick` plus `-Profile recovery` (must stay 3,644/3,644
   byte-identical) plus host tests (`cargo test -p raios-core`). New evidence:
   serial `recovery.lifeline_table` renders the five spec method names, each
   with `implemented=true|false` honestly set, plus
   `lifeline_vocabulary_sha256`.
4. **Fail-closed:** all five methods still answer `capability_denied` (only
   the table read is new); table lists `transport=serial_local`,
   `trust_state=local_physical_console`; no mutation anywhere.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8A-1 pinned lifeline method table
Goal: Add a dedicated frozen LIFELINE_METHODS table + read-only recovery.lifeline_table method, dispatched before AGENT_METHODS, with a record-model rendered table + vocabulary hash; all five raios.recovery.v0 methods stay capability_denied.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, device-protocol/recovery-v0.md, seed-kernel/src/agent_protocol.rs (dispatch + method! macro), seed-kernel/src/agent_protocol_recovery.rs (existing vocabulary emitter :981), raios-core/src/ record model files, AGENTS.md.
Allowed write set: seed-kernel/src/recovery_lifeline.rs (new), seed-kernel/src/agent_protocol.rs (dispatch hook + mod decl only), seed-kernel/src/main.rs or lib.rs mod list if needed, raios-core/src/<record model file for the new schema>, raios-core host tests.
Forbidden: vm-harness/**, any hello_service/** or attested descriptor sources, echo_service.rs, wasm_runtime.rs, openai*.rs, docs/** except nothing, release/**. If the build fails with a descriptor attestation mismatch you touched something attested: STOP and report.
Constraints: no new hand-rolled emit/hash code — the table renders through the raios-core record model + shared factory; lifeline dispatch must not call into wasm_runtime, openai, or service mutation helpers; &'static str lattices for all vocabulary; everything labeled current_boot; no heap-unbounded input parsing.
Definition of done: cargo fmt --all -- --check clean; cargo test -p raios-core green; kernel builds (scripts/build-seed-kernel.ps1 -Profile release); serial `agent recovery.lifeline_table` renders schema raios.recovery_lifeline_table.v0 with 5 methods, implemented flags all false except none, and lifeline_vocabulary_sha256; recovery.restart_last_good et al. still return capability_denied.
Report format: files changed; verbatim output of fmt + host tests + build; the rendered table JSON copied from a local serial run if you ran one, else state you could not run QEMU; risks; out-of-scope observations (report, don't fix).
Stop conditions: dispatch hook cannot be added without restructuring AGENT_METHODS; attestation mismatch; any existing test/needle would change.
```
6. **STOP-tripwires:** needing to restructure `AGENT_METHODS` itself; any
   change to the frozen `recovery` profile output; attested-source touch.

---

### M8A-2 — Real `recovery.snapshot` (read-only)
1. **Capability:** the agent can read a real `raios.recovery_snapshot.v0` over
   serial: core state, trust state, live-world state, crashed/disabled
   services, allowed vs denied actions with reasons — the first spec method
   that actually works.
2. **Files:** `recovery_lifeline.rs` (handler); raios-core record entry for
   `raios.recovery_snapshot.v0` (shape per `device-protocol/recovery-v0.md:96`
   — but `last_good_set`/`boot_success_mark` honestly rendered as
   `current_boot`/`missing` until M7C); reads from `service_inventory.rs` and
   `current_boot_service.rs` health views (read-only). Verify at execution
   which inventory accessor M6 left behind.
3. **Verification:** `-Profile quick` + `-Profile recovery` unchanged; host
   tests for the record shape. New evidence needles land with M8A-3's profile:
   snapshot schema line, `"safe_mode": false`, `"trust_state":
   "local_physical_console"`, `allowed_actions` containing only
   `recovery.snapshot` + `recovery.lifeline_table`, `denied_actions` naming
   the other four with reasons.
4. **Fail-closed:** snapshot contains NO keys/passphrases/prompts/raw memory
   (redaction asserted by negative needles: profile greps serial log for the
   embedded test key marker and Wi-Fi pass — zero hits); mutating methods
   still denied; unknown-method still denied.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8A-2 real recovery.snapshot
Goal: Implement recovery.snapshot as a real read-only lifeline method rendering raios.recovery_snapshot.v0 from live inventory/health state via the record model.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, device-protocol/recovery-v0.md (snapshot shape :96-149), seed-kernel/src/recovery_lifeline.rs, seed-kernel/src/service_inventory.rs, seed-kernel/src/current_boot_service.rs, raios-core record model, AGENTS.md.
Allowed write set: seed-kernel/src/recovery_lifeline.rs, raios-core/src/<record model file>, raios-core host tests.
Forbidden: agent_protocol.rs beyond flipping the lifeline table implemented flag for recovery.snapshot, vm-harness/**, echo/hello service sources, wasm_runtime.rs, openai*.rs. Attestation mismatch = STOP.
Constraints: read-only — zero state mutation; record-model rendering only; secrets never enter the snapshot (no api key, ssid, passphrase fields or values); fields for last_good/boot_success render honest placeholders scope=current_boot / "missing" — do NOT invent persistence; denied_actions must carry typed reasons matching the spec's missing_evidence style.
Definition of done: fmt + raios-core tests green; kernel builds; serial `agent recovery.snapshot` renders the schema with core/trust/live_world/crashed_services/disabled_modules/allowed_actions/denied_actions; the other four methods still capability_denied; lifeline_vocabulary_sha256 updated deliberately (report old+new).
Report format: files changed; verbatim fmt/test/build output; rendered snapshot JSON if locally run; old and new vocabulary hash; risks; out-of-scope notes.
Stop conditions: inventory accessors insufficient without modifying forbidden files; any secret value would be needed to render a field.
```
6. **STOP-tripwires:** any field requiring persistence that doesn't exist
   (would be fake); secret-bearing field in the snapshot.

---

### M8A-3 — Fault injection + wedge-survival proof + NEW focused profile
1. **Capability:** with the echo Wasm service really wedged (fuel-exhaustion
   trap), the lifeline still answers: `recovery.snapshot` lists
   `svc.demo.echo` as crashed — proven by a repeatable focused profile.
2. **Files:** `seed-kernel/src/echo_service.rs` + `wasm_runtime.rs` (a labeled
   test-infrastructure method, e.g. `echo.invoke_fuel_starved`, that performs
   a REAL invoke with a tiny fuel budget → real wasmi trap → health flips to
   `crashed`); `current_boot_service.rs` (add `crashed` to the health lattice,
   per OWNER DECISION 3 default); NEW
   `vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1`;
   `vm-harness/shadow-vm-smoke.ps1` (register `recovery-lifeline` in the
   profile block, `:167-222`, verify position at execution).
3. **Verification:** NEW focused profile `recovery-lifeline`:
   `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile recovery-lifeline`.
   Needle groups: (a) baseline echo `running` + snapshot healthy; (b)
   fuel-starved invoke → trap event (typed, `current_boot`) + health
   `crashed`; (c) LIFELINE STILL ANSWERS: `recovery.lifeline_table` and
   `recovery.snapshot` render fully, `crashed_services` contains
   `svc.demo.echo` with a stable `last_error_id`; (d) mutating methods still
   denied; (e) redaction negatives from M8A-2; (f) frozen `-Profile recovery`
   re-run stays 3,644/3,644. Also `-Profile quick` green.
4. **Fail-closed:** the fault-injection method is itself bounded test
   infrastructure: it only fuel-starves echo (fixed id), is labeled
   `test_infrastructure=true` in its event, and cannot target other ids; the
   wedge must NOT take down the dispatcher (trap is caught, service marked
   crashed, kernel loop continues).
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8A-3 wedge fault injection + recovery-lifeline profile
Goal: Add a labeled fuel-starved echo invoke that really traps and marks svc.demo.echo crashed, and a new focused VM profile proving the lifeline still answers while echo is wedged.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, seed-kernel/src/echo_service.rs, seed-kernel/src/wasm_runtime.rs (fuel/StoreLimits), seed-kernel/src/current_boot_service.rs (health lattice), vm-harness/shadow-vm-smoke-profile-quick.ps1 (profile shape), vm-harness/shadow-vm-smoke.ps1 profile block, AGENTS.md.
Allowed write set: seed-kernel/src/echo_service.rs, seed-kernel/src/wasm_runtime.rs, seed-kernel/src/current_boot_service.rs, seed-kernel/src/recovery_lifeline.rs, seed-kernel/src/agent_protocol.rs (one dispatch entry for echo.invoke_fuel_starved), vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1 (new), vm-harness/shadow-vm-smoke.ps1 (register profile only).
Forbidden: hello_service/**, descriptor_sources.rs, build.rs, existing recovery/quick/full profile scripts, release/raios-stage0.img. Attestation mismatch = STOP.
Constraints: the trap must be REAL (tiny fuel budget on a real invoke), not simulated; catch the trap, set health crashed via the &'static str lattice, keep the cooperative loop alive; event labeled test_infrastructure + current_boot; hard-code target svc.demo.echo; new profile follows the existing needle/report conventions and writes raios.vm_test_report.v0 under release/vm-reports.
Definition of done: fmt green; kernel builds; you may not be able to run QEMU in-sandbox — write the profile and report that the ORCHESTRATOR must run: -Profile recovery-lifeline (new, all needles green), -Profile recovery (3644/3644 unchanged), -Profile quick. Needle groups a-f from the map slice all present in the profile script.
Report format: files changed; verbatim fmt/build output; list of needles added with exact needle strings; risks; out-of-scope notes.
Stop conditions: trap cannot be caught without unwinding/aborting the kernel loop; health lattice change would alter existing profile output; fuel API absent where expected.
```
6. **STOP-tripwires:** the wedge kills the dispatcher (design assumption
   broken — lifeline is NOT independent; stop, report to owner, this changes
   M8's shape); frozen `recovery` profile needle flips.

**M8A close:** orchestrator runs recovery-lifeline + recovery + quick green,
commits with report filenames, updates PROJECT_STATUS/OWNER_DASHBOARD
(capability sentence: "when a service crashes, the system can still report it
and its recovery options over the lifeline").

---

### M8B-1 — `recovery.disable_module` (current boot)
1. **Capability:** the agent can disable a known replaceable service by exact
   id for the current boot; the disabled service stops answering, inventory
   and snapshot show it disabled, and the decision is durably audited.
2. **Files:** `recovery_lifeline.rs` (executor); `current_boot_service.rs` /
   `echo_service.rs` (stop + disabled flag); raios-core record entry for the
   recovery action record `raios.recovery_action.v0`; durable append via the
   M3/M6 append-readback-inspect path (verify at execution what M6's
   generalized transaction helper looks like — REUSE it, do not fork it).
3. **Verification:** extend `-Profile recovery-lifeline`: disable
   `svc.demo.echo` → action record appended+readback+inspected needles →
   inventory `disabled=true` → echo invoke denied with
   `service_disabled` → snapshot `disabled_modules` lists it. Fail-closed
   needles: unknown id denied, `core_owned` id (e.g. `core.serial`) denied,
   lifeline self-disable denied, pattern/"disable all" denied. Frozen
   `recovery` + `quick` green.
4. **Fail-closed:** exactly the spec's rules (`recovery-v0.md:195-208`);
   mutation happens ONLY after the durable action record passes inspection;
   deny without audit media present.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8B-1 recovery.disable_module current-boot executor
Goal: Implement recovery.disable_module for exact known non-core ids: durable action record first (append/readback/inspect), then stop+disable the service for the current boot; all spec denials fail closed.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, device-protocol/recovery-v0.md:195-208 + denial rules :244, seed-kernel/src/recovery_lifeline.rs, the M6 promotion/rollback transaction helper (locate via docs/plan-reviews/m6-promotion-loop-map-2026-07-06.md and current PROJECT_STATUS), seed-kernel/src/current_boot_service.rs, seed-kernel/src/echo_service.rs, raios-core/src/scoped_rollback_apply.rs.
Allowed write set: seed-kernel/src/recovery_lifeline.rs, seed-kernel/src/current_boot_service.rs, seed-kernel/src/echo_service.rs (disable hooks only), raios-core/src/<record model + action record>, raios-core host tests, vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1 (extend).
Forbidden: hello_service/**, descriptor_sources.rs, build.rs, ahci.rs internals (use the existing append/readback API), frozen profile scripts, release/raios-stage0.img. Attestation mismatch = STOP.
Constraints: REUSE the generalized M6 transaction/append discipline — do not write a second durable-append implementation; deny before mutate on ANY missing evidence; target must be exact id, known, not core_owned, not the lifeline; everything current_boot labeled; record model only for new schema.
Definition of done: fmt + host tests green; kernel builds; profile script extended with the positive chain + 4 fail-closed needles from the map; orchestrator runs recovery-lifeline / recovery / quick.
Report format: files changed; verbatim fmt/test/build output; exact new needle strings; which M6 helper was reused (file:line); risks; out-of-scope notes.
Stop conditions: no reusable M6 durable-transaction helper exists (report — the map assumed M6C delivered it); disable would require touching core-owned dispatch.
```
6. **STOP-tripwires:** M6's transaction helper missing/incompatible;
   any path where disable could hit a `core_owned` service.

---

### M8B-2 — `recovery.restart_last_good` v0 (current-boot verified set)
1. **Capability:** with echo wedged or disabled, the agent can restart the
   last-good service set for the current boot — the kernel re-instantiates the
   boot-time attested built-in set from its verified embedded bytes and health
   returns to `running`.
2. **Files:** `recovery_lifeline.rs`; a RAM last-good pointer = hash of the
   boot-verified service set (compute at boot from the attested
   descriptors — verify at execution where boot-time verification records
   live); restart path re-uses `echo_service.rs` load/start (`:285/:333`,
   verify) — NOT a new loader. Action record as in M8B-1.
3. **Verification:** extend `-Profile recovery-lifeline`: wedge echo (M8A-3
   method) → `recovery.restart_last_good` → action record needles → echo
   `running` again → snapshot healthy → echo invoke actually answers
   (positive round-trip). Fail-closed: restart denied when active set is
   healthy (spec: only when crashed/degraded/stopped); denied when last-good
   pointer absent (host-test the pointer-absent branch). Frozen `recovery` +
   `quick` green. This is the flagship M8 capability — after it lands, run
   FULL profile once as checkpoint.
4. **Fail-closed:** restarts ONLY the hash-verified boot set from attested
   bytes; never loads unknown artifacts; `scope=current_boot` labeled
   explicitly (this is NOT durable last-good yet — M8C upgrades it); denied
   without audit media.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8B-2 restart_last_good over the current-boot verified set
Goal: Implement recovery.restart_last_good: when the active set is crashed/degraded/stopped, append+verify a recovery action record, then re-instantiate the boot-verified attested service set (echo) from embedded bytes and restore health.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, device-protocol/recovery-v0.md:168-193, seed-kernel/src/recovery_lifeline.rs, seed-kernel/src/echo_service.rs (load/start), seed-kernel/src/current_boot_service.rs, M8B-1's action-record code, AGENTS.md.
Allowed write set: seed-kernel/src/recovery_lifeline.rs, seed-kernel/src/echo_service.rs (restart hook only), seed-kernel/src/current_boot_service.rs, raios-core/src/<record model>, raios-core host tests, vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1 (extend).
Forbidden: hello_service/**, descriptor_sources.rs, build.rs, wasm_runtime.rs beyond what restart strictly needs, frozen profiles, release/raios-stage0.img. Attestation mismatch = STOP.
Constraints: restart = re-run the EXISTING verified load/start path, no new loader; last-good pointer is a boot-computed hash over the attested set, rendered scope=current_boot; deny when active set healthy, when pointer absent, when audit append fails; action record before mutation; no unknown artifacts ever.
Definition of done: fmt + host tests green; kernel builds; profile extended with wedge->restart->running->positive echo round-trip chain plus the two denial needles; orchestrator runs recovery-lifeline / recovery / quick and then ONE full profile checkpoint.
Report format: files changed; verbatim fmt/test/build output; exact needle strings; risks; out-of-scope notes.
Stop conditions: re-instantiation requires bypassing any verification step of the normal load path (never bypass — report); restart cannot restore health without rebooting.
```
6. **STOP-tripwires:** any restart shortcut that skips artifact verification;
   full-profile checkpoint red (Red Gate).

**M8B close:** capability sentence for the owner: "if a service breaks, you
can tell the system over the emergency line to shut it off or put the
known-good version back — while it is broken."

---

### M8C-1 — Durable last-good + SAFE mode integration (REQUIRES M7C closed)
1. **Capability:** the lifeline's last-good pointer comes from the durable
   M7C control record; `recovery.snapshot` reports real
   `boot_success_mark`/`safe_mode`; in SAFE mode the lifeline blocks
   persistent writes but still permits bounded current-boot repair.
2. **Files:** `recovery_lifeline.rs` + whatever M7C landed (control.json
   reader, last-good pointer, SAFE flag) — the M7 map owns those paths;
   REVALIDATE this slice against the closed M7C code before dispatch (mini
   Slice 0). Per OWNER DECISION 2 default, add read-only
   `recovery.rollback_preview` rendering the M7C rollback target; the
   mutating switch stays in M7C's path.
3. **Verification:** extend `-Profile recovery-lifeline`: snapshot shows
   durable `last_good_set` hash + `boot_success_mark` present;
   restart_last_good needle now shows `scope` upgraded from `current_boot` to
   the durable pointer source; SAFE-mode needles (M7C profile should already
   boot a SAFE case — reuse its mechanism, verify at execution): in SAFE,
   persistent-write actions denied with `safe_mode`, snapshot
   `safe_mode:true`, `recovery.snapshot` + `disable_module` still work.
   Frozen `recovery` + `quick` green.
4. **Fail-closed:** no restart of a set lacking success marker; SAFE blocks
   all persistence; `rollback_preview` mutates nothing.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8C-1 lifeline x M7C last-good/SAFE integration
Goal: Bind restart_last_good to the durable M7C last-good pointer, surface boot_success_mark and safe_mode in recovery.snapshot, add read-only recovery.rollback_preview, enforce SAFE-mode write blocking in the lifeline.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, the closed M7 map + M7C code (locate via docs/PROJECT_STATUS.md), docs/image-layout-v0.md, seed-kernel/src/recovery_lifeline.rs, device-protocol/recovery-v0.md:210-224.
Allowed write set: seed-kernel/src/recovery_lifeline.rs, raios-core/src/<record model>, raios-core host tests, vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1 (extend). Read M7C modules; modify them ONLY if the orchestrator explicitly extends this write set after Slice-0 revalidation.
Forbidden: destructive disk operations of any kind; writing to boot slots; frozen profiles; release/raios-stage0.img. Attestation mismatch = STOP.
Constraints: lifeline READS M7C state, it does not reimplement it; deny restart when success marker missing; SAFE mode = bounded current-boot repair only (spec table); rollback_preview is render-only.
Definition of done: fmt + host tests green; kernel builds; profile extended per map slice 3.; orchestrator runs recovery-lifeline / recovery / quick.
Report format: files changed; verbatim outputs; exact needle strings; which M7C APIs were consumed (file:line); risks; out-of-scope notes.
Stop conditions: M7C API shape does not match this packet (stop, orchestrator re-plans); anything would write to a boot slot or control record from the lifeline.
```
6. **STOP-tripwires:** any lifeline write to control.json/boot slots (that is
   M7C's authority, not M8's); SAFE semantics ambiguity → owner.

---

### M8D-1 — `recovery.load_artifact_by_hash` (REQUIRES M7D closed)
1. **Capability:** the agent can name an exact hash and the kernel loads that
   artifact from the LOCAL M7D recovery/artifact store — only if the full
   pre-existing evidence chain (manifest, VM report, attestation, grant,
   rollback target) binds that exact hash — restoring a known-good service
   even if the normal promotion path is down.
2. **Files:** `recovery_lifeline.rs`; M7D store read API; M6 evidence-gate
   checks REUSED (verify at execution — same gates, invoked from the lifeline,
   never weakened). No network anywhere in this path.
3. **Verification:** extend `-Profile recovery-lifeline`: stage a known-good
   artifact via M7D's documented path → wedge world → `load_artifact_by_hash`
   → evidence-chain needles (each hash binding checked) → service running.
   Fail-closed needles: unknown hash denied; hash present but ANY evidence
   piece missing denied (`missing_evidence` typed); URL/bytes/code in request
   denied; SAFE + persistence denied. Frozen `recovery` + `quick` green, then
   FULL profile — this closes M8.
4. **Fail-closed:** never fetches (no URL, no provider text, no download);
   never accepts inline bytes; never loads anything the normal M6 path could
   not have authorized; spec `recovery-v0.md:226-242` rules verbatim.
5. **Worker packet:**
```text
You are a worker agent executing one bounded packet. Do not broaden scope.
Packet: M8D-1 recovery.load_artifact_by_hash from the local store
Goal: Implement recovery.load_artifact_by_hash: exact-hash lookup in the M7D local store, full M6 evidence chain re-verified for that hash, durable action record, then load via the existing verified load path.
Read first: docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md, device-protocol/recovery-v0.md:226-242, the closed M7D store API + M6 gate code (locate via docs/PROJECT_STATUS.md), seed-kernel/src/recovery_lifeline.rs.
Allowed write set: seed-kernel/src/recovery_lifeline.rs, raios-core/src/<record model>, raios-core host tests, vm-harness/shadow-vm-smoke-profile-recovery-lifeline.ps1 (extend).
Forbidden: net.rs, openai*.rs, any network call from this path; weakening or forking any M6 evidence gate; frozen profiles; release/raios-stage0.img; destructive disk ops. Attestation mismatch = STOP.
Constraints: request carries a hash ONLY — reject URLs, inline bytes, free-form text with typed denials; reuse M6 gates and the existing load path unchanged; action record before load; label results with real store provenance from M7D.
Definition of done: fmt + host tests green; kernel builds; profile extended with the positive chain + 4 denial needles; orchestrator runs recovery-lifeline / recovery / quick, then the FULL profile for milestone close.
Report format: files changed; verbatim outputs; exact needle strings; M6 gate call sites reused (file:line); risks; out-of-scope notes.
Stop conditions: any gate would need loosening for recovery context (NEVER — stop and report); store API mismatch.
```
6. **STOP-tripwires:** pressure to loosen any evidence gate "because it's
   recovery"; anything resembling network retrieval (that is the parked
   ota/registry world and REQUIRES a new ADR).

## M8 Close Checklist (orchestrator)
1. `-Profile recovery-lifeline` green; `-Profile recovery` 3,644/3,644
   unchanged; `-Profile quick` green; FULL profile green, report filename in
   the closing commit.
2. `scripts/scan-secrets.ps1` clean; serial logs contain no key/passphrase.
3. Docs closure: PROJECT_STATUS (per slice), ROADMAP cursor, OWNER_DASHBOARD
   plain-language entry ("raiOS now has a working emergency line: even when a
   program inside it breaks, you can see what broke and put the last working
   version back"), and this map's revalidation notes committed.
4. End-of-session checks from AGENTS.md (file sizes, fmt, gate check).

## Global STOP-Tripwires (all slices)
- Anything requiring a NEW ADR: pinned provider lifeline route, unparking
  ota/registry/fake-cloud, network artifact retrieval, new trust anchors.
- Any trust-model change: new signing authority, gate loosening, capability
  granted without the evidence chain.
- Any destructive disk operation, any write to boot slots/control records
  from lifeline code, any overwrite of `release/raios-stage0.img`.
- Any flip of a needle in the frozen `recovery` (3,644) profile.
- The wedge test showing the dispatcher does NOT survive a service fault.
- Attested-source changes not planned in the packet (re-sign via
  `target/descriptor-resign` only when a packet explicitly includes it; none
  of the M8 packets do by design).
- Full profile red at any checkpoint (Red Gate: repair only).

## Non-Goals
- No second transport (provider route) — OWNER DECISION 1 / new ADR.
- No mutating `recovery.rollback` through the lifeline in M8 (preview only,
  OWNER DECISION 2 default); pointer switching lives in M7C.
- No address-space isolation of the lifeline (post-M11 / core-generation
  work), no core-generation handoff, no Wi-Fi dependency, no new artifact
  intake of any kind.
