# raiOS Orchestrator Playbook (cheap-orchestrator edition)

Authored 2026-07-06 ahead of M7+ execution. Intended home: `docs/ORCHESTRATOR_PLAYBOOK.md`.
Companion files: `AGENTS.md` (the rules), the active milestone map in
`docs/plan-reviews/` (the design), this file (the procedure). A session run
from ONLY these three files plus `docs/DEBUGGING.md` lookups must be
acceptable.

## 1. Who this is for

This playbook is for a mid-tier orchestrator model (Codex-class or a smaller
Claude) executing pre-authored milestone maps (M7 Persistence Foundation and
later) without a strong orchestrator present.

The philosophy in one paragraph: **the maps contain the design; the harness
contains the truth; your job is DISPATCH + VERIFY + RECORD, never redesign.**
You do not invent slices, do not reorder them, do not "improve" packet
prompts, and do not interpret a red run as "probably fine". If the map
conflicts with reality (a file moved, a line number is wrong, a claimed
mechanism does not exist): STOP implementation, fix the map first (that is a
map-revalidation slice — every M7+ map defines Slice 0 for exactly this),
commit the map fix on its own, then continue. Where a real decision is
unavoidable, the map marks it `OWNER DECISION` — never make that call
yourself.

## 2. Session ritual (run this checklist in order, every session)

1. Read `docs/OWNER_DASHBOARD.md` — what the owner believes is true.
2. Read the Agent Handoff Cursor at the top of `docs/ROADMAP.md` — the
   active milestone and last verified state.
3. Read the active milestone map in `docs/plan-reviews/` (the cursor names
   it). Find the first slice not marked done.
4. Run `git status --short`. Expected: clean, or only files you know about
   from a previous interrupted session. **Unexpected dirty files or any
   unexpected ` D` (deleted) entry = stop-the-line: do not dispatch, report
   to the owner.** Never revert or sweep foreign changes.
5. Red Gate check: find the newest `release/vm-reports/shadow-*.json`, read
   its `result` field and `profile`. If the newest FULL-profile report says
   `"result": "failed"`, the Red Gate Rule applies (AGENTS.md, verbatim):
   *"While the full Shadow VM profile is red, the only permitted work is
   fixing it: no new slices, no new gates, no new schemas. Every commit
   message must name the passing report file for the verification tier that
   was run."* A red full gate turns the session into repair-only.
6. If this is the first session on a new milestone map: execute the map's
   Slice 0 (map revalidation) before anything else — re-check every
   file:line claim in the map against HEAD, update the map where reality
   diverged, commit the map update as its own commit.

## 3. Slice execution loop

Repeat per slice. Never run two slices' packets concurrently unless the map
explicitly says their write sets are disjoint and parallel-safe.

### 3.1 Take the next slice

Take the NEXT slice from the active map. Never skip, never reorder, never
merge slices without the owner saying so. Read the slice's capability
sentence, verification spec, fail-closed requirements, and STOP-tripwires
before dispatching.

### 3.2 Dispatch the embedded worker packet

Every M7+ map slice embeds a ready-to-paste Codex worker packet. Paste it
verbatim; only fill in placeholders the map marks as fill-in (e.g. a report
filename or hash from a previous slice). Dispatch per the `codex-worker`
skill:

```powershell
codex exec -s workspace-write -C C:\Users\admin\Documents\raios2 -o C:\Users\admin\Documents\raios2\target\worker-reports\<packet-id>.md @'
<packet prompt pasted verbatim from the map>
'@
```

Mechanics that matter (all verified incidents, not theory):

- **stdin gotcha:** dispatched from a non-TTY, codex waits forever on
  "Reading additional input from stdin...". Close stdin: `'' | codex exec ...`
  or pipe the packet file: `Get-Content packet.md -Raw | codex exec -s workspace-write -o out.md -`
- `-s workspace-write` always. **NEVER** use
  `--dangerously-bypass-approvals-and-sandbox`.
- `-o <file>` captures the worker's final report; read it after completion.
- Default effort: add `-c model_reasoning_effort=high` unless the map says
  the packet is risky (trust/rollback/boot) — then leave the user-config
  xhigh default.
- Usage limit: codex exits 1 with "You've hit your usage limit ... try again
  at <time>". Wait past the stated reset; do not retry-loop.
- `git status --short` BEFORE dispatch and again BEFORE commit.

### 3.3 Review the worker report

Check: did it stay inside the allowed write set (compare `git status --short`
against the packet), did it run the checks the packet required with real
pasted output, did it report scope creep instead of fixing it. A worker that
touched forbidden files = revert those files' changes are NOT accepted;
dispatch a fix via resume or restore from HEAD and report.

### 3.4 Run the slice's verification YOURSELF

**Never trust a worker-claimed green.** Precedent (M2 Collapse Batch 4,
2026-07-05): the worker's own field comparison claimed 1120/1120 fields
identical; the quick profile's golden needles caught 10 genuinely dropped
fields. Workers also cannot run the VM harness or commit from their sandbox.
Run the exact command the map slice specifies (section 4 has the matrix),
then read the report JSON yourself:

- `result` must be `"passed"`.
- `evidence_binding.predicate_count` / `predicate_passed_count` /
  `predicate_failed_count` — failed must be 0; compare the total against
  what the map slice says the new needle count should be.
- The map slice names new needles/evidence that MUST exist in this run.
  Grep the report's `predicates` array for them. A green run that lacks the
  slice's new needles is NOT verification of the slice.
- Confirm the `.sha256` sidecar exists next to the report.

### 3.5 Commit (orchestrator-only duty)

Workers cannot commit (sandbox denies `.git/index.lock`). Before committing:
`git status --short` again; foreign files never ride along; run
`scripts\scan-secrets.ps1` if anything near keys/images/ESP was touched.
Message format, matching recent history (`git log --oneline`):

```text
M7A-1: <the slice's capability sentence>

<1-3 lines of what/why if needed>
Verified: <profile> release/vm-reports/shadow-....json (<passed>/<total>).

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
```

The report filename in the commit message is mandatory (Red Gate Rule). No
slice ends with uncommitted source changes.

### 3.6 Record

- `docs/ROADMAP.md`: update the Agent Handoff Cursor ("Done in M7 so far"
  style — slice id, capability sentence, report file + counts, commit hash).
- `docs/PROJECT_STATUS.md`: detailed entry (same facts, more detail); any
  failures classified (section 5).
- `docs/OWNER_DASHBOARD.md`: every session. Plain language for a
  non-programmer German-speaking owner: short sentences, no jargon, no
  predicate counts in the lead — lead with what the system can now DO that
  it could not before. Keep the ~30-content-line cap.
- Mark the slice done in the milestone map (a one-line "done, commit X,
  report Y" annotation is enough).

### 3.7 End of session

Run the AGENTS.md End-Of-Session Checks and paste their output: file-size
check on touched `.rs` files (none above 5,000 lines without a documented
split plan), gate check (newest full-profile report `result: passed` and
newer than the last commit, OR an explicit Red Gate note naming the repair
work), and `cargo fmt --all -- --check`.

## 4. Verification matrix

Which tier (from the AGENTS.md Verification Budget Rule — the map's
per-slice spec always wins when it is stricter):

| Change class | Tier |
|---|---|
| docs-only | targeted diff check, no VM |
| refactor/UI/formatting | `cargo fmt --all -- --check` + smallest build/test |
| host-testable logic (raios-core) | `cargo test --locked -p raios-core` (<1s) + quick when kernel-facing |
| normal kernel slice | quick profile |
| trust, storage, rollback, recovery, authority, descriptor, boot, harness | focused profile that exercises the changed path — ALWAYS, no exceptions |
| pre-milestone-closure, release image handoff | FULL profile + secret scan |

Quick often, focused when the touched boundary is risky, full rarely
(checkpoints). Batch 3-5 small same-boundary non-authorizing hops before the
next focused run only while the prior smoke stays green.

Exact commands (from `docs/DEBUGGING.md`; run from the repo root):

```powershell
# quick (~5 min, ~486 needles as of M5)
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick

# FULL (~17 min, 7,825 needles as of M5) — use an outer tool timeout of at least 30 minutes
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile full -TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10

# focused examples (the map names which one a slice needs)
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile hello-rollback-dry-run -TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile module-audit-rollback -TimeoutSeconds 360 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile recovery -TimeoutSeconds 180
```

Build + package when a slice requires a fresh image before the smoke:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release
```

Reports land in `release/vm-reports/shadow-YYYYMMDD-HHmmss-<pid>.json` with a
`.sha256` sidecar (`<hash>  <path>`). Reading a report: `result`,
`evidence_binding` counts, `failures` (empty on pass), `predicates` (the
needle list), `serial_transport_failure` (null on healthy transport),
`qemu_process` teardown snapshots, `serial_log.path` (forensics; failed runs
keep their `%TEMP%\raios-shadow-*` dir, passed runs auto-clean it).

New focused profiles: when a map slice defines a NEW named profile, the
worker packet includes adding it to the `-Profile` `ValidateSet` in
`vm-harness/shadow-vm-smoke.ps1` (line ~12 at time of writing — verify at
execution) plus a `shadow-vm-smoke-profile-<name>.ps1` slice file. Follow the
map; do not design profiles yourself.

## 5. Failure protocol

Classify EVERY red run BEFORE any retry (AGENTS.md Failure Classification
Rule). Write into the failure classification log at the top of
`docs/PROJECT_STATUS.md`: report filename, failing predicate name(s), and a
one-line `host-transport` vs `guest-behavior` verdict with evidence.

Classification inputs, in order:

1. The report's `serial_transport_failure` field self-classifies transport
   deaths: `qemu_exited` (with exit code), `listener_missing_process_alive`,
   `connect_timeout_listener_present`.
2. `qemu_process.before_teardown.state` — was QEMU alive when the harness
   gave up?
3. The serial log tail (path in the report) — a cleanly completed last
   response points at transport; a panic/page-fault or mid-response cut
   points at the guest.

Then:

- **Classified transport flake** (e.g. oversized `audit.events` scrape
  timeout with QEMU alive and a clean serial tail — known class, precedents
  2026-07-03 and 2026-07-06): ONE retry, on a fresh serial port if the
  failure was connect-related. If the retry passes with no code change, log
  it; note that a failing-then-passing PREDICATE (not transport) is logged
  as a suspected intermittent guest bug, never closed as a host flake.
- **Real needle failure** (guest emitted wrong/missing output): this is a
  worker fix loop, not a retry. Resume the worker with the failing needle
  evidence: `codex exec -s workspace-write -C <repo> -o <report> resume
  <session-id> "<failing predicates + expected vs actual>"` (options before
  `resume`, prompt as final argument, no stdin `-`). Re-verify yourself
  after the fix.
- **NEVER edit needles to make a run green.** Needle changes are legitimate
  only when the map slice explicitly says outputs change, and then only with
  old-vs-new key-order proofs per the M2 discipline (scripted comparison
  showing 0 missing fields and identical ordering, kept as evidence).
- Red Gate Rule, verbatim (AGENTS.md): *"While the full Shadow VM profile is
  red, the only permitted work is fixing it: no new slices, no new gates, no
  new schemas. Every commit message must name the passing report file for
  the verification tier that was run. The final commit of a session requires
  a green full-profile report filename or an explicit Red Gate note naming
  the repair work done."*

## 6. Mechanical hazards checklist

- **Attestation re-sign:** `seed-kernel/build.rs` verifies P-256 signatures
  over descriptors and an ordered length-framed source SET (the
  `hello_service/` modules and other attested sources). Touching ANY file in
  that source set breaks the build until re-signed via the descriptor-resign
  tool under `target/descriptor-resign` (invocation documented in
  `docs/plan-reviews/m2-de-hello-ify-plan-2026-07-05.md`; verify the exact
  binary path at execution time). Map packets that touch attested sources
  include the re-sign step — do not skip it, and do not "fix" a signature
  failure by weakening build.rs.
- **CRLF:** signed source bytes are EOL-sensitive; a Windows checkout once
  broke the signed snapshots via CRLF conversion (fixed by `.gitattributes`
  forcing LF, commit `943a9a0`). If attestation fails right after a
  checkout/clone or on CI, suspect line endings before suspecting code.
- **Disk fill:** the harness keeps `%TEMP%\raios-shadow-*` dirs for FAILED
  runs (forensics). 356 leftovers once filled 23 GB mid-batch. Check
  periodically and delete old failed-run dirs after their classification is
  logged.
- **`release/raios-stage0.img` is the currently bootable artifact.** Never
  overwrite it unless the replacement image just booted green in QEMU. Never
  commit key-bearing local images (`raios-stage0-local-openai.img` etc.).
- `git status --short` before dispatch AND before commit; unexpected ` D`
  entries = stop-the-line (2026-06-10 incident: a source file vanished
  mid-wave); foreign dirty files are never swept into a slice commit
  (2026-06-10 incident: user files rode along in a wave commit).
- One worker per write set; two packets touching the same file are
  sequenced, never parallel. The serial TCP port is single-client: never run
  two harness instances or a manual serial client alongside a smoke.
- No new `raios.*.v0` schema as hand-rolled emit/hash code — record-model
  entries only (ADR 0005 mechanism-before-vocabulary). No fake persistence:
  everything stays `current_boot`-labeled until the real persistence slice
  that the map says makes it durable. e1000 only, no virtio. Bare metal
  only (ADR 0005).

## 7. STOP-AND-ASK-OWNER tripwires

Stop the session and ask (in plain language, with 2-3 options if a choice is
needed) before doing ANY of these:

1. Anything requiring a new ADR — especially unparking `ota/`, `registry/`,
   or `fake-cloud/` (parked by ADR 0005 §4; unparking REQUIRES a new ADR),
   vocabulary compaction, or any trust-model change (new signature
   authorities, chain validation policy, key handling).
2. Destructive disk or USB operations on real hardware (`write-stage0-usb`,
   repartitioning, anything that erases media).
3. Dependency version changes (pinned nightly-2024-10-15, wasmi =0.31.2,
   smoltcp 0.10, embedded-tls 0.17, Limine — all pinned deliberately).
4. Claiming a milestone closed — the owner confirms closure, always.
5. Spending real provider API keys (any smoke needing a live `OPENAI_API_KEY`
   beyond a fake smoke key).
6. Anything the active map marks `OWNER DECISION`.
7. Overwriting `release/raios-stage0.img` when the replacement has not
   booted green, or any push while the secret scan is unhappy.
8. A map/reality conflict too large for a mechanical map fix (the design
   itself looks wrong): stop, describe the conflict, propose nothing.

## 8. Milestone closure procedure

A milestone may be *proposed* closed only when ALL hold:

1. The milestone's capability sentence is verified TRUE by evidence you ran
   yourself (name the exact report files).
2. A final FULL profile is green on the final committed state, plus the
   secret scan.
3. Docs updated: ROADMAP cursor block in the established closure style
   ("**Mx ... closed <date>.** Capability sentence verified TRUE: ...
   Evidence: ... Slices ... (commits ...)"), PROJECT_STATUS detail entry,
   OWNER_DASHBOARD in plain language.
4. The owner confirms. Until then the cursor says "Mx closure proposed,
   awaiting owner", and the next milestone is NOT started.

## 9. Troubleshooting table (sourced from docs/DEBUGGING.md)

| If | Then |
|---|---|
| "Timed out connecting to QEMU serial TCP port ..." | Stale `qemu-system-x86_64` or occupied port: `Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue \| Stop-Process -Force`, rerun on a fresh `-SerialTcpPort`. |
| Host TCP write exception / truncated long command after earlier predicates passed | Classify first (section 5), then rerun with `-SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10 -TimeoutSeconds 300` on a fresh port. |
| Outer tool timeout (~10 min) during a full run | Not a failure by itself — full runs need a 30+ min outer timeout; inspect the generated report and the run's `serial.log` before concluding anything. |
| Report `serial_transport_failure: qemu_exited`, clean serial tail | Suspected silent guest reset (`-no-reboot` makes a guest reset look like a clean QEMU exit) or a concurrent `-StopExisting` launch killing every QEMU on the machine. Classify; check for parallel QEMU users. |
| Large `audit.events` / `memory.recent_events` scrape times out, QEMU alive | Known host-transport scrape class (2026-07-03, 2026-07-06 precedents). Classify, one retry. |
| Same predicate fails, then passes on retry with no code change | Log as suspected intermittent GUEST bug in PROJECT_STATUS — do not close as host flake (the 2026-07-05 stack overflow started life looking exactly like this). |
| Build fails on descriptor/source-set signature mismatch | An attested source changed: run the re-sign step (section 6). If nothing was intentionally changed, check CRLF/.gitattributes first. |
| CI red, local green | Suspect line endings or an unpinned toolchain difference; CI runs the pinned build + host tests + headless quick per push. |
| QEMU or packaging fails oddly, disk nearly full | Delete old `%TEMP%\raios-shadow-*` failed-run dirs (after classification is logged). |
| codex hangs on "Reading additional input from stdin..." | Close stdin: `'' \| codex exec ...` or pipe the packet content with trailing `-`. |
| codex exits 1 "usage limit" | Wait past the stated reset time, then re-dispatch. No retry loop. |
| Worker report claims a check passed but you cannot reproduce | Believe your run, not the report (M2 Batch 4 precedent). Resume the worker with your evidence. |
| Serial console live view needed | `Get-Content $env:TEMP\raios-stage0.serial.txt -Wait` (interactive runner); harness runs log to their own `%TEMP%\raios-shadow-*\serial.log`. |
