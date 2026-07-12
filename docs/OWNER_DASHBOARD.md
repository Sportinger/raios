# Owner Dashboard

Updated: 2026-07-12.

Current capability: a user can type `/build <request>` in Genesis. raiOS sends
the request through its existing pinned-trust direct provider path, accepts only
the bounded typed `RAIOS_UI_SPEC_V1` data language, compiles it locally to
canonical hash-bound RUIP, and keeps the draft inert until a physical click.
The signed `svc.user.shell` then runs it current-boot in a fresh metered Wasmi
instance with exactly six UI imports; state commits only after a valid frame.
An accepted provider-authored draft can now be refined with
`/revise <feedback>` while retaining parent/root hash lineage; a rejected
replacement leaves the prior runnable draft intact.

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

Still denied: arbitrary external/native/Wasm intake, file/network/secret access
from generated programs, durable program install/state, broad mutation,
promotion, rollback application, TPM auto-unlock, physical persistence, and live
Surface association/`PORT_RELEASE`/DHCP. Provider TLS still lacks full WebPKI
chain and trusted-time validation.

Stick: the owner reports it has been found, but this session did not enumerate
or touch it. The next G7 action is read-only identity/layout/fingerprint
preflight; never assume the former Disk 2 number or recreate a missing fingerprint.

Next product slice: W6 separately installs an approved W5 artifact into the
existing content-addressed store/log, autoloads it after reboot, confirms
last-good, and proves rollback plus uninstall. Install, promotion, persistence
and rollback remain closed until that focused storage/recovery/boot profile is
green. Next hardware slice:
the explicit read-only G7 stick preflight. Neither grants physical-write authority.

Refactor decision: the owner ordered a real kernel-mass refactor
(2026-07-12, cost accepted). `docs/plan-reviews/kernel-mass-refactor-plan-2026-07-12.md`
defines the four-phase program: inventory, readability splits, host
relocation of pure logic, retirement of superseded evidence, then vocabulary
compaction (formal Batch 6, reopening ADR 0006) last. Plan only — no code
moved yet; P0 is a read-only inventory slice.

Planning update: `docs/plan-reviews/secure-ai-workspace-and-media-app-plan-2026-07-12.md`
defines the final-path secure source workspace, quarantined acquisition,
reproducible Rust-to-Wasm build, and split Wasm/native media application shape.
Its W1-W5 workspace/dependency/build/current-boot-run path is now implemented
and verified;
Cargo resolution, verified origin/license truth, archive extraction, fetch,
dependency execution, install, native-code, media, GPU and
physical-storage authority remain closed.
