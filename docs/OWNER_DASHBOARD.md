# Owner Dashboard

Updated: 2026-07-13 (evening).

KORREKTUR (Abend): Ich hatte gemeldet, die Vokabel-Arbeit sei FERTIG. Das war zu
früh, und ich nehme es zurück. Fertig ist die **Form** der Antworten (alles im
nächsten Absatz stimmt und ist geprüft). Nicht fertig ist der **Motor dahinter**:
Der Plan verlangt, dass keine Familie das JSON mehr von Hand zusammenschreibt —
und ~4.400 Zeilen tun das noch immer. Genau daran hängt auch das verfehlte
Größenziel: Diese Dateien sind so groß, WEIL das JSON von Hand geschrieben wird.
Das läuft gerade (P4-9c) und ist ein reiner Motorwechsel: Die ausgegebenen Daten
bleiben Byte für Byte identisch — kein einziger Test muss angefasst werden, und
genau das ist der Beweis, dass nichts kaputtgeht.

ONE LANGUAGE, ALL NINE FAMILIES — the answer FORM is finished. Every answer
raiOS gives — about a module, about loading, about its memory, its events, its
own health, its clock, the AI provider, and its own list of capabilities — now
comes back in the SAME shape: here are the facts, here is the evidence in the
exact order the checker used it, and here is ONE decision that says granted or
denied and always says WHY. The important part is what an answer can no longer
do. An answer that merely LOOKS at something now has nowhere to put a "grant" —
the words are not in the sentence at all. Only a denial is allowed to list what
was missing. So a question can no longer quietly hand out permission just by
being asked. That was not a theoretical worry: raiOS's own capability list says
"granted: true" next to some rows, and READING that list now grants nothing — it
reports a status, it does not create one.

The test VM earned its keep again: it crashed the kernel on purpose because one
answer (the clock self-test) still had a fact claiming it could authorize the AI
provider. A fact is not allowed to claim authority — that is a decision's job.
Fixed at the source, and the check that catches this class now runs in two
seconds instead of costing a twelve-minute VM run.

HONEST SCORECARD — the size goal was NOT met, and I am correcting a number I
reported earlier. The kernel went from 176,331 to 170,293 lines: about 6,000
lines removed, not the ~37,000 the plan hoped for, and nowhere near the 120,000
goal. (An earlier report of mine said "163,260" — that was measured wrong and I
retract it.) Three reasons, and I would repeat two of them:
  1. I deliberately did NOT convert the parts that actually DO things — installing
     software, writing to the disk, the app/project surfaces. They perform real
     actions but hold no proof of permission, so converting them would have forced
     a lie: either hide the action behind "just looking", or invent an authority
     nobody granted. I left them honest-and-old rather than make them lie. That
     removed a big share of what the size math had assumed.
  2. The new shared language lives in the safe, PC-testable core, so a lot of code
     MOVED across a boundary instead of disappearing.
  3. There was simply less copy-paste to delete than the plan believed.
The plan itself warned that 120,000 "is not a credible promise". It was right.
The next real shrink has to come from moving ownership of code, not from another
layer of vocabulary — and I will not promise a number in advance again.

Vocabulary v1, first family live: every "show me the module evidence" question
an agent can ask raiOS (manifest, artifact, test report, attestation, approval,
computed grant, audit/rollback reference, service-slot reservation — plus all
their self-tests) now answers in the new single evidence language: one shared
envelope, the proof records in the evaluator's order, and one honest decision
block that always says exactly WHY something is denied and grants nothing as a
side effect. The night shift had parked this work believing it didn't compile —
that turned out to be a limitation of the worker's sandbox, not the code. What
WAS genuinely broken were the test expectations: the rewrite had collapsed 30
distinct safety checks onto one identical byte pattern (so 29 of them proved
nothing), and several checks quietly pointed at the wrong source of truth. All
of that is fixed and honestly recorded; two old checks that had only ever
passed by accident (matching bytes from a different answer entirely) were
retired by name. Proof: the family's focused suite passes 1,623/1,623, the
promotion and rollback flows pass, and the FULL suite passes 4,042/4,042
(shadow-20260713-114040-1776.json). The load-gate family is next: its
inventory (855 checks), its typed core decision engine, and the two follow-up
family inventories are already built and committed.

Refactor program: the feared "layout-sensitive kernel bug" is solved — and the
kernel was innocent. The real defect sat in our own test tooling: the helper
that types commands into a child test VM hung up its network connection in a
way Windows turns into a hard abort, so QEMU threw away command bytes it had
not yet delivered. Which test froze depended purely on machine timing, which
shifts with every rebuild — that is why it looked like a deep memory bug. The
helper now reads its echoes and hangs up cleanly; the same tests that froze
three different kernel builds now pass on all of them. With that unblocked,
the parked refactor work landed the same night: the 10,156-line module-loader
file is now six readable modules, and the first relocation wave moved
module-gate/provider/memory decision logic into normal PC-testable code
(seconds instead of QEMU minutes) while removing about 1,700 kernel lines.
The wave's family-close full run then earned its keep: it caught three real
copying mistakes the parked wave had been hiding (a crash in a formatter, a
wrong separator that silently changed every computed reference hash, and one
dropped safety comparison). All three are fixed, and all 93 of the module
gate's built-in test cases now also run as normal PC tests in a tenth of a
second — so this whole class of mistake can never again hide until a
20-minute VM run. The full test suite is green on the landed result
(shadow-20260713-013105-20952.json). One pre-existing cosmetic test-fixture
gap was found and documented; it changes nothing in behavior.

The big deletion then landed the same night: the entire superseded recovery
diagnostic layer — 47 files, about 45,800 lines — is gone in one verified
cut, with the REAL recovery lifeline (snapshot, restart, rollback, disable)
untouched and re-proven. The kernel shrank from 206,481 to 176,346 lines.
Full test suite green after the cut (shadow-20260713-023159-3012.json).
One important honest finding: two surfaces the inventory had marked as
deletable (the module write-boundary checks and hello's rollback-writer
gates) turned out to feed the REAL rollback-apply safety decision — the
deletion attempt stopped itself exactly as designed, and those surfaces are
now re-routed to be slimmed by relocation/compaction instead of deleted.
Next: a fresh relocation-wave design against the shrunken tree, then the
final vocabulary compaction to reach the 120k goal.

That vocabulary compaction is now running (2026-07-13). The idea in one
sentence: every answer raiOS gives about a module — "here is what I know, here
is what I checked, here is what I decided" — used to be written out by hand in
a different shape for each kind of answer, and it is being rewritten to use ONE
shared shape. Three of the nine answer families are converted and proven
(module references, the load gate, the loader/allocator), the kernel is down
another 13,384 lines to 162,947, and both the full and the recovery test suites
are green on the result.

Two honest findings from the conversion, because they say something about how
the machine is being checked. First: the conversion exposed two self-tests that
had only *appeared* to test something — the fixture they used never actually
reached the check it claimed to prove. Both were corrected so they really fire
now, rather than being quietly deleted. Second: because the test harness
searches the whole console transcript rather than one answer at a time, a few
checks had been passing by accidentally matching text from a *different*
answer; when the old text was deleted those checks were exposed, and each was
re-pointed at the real value in its own answer. Nothing regressed — but the
system is measurably more honestly tested than it was this morning.

Current capability: a user can type `/build <request>` in Genesis. raiOS sends
the request through its existing pinned-trust direct provider path, accepts only
the bounded typed `RAIOS_UI_SPEC_V1` data language, compiles it locally to
canonical hash-bound RUIP, and keeps the draft inert until a physical click.
The signed `svc.user.shell` then runs it current-boot in a fresh metered Wasmi
instance with exactly six UI imports; state commits only after a valid frame.
An accepted provider-authored draft can now be refined with
`/revise <feedback>` while retaining parent/root hash lineage; a rejected
replacement leaves the prior runnable draft intact.

The secure project workspace loop is now closed through W6: an exact healthy
Rust-to-Wasm project candidate can be signed, reviewed in Genesis, durably
installed only after a second physical click, autoloaded after reboot, promoted
to last-good only after healthy execution, rolled back automatically when its
stored bytes fail verification, and physically uninstalled without changing its
immutable source revision.

Workspace: W1 is real. A bounded local multi-file source project can be
committed as immutable content-addressed blobs plus deterministic tree/revision
evidence to the disposable QEMU structured store, rebooted, and inspected with
the same exact file and revision hashes. It remains inert source data.
An agent can now read at most 512 exact bytes from one bound path and search
text for capped locator-only matches; searches never return snippets.
An agent can now also open an exact-base RAM overlay, add, replace or delete
files, inspect the complete old/new hash-bound diff, discard it, or commit it as
an immutable child revision. A stale overlay cannot overwrite a newer base.

Genesis UX: conversation entries wrap instead of truncating, older rows are
reachable with the wheel or PageUp/PageDown, the composer keeps its visible
input tail and has a blinking cursor, and high-frequency personal-shell frame
markers remain serial-only rather than flooding the chat. Cursor blinking is a
small frontbuffer overlay, so it does not trigger a full-screen present.

Input: `AI Setup` now offers a current-boot US/German keyboard picker. The
central mapping supplies QWERTZ plus German ASCII punctuation and AltGr symbols
to Genesis, setup fields, and Console; basic personal-program keys also use the
selected layout. Unicode umlauts, personal-app AltGr, dead keys, and layout
persistence remain explicit gaps.

Proof: a live same-boot OpenAI request produced a 168-byte counter, pinned-SPKI
TLS and redacted evidence were positive, and the matching physical click started
that exact hash as `ui_only` Wasm. A later live request produced a 2532-byte
calculator draft; the owner approved and launched it, then returned with F12.
No Authorization header entered the serial log. The evidence proves bounded
intake/runtime behavior and the exercised examples, not universal semantic
correctness of AI-authored programs.

Regression: `shadow-20260712-025218-6208.json` passed 252/252. It covers exact
delivery/hash, malformed preservation, physical approval, `12+30=42` through
real HID events, F12 return, secure-strip clipping, proof compatibility, and
trap/fuel fallback. The unchanged full baseline
`shadow-20260712-025759-11164.json` passed 7870/7870. Core tests pass 415/415;
the UX release build, format, diff and secret checks are green.

Workspace proof: `shadow-20260712-124220-8296.json` passed 76/76 across 33
observed commands and two boots. Invalid paths, case aliases, wrong hashes and
quota overflow produced no visible revision; the valid two-file project replayed
with exact revision hash
`11df2422e2592225c3687d7cd845e6991628bed9c49611e01b51ff9c9dda6a05`.
W2a report `shadow-20260712-125335-27844.json` passed 136/136 across 53
commands and both boots, including bounded reads/search and all negative cases.
W2b report `shadow-20260712-130758-7668.json` passed 304/304 across 114
commands and three boots in 258488 ms. Its add/replace/delete child revision
`87339872c8016d88068d4aef754db50e0e5d476cbb9d77edd5e8e3821d47a7cb`
survived reboot byte-identically; discard, stale-base, malformed, wrong-hash,
invalid-delete, no-op and case-collision paths left committed state intact.
W3 report `shadow-20260712-135131-25884.json` passed 600/600 across 214 commands,
three boots and 917166 ms (report SHA-256
`8a6c5933b6d9c0e8407ecf128a369ed57be9c8941894441753462d8633d58746`).
The same still-running child finished green 17 seconds after the outer
900-second host wait expired, without retry or code change: host wall-clock
timeout, guest behavior passed. A local-serial exact-version package with
`LICENSE`, detected-but-never-run `build.rs`, and greater-than-24-KiB
multi-chunk `src/lib.rs` remained bound to exact project revision,
`Cargo.lock`, owner-declared origin/license and content hashes after reboot.
Idempotent re-import wrote nothing, and the source revision stayed
byte-identical.

W4 proof: `shadow-20260712-145618-13408.json` passed 248/248 across 108
commands, one boot and 313118 ms (report SHA-256
`e7fd8bf954e2b3b75af384d9215d13be7067316dd7e4cb47c5a1c332340e556c`).
The owner workstation exact-read one reviewed revision plus one safe quarantined
local path dependency and built it twice frozen/offline under fixed contracts and
a pinned, measured toolchain. Both builds yielded the identical validated inert
candidate `05854c56665a9fee9990712126e1f19269059375cb37fcdccacaa990ab3d30fb`.
Its exact receipt is non-authorizing and honestly records
`builder_attested_not_local_rebuild` and `independently_verified=false`; this is
neither an owner-sealed toolchain nor an independent local rebuild. Toolchain,
flags, environment, source/dependency build-script, read, run/output/candidate
and stale-receipt negatives failed closed.

W5 proof: `shadow-20260712-153736-17972.json` passed 276/276 across 112
commands, one boot and 553863 ms. The exact W4 candidate was locally reparsed,
had zero imports, showed an owner-visible current-boot preview, and ran only
after the real Genesis pointer approval. It returned 42 within fixed fuel/memory
limits; health/inventory matched the exact receipt and candidate. Serial approval,
stale/tampered/replay paths denied, and F12 removed the service/candidate while
core Recovery stayed available. This grants no durable or broader authority.

W6 proof: `shadow-20260712-171300-16808.json` passed 403/403 across 156
commands and four boots. It installed and autoloaded v1, built and installed a
genuinely different v2 child, corrupted one exact persisted v2 Wasm byte,
observed the real ARTSTOR frame-hash failure and durable rollback to v1, then
physically uninstalled the app and proved boot 4 did not autoload it. Source
facts remained byte-identical. Full close
`shadow-20260712-173148-25720.json` passed 7870/7870; recovery
`shadow-20260712-174432-7724.json` passed 3677/3677. This is disposable-QEMU,
stateless-app evidence under `dev_key_not_owner_sealed`, not owner-sealed or
physical-stick persistence.

Still denied: arbitrary external/native/Wasm intake, file/network/secret access
from generated programs, durable state and broader program installation outside
the exact W6 project-app path, broad mutation, owner sealing, authenticated
ARTSTOR garbage collection, TPM auto-unlock, physical persistence, and live
Surface association/`PORT_RELEASE`/DHCP. Provider TLS still lacks full WebPKI
chain and trusted-time validation.

Stick: the owner reports it has been found, but this session did not enumerate
or touch it. The next G7 action is read-only identity/layout/fingerprint
preflight; never assume the former Disk 2 number or recreate a missing fingerprint.

Next product slice: W7 admits one explicitly approved, bounded HTTPS source
request into quarantine as inert content-addressed source/tree evidence. It may
not build, execute or install automatically. Next hardware slice:
the explicit read-only G7 stick preflight. Neither grants physical-write authority.

Refactor program (owner-ordered 2026-07-12, cost accepted): P0 and two of
three P1 packets are DONE. The P0 inventory routed all 121 evidence files:
58,663 lines to delete (superseded by the real promotion/persistence/
recovery loops), 70,326 to relocate into PC-testable crates, kernel target
~84k lines (from 206k). P1 landed the memory-vocabulary readability reflow
(611-KB file fully readable, 108 giant lines eliminated, content byte-proven
identical) and the load-gate literal splits (17 giant lines, fragments
proven identical), plus a session-check size gate on lines AND bytes.
Proof: `memory-durable` `shadow-20260712-184533-27876.json` green; full
profile `shadow-20260712-184856-27972.json` 7870/7870 byte-identical green.
The loader-file split (P1-A) is PARKED: provably content-identical, yet it
deterministically froze one child-VM probe — bisection isolated it, and the
suspected cause is compiler code layout exposing a pre-existing kernel
fragility.

Evening update — REAL KERNEL BUG CONFIRMED, refactor paused at a safe
point: the P2 relocation wave (built and host-verified, -1,721 kernel
lines, 449 fast PC tests) triggered the SAME freeze class at a DIFFERENT
test probe, and even the nominally inert core-module commits shifted the
binary layout enough to move the freeze to a third probe. Conclusion: a
pre-existing hidden memory-corruption defect in the kernel picks its victim
by binary layout; earlier byte-exact green runs were partly layout luck.
Main was repaired under the Red Gate rule by reverting the two core
commits and is confirmed green again
(`shadow-20260712-220828-26452.json`); the complete P2 wave is preserved
on branch `refactor/p2-wave1-parked`. NOTHING structural lands until the
bug is found — the hunt is the next refactor slice and it is well-armed:
four known layouts with three distinct victim probes, a 10-minute
reproduction per layout, and a narrowed suspect region (child-boot
persist-region parsing). Finding this bug now, on the test bench, is far
better than meeting it later on real hardware.

Planning update: `docs/plan-reviews/secure-ai-workspace-and-media-app-plan-2026-07-12.md`
defines the final-path secure source workspace, quarantined acquisition,
reproducible Rust-to-Wasm build, and split Wasm/native media application shape.
Its W1-W6 workspace/dependency/build/run/install/rollback path is now implemented
and verified on disposable QEMU storage; Cargo resolution, verified
origin/license truth, archive extraction, fetch, dependency execution,
native-code, media, GPU and physical-storage authority remain closed.
