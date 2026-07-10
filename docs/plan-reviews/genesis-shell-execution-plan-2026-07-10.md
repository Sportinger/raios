# Genesis Shell + Personal Shell Execution Plan

Status: execution-ready owner plan, 2026-07-10

Execution cursor: I0/G0 is complete at `73c9677` and C0/ADR 0012 is complete at
`8e5ff8d`; A0/B0, A1/G1, C1/G5.1 and D0 are complete. C1's isolated AHCI
format/append/reboot proof is `shadow-20260710-032738-34812.json` (9/9); C2 and
C3 foundations are complete. A2/G2 core Context/recovery interaction is verified
by `shadow-20260710-034302-30252.json` (181/181): shared typed problem facts,
cached/redacted Recovery, and its shared Lifeline actions are live. The bounded
zeroizing secure overlay is not yet a Broker/use path. C4/I3 unarmed groundwork
(`95b7bf4`, `d27c96d`, `f90e7db`, and C1 extraction `0920346`) adds exact
replayed/readback keyring restore, typed ciphertext records and complete-history-
only nonce reconstruction; focused regression
`shadow-20260710-040559-24348.json` passed 9/9 with zero failures. It grants no
Vault set/unlock/decrypt/plaintext use, consumer, audit, physical-target or durable
credential authority. I2/G3 passed its required Sol review. After the original
ignored local signer was lost, the owner accepted ADR 0013 and its tracked
`descriptor-resign` host tool now proves raw-byte P-256 sign/verify with an
altered-byte rejection. It remains explicit local `dev_key_not_owner_sealed`
provenance, not OTA or runtime authority. I2/G3 is now verified by
`shadow-20260710-121953-4964.json`: the exact signed `svc.user.shell` proof
executes in one fresh metered Wasm instance through only the six specified `ui.*`
imports, with bounded validated display-list return, malformed/trap/fuel negatives,
and no loader, persistence, secret, network, provider, recovery, or broad mutation
authority. It remains non-default current-boot test infrastructure. **AB/G4 is now
complete**: `shadow-20260710-124838-24564.json` passed 206/206 with signed entry,
sanitized input, core-only F12, trap/fuel fallback, dynamic current-boot inventory
removal, Recovery after fallback, five bound QEMU captures, and a byte-identical
secure-strip pixel hash across personal rendering. I3/G5.4 is independently
review-gated. Its unarmed Broker foundation now requires opaque complete-history,
mutation and use evidence and exposes only fixed NXP/OpenAI one-use outputs; core
tests pass 396/396 and `shadow-20260710-132631-20352.json` passes 9/9. Runtime secret
history is now bound from the exact revalidated QEMU store on both boots by
`shadow-20260710-133203-24112.json` (11/11). Runtime secret authority remains
unclaimed: the distinct owner software-pinned Core Policy now binds the complete
Limine executable to exact A/1 BOOTCTL state (`shadow-20260710-145039-13864.json`,
5/5), and the Broker retains that verified identity beside the replay on both boots
(`shadow-20260710-150107-28328.json`, 13/13) without accepting caller policy data.
The first armed I3 slice is verified by `shadow-20260710-160920-28360.json` (29/29):
one-time RR1 display/checksum, physical re-entry, exact wrapper commit/readback,
independent reboot/replay and recovery unlock all pass without RR1 in serial/report
evidence. The provider half of I3/G5.4 is now verified by
`shadow-20260710-174308-19744.json` (42/42): physical Genesis entry saves an encrypted
OpenAI credential on the exact disposable QEMU C1 store; after reboot/replay and RR1
unlock, a contained exact Authorization-header consumer receives it only after durable
`local_only` pre-use audit readback, typed reparse and rescan. Production OpenAI is
wired behind real pinned trust. This does not prove live network provider success,
physical persistence, WiFi Vault use, forget/SAFE behavior, TPM auto-unlock, Secure
Boot, deterministic ESP A/B selection or anti-rollback.

Target orchestrator: Codex 5.6, reasoning effort `xhigh`

Execution mode: autonomous until completion or a named safety tripwire
Repository: `C:\Users\admin\Documents\raios2`

## 0. Capability sentence and finish line

After this plan is complete, a user can boot the prepared raiOS USB into a calm,
universal Genesis shell, reach and configure AI from the first screen (and talk to it
when network, provider trust, and a key are available), configure the existing WiFi
path, inspect typed system context, enter the real recovery
lifeline, and run a signed replaceable personal-shell Wasm service inside a clipped,
capability-gated surface that cannot cover Genesis, recovery, permission, or secret
UI. After one successful Genesis setup, an approved machine can also reconnect to the
known WiFi and reuse its provider credential after reboot without asking for those
individual secrets again: both live only as authenticated ciphertext in the persistent
Secret Vault, and plaintext use remains purpose-bound to the trusted WiFi/provider
paths.

The delivered stick starts in Genesis and honestly says `Personal shell: not
created`. The proof personal shell is test infrastructure for the real runtime path;
it is not silently installed as the user's shell and it does not pretend that
arbitrary generated external modules are accepted yet.

The plan is finished only when all of the following are true:

- the exact final image boots in QEMU into the new Genesis design at 1280x800;
- the current e1000/DHCP, direct-provider, Marvell firmware/scan, USB input, serial,
  persistence, and recovery behavior remains present and honestly labeled;
- the personal-shell display-list boundary is real, signed, executed by wasmi, and
  fail-closed under negative tests;
- new behavior lives in the final modular boundaries from section 5; no temporary
  shell/storage/security monolith or duplicate fallback remains (hardware drivers keep
  the explicit exception stated there);
- recovery remains usable without cloud AI and survives a personal-shell trap;
- a dedicated, identity-checked raiOS data partition hosts a crash-consistent encrypted
  Secret Vault separate from normal memory/audit records and from both ESP slots;
- a fake WiFi passphrase and fake provider key survive a real two-boot VM cycle,
  decrypt only after an authorized vault unlock, reconnect/reuse through their exact
  consumers, and never appear in serial, screenshots, provider context, audit, or the
  personal shell;
- a personal-shell/service crash does not lose the unlocked core-owned vault handle;
  SAFE/recovery can explicitly reconnect without re-entering each credential, while a
  failed key policy or corrupt vault fails closed;
- final focused structured-store, Secret Vault, recovery and Genesis profiles plus the
  full Shadow VM profile pass;
- the default release artifact contains no provider secret, recovery key, VMK, wrapper
  or Vault record;
- the already prepared SanDisk stick is refreshed without repartitioning or
  touching `SEED_DATA`, and source/readback kernel hashes match;
- the real-Surface WiFi association/`PORT_RELEASE`/DHCP gap remains explicitly open
  unless it is physically proven later; guided input stability is already proven.

## 1. Owner decisions carried by this plan

Launching the goal prompt at the end of this document means the owner approves only
these authority-bearing changes:

1. the exact fail-closed personal-shell UI Wasm imports and compositor boundary in
   section 6;
2. execution of the new checked-in, dev-key-signed proof service as non-default,
   `current_boot` test infrastructure with `owner_sealed:false`; and
3. authenticated Secret Vault writes for exactly WiFi passphrases and provider API
   keys to an already provisioned, dedicated raiOS data partition that passes the M13
   identity/region/write-authority gates; AES-256-GCM record encryption, HKDF-SHA-256
   key separation, explicit zeroization, a TPM-sealed VMK wrapper, and a high-entropy
   recovery-key wrapper as specified in section 7;
4. direct pinned dependencies on the versions already present in `Cargo.lock` through
   `embedded-tls`: `aes-gcm = 0.10.3`, `hkdf = 0.12.4`, and `zeroize = 1.8.2`, all
   `no_std`/minimal-feature and used instead of custom cryptography; and
5. a non-formatting ESP-A refresh of the already prepared, identity-checked SanDisk
   currently known as Disk 2.

It does not approve raw framebuffer access for Wasm, raw secret access, arbitrary
external artifact intake, a provider-as-recovery shortcut, positive WiFi link claims,
automatic secret writes to the boot stick/ESP/Windows/foreign partitions, general
secret export, password-derived keys without a reviewed memory-hard KDF, disk
repartitioning, other dependency upgrades, or a broader mutation capability.

The implementation must record the shell boundary in ADR 0011 and the Secret Vault /
storage/key-custody boundary in ADR 0012 before their authority slices. These are not
new owner questions: this conversation plus the final goal prompt is the decision.
Any materially broader design still stops.

## 2. Codebase baseline that Slice 0 must revalidate

The detailed current state remains authoritative in `docs/PROJECT_STATUS.md` and
`docs/ROADMAP.md`. This section is a map, not a copied status ledger.

### Current visible shell

- `seed-kernel/src/main.rs` creates `ui::StatusUi`, initializes the console/provider/
  USB/WiFi/input/network paths, polls them, and requests redraws. Its boot/event-loop
  wiring is a single-writer integration file.
- `seed-kernel/src/ui.rs` is approximately 2,900 lines. It currently owns visual
  rendering, pointer hit-testing, AI/CONSOLE/SET tabs, system-status details, settings,
  guided WiFi state, WiFi dialogs, and cursor drawing.
- `seed-kernel/src/console.rs` is approximately 2,050 lines. It mixes interactive UI
  state, focus, chat and command buffers, secret-entry modes, provider/WiFi setup,
  serial commands, and agent dispatch. It currently boots into `UiView::Console`.
- `seed-kernel/src/framebuffer.rs`, `text.rs`, and `input.rs` provide useful mature
  primitives: double buffering, 8x8 antialiased glyphs, scaling, keyboard events, and
  pointer input. Do not replace them as part of this plan.
- The current 1280x800 view uses one-pixel-scale 8x8 text, a fixed status-detail area,
  and technical AI/CONSOLE/SET navigation. It works, but it is not the universal
  Genesis design.

### Existing behavior that the redesign must call, not duplicate

- Chat already flows from the interactive state through `provider::submit_text`;
  provider events return through `provider::poll` and are written into the current
  chat/output buffers.
- Provider key and WiFi passphrase entry are masked and RAM-only.
- `raios-core/src/memory_record.rs` deliberately makes secret plaintext structurally
  non-durable (`secret_never_durable_until_sealed_secret_design`). The vault must add a
  separate encrypted-record type and must not weaken that invariant or smuggle
  ciphertext/plaintext into ordinary memory records.
- `seed-kernel/src/owner_key.rs` currently creates only a 32-byte RAM candidate and
  reports TPM2 ACPI/interface/status-read evidence. It has no TPM command transport,
  seal/unseal path, persistent key wrapper, or secret accessor. The Vault Master Key is
  a separate key family and grants no module-promotion authority from ADR 0007.
- M13 in `docs/ROADMAP.md` already fixes the durable-storage direction: a structured,
  crash-consistent, encrypted store on a dedicated raiOS partition, never the boot
  stick, ESP, Windows partition, or immutable recovery core. No M13 structured-store
  runtime exists yet; the Secret Vault block must build the real required slice rather
  than write encrypted blobs into RECLOG.
- AES-GCM 0.10.3, HKDF 0.12.4, and zeroize 1.8.2 are already locked transitively, but
  are not direct raios-core/seed-kernel dependencies today.
- Commit `cf323a7` extends the guided WiFi flow through a bounded Marvell association,
  firmware-supplicant/PMK path, `PORT_RELEASE`-gated WPA2 link, PFU RX/TX and smoltcp
  DHCP attachment. It is host/QEMU verified by
  `shadow-20260710-010658-31684.json` (542/542), while the positive Surface radio/
  `PORT_RELEASE`/DHCP proof remains open. Genesis and Vault work must preserve this
  exact implemented-vs-physically-proven distinction and its RAM-only credential path.
- `seed-kernel/src/recovery_lifeline.rs` is the dedicated first dispatch path. Its real
  surface includes `recovery.snapshot`, bounded `recovery.disable_module`,
  `recovery.restart_last_good`, and exact-hash local recovery loading. Rollback apply
  remains denied where the retained evidence/authority is absent.
- `seed-kernel/src/wasm_runtime.rs` already has per-instance wasmi linkers, fuel,
  memory limits, signed artifacts, exact import-list evidence, and five `env` imports.
  `raios-core/src/scoped_wasm_import_grant.rs` currently denies all imports outside
  `env`.
- `vm-harness/capture-readme-screenshots.ps1` proves HMP `screendump` and PPM-to-PNG
  conversion, but requires a real OpenAI key, enables an unverified provider path,
  kills all QEMU instances, and deletes a target directory. It must not be reused as
  the release acceptance harness without a safe extraction.
- `scripts/update-usb-esp-a.ps1` updates only ESP A and verifies the copied kernel
  hash. It does not reformat `SEED_DATA`. It still needs an explicit whole-layout and
  disk-identity preflight before autonomous use.

### Current truth and dirty-tree baseline

At plan authoring time:

- latest full gate: `release/vm-reports/shadow-20260708-150428-34396.json`,
  `full`, passed 7867/7867;
- latest quick: `release/vm-reports/shadow-20260710-010658-31684.json`, passed
  542/542;
- the following untracked paths are foreign owner work and must not be deleted,
  staged, overwritten, or absorbed into a commit:
  `.cargo-home/`, `release/enum-console-shot.png`,
  `release/iommu-wifi-shot.png`, `release/raios-stage0-preview.img`,
  `release/set-wifi-scan-shot.png`, `release/ui-pill-detail-shot.png`, and
  `release/usb-write-result.txt`.

The former concurrent WiFi association/supplicant work is now frozen in `cf323a7`; the
workspace-wide rustfmt repair is `da6e458`. Those source paths are available again, but
their new association/zeroization/link-loss behavior is a preservation boundary rather
than disposable design code.

Disk 2 was absent during plan authoring and is visible again at I0, but its trusted
`target/usb-handoff/disk2-fingerprint.json` does not yet exist. Final USB work therefore
still fails closed until the exact SanDisk identity/layout/RECLOG evidence is captured
and rechecked; the agent must never trust the number alone.

## 3. Target architecture

```text
L0  Permanent Core
    boot, isolation, capability decisions, TPM/recovery key custody,
    Secret Broker, recovery authority, minimal framebuffer/input/compositor primitives

L1  Genesis + Recovery Shell
    universal and core-owned; conversation, typed context, secure attention,
    vault unlock/provisioning, permissions/secrets, network/provider setup,
    recovery workshop

L2  Personal Shell
    signed replaceable Wasm service in slot svc.user.shell;
    unique UI and interaction inside a bounded personal surface

L3  Personal Services
    tools, apps, provider adapters, builders and later user extensions
```

### Boot and failure routing

```text
Core boots
  -> identify the dedicated raiOS data partition read-only
  -> Genesis starts and reads real typed facts
       -> valid TPM wrapper: unlock VMK into a non-exportable core handle
       -> TPM policy unavailable/mismatch: offer the high-entropy recovery key
       -> missing/foreign/corrupt store: stay RAM-only and report the exact denial
       -> no personal shell: remain in Genesis
       -> healthy approved personal shell: user may enter it
       -> personal shell traps, exceeds fuel, or becomes unhealthy: return to Genesis
       -> SAFE/probation/problem entry: Genesis opens its Recovery context
```

Genesis is not a conventional desktop. It is the system workshop and the dependable
surface from which a user creates, repairs, or replaces the personal shell.

The current direct OpenAI implementation may remain during this block to preserve a
working vertical path, but it is a transitional normal provider service. Genesis may
request AI work through it; it may not turn TLS/HTTP/provider logic into permanent
recovery-core identity. The core continues to own trust labels, key custody,
redaction/export decisions, and final action authorization. Offline recovery remains
fully usable.

Genesis never stores a password itself. It sends a secret-setting intent to the
core-owned Secret Broker. The broker encrypts only after store, key, policy, audit,
and readback gates authorize the exact operation. Normal boot may auto-connect an
approved WiFi profile after a valid unlock. SAFE/recovery may unlock the same vault,
but outbound connection requires one explicit trusted Genesis action; no personal
shell or provider response can trigger it.

### Trusted visual ownership

- Genesis and recovery are drawn by the core-owned `ShellHost`.
- The core owns the top secure strip, secure-attention input, secret entry,
  permissions, recovery confirmation, and fatal-error surfaces.
- Secret plaintext is accepted only by `secure_overlay`, copied into one bounded
  broker buffer, consumed by an exact typed operation, and zeroized. Shell state,
  screenshots, diagnostics, context and ordinary memory records hold only secret IDs
  and states such as `missing`, `locked`, `available`, or `denied`.
- A personal shell receives no framebuffer pointer. It submits a bounded display list
  to a validator and compositor.
- The personal surface is clipped below the secure strip. Trusted overlays are drawn
  after the personal frame and cannot be occluded.
- AI/provider output is untrusted text. It can propose actions but cannot invoke
  mutations without the existing capability/recovery gates.
- UI facts are projections over existing typed system sources. The renderer must not
  build a second health, capability, service, provider, or recovery truth store.
- Recovery serial commands and Genesis buttons are peer adapters over one shared typed
  snapshot/action executor. The UI must not call a text/serial dispatcher or
  reimplement an evaluator to perform an action.

## 4. Universal Genesis design specification

The default 1280x800 composition is deliberately quiet: graphite, off-white text, one
blue accent, and amber/red only for states that need attention. No rainbow status
tiles, terminal-green decoration, mascot, animation, theme engine, wallpaper, dock,
or fake app grid belongs in Genesis.

```text
┌ raiOS · Genesis                       Core safe · Recovery ready ┐
│                                                                 │
│ Conversation                              Context               │
│                                                                 │
│ Welcome. What should your raiOS become?   Personal shell        │
│                                           Not created           │
│                                           AI connection         │
│                                           Ready / needs key     │
│                                           Network               │
│                                           Connected / detected  │
│                                           Secret Vault          │
│                                           Ready / locked / none │
│                                           Problems              │
│                                           0 critical            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Describe what raiOS should become…                         [↑]  │
└─────────────────────────────────────────────────────────────────┘
```

### Recovery projection of the same shell

```text
┌ raiOS · Genesis                         Recovery · SAFE/ready ┐
│ Conversation                              Recovery context    │
│ Ask what happened…                        Current problem     │
│                                           Last known good     │
│                                           Crashed services    │
│                                           Disabled modules    │
│                                           Available actions   │
│                                           Restart last good   │
│                                           Disable module      │
│                                           Load local hash     │
├───────────────────────────────────────────────────────────────┤
│ Ask about this problem…                                      │
└───────────────────────────────────────────────────────────────┘
```

Only actions that the existing lifeline can really evaluate are shown as available.
A denied rollback or absent artifact remains visible as unavailable with a short
reason; it must not become a decorative enabled button.

### Layout and typography

- At 1280x800 and similar modes, use logical scale 2 so the existing 8x8 font renders
  as readable 16-pixel glyphs. Use a compact 640x400 logical layout.
- Conversation receives about two thirds of the content width; Context receives one
  third. On a narrow mode, Context moves below Conversation rather than truncating
  critical status.
- Use 16 logical pixels of outer margin, 12 between panels, a 36-40 high secure strip,
  and a 44-52 high composer.
- Body line height is at least 12 logical pixels at scale 2. Avoid all-caps except tiny
  fixed status words.
- Context shows no raw boot log, provider response, secret, full device topology, or
  long technical reason. Diagnostics remains one explicit expert view.
- If input or USB enumeration is degraded, the Context problem row exposes the same
  compact real `ENUM`/completion-code evidence that currently makes the console-first
  Surface boot photograph useful; one action opens full Diagnostics. The redesign may
  change presentation, not remove the no-input debugging signal.
- Focus is always visible. Tab/Shift-Tab, Enter, Escape, pointer, and serial behavior
  remain usable.
- F12 is the secure-attention key and always returns from a personal shell to Genesis.
  The core-owned secure strip also exposes a pointer target for Genesis/Recovery.

### Interaction model

- First boot focuses the composer. There is no empty default desktop and no skip into
  a fake personal shell.
- Clicking AI connection opens the existing trusted provider/API-key overlay.
- Clicking Network/WiFi invokes the existing real guided WiFi flow.
- Clicking Secret Vault opens only core-owned actions: unlock, save/replace a WiFi or
  provider credential, forget a credential, or show recovery-key status. It never
  offers reveal/copy/export plaintext.
- Clicking Recovery or pressing F12 opens the recovery context.
- Diagnostics is secondary and calm, not a top-level console tab. Serial commands and
  the full expert output remain available.
- API keys and WiFi passwords are entered only in core-owned masked overlays. They are
  never forwarded to the personal shell or rendered into context.

## 5. Modular source target

The redesign must leave stable ownership boundaries, not merely move a monolith. Use
this target unless Slice 0 proves a concrete compiler/module constraint and records a
mechanical correction in this plan.

Modularity is part of Definition of Done, not a cleanup phase. Every lane creates code
in its intended long-lived module from its first source commit. Do not accumulate a
temporary second `ui.rs`, `main.rs`, `lib.rs`, `console.rs`, `misc.rs`, `helpers.rs` or
`utils.rs` monolith and promise to split it later. Composition roots stay short; domain
logic, pure policy, storage encoding, hardware transport, presentation and harness
logic keep separate owners. Cross-module calls use the smallest concrete typed
interface that already serves the real path—no generic framework or one-implementation
factory merely to appear modular.

```text
seed-kernel/src/
  ui.rs                    tiny compatibility shim: RuntimeStatus re-export only
  shell_host/
    mod.rs                ShellHost facade, mode transitions, framebuffer ownership
    state.rs              focus, composer, chat/current-boot interaction state
    context.rs            read-only projections of typed system/provider/service facts
    genesis.rs            Genesis composition; no driver/provider mutation
    diagnostics.rs        expert console presentation only
    wifi_flow.rs          existing guided WiFi controller and visual flow
    secure_overlay.rs     provider/WiFi secret, permission and recovery overlays
    recovery.rs           read-only recovery view model + shared typed action adapter
    personal_surface.rs   validated personal frame, clipping and fallback

  structured_store/
    mod.rs                M13 mount/index/append facade over one approved data region
    replay.rs             bounded scan, transaction recovery and index rebuild

  secret_vault/
    mod.rs                tiny core-owned facade and lock/unlock state
    keyring.rs            VMK handle, TPM/recovery wrappers and key epochs
    store.rs              encrypted record/tombstone transactions only
    broker.rs             exact-purpose secret set/use/forget leases and zeroization

  tpm2_transport.rs       bounded CRB/TIS command transport; no policy decisions

  personal_shell_service.rs
                           one-shot Wasm invocation, health/trap and dynamic lifecycle
  system_problem_facts.rs pure ProblemFact projection shared by protocol and ShellHost

  console.rs              serial command parser/output adapter; no framebuffer layout
  framebuffer.rs          unchanged drawing/storage primitive
  text.rs                 unchanged glyph primitive unless a proven small fix is needed
  input.rs                unchanged device queue plus the minimal F12 secure-attention map

raios-core/src/
  genesis_layout.rs       pure responsive geometry/hit-testing with host tests
  ui_frame.rs             no_std display-list decoder, limits, validation and tests
  scoped_wasm_import_grant.rs
                          exact per-service UI import authorization
  structured_store.rs     typed on-medium frames/transactions and pure validators
  secret_vault.rs         encrypted envelope/AAD/version/nonce/key-epoch typed model
  scoped_secret_use.rs    pure pairwise-denial policy evaluator
  tpm2_commands.rs        pure TPM2 command encoding/response validation

wasm-guests/
  svc-personal-shell-proof/
                          minimal signed proof guest, clearly test infrastructure
```

`main.rs` talks only to the `ShellHost` facade. It must not learn Genesis panel
geometry, WiFi dialog state, display-list opcodes, or personal-shell details.
Keep the current event-loop call shape during the migration (`new`, normal/forced
render, pointer-only render, pointer interaction) so the redesign does not also
rewrite scheduling. Keep `seed-kernel/src/ui.rs` as a tiny non-visual compatibility
shim that only re-exports `system_status::RuntimeStatus`; many provider/protocol files
currently name `ui::RuntimeStatus`, and touching them would violate the preservation
boundary. New `ShellHost` code uses `system_status::RuntimeStatus` directly. The shim
is not a renderer or fallback and may be removed only in a later cross-service cleanup.

The interactive UI state currently mixed into `console.rs` moves into
`shell_host/state.rs`. Serial parsing and command execution stay in `console.rs`; the
serial console becomes an adapter to shell state, not the owner of the visual shell.
This removes the current `ui <-> console` ownership cycle.

The current renderer/controller body in `ui.rs` is removed after the compatibility
transition; only the short `RuntimeStatus` re-export remains. Do not leave the old and
new renderers alive behind a silent fallback.

`system_problem_facts.rs` owns the pure reusable `ProblemFact` projection. Both the
existing `problem.list` emitter and `ShellHost` consume it; the renderer must not copy
the serial-emitter conditions. Recovery BOOTCTL facts are refreshed once on recovery
entry, explicit refresh, or a relevant state transition and cached as a redacted view.
Never call the real bounded AHCI `current_boot_last_good_view()` on every redraw.

The Secret Vault has exactly four kernel modules because their trust boundaries differ:
`keyring` never parses store records, `store` never decides consumers, `broker` never
writes raw media, and `mod.rs` only composes their typed results. Do not create a
generic credential framework, plugin API, secret Wasm import, or a second persistence
backend. `structured_store` is the single M13 backend shared later by memory/artifacts;
the vault is merely its first security-sensitive namespace.

### Driver exception

Do not refactor or split hardware drivers merely to satisfy an aesthetic module count.
Register sequencing, DMA/ring ownership, interrupt state and timing-sensitive bring-up
may stay together while the behavior is still being discovered. Stable pure protocol
pieces such as command builders, parsers or state machines may be extracted when that
already improves host testing, but the Genesis/Vault lanes must not reorganize drivers
as collateral work. `shell_host/wifi_flow.rs` is UI/controller code and is not covered
by this exception. The exception relaxes the soft module-size target only; the hard
AGENTS.md touched-file limit and required split plan still apply.

Soft size target: each new module should normally stay below 800-1,200 lines. A touched
module over 1,500 lines needs an immediate boundary review; no touched `.rs` file may
end above 5,000 lines without the documented split plan required by `AGENTS.md`.

Every Terr completion packet reports its module boundary, public surface and resulting
line counts. The orchestrator rejects a capability as incomplete if its behavior works
only because unrelated responsibilities were placed in a shared root or catch-all
module. Conversely, do not fragment cohesive code into tiny forwarding files: split at
ownership, trust, lifecycle or test boundaries, not arbitrary line counts.

### Honest service inventory transition

`service_inventory.rs` currently calls statically linked `svc.ui.framebuffer`
replaceable. Do not preserve that fiction after the real boundary exists:

- after G1/G2, add or rename the real core-owned non-replaceable Genesis/ShellHost
  entry only when the implementation actually exists;
- after G4, expose the replaceable personal shell only in dynamic/current-boot inventory while
  a real `svc.user.shell` instance is loaded or running. The release default
  `personal_shell: not_created` must not acquire a fake static service row;
- never claim that the old monolithic renderer and the new personal service are both
  independently isolated when they are not.

## 6. Personal-shell Wasm ABI and fail-closed boundary

This section is the complete owner-approved import expansion. Any additional import
requires a new decision.

Stable service slot: `svc.user.shell`.

Exact imports:

| Module | Name | Exact Wasm signature | Purpose |
|---|---|---|---|
| `ui` | `viewport` | `() -> i64` | Packed unsigned logical width in high 32 bits and height in low 32 bits. |
| `ui` | `context_len` | `() -> i32` | Exact byte length of the immutable redacted context packet staged for this invocation. |
| `ui` | `context_read` | `(ptr: i32, cap: i32) -> i32` | Copy the entire staged context packet and return its byte length. |
| `ui` | `input_len` | `() -> i32` | Exact byte length of the immutable sanitized input packet staged for this invocation. |
| `ui` | `input_read` | `(ptr: i32, cap: i32) -> i32` | Copy the entire staged input packet and return its byte length. |
| `ui` | `frame_submit` | `(ptr: i32, len: i32) -> i32` | Copy and atomically validate one display list; return a V1 result code. |

No `secret.*`, network, raw block, raw input-device, framebuffer, pointer, DMA, time,
provider, recovery, capability-decision, or generic host-call import is granted.

Invocation semantics are V1-final for this block:

- the runtime stages context and input once before entering Wasm; both byte slices are
  immutable until that invocation ends, so `*_len` and `*_read` cannot observe
  different states;
- V1 context is exactly 32 bytes; input is at most 1,040 bytes (16-byte header plus
  64 fixed events); `cap` must be at least the exact staged length;
- negative pointers/lengths, integer overflow, insufficient guest capacity, or a
  guest-memory bounds failure trap the invocation before any host state changes;
- each of the six imports may be called at most once per invocation; a successful
  invocation must call `frame_submit` exactly once;
- `frame_submit` copies guest bytes into host scratch before validation and never
  renders from live guest memory;
- result `0` means accepted; `-1` means wrong ABI version; `-2` malformed/truncated or
  invalid UTF-8; `-3` a declared hard limit was exceeded; `-4` a second submit was
  attempted. A negative result leaves no pending frame, and an ignored rejection is
  still reported by the host as `frame_rejected`, not success;
- every render/input event uses a fresh stateless one-shot Wasm invocation. V1 claims
  no persistent guest-instance state. The proof changes its frame from the staged
  input packet, not from hidden retained memory;
- each invocation receives a fixed 250,000 wasmi fuel budget. Native import work is
  bounded by the one-call rule and the byte/command caps above; fuel exhaustion or a
  seventh/repeated host call rejects the frame and returns to Genesis.

All packet integers are little-endian. Reserved fields and command padding must be
zero or validation fails. V1 packets are frozen as follows:

```text
Context (32 bytes)
  0  [4]  "RCTX"
  4  u16  version = 1
  6  u16  length = 32
  8  u32  invocation_id
 12  u16  viewport_width
 14  u16  viewport_height
 16  u16  service_count
 18  u16  problem_count
 20  u16  denied_capability_count
 22  u16  flags: bit0 personal_focus, bit1 recovery_ready; all others zero
 24  u32  active_task_id (stable typed id, 0 = none)
 28  u32  reserved = 0

Input header (16 bytes), followed by 0..64 fixed 16-byte events
  0  [4]  "RINP"
  4  u16  version = 1
  6  u16  header_length = 16
  8  u32  invocation_id (must equal Context)
 12  u16  event_count
 14  u16  reserved = 0

Input event (16 bytes)
  0  u8   kind: 1 key, 2 pointer_move, 3 pointer_button
  1  u8   flags: bit0 pressed, bit1 repeat; remaining bits zero
  2  u16  sanitized key/button code
  4  i16  x
  6  i16  y
  8  i16  dx
 10  i16  dy
 12  u16  sanitized modifiers
 14  u16  reserved = 0

Frame header (16 bytes), followed by aligned commands
  0  [4]  "RFRM"
  4  u16  version = 1
  6  u16  header_length = 16
  8  u32  total_length
 12  u16  command_count
 14  u16  flags = 0

Command prefix (4 bytes)
  0  u8   opcode
  1  u8   flags = 0
  2  u16  payload_length
  ...     payload, then zero padding to a 4-byte boundary

Opcodes
  1 CLEAR:       rgba u32
  2 FILL_RECT:   x,y,w,h u16 + rgba u32
  3 STROKE_RECT: x,y,w,h u16 + rgba u32
  4 TEXT:        x,y u16 + rgba u32 + text_len u16 + reserved u16 + UTF-8 bytes
  5 FOCUS_HINT:  x,y,w,h u16
```

F12/secure-attention is consumed before an Input packet is staged and can never be
observed or blocked by the guest. `EnvelopeState` gains an optional
`PersonalShellInvocation` containing viewport, immutable context/input bytes, a
six-bit call ledger, and one pending validated frame. Other services never receive
that state or the UI imports.

### Display-list V1

The V1 decoder in `raios-core::ui_frame` supports only:

- clear personal surface;
- filled rectangle;
- one-pixel rectangle outline;
- UTF-8 text from the existing bounded font set;
- optional focus/cursor hint constrained to the personal surface.

Hard limits are constants tested on the host and rechecked in the kernel:

- maximum frame bytes: 16 KiB;
- maximum commands: 256;
- maximum total text bytes: 4 KiB;
- maximum one text run: 512 bytes;
- all arithmetic checked for overflow;
- unknown ABI version/opcode, truncated payload, invalid UTF-8, impossible dimensions,
  or excess limit rejects the whole frame before drawing;
- every coordinate is clipped to the personal surface;
- no command can address the secure strip or trusted overlay layer;
- no partially validated frame is presented;
- the last known good personal frame may remain visible only while the service is
  healthy; a trap/fuel/memory/validation failure returns to Genesis.

The context/input packet is a small versioned ABI type implemented in the typed model,
not hand-built JSON. It contains only the facts required by the proof:

- viewport and focus state;
- sanitized key/pointer events routed to the personal surface;
- coarse service/problem/capability counts and stable IDs already classified for
  local display;
- no raw prompts, provider output, key material, WiFi password, raw boot log, or
  unclassified memory.

### Grant policy

- Extend `KNOWN_HOST_IMPORTS` with exactly the six pairs above.
- `policy_allows_beyond_env` must not become a broad global `true`.
- Replace the current boolean-only artifact pin in the scoped evaluator. Its decision
  input and output must bind the concrete service id, artifact SHA-256, verified
  descriptor-source/signature evidence hash, verified artifact-signature/attestation
  evidence hash, computed grant/evidence hash where that path supplies one, and the
  exact ordered import-list hash. `artifact_sha256_present:true` alone is never enough
  for `svc.user.shell`.
- The scoped evaluator authorizes the exact ordered UI list only for
  `svc.user.shell`, only with those verified and mutually bound signed artifact/load
  facts, and only when every requested import has a concrete linker implementation.
  The specific artifact may change in the future only through a new verified decision
  carrying its new concrete hash; no service-wide bearer boolean is retained.
- Existing services retain their exact current import lists and hashes unless the
  required hash change is intentionally regenerated and proven.
- A subset, superset, duplicate, reorder, wrong service id, wrong artifact binding,
  missing implementation, or undeclared import denies before instantiation.
- The actual linked list and authorized list come from the same evaluator output and
  remain byte-identical in evidence.

### Proof artifact

Add a checked-in signed `svc.user.shell` proof guest through the same build,
descriptor, signature-envelope, artifact-hash, wasmi, fuel, and service lifecycle
path used by current signed guests. Label it `current_boot` test infrastructure with
`trust_tier: dev_key_not_owner_sealed`, `owner_sealed:false`, no external byte intake,
no persistent install, and no provider auto-load. The proof must:

- render a visibly distinct personal surface;
- receive one sanitized input event and update its frame;
- fail when `ui.frame_submit` is not granted;
- fail when asking for one unknown/broader import;
- submit an out-of-bounds command that is clipped or rejected as specified;
- submit a malformed frame that is rejected atomically;
- intentionally trap so the VM profile proves automatic return to Genesis;
- never become the default user's personal shell on the release stick.

Do not overload `env.output_write` as display authority. A generic captured byte
buffer and a validated visual compositor are different trust surfaces.

## 7. Persistent Secret Vault and durable storage boundary

Capability sentence: after one Genesis setup, raiOS can persist exactly a WiFi
passphrase and provider API key as authenticated ciphertext, unlock them after reboot
through a TPM-bound or high-entropy recovery-key wrapper, and let only their exact
trusted consumers use them without revealing plaintext to Genesis state, AI, the
personal shell, logs, or ordinary memory.

This is not a UI convenience slice. It completes the necessary M13 structured-store,
encryption-at-rest, and key-custody path. Every positive write/unlock/use claim needs
real store, crypto, reboot and denial evidence.

### Non-negotiable boundaries

- Genesis is a secure input and status adapter; it is not the vault.
- The vault stores only two V1 secret kinds: `wifi_passphrase` (maximum 63 bytes) and
  `provider_api_key` (maximum 256 bytes). Adding cookies, tokens, arbitrary blobs,
  SSH keys, chat, or personal-shell secrets needs a later decision.
- Ordinary `raios.memory_record` continues to reject `secret`. Do not weaken or add a
  fallback around `secret_never_durable_until_sealed_secret_design`.
- The target is the dedicated M13 raiOS data partition approved in the roadmap. The
  boot stick, `SEED_ESP_A/B`, Windows, foreign GPT partitions, RECLOG, ARTSTOR and the
  immutable recovery core are never vault targets.
- Missing/foreign markers, corrupt superblocks, ambiguous devices, unavailable write
  authority, locked keys, bad tags, stale versions, or policy mismatch keep the
  relevant operation denied and the current RAM-only behavior intact.
- The default release image and tracked files contain no secret, recovery key, VMK,
  TPM private blob unique to a real installation, or usable development vault key.
- The Vault Master Key is distinct from the ADR 0007 promotion-authority key and
  grants no code load, capability, storage-region, recovery, or provider authority.

### One real structured store

Build the smallest durable M13 backend that remains on the final path: an append-only,
log-structured store on the dedicated marked partition. It is shared infrastructure,
not a secret-specific raw region.

```text
partition identity + dual hash-checked superblock copies
  -> bounded append segments
       -> PREPARE frame
       -> DATA frame(s)
       -> COMMIT frame
       -> later TOMBSTONE for deletion
  -> replay accepts only hash-linked, readback-verified committed transactions
  -> in-RAM index rebuilt from committed frames after every boot
```

Required properties:

- exact GPT type/partition identity and store UUID rechecked before every write;
- two superblock copies with version, geometry, generation, unkeyed SHA-256 corruption
  check and active
  selection; neither copy is overwritten until its replacement was flushed/read back;
- monotonically increasing 64-bit transaction and record versions with overflow
  denial;
- frame length, namespace, record id, version, previous-frame hash, payload hash and
  CRC/hash validation before allocation or indexing;
- append -> flush -> readback -> reparse -> commit; no response says `stored` before
  the committed replay sees the exact ciphertext envelope;
- torn PREPARE/DATA without COMMIT is ignored after reboot; a corrupt committed chain
  locks the affected namespace and never falls back to older unverified bytes;
- `vault` is one namespace. Memory/artifact migration can use the same backend later,
  but this plan does not duplicate those systems or claim that migration complete;
- capacity exhaustion returns an explicit denial. Initial append-only/no-compaction is
  acceptable because it is the final log format; add garbage collection only when
  measured capacity requires it.

The superblock hash detects corruption; it is not called a signature or an authority
proof. After unlock, AEAD tags plus the committed hash chain authenticate Vault records
under the VMK. No unnamed signing key or plain hash is allowed to masquerade as store
authenticity.

### Cryptographic envelope V1

Use the already locked RustCrypto implementations, directly pinned with minimal
`no_std` features:

```text
AEAD       AES-256-GCM, 128-bit tag
KDF        HKDF-SHA-256
zeroize    zeroize every VMK/DEK/plaintext/temp request buffer
VMK        32 random bytes from core entropy, never written plaintext
nonce      12 random bytes, stored in the envelope, duplicate-rejected per key epoch
DEK        HKDF(VMK, store_uuid || key_epoch || secret_id || record_version || kind)
```

Each encrypted record is a typed record-model entry, not hand-emitted JSON. Its AAD
binds at least:

```text
schema/version
store_uuid
key_epoch
secret_id
secret_kind
consumer_id
allowed_operation
target binding (SSID/BSSID hash or exact provider host)
record_version
plaintext_length
previous committed record hash or null
```

The envelope stores the AAD hash, nonce, ciphertext and 16-byte tag. Plaintext is
accepted only from the core-owned secure overlay, is never serialized before
encryption, and is zeroized whether encryption, append, readback or policy succeeds or
fails. Decryption copies into one fixed-capacity broker lease; it never returns a
general `Vec<u8>`, string, debug representation or reveal API.

Nonce/key rules:

- every secret update gets a strictly larger committed version and therefore a new
  HKDF-derived DEK;
- the 96-bit random nonce comes from the same ready entropy source, is checked against
  retained envelopes in the epoch, and duplicate/entropy failure denies before
  encryption;
- an uncertain/torn version is never reused; sequence gaps are valid;
- rotate `key_epoch` before the per-key invocation ceiling, on owner reset, or after a
  suspected nonce/key compromise; never shorten the 128-bit tag.

AES-GCM is chosen because its implementation is already locked through
`embedded-tls`; the implementation still follows NIST's authenticated-encryption and
unique-IV requirements. Do not write AES/GHASH/HKDF manually.

### Keyring and unlock model

The VMK has no plaintext persistent form. `secret_vault/keyring.rs` manages only an
opaque in-RAM handle and two wrapper families:

1. **TPM wrapper for automatic normal-boot unlock.** Implement a bounded TPM2 CRB/TIS
   command transport from the already parsed ACPI interface. Seal the 32-byte VMK to a
   policy representing the approved machine plus current/next/last-good core
   generations. Persist only the TPM public/private sealed blobs, policy digest and
   evidence hashes. A positive `vault_vmk_tpm_sealed` /
   `tpm_vmk_wrapper_ready` / automatic-unlock claim requires a real successful
   create/load/unseal cycle and readback-bound wrapper record; ACPI table presence or a
   status-register read is not sealing evidence. ADR 0007 `owner_sealed` is a separate
   promotion-authority ceremony and is never changed by a Vault VMK wrapper.
2. **Recovery wrapper.** Generate a separate random 32-byte recovery key during first
   Genesis provisioning, show it exactly once through a core-owned surface, and never
   store it. HKDF derives a recovery KEK that AES-GCM-wraps the same VMK with store/
   epoch/policy AAD. This is a high-entropy recovery key, not a human password; adding
   password unlock waits for a reviewed memory-hard KDF.

Recovery-key V1 presentation is dependency-free and exact:

```text
RR1-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-cccc
```

The eight `xxxxxxxx` groups are the 32 random bytes in hexadecimal; `cccc` is the first
two bytes of SHA-256 over `"raios-recovery-v1" || key`. Input accepts upper/lowercase
hex but requires the exact prefix, group counts, separators and checksum. Genesis shows
it once, requires one complete re-entry confirmation before committing the wrapper,
then zeroizes display/input/parser buffers. This is typo detection, not extra entropy or
a password KDF.

Normal boot may automatically unseal and reconnect the last approved WiFi profile.
If the TPM policy changed unexpectedly, Genesis asks once for the recovery key, never
for every stored credential. During an approved core update, create and readback-
verify the next-generation wrapper before selecting the new slot; retain the
last-good wrapper until boot success. A rollback changes the core wrapper, not the
latest committed secret versions.

SAFE mode may unlock through a valid last-good/recovery wrapper, but it does not make
an outbound connection automatically. One explicit Genesis Recovery action may use a
known profile through the same broker policy. If no wrapper succeeds, local recovery
remains usable and all secret-dependent network/provider operations stay denied.

Recovery-wrapper HKDF salt/info, nonce and AAD use the exact canonical byte encoding in
ADR 0012. A fresh stored 96-bit nonce is mandatory for every wrapper, and reuse under
one recovery KEK denies before encryption.

### Exact broker policy

V1 has no generic `get_secret` function. The pure
`raios-core::scoped_secret_use` evaluator accepts only:

| Secret kind | Exact consumer | Exact purpose/target | Result |
|---|---|---|---|
| `wifi_passphrase` | trusted native WiFi supplicant path | associate the bound SSID/BSSID/security profile | one bounded ephemeral lease, then zeroize |
| `provider_api_key` | `svc.provider.openai_direct` transition path | one already trust-authorized request to exact host `api.openai.com` | append/use credential in the bounded request path, then zeroize |

Wrong kind, consumer, operation, target hash/host, boot scope, service generation,
trust decision, record version, key epoch, tag, or audit/store evidence yields a
pairwise-unique denial before decryption where possible and always before consumer
use. Personal shells, AI output, diagnostics, serial, recovery artifact loaders and
Wasm receive no secret import or raw lease.

Set/replace, forget and recovery-key unlock originate only from explicit trusted
Genesis/Recovery actions. Normal-boot TPM unlock may occur automatically only after the
exact positive TPM-wrapper policy/evidence gate; no other automatic unlock path exists.
Plaintext entry exists only in the core-owned secure overlay and is bounded to 63-byte
WiFi passphrases or 256-byte provider keys. Personal shells, provider output, AI output
and ordinary services cannot invoke these mutations.

The current WiFi supplicant builder and direct provider path may receive the bounded
plaintext only inside their final native call after a positive evaluator decision.
Longer-term opaque crypto/header operations may reduce exposure further, but do not
invent a fake opaque service during this block. The driver/provider buffers are
zeroized immediately after the command/request bytes have been handed to their
existing protected transport.

### Secret lifecycle and rollback

- `set`: encrypt -> append/readback -> commit -> re-decrypt/tag-check -> publish the
  new version; until then the previous committed secret remains active.
- `use`: authorize exact purpose -> decrypt to bounded lease -> consume once ->
  zeroize -> audit only ids/hashes/outcome.
- `forget`: append a committed tombstone, evict any lease, zeroize RAM; old ciphertext
  remains cryptographically inaccessible after key-epoch destruction/compaction.
- system rollback does not roll credentials back. Vault versions are monotonic and
  independent from personal-shell/core last-good selection; only key wrappers follow
  current/next/last-good boot generations.
- audit never stores plaintext, ciphertext, nonce, tag, raw SSID, Authorization
  header or recovery key. It records stable ids, versions, evidence hashes, consumer,
  purpose and outcome.

### Required evidence

The dedicated `secret-vault` focused profile uses a separate marked QEMU data disk and
fake sentinel credentials only:

1. provision VMK and recovery wrapper, store both V1 secret kinds, reboot, unlock with
   the recovery key through real Genesis input, and prove the exact consumers can use
   them without re-entering the individual credentials;
2. keep the sentinel absent from serial logs, screenshots, reports, audit, context,
   crash output and provider request-envelope diagnostics;
3. prove wrong consumer/purpose/host/BSSID, personal-shell request, locked vault,
   wrong recovery key, stale version, duplicate nonce, missing commit, torn frame,
   changed ciphertext/tag/AAD, foreign partition and corrupt superblock all deny;
4. power-cut between PREPARE/DATA/COMMIT and prove reboot selects only the previous
   committed secret;
5. crash the personal shell, WiFi and provider adapters independently and prove the
   core vault remains available without leaking or widening the lease;
6. run recovery profile after unlock and prove SAFE requires explicit connect while
   local recovery remains cloud-independent;
7. prove the default release image and boot stick contain no sentinel or vault key.

QEMU currently has no TPM device/tooling, so recovery-wrapper reboot is the mandatory
automated positive path. TPM auto-unlock may be claimed complete only after either a
real swtpm-backed profile using the same command transport or a real Surface seal/
reboot/unseal capture succeeds. Until then the UI says `Vault: recovery unlock` and
never `TPM auto-unlock ready`.

Primary specifications for Slice 0: the official
[TCG TPM 2.0 Library](https://trustedcomputinggroup.org/resource/tpm-library-specification/)
for command/policy structures and
[NIST SP 800-38D](https://csrc.nist.gov/pubs/sp/800/38/d/final) for GCM/GMAC.

## 8. Preservation matrix

| Existing capability | Required result after redesign | Primary proof |
|---|---|---|
| Limine/UEFI framebuffer boot | Exact release image reaches Genesis at 1280x800. | build, package, QEMU screenshot |
| e1000 + IPv4 DHCP | Existing configured QEMU network marker remains. | final `quick -Network` serial predicates |
| Direct provider path | Existing no-pin fail-closed path remains; optional pinned temp-image smoke only if local env already exists. | quick/full; optional `openai-direct-smoke` |
| API-key entry | Reachable from Genesis and masked. Before Vault unlock it remains honest RAM-only state; after an authorized save it is referenced only by typed secret ID and survives reboot as authenticated ciphertext. | Genesis + `secret-vault` profiles |
| Marvell firmware/HW_SPEC/live scan | Existing code untouched except the mechanically moved UI flow; real results still render. | `raios-core` Marvell tests + current bare-metal evidence |
| WiFi completion honesty | Preserve the exact association/link/DHCP truth that exists when G5 begins; Vault integration must not upgrade a denial into success. A saved passphrase permits only the same scoped supplicant attempt. | WiFi host tests + `quick -Network` + `secret-vault` |
| USB keyboard/mouse + serial | Tab, Enter, Escape, pointer and serial remain; F12 is added as secure attention. | Genesis profile + quick |
| Recovery lifeline | Same dedicated dispatch and authorization gates; UI does not bypass them. | recovery + Genesis profiles |
| Ordinary system memory | Existing `secret_never_durable_until_sealed_secret_design` denial remains. Secret Vault is a separate typed namespace and does not turn `memory_record` into a secret database. | host tests + full |
| Structured persistent store | One identity-checked, crash-consistent M13 backend; torn or uncommitted frames never become authoritative. | `structured-store` focused profile |
| WiFi/provider secret persistence | Only authenticated ciphertext is durable; a two-boot fake-secret test proves save, unlock and scoped use without plaintext exposure. | `secret-vault` focused profile |
| Key custody | Recovery-key unlock is real and automated. TPM auto-unlock is claimed only after real TPM command transport and positive VM or hardware evidence. | `secret-vault`; optional TPM-focused evidence |
| Persistence/rollback stores | Existing recovery/rollback stores keep their authority. System rollback does not silently roll credentials back or weaken Vault monotonicity. | full/recovery + `secret-vault` |
| Personal shell | Runs only through signed Wasm/import/compositor boundary; trap returns to Genesis. | Genesis focused profile |
| Prepared USB data | ESP A only refreshed; ESP B and `SEED_DATA/RECLOG` identities unchanged and contain no Vault records, wrappers, keys or secret ciphertext. | pre/post partition identity + hash log + secret scan |

The design lanes must not edit `seed-kernel/src/net.rs`,
`seed-kernel/src/e1000.rs`, `seed-kernel/src/openai.rs`, the TLS/parser sources,
`seed-kernel/src/marvell_wifi_pcie.rs`, `seed-kernel/src/wifi.rs`,
`seed-kernel/src/usb.rs`, storage/rollback sources, or provider-trust sources. If a
real compile/API need crosses that boundary, the worker stops and the
orchestrator decides whether it is a narrow integration change or out of scope.
G5 is the sole exception: after the concurrent WiFi-association work is cleanly
integrated, its singleton broker-adapter step may make the smallest reviewed edits to
the WiFi/provider call sites. Storage, driver, provider-trust and rollback authority
remain separate leases; no design worker may edit them opportunistically.

## 9. Agent roles

### Orchestrator: Codex 5.6 xhigh

The orchestrator is the only integrator and truth authority. It:

- reads all startup docs and this entire plan;
- owns the live lease table, architecture invariants, shared-file locks and task
  graph;
- dispatches Sol/Terr/Luna, reviews every full diff, and resolves interfaces;
- alone edits or delegates with a singleton lease for `main.rs`, module roots,
  `build.rs`, attested descriptors/signatures, Cargo files, harness dispatcher,
  storage-driver integration, Secret Vault/TPM authority joins, release image,
  status/roadmap/dashboard, QEMU and USB;
- runs builds, packages, VM profiles, visual inspection, secret scan and USB write;
- classifies every failed VM report before retry;
- keeps three worker slots fed from the ready queue when the runtime exposes four
  total slots; it does not consume a worker-sized feature body itself while a disjoint
  ready packet can be delegated;
- never asks the user for taste or routine design choices covered by this plan.

The orchestrator should write only the smallest singleton joins: module exports,
shared entry points, Cargo/descriptor wiring, authority flips and evidence dispatch.
Feature bodies belong to leased Terr lanes. This is an execution rule, not a new
abstraction or coordination service.

### Sol: scoping and trust review

Use Sol at Slice 0 and before each authority/block close, including ADR 0012 and the
first persistent-secret write. Sol is normally read-only.
It revalidates actual files/symbols against HEAD, tests whether a proposed boundary is
real, identifies trust or authority drift, and returns exact corrections. Sol does not
expand product scope or implement code unless the orchestrator explicitly assigns a
small isolated repair.
Sol is just-in-time: it temporarily takes one worker slot at a named integration
barrier, returns a bounded decision packet, and exits. Do not reserve a permanent Sol
slot while implementation packets are ready.

### Terr: implementation

Terr workers write code. Spawn several only when their leases are disjoint. Every
packet includes:

- one capability sentence;
- exact allowed files/globs;
- named forbidden shared files;
- input/output interface agreed before dispatch;
- smallest local check;
- stop-on-scope-creep and no-fallback rules.

Use Terr `xhigh` for evaluator/compositor/recovery/wasmi/attestation, structured
storage, cryptography, TPM transport and Secret Broker work. Use Terr `high` for
renderer/state/harness mechanical work.

Prefer three long-lived Terr ownership tracks over repeatedly spawning tiny agents:
ShellHost/UI, personal-shell runtime, and Vault/storage. A lead may spawn a helper only
when a free slot exists and the helper receives a strict subset of the lead's file
lease; the orchestrator remains the lease authority.

### Luna: docs and evidence

Luna is an eventually-consistent side lane throughout the long execution. As soon as a
capability commit or report is frozen, Luna may update its completed facts while Terr
workers already build the next disjoint packets. It updates concise completion markers,
`PROJECT_STATUS`, `ROADMAP` only if the cursor really moves, `OWNER_DASHBOARD`, and only
those debugging/bare-metal instructions whose commands changed. It does not edit code,
reinterpret failures, claim pending/hardware success, or make architecture calls.

Intermediate docs need not be polished or narrate every implementation step. They must
remain factually true and point to the commit/report/source evidence. The orchestrator
reviews the final truth set at I4. Luna uses a free slot when available; with four total
slots, rotate one worker slot briefly after a barrier while the other code tracks keep
running—never stop all implementation merely to improve prose.

## 10. Same-workspace parallel protocol

Parallel work is allowed only through exclusive write leases. Keep the lease table in
the orchestrator context; do not add a coordination framework to the repo.

Each lease records:

```text
lane | agent | base HEAD | pre-dispatch git status | exact files/globs |
required local check | state
```

Rules:

- read anywhere, write only leased paths;
- needing another file means stop and message the orchestrator;
- one worker per file, even if edits appear to be on different lines;
- no `git clean`, reset, rebase, checkout of foreign work, or `git add -A`;
- current foreign untracked files remain untouched;
- `main.rs`, `console.rs` during its extraction, module roots, `build.rs`, Cargo files,
  the common Shadow VM harness, docs ledgers, release artifacts, QEMU/ports, Git index,
  and Disk 2 are singleton locks;
- each Cargo-writing/testing lane uses its own ignored
  `CARGO_TARGET_DIR=target/lanes/<lane-id>` so compiler locks and artifacts do not
  serialize unrelated local checks;
- a worker may run its smallest scoped host test while other disjoint source lanes are
  active, but the result is lane-local signal because every build sees the shared
  worktree; package/release builds and all authoritative evidence wait for quiescence;
- Cargo manifest/lockfile or `build.rs` edits require a brief all-lane quiescence
  window because they change every lane's build graph;
- QEMU and serial are always single-run because `-StopExisting` and the serial TCP
  listener can destroy or block another run. A new harness may be authored in parallel,
  but only the orchestrator launches authoritative QEMU evidence;
- non-authoritative local harness work must use a unique ignored run directory and
  unique ports and must stop only its own PID.

Because all workers share one worktree, a build sees every lane's uncommitted work.
Parallel work therefore stays continuous between short integration barriers:

```text
dispatch every ready disjoint packet
  -> lane-local checks in isolated target dirs
  -> affected packets reach a named integration barrier
  -> affected lanes quiesce for host-only checks; all source writers briefly quiesce
     for package/QEMU evidence because the shared worktree enters the artifact as one
  -> orchestrator reviews their complete diff and singleton joins
  -> integrated build/focused profile/secret scan
  -> capability commit
  -> immediately refill freed slots from the ready queue
```

The user's tolerance for commit chaos is permission, not a requirement. The existing
playbook makes commits an orchestrator duty, and that remains the safer, evidence-
honest rule. If the runtime truly prevents orchestrator-only commits, serialize a
Git-index lock and stage only exact leased paths; never cherry-pick a commit already
made in the same shared worktree.

Do not wait for every track to finish before integrating one proven capability. Do not
commit unintegrated scaffolding either: each barrier closes a real positive behavior,
and unrelated ready tracks continue immediately afterward.

## 11. Agent-native execution graph

The identifiers below are task-packet IDs, not a command to execute the document from
top to bottom. After G0, the orchestrator schedules from dependencies and available
file leases. G1/G2 form Track A, G3/G4 Track B and their A+B join, G5 Track C, and the
harness packets form Track D across all three. G6 and G7 are final integration
barriers.

```text
I0 / G0: revalidate + freeze contracts
  ├─ A1 / G1: Genesis host ──> A2 / G2: context, overlays, recovery ──┐
  ├─ B1 / G3 foundation: UI frame, grant evaluator, proof guest ─────┤
  ├─ C0 / G5.0: ADR 0012 + Vault/store contracts                    │
  │    ├─ C1 / G5.1: structured store ─┐                            │
  │    ├─ C2 / G5.2: crypto + policy ──┼─> I3 / G5.4 Broker join ─> C5 / G5.5
  │    └─ C3 / G5.3: keyring + TPM ────┘           ▲                │
  └─ D: capture/profile harness packets follow frozen APIs          │
                                                                    │
        A1 + A2 + B1 ─> I2 / G3 runtime authority ─> AB / G4 ───────┤
                                                                    │
        AB + C5 + all focused evidence ─> I4 / G6 release freeze ─> I5 / G7 USB
```

### Scheduling policy

- With four total slots, keep one orchestrator plus up to three active workers. The
  default post-G0 allocation is Terr-A (ShellHost), Terr-B (personal-shell core/guest),
  and Terr-C (Vault/store). Rotate one slot to a Terr-D harness packet whenever its API
  is frozen or another track waits at a join.
- G1 does not have to finish before B1 or C0-C3 starts. G5.1, G5.2 and G5.3 are
  intentionally concurrent after C0; their integration waits, their pure positive
  behavior does not.
- Within one track, obey the arrows. Across tracks, waiting is forbidden when a ready,
  disjoint packet exists.
- Sol temporarily replaces one worker immediately before I2 and I3. Luna may briefly
  replace one worker after any completed capability/report and document it while the
  remaining tracks continue. Neither remains idle in a slot.
- Shared-file joins, authority changes, package builds, QEMU profiles, commits and USB
  remain orchestrator-singleton. This is the small serial spine; all other work should
  overlap.
- At every dispatch or completion, recompute the ready queue from dependencies and
  leases. Do not build a scheduler in the repository; a short table in orchestrator
  context is enough.

### Integration barriers

| Barrier | Opens when | What becomes authoritative |
|---|---|---|
| I0 | ADR 0011, cross-track contracts and current dirty-tree ownership are frozen | agents may implement against named interfaces; C0 then freezes ADR 0012 internals |
| I1 | A1 plus its capture harness are ready | Genesis visibly boots without losing existing setup paths |
| I2 | A2 + B1 + Sol trust review are ready | exact personal-shell imports/runtime may be armed |
| I3 | A2 + C1/C2/C3 + clean WiFi API + Sol review are ready | first Vault write/unlock/scoped-use authority may be armed |
| I4 | AB, C5 and all required focused reports are green | exact release candidate freezes for full regression |
| I5 | I4 evidence and exact Disk-2 fingerprint pass | ESP-A-only USB refresh may occur |

### Parallel documentation cadence

- After each capability commit or focused report, send Luna the capability sentence,
  exact commit, report filename/result and changed command/file pointers. That packet
  is enough; Luna does not reread the entire implementation history.
- Luna writes only completed facts. Work in progress stays in the orchestrator's live
  lease/ready table and is not promoted into authoritative status prose.
- Terr begins the next ready packet immediately; ordinary status/dashboard updates are
  not an integration barrier and may land in the next docs commit.
- ADR acceptance, owner decisions, Red Gate/failure classification and secret/storage
  evidence are exceptions: they must be exact and recorded at their named barrier.
- At I4, reconcile `PROJECT_STATUS`, `ROADMAP`, `OWNER_DASHBOARD` and plan completion
  markers once against commits and report files. This is the only full documentation
  consistency pass before USB handoff.

### I0 / G0 — Revalidate, freeze contracts, record the owner decision

Capability: the build agents have an exact, current, owner-approved shell boundary
and can implement it without guessing trust or file ownership.

Owner: orchestrator with Sol read-only review and Luna docs-only ADR draft.

Tasks:

1. Perform the full `AGENTS.md` startup ritual and `git status --short`.
2. Read the newest reports and apply the Red Gate rule. Classify any unclassified
   failed newest run before doing anything else.
3. Revalidate every path/symbol in sections 2, 5, 6, and 7 against HEAD, including
   the final module home and exclusive owner of each new source file.
4. Record `docs/architecture-decisions/0011-genesis-and-personal-shell-boundary.md`
   with the decisions in sections 1, 3, and 6. Mark it owner accepted by the goal
   prompt, not as a speculative proposal.
5. Freeze the cross-track Rust interfaces between `ShellHost`, shell state, recovery
   projection, secure overlays, personal compositor, Vault status and scoped secret
   requests before parallel code dispatch. C0 may refine Vault-internal interfaces but
   may not make Track A wait or move its ownership boundary.
6. If Disk 2 is present, compare it read-only with trusted prior project evidence:
   `PROJECT_STATUS` identifies the prepared SanDisk as Disk 2 with the raiOS
   ESP-A/ESP-B/SEED_DATA layout and a valid existing RECLOG chain. Record a local
   ignored `target/usb-handoff/disk2-fingerprint.json` containing number, bus/vendor,
   serial/UniqueId where available, size, GPT disk GUID, partition GUID/type/start/
   size, ESP-B identity hash, SEED_DATA superblock hash, bounded RECLOG-region hash,
   and the inspector result. Record only the fingerprint-file SHA-256 in tracked docs.
   Absence is not a blocker for code, only for final USB completion; no unverified
   replacement fingerprint may be invented later.

Checks: targeted doc diff and whitespace check. No VM for docs-only G0.
Commit ADR 0011 and any mechanical map correction as the G0 docs commit before code;
its message names the current green full baseline report even though G0 does not rerun
the VM.

Tripwires: unapproved new dependency, materially broader import, unapproved trust/key
custody change,
unclassified full red, or a map/reality conflict larger than a mechanical correction.

### A1 / G1 — Modular Genesis host and visible boot design

Capability: a user can boot directly into the new readable Genesis Conversation /
Context / Composer design, while the existing console and provider chat still work.

Ready after I0. Run in parallel with B1 and C0/C1-C3. Terr-A owns this track; temporary
sub-lanes are allowed only for the disjoint leases below.
The WiFi lease is released at `cf323a7`. Never broaden a lease merely to make the graph
appear busy.

Parallel Terr lanes after frozen interfaces:

- `G1-state`: extract interactive visual state from `console.rs` into
  `shell_host/state.rs`; exact lease: `seed-kernel/src/console.rs` and
  `seed-kernel/src/shell_host/state.rs`; keep serial parsing behavior byte-compatible;
  make Genesis the default view. This packet may prepare its extraction in parallel,
  but it is not independently committable: `console.rs` still names
  `ui::RuntimeStatus`, and `crate::shell_host` does not exist until the orchestrator's
  module-root join. Its authoritative compile/commit occurs only with that join.
- `G1-render`: exact lease: `seed-kernel/src/ui.rs` plus new
  `seed-kernel/src/shell_host/{genesis,diagnostics,wifi_flow}.rs` and
  `raios-core/src/genesis_layout.rs`. Mechanically extract the complete current guided
  WiFi controller/render path before reducing `ui.rs` to its `RuntimeStatus`
  compatibility re-export, then implement the design in section 4. The pure layout
  module must host-test 1024x768, 1280x800, and 1920x1080 logical geometry. This lane
  must not edit
  `console.rs`, `main.rs`, or `shell_host/mod.rs`.
- `G1-harness`: create a no-secret `vm-harness/capture-genesis-shell.ps1` by safely
  extracting HMP screenshot conversion; that new script is its complete write lease.
  It must use the normal release image, a unique temp run dir/ports, stop only its own
  QEMU PID, keep build output, and require no API key or unverified TLS.

Orchestrator integration lease:

- create `shell_host/mod.rs` and module wiring;
- export `raios_core::genesis_layout` from the singleton `raios-core/src/lib.rs`;
- update `main.rs` to depend only on `ShellHost`;
- delete the old render path once the new one works, retaining only the non-visual
  `ui.rs` compatibility shim; no silent dual-renderer fallback;
- keep framebuffer/text/provider/network driver behavior unchanged.

G1 may not create an intermediate commit that boots Genesis but loses provider setup
or the clickable guided WiFi entry. Before deleting the old `ui.rs` renderer body,
wire the mechanically
extracted current provider/WiFi setup through Genesis or an explicitly labeled trusted
setup overlay. G2 may improve its context and presentation, but every G1 commit keeps
the existing capability reachable.

Checks:

- scoped rustfmt for touched files;
- `cargo test --locked -p raios-core genesis_layout`;
- `cargo fmt --all -- --check`, recording only already-known unrelated drift if it is
  still present;
- release kernel build and package;
- quick Shadow VM profile;
- a no-secret screenshot at the real 1280x800 QEMU mode plus host-tested layout sizes
  above; do not invent a larger GOP screenshot mechanism if Slice 0 cannot validate
  one;
- orchestrator visually inspects clipping, readability, focus, cursor, and absence of
  stale old tabs.

Commit only after the integrated wave is green. Capability sentence leads the commit.

### A2 / G2 — Typed Context, trusted setup overlays, real recovery UI

Capability: from Genesis a user can inspect useful system context, configure the
existing AI/WiFi paths through trusted overlays, and enter real recovery without using
the technical console.

Ready after A1 reaches I1. It does not wait for B1 or C1-C3. Terr-A keeps ownership so
the ShellHost boundary remains coherent while other tracks continue.

Parallel Terr lanes:

- `G2-context`: implement `shell_host/context.rs` from existing snapshots,
  service/problem/capability/provider facts; no duplicated truth state. Extract the
  current `problem.list` conditions into pure `system_problem_facts.rs` and make both
  `agent_protocol_system.rs` and ShellHost consume the same `ProblemFact` iterator.
  Cache the BOOTCTL-derived recovery projection on entry/refresh/state transition;
  never perform an AHCI read on the regular redraw cadence. Exact lease:
  `seed-kernel/src/shell_host/context.rs`, new
  `seed-kernel/src/system_problem_facts.rs`, and
  `seed-kernel/src/agent_protocol_system.rs`.
- `G2-wifi`: own the already mechanically extracted `shell_host/wifi_flow.rs` and
  preserve the exact firmware/HW_SPEC/live-scan/SSID/RAM-password sequence and honest
  final denial while integrating it into Context and the trusted overlay. That file is
  its exact lease; it must not change driver/network authority.
- `G2-secure`: implement `secure_overlay.rs` for masked provider/WiFi secrets and
  trusted prompts; no secret leaves current core-owned buffers. That new file is its
  exact lease.
- `G2-recovery`: implement `shell_host/recovery.rs`. Extract a reusable structured
  read-only recovery snapshot builder or derive it from the same existing sources;
  never parse serial JSON and never clone recovery authority. Extract a shared typed
  recovery action executor from the current lifeline implementation; both
  `recovery_lifeline::dispatch` and Genesis buttons become adapters over that executor
  and the same scoped evaluators. The worker lease is only the new ShellHost file; the
  orchestrator owns edits to the existing lifeline/executor sources.
- `G2-harness`: create
  `vm-harness/shadow-vm-smoke-profile-genesis-ui.ps1` for the non-Wasm Genesis/
  overlays/recovery baseline. This file is its exact lease; common harness wiring is
  orchestrator-only.

Singleton integration may touch `recovery_lifeline.rs` only to share a pure snapshot
builder used by both serial emission and UI. It must not change the pinned vocabulary
or action evaluators. The orchestrator wires `genesis-ui` through the common profile
ValidateSet/dispatch and monitor-port condition and adds bounded HMP helpers now, in
G2, so automated UI evidence precedes the Wasm authority work. That singleton lease
includes `vm-harness/shadow-vm-smoke.ps1` and any one shared HMP helper it extracts.

Checks:

- Marvell host tests, at least
  `cargo test --locked -p raios-core marvell_wifi_cmd`;
- release build/package;
- quick profile;
- focused recovery profile;
- no-secret Genesis, provider overlay, diagnostics, and recovery screenshots;
- run the new `genesis-ui` focused profile. It deterministically
  drives keyboard navigation/HMP input, asserts that a fake runtime secret sentinel is
  masked and absent from serial/screenshot text, proves the WiFi entry reaches the
  existing real flow or its honest QEMU `device_not_detected` denial, and proves a
  Recovery action reaches the shared typed lifeline executor;
- QEMU keyboard and pointer navigation through all reachable overlays. Screenshots
  support visual review but are not the sole dispatch/secret-boundary evidence.

### B1 + I2 / G3 — Typed display-list core and exact UI import authority

Capability: raiOS can authorize and validate one exact personal-shell display-list
surface without granting framebuffer, secret, network, recovery, or broad host-call
access.

Foundation readiness: Terr-Core and Terr-Guest may start immediately after I0 and ADR
0011 against the frozen ABI, in parallel with A1/A2 and C0-C3. Their decoder, evaluator,
guest and host tests grant nothing by themselves.

Authority readiness at I2: only after A2 and B1 are both ready, Sol checks their full
diff and ADR 0011 invariants. The runtime/linker integration below is a genuine
authority flip; do not arm it if the final goal prompt did not explicitly approve the
section 6 imports.

Parallel lanes:

- Terr-Core (`xhigh`): owns new `raios-core/src/ui_frame.rs` and
  `raios-core/src/scoped_wasm_import_grant.rs`. Implement the decoder/validator/limits
  and exact per-service grant policy with exhaustive pairwise denial reasons. The
  orchestrator owns the short `raios-core/src/lib.rs` module export because module
  roots are singleton integration files.
- Terr-Guest (`high`): owns `wasm-guests/svc-personal-shell-proof/**` only. Build the
  minimal guest source against the frozen ABI; no descriptor or signature edits.
- Terr-Harness (`high`, Track D): may scaffold fixtures after the ABI freeze, then
  extends the existing
  `vm-harness/shadow-vm-smoke-profile-genesis-ui.ps1` and capture harness with personal
  shell assertions. It must assert real positive behavior plus forbidden-import,
  malformed-frame, overdraw, and trap fallback.

At I2, orchestrator/trust integration is sequential and singleton while unrelated C
track workers continue in disjoint files:

- implement only the six `ui` host imports in `wasm_runtime.rs`;
- add `seed-kernel/src/personal_shell_service.rs` as the stateless render-per-event
  adapter. It stages V1 context/input, invokes one fresh Store/Instance with the fixed
  fuel budget, returns a typed accepted/rejected frame result plus health/trap state,
  and owns no framebuffer;
- let `ShellHost` call that adapter after sanitized input/state transitions and consume
  only a fully validated frame; `main.rs` keeps only the existing ShellHost poll/render
  facade and learns no Wasm details;
- wire evaluator output directly into the per-instance linker;
- extend `scripts/build-wasm-guest.ps1` for the proof guest;
- add artifact/descriptors and perform the documented attestation/signing procedure;
- edit `build.rs` without weakening signature checks;
- register the typed `ui.personal_shell_proof` diagnostic in a dedicated
  `agent_protocol_ui.rs` adapter and the existing protocol method table;
- preserve the G2 common-harness wiring while extending its focused assertions; do
  not create a second UI profile or duplicate HMP helpers.

The G3 singleton lease explicitly includes `raios-core/src/lib.rs`,
`seed-kernel/src/main.rs`, `seed-kernel/src/agent_protocol.rs`, new
`seed-kernel/src/agent_protocol_ui.rs`, `seed-kernel/src/wasm_runtime.rs`, new
`seed-kernel/src/personal_shell_service.rs`, `scripts/build-wasm-guest.ps1`,
`seed-kernel/build.rs`, the new artifact/descriptors/signatures,
and any exact import-grant audit-id mapping Slice 0 finds necessary. Add the proof
guest to root `Cargo.toml`/`Cargo.lock` only if Slice 0
confirms it participates in the workspace; otherwise preserve the current isolated
guest-manifest build pattern. These files are never edited by two lanes concurrently.

Checks before kernel integration:

- `cargo test --locked -p raios-core ui_frame`;
- `cargo test --locked -p raios-core scoped_wasm_import_grant`;
- deterministic guest rebuild/hash check;
- secret scan after descriptor/signature work.

Checks after integration:

- release build/package;
- `genesis-ui` focused VM profile;
- inspect report JSON, new positive and denial needles, counts, sidecar, and serial
  evidence; a green profile without the named negative cases is false green.

### AB / G4 — Personal-shell lifecycle, secure attention, crash fallback

Capability: a user can enter a signed personal shell, interact with it, press F12 to
return to Genesis, and automatically recover to Genesis after a personal-shell trap.

Ready after I2. This is the A+B join; it does not wait for C1-C3 or Vault integration.

Tasks:

1. Terr-AB owns only `shell_host/personal_surface.rs` as the sole owner of validated
   personal frames and personal focus/input routing.
2. A separate tiny input lease owns only `input.rs` for the minimal F12 mapping;
   intercept it before guest delivery.
3. Activate the signed proof through the real current-boot service/runtime path.
   The exact non-default entry is the typed diagnostic/serial method
   `ui.personal_shell_proof`; Genesis Diagnostics exposes the same action as
   `Run signed shell proof`. Both call one shared typed starter and label the result
   test infrastructure.
4. Render the secure strip after the personal frame and prove it cannot be overdrawn.
5. Route one input event to the proof service and observe a changed frame.
6. Trigger its trap/fuel case and prove the service cannot retain input or screen
   ownership; Genesis becomes active and recovery remains callable.
7. Update service inventory only now: a static core-owned `core.ui.genesis` row may
   replace the old framebuffer fiction, while `svc.user.shell` is projected into
   dynamic/current-boot inventory only during a real loaded/running proof instance and
   disappears after exit/trap. Never add it to the release-default static table.
8. Keep the release default at `personal_shell: not_created`; proof activation stays a
   test/diagnostic path.

The orchestrator owns the singleton AB join in `shell_host/mod.rs`,
`personal_shell_service.rs`, `agent_protocol_system.rs`, `service_inventory.rs`,
`main.rs` and the common harness wiring. Do not overlap A2 Context edits to
`agent_protocol_system.rs` with this join.

Checks:

- host tests from G3;
- release build/package;
- `genesis-ui` focused profile including screenshot captures before personal entry,
  inside proof shell, after F12, and after trap fallback;
- focused recovery profile after the trap;
- quick profile for existing boot/network/provider behavior.

### Track C / G5 — Persistent Secret Vault and M13 structured store

Capability: after one trusted Genesis setup, a user can reboot, unlock the Vault with
the real recovery key, and let raiOS reconnect to the same WiFi or reuse the same
provider credential without re-entering it or exposing it to the shell.

C0-C3 begin after I0 and run parallel to Tracks A and B. They do not touch the active
foreign WiFi-association/supplicant files. Only I3/G5.4 waits until that slice is
committed or otherwise frozen and its final API is revalidated. G5 may not overwrite,
reformat or absorb that work. Sol performs its post-WiFi review immediately before I3,
not before the independent store/crypto/keyring work.

#### G5.0 — Record the storage and key-custody contract

Ready after I0; run concurrently with A1 and B1. This packet freezes C-track internals
without reopening the cross-track API frozen at I0.

1. Create `docs/architecture-decisions/0012-secret-vault-storage-and-key-custody.md`
   from section 7 and the M13 target already recorded in `ROADMAP`.
2. Record that the only authorized target is an already provisioned, dedicated,
   identity-checked raiOS data partition on an internal storage device. The boot stick,
   ESPs, `SEED_DATA/RECLOG`, Windows volumes and foreign media are denied.
3. Freeze typed store, ciphertext-envelope, key-wrapper and broker interfaces before
   dispatch. Record the exact partition GUID/type/label contract without creating or
   resizing a physical partition.
4. Record the two honest positive claims separately: recovery-key unlock is required
   in QEMU; TPM auto-unlock remains `not_proven` until backed by a real TPM command
   transport plus swtpm or approved hardware evidence.
5. Record `physical_target_driver_supported` as a separate gate. The current repo has
   AHCI but no NVMe driver; QEMU-AHCI evidence cannot prove the Surface's internal
   target is writable. Missing controller support blocks only physical persistence and
   never redirects writes to USB/Windows.

Owner: orchestrator, with Sol read-only review and Luna limited to the ADR text.
Check: targeted doc diff/whitespace check. ADR 0012 is committed before C1-C3 source
work; Tracks A and B do not wait for that docs commit.

#### G5.1 — One real crash-consistent structured-store backend

Ready after C0. Run concurrently with G5.2, G5.3 and Tracks A/B.

Terr-Storage (`xhigh`) owns only:

- `raios-core/src/structured_store.rs`;
- `seed-kernel/src/structured_store/**`;
- focused pure host tests colocated with those modules.

Implement the section 7 PREPARE/DATA/COMMIT/TOMBSTONE log and deterministic replay.
The store accepts an already validated block-region capability; it must not enumerate,
choose, format or repartition disks. Every open rechecks GPT identity, store UUID,
generation and bounds. Writes use readback verification; only complete committed
transactions become authoritative. Initial capacity exhaustion is an explicit denial,
not an untested compactor or silent overwrite.

No generic final block-region API exists yet: AHCI sector I/O is private and current
validators are tied to `SEED_DATA`. The orchestrator owns a singleton extraction of the
smallest bounded block-region capability plus a pure dedicated-partition GPT validator;
it must bind exact controller, port, device and partition identity rather than reuse the
broad first mass-storage-controller match. Terr-Storage receives only that validated
capability and never raw disk selection.

Terr-Disk owns only a new `scripts/make-structured-store-image.py`, which constructs a
dedicated marked disposable QEMU data disk without reusing `SEED_DATA`. The
orchestrator integrates module roots, the bounded driver join and this test disk. Create
a focused `structured-store`
profile that proves format-on-explicit-test-media only, reopen, torn frame, torn commit,
wrong partition identity, stale generation, bounds denial, checksum/hash-chain failure
and power-cut replay. It must never point at a host physical disk.

Checks: pure record/replay tests, scoped rustfmt, release build/package and green
`structured-store` focused report. This is a risky storage boundary, so do not batch it
with cryptographic authority before its focused report is green.

#### G5.2 — Typed ciphertext envelope and scoped-use policy

Ready after C0. Run concurrently with G5.1, G5.3 and Tracks A/B. The orchestrator takes
one short all-lane quiescence window for the exact Cargo dependency edits before Terr-
Core starts; afterward its isolated target directory is independent.

Terr-Core (`xhigh`) owns only:

- `raios-core/src/secret_vault.rs`;
- `raios-core/src/scoped_secret_use.rs`;
- their colocated host tests.

Add direct exact-version dependencies only for the versions already locked:
`aes-gcm = 0.10.3`, `hkdf = 0.12.4`, and `zeroize = 1.8.2`, with the smallest `no_std`
feature sets verified by Slice 0. Do not add a human-password KDF, custom cipher,
generic secret framework, plugin API or second storage backend.

Implement the exact envelope, nonce/version rules and AAD bindings from section 7.
Tests include known-answer or upstream-compatible vectors, round trip, tampered tag,
tampered AAD, wrong store UUID/epoch/consumer/operation/target/kind, duplicate nonce,
stale version and zeroization-sensitive buffer ownership. Pairwise policy denials are
typed and stable; none return plaintext.

The orchestrator alone exports the modules from `raios-core/src/lib.rs` and changes
Cargo manifests/lockfile. Checks: targeted host tests, `cargo tree`/lockfile diff proving
no unrelated upgrade, scoped rustfmt and secret scan.

#### G5.3 — VMK wrappers, real recovery unlock and honest TPM boundary

Ready after C0 and the frozen crypto/key-wrapper types; command encoding and transport
work may proceed concurrently with G5.1/G5.2. The final recovery-wrapper join waits for
the small required G5.2 primitives rather than idling the whole lane.

Terr-Keyring (`xhigh`) owns only:

- `raios-core/src/tpm2_commands.rs`;
- `seed-kernel/src/tpm2_transport.rs`;
- `seed-kernel/src/secret_vault/keyring.rs`;
- their focused tests.

Implement the random 32-byte VMK, versioned recovery wrapper, current/next/last-good
wrapper generations and the bounded unlock state machine from section 7. The recovery
key is a random 32-byte high-entropy value, displayed once through a core-owned secure
overlay and confirmed before persistence; raiOS never substitutes a low-entropy human
password. HKDF-SHA256 derives the recovery KEK, and AES-GCM wraps the VMK.

Extend the current ACPI TPM discovery only with actual bounded CRB/TIS command
transport and the minimum TPM 2.0 create/load/unseal/policy operations needed by ADR
0012. Discovery or status reads alone never set `vault_vmk_tpm_sealed`,
`tpm_vmk_wrapper_ready`, or `auto_unlock_ready`; ADR 0007 `owner_sealed` is unaffected.
If no swtpm fixture or approved physical TPM is available, land the tested command
codec and fail-closed transport state, but keep the positive TPM claim open. The real
recovery-wrapper two-boot path remains mandatory and is not a fake TPM substitute.

This slice does not promote owner authority, accept arbitrary PCR policies or weaken
update/rollback rules. Checks: command codec/length/bounds tests, wrapper rotation and
corruption tests, release build, and the keyring part of `secret-vault` on a dedicated
QEMU test disk.

#### G5.4 — Vault store, Broker and the two exact consumers

Progress (2026-07-10): the real RR1 provisioning/write/reboot/recovery-unlock subset
is green in `shadow-20260710-160920-28360.json` (29/29). The provider sub-slice is green
in `shadow-20260710-174308-19744.json` (42/42): physical masked save, encrypted exact-C1
commit, second-boot replay/RR1 unlock, durable local-only pre-use audit with verified
readback/reparse/rescan, and one contained exact Authorization-header consumer all
pass; the dynamic sentinel is absent from every required artifact. The production
OpenAI writer requires real pinned trust before it can request the same one-use lease.
This report does not claim a live network provider request or physical persistence.
WiFi Vault use, forget/SAFE behavior and TPM auto-unlock remain open, so I3/G5.4 is not
complete.

I3 readiness: C1/C2/C3 are green, A2 exposes the frozen secure-overlay/status API, the
foreign WiFi work is clean/frozen, and Sol has approved the exact authority diff. Only
this join waits for those dependencies.

Pre-I3 unarmed groundwork is allowed and has landed: recovery-keyring restoration
requires exact replayed/readback wrapper equality and approved-core policy; the store
codec carries ciphertext records only; retained nonce metadata requires complete,
ordered, replay-verified history. These facts do not satisfy I3, do not create a Vault
operation, and do not relax any readiness requirement above.

Terr-Vault (`xhigh`) owns only:

- `seed-kernel/src/secret_vault/mod.rs`;
- `seed-kernel/src/secret_vault/store.rs`;
- `seed-kernel/src/secret_vault/broker.rs`;
- focused module tests.

Compose, do not merge, the boundaries: `keyring` unwraps keys but does not parse the
store; `store` persists typed ciphertext but does not authorize consumers; `broker`
authorizes exact operations but does not write media. The public API offers
save/replace/forget/status and scoped `use_for_wifi` / `use_for_provider` operations;
there is no `get_secret`, reveal, export, debug dump or Wasm import.

After those modules and tests are stable, the orchestrator takes a singleton lease for
the smallest adapters in the now-current WiFi supplicant, `provider_config.rs`,
`openai.rs`, `secure_overlay.rs`, `shell_host/context.rs` and module roots. Existing
RAM buffers become bounded input/cache only. The Broker releases a zeroized plaintext
buffer solely for the exact consumer, target and operation in section 7. It does not
grant networking, provider trust, association success or TLS trust.

This join removes or confines the current generic `wifi::copy_passphrase()` and
`provider_config::copy_api_key()` plaintext accessors behind the exact Broker policy.
The already present `ConnectionJob` drop-zeroization remains, and the OpenAI request
path must zero its 256-byte key/header scratch after the transport consumes it on every
success and failure path.

Genesis gains only Vault state and trusted actions: unlock, save/replace, forget and
explicit SAFE reconnect. Normal boot may automatically attempt the already authorized
known WiFi after successful Vault unlock. SAFE may unlock for recovery but never
auto-connects. A crashed WiFi/provider/personal-shell service cannot retain a plaintext
buffer or delete the core-owned Vault handle.

Checks: scoped host tests, existing WiFi/provider tests, release build/package,
`genesis-ui`, `quick -Network`, secret scan and negative serial/screenshot inspection.

#### G5.5 — Reboot, crash, corruption and recovery evidence

Ready after I3. Track D may have authored the separate profile earlier against frozen
needles, but the orchestrator alone launches its authoritative two-boot runs.

Track D owns the new focused profile file. The orchestrator owns a separate bounded
`vm-harness/secret-vault-reboot.ps1` driver (or an exact extension of the existing
persistence reboot harness), because a dot-sourced smoke profile cannot reboot its own
VM. Common dispatcher wiring remains singleton.

The orchestrator creates one focused `secret-vault` profile and singleton common-
harness wiring. It uses only sentinel credentials on a dedicated disposable QEMU disk
and proves:

1. first boot creates the store only after explicit test-media authorization, creates
   the recovery wrapper, saves one fake WiFi passphrase and one fake provider key, and
   records only typed IDs/status in ordinary memory/audit/context;
2. second boot reopens the exact store, unlocks with the recovery key, and delivers
   each sentinel only to its exact fake/contained consumer target;
3. neither sentinel occurs in serial, report JSON, captured framebuffer-visible text,
   raw screenshot bytes where meaningful, audit,
   context, personal-shell input/frame, crash output, default release image, ESP or
   RECLOG evidence;
4. wrong consumer/target/operation/kind, changed partition identity, stale wrapper,
   nonce reuse, tag/AAD corruption, torn commit and power cut all fail closed;
5. WiFi/provider/personal-shell crashes return control to Genesis without losing the
   core Vault handle or leaking plaintext;
6. NORMAL may attempt a scoped reconnect after unlock, while SAFE requires explicit
   user action; missing/corrupt Vault remains a visible RAM-only denial;
7. forget appends a tombstone and prevents later use without pretending old flash
   cells were physically erased.

At G5 block close run the focused `structured-store`, `secret-vault`, `genesis-ui` and
`recovery` profiles, a separate `quick -Network`, then `full`, followed by the secret
scan. The full/recovery requirement applies here because G5 closes a durable storage,
recovery and secret-custody block. Every failed VM predicate is classified before a
retry.

Absence of the real target partition or a TPM fixture does not block the durable code,
QEMU disk proof or recovery-key path. It blocks only claims of physical persistence or
TPM auto-unlock. Never manufacture those claims by writing the boot USB.

### I4 / G6 — Final regression, visual acceptance, release artifact

Capability: the exact release candidate is demonstrably bootable, recoverable,
network-preserving, free of plaintext secrets, capable of encrypted credential reuse,
and visually matches the universal Genesis design.

The orchestrator freezes code and runs, serially:

1. full diff read against the plan/ADR and every preservation boundary;
2. scoped file-size check for all touched `.rs` files;
3. `cargo fmt --all -- --check` with no new formatting regression;
4. relevant host tests including `raios-core`, UI-frame/import-grant, and Marvell;
5. fresh release build then package;
6. `structured-store` and `secret-vault` focused profiles, including the two-boot
   recovery-key unlock and sentinel-absence assertions;
7. `genesis-ui` focused profile;
8. focused recovery profile;
9. a separate `quick -Network` run that actually attaches e1000 and proves DHCP; the
   default no-NIC quick/full runs do not count as network preservation evidence;
10. full Shadow VM profile with the documented long timeout and serial chunking;
11. `scripts/scan-secrets.ps1`, plus an exact sentinel search over release artifacts,
    reports, serial logs and screenshots/OCR text;
12. no-secret QEMU captures of Genesis, Vault locked/ready state, trusted provider
    setup, diagnostics,
    recovery, proof personal shell, and post-trap Genesis at 1280x800;
13. host tests for Genesis layout at 1024x768, 1280x800, and 1920x1080;
14. compare final image/kernel hashes with the artifacts used by the reports and prove
    the default image/ESP contains no Vault record, wrapper, key or sentinel.

Network command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick -Network -TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

Do not require or spend a real provider key. If a valid local key and pin already
exist and the optional direct-provider smoke is useful, package only an ignored temp
image and delete it afterward. It is not a release prerequisite.

Any failed VM run is classified in `PROJECT_STATUS` before retry. A third repeated
failure stops. A red newest full report turns all work into Red Gate repair.

### I5 / G7 — Non-destructive USB handoff and final docs

Capability: the owner has a prepared raiOS stick whose ESP A contains the exact
QEMU-verified Genesis release while ESP B and `SEED_DATA/RECLOG` remain intact and the
stick contains no Secret Vault material.

Preflight, read-only:

1. Disk 2 must be present, USB, SanDisk, not boot/system/read-only, and match the G0
   fingerprint's UniqueId/serial where available, exact size, GPT disk GUID, and
   partition GUIDs. If neither a stable UniqueId nor the recorded GPT identity is
   available, stop.
2. GPT layout must contain the exact recorded ESP A, ESP B, and `SEED_DATA`
   type/start/size identities.
3. Recompute the ESP-B identity, SEED_DATA superblock, and entire bounded RECLOG-region
   hashes and match the G0 fingerprint before write.
4. The final packaged kernel hash must equal the kernel exercised by final QEMU/VM
   evidence.
5. Scan the source image and mounted ESP-A staging tree for Vault namespaces, wrappers,
   VMKs, recovery material and sentinel values; any match stops the write.
6. The shell must be elevated. If UAC/admin is unavailable, stop safely; do not bypass
   Windows security or fall back to a destructive writer.

Before the real write, harden `scripts/update-usb-esp-a.ps1` inside its singleton G7
lease. Add a mandatory `-ExpectedFingerprintPath` for autonomous mode and reuse the
read-only inspection logic from `scripts/make-gpt-persist-image.py --inspect-json` (or
a narrow physical-drive adapter over the same parser). The updater itself, immediately
before copy, must re-resolve Disk 2 and compare every identity/layout/pre-write hash;
an external preflight alone is insufficient. Add `-PreflightOnly` to exercise the
same checks without mutation. Re-resolve the same identity after copy and fail if the
device changed. Do not add formatting, partition, raw SEED_DATA write capability or
Secret Vault provisioning. The dedicated internal data partition is a different
identity and is never created, resized or substituted by this wave.

Write:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild -ExpectedFingerprintPath target\usb-handoff\disk2-fingerprint.json
```

Never call `write-stage0-usb.ps1`, `Clear-Disk`, `diskpart clean`, format, repartition,
or another disk. The owner authorization covers only the existing ESP-A refresh.

Post-write:

- verify source/destination `kernel.elf` SHA-256;
- re-read the partition identities and prove byte-identical ESP-B identity,
  SEED_DATA-superblock, and bounded RECLOG-region hashes;
- preserve the script log path and final hash in the session handoff;
- safely dismount/eject if supported without disturbing evidence;
- do not claim a physical Surface boot, WiFi association, or ten-second input-idle
  test occurred unless somebody actually performed it.

After code/evidence freeze, Luna updates status, roadmap only if appropriate,
dashboard, the completion markers in this plan, and changed command docs. The final
orchestrator reviews and commits those docs after a targeted diff/whitespace check and
secret scan. The G7 docs-only commit names the unchanged final G6 `structured-store`,
`secret-vault`, `genesis-ui`, `recovery`, `quick -Network`, and `full` reports. Then
run and paste all AGENTS.md
end-of-session checks so the post-full docs commit has an explicit evidence chain.

## 12. Autonomous stop conditions

Do not ask the user about colors, wording, module names, ordinary compiler fixes,
agent scheduling, or other choices settled here. Continue with best judgment.

Stop safely and report evidence if any of these occurs:

- newest full report is red: switch immediately to Red Gate repair-only work and stop
  only after the bounded repair protocol is exhausted;
- an unexpected tracked dirty/deleted file or overlapping lease appears;
- a required change needs a seventh UI import, raw secret exposure, raw framebuffer,
  an unapproved dependency/version, broader trust authority, or destructive storage
  action;
- any path attempts to place Vault records, ciphertext, wrappers, recovery material or
  VMKs on the boot stick, ESPs, `SEED_DATA/RECLOG`, Windows or foreign media;
- a worker needs to format/repartition a physical disk, invent a replacement storage
  identity, add custom cryptography, accept a human password without an approved KDF,
  add a generic `get_secret`/Wasm import, or bypass the Broker;
- a new uncommitted WiFi/association change overlaps the singleton G5 adapter files;
  wait for its owner or stop rather than merging over it;
- attestation/signing tooling is missing and cannot be recreated mechanically from
  documented in-repo sources;
- the same guest behavior fails three times after bounded fixes;
- a real provider key would be required;
- at G7, Disk 2 is absent or lacks the G0 fingerprint, identity/layout mismatches, is
  boot/system, elevation is missing, or any write would need format/repartition;
- the only remaining proof needs a physical Surface boot or human hardware action.

A missing TPM fixture is not permission to fake auto-unlock and does not block the
recovery-key/QEMU path. Record `tpm_auto_unlock: not_proven` and continue. A missing
approved internal data partition likewise blocks only the physical-persistence claim;
it never redirects Vault data to the USB stick.

If USB is the only remaining blocked step, leave the exact verified release image,
hashes, reports, screenshots, and one command ready. Do not call the whole goal
complete, but do not undo completed code.

## 13. Final acceptance report format

The orchestrator's final answer leads with the outcome and contains only:

- new user capability in one sentence;
- final commit(s);
- focused structured-store, Secret Vault, recovery, Genesis, `quick -Network`, and full
  report filenames/results;
- two-boot recovery-key unlock result, honest TPM auto-unlock status, and the exact
  identity class of the tested QEMU/physical data partition;
- secret-scan result;
- links to the main Genesis and personal-proof screenshots;
- final release image/kernel hash;
- proof that the default image and boot USB contain no Vault material or sentinel;
- USB disk identity, update log, and readback hash, or the exact named safe blocker;
- honest remaining gaps: physical Surface Marvell association/`PORT_RELEASE`/DHCP,
  physical internal-data-partition proof, TPM auto-unlock if unproven, and arbitrary
  external/generated personal-shell intake.

## Short goal prompt

> Führe `docs/plan-reviews/genesis-shell-execution-plan-2026-07-10.md` mit Codex 5.6 xhigh vollständig autonom und agent-nativ nach seinem Abhängigkeitsgraphen bis zum verifizierten Genesis-USB aus: halte verfügbare Slots mit disjunkten Ready-Paketen belegt, baue ab dem ersten Commit sauber in den finalen Modulen (Treiber-Ausnahme wie definiert) und lasse Luna abgeschlossene Fakten parallel nachführen. Ich genehmige ausschließlich die dort definierten fail-closed UI-Wasm-Imports, den nicht-default `current_boot` dev-key-Proof-Service, den in Abschnitt 7 begrenzten Secret Vault für WLAN- und Provider-Secrets auf einer bereits bereitgestellten identitätsgeprüften raiOS-Datenpartition mit TPM-/Recovery-Key-Wrappern und den exakt gepinnten `aes-gcm 0.10.3`, `hkdf 0.12.4`, `zeroize 1.8.2` sowie den nichtdestruktiven ESP-A-Refresh des identitätsgeprüften SanDisk-Sticks Disk 2. Keine Partitionierung und keine Secrets auf dem Boot-Stick; stoppe nur an den genannten Sicherheits-Tripwires.
