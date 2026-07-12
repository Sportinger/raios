# Owner Dashboard

Updated: 2026-07-12.

Current capability: a user can type `/build <request>` in Genesis. raiOS sends
the request through its existing pinned-trust direct provider path, accepts only
the bounded typed `RAIOS_UI_SPEC_V1` data language, compiles it locally to
canonical hash-bound RUIP, and keeps the draft inert until a physical click.
The signed `svc.user.shell` then runs it current-boot in a fresh metered Wasmi
instance with exactly six UI imports; state commits only after a valid frame.

Input: `AI Setup` now offers a current-boot US/German keyboard picker. The
central mapping supplies QWERTZ plus German ASCII punctuation and AltGr symbols
to Genesis, setup fields, and Console; basic personal-program keys also use the
selected layout. Unicode umlauts, personal-app AltGr, dead keys, and layout
persistence remain explicit gaps.

Proof: a live same-boot OpenAI request produced a 168-byte counter, pinned-SPKI
TLS and redacted evidence were positive, and the matching physical click started
that exact hash as `ui_only` Wasm. No Authorization header entered the serial
log. Key-bearing image, target lane and log were deleted after hashing.

Regression: `shadow-20260712-025218-6208.json` passed 252/252. It covers exact
delivery/hash, malformed preservation, physical approval, `12+30=42` through
real HID events, F12 return, secure-strip clipping, proof compatibility, and
trap/fuel fallback. Core tests pass 415/415; format and release build are green.

Still denied: arbitrary external/native/Wasm intake, file/network/secret access
from generated programs, durable program install/state, broad mutation,
promotion, rollback application, TPM auto-unlock, physical persistence, and live
Surface association/`PORT_RELEASE`/DHCP. Provider TLS still lacks full WebPKI
chain and trusted-time validation.

Stick: the owner reports it has been found, but this session did not enumerate
or touch it. The next G7 action is read-only identity/layout/fingerprint
preflight; never assume the former Disk 2 number or recreate a missing fingerprint.

Next product slice: design durable program installation only through the existing
persistence, evidence, capability and rollback gates. Next hardware slice: the
explicit read-only G7 stick preflight. Neither grants physical-write authority.

Planning update: `docs/plan-reviews/secure-ai-workspace-and-media-app-plan-2026-07-12.md`
defines the final-path secure source workspace, quarantined acquisition,
reproducible Rust-to-Wasm build, and split Wasm/native media application shape.
It is design-only and grants no workspace, fetch, build, install, native-code,
media, GPU, or physical-storage authority.
