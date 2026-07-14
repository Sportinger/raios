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

Last updated: 2026-07-14.

**M11-9 DNSPARSE VERIFIED-CLOSED (2026-07-14) — the FOURTH real relocation, plus the
first hard memory bound for buffer guests.** The DNS query/A-response parser lives in
the no-dep crate `raios-dns-parse` (re-exported as `raios_core::dns_parse`; net.rs
873→721, live path behaviorally identical, 12 pinned-fixture host tests) AND runs as
the signed guest `svc.demo.dnsparse` (exactly 3 env imports) whose result the core
independently re-parses and cross-checks — decoded values AND exact output-record
sha256 — on happy/truncated/pointer-loop paths, with pre-instantiation import denial
(`wasm.dnsparse_probe`, typed record model). HARDENING: buffer-guest stores had
DEFAULT StoreLimits (wasmi default = UNLIMITED memory; only fuel was bounded) — now
2 MiB / 1 instance / 1 memory / 1 table / 64 elements with the limiter attached. The
spec said tables(0); the worker HALTED on its stop condition (httphead/certspki each
instantiate one funcref table), and the bound follows measurement — narrowed, never
widened. Commits 2c1967e / 1ee84af / 47265c7. EVIDENCE: m11-9-dnsparse
shadow-20260714-110049-6012.json; mandatory shared-runner regressions
m11-buffer-channel shadow-20260714-110257-27064.json, m11-6-certwindow
shadow-20260714-110359-23996.json, m11-7-httphead shadow-20260714-110517-26780.json,
m11-8-certspki shadow-20260714-110617-15500.json; block close FULL
shadow-20260714-110906-26004.json (2,685/0, same count as the P4 baseline) + RECOVERY
shadow-20260714-111420-25148.json (152). BOOKKEEPING: M11-8 = certspki (fdb1ce2 +
802ee6f, 2026-07-08, m11-8-certspki shadow-20260708-223318-28048.json) was never
recorded here — recorded now; that collision is why this slice is M11-9 while its
scope doc is `docs/plan-reviews/m11-8-next-parser-relocation-scope-2026-07-14.md`.
Next parser candidate per that scope: HTTP chunked-body + provider JSON extraction
(openai.rs, ~125 LOC, as ONE provider-body crate); TLS belongs to the beyond-env lane.

**OWNER DECISION (2026-07-14): W7 waits for Wasm net-imports — no native Stage-0
adapter.** The full W7 design (fixed-source pinned-HTTPS fetch, M12+ convergence,
RAM-only quarantine, denial matrix) is recorded in
`docs/plan-reviews/w7-quarantined-network-acquisition-scope-2026-07-14.md` (e696a62)
for the lane reopen. The active non-hardware lane is the ADR-0008 beyond-env import
architecture, scoped in `docs/plan-reviews/m11-beyond-env-net-imports-scope-2026-07-14.md`
(c176f60): pre-bound `net.*` (core owns endpoint/pin/lease), opaque-session `crypto.*`
(keys never in guest memory), M12-convergent `acquire.*`; TLS/HTTP state machines in
Wasm as evidence only. Slices 1-7 are grants-nothing (`policy_allows_beyond_env` stays
false EVERYWHERE); slice 8 is the explicit owner-approved arming diff for exactly
`svc.net.acquire.w7`. Critical proof obligation: F12 kill + singleton-TCP-lease release
while the peer is silent, else recut to non-blocking imports — an unkillable kernel
stall is a stop condition, not a ship. Owner detail decisions 1-4 proceed on the
recorded recommendations (veto open), 5 after slice-2 evidence, 6 at arming, 7 deferred.
**NET-1 is DONE** (m11-wasm-import-grant shadow-20260714-114527-24812.json, 560 host
tests; it also caught and fixed a real honesty-report same-source divergence — see
PROJECT_STATUS). **Owner decision 5 RESOLVED BY EVIDENCE and the lane RECUT to wasmi
resumable execution** (NET-2 halted on its stop condition: F12 is polled from the main
loop, a blocking Wasm call blocks it — addendum
`docs/plan-reviews/m11-net-imports-resumable-execution-addendum-2026-07-14.md` replaces
slices 2-4). **NET-2R is DONE and verified**: m11-beyond-env-lifecycle
shadow-20260714-123624-28556.json 183/183 — real wasmi suspension with
recovery/serial responsive, physical F12 kill via QEMU monitor within the 250 ms bound,
exactly-once teardown across eight exit paths, second-run-after-kill release proof.
**NET-3 is DONE and verified**: m11-net-imports (-Network)
shadow-20260714-130148-25792.json 162/162 first-run — generation-checked singleton TCP
lease guards every entry point, native OpenAI claims/releases it centrally (the old
unowned submit-time tcp_abort is gone), one-step non-blocking ops ready for the NET-4
pump; tls_io's synchronous polling is retained as NAMED DEBT, not W7 proof. **NET-4 is DONE**: m11-net-imports (-Network) shadow-20260714-133923-17376.json 183/183 —
the four net.* shims are real suspension points driven by the pump over the NET-3 lease,
proven with REAL bytes (DNS-over-TCP to slirp 10.0.2.3:53, answer read by the guest after
resume), silent-peer timeout, F12-kill-before-timeout, denial-before-instantiation for
net.*-requesting signed modules, plus the central wasm_execution_busy reentrancy gate.
**THE NET FOUNDATION BLOCK (slices 1-4) IS VERIFIED-CLOSED**: quick -Network
shadow-20260714-135736-9236.json + m8-lifeline shadow-20260714-135959-18000.json + FULL
shadow-20260714-140106-6508.json (2,685/0) + RECOVERY shadow-20260714-140417-27588.json
(152). The close also exposed and fixed THREE stale quick-profile expectations left by P4
slices that rewrote the profile from its manifest and never ran it — kernel right, needles
wrong, each fix made stricter; the resulting process rule is in PROJECT_STATUS and
docs/DEBUGGING.md. **The mandatory wasm_runtime.rs split is DONE** (pure move: 4,597 lines -> a 7-file module,
largest 981; no caller outside changed, 107/107 external refs, 79/79 pub(crate) surface;
vm-harness untouched, so the green m11-buffer-channel / m11-9-dnsparse /
m11-beyond-env-lifecycle / m11-net-imports runs ARE the equivalence proof). **NET-5 + NET-5B are DONE and verified**: m11-crypto-imports (-Network)
shadow-20260714-154639-20396.json 187/187 — the eight fixed crypto.* imports run over the
pinned primitives with keys in a core-owned opaque session (only public bytes ever reach
guest memory), driven end-to-end by a labeled fixture whose linker alone receives them,
with 32 typed denials proven in-guest. The self-bless guard holds: application traffic
keys require six CORE-recorded facts (state, transcript, CertificateVerify observed,
math_valid, pin_match, server Finished valid) — a guest can trigger calls but cannot set
one of them. Two orchestrator corrections are recorded in PROJECT_STATUS: p256's `pkcs8`
feature was dragging der/spki/base64ct+PEM into the permanent core (replaced by
raios-x509-spki::parse_p256_spki — our own no-dep parser; p256 is now math-only), and the
eight shims had been DEAD CODE (linked nowhere, so the kernel glue would have first run at
arming). **NET-6 is DONE and verified**: m12-distribution-provenance shadow-20260714-161948-11204.json
236/236 — serial delivery and the labeled acquire.* fixture finalize the SAME candidate
sha256 and a BYTE-IDENTICAL receipt through ONE shared M12 seam (no second verifier, no
guest path into the intake sink), every failure preserves the prior candidate, and acquire.*
still denies before instantiation. **NET-7 is DONE and verified**: m11-acquisition-service shadow-20260714-170818-14076.json
161/161 — the signed svc.net.acquire.w7 guest (the first real beyond-env-using program:
TLS+SPKI+HTTP+chunk driver over the 16-import surface) exists, is independently re-parsed
and import-set-checked by the kernel, and its grant DENIES before instantiation
(import_beyond_env_not_owner_authorized). Pure logic is host-tested against a mock ABI with
7 malformed-TLS + 10 malformed-HTTP fail-closed vectors. A correctness fix landed before
arming: the kernel's independent import-set check was positional and the compiler emits
imports in first-use order — now SET equality (still rejects extra/missing/duplicate).
**NET-8 IS ARMED AND COMMITTED (owner decision 2026-07-14), live download NOT yet
demonstrated.** svc.net.acquire.w7 is owner-armed via one gated site (two literal approval
constants + local.qemu.w7); m11-acquisition-service proves the armed reality (armed / mismatch
still denies / grants nothing durable), and the shared-file regressions (m11-net-imports /
crypto-imports / beyond-env-lifecycle) pass on the NET-8 kernel. The live end-to-end fetch is
NOT proven: two blockers fixed (QEMU guestfwd-to-10.0.2.100 bridge; the encode_w7_request
byte-framing bug) and one remaining (the W7 invocation never reaches tcp_open — a dispatch
bug to instrument). Also new: **AGENTS.md now separates Dev-Phase from Operating rules** —
arming a bounded RAM-only VM capability before its live demo is allowed if honestly labeled;
four rules stay strict (no faked evidence, no secrets, run rewritten checks, no
durable/hardware authority without evidence).
**NEXT DELIBERATE BLOCK — finish the loop (owner-requested):** (1) fix the W7 dispatch bug
(instrumented) so the download actually runs; (2) prove the live download, ideally against a
REAL internet server (raiOS already reaches the real internet in QEMU — this sidesteps the
Windows-QEMU host-loopback friction); (3) wire the inert downloaded candidate into the
EXISTING M6 run + W6 durable-install/rollback machinery so a downloaded program can run and
install — reusing proven machinery, not weakening the quarantine. Then NET-9 the full W7
denial matrix. The hardware cursor is unchanged and Surface-gated (G7 read-only stick
preflight, WiFi association/PORT_RELEASE/RX-TX/DHCP proof, ownerkey TPM capture).

**P4 EVIDENCE-VOCABULARY-V1 IS CLOSED — BOTH HALVES (2026-07-13 ~23:15).** I once called
it closed after only the envelope; that was wrong and is now actually true.
1. THE ENVELOPE: nine families, one `raios.evidence_response.v1` (below).
2. THE EMISSION DRIVER (design section 1): *"No family may call raw(), json_str(), or
   raw_bool() inside a JSON object after conversion."* ~4,400 hand-written JSON sites are
   now typed `record::Value` trees (memory 2,019→4; hello emitters 1,413→12; load-gate
   render 625→2; provider 255→4; hello runtime 112→0). Every survivor is transport framing
   OUTSIDE the record tree. The `EMIT_BINDING_FACTS_OBJECT` splice flag is deleted.
EVIDENCE: FULL shadow-20260713-230752-3548.json (2,685/0) + RECOVERY
shadow-20260713-231312-25044.json + PERSISTENCE shadow-20260713-231408-29132.json; 548 core
tests. Proven byte-identical: five conversions, five green runs, vm-harness NEVER touched.
LINE COUNT: 176,331 -> 166,094 (-10,237). The plan's center was 139,281 — still missed, and
the P4 block in `docs/PROJECT_STATUS.md` says exactly why. The next reduction must come from
measured ownership moves, NOT another vocabulary layer.

**P4 ENVELOPE (done).** All nine families answer in
one `raios.evidence_response.v1` envelope; an OBSERVED decision has no
grants/effects keys at all, only a DENIAL renders them. Evidence: FULL
shadow-20260713-200540-17832.json (2,685 predicates, 0 failed) + RECOVERY
shadow-20260713-201215-19892.json + PERSISTENCE shadow-20260713-201342-22980.json;
548 core tests. Read the P4 block in `docs/PROJECT_STATUS.md` BEFORE planning the
next reduction — it records (a) the retraction of a mis-measured line count and the
real one (176,331 -> 170,293, i.e. -6,036, NOT the planned -37,050), (b) the NAMED
CARVE-OUTS deliberately still on the old vocabulary (project, distribution/registry,
Wasm/program, genesis-ui, plus materialization/scratch and memory-record append)
because converting them would have forced a lie about authority, and (c) the
mandatory reserved-key pre-flight grep. The next reduction must come from measured
ownership moves, NOT another vocabulary layer.

Current milestone: **M10 + M11 IN PROGRESS** — both have their grants-nothing
mechanism foundations committed; the substantive remainder of each is
owner-gated (see below). **M6 Promotion Loop, M7 Persistence, M8 Recovery
Lifeline, and M9 Durable Memory & Context Broker v1 (ADR 0004 Phase D) are
COMPLETE.**

**Owner-directed Genesis execution lane opened (I0/G0, 2026-07-10):**
`docs/plan-reviews/genesis-shell-execution-plan-2026-07-10.md` and accepted
ADR 0011 now fix the core-owned Genesis/recovery surface plus the exact bounded
personal-shell Wasm ABI. This is a contract, not a runtime claim: renderer,
imports, proof service, Vault, structured storage, and USB handoff remain
unverified until their named slices and reports land. Disjoint foundation work
may proceed while the next hardware task remains positive Surface association,
`PORT_RELEASE`, RX/TX, and DHCP proof; no provider-access or durable-secret
claim is opened by I0.

**AI-authored current-boot program loop complete (2026-07-12):** Genesis
`/build` now sends a real evidence-bound `program.ask`, accepts only bounded
typed `RAIOS_UI_SPEC_V1`, compiles locally to canonical RUIP, retains the draft
inert by hash, and requires physical approval before the signed six-import
`svc.user.shell` Wasm engine runs it. Live pinned-SPKI OpenAI authoring and
same-boot activation are proven; focused key-free Genesis regression
`shadow-20260712-025218-6208.json` passes 252/252. This opens no durable install,
external arbitrary Wasm, file/network/secret access, promotion or rollback
authority. The next product slice here is durable program installation only
after its persistence/evidence/rollback gates are designed; the active hardware
cursor remains the read-only G7 identity tripwire.

**Secure project workspace W1 complete (2026-07-12):** a bounded local
multi-file source project now commits as immutable content-addressed blobs plus
deterministic tree/revision evidence to the exact disposable QEMU structured
store and replays byte-identically after reboot. Focused report
`shadow-20260712-124220-8296.json` passes 76/76, including invalid paths, case
collision, wrong hash, quota denial, manifest-last visibility and explicit
no-build/no-install/no-execute posture. `/revise <feedback>` also reuses the
existing current-boot provider/spec/runtime path with parent/root lineage while
preserving the prior valid draft on failure. W2 project query/edit and W3
dependency quarantine, W4 reproducible workstation build and W5 tested
current-boot execution are now complete as recorded below. W6 durable
install/autoload/rollback is also complete. The active product cursor is W7
quarantined network acquisition; automatic build/install and broader mutation
remain closed. Hardware
I5/G7 remains a separate read-only identity tripwire.

**W2a project query complete (2026-07-12):** exact project/path byte-range
reads and locator-only text search now replay and rehash the W1 revision on
every call. `shadow-20260712-125335-27844.json` passes 136/136 across 53 commands
and both boots, including wrong-project/path, range/query/limit denials and
explicit no-write/no-export/no-build/no-install/no-execute posture. The W2b
result below closes the remaining overlay edit/diff/commit/discard slice.

**W2b project overlay editing complete (2026-07-12):** an agent can bind a
RAM-only overlay to the exact verified latest revision, add/replace/delete
files, inspect the deterministic old/new hash-bound diff, discard it, or commit
one immutable `agent_overlay_commit` child. Stale, malformed, hash-mismatched,
no-op, invalid-delete and case-collision paths leave the stored base unchanged.
`shadow-20260712-130758-7668.json` passes 304/304 across 114 commands and three
boots; the exact child replayed byte-identically after reboot. W2 is complete.
Direct cloud-provider tool use, build, install, load and execution remain closed.

**W3 dependency quarantine complete (2026-07-12):** a user can local-serial
import chunked exact-version dependency package sources, bind an owner-declared
origin locator and license evidence plus the exact `Cargo.lock` blob to one
immutable project revision, and inspect exact file/chunk/tree/bundle hashes
after reboot. The verified package included `LICENSE`, detected-but-never-run
`build.rs`, and a
greater-than-24-KiB multi-chunk `src/lib.rs`; build-script execution and every
network/export/compiler/build/install/load/execute authority stayed false.
Idempotent re-import verified existing chunks/manifest without writes, and the
source revision remained byte-identical. Focused report
`shadow-20260712-135131-25884.json` passes 600/600 across 214 commands, three
boots and 917166 ms. The host's 900-second wait expired 17 seconds before the
same child produced this green report, so the recorded timeout was host
wall-clock transport, not guest failure; no retry or code change occurred. W3
does not claim Cargo semantic parsing, verified origin/license truth, archive
extraction or network fetch. W4 is complete below.

**W4 reproducible offline workstation build complete (2026-07-12):** the owner
workstation exact-read one reviewed immutable Rust revision plus one safe
quarantined local path dependency, then built it twice `--frozen` and `--offline`
under exact flags/environment and a pinned, measured toolchain. Both outputs were
byte-identical; the validated inert current-boot candidate is
`05854c56665a9fee9990712126e1f19269059375cb37fcdccacaa990ab3d30fb`.
`shadow-20260712-145618-13408.json` passes 248/248 across 108 commands, one boot
and 313118 ms. The receipt is `builder_attested_not_local_rebuild`,
`independently_verified=false`, and grants no install/load/execute/promotion or
persistence authority; this is not an owner-sealed toolchain or independent local
rebuild claim. Contract, build-script, read, run/output/candidate and stale-receipt
negatives failed closed.

**W5 tested current-boot application complete (2026-07-12):** the exact W4
candidate is reparsed locally, accepted only with an observed zero-import
surface, previewed by core Genesis and run only after physical pointer approval.
The real fixture returned 42 under 250000 fuel, 4 MiB memory, one instance/one
memory/zero tables; stale/tampered/replay/serial-approval paths denied, and F12
dropped the RAM service/candidate while Recovery stayed available.
`shadow-20260712-153736-17972.json` passes 276/276 across 112 commands, one boot
and 553863 ms. No install, persistence, promotion, native load or broader app
capability opened at W5.

**W6 durable project install and rollback complete (2026-07-12):** the exact
healthy W5 candidate and canonical receipt can now be signed, previewed in core
Genesis, durably committed only after a second physical pointer approval, and
replayed/autoloaded on a success-marked Normal boot. Probation records an attempt
before byte intake and marks last-good only after healthy execution. Focused
report `shadow-20260712-171300-16808.json` passes 403/403 across 156 commands and
four boots, including a real v2 ARTSTOR byte corruption, persisted rollback to
v1, physical uninstall, no boot-4 autoload and unchanged source revisions. W6
close is backed by full `shadow-20260712-173148-25720.json` at 7870/7870 and
recovery `shadow-20260712-174432-7724.json` at 3677/3677. Trust remains
`dev_key_not_owner_sealed`, state is explicitly stateless, and physical-stick
persistence is not claimed. The active product cursor is W7 quarantined network
acquisition; downloaded bytes must remain inert until separately reviewed.

**M13/Secret Vault contract accepted (C0/G5.0, 2026-07-10):** ADR 0012
fixes the dedicated internal-partition boundary, crash-consistent structured
store, exact cryptographic pins, RR1 recovery wrapper, TPM evidence gate, and
two-consumer broker. C1 has now proven the isolated QEMU store mechanism and the
provider slice has proven RR1 recovery unlock there; physical-target support,
production durable-secret persistence, TPM auto-unlock, and Vault-VMK sealing
remain unproven. ADR 0007 owner sealing is unaffected.

**Current Genesis/Vault cursor:** I4/G6 and the current-boot AI program loop are
complete for QEMU. The owner reports the USB stick has been found, but it was not
enumerated or touched in the program session. I5/G7 therefore resumes only at a
read-only candidate/identity/fingerprint preflight; never assume the former Disk 2
number and never recreate a missing fingerprint. The exact QEMU G6 release
candidate and its final six profiles remain green; physical F12 outside the
personal shell opens core Recovery. The historical last probe found no Disk 2 and
no G0 fingerprint, so no physical write is permitted until both are re-established
read-only. Physical boot/persistence, TPM auto-unlock and live Surface network remain
closed. Live pinned provider access is proven only for the current-boot UI-program
slice; broader provider authority remains closed.

**I4/G6 final evidence (2026-07-11):** structured-store
`shadow-20260711-024108-8004.json` 13/13, secret-vault
`shadow-20260711-024147-24008.json` 155/155, genesis-ui
`shadow-20260711-024805-24880.json` 213/213, recovery
`shadow-20260711-024914-26232.json` 3677/3677, quick `-Network`
`shadow-20260711-025422-12600.json` 544/544 with e1000+DHCP, and final full
`shadow-20260711-025731-23460.json` 7870/7870 all pass. The final image/report base is
`8f3bc250...51b93ba`; the built, ESP-copy and FAT-contained kernel is
`e617d2de...ddabace`. Ten accepted 1280x800 originals have no pure-black pixels and
the same left Core secure strip. The no-data-disk full run remains honestly
Core-Policy-denied; it is not owner-verified evidence.

**C1/G5.1 structured store verified (2026-07-10):** the focused
`shadow-20260710-032738-34812.json` profile passed 9/9 after an isolated
16-MiB QEMU fixture was admitted by exact BDF/port/device/GPT identity,
blank-formatted, dual-superblock readback-checked, append/flushed, and replayed
after a second boot. This is deliberately QEMU fixture only; no physical target
or secret authority is open.

**C4/I3 unarmed Vault composition foundation (2026-07-10):** commits `95b7bf4`,
`d27c96d`, and `f90e7db` add replayed/readback-verified recovery-keyring restore,
a typed ciphertext-only Vault record codec, and complete-history-only nonce
reconstruction. The `0920346` C1 extraction lets the composition root name the
already bounded disposable QEMU port; the focused regression
`shadow-20260710-040559-24348.json` passed 9/9 with zero failures. No Vault set,
unlock/decrypt, plaintext use, WiFi/provider use, audit, physical-target, or
durable-secret authority is armed. The post-review Broker foundation now consumes
only opaque complete-history, mutation and use evidence; its one-use outputs are the
bounded NXP WPA2 command and exact OpenAI header, never plaintext access. Core tests
pass 396/396 and focused regression `shadow-20260710-132631-20352.json` passes 9/9.
The exact QEMU store now also delivers its opaque identity-revalidated complete
history into the Broker on both boots; `shadow-20260710-133203-24112.json` passes
11/11. The new distinct owner software-pinned Core Policy now verifies the complete
Limine executable and exact Normal/Probation BOOTCTL slot/generation;
`shadow-20260710-145039-13864.json` passes 5/5. It is explicitly not Secure Boot,
TPM measurement, deterministic ESP A/B selection, or anti-rollback. The Broker now
retains that verified identity beside the complete replay on both boots, and rejects
caller-supplied/replacement policy identity; focused
`shadow-20260710-150107-28328.json` passes 13/13. The first armed I3 vertical slice is
green: `shadow-20260710-160920-28360.json` (29/29) proves one-time RR1 display, physical
re-entry, exact QEMU-store wrapper commit/readback, independent reboot/replay and RR1
Broker unlock with no RR1 in either serial log or report. The provider and WiFi halves
of I3/G5.4 are now green together in `shadow-20260710-192431-4220.json` (56/56):
physical Genesis input can save encrypted OpenAI and exact-SSID/BSSID-bound WPA2
credentials on the disposable QEMU C1 store; after reboot/replay and RR1 unlock, each
reaches only its exact one-use consumer after a durable `local_only` pre-use audit.
Wrong-BSSID and auditless WiFi use deny. That report does not prove a live provider
request, association/link/`PORT_RELEASE`/DHCP, physical persistence or TPM auto-unlock.
The physical two-confirmation forget path is now green in
`shadow-20260710-195715-22816.json` (78/78): both fixed slots append/readback/replay
version-2 tombstones across a third boot and deny as `secret_forgotten` before audit or
consumer use. **I3/G5.4 SAFE explicit reconnect is verified** by
`shadow-20260710-204801-23168.json` (95/95): Normal/Probation may exact-match the saved
WiFi after unlock; an owner-signed last-good SAFE boot emits no provider/WiFi use until
one physical Genesis action, then exactly one metadata-only `local_only`
`safe_recovery` audit and contained WiFi consumer. General SAFE durable-write denial and
the provider path are unchanged; a move-only token reaches only `SupplicantPmk`, and
`AlreadyReady` now requires the exact retained target. **G5.5a torn/power-cut recovery
is verified** by `shadow-20260711-001242-11280.json` (118/118): after an exact-C1 WiFi
version-2 PREPARE plus TOMBSTONE with no COMMIT and a hard QEMU stop, reboot/replay keeps
the committed version 1 usable after unlock and preserves the core Vault handle. The
report itself and default release/recursive ESP pass the dynamic sentinel scan.
**G5.5b corruption/isolation continuity is verified** by
`shadow-20260711-010926-21860.json` (152/152): copied foreign-GUID media denies before
fixture acceptance; a copied CRC-corrupt log becomes `StoreChainLocked`; both expose a
physical visible RAM-only denial and grant no Vault authority. Copied tag/AAD/binding/
nonce and stale-policy/context/corrupt-wrapper cases deny. A physical Escape permits
the bounded typed personal-trap proof, recovery remains responsive, and physical Vault
reopen preserves the unlocked handle without extra provider/WiFi use. The green recovery
and full reports named in the cursor close G5 only for disposable-QEMU evidence;
physical persistence and TPM auto-unlock remain unproven.

**Genesis execution progress (A1/C3, 2026-07-10):** the normal release image
now starts in the core-owned Genesis shell (Conversation, Context, Composer,
secure strip, AI/WiFi setup); the legacy renderer was deleted, with serial and
guided WiFi behavior preserved through the shared console/action adapter. The
no-secret 1280x800 capture is `target/captures/genesis-shell-a1.png`. The TPM
codec/CRB-TIS transport and RR1 recovery-wrapper foundation are compiled and
host-tested but grant no auto-unlock, secret broker, or durable-secret claim.
The C1 QEMU-only proof is green; physical targets remain denied. A2/G2 trusted
Context, overlays and recovery was the prerequisite for the broker join.

**Genesis trusted interaction verified (A2/G2, 2026-07-10):** Genesis and the
agent protocol now share typed current-boot problem facts; its secure strip opens
a cached/redacted Recovery projection, and Recovery restart/disable controls use
the same typed Lifeline executor as serial. Existing masked RAM-only provider/WiFi
setup remains reachable from Genesis. The focused `genesis-ui` report
`shadow-20260710-034302-30252.json` passed 181/181, release build/package passed,
and `target/captures/genesis-shell-a2.png` was inspected. At A2 the zeroizing secure
overlay was only a bounded input foundation; I3 now uses that same core-owned ingress
for the exact provider/WiFi Vault paths described above, without exposing plaintext.
Recovery hash-load and rollback authority remain closed. B1's
display-list/import-grant foundation is already present. I2/G3 passed its required
Sol review. The owner accepted ADR 0013 after the ignored local signer was lost;
its tracked `descriptor-resign` tool is host-tested for exact raw-byte P-256
sign/verify and altered-byte rejection. It supplies only explicit
`dev_key_not_owner_sealed` descriptor provenance and does not unpark OTA or grant
runtime authority. **I2/G3 is now verified**: the signed `svc.user.shell` proof
runs in a fresh metered Wasm instance through only its six listed `ui.*` imports;
focused `shadow-20260710-121953-4964.json` passed. It is current-boot test
infrastructure only: no general loader, external artifact intake, secret, network,
provider, recovery, persistence, or mutation authority is open. The Genesis
AB/G4 join is now verified by `shadow-20260710-124838-24564.json` (206/206): the
non-default proof enters the clipped core-owned surface, accepts sanitized input,
leaves through core-only F12, falls back after trap/fuel, and never changes the
secure-strip pixels. Its current-boot inventory row exists only while active. The
I3/G5.4 provider, contained WiFi-use, physical forget and SAFE explicit-reconnect paths
are verified by the focused reports above. G5.5a power-cut recovery, G5.5b copied-
corruption/personal-trap continuity, recovery and full are green; disposable-QEMU G5
and QEMU release-candidate G6 are closed, and the cursor is the read-only G7 tripwire.

**M11 Kernel Slimming progress (all grants-nothing / strictly-more-restrictive,
committed):** M11-1 kernel internet-parsing SURFACE baseline (the measurably-
shrinks reference, ~9640 candidate LOC); ADR 0008 the per-service-import-grant
architecture PROPOSAL (owner decision); M11-2 the fail-closed import-grant
EVALUATOR (raios-core; known set env.log/env.counter_get; beyond-env owner-gated);
M11-3 ENFORCEMENT — each Wasm instance's Linker is built from ONLY the
evaluator-authorized imports, verified ⊆ module.imports() before instantiation,
strictly more restrictive (review SOUND, profile m11-wasm-import-grant 185/185);
M11-3a DURABLE per-service import-grant AUDIT (capability_grant/local_only/
deduped, memory-durable 154/154). **ADR 0008 ACCEPTED by the owner (2026-07-08):
Option A (exact per-service import list + evaluator + per-instance Linker) PLUS
trust-shape Option 2 (the TLS/parser verifier moves INTO the Wasm service; the
permanent core keeps trust-label authority + provider request/export authorization
+ API-key custody — the service produces evidence, never blesses itself).**
**M11 relocation now BUILDING slice by slice:** M11-5a byte-buffer data-channel
MECHANISM (3 DEFAULT-DENY env imports env.input_len/input_read/output_write +
runtime plumbing; grants nothing — echo/hello/granted_candidate byte-identical;
m11-wasm-import-grant 197/197); M11-5b the FIRST signed service to USE it —
svc.demo.bufecho reads host-staged bytes and writes them straight back through
ONLY those 3 imports, dev-key-signed (scalar 1, dev_key_not_owner_sealed, build.rs
hard-gates the signature + asserts the honest tier), output surfaced only as
len+sha256, import-grant audit honestly RAM-only (boot_control_safe_mode) —
m11-buffer-channel 196/196, MAX-EFFORT 4-lens adversarial review SHIP. **M11-6 the
FIRST REAL kernel-code relocation (DONE):** M11-6a carved the pure DER X.509
validity-window parser into a standalone no-dep crate `raios-x509-time` (grants
nothing; raios-core re-exports it so every call site is byte-identical; sha2 is
dev-dep-only so the guest never builds crypto — raios-core itself can't reach
wasm32); M11-6b runs that SAME parser inside a dev-key-signed sandboxed Wasm
service `svc.demo.certwindow` (imports ONLY the 3 byte-buffer fns) on the real
M10C-4 cert, and the kernel INDEPENDENTLY re-parses the same cert and cross-checks
the guest's 18-byte result on BOTH the happy AND truncated-error paths — Option 2:
guest = evidence, core = authority; deterministic sandboxed re-execution, NOT a
diverse reimpl; every trust/authorize/durable flag hard-false, policy_allows_
beyond_env still false. m11-6-certwindow 193/193, m11-buffer-channel regression
196/196 (echo/bufecho byte-identical after the additive raw_captured_output field +
threaded per-call fuel_budget), MAX-EFFORT 4-lens review SHIP (incl. independent
re-verification of both scalar-1 signatures). NEXT relocation slices: more internet-
facing parsers, then beyond-env host imports (net/tls/crypto/time/secret) + per-
service secret custody toward the full TLS verifier in Wasm; policy_allows_beyond_env
stays false until an explicit later slice arms it. **M11 first-real-relocation
milestone VERIFIED-CLOSED (2026-07-08): full 8205/8205, recovery byte-identical
3870/3870.** **M11-7 (2026-07-08): SECOND real relocation — the kernel's HTTP-
response header parsers moved to the no-dep crate raios-http-parse + a signed
svc.demo.httphead guest cross-checked by the core (httphead 194/194); M11-7c fixed
a real latent bug it surfaced (parse_content_length always returned None →
non-chunked Content-Length completion never fired). M11-7 block VERIFIED-CLOSED:
full 8205/8205, recovery 3870/3870.** Process changes (owner 2026-07-08): the
per-slice max-effort review is DROPPED (host DoD + own diff read + focused profile
+ secret scan instead); scoping + implementation both run as CODEX workers, not
Claude workflows. OWNER SIDE TRACK (parallel): the Surface Pro 4 Marvell 88W8897
WiFi driver — scoping, pure firmware sequencer, register-write plan, triggered
BAR2/DMA firmware bring-up, GET_HW_SPEC mailbox probe, legacy live-scan response
parsing, guided setup, firmware descriptor registration, association, event/RX/TX
rings, and the smoltcp WiFi backend are committed. The earlier pre-registration
RX-PFU experiment that froze the Surface remains historical; the current rings
are only published after their descriptor-registration and MAC-control responses
validate. The Windows USB writer can now raw-write the existing GPT
`SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA` persistence layout to a stick with valid
empty RECLOG, and the kernel can now read that same layout through xHCI USB
Mass Storage/BOT and report `MSC SEED`; the first strictly scoped USB
`WRITE(10)` path is now in place for bare-metal diagnostics: after GPT +
`RAIOS_DATA_SB_V0` validate, raiOS appends local-only diagnostic frames into
`SEED_DATA/RECLOG` for boot and hub-mouse recovery events, readback/reparses
them, and reports `MSC LOG`. Real Disk 2 evidence proved endpoint-only rearm
did not restore reports, and later root-cause frames showed the hub port and
xHCI endpoint still healthy (`m_port=259 m_chg=0 m_ep=1`) when the fixed-time
mouse freeze occurred after movement. The current image stops periodic hub
child-port EP0 control polling after the hub mouse has produced reports. An
earlier diagnostic image also parked WiFi post-ready event-ring/HW_SPEC
auto-probes while isolating that input loss. The current path supersedes that
experiment with registered rings after stable guided input was proven and
corrects the pre-ready interrupt quarantine against the Linux mwifiex
reference: it disables `PCIE_HOST_INT_MASK` at `0xC34`, programs the status
mask, and clears pending status with write-zero-to-clear polarity before
disabling WiFi DMA/INTx and writing `DRV_READY`. It now keeps BAR memory
decoding available only for a bounded `FW_STATUS` poll and claims firmware-ready
solely on `0xFEDCBA00`. The owner confirmed the real Surface reaches ready,
completes one bounded `GET_HW_SPEC` bus-master window, and keeps hub input
responsive; a later owner test proved real `SCAN_EXT` command completion but
also reproduced the hub-mouse interrupt stall. RECLOG stayed valid 66/66 and
showed no new reports despite endpoint rearm plus a completed hub-port reset;
physical unplug/replug recovered input. Because upstream mwifiex returns
extended-scan BSS data through an event but legacy scan BSS data directly in the
command response, the refreshed image uses bounded legacy scan `0x0006`,
strictly parses its BSS descriptors, and feeds valid networks into the existing
`[LIVE]` list. The owner proved real SSIDs and the complete guided path through
RAM-only password entry with stable keyboard/mouse input. The current image now
continues that exact path through real firmware ring registration, open or
WPA2-PSK/CCMP association, WPA2 `PORT_RELEASE`, PFU Ethernet RX/TX, and smoltcp
DHCP. Link is granted only after the required positive firmware evidence; all
unsupported security and incomplete/malformed responses fail closed. The next
WiFi step is the positive Disk 2 bare-metal association/PORT_RELEASE/RX-TX/DHCP
proof. Provider access and durable WiFi-secret claims remain denied until it.

**M12+ opener + honesty capstone (committed, grants nothing):** M12-1 external-
acquisition HONESTY evaluator (download = candidate intake NEVER install; a
distribution signature is provenance NOT load-worthiness); ADR 0009 the external-
artifact-distribution architecture PROPOSAL (owner decision; parked ota/registry/
fake-cloud NOT unparked). `system.honesty_report` — one read-only manifest of
raiOS's complete honest posture (provider trust / time / cert-time / export /
Wasm import enforcement / external acquisition / owner-seal), computed from the
committed evaluators, with `no_dishonest_overclaim`, owner_sealed:false,
dev_key_not_owner_sealed. Plus M10C-3 (raios-core X.509 validity DER parser) +
M10C-4 (live kernel cert-time check on a REAL embedded certificate) — the M10
cert-time chain is now complete end to end (clock -> parser -> comparator -> live
real-cert check), unverified-basis, grants nothing. **M10+M11+M12 mechanism block
VERIFIED-CLOSED: full 8205/8205, recovery byte-identical 3870/3870.**

**THE OWNER-GATED FRONTIER (updated 2026-07-08):** (1) ADR 0008 — **ACCEPTED
(A+2); BUILDING — 2 real relocations DONE + closed** (M11-6 X.509 parser + M11-7
HTTP-response parsers now run in signed cross-checked Wasm sandboxes; both blocks
VERIFIED-CLOSED full 8205/recovery 3870). (2) ADR 0009 — **ACCEPTED (Option A);
BUILDING — first slice DONE:** M12+ Phase A the raios-core provenance-verify
primitive (separate scalar-2 publisher key + domain tag; grants nothing) + Phase B
the kernel bridge (a distribution publisher signature is verified against the
KERNEL-recomputed retained candidate.sha256, but the candidate stays inert — load/
execute/install still denied; provenance != load-worthiness; m12-distribution-
provenance 223/223). **Slice 2 DONE:** a local content-addressed registry — a raios-core
entry model + selection evaluator (recomputes sha256, grants nothing) + the kernel
read-only registry-selection diagnostic that stages a valid-provenance entry into the
existing candidate-intake path as an INERT candidate (load/execute/install/persist
still denied; m12-distribution-provenance 225/225). **Slice 3 DONE** (commit e3b34d2):
multi-entry registry — hold + select-by-hash among many content-addressed entries with
dedup + capacity bound + tamper rejection (recompute, never trust a stored hash); still
inert. **Slice 4 DONE** (commit 1fac7b9): chunked delivery — a large artifact arrives as
ordered content-addressed chunks that reassemble only when every chunk is present AND the
recomputed whole-sha256 matches the declared target, else fail-closed; the finalized whole
is still an inert candidate (35 raios-core tests green across Slices 2-4). **Slice 5 DONE:**
the kernel bridge now selects between two signed local registry entries by hash and stages the
selected artifact as an inert candidate; its chunked bufecho selftest reassembles out-of-order
chunks before staging, still grants nothing (`m12-distribution-provenance` 204/204). **Slice 6
DONE:** a real serial local delivery transport now accepts a signed artifact as bounded
content-addressed chunks, denies bad chunk hashes before staging, and stages a valid
reassembled artifact only as an inert candidate (`m12-distribution-provenance` 215/215).
**Slice 7 DONE:** a non-builtin current-boot local catalog now retains signed artifact
metadata, starts the same bounded chunk transport by content-hash selection, denies wrong
selectors before delivery starts, and still stages only an inert candidate after chunk,
whole-hash, and provenance verification (`m12-distribution-provenance` 228/228).
**Slice 8 DONE:** a real host-side static/CAS source now publishes the signed artifact
into the local registry, exports the registry blob as bounded serial catalog/chunk
commands, and proves those exported commands feed the guest local catalog while still
staging only an inert candidate (`m12-distribution-provenance` 229/229). **Slice 9
DONE:** the local registry/export packet now carries the receiver-required raiOS Wasm
artifact identity descriptor, current-boot load descriptor, P-256 keys, and signatures,
with host-side signature/hash binding checks, still as non-authorizing provenance
(`m12-distribution-provenance` 229/229). **Slice 10 DONE:** the guest local catalog
now retains the host-exported receiver-identity hash/binding metadata as
current_boot RAM-only local-only evidence, surfaces it on catalog begin/finalize,
and still reports guest signature verification not performed plus M6/M7 reverify
required (`m12-distribution-provenance` 231/231). **Slice 11 DONE:** the host export
now carries the raw receiver descriptor/key/signature bytes through a bounded evidence
channel, and the guest recomputes hashes, re-verifies both P-256 signatures,
re-checks descriptor bindings, and marks receiver identity complete only after guest
verification, still without install/load authority (`m12-distribution-provenance`
240/240). **Slice 12 DONE:** raiOS now exposes a receiver-identity load preflight
that refuses to evaluate before guest-complete receiver evidence exists, then
names the missing M6/M7/provider/owner gates while still denying load/install
(`m12-distribution-provenance` 244/244). Next M12+ slice: bind that preflight to
the inert retained candidate produced by catalog finalize, so the missing-gate
diagnostic is tied to the actually reassembled artifact hash, still without
load/install authority. **Slice 13 DONE:** the receiver-identity load preflight
now refuses to name the missing M6/M7/provider/owner gates until the local
catalog delivery has actually reassembled and staged the matching inert
retained candidate (`m12-distribution-provenance` 246/246). **Slice 14 DONE:**
the generic `raios.module_load_gate.v0` denial now carries that same
catalog-finalized receiver preflight, so a real load request sees the bound
receiver/candidate facts while load/install/execute/persist stay false
(`m12-distribution-provenance` 246/246). **Slice 15 DONE:** that receiver preflight is now
an eleventh non-authorizing source fact in the denied load-gate
loader-runtime readiness map, so the real load denial can distinguish
receiver/candidate bound from the still-missing M6/M7/provider/owner gates
(`m12-distribution-provenance` 246/246). **Slice 16 DONE:** the denied
load-gate loader-runtime readiness now reports
`m6_m7_reverify_input_check`: receiver preflight is ready, but M6
reverification evidence and M7 loader-policy evidence are explicitly missing,
and all enter/load authority flags remain false
(`m12-distribution-provenance` 246/246). Next M12+ slice: add the first
concrete M6 reverify input diagnostic that consumes this check, still
read-only and non-authorizing. **Slice 17 DONE:** that check now includes
`m6_reverify_input_diagnostic`, so a real load denial can distinguish
receiver/candidate preflight ready from the still-missing M6 reverify evidence
while all M6 enter/load authority flags remain false
(`m12-distribution-provenance` 246/246). Next M12+ slice: add the first
concrete M7 loader-policy input diagnostic that consumes the M6 diagnostic,
still read-only and non-authorizing. **Slice 18 DONE:** the same check now
includes `m7_loader_policy_input_diagnostic`, so a real load denial can
distinguish "M6 evidence absent before M7 policy" from the separately missing
M7 loader-policy evidence while all M7/load authority flags remain false
(`m12-distribution-provenance` 246/246). **Slice 19 DONE:** the same check now
includes `provider_trust_input_diagnostic`, so a real load denial can
distinguish "M7 loader-policy evidence absent before provider trust" from the
separately missing provider-trust evidence while all provider/load authority
flags remain false (`m12-distribution-provenance` 246/246). Next M12+ slice:
add the first read-only owner-key provisioning posture diagnostic: persistent
install requires hardware-bound owner key material, RAM boot may only create an
ephemeral `current_boot` key, and `owner_sealed` remains false until real
sealing evidence exists. **Owner-key provisioning slice DONE:** `system.honesty_report`
now reports `owner_key_provisioning`: persistent install policy is
hardware-bound owner-key generation, RAM policy is ephemeral `current_boot`
only, no key material is exported, no key is falsely reported as generated, and
owner-seal/install/load/durable authority stay false
(`m12-distribution-provenance` 246/246). Next owner-key slice: consume real
entropy readiness plus hardware/TPM-binding state before any RAM ephemeral key
or persistent owner key can be generated. **Owner-key evidence input slice
DONE:** the same honesty report now includes `owner_key_evidence_input` with
observed `core.entropy` readiness/RDRAND evidence and explicit missing
hardware/TPM-binding state, so RAM ephemeral key work has a real input while
persistent owner-key sealing remains denied (`m12-distribution-provenance`
246/246). **Owner-key RAM candidate slice DONE:** raiOS now generates a real
RAM-only `current_boot` owner-key candidate from observed entropy, exposes only
a non-secret handle/fingerprint, classifies key material as secret/non-exported,
and keeps owner-seal/install/load/durable authority denied
(`m12-distribution-provenance` 246/246). Next owner-key slice: add the first
real hardware/TPM-binding evidence probe for persistent owner-key provisioning;
no persistent owner seal until hardware-bound sealing evidence exists.
**Owner-key hardware evidence probe slice DONE:** the honesty report now proves
the real Limine RSDP/ACPI path was used to validate the ACPI root and search
for a `TPM2` ACPI table; the focused QEMU profile reports ACPI present/root
valid but `tpm2_acpi_absent`, so persistent owner-key input stays denied
(`m12-distribution-provenance` 246/246). **Owner-key TPM interface bridge slice
DONE:** when a real `TPM2` ACPI table is present, raiOS now parses and reports
the table physical address, platform class, control-area/FIFO base, start
method, interface kind, and non-authorizing interface-status posture; focused
QEMU still proves the absent path, RAM-only `current_boot` key generation still
happens, and persistent install/owner-seal/load/durable authority stay denied
(`m12-distribution-provenance` 246/246). **Owner-key Surface capture command
DONE:** `ownerkey` prints the same RAM-key/TPM2/status/NO-authority posture in
short console/serial form for real Surface capture
(`m12-distribution-provenance` 252/252). **Owner-key TPM status-read plan DONE:**
`ownerkey` and `system.honesty_report` now also report the computed read-only
TPM status-register plan (CRB control-status or TIS STS address/width) while
QEMU's absent-TPM path correctly reports no plan and no authority
(`m12-distribution-provenance` 253/253). Next owner-key slice: boot this on the
real Surface path, run `ownerkey` and `system.honesty_report`, capture actual
TPM2 details/status-read plan, then add the narrow volatile read of that
planned status register; no persistent owner seal until a real seal/unseal
evidence loop exists. (3)
provide real
trust inputs (a cryptographically
trusted time source, real CA roots, a live second provider) → I finish M10 real
validation — STILL YOURS. (4) the owner-key sealing ceremony → the FINAL step —
STILL YOURS. Until then every label stays honestly unverified / dev_key_not_owner_
sealed. **OWNER SIDE TRACK (parallel, Codex workers): Surface Pro 4 Marvell 88W8897
WiFi driver — firmware sequencer, bounded BAR2/DMA firmware bring-up,
GET_HW_SPEC, bounded legacy response scan, validated live SSID parsing, and a
clickable guided UI through RAM-only credential entry are committed and
VM-smoked; the owner has proved real scan results on Surface hardware. Active
RX-PFU/event rings remain parked after the all-ones MMIO/freeze evidence. Next
WiFi slice after the guided-flow hardware check is the first real fail-closed
association step; link/DHCP authority and durable secrets remain denied.** Process (owner
2026-07-08): per-slice max-effort review DROPPED; scoping + implementation both run
as Codex workers; Claude orchestrates/verifies.

M10 progress (all grants-nothing, honestly labeled, committed): M10A-1 provider-
trust HONESTY evaluator (can never overclaim chain/time validation; webpki
overclaim denied) + M10A-2 kernel reports its real honest posture; M10B-1
provider-agnostic ProviderTrustDescriptor + M10B-2 descriptor-driven kernel
honesty (proven for OpenAI AND a synthetic second provider, one shape); M10C-1
honest CMOS-RTC wall clock (source cmos_rtc_unverified, trusted:false, grants
nothing) + M10C-2 cert-validity-window awareness vs that clock (within/not-yet/
expired, UNVERIFIED-BASIS, fixed-window deterministic proof). quick 609/609.
**M10 REMAINDER IS OWNER/PRODUCTION-GATED (explicit TODOs):** a cryptographically
TRUSTED time source (NTP/NTS/Roughtime/platform); real X.509 DER notBefore/notAfter
parse of a LIVE pinned handshake cert; real certificate-CHAIN validation (needs CA
roots; vendor TLS does not expose intermediates); a LIVE second provider (real
pins + network). Making cert-time/chain contribute to POSITIVE trust requires all
of the above. Until then, every trust label stays honestly "unverified/not_validated".
M9 delivered: M9A durable typed memory (records/decisions/problems, supersede-not-
overwrite), M9B agent-authored confined observations, M9C-1 the read-only context
broker (fail-closed reparser + supersede/R1 + classification), M9C-2 the full
provider-export path (2a classification-firewall gate, 2b deduped denial audits,
2c the AUTHORITY FLIP — authorize + durable export_audit on a test-only positive
vector, honest dev_key_not_owner_sealed, NO transmission), and M9D reboot
durability (memory survives a real power-cycle byte-intact). M9 CLOSE proven:
full 8168/8168, recovery byte-identical 3833/3833. Owner TODO carried forward:
real provider transmission needs production network + a genuinely verified TLS
pin; the owner-key sealing ceremony remains the final M12+ step. (Historical
note: the M6 opener below was the cursor as of 2026-07-06.)
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
review: SHIP. Regression green (quick/recovery byte-identical/m6c-promotion/full 7834).
**M9A-3 done (2026-07-07 — CLOSES the M9A block):** M9A-3a added the write-side supersede confinement
(audit kinds can never be authored as superseding records; supersedes ≤8; no self-supersede; decision needs
entity+source, problem needs entity+status) with the read-side R1 rule explicitly deferred to M9C. M9A-3b then
durably wrote THREE truthful system-authored facts — a general `decision` (module sharing is owner-confirmed
vision), a `problem` (memory.* mutations still denied), and a refined `decision` that SUPERSEDES the first
(sharing = candidate intake, NEVER install) — proving supersede-not-overwrite: `memory-durable` 77/77 with all
three records' pinned golden hashes matched (exact bytes on disk, including B's `supersedes:[A.id]`). An
adversarial review caught + fixed a top-level over-claim (the trio response now derives success from the real
per-record evidence). Grants nothing new; system-authored only (agent write is M9B); dev_key_not_owner_sealed
/ current_boot. Regression green (quick/recovery byte-identical/m6c-promotion/full).
**M9B-1 done (2026-07-07 — first AGENT-authored durable write):** M9B-1a added the evaluator
`agent_authored` confinement (agent → observation-only, no supersede, local_only). M9B-1b added a new
narrow `memory.observation_log_append` method: the agent supplies entity/predicate/value/source as a
base64 blob; the kernel FORCES id (per-boot counter), kind=observation, classification=local_only,
authority=agent, source.method, tags, supersedes=[] and appends through the shared gauntlet. The broad
`memory.record_observation` (and all memory.* mutations) STAY denied — a new method, not a flip.
`memory-durable` 105/105 (golden-pinned agent record landed byte-exact + a 5-case fail-closed denial
matrix + the undercharge guard). Max-effort review SHIP (parser-escape / authority-forge / undercharge
all closed; a fail-closed frame-exceeds-charge guard added). A host-transport UART-FIFO overflow on the
long agent command was fixed by pacing the send (like submit_candidate_chunk). Grants nothing new; dev_key
/ current_boot. Next: **M9C** (the read-only broker — R1 supersede-target rule + the LOW-1/LOW-3 trust
rules above + typed-fact reads drawing on the durable M9A/M9B records). (M7 map
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

**M9C broker-trust rules (from the M9B-1b review, deferred to M9C):** (1) order agent records by
the RECLOG frame seq / boot_id, NEVER by the payload `sequence` field — for agent observations that
field is an agent-local attempt counter (advances on gauntlet-denied attempts, may gap/differ from
the frame seq). (2) Trust the kernel-FORCED `authority="agent"` + `source.method` as the authority
signal; NEVER trust the agent-supplied `source.record_id` (a spoofable locator that can be made to
look system-authored). Both are read-side; the M9B-1b write path already forces the authority fields
and confines agents to non-supersede local_only observations.

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

### M13 Durable Storage Substrate (pre-planned 2026-07-09)

Capability sentence: "raiOS persists its typed, classified,
provenance-bound facts to a REAL, structured, crash-consistent,
encrypted store on a real block device — with capacity and structure
beyond today's fixed durable region — while write authority to the
owner's real medium stays owner-gated and the immutable recovery core
is never touched."

Motivation: today (through M9D) durable memory records + the M7D
content-addressed artifact store survive a real reboot BYTE-INTACT, but
they live in a FIXED reserved durable region — not a real block device,
no filesystem structure, no capacity growth, no encryption-at-rest.
Owner asked (2026-07-09) to plan the "richtige Persistenz": a real
filesystem / larger block-device backend + encryption-at-rest. This
milestone builds the MECHANISM; the authority to write the owner's real
medium stays owner-gated (same discipline as every durable-write flip:
`dev_key_not_owner_sealed` until the sealing ceremony).

TARGET MEDIUM (owner decision 2026-07-09): a DEDICATED, separate
partition on the internal SSD — NOT the boot stick, NOT alongside
Windows. The owner creates one empty partition (once, from Windows);
raiOS writes ONLY to that designated partition and REFUSES every other
partition by construction — the Windows partition, the EFI system
partition, the Windows recovery partition, and the immutable raiOS
recovery core are all rejected fail-closed. The target partition must be
identified by a stable, verified marker (a raiOS GPT type GUID and/or a
signed superblock magic) that raiOS re-checks on every mount before any
write; a missing/foreign marker => read-only, never write. Storage DMA is
confined by the IOMMU/VT-d track (ADR 0010) once that enforces; until
then it is honestly labeled "owner-trusted, not IOMMU-confined".
Persistence still obeys ADR 0004: audit + rollback exist before any
long-term-memory authority; no fake persistence; typed/classified/
provenance-bound facts, never a raw log.

Slices (each grants nothing until an explicit owner-gated flip):
- M13A Block device — A-1 READ-ONLY block driver for the real boot
  medium (starting with USB Mass Storage/BOT on the prepared GPT stick:
  identify device, read sectors, verify GPT/SEED_DATA) **DONE for the
  prepared USB stick**; A-2
  bounded, audited, write-then-read-back verified WRITE to a dedicated
  owner-approved region only (fail-closed; recovery core + foreign
  partitions refused by construction).
- M13B Structured store / filesystem — B-1 on-medium layout (superblock
  + typed content-addressed record index + free-space map) beyond the
  fixed region; B-2 migrate the M9 durable memory records + M7D artifact
  store onto it behind the SAME typed API, re-prove M9D reboot
  byte-intactness on the new backend; B-3 crash-consistency (journaled /
  log-structured atomic writes) proven by a mid-write power-cycle test.
- M13C Encryption-at-rest — C-1 authenticated encryption of durable
  records/artifacts with a key derived from the owner seal (data-at-rest
  meaningless without owner key K; dev-key-derived + honestly labeled
  until the ceremony); C-2 tamper-evident integrity over the store +
  audit log (mutation of the medium is detected, fail-closed).

Owner-gated inputs: which medium/region raiOS may write (recommended:
boot medium / dedicated partition, never Windows/NVMe without explicit
approval); the sealing ceremony that turns the dev encryption key into
the real owner-sealed key. A detailed file:line map (like the M10/M11
maps) is a follow-up scoping pass before implementation.

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
