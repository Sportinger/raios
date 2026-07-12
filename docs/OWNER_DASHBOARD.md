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

Still denied: arbitrary external/native/Wasm intake, file/network/secret access
from generated programs, durable program install/state, broad mutation,
promotion, rollback application, TPM auto-unlock, physical persistence, and live
Surface association/`PORT_RELEASE`/DHCP. Provider TLS still lacks full WebPKI
chain and trusted-time validation.

Stick: the owner reports it has been found, but this session did not enumerate
or touch it. The next G7 action is read-only identity/layout/fingerprint
preflight; never assume the former Disk 2 number or recreate a missing fingerprint.

Next product slice: W3 dependency quarantine for one locked, inspectable,
inert dependency bundle bound to an exact source revision. This is not yet a
direct cloud-provider editing toolloop; build, install, execute and rollback
follow only after their named gates. Next hardware slice:
the explicit read-only G7 stick preflight. Neither grants physical-write authority.

Planning update: `docs/plan-reviews/secure-ai-workspace-and-media-app-plan-2026-07-12.md`
defines the final-path secure source workspace, quarantined acquisition,
reproducible Rust-to-Wasm build, and split Wasm/native media application shape.
Its W1/W2 workspace path is now partially implemented and verified; fetch,
dependency execution, build, install, native-code, media, GPU and
physical-storage authority remain closed.
