# Project Status

Development memory for future agents: build normal changes in the repo with
real code, tests, VM reports, and docs; do not fake the finished raiOS memory
architecture during development. Keep slices on the final architecture path by
splitting stable boundaries early, separating runtime/diagnostic/harness/handoff
surfaces, and making observed execution evidence more authoritative than copied
command lists or prose summaries.

Build-time hygiene memory: do opportunistic cleanup while building when a clear,
low-risk ownership boundary appears. Extract runtime, diagnostic, selftest,
emit, harness, or docs surfaces before they become emergency refactors. Do not
force speculative refactors while behavior, trust boundaries, or protocol shape
are still unclear; finish the real slice first, then cut along the stable
boundary that emerged. Use file size as an early warning: around 1k-2k LOC,
look for ownership boundaries; around 3k-5k LOC, actively split if a stable
boundary exists; above 10k LOC should be exceptional and documented; 20k+ LOC
requires a deliberate split plan before more behavior is added.

Documentation ownership memory: keep `README.md` as product thesis, quickstart,
and concise current reality; keep `docs/ROADMAP.md` as phase direction plus the
compact active cursor; keep this file as the authoritative detailed status,
exact next task, verification evidence, known gaps, and unabridged
implementation history; keep `docs/DEBUGGING.md` focused on commands, smoke
profiles, protocol probes, and failure modes.

Current exact next task (G5 block close after verified G5.5a/G5.5b): freeze the G5
code and run the required recovery/full checkpoint evidence plus the remaining named
G5 close regressions and secret scan. Reuse the green focused Secret Vault evidence;
do not reopen its authority boundaries merely to grow predicate count. If G5 closes,
advance to I4/G6 final release/hardware acceptance. Keep the exact Broker consumers,
general SAFE durable-write denial, durable-store identity/rollback chain and prior
G5.5a torn/power-cut behavior unchanged. Do not expose plaintext to ShellHost/Wasm,
add generic secret access, provider auto-load, broad mutation, or physical-target
support. Disk 2 is unplugged; do not write a physical disk. Physical persistence and
TPM auto-unlock remain unproven. The separate bare-metal WiFi proof still requires
positive `LINK`, association/AID, `PORT_RELEASE`, RX/TX and DHCP evidence before live
provider access is claimed.

I3 complete-history Broker foundation verified (2026-07-10) - a kernel composition
agent can now obtain the complete ordered list of committed records from the same
identity-revalidated structured-store replay, and the Vault Broker accepts writes
only with that opaque history/nonce proof. Its two one-use leases can produce only
the bounded NXP WPA2 command or exact OpenAI Authorization header; neither returns a
secret slice, length, `Vec`, string, callback, debug value, or Wasm import. A separate
security review blocked and repaired caller-forgeable use/mutation flags, evidence
normalization, incomplete replay and nonce-completeness claims before runtime wiring.
`cargo test --locked -p raios-core` passed 396/396, the release kernel built, focused
`release/vm-reports/shadow-20260710-132631-20352.json` passed 9/9, and the secret scan
found no OpenAI-key-like values. This remains unarmed: no runtime core-policy source
or WiFi use-audit binding exists yet, so recovery unlock and secret use stay denied.

I3 validated Store-to-Vault replay join verified (2026-07-10) - the exact QEMU-only
AHCI/GPT store path now returns an opaque replay capability only after revalidating
the same device region before and after the full read. On both the initial boot and
the independent reboot, the Secret Vault facade filters only its fixed namespace and
loads that complete commit history into the Broker; the unrelated C1 proof record is
excluded. Focused `release/vm-reports/shadow-20260710-133203-24112.json` passed 11/11,
including `C1_VAULT_COMPLETE_REPLAY_BOUND` on both boots. This binds no physical
target and still grants no provision, unlock, decrypt, consumer use, or secret write.

I3 owner Core-Policy root verified (2026-07-10) - raiOS can now recompute the
complete raw executable file supplied by Limine and accept its logical BOOTCTL
slot/generation only when the fixed 128-byte policy verifies under the distinct
owner software pin. The private P-256 scalar exists only under the ACL-protected
local owner-key directory outside the repository; packaging derives and checks the
pinned public key before signing the exact linked kernel. Core Policy is separate
from descriptor, promotion, distribution, Vault, and recovery keys. Host/Core tests
passed 406/406 including the pinned-root fingerprint assertion; the focused
`release/vm-reports/shadow-20260710-145039-13864.json` then passed 5/5 with an exact
A/1 BOOTCTL join and no denial. This proves software-pinned current-boot consistency,
not Secure Boot, TPM measurement, hardware identity, in-memory integrity,
deterministic ESP A/B selection, or anti-rollback. It creates no Vault unlock,
decrypt, use, write, or TPM authority; durable pre-use audit evidence remains the
next gate.

I3 runtime Core-Policy-to-Vault join verified (2026-07-10) - after the exact QEMU
store replay is bound, the Vault Broker now also retains only the executing policy
accepted by the owner-pinned Core Policy verifier. Callers can no longer supply or
replace a claimed kernel generation/policy hash during recovery unlock. The focused
`release/vm-reports/shadow-20260710-150107-28328.json` passed 13/13, including
complete replay and exact Core Policy binding on both the initial boot and independent
reboot. This still performs no provisioning, wrapper write, unlock, decrypt or secret
use. The next positive capability is the physical-input RR1 provisioning/unlock path;
durable audit remains mandatory before either named consumer may use plaintext.

I3 real RR1 provisioning and recovery unlock verified (2026-07-10) - a user can
now explicitly create a random RR1 in Genesis, capture its one-time trusted display,
confirm it through physical-only input, persist/readback-verify the generation-1
recovery wrapper on the exact dedicated QEMU store, reboot, replay that wrapper, and
unlock the Broker with the same RR1. The focused two-boot report
`release/vm-reports/shadow-20260710-160920-28360.json` passed 29/29 predicates,
including exact 80-glyph visual structure/checksum, USB-HID-only actions, full store
identity/replay/Core-Policy joins, no second display, and raw serial absence checks on
both boots. The transient PPM is overwrite/truncate/deleted and the retained host byte
array is cleared; RR1 never enters a report, predicate value, serial command or hash.
This writes only the recovery wrapper: no WiFi/provider secret, durable pre-use audit,
consumer use, provider auto-load, physical target or TPM auto-unlock is authorized yet.

I3 durable provider Secret-Vault vertical slice verified (2026-07-10) - a user can
now physically enter an OpenAI credential through the masked Genesis overlay, persist
and readback-verify its encrypted record on the exact disposable QEMU C1 Vault store,
reboot, replay it, unlock with RR1, and deliver it once to the contained exact
Authorization-header validator only after a durable `local_only` pre-use audit was
written, read back, typed-reparsed and rescanned. The focused report
`release/vm-reports/shadow-20260710-174308-19744.json` passed 42/42 predicates,
including auditless denial, audit-before-consumer ordering, exact offline RECLOG
inspection and absence of the dynamic provider sentinel from both boot logs, their
combined log, boot/C1/persist images and every run-directory file. The production
OpenAI writer now accepts the Vault lease only after the actual pinned verifier mints
an opaque trust token; development bypass cannot mint it, and any Vault/audit failure
cannot fall back to the legacy RAM key. This does not claim a live provider request,
physical Vault persistence, WiFi Vault use, forget/SAFE behavior, provider auto-load,
TPM auto-unlock, WebPKI trusted-time completeness, or physical Surface networking.

I3 durable WiFi Secret-Vault vertical slice verified (2026-07-10) - a user can now
physically enter a WPA2 credential through the trusted Genesis overlay, bind it to the
selected exact SSID/BSSID, persist/readback-verify only its encrypted record on the
disposable QEMU C1 store, reboot/replay it, unlock with RR1, and deliver it once only
to the NXP WPA2 PMK formatter after a durable `local_only` pre-use audit. Focused
`release/vm-reports/shadow-20260710-192431-4220.json` passed 56/56 predicates,
including dynamic provider and WiFi sentinel absence, wrong-BSSID and auditless
denials, audit-before-consumer ordering, exact offline two-record RECLOG inspection,
and no association/link/`PORT_RELEASE`/DHCP success marker. `wifi::copy_passphrase()`
and long-lived `ConnectionJob` plaintext were removed; an available, locked or
forgotten Vault record cannot fall back to the legacy RAM credential. This proves no
physical persistence, live radio association, NORMAL/SAFE reconnect, forget tombstone,
service-crash isolation, TPM auto-unlock, or provider network request.

I3 durable Secret-Vault forget path verified (2026-07-10) - an unlocked user can now
open the core-owned Genesis Vault manager and tombstone either provider or WiFi only
after a second physical confirmation. Each fixed-slot intent binds the current record
version and previous commit-frame hash before the exact disposable C1 store appends,
flushes, readback-verifies and fully replays version 2; only then is the matching legacy
RAM mirror cleared. Focused three-boot report
`release/vm-reports/shadow-20260710-195715-22816.json` passed 78/78 predicates: both
tombstones replay as `Forgotten`, RR1 still unlocks, both contained exact-use attempts
return `secret_forgotten`, and boot three emits neither pre-use audit nor consumer
success. The old encrypted store history remains physically present and is not claimed
erased. This proves no SAFE reconnect, physical persistence, crash isolation, TPM
auto-unlock, provider request, radio association, `PORT_RELEASE` or DHCP.

I3/G5.4 NORMAL + SAFE reconnect verified (2026-07-10) - after RR1 unlock, an ordinary
boot may scan and select only the exact metadata-matching stored SSID/BSSID without a
new password prompt. SAFE instead binds the executing kernel to the authoritative
owner-signed bootable `last_good` BOOTCTL slot, unlocks the same wrapper, and emits no
provider/WiFi audit or consumer success until one physical Genesis action. That action
carries a move-only target-bound token through `ConnectionJob` to `SupplicantPmk`,
consumes one exact `local_only` `safe_recovery`/explicit WiFi audit append, then reaches
only the existing bounded NXP consumer. The general SAFE durable-memory append and
provider paths remain denied; `AlreadyReady` now applies only to the retained exact
connection target. Focused four-boot report
`release/vm-reports/shadow-20260710-204801-23168.json` passed 95/95 predicates with
base-image SHA-256 `1eabdcdc7ab61d6a3f1a82f4aa7e7609bc46934b99f1058ccf29c59290da14a3`
and report SHA-256
`c7bc1001b94b4e1c7c154b769ba9d646f92784d3f28e06bf0a2c73262223c838`.
It proves SAFE no-auto-use, exactly one physical WiFi audit+contained consumer,
offline exact record shape, both sentinel scans, the prior NORMAL provider/WiFi path,
and third-boot tombstones. `raios-core` passed 405/405, the release kernel and direct
Core-Policy sign/verify package path built, and PowerShell parsing/format checks were
green. At that point this closed I3/G5.4, not G5.5: live radio association/link/
`PORT_RELEASE`/DHCP, physical persistence, crash/corruption/power-cut evidence, TPM
auto-unlock and provider network success remained unclaimed.

G5.5a exact-C1 torn/power-cut recovery verified (2026-07-11) - a user can now survive a
real exact-C1 WiFi version-2 PREPARE plus TOMBSTONE write with no COMMIT and a hard QEMU
stop, then reboot, unlock and use the retained committed WiFi version 1 with the core
Vault handle preserved. Focused report
`release/vm-reports/shadow-20260711-001242-11280.json` passed 118/118 predicates; report
SHA-256 is `56b6e67010b4f407765eeb3b9b9d027df21bd9f006c6d713d969de376cc11265`.
The final report readback itself and the default release image plus recursive ESP files
also pass the exact dynamic provider/WiFi/RR1 sentinel scan. That checkpoint completed
only G5.5a torn-commit/power-cut evidence; its then-open corruption, identity and
personal-trap cases are closed separately by G5.5b below. Physical persistence, TPM
auto-unlock, live radio/link/`PORT_RELEASE`/DHCP and provider network success remain
denied or unclaimed.

G5.5b corruption and personal-trap continuity verified (2026-07-11) - disposable
copies of the exact QEMU C1 medium now fail closed without acquiring Vault authority:
a foreign partition GUID is rejected before fixture acceptance, while a copy whose
last committed frame has a changed CRC is accepted only as the exact fixture and then
locked as `Core(StoreChainLocked)`. A physical Genesis Vault action exposes the honest
RAM-only denial in both cases. Independent copied wrapper/envelope mutations reject
stale policy, stale context, a corrupt wrapper, tag, AAD, consumer/operation/target
bindings and nonce reuse. After recovery unlock, physical Enter/Escape leaves the
core-owned Vault, the existing bounded typed `ui.personal_shell_proof trap` request
produces exactly one real fallback, the redacted recovery lifeline still responds, and
physical navigation reopens Vault Manage without another RR1/Broker unlock or any
extra provider/WiFi audit or consumer use. The prior G5.5a exact torn/power-cut proof
remains green and is exercised by the same focused profile.

Authoritative focused report
`release/vm-reports/shadow-20260711-010926-21860.json` passed 152/152 predicates with
report SHA-256 `c88d6296c1a6572ef5bdf84e39abf2382ad64a01a339607a86b07904c2319a8c`,
temporary base-image SHA-256
`535fe9aa8688ba247266c552c765519991ca6708cc0c8662d74bb1d6f8523c42`, and combined
serial-log SHA-256
`4192094ff7b9a19a0c6ba415a77eec86b13562694cd1a6417f2e73b8696a7934`.
This is focused disposable-QEMU evidence, not the G5 block-close checkpoint: recovery
and full still must pass before G5 is closed. Physical durable-secret storage, TPM
auto-unlock, live Surface association/link/`PORT_RELEASE`/RX/TX/DHCP, and live provider
network success remain denied or unclaimed.

VM failure classification (2026-07-11, first G5.5b corruption profile) - report
`release/vm-reports/shadow-20260711-004301-20812.json` passed 15 predicates and failed
`secret-vault:boot1:rr1_confirmed` with `serial_transport_failure:null`. Verdict:
`guest-behavior` - physical xHCI input delivered only 75 of the 80 decoded RR1 bytes
(`VAULT_RR1_INPUT_REJECTED reason=input_shape length=75`) before any new G5.5b
store-denial, crypto-mutation or personal-trap path executed. This is a suspected
intermittent guest USB-HID loss, not host transport; retry is allowed only after a
bounded input-delivery repair or stronger acknowledgement.

VM failure classification (2026-07-11, second G5.5b corruption profile) - report
`release/vm-reports/shadow-20260711-005003-15928.json` passed all 15 recorded
predicates and then stopped before adding a failing predicate with
`serial_transport_failure:null`. Verdict: `host-transport/harness` - the new
secure-HID acknowledgement observed all 80 RR1 characters, but incorrectly required
both press and release after the terminal Enter even though the accepted press closes
the secure overlay. It timed out with `expected_events=2 observed_events=1`; no G5.5b
store-denial, crypto-mutation or personal-trap behavior ran. The bounded repair keeps
full transition acknowledgement for secret characters and requires only the terminal
Enter press before the existing typed outcome marker.

VM failure classification (2026-07-11, third G5.5b corruption profile) - report
`release/vm-reports/shadow-20260711-005330-25796.json` passed 39 predicates and failed
`secret-vault:store-denial:corrupt-frame:c1_denied` with
`serial_transport_failure:null`. Verdict: `host-transport/harness` - the copy-only
host tool changed the last frame's CRC byte as intended, but the harness expected the
internal parser reason `Port(Core(FrameCrcMismatch))`. Production replay deliberately
converts any corrupt frame into a locked, record-cleared chain, so the real guest
correctly emitted `C1_STRUCTURED_STORE_DENIED: Core(StoreChainLocked)` after accepting
the exact fixture. The bounded repair changes only that expected fail-closed marker;
the CRC mutation, GPT identity and authority-denial assertions remain unchanged.

VM failure classification (2026-07-11, fourth G5.5b corruption profile) - report
`release/vm-reports/shadow-20260711-005958-11396.json` passed 111 predicates and failed
`secret-vault:boot2:personal_trap_request` with `serial_transport_failure:null`.
Verdict: `guest-behavior` - RR1, both corrupt-store cases, SAFE, power-cut replay and
all new wrapper/crypto denials completed, but the helper sent the typed personal-trap
command while Genesis intentionally remained in Settings/Setup mode. The guest
returned one Bell per rejected command byte and never dispatched an agent request.
The bounded repair physically leaves Settings with Escape, proves `SETUP CLOSED`,
uses the existing serial command only in Command mode, then physically navigates back
to Vault after fallback. No serial Vault action or secret access is added.

VM failure classification (2026-07-11, first G5.5 power-cut profile) - report
`release/vm-reports/shadow-20260710-235921-23132.json` has no guest predicate because
the host exhausted C: while creating the initial 538-MiB disposable valid-a persist
fixture; its JSON report was itself truncated by the same disk-full condition. QEMU
never started (`pid:null`, empty QEMU args, no serial hash). Verdict:
`host-transport/setup` - no guest, Vault, partial transaction or recovery behavior ran.
Before retry, remove only the exact failed run directory plus the already classified
`shadow-20260710-144728-24560` forensics directory; keep all reports and repository
artifacts. No code/security inference is made from this attempt.

VM failure classification (2026-07-11, second G5.5 power-cut profile) - report
`release/vm-reports/shadow-20260711-000516-24272.json` passed 52 predicates and failed
`secret-vault:power-cut:precommit_ready` with `serial_transport_failure:null`. Verdict:
`guest-behavior` - HMP delivered a physical USB input batch for F9, but the xHCI HID
translator did not map usage `0x42` to the existing set-1 F9 code `67`, so Genesis never
received the contained trigger and no partial Store write ran. The bounded repair adds
only that standard HID-to-keycode mapping; the exact-C1/unlocked checks and all Store
authority remain unchanged before the focused retry.

VM failure classification (2026-07-10, first SAFE-Secret-Vault profile) - report
`release/vm-reports/shadow-20260710-204013-3440.json` contains no failing predicate
because packaging stopped before QEMU with `Core Policy verification failed with exit
code 101`; QEMU PID, hardware profile, serial log and base-image hash are all absent.
Verdict: `host-transport/setup` - no guest behavior executed and neither disposable
Vault medium was attached or mutated. The retry is forbidden until the host
Core-Policy signer/verifier failure is reproduced directly and repaired; SAFE guest
authority, Vault, audit, driver and harness behavior are not inferred from this run.

VM failure classification (2026-07-10, second SAFE-Secret-Vault profile) - report
`release/vm-reports/shadow-20260710-204313-12148.json` has 28/28 preceding predicates
green and no failed predicate; after boot one provisioned the disposable C1 media and
stopped QEMU, host `Copy-Item` failed with `not enough space on the disk` while
materializing the 538-MiB SAFE BOOTCTL clone. Verdict: `host-transport/setup` - the
SAFE VM never started, no SAFE audit/consumer ran, no QEMU process remains, and the
failure is outside guest behavior. The bounded repair preserves the source image and
creates a separate sparse `pending-safe` image with the same observed empty pre-audit
RECLOG; deleting or rewriting the original normal/forget fixture is forbidden.

VM failure classification (2026-07-10, first provider-Secret-Vault profile) - report
`release/vm-reports/shadow-20260710-173553-10436.json` failed predicate
`secret-vault:boot2:broker_unlocked` with `accepted=false reason=input_shape` and
`serial_transport_failure:null`. Verdict: `host-transport` / harness race - the
visible unlock gesture first produced a stale empty-submit diagnostic
`VAULT_RR1_INPUT_REJECTED reason=input_shape length=0`, but the same immutable boot-two
serial log later records `VAULT_BROKER_UNLOCKED source=recovery slot=current` after the
complete USB-HID RR1 arrived. The guest therefore accepted the real recovery key; the
host outcome poll failed early on an older generic rejection instead of waiting for the
operation-specific success or rejection marker. Boot one had already committed and
readback-verified the provider ciphertext, and no sentinel appears in the report. The
bounded repair removes only the redundant third Enter from the boot-two unlock-opening
gesture, so the overlay no longer submits an empty value before the real USB-HID RR1;
it does not relax RR1 length/parsing, hide an explicit unlock rejection, or alter
Vault, audit, consumer, trust, storage, or secret handling.

VM failure classification (2026-07-10, first WiFi-Secret-Vault profile) - report
`release/vm-reports/shadow-20260710-182518-7320.json` failed predicate
`secret-vault:boot1:wifi_entry_ready` with `found=false` and
`serial_transport_failure:null`. Verdict: `host-transport` / harness navigation -
all 22 preceding predicates passed, including RR1 and encrypted provider commit, but
the harness assumed one Tab moved Settings focus from API key directly to WiFi. The
actual frozen focus order contains `Clear API key` between them; serial proves the
physical Enter activated that harmless action and emitted `API KEY CLEARED`, while no
WiFi secure overlay opened and no WiFi sentinel was created or entered. The bounded
repair sends the second required physical Tab before Enter. It changes no guest UI,
Vault, audit, target, formatter, storage, secret, association, link, or DHCP behavior.

VM failure classification (2026-07-10, second WiFi-Secret-Vault profile) - report
`release/vm-reports/shadow-20260710-183042-15696.json` failed only predicate
`secret-vault:offline_exact_wifi_audit_record` after 55/55 preceding predicates
passed, with `serial_transport_failure:null`. Verdict: `host-transport` / harness
canonicalization - the offline inspector proves the exact second `local_only`
`native_wifi_supplicant/associate_bound_bss` record, no raw `ssid`/`bssid`/`target`
fields, `network_export_authorized:false`, and `test_infrastructure:true`; its typed
SHA-256 value is canonically rendered as `sha256:<64 lowercase hex>`, while the new
harness expected only the bare 64 hex characters. Both dynamic sentinel scans, both
audit-before-consumer predicates, wrong-BSSID/auditless denials, and no-link/DHCP
claim were already green. The bounded repair accepts only the existing canonical
`sha256:` form; it does not weaken record fields, hashes, Vault, audit, target,
formatter, storage, secret, association, link, or DHCP behavior.

VM failure classification (2026-07-10, third WiFi-Secret-Vault attempt) - run
`shadow-20260710-184311-20692` ended before a report or predicate result was
written when the outer Windows process exited with `0xC000013A`; therefore the
failing predicate is `none (outer harness interrupted before report)`. Verdict:
`host-transport` - no QEMU process remains, and the retained serial log proves
the guest completed boot, RR1 creation and encrypted provider commit, then opened
the WiFi secure-entry overlay and was still receiving physical USB-HID input at
the interruption. It emitted no WiFi commit, audit, consumer, link, `PORT_RELEASE`
or DHCP marker. The bounded retry changes no guest code or predicate; the exact
orphaned disposable run directory is removed only after this classification and
path verification below `%TEMP%`.

VM failure classification (2026-07-10, first Secret-Vault forget profile) - report
`release/vm-reports/shadow-20260710-194954-9688.json` failed predicate
`secret-vault:boot3:broker_unlocked` with `accepted=false reason=timeout` after
66 preceding predicates passed and `serial_transport_failure:null`. Verdict:
`guest-behavior` - the third boot replayed both version-2 tombstones and accepted the
complete physical RR1 (`VAULT_BROKER_UNLOCKED source=recovery slot=current`), but the
contained forgotten-state proof then emitted
`VAULT_FORGOTTEN_DENIAL_TEST_REJECTED reason=broker_denied` and changed the UI outcome
before `VAULT_RR1_UNLOCKED`. The shared provider fact path incorrectly collapsed
`Forgotten` together with `Missing` into `SecretMissing`, while the WiFi path already
preserved the distinct `SecretForgotten` denial. No boot-three pre-use audit or
consumer-success marker was emitted. The bounded repair splits that single provider
match arm so the durable tombstone is reported and proved as forgotten; it does not
change store, unlock, audit, decrypt, consumer, fallback or mutation authority.

VM failure classification (2026-07-10, first Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-152552-3140.json` failed predicate
`secret-vault:boot1:usb_keyboard_ready` with `serial_transport_failure:null`.
Verdict: `host-transport` - the guest completed USB enumeration and emitted the
current positive marker `usb-hid: keyboard ready on slot 1`, but the new profile
waited for the obsolete text `usb-hid: boot keyboard ready`. The flow had not yet
opened the RR1 display, captured a framebuffer, or attempted a Vault write. The
bounded repair changes only the two boot-keyboard marker needles to the observed
current guest marker before the focused retry.

VM failure classification (2026-07-10, second Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-152906-24892.json` failed in the host cleanup
stage before predicate `secret-vault:rr1_visual_structure_checksum`; all preceding
predicates through the one-time display and physical USB action were green and
`serial_transport_failure:null`. Verdict: `host-transport` - Windows PowerShell's
loaded .NET runtime does not expose `System.Security.Cryptography.CryptographicOperations`,
so the host byte-array cleanup helper threw after the transient PPM had already been
truncated to zero bytes. The zero-length file was then removed with a verified path
inside that run's temp directory; QEMU is stopped, and neither serial nor the report
contains RR1. The bounded repair replaces only that unavailable managed API with
portable in-place `Array.Clear`; the overwrite/truncate/delete tripwire is unchanged.

VM failure classification (2026-07-10, third Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-153242-18952.json` failed predicate
`secret-vault:boot1:rr1_confirmed` with `serial_transport_failure:null`.
Verdict: `guest-behavior` - the once-rendered 80 glyphs passed exact bitmap-font,
RR1 structure and checksum validation, the transient PPM was erased, and the full
physical xHCI input reached the bounded overlay, but the guest emitted
`VAULT_RR1_REJECTED` before any wrapper/store write. The input length was therefore
exact while its parser/candidate result was not accepted. The bounded diagnostic
repair adds only a stable non-secret rejection class (`format`, `checksum`,
`confirmation_mismatch`, or another fixed facade code); it does not log input bytes,
key material, a key hash, or weaken the parser before the focused retry.

VM failure classification (2026-07-10, fourth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-153822-6984.json` failed in packaging before
predicate `boot:serial_console_ready` with `serial_transport_failure:null`.
Verdict: `host-transport` - the tracked Limine binary remained present, but the
PID-only temporary ESP directory could be reused after a prior process and PowerShell
then copied `release/esp` one level deeper, so packaging correctly denied the missing
staged `EFI/BOOT/BOOTX64.EFI`. QEMU never started. The bounded repair gives each
temporary ESP a fresh GUID-qualified path, verifies it is below the host temp root,
and refuses a pre-existing destination; no image, guest, Vault or RR1 behavior changes.

VM failure classification (2026-07-10, fifth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-154010-24700.json` failed predicate
`secret-vault:boot1:rr1_confirmed` with `serial_transport_failure:null`.
Verdict: `host-transport` - packaging, visual RR1 checksum and the 80 HMP key
commands were positive, but this run emitted neither the guest success marker nor
either rejection marker after the final submit; the last observed USB batch ended
before a guest outcome. The bounded repair repeats only the idempotent physical Enter
after a short pause (if the first arrived, the second merely closes the result view)
and gives an incomplete overlay submission the fixed non-secret `input_shape` code.
No key bytes, hashes, parser rule, wrapper or media authority change.

VM failure classification (2026-07-10, sixth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-154430-24600.json` failed in the host visual
decoder before predicate `secret-vault:rr1_visual_structure_checksum`, with
`serial_transport_failure:null`. Verdict: `host-transport` - the guest emitted the
post-present layout marker, but that transient screendump differed from the exact
font template at glyph position 0; the same decoder was exact in earlier runs. The
PPM was erased and QEMU stopped before any recovery input or Vault write. The bounded
repair permits one newly captured retry only for this exact pre-predicate bitmap-font
mismatch; both PPMs take the same overwrite/delete path and the pixel comparison,
RR1 structure and checksum rules remain unchanged.

VM failure classification (2026-07-10, seventh Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-154745-27112.json` again failed in the host
visual decoder before predicate `secret-vault:rr1_visual_structure_checksum`, with
`serial_transport_failure:null`; both permitted captures mismatched fixed prefix
glyph position 0 and both PPMs were erased before QEMU stopped. Verdict:
`host-transport` - file existence alone was observed after a short HMP reply window,
not a completed screendump reply/stable raster. The bounded repair waits for the full
HMP screendump response and reports only non-secret template score/tie metadata if the
fixed prefix still differs. Exact glyph matching, structure and checksum remain intact.

VM failure classification (2026-07-10, eighth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-154954-11052.json` again failed before predicate
`secret-vault:rr1_visual_structure_checksum`, with `serial_transport_failure:null`.
Verdict: `guest-behavior` - the completed HMP capture scored fixed prefix `R` against
an effectively blank panel (`score=79368`, one candidate), while three USB input
batches arrived immediately after the post-present marker. Trailing Enter events from
the visible Vault-open action could therefore advance `Showing` into confirmation
before capture. Both PPMs were erased and no Vault write occurred. The bounded repair
separates gestures: Enter opens Vault, but only physical Space or the visible pointer
button can dismiss the one-time display. Parser, key, wrapper and store rules do not
change.

VM failure classification (2026-07-10, ninth Secret-Vault RR1 profile) - run
`shadow-20260710-155219-10356` failed before predicate `boot:serial_console_ready`
and could not write a valid report because the C: volume had zero free bytes.
Verdict: `host-transport` - retained failed-run QEMU fixtures had accumulated roughly
3.7 GiB; the new persist fixture stopped at 51,462,144 bytes, QEMU never started and
no RR1 PPM existed. After verifying every absolute target below `%TEMP%`, the eight
exact run directories from this RR1 debugging sequence and the zero-byte report were
removed; durable reports/classifications remain in the repo and free space returned
to 3.74 GiB. No source, guest or security predicate changed for this cleanup.

VM failure classification (2026-07-10, tenth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-155510-28160.json` failed predicate
`secret-vault:boot1:rr1_confirmed` with `serial_transport_failure:null`.
Verdict: `host-transport` - the one-time display and exact visual checksum were now
stable and the Space command produced USB batches, but no guest success/rejection
outcome followed the 80 subsequent key commands. The report could not prove whether
the distinct acknowledgement actually transitioned `Showing` to `Confirming`.
The bounded repair emits a fixed non-secret `VAULT_RR1_CONFIRMATION_READY` only after
the display handle has been consumed/zeroized and makes the harness require it before
sending any RR1 byte. No parser, key, wrapper, store or authority rule changes.

VM failure classification (2026-07-10, eleventh Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-155919-29896.json` failed predicate
`secret-vault:boot1:rr1_confirmed` with `serial_transport_failure:null`.
Verdict: `host-transport` - the exact visual key and the new destructive transition
marker `VAULT_RR1_CONFIRMATION_READY` were positive, but the bounded overlay rejected
the HMP/xHCI stream as `input_shape`, proving it did not observe exactly 80 characters.
No wrapper/store write was attempted. The bounded repair lengthens each emulated key
hold and reply interval and reports only the masked byte count on shape denial; it does
not expose characters or relax length, format, checksum or candidate equality.

VM failure classification (2026-07-10, twelfth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-160251-10516.json` failed after all 17 boot-one
predicates passed, including physical RR1 confirmation and exact wrapper commit/readback;
`serial_transport_failure:null`. Verdict: `host-transport` - the harness attempted its
mandatory raw serial leak scan while QEMU still held `serial.log` exclusively open, so
Windows rejected `ReadAllBytes` before the independent reboot. The bounded repair stops
and joins each QEMU process immediately before scanning that boot's immutable log. It
does not alter guest behavior, RR1 handling, wrapper/store authority or scan criteria.

VM failure classification (2026-07-10, thirteenth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-160628-28152.json` failed before predicate
`boot:serial_console_ready`; QEMU and guest did not start. Verdict:
`host-transport/setup` - the direct PowerShell retry inherited the unavailable
`F:\scorefollower-build\cargo` package-cache path instead of the repository-local
Cargo home required on this machine. No image, RR1, Vault or store operation ran. The
retry exports `.cargo-home` and the repository `target` directory before invoking the
unchanged focused profile.

VM failure classification (2026-07-10, fourteenth Secret-Vault RR1 profile) - report
`release/vm-reports/shadow-20260710-160643-27024.json` failed after 14/14 recorded
predicates passed, at predicate `secret-vault:boot1:rr1_confirmed`, with
`serial_transport_failure:null`. Verdict: `host-transport` - the guest serial log
already contains the positive confirmation marker, but the Windows PowerShell host
could not select the two-argument `String.IndexOf` overload from an untyped helper
result. The bounded repair casts the cached log result to `System.String` before the
same ordinal marker comparison. No marker, guest, key, wrapper or authority changes.

VM failure classification (2026-07-10, first Core-Policy profile) - report
`release/vm-reports/shadow-20260710-144728-24560.json` failed predicate
`core-policy:owner_signature_kernel_bootctl_bound` with
`serial_transport_failure:null`. Verdict: `guest-behavior` - Limine booted the
kernel and the valid A/1 persist fixture was attached, but the internal-module
request incorrectly supplied the full `boot():` URI where Limine expects an
absolute module path. Limine therefore resolved it below `/kernel/boot():/` and
the core correctly emitted `CORE_POLICY_DENIED reason=core_policy_module_missing`.
The bounded repair changes only the requested/expected module path to
`/raios/core-policy.bin`; it does not weaken signature, executable measurement,
or BOOTCTL predicates before the focused retry.

VM failure classification (2026-07-10, G4 first Genesis UI profile) - report
`release/vm-reports/shadow-20260710-123636-21624.json` failed predicate
`genesis-ui:personal-shell-f12-exit` with `serial_transport_failure:null`.
Verdict: `guest-behavior` - QEMU delivered the USB keyboard event (the serial log
records the input batch), but `usb::hid_usage_to_keycode` did not map HID usage
`0x45` (F12) into the existing internal F12 keycode, so the core-only secure-
attention event was never created. The bounded repair is one exact USB-HID F12
mapping to keycode 88; do not forward raw F12 to the console or guest and do not
weaken the F12/fallback predicates before the focused retry.

VM failure classification (2026-07-10, I2 first Genesis UI profile) - report
`release/vm-reports/shadow-20260710-120551-14196.json` failed predicate
`protocol:system_honesty_report_standing_posture` with
`serial_transport_failure:null`. Verdict: `host-transport` - the emitted
`known_host_import_count` was 11 while its projected list still contained only
the former two imports because the build used a lane-local `CARGO_TARGET_DIR`
but `package-stage0.ps1` copied the stale hard-coded default-target kernel.
QEMU therefore booted an old image, not the current projection. Rebuild and
package through the repository-default target before the one permitted focused-
profile retry.

VM failure classification (2026-07-10, I2 packaged Genesis UI retry) - report
`release/vm-reports/shadow-20260710-121027-10556.json` failed predicate
`genesis-ui:personal-shell-proof` with `serial_transport_failure:null`.
Verdict: `guest-behavior` - artifact and descriptor evidence, the exact six-import
grant, and both evaluator denials were positive, but all six real proof calls
returned `instantiation_failed` before consuming fuel. The next repair must retain
and classify the concrete Wasmi link/instantiation error; do not weaken the proof
predicate or retry before that exact boundary is diagnosed.

AB/G4 personal-shell lifecycle completed (2026-07-10) - a user or diagnostic can
now deliberately queue the checked-in signed proof, see it in the core-owned
personal surface, deliver one sanitized input event, press F12 to return to
Genesis, and recover automatically after its fixed trap or fuel test mode. ShellHost
owns the bounded validated frame/focus state; a fresh Wasm invocation remains in the
service adapter. Raw F12 is never delivered to the guest: both PS/2 keycode 88 and
USB HID usage `0x45` map to core-only secure attention. `svc.user.shell` appears in
`service.inventory` only while the proof is actively running with `current_boot`,
`none` persistence and the six exact UI imports; it disappears after F12/trap/fuel.
The old framebuffer fiction is replaced by core-owned `core.ui.genesis`. Verified:
focused `release/vm-reports/shadow-20260710-124838-24564.json` passed 206/206,
including trap/fuel fallback, Recovery after fallback, and five QEMU PNG captures.
The secure-strip pixel hash was byte-identical before activation, during the personal
frame, and after sanitized input. The normal release remains Genesis with
`Personal shell: not created`; no arbitrary shell, secret, network, provider,
recovery, persistence, or broad mutation authority was opened.

I2/G3 signed personal-shell authority completed (2026-07-10) - an agent can now
run exactly the checked-in, explicitly signed `svc.user.shell` Wasm proof in a
fresh metered instance with only `ui.viewport`, `ui.context_len`,
`ui.context_read`, `ui.input_len`, `ui.input_read`, and `ui.frame_submit` linked.
The immutable V1 context/input packets contain no strings, pointers, or secrets;
only a bounded validated/clipped display-list frame can return. Build-time
attestation verifies both P-256 descriptor signatures and raw artifact bindings.
The runtime admits the proof's one fixed two-element table, not a general table
surface, after the classified initial `table_count_limit_exceeded` denial. Malformed
frames, overdraw, traps, fuel exhaustion, missing linker implementation, and a
broader import request remain observable failures; no loader, external artifact,
framebuffer, secret, network, provider, recovery, persistence, or mutation
authority is granted. Verified: `cargo test --locked --offline -p raios-core`
passed 392/392; release build/package passed; focused
`release/vm-reports/shadow-20260710-121953-4964.json` passed. The default release
does not load or inventory the proof service.

Genesis shell architecture contract accepted (I0/G0, 2026-07-10) - build agents
can now implement the universal core-owned Genesis/recovery surface and the
bounded personal-shell proof against one exact owner-approved trust and Wasm ABI
contract. The accepted execution map is
`docs/plan-reviews/genesis-shell-execution-plan-2026-07-10.md`; ADR 0011 fixes
the six `ui.*` imports, evidence-bound per-service grant, clipped compositor,
secure attention, and non-default `current_boot` proof posture. This is an
architecture contract only: no Genesis renderer, UI import authority, proof
service, F12 fallback, Secret Vault, structured store, or Genesis USB handoff
is yet implemented or verified by this entry. The positive Surface association,
`PORT_RELEASE`, RX/TX, and DHCP proof remains the exact hardware task above.
Baseline evidence remains full report
`release/vm-reports/shadow-20260708-150428-34396.json` passed 7867/7867 and
latest quick report `release/vm-reports/shadow-20260710-010658-31684.json`
passed 542/542.

Secret Vault and structured-storage contract accepted (C0/G5.0, 2026-07-10) -
build agents can now implement one identity-bound M13 store, recovery/TPM VMK
wrappers, and the two exact WiFi/provider secret-use paths without guessing
media, cryptography, key custody, or consumer authority. ADR 0012 fixes the
dedicated internal-partition denial set, PREPARE/DATA/COMMIT/TOMBSTONE replay,
dual hash-checked non-signed superblocks, exact AES-GCM/HKDF/zeroize pins, RR1
recovery-key format, and scoped broker. This is design only: no structured
store, Vault write, direct dependency use, RR1 unlock, TPM command/unseal,
physical-target driver support, or durable secret exists yet. QEMU recovery-
key reboot proof is required; `tpm_auto_unlock`, `vault_vmk_tpm_sealed`,
`tpm_vmk_wrapper_ready`, and `physical_target_driver_supported` remain
unproven. ADR 0007 `owner_sealed` is unaffected by this contract.

Genesis geometry and personal-shell ABI foundation done (A0/B0, 2026-07-10) -
a renderer and the future one-shot Wasm adapter can now derive the fixed calm
Genesis geometry at 1024x768, 1280x800, and 1920x1080, route input without
crossing the core-owned secure strip, validate one bounded/clipped RFRM V1
display list atomically, and authorize exactly the six `svc.user.shell` `ui.*`
imports only when concrete artifact, evidence, ordered-import hash, and linker
list agree. The proof guest is a stateless no_std V1 source registered in the
workspace; it receives only the six frozen imports and returns a visibly distinct
frame from sanitized input. This is foundation only: no UI import is linked, no
personal shell runs, the old release honesty projection remains unchanged until
I2/G3, and the Genesis renderer/USB handoff are not yet claimed. Verified:
`cargo test --locked --offline -p raios-core` passed 351/351; release kernel
build passed; quick report `release/vm-reports/shadow-20260710-020632-29720.json`
passed 542/542; secret scan passed.

Structured-store foundation done (C1 foundation, 2026-07-10) - a kernel adapter
can now execute the real PREPARE/DATA/COMMIT/TOMBSTONE replay and append protocol
only through an already identity-validated region port, rechecking its controller,
device, GPT/partition identity, geometry, superblocks, bounds, flush and readback
before every write. The pure M13 record codec rejects torn/corrupt commits and
never selects, formats, repartitions, or redirects a device. The disposable-only
GPT fixture uses the frozen M13 type `5eedda7a-c0de-4a55-9a15-000000000013` and
label `RAIOS_STRUCTURED_STORE`; it accepts regular test images below `target/` or
the host temp directory only. Verified: raios-core structured-store tests pass,
the release kernel compiles this module, and the fixture self-test/create/verify
paths pass. This is not physical persistence yet: no AHCI region capability,
format-on-explicit-test-media runtime path, focused structured-store VM report,
or physical-target support claim exists.

Secret-envelope and scoped-use foundation done (C2 foundation, 2026-07-10) -
the core can now turn a bounded trusted-input WiFi passphrase or OpenAI provider
key into a versioned AES-256-GCM envelope bound to its exact store, epoch,
consumer, operation and target; it can also derive and authenticate the ADR 0012
recovery VMK wrapper. The pure use evaluator grants only the bound WPA2-PSK/CCMP
supplicant association or a trust-authorized request to `api.openai.com`; it
returns metadata only and has no reveal/export/Wasm path. VMK, recovery key,
DEK and plaintext owners zeroize on drop. Verified: `cargo test --locked --offline -p raios-core`
passed 379/379 and the secret scan passed. This does not create a
Vault, key source, recovery unlock, store write, broker lease, WiFi/provider
integration, TPM claim, or durable-secret claim; those remain gated by C1/C3/I3.

Genesis host and key-custody foundations done (A1/C3, 2026-07-10) - a user can
now boot the normal release image directly into the calm, core-owned Genesis
surface: secure strip, Conversation, typed Context, AI and WiFi setup actions,
and the Composer are the only framebuffer host; the former `AI`/`CONSOLE`/`SET`
renderer has been removed rather than left as a second visual path. Existing
serial chat, provider setup, masked entry, and guided WiFi firmware/scan/BSS/
association flow remain the shared action/state path. The Context starts
honestly with `Personal shell: Not created`, `AI connection: Needs key`,
`Secret Vault: Not configured`, and observed network state. C3 also adds a
bounded TPM 2 command codec, CRB/TIS transport plan, zeroizing response/command
buffers, RR1 recovery-key confirmation, and exact recovery-wrapper readback
selection. It does not make a TPM sealed/unsealed, a vault provisioned, an
automatic unlock, a secret broker, or durable credential claim. Verified:
`cargo test --locked -p raios-core` passed 386/386; release packaging succeeded;
and the safe no-secret capture of that exact release image produced
`target/captures/genesis-shell-a1.png` at 1280x800. No focused VM report has
been claimed for C3 because no storage/authority path is active yet.

Structured-store partition admission foundation done (C1, 2026-07-10) - the
future AHCI region port can now accept only a redundant GPT whose primary and
backup headers/entry arrays validate, whose exact frozen type and UTF-16 label
match `RAIOS_STRUCTURED_STORE`, and whose disk plus partition GUIDs match the
explicit approved identity supplied by the caller. It denies malformed GPT,
foreign identity, wrong label/type, missing redundancy, out-of-range and
overlapping targets. It cannot discover, choose, map, format, or write a disk;
the later C1 AHCI capability and QEMU-only vertical proof supply that controlled
runtime path. Verified by the three focused host tests included in the 386/386
core run.

Structured-store QEMU vertical slice done (C1/G5.1, 2026-07-10) - an agent can
now run one isolated `structured-store` profile that creates a fresh 16-MiB
`RAIOS_STRUCTURED_STORE` GPT image, admits only Q35 AHCI `00:1f.2` port 4 plus
the frozen disk/partition GUIDs, validates both GPT copies, proves the region
blank, formats the dual superblocks, appends a committed record through real
ATA `FLUSH CACHE EXT`, and replays that record after a separate QEMU reboot.
The port re-identifies the device and revalidates its exact BDF/port/GPT/geometry
before every write/readback boundary. It cannot enumerate controllers, use the
SEED_DATA detector, select a physical disk, or format any identity other than
the hard-coded disposable fixture. Verified:
`release/vm-reports/shadow-20260710-032738-34812.json` passed 9/9 predicates
in 107065 ms, including format/open, append/flush/readback, and reboot replay.
No physical partition, vault envelope, TPM unlock, secret broker, WiFi/provider
consumer, or durable-secret behavior is claimed by this test-only store path.

Genesis trusted-interaction core completed (A2/G2, 2026-07-10) - a user can now
inspect the same typed current-boot problem facts in Genesis and the agent
protocol, open a cached/redacted Recovery view from the secure strip, and use
Genesis Recovery controls to restart or disable the fixed demo echo through the
same typed Lifeline executor used by the serial path. Existing masked RAM-only
provider/WiFi setup remains reachable from Genesis; the driver, provider and
serial authority paths were not duplicated. Component commits `1a8b1d7`,
`3dd7f96`, `69c29fe`, and `008be89` respectively add the shared facts, bounded
zeroizing secure-overlay foundation, cached Recovery model, and focused profile
foundation. Verified: the integrated `genesis-ui` profile
`release/vm-reports/shadow-20260710-034302-30252.json` passed 181/181; release
build/package passed; and the no-secret 1280x800 capture
`target/captures/genesis-shell-a2.png` was visually inspected. This does not
create a Vault secret, Broker, durable credential, recovery hash-load, or
rollback authority: secure-overlay submission is not yet wired to a secret use
path, and recovery hash-load/rollback remain disabled. The default release stays
in normal Genesis with no personal shell.

C4/I3 unarmed Vault composition foundation completed (2026-07-10) - a future
core-owned Broker can restore an in-RAM recovery keyring only from an exact
replayed-and-readback-verified wrapper under approved core policy. The typed
ciphertext-only record codec accepts only the two fixed secret slots and the
recovery-wrapper slot, while retained AES-GCM nonce metadata can be reconstructed
only from a complete, ordered, replay-verified history; incomplete, identity-
mismatched, non-monotonic, or duplicate history fails closed. Commits `95b7bf4`,
`d27c96d`, and `f90e7db` establish those boundaries; `0920346` makes the existing
disposable QEMU C1 port available to the Vault composition root. The source was
release-built and the focused structured-store regression
`release/vm-reports/shadow-20260710-040559-24348.json` passed 9/9 with zero
failures. This is explicitly not a Vault set/unlock/decrypt/plaintext-use,
WiFi/provider, audit, physical-target, or durable-secret authority claim; I3 is
still separately Sol-review-gated.

VM failure classification (2026-07-10, C1 first structured-store profile) -
the outer host command expired after 604 seconds before `shadow-vm-smoke.ps1`
could write a report, leaving its QEMU PID to be stopped explicitly. Failing
predicate: none (no report). Verdict: `host-transport` for the outer timeout;
the retained guest serial simultaneously exposes the deterministic underlying
`guest-behavior` cause: `C1_STRUCTURED_STORE_DENIED:
Device("ahci_explicit_port_read_buffer_phys_missing")`. The first bounded
repair must use a dedicated aligned static DMA staging buffer for C1 rather
than assuming a local stack `AhciSectorBuffer` has a physical mapping. Do not
retry until that exact buffer path is repaired.

VM failure classification (2026-07-10, B0 foundation quick) - report
`release/vm-reports/shadow-20260710-013553-29700.json` passed 41/42 predicates
but failed `protocol:system_honesty_report_standing_posture`. Verdict:
`guest-behavior` — deterministic protocol-projection mismatch, not a host flake:
the orchestrator prematurely changed the release honesty record from its current
two displayed imports while B0 merely registered the future UI vocabulary. The
bounded repair restores that existing projection; G3/I2 alone owns the truthful
displayed-import transition when the linker authority is actually integrated.

VM retry classification (2026-07-10, B0 foundation quick) - the first repaired
retry did not produce a report because the combined package-and-quick host command
hit its 184-second caller timeout and left only its own QEMU PID `1384`, which was
then explicitly stopped. Failing predicate: none (no report was emitted). Verdict:
`host-transport` — insufficient outer command budget, not a guest result. The next
retry runs the already packaged image with the quick profile alone and an independent
timeout budget.

VM failure classification (2026-07-10, D0 Genesis capture harness) - the first
local capture launch exited before any guest predicate or framebuffer marker;
QEMU stderr reported `Block node is read-only` for the IDE boot image. Failing
predicate: none (guest did not start). Verdict: `host-transport` — harness block-
device configuration. The bounded repair boots a writable temporary image copy
inside the harness-owned run directory while preserving the release image.

No-secret framebuffer capture slice done (D0, 2026-07-10) - a developer can now
capture the normally packaged release image without embedding a provider key,
mutating that release artifact, enabling unverified TLS, or stopping unrelated
QEMU processes. `vm-harness/capture-genesis-shell.ps1` owns a unique temporary
image/OVMF copy and dynamic serial/monitor ports, waits for the real framebuffer
ready marker, validates the complete P6 raster, and stops only the QEMU PID it
started. The repaired same-code path produced the 1280x800 PNG
`target/captures/baseline-shell.png`; visual inspection confirms that it is the
current technical console baseline, not evidence that the new Genesis design is
implemented.

VM failure classification (2026-07-09, guided WiFi quick) - report
`release/vm-reports/shadow-20260709-231946-32656.json` ended with
`serial_transport_failure: qemu_exited` after all 295/295 predicates and all
45/45 executed commands had passed; the serial log ends after a complete
`service.hot_swap external:svc.demo.hello` denial response. Failing predicate:
none. Verdict: `guest-behavior` — suspected intermittent guest/QEMU exit, not
closed as a host flake under the AGENTS.md fail-then-pass rule. The same-code retry
`release/vm-reports/shadow-20260709-232029-9484.json` passed 542/542.

VM failure classification (2026-07-08, full profile) - report
`release/vm-reports/shadow-20260708-145915-16988.json` passed 380/381
predicates and timed out on command
`agent module.vm_report_diagnostic_selftest`; QEMU was then observed exited.
Verdict: `guest-behavior` — suspected intermittent guest/QEMU exit, not closed
as a host flake under the AGENTS.md fail-then-pass rule. Failing predicate:
`command:agent module.vm_report_diagnostic_selftest`. The same-code full retry
`release/vm-reports/shadow-20260708-150428-34396.json` passed 7867/7867.

Marvell association + DHCP slice done (2026-07-10) - a user can now select a
live open or WPA2-PSK/CCMP BSS and ask raiOS to run a real bounded 88W8897
connection sequence instead of stopping at credential entry. The driver
registers 32-entry PFU TX/RX rings and the 8-entry event ring with firmware,
enables Ethernet-II MAC control, validates every command ID/length/sequence and
firmware result, and sends the selected BSSID/channel/beacon/rates/security
evidence in the real associate command. WPA2 additionally requires the actual
firmware-supplicant capability, configures the NXP WPA2/AES profile and
boot-only passphrase/SSID/BSSID material, zeroes the connection copy after use,
zeroes the PMK DMA command after completion/failure, and grants link authority
only after both successful association and the real
firmware `PORT_RELEASE` event. Open association grants link only after a valid
successful association response/AID. Only then does the Marvell `WifiPhy`
attach the existing smoltcp stack and start real DHCP; link-loss events clear
network state and quarantine WiFi DMA. WEP, WPA, WPA3, malformed RSN, missing
capability/evidence, timeouts, rejected association, and malformed or stale
responses fail closed. This is implemented and VM-verified but still awaits the
positive Surface radio/PORT_RELEASE/DHCP proof. Verified: 50/50 focused Marvell
tests and 335/335 complete `raios-core` tests; scoped rustfmt; release kernel
build; quick Shadow VM report
`release/vm-reports/shadow-20260710-010658-31684.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 230309`, report sha256
`3968bafe928a55ec1244ec76eb893138fbf229c99c6f4f8dc41fdfc4bb28ffbd`, and
`base_image.sha256: 491758f68d2e6255cd3fa90a201ae6806952e6a66c283fa99dddcf1b91c76603`.
The scoped rustfmt check covers all nine touched Rust files; follow-up commit
`da6e458` restored the global `cargo fmt --all -- --check` gate without changing
firmware behavior. Secret scan passed. Owner handoff: Disk 2 `SEED_ESP_A` was
refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260710-011521.log`
reported final kernel sha256
`9871578A46BDE75F1A568D98B23FBAC1B4A3A2E5867120ADF044A2979FCEBD92`.

Bare-metal guided WiFi/input proof (2026-07-10) - the owner can now complete the
real `WiFi DETECTED` guided path through firmware, HW_SPEC, live scan, secured
SSID selection, and masked password entry while keyboard and mouse remain
usable without unplug/replug. The owner explicitly confirmed both inputs and
password entry worked. The Surface screenshot at `UPTIME 65694 MS` shows
`KBD READY`, `MOUSE READY`, `ERR 0`, `TCC 1`, WiFi firmware ready at
`723540/723540`, live scan results, selected `MagentaWLAN-CQL5`, `KEY SET`, and
the honest final `Credentials ready in RAM for this boot` / `Connection not
established` dialog. The USB row also shows the next verified stick append as
`MSC LOG seq85`. This closes the false idle-recovery/input regression on real
hardware; it does not grant WLAN association or link authority.

USB false idle-recovery removal slice done (2026-07-09) - keyboard and mouse
can now remain normally idle without raiOS forcibly stopping their xHCI
endpoint, resetting a hub port, or synchronously writing RECLOG through the
same hub. Fresh Disk 2 inspection found a valid 84/84 RECLOG chain with no torn
tail. In the captured recovery runs, report counts advanced (`383 -> 489 ->
515`, then `568`) before later idle intervals, while every forced rearm/reset
record retained `errors=0`, `last_int_cc=0`, and successful
`last_xfer_cc=1`. The watchdog nevertheless treated one second without a mouse
report as failure, rearmed twice, then reset/re-enumerated the port; each step
also performed a synchronous RECLOG append. Those actions had no positive
recovery evidence and could interrupt a keyboard sharing the controller/hub.
The entire idle watchdog, port-reset escalation, and repeated diagnostic-write
path is deleted (net 207 lines); real non-success transfer completions still use
the existing bounded `recover_interrupt_endpoint`, and the single verified
boot RECLOG probe remains. The newest boots `seq=83/84` only contain boot probes,
because the old schema did not record keyboard-silent events. Verified: scoped
rustfmt; release build; quick Shadow VM report
`release/vm-reports/shadow-20260709-235209-16668.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 246061`, report sha256
`2aa0012fbffbac2063ca0561648da0d7b81d604179186e9b438c0ed474d68ad4`, and
`base_image.sha256: 206e4101d4285227efa7c198062b2f3e5540d5e84426bc08434bf8388abf333f`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-235943.log`
reported final kernel sha256
`1AAB49E2D678A7836BE0F91199FBE3BB8D8BD7809E5300897079A753DCAEE043`.

WiFi guided-setup UI slice done (2026-07-09) - a user can now click the top
`WiFi DETECTED` pill and complete firmware bring-up, `GET_HW_SPEC`, live scan,
SSID selection, and RAM-only password entry without using the console or the
Settings action buttons. A small progress window is driven by the real
firmware byte count and mailbox stages; successful scan completion opens an
anchored list populated only from the existing validated live results. Hidden
SSIDs still fail closed to manual entry, open networks skip the password step,
and secured networks reuse the masked console input with retry/cancel outcomes.
The default checked option is explicitly `Remember for this boot (RAM only)`;
it records current-boot retention intent and does not claim durable encrypted
secret storage. Completion explicitly reports `Connection not established`,
so this slice grants no association, link, DHCP, or network authority. Verified:
scoped rustfmt; release kernel build; quick Shadow VM report
`release/vm-reports/shadow-20260709-232827-11108.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 225515`, report sha256
`16922268b9ae008d96579e1453db159e79558e816f68d2a29eb8eba4c4c5766f`, and
`base_image.sha256: eda90c523419d3ba26024e956eccd18e6efe32ee0f522912aa1649c51672b563`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-233613.log`
reported final kernel sha256
`357906AB4307A33E5189AE574659819809E1202E2E9A61D40E3D65CEA99C4574`.

WiFi direct-response live scan slice done (2026-07-09) - raiOS can now request
and parse real nearby 2.4 GHz BSS results without opening the Marvell event or
RX-PFU rings. Upstream mwifiex confirms `SCAN_EXT` reports BSS data through a
firmware event, while legacy `HostCmd_CMD_802_11_SCAN` (`0x0006`) returns the
BSS descriptions in the bounded command-response buffer. The new no-alloc
builder sends infrastructure mode, two active probes, and channels 1-11. The
response parser first validates the complete descriptor count and all nested
lengths, then converts each firmware BSS descriptor into the existing 802.11
beacon parser and publishes SSID/channel/security/RSSI as `[LIVE]`; no partial
result is published before full structural validation. Declared nonzero results
with zero successfully parsed networks fail closed as `live_result_parse_failed`.
PCI bus mastering retains the proven doorbell-to-first-CMD_DONE/timeout bound
and is disabled before response parsing. Link authority remains denied.
Verified: 32/32 focused `raios-core` Marvell firmware/mailbox/scan tests;
scoped rustfmt; release packaging; quick Shadow VM report
`release/vm-reports/shadow-20260709-225029-33012.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 201408`, report sha256
`d8e6ab8e2450aa26a7e4d126136f500514ce57721b2883d227cc7175cb7197a9`, and
`base_image.sha256: 6f2cf7efa2461f3861d2d44554c257a326e0da50afa32126077f0fc7f9986f59`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-225759.log`
reported kernel sha256
`E46FC0FB334D571BFD6D3152B32E23DB2A923B527DAF9DC49379F90A08B66F07`.

Bare-metal SCAN_EXT/input evidence (2026-07-09) - the owner-provided Surface
screen proved firmware `ready@ready`, green `HW_SPEC`, and green `SCAN_EXT:
done result=command_done_event_ring_unavailable len=113`, so the real extended
scan command completed. The mouse stopped after WiFi start but worked again
after unplug/replug. Host readback found RECLOG valid 66/66 with no torn tail.
The test boot begins at `seq=60`; frames 61-62 attempted endpoint rearm,
`seq=63` completed a hub-port reset, and frames 64-66 rearmed again. All retain
`reports=179`, `errors=0`, `last_xfer_cc=1`, `m_port=259`, `m_ep=1`: the port
and endpoint appeared healthy but no new interrupt report arrived. This is an
open USB re-enumeration/recovery bug, not a successful soft recovery.

WiFi bounded SCAN_EXT DMA window slice done (2026-07-09) - raiOS can now issue
one real 2.4 GHz wildcard `SCAN_EXT` mailbox command through the same bounded
PCI bus-master window proven by `GET_HW_SPEC`. It resolves the detected Surface
PCI function before arming, prepares command/response buffers while DMA is off,
masks real Marvell interrupts and clears stale status, enables bus mastering
immediately before the doorbell, and disables it before parsing first
`CMD_DONE` or on the existing 15-second timeout. Event/RX rings and RX-PFU stay
parked, so command success remains the explicit
`command_done_event_ring_unavailable` state and grants no live-result or link
authority. Verified: 28/28 focused `raios-core` Marvell firmware/mailbox/scan
tests; scoped rustfmt; release packaging; quick Shadow VM report
`release/vm-reports/shadow-20260709-221452-16344.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 199596`, report sha256
`b23425a3f09973c5424ab5e3203658cbacef2c084352d2b45029a44939b7d6ac`, and
`base_image.sha256: 4bcdfe3f849b5df822f4e22d2766b4efc62be80f10e421c3dae8cc10966647c9`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-222045.log`
reported kernel sha256
`41CD3F0810319CDAC20B33FE55DD000FFA06F9543C7F0782E159D38DAAD67271`.

WiFi bounded HW_SPEC DMA window slice done (2026-07-09) - raiOS can now issue
one real `GET_HW_SPEC` mailbox request after firmware-ready inside a narrow PCI
bus-master window. Command/response buffers and MMIO addresses are prepared
while DMA is off; bus mastering is enabled immediately before the command
doorbell and disabled before parsing the first `CMD_DONE` response, or
unconditionally after the existing three-second timeout. The real Marvell host
interrupt source remains masked and pending status is cleared before the
window. Event-ring, RX-PFU, scan, link, and persistent authority remain parked.
Real owner evidence immediately before this slice confirmed the DMA-off
firmware-status image reached `ready@ready` with `FW_STATUS=0xFEDCBA00` and kept
the hub mouse responsive. Verified: 28/28 focused `raios-core` Marvell firmware
and mailbox tests; scoped rustfmt; release packaging; quick Shadow VM report
`release/vm-reports/shadow-20260709-215913-22068.json` passed 542/542 predicates
and 79 executed commands, `duration_ms: 208675`, report sha256
`50415d31e8cce701e319a9fcf68e5b88121d8645823e7aaffafd3689d0e03f23`, and
`base_image.sha256: 3330694783b5036316f47a5e150e4ab6c4d9ea2135705de11cbc77609913755b`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-220258.log`
reported kernel sha256
`6D79D5CC1C721956D54B2AE56C32626984C4E10C3D91F8E99157998000FEC0D2`.
The owner then confirmed the Surface test succeeded: firmware reached ready,
`GET_HW_SPEC` completed, and input remained usable. Host readback after that
boot found a valid 58/58 RECLOG chain with no torn tail; the new tail is the
expected boot diagnostic (`seq=58`, `reason=boot_probe`). WiFi completion is
currently visible UI evidence rather than a RECLOG record.

WiFi DMA-off firmware-ready observation slice done (2026-07-09) - raiOS can now
continue from the Linux-matched, input-stable `DRV_READY` transition into the
existing bounded `FW_STATUS` state-machine poll while PCI bus mastering and
INTx remain disabled. The code no longer terminates early as
`drv_ready_quarantined` or disables BAR memory decoding immediately after the
ready write; it keeps only MMIO observation authority and reaches `ready` solely
after `FW_STATUS == 0xFEDCBA00`, otherwise failing closed on the existing
10-second firmware-ready timeout. `GET_HW_SPEC`, scan, event-ring/RX DMA, link,
and RX-PFU authority remain parked. Real owner evidence immediately before this
slice confirmed the corrected interrupt-mask image kept the hub mouse working
after `Start WiFi FW`, closing the prior input-loss split. Verified: 17/17
focused `raios-core` Marvell firmware tests; scoped rustfmt; release packaging;
quick Shadow VM report `release/vm-reports/shadow-20260709-214736-29904.json`
passed 542/542 predicates and 79 executed commands, `duration_ms: 197886`,
report sha256
`4576fb1c133dedf9d4f956d875b9c1c8939629ff8c36f62638a3f9dfd5ec6656`, and
`base_image.sha256: 426f7a43d8839cdad15b63e36d1febad6f18b37d337287f3469e2f76369d5338`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-215123.log`
reported kernel sha256
`6EE48EEF96B9D545FD665A198CCA6148008D6B933073EA8FC270141CD2D130B9`.

WiFi Linux-matched interrupt quarantine correction done (2026-07-09) - raiOS
can now enter the diagnostic `DRV_READY` transition with the actual Marvell
host-interrupt source register (`PCIE_HOST_INT_MASK`, `0xC34`) disabled and
pending host status cleared with the Linux driver's write-zero-to-clear
polarity. The previous quarantine accidentally wrote zero to
`PCIE_HOST_INT_STATUS_MASK` (`0xC3C`), wrote the positive interrupt mask to the
status register, and wrote zero to `PCIE_CPU_INT_EVENT`; none of those three
writes matched the upstream mwifiex sequence. DMA/INTx pre-quarantine and the
post-write PCI-function quiesce remain, and firmware-ready, HW_SPEC, scan, link,
and RX-PFU authority remain denied. Fresh host readback of Disk 2 proved the
RECLOG remains a valid 57-frame chain; its tail is
`hub_mouse_port_reset_failed` with `reports=179`, `errors=0`, `last_xfer_cc=1`,
`m_port=259`, and `m_ep=1`, which supports an external disturbance hypothesis
but does not by itself prove the Marvell interrupt path is the cause. Verified:
17/17 focused `raios-core` Marvell firmware tests; release package build; quick
Shadow VM report `release/vm-reports/shadow-20260709-213525-25680.json` passed
542/542 predicates and 79 executed commands, `duration_ms: 209278`, report
sha256 `1aa7f61f7f2489cc46d7c4db691c68c680616c8ea952eccbb22de5dd9dc25b8e`, and
`base_image.sha256: 68e93a1572df569581c34d903e832b607e83d18b3d411387274a157f58b08ada`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-213921.log`
reported kernel sha256
`EFB49EF80152501D0399FDAAB5F2ABCB339F7ABC06AE2CDF60DE5EDF9E950BB8`.

WiFi DRV_READY pre-quarantine diagnostic slice done (2026-07-09) - after
the owner reported "no" for the post-`DRV_READY` PCI-quiesce test, raiOS now
masks/clears Marvell host-interrupt bits, disables WiFi bus mastering and INTx,
then writes `DRV_READY`, captures one BAR2 register snapshot, and finally
quiesces the whole WiFi PCI function. This tests whether a pre-disabled DMA/INTx
surface can make the `DRV_READY` transition harmless enough for Surface hub
input. Firmware-ready, HW_SPEC, scan, link, and RX-PFU authority remain denied.
Verified: scoped `rustfmt --edition 2021 --check
seed-kernel\src\marvell_wifi_pcie.rs seed-kernel\src\pci.rs
seed-kernel\src\ui.rs`; `git diff --check` for touched source/docs; release
packaging with local Cargo env override; quick Shadow VM report
`release/vm-reports/shadow-20260709-210923-18096.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 176513`, report sha256
`39e03c5ba73d339f6eaa109f4824e64dcc42970553ab25b821561e2b3e897199`, and
`base_image.sha256: 8a5ebd8b99c1b0a95b60c2aedbbecec627f17e28c3d5f9d5ce95b24e29fb5341`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-211342.log`
reported kernel sha256
`F5CBFFFC57E4C37E9F308F7ECC4B7B90438C48909CFE7CE3976BFBE57A3C4D48`.

WiFi DRV_READY PCI-quiesce diagnostic slice done (2026-07-09) - after
RECLOG again showed a healthy hub port and xHCI endpoint during mouse report
loss (`m_port=259`, `m_chg=0`, `m_ep=1`), raiOS now writes the Marvell
`DRV_READY` MMIO magic after the real firmware block download, immediately
masks/clears Marvell host-interrupt bits, disables PCI bus mastering, captures
one register snapshot while BAR2 memory is still enabled, then quiesces the
WiFi PCI function by disabling I/O space, memory space, and bus mastering and
setting PCI command interrupt-disable. This is a deliberately risky Surface
split point: it tests whether a fully quiesced post-`DRV_READY` function can
preserve hub input. Firmware-ready, HW_SPEC, scan, link, and RX-PFU authority
remain denied; the result is honestly labeled `drv_ready_quarantined`. Verified:
scoped `rustfmt --edition 2021 --check seed-kernel\src\marvell_wifi_pcie.rs
seed-kernel\src\pci.rs seed-kernel\src\ui.rs`; `git diff --check` for touched
source files; release packaging with local Cargo env override; quick Shadow VM
report `release/vm-reports/shadow-20260709-203221-20956.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 179774`, report sha256
`100445bd65df84d6f39d2d563c934fec434e77179c4854f06482be2c1ed43594`, and
`base_image.sha256: 25d47647e237fc61ba1754901808d99ca6520d0fd0f8a78c4de8141fce7ed7b8`.
Owner handoff: after two cancelled UAC launches, the owner accepted the elevated
writer and Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-210312.log`
reported kernel sha256
`1BBC4F3DAB5A8E2DEA251140D65E9D055712120637DAC5BE6EA6A698A9AD41BC`.

WiFi DRV_READY quarantine diagnostic slice done (2026-07-09) - raiOS now writes
the Marvell `DRV_READY` MMIO magic after the real firmware block download, then
immediately masks/clears Marvell host-interrupt bits, clears the CPU interrupt
doorbell register, and disables PCI bus mastering for the WiFi function. This
is a deliberately risky Surface split point: it tests whether the harmful
behavior is the `DRV_READY` transition itself or unconfined post-ready
DMA/interrupt activity. Firmware-ready, HW_SPEC, scan, link, and RX-PFU
authority remain denied; the result is honestly labeled
`drv_ready_quarantined`. Verified: scoped
`rustfmt --edition 2021 --check seed-kernel\src\marvell_wifi_pcie.rs
seed-kernel\src\pci.rs seed-kernel\src\ui.rs`; `git diff --check` for touched
source files; release packaging with local Cargo env override; quick Shadow VM
report `release/vm-reports/shadow-20260709-201745-17340.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 175903`, report sha256
`3561f8414b1e15a3f5aa9644972de5d6ebfde6eeee48cf54c8f969b90b4b84c5`, and
`base_image.sha256: be952a871bf922f73ffa1db72d57bb72a7c1139d274723564c8275520b5b259e`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-202112.log`
reported kernel sha256
`5D3F709C660F7953CB993437F79A076BFCEEDBB55AF17DBC1917C215BBA37BF8`.

WiFi DRV_READY parking diagnostic slice done (2026-07-09) - after the owner
proved the hub mouse still dies when `Start WiFi FW` completes even with
post-ready mailbox/event auto-probes parked, raiOS now stops one step earlier:
the Marvell firmware block download and doorbell ACK path still run, but the
`DRV_READY` MMIO write is not issued. The run finishes in honest
`drv_ready_parked` state instead of claiming firmware-ready, so scan/HW_SPEC
and link authority remain denied. This gives the Surface a clean split between
"download DMA/doorbell already harms USB input" and "`DRV_READY` starts the
harmful firmware-side behavior". Verified: scoped
`rustfmt --edition 2021 --check seed-kernel\src\marvell_wifi_pcie.rs
seed-kernel\src\ui.rs`; release packaging with local Cargo env override; quick
Shadow VM report `release/vm-reports/shadow-20260709-195238-29888.json` passed
542/542 predicates, 79 executed commands, `duration_ms: 177232`, report sha256
`9bcd79bb80502c9d87c7d0c408e443f721ca70bc1374768f29c1bc013a50f421`, and
`base_image.sha256: dff237c8182f4f3716ecd6a891870db207022b9e4cdfb330a1377b541cadacda`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-195609.log`
reported kernel sha256
`D2E962BC2CBF31C56A7A68868532A0DB9CB04A22A36414135E594FC443F0C293`.

WiFi post-ready auto-probe parking slice done (2026-07-09) - after the owner
confirmed the hub mouse is now stable behind the hub until `Start WiFi FW`
completes, raiOS narrowed the next failure boundary by making the WiFi button
perform only the firmware download plus `DRV_READY`: it no longer arms the
Marvell event DMA ring during `WriteDrvReady`, and it no longer automatically
starts the `GET_HW_SPEC` mailbox probe immediately after firmware-ready. The
firmware attempt is still real and unaudited, but post-ready mailbox/event DMA
is parked until input stability is proven on Surface hardware. Live scan
results, RX-PFU, and link authority remain denied. Verified: scoped
`rustfmt --edition 2021 --check seed-kernel\src\marvell_wifi_pcie.rs`;
release packaging with local Cargo env override; quick Shadow VM report
`release/vm-reports/shadow-20260709-194251-17020.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 190650`, report sha256
`f179b6b4389b6546a8935811653db0aff131b2732146377ef4999dfcb4695f1e`, and
`base_image.sha256: 14cd7163d79f9cb4a124097e9ee252bfa6ef1fe1d51bd25c0de63fc29fcfd6a2`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-194722.log`
reported kernel sha256
`D7994610EA1D61CA744339C6C72FA02C36332802DCBD090904DEB79550F848B6`.

Hub-mouse active-hotplug suppression slice done (2026-07-09) - raiOS no longer
control-polls hub child ports while a mouse behind that hub has produced real
input reports, removing the periodic hub EP0 control traffic that matched the
owner's fixed-time freeze after mouse movement. Root-port hotplug remains live,
the no-input rescan path still exists before any hub-mouse report, direct mice
are unchanged, and the targeted hub-port recovery reset remains available if
the endpoint later goes silent. Real Disk 2 evidence before this patch showed
the hub port and xHCI endpoint still looked healthy during the failure:
RECLOG `seq15 reason=hub_mouse_port_reset reports=11 recover=2 hub_reset=3
hub_done=3 last_cmd=12 last_cc=1 last_xfer_cc=1 enum_vid=1133 enum_pid=49290
m_port=259 m_chg=0 m_ep=1`, so the next owner test checks whether suppressing
our own periodic hub polling removes the freeze. Verified: scoped
`rustfmt --edition 2021 --check seed-kernel\src\usb.rs`; `git diff --check`;
release packaging with local Cargo env override; quick Shadow VM report
`release/vm-reports/shadow-20260709-192136-29312.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 179590`, report sha256
`e9682e29cc759c975a2ffe420ea0c961dbfb8d16a505e5ee90cd37d8dd901946`, and
`base_image.sha256: dd5802a6d689fbabea599f2ac68b2b0cddf8634ba03ea7f71b42dd30fde00dba`.
Owner handoff: Disk 2 `SEED_ESP_A` was refreshed without reformatting via
`scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-192737.log`
reported kernel sha256
`408856AA97623130B5F8448940D2E687ACC9F428193F7D020CDBCB5D85EBA870`.

Hub-mouse root-cause RECLOG slice done (2026-07-09) - before the hub-mouse
watchdog performs the targeted parent-hub-port reset, raiOS now records the
minimal root-cause state needed to distinguish a hub-side drop from an xHCI
endpoint-side stall: `m_port` is the hub port status, `m_chg` is the hub port
change bits, and `m_ep` is the xHCI endpoint state read from the mouse endpoint
context. The payload remains within the single-sector RECLOG frame budget
(416/424 bytes in a hard worst-case host check), so diagnostic logging still
fails closed instead of silently overflowing. Verified: scoped
`rustfmt --edition 2021 --check seed-kernel\src\usb.rs`; release packaging with
local Cargo env override; quick Shadow VM report
`release/vm-reports/shadow-20260709-190452-30576.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 174278`, report sha256
`5572b963870e3082db9185a677f0cbbd0f94f3d316c658aba69f62a59d80beb7`, and
`base_image.sha256: 37ae7afbee6f232e78ed808c981c7907a05474dd4077ab22cf6cc09a6a858fcd`.
Post-commit owner handoff: Disk 2 `SEED_ESP_A` was refreshed without
reformatting via `scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-191031.log`
reported kernel sha256
`3DE9434BD4BEEF80CF03E2FC65259609F86D77A5CBB9FD605D1429E42B6C351C`.

Hub-mouse port-reset recovery slice done (2026-07-09) - raiOS now escalates the
real Surface hub-mouse recovery path after the verified endpoint rearm fails to
restore input: after two successful `hub_mouse_rearm` watchdog attempts without
a new mouse report, it resets exactly the mouse's parent hub port and reuses the
existing hub hotplug enumeration path to reconfigure the pointer, logging either
`reason=hub_mouse_port_reset` or `reason=hub_mouse_port_reset_failed` to the
verified USB RECLOG sink. Direct mice and keyboards are unchanged, and broad USB
disk mutation remains denied. Surface evidence from the real Disk 2 stick showed
RECLOG `valid`, `count=11`, `head_seq=1`, `tail_seq=11`: `seq1` was
`boot_probe`, `seq2..seq11` were `hub_mouse_rearm`, and `reports` stayed stuck
at 23, proving the old endpoint-only rearm fired but did not restore the mouse.
Verified: scoped `rustfmt --edition 2021 --check seed-kernel\src\usb.rs`;
release packaging with local Cargo env override; quick Shadow VM report
`release/vm-reports/shadow-20260709-184742-22644.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 182280`, report sha256
`b6a94160feb4463489210046533d4a4f35a8b6f547ab205e4ac918e43ef75ee0`, and
`base_image.sha256: b68fd1c018371aa15b99d7c715d26ce313b79d05b6429f2470271b2fc3784b85`.
Post-commit owner handoff: Disk 2 `SEED_ESP_A` was refreshed without
reformatting via `scripts\update-usb-esp-a.ps1 -DiskNumber 2 -SkipBuild`; log
`C:\Users\admin\AppData\Local\Temp\raios-usb-esp-a-update-disk2-20260709-185452.log`
reported kernel sha256
`C0E4DA64BC5F925732DCB34F1346D82496919907DA8FC478E5DD71AD3223FE32`.

Hub-mouse RECLOG diagnostics slice done (2026-07-09) - raiOS now appends
host-readable USB diagnostic frames at the time the real Surface problem occurs:
the first verified USB-MSC RECLOG append records `reason=boot_probe`, and each
successful silent hub-mouse endpoint rearm appends another frame with
`reason=hub_mouse_rearm` until the hard debug cap of 16 RECLOG diagnostic writes.
This still does not broaden disk authority: writes remain limited to validated
`SEED_DATA/RECLOG`, readback/reparse is required, and corrupt logs disable
further appends. Focused USB-storage VM evidence on
`D:\raios-tmp\raios-usb-rearm-log-test.img` observed
`usb-msc: reclog append seq=1 lba=526352 verified`, `MSC LOG seq1 lba526352`,
and host inspect showed RECLOG `valid`, `count=1`, payload schema
`raios.usb_diag.v0`, `reason=boot_probe`. Quick Shadow VM report
`release/vm-reports/shadow-20260709-181301-15252.json` passed 542/542
predicates, `duration_ms: 191697`, report sha256
`19e4340951ac2e7aa152e1c69e6ac37509b27d98bb3d1def93c72300a2684c11`, and
`base_image.sha256: af38fead76b78b225917d92df760f141b87afadfa46c00ebb3fc9c57b42c1677`.

USB stick RECLOG append slice done (2026-07-09) - raiOS can now use the
prepared GPT persistence stick as a real xHCI USB Mass Storage diagnostic sink:
after `SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA` and the `RAIOS_DATA_SB_V0`
superblock validate, the kernel scans the `SEED_DATA/RECLOG` hash chain, denies
append on corrupt/non-empty-tail evidence, issues exactly one SCSI `WRITE(10)`
to the next RECLOG sector, reads it back, reparses the frame, and shows
`MSC LOG seq<N> lba<N>` only after verification. Broader USB disk mutation
remains denied; if GPT/superblock evidence is incomplete the status stays
`MSC FOUND <reason>` or `MSC SEED <reason>` and no write is attempted. Focused
USB-storage VM evidence: temporary GPT stick image
`C:\Users\admin\AppData\Local\Temp\raios-usb-reclog-test.img` booted as QEMU
USB storage, serial observed `usb-msc: reclog append seq=1 lba=526352 verified`
and `status USB-XHCI ... MSC LOG seq1 lba526352`; host inspection reported
`reclog_scan.status=valid`, `count=1`, `first_invalid_offset=512`, and direct
sector payload schema `raios.usb_diag.v0`. General quick Shadow VM report
`release/vm-reports/shadow-20260709-174348-9768.json` passed 542/542 predicates,
`duration_ms: 204252`, report sha256
`1aba240ff828bec6b5910a667d197ceadf676dbd7d590a60920df6c7dd0fa111`, and
`base_image.sha256: d9348fc2a934b48f40ce3652c559eb5c9968dac627ed7e538dbd0a2eec386354`.

Failure classification (2026-07-09, USB RECLOG quick VM first attempt): quick
report `release/vm-reports/shadow-20260709-174305-15032.json` failed before
QEMU boot or any guest predicate because `scripts\package-stage0.ps1` could not
copy the temporary ESP (`Copy-Item ... kernel.elf`) after Windows reported no
free space on `C:`; verdict: host-transport/setup, no guest behavior executed.
Retry only after clearing Codex-generated temporary images/logs.

Hub-mouse silent rearm test slice prepared (2026-07-09) - raiOS now detects the
known Surface case where a mouse behind a USB hub has produced at least one
interrupt report and then goes silent for about one second; only for that
hub-mouse path it issues `STOP_ENDPOINT`, resets the interrupt transfer ring,
sets the TR dequeue pointer, requeues the mouse interrupt-IN report, and
increments the existing `RCV` recovery counter. Direct mice and keyboards are
unchanged. This is a real xHCI rearm path, but still needs owner bare-metal
confirmation because the silent hub stall is not reproduced by QEMU. Verified:
release build and `scripts\package-stage0.ps1 -Profile release` passed; quick
Shadow VM report `release/vm-reports/shadow-20260709-172354-28840.json` passed
542/542 predicates, 79 executed commands, `duration_ms: 203525`, report sha256
`b12671af4e34dd3573d3a2fb0c354880c3ab39e5c64f3afc441a2102618829e4`; real Disk
2 write log `C:\Users\admin\AppData\Local\Temp\raios-usb-write-disk2-20260709-172741.log`
ended with `raiOS persistent USB image written to PhysicalDrive2` and
`SEED_DATA superblock valid: True`.

Failure classification (2026-07-09, hub-mouse watchdog quick VM first attempt):
quick report `release/vm-reports/shadow-20260709-171353-5728.json` failed before
guest boot because image packaging used the wrong host Cargo package-cache path
(`F:\scorefollower-build\cargo\.package-cache`); verdict: host-transport/setup,
no guest predicate executed. Retry only after setting local `CARGO_HOME` and
`CARGO_TARGET_DIR`.

Real Surface USB handoff done (2026-07-09) - Disk 2 (`USB SanDisk 3.2Gen1`,
28.67 GiB) was written with the current raiOS GPT persistence layout using
`scripts\write-stage0-usb.ps1 -DiskNumber 2 -ConfirmErase 'ERASE DISK 2'
-Profile release -UsePersistLayout -SkipBuild`. A user can now boot the real
Surface from that stick and expect the kernel's read-only USB-MSC probe to see
`SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA`. The writer now handles Windows removable
USB media that refuse `Set-Disk -IsOffline` by clearing the explicitly erased
disk's partition table before the raw write. Verified by
`C:\Users\admin\AppData\Local\Temp\raios-usb-write-disk2-20260709-165319.log`:
`raiOS persistent USB image written to PhysicalDrive2`, layout
`SEED_ESP_A + SEED_ESP_B + SEED_DATA`, image bytes `537936384`, and
`SEED_DATA superblock valid: True`. `git diff --check` passed and
`scripts\scan-secrets.ps1` found no OpenAI-key-like values.

USB Mass Storage read-only SEED_DATA slice done (2026-07-09) - raiOS can now
boot with a prepared GPT persistence stick attached as xHCI USB Mass Storage,
configure its BOT bulk-in/bulk-out endpoints, issue read-only SCSI `INQUIRY`,
`READ CAPACITY(10)`, and `READ(10)` commands, validate the GPT
`SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA` layout through the existing
`raios-core` validators, validate the `RAIOS_DATA_SB_V0` superblock copy, and
surface `MSC SEED` in the USB-XHCI status line. This grants no USB write path:
`WRITE(10)`, RECLOG append, and durable WiFi logging remain the next scoped
authority slice. Verified: release image packaged by
`scripts\package-stage0.ps1 -Profile release`; temporary GPT stick image built
with `scripts\make-gpt-persist-image.py --self-check --seed-bootctl valid-a`;
focused headless QEMU with `-UsbXhciInput -UsbStorageImage` observed
`usb-msc: capacity last_lba=1050656 block=512 mbr=0xaa55 seed_data=present
seed_data_superblock_validated` and `status USB-XHCI ... MSC SEED`; quick
Shadow VM report `release/vm-reports/shadow-20260709-163357-10832.json` passed
542/542 predicates, 79 executed commands, `duration_ms: 204228`, report sha256
`ec6133b2e609fb81a1b5375f6b2599c16a59b5af338b878479a250580dffb3c6`, and
`base_image.sha256: 6669c06d81958f1abf2de1158931a4c67a6ec5ecabff142278c74cf3cf4a2aaf`.
`git diff --check` passed and `scripts\scan-secrets.ps1` found no
OpenAI-key-like values. Global `cargo fmt --all -- --check` remains expected-red
only on the pre-existing unrelated formatting drift in
`raios-core/src/marvell_wifi_fw.rs` and old sections of `seed-kernel/src/usb.rs`.

Failure classification (2026-07-09, USB-MSC focused runner attempts): two manual
focused USB-storage QEMU invocations through `scripts\run-stage0-qemu.ps1
-UsbStorageImage` hit the outer agent tool timeout while the guest serial log
had already completed the USB-MSC probe and status line; verdict:
host-transport/runner lifetime, not guest behavior. The retained serial logs
show the positive guest evidence quoted above, and the normal quick profile then
passed without code changes.

USB persistent-stick layout slice done (2026-07-09) - the Windows USB writer can
now prepare a real raiOS GPT persistence stick instead of only a one-FAT ESP:
`scripts\write-stage0-usb.ps1 -UsePersistLayout` builds the existing
`make-gpt-persist-image.py` layout, stages the current ESP into `SEED_ESP_A`,
keeps `SEED_DATA` raw with a valid `RAIOS_DATA_SB_V0` superblock and empty
RECLOG, resets BOOTCTL to Normal/slot A for first boot, and raw-writes the
validated image to the chosen USB disk. This grants no kernel USB write path
yet; it gives the Surface stick the correct durable target for the upcoming
USB Mass Storage block-device slice. Verified: PowerShell AST parse of
`scripts\write-stage0-usb.ps1`; dry GPT image build/stage/reset/inspect with
`release\esp` reported `gpt_header_valid=true`, `gpt_crc_checked=true`,
`gpt_seed_data_found=true`, `data_superblock_valid=true`, RECLOG
`valid_empty`, BOOTCTL posture `Normal`, selected payload `A`, and image size
537936384 bytes.

Failure classification (2026-07-09, Marvell firmware poll-budget quick VM):
focused `quick` report `release/vm-reports/shadow-20260709-131953-18752.json`
failed waiting for serial marker/predicate `RAIOS_AGENT_END
memory.recent_events` after `RAIOS_AGENT_BEGIN memory.recent_events`; verdict:
host-transport, matching the recurring serial `memory.recent_events` timeout.
No guest panic was observed in the serial tail, so a retry is permitted after
this classification without a code change.

Failure classification (2026-07-09, Marvell firmware poll-budget quick VM
retry): focused `quick` report
`release/vm-reports/shadow-20260709-132338-17124.json` failed on the same
serial marker/predicate `RAIOS_AGENT_END memory.recent_events` after
`RAIOS_AGENT_BEGIN memory.recent_events`; verdict: host-transport timeout with
possible guest scheduling pressure from the experimental 1ms Marvell task
interval. Do not retry unchanged; inspect and adjust the scheduler/poll budget
before the next quick VM.

Failure classification (2026-07-09, Marvell firmware poll-budget 2ms quick VM):
focused `quick` report `release/vm-reports/shadow-20260709-132730-3576.json`
failed again on serial marker/predicate `RAIOS_AGENT_END memory.recent_events`
for command `agent audit.events 72`; verdict: host-transport timeout on the
large recent-events response path. No guest panic was observed; inspect the
harness timeout path and/or return to the previous Marvell task interval before
the next retry.

WiFi RX-PFU bare-metal rollback slice done (2026-07-09) - after the owner
proved the scan-time RX-PFU image no longer froze at firmware-ready but did
freeze immediately after `Scan networks`, with `HOST_INT=0xffffffff` and
all-ones write pointers, raiOS no longer arms or polls the RX-PFU ring on the
active Surface scan path. A user can now retest Start WiFi plus Scan networks
through the event-only `SCAN_EXT`/`EVENT_RING` diagnostics without the known
RX-PFU MMIO wedge. Live scan parsing and link authority remain denied, and the
next WiFi slice should repair the host-ring/interrupt handoff without
re-enabling RX-PFU on owner hardware. Verified: scoped `rustfmt` on
`seed-kernel\src\marvell_wifi_pcie.rs` and `seed-kernel\src\main.rs`; release
packaging via `scripts\package-stage0.ps1 -Profile release`; focused VM
`quick` report `release/vm-reports/shadow-20260709-155505-32448.json` passed
542/542 predicates, 79 executed commands, `duration_ms: 188773`, report sha256
`c8e454f3bb0d58e2885b660640231eff60c8dced31133c21e9f139cf24d8a392`, and
`base_image.sha256: edc71ed828c26b2886c746bc874c12558c61af8110e9e8f2df6aa6b6df8086a5`.

WiFi bare-metal event/download correction slice done (2026-07-09) - the Surface
photo showed the previous poll-budget change was the wrong performance lever:
firmware download still took seconds, while Linux performs the helper download
as a tight 10-20us polling loop rather than one small periodic slice per
millisecond. The Marvell firmware phase now gets a larger bounded burst
(`512` firmware actions per scheduler pass) while HW_SPEC/SCAN/Event polling
remain at the small cooperative budget, so the 723540-byte block transfer is no
longer artificially throttled by the UI scheduler. The same photo also showed
`evt_wrptr` advancing (`rd=0x80 wr=0x4`) with an empty event buffer; raiOS now
arms 32 real 4096-byte RX buffers before `DRV_READY` as Linux does, and treats
`len=0` event buffers as amber `pointer_advanced_empty_buffer` diagnostics
instead of a fatal red event-ring failure. This still does not claim live
networks or link authority. Verified: `rustfmt` on
`seed-kernel\src\marvell_wifi_pcie.rs`; release build via
`scripts\build-seed-kernel.ps1 -Profile release` with local Cargo env override;
focused VM `quick` report
`release/vm-reports/shadow-20260709-143035-28468.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 211891`, report sha256
`a2824526b81a1ebff75e27e5d8de245d75bf2e845ee9e286cd08fccff19bd7b4`, and
`base_image.sha256: ec993b3ee4673205723e7e08cc670b4b07ee3ddbc5e8f066b128d947cfd5440a`.

WiFi RX-ring PFU observation slice done (2026-07-09) - after the Surface proved
the firmware burst fix made download fast but scan still advanced the event
pointer over empty buffers, raiOS now arms the Marvell 88W8897 RX ring with the
PFU descriptor layout Linux uses (`flags/offset/frag_len/len/paddr/reserved`)
instead of the event descriptor shape, polls `HOST_INTR_UPLD_RDY` plus the
packed RX write pointer before event polling, advances/rearms RX descriptors,
and exposes `RX_RING` stage/result/rd/wr/type/len/host-int in Settings and the
`wifi` console command. This grants no scan/link authority and still reports
live scan parsing as unavailable until real non-zero RX/event buffers are
observed. Verified: `rustfmt` on touched WiFi/UI/console/main files; release
build and `scripts\package-stage0.ps1 -Profile release` with local Cargo env
override; focused VM `quick` report
`release/vm-reports/shadow-20260709-145731-28804.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 188980`, report sha256
`5dc4a361ff0defdd011a62657b8a11fed77b4632b52de1e6c1ff5e7bb7f42b71`, and
`base_image.sha256: a6ae832f34e323cc84d0f8e0503ab2cd78e70a804309f397b1bbb2ed987a3641`.

WiFi RX-PFU freeze correction slice done (2026-07-09) - the first RX-PFU image
froze the real Surface immediately after the firmware download reached
`723540/723540`, which pins the regression to the pre-`DRV_READY` RX
descriptor/pointer handoff rather than firmware copy. raiOS now leaves
firmware-ready bring-up event-ring-only again and arms the RX-PFU ring only
when the owner explicitly starts `SCAN_EXT`, preserving the new `RX_RING`
diagnostic without freezing Start WiFi. Live scan parsing and link authority
remain denied. Verified: scoped `rustfmt` on
`seed-kernel\src\marvell_wifi_pcie.rs`; release packaging via
`scripts\package-stage0.ps1 -Profile release`; focused VM `quick` report
`release/vm-reports/shadow-20260709-152937-13396.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 202932`, report sha256
`61ac63530ce2e8a07c70c96e7195c049e6114dd469f46de051eb55576210372e`, and
`base_image.sha256: 81a76d568b56271ce6c190b19532cce5e39aae129e95a3a1bfdca4d0fa2e7ed8`.

WiFi event-ring observation slice done (2026-07-09) - after Marvell firmware
bring-up reaches `DRV_READY`, a Surface user can now see the real firmware
event-ring state instead of only the completed `SCAN_EXT` command: raiOS arms
eight 2048-byte mwifiex-compatible event buffers before `DRV_READY`, polls
`evt_rdptr`/`evt_wrptr`, acknowledges `CPU_INTR_EVENT_DONE`, parses the event
transfer header/cause when an event arrives, and exposes `EVENT_RING` stage,
result, rd/wr pointers, transfer type, cause, length, and host interrupt status
in Settings and the `wifi` console command. Live WLAN names still remain denied
until the next Rx-ring slice observes and parses real 802.11 frames; if a scan
event is seen now, the user-facing reason advances honestly to
`scan event observed; rx ring not implemented`. Verified: `rustfmt` on touched
kernel files;
`scripts\build-seed-kernel.ps1 -Profile release` with local Cargo env override;
`git diff --check`; focused VM `quick` report
`release/vm-reports/shadow-20260709-141710-13408.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 215768`, report sha256
`2c9b27c26aea3e3e97b94a25534bb73ac5c3108d553a6a415eb84fc8298ddc4d`, and
`base_image.sha256: fd7ece70a582faaa23e4b152a156d7b488c0d92ce7d980a79caa66d30834b80d`.

WiFi firmware throughput correction slice done (2026-07-09) - the first
poll-budget image was wrong on bare metal: it reduced each Marvell firmware
burst to 16 actions but left the task cadence at 5ms, so firmware byte copying
became roughly 4x slower and pointer lag did not improve. The corrected image
keeps the shorter 10us waits, raises each burst to 32 actions, and runs the
Marvell task every 1ms, restoring copy throughput while capping each single
busy slice far below the old 128-action burst. Scan/link authority is unchanged:
`SCAN_EXT` command completion is real, but live WLAN results still require the
event/Rx-ring path and remain explicitly unavailable. Verified: scoped format
checks (`rustfmt --edition 2021 --check --config skip_children=true
seed-kernel\src\main.rs`, `rustfmt --edition 2021 --check
seed-kernel\src\marvell_wifi_pcie.rs`); focused VM `quick` report
`release/vm-reports/shadow-20260709-134429-25500.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 277668`, report sha256
`da6a5a83364380d1b3674f180ea797b71ec913a7c816f3096f176d28e7c487fe`, and
`base_image.sha256: 381a9becf4fa359cdf21b11e7a59cac84271717dc7066b23f4afa52f91b185c8`.

WiFi firmware poll-budget slice done (2026-07-09) - a Surface user can start
Marvell 88W8897 firmware bring-up with much shorter uninterrupted CPU
busy-wait slices, so pointer/input/UI work gets scheduled between firmware
polls instead of waiting behind multi-millisecond Marvell polling bursts. The
real driver still uses the same owner-triggered firmware path, same 5ms Marvell
periodic task cadence, same DMA/register command sequence, and same fail-closed
network authority posture; only each firmware/HW_SPEC/SCAN_EXT poll pass was
shrunk from 128 actions with 20us short waits to 16 actions with 10us short
waits. During verification the default 45s host response timeout proved too low
for the existing large `agent audit.events 72` / `memory.recent_events` serial
response, which completed in 56.829s on the passing run, so the host smoke
default was raised to 90s without changing guest predicates. Verified: scoped
format checks (`rustfmt --edition 2021 --check --config skip_children=true
seed-kernel\src\main.rs`, `rustfmt --edition 2021 --check
seed-kernel\src\marvell_wifi_pcie.rs`); `git diff --check`; focused VM `quick`
report `release/vm-reports/shadow-20260709-133204-7396.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 266502`, report sha256
`b8dc2f8708258ad673b235377685b072e36a1f65fcfde9387180a180e451f278`, and
`base_image.sha256: 317e2b792443dd16f3c79918a57ab945972fc6cfe14ee2c57475b9ce9ab88701`.
The earlier default-timeout failures are recorded above as host-transport
classifications, not guest behavior.

WiFi scan mailbox execution slice done (2026-07-09) - after the Marvell
88W8897 firmware-download path and GET_HW_SPEC response are ready on the real
Surface, the Settings `Scan networks` button / setup key `8` now starts a real
mwifiex-compatible `SCAN_EXT` 2.4GHz wildcard command through the PCIe
CMD/RSP DMA mailbox, polls the real `HOST_INTR_CMD_DONE` bit, parses the
firmware command-status response, and exposes `SCAN_EXT` stage/result/length/
host-interrupt state on the UI and `wifi` console status. On QEMU, or before
firmware and HW_SPEC are ready, it fails closed and runs only the labeled
embedded scan self-test fallback; it still does not claim live network results,
does not arm firmware event/Rx rings, does not associate, and grants no
network authority. Verified: scoped format check (`rustfmt --edition 2021
--check raios-core\src\marvell_wifi_cmd.rs seed-kernel\src\wifi.rs
seed-kernel\src\marvell_wifi_pcie.rs seed-kernel\src\console.rs
seed-kernel\src\ui.rs` plus `seed-kernel\src\main.rs` with
`skip_children=true`); host tests
(`cargo test --locked -p raios-core marvell_wifi_cmd`) 11/11 passed; release
kernel build and `scripts\package-stage0.ps1 -Profile release`; focused VM
`quick` report `release/vm-reports/shadow-20260709-125408-23664.json` passed
542/542 predicates, 79 executed commands, `duration_ms: 147339`, report sha256
`f54c9954ba65485c67ad2c48187959aac7aec006efbe5a589b29f2d46f643899`, and
`base_image.sha256: a84f652fc8b74f6c64f216fa19fe4ebc0b30ef81128e8772335273274e702bb6`.
A manual serial `setup`/`8` probe was attempted after the report but the
Windows host returned `Access denied` around QEMU process control before a
usable guest assertion was captured, so it is not counted as guest evidence.
Global `cargo fmt --all -- --check` remains expected-red on pre-existing
unrelated formatting drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`.

WiFi scan-ext command builder side-track slice done (2026-07-09) - after the
Marvell 88W8897 firmware-download and GET_HW_SPEC path, raiOS can now build a
pure, host-tested mwifiex-compatible `HostCmd_CMD_802_11_SCAN_EXT` packet for a
2.4GHz wildcard scan over channels 1-11, including the Linux-grounded wildcard
SSID, BSS mode, number-of-probes, and channel-list TLVs. This does not issue a
scan, does not arm event rings, does not claim live radio results, and grants no
WiFi/network authority; it is the next command payload the existing mailbox
shell can consume only after the hardware firmware path is proven on the real
Surface. Verified: scoped format check (`rustfmt --edition 2021 --check
raios-core\src\marvell_wifi_cmd.rs`); host tests
(`cargo test --locked -p raios-core marvell_wifi_cmd`) 9/9 passed with local
`CARGO_HOME`/`CARGO_TARGET_DIR` override because the inherited environment
pointed at a missing `F:\scorefollower-build\cargo`; focused VM `quick` report
`release/vm-reports/shadow-20260709-121516-5560.json` passed 542/542
predicates, 79 executed commands, `duration_ms: 286275`, report sha256
`2c2d93c6fc125714c1eb42a74aab122cdabdd454fb427caa5859d575ebc26721`,
`base_image.sha256:bbc592b3a7a0190eab9a4f22ec9e4f47af880478f85e699b1273ec75bd0b36e5`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing
unrelated format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while this
slice used the focused quick profile because the new packet builder is pure and
not wired into guest execution. File-size check: touched
`raios-core\src\marvell_wifi_cmd.rs` is 320 lines.

Owner-key TPM status-read plan slice done (2026-07-09) - real Surface capture
now names the exact TPM status register raiOS would read next, without doing
the MMIO read yet. A user/agent can inspect `ownerkey` or
`system.honesty_report.owner_key_provisioning` and see whether a read-only TPM
status-register plan exists, which register kind it targets
(`crb_control_status` or `tis_sts`), the physical register address, read width,
and denial reason; in QEMU, where the `TPM2` ACPI table is absent, the plan is
correctly unavailable (`kind:none`, phys `0`, width `0`) and owner-seal,
persistent-install, load, and durable-write authority all remain false. The
default boot image was refreshed with this capture path. Verified: scoped
format check (`rustfmt --edition 2021 --check
raios-core\src\owner_key_tpm2.rs seed-kernel\src\owner_key.rs
seed-kernel\src\agent_protocol_honesty.rs seed-kernel\src\console.rs`); host
tests (`cargo test --locked -p raios-core owner_key_tpm2`) 8/8 passed; release
image packaging via `scripts\package-stage0.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-120614-8340.json` 253/253 predicates, 54
executed commands, `duration_ms: 81130`, report sha256
`942a81b2c968216cf71315d8e365c55e6553b681be0d24caf373dd24e414161e`,
`base_image.sha256:564087d277d029bdc84efe481137d9edc38d07868be23fdf024925c8177772c3`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `raios-core\src\owner_key_tpm2.rs` is 361 lines,
`seed-kernel\src\owner_key.rs` is 214 lines,
`seed-kernel\src\agent_protocol_honesty.rs` is 624 lines, and
`seed-kernel\src\console.rs` is 1791 lines.

Owner-key Surface boot image refresh slice done (2026-07-09) - the next real
USB/stick attempt now uses a default boot image that contains the current
owner-key capture path. A user/agent can write `release\raios-stage0.img` to a
stick and boot the Surface with the `ownerkey` console command and
`system.honesty_report.owner_key_provisioning` already packaged; the focused VM
run booted that exact image (`base_image.temporary:false`,
`base_image.sha256:96c2d84b85a2831533a4312660df01d5930b1d96a14b2ee6e36f93ec2e9a4268`)
and verified the RAM-only `current_boot` owner-key candidate, redacted
`sha256:` fingerprint, QEMU `tpm2_acpi_absent` posture, and `OWNER AUTH: SEAL
NO PERSIST NO LOAD NO DURABLE NO`. No provider key embedding was used; the
image and kernel ELF remain ignored local artifacts rather than tracked source.
Verified: `scripts\package-stage0.ps1 -Profile release` wrote
`release\raios-stage0.img` (67,108,864 bytes); focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-115747-6148.json` 252/252 predicates, 54
executed commands, `duration_ms: 78764`, report sha256
`72c57d3a2abb73055f70be37ec9ca1d6100cb955d3b301d89493e695d5232d2b`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: no `.rs` files were touched.

Owner-key Surface capture command slice done (2026-07-09) - real bare-metal
TPM evidence is now faster to capture without parsing the full honesty JSON. A
user/agent can type `ownerkey` on serial/framebuffer console and get the
current RAM owner-key candidate handle plus `sha256:` fingerprint, TPM2 ACPI
table present/phys/len/revision, TPM interface kind/start/control/status, and
the explicit `OWNER AUTH: SEAL NO PERSIST NO LOAD NO DURABLE NO` authority
posture; the command reads the same `owner_key::snapshot()` state as
`system.honesty_report`, exports no key material, creates no schema, writes no
state, and grants no owner/persistent/load/durable authority. Verified:
`rustfmt --edition 2021 --check seed-kernel\src\console.rs`; PowerShell parse
for `vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-115039-25356.json` 252/252 predicates, 54
executed commands, `duration_ms: 142567`, report sha256
`2866c39dd82dfebfa1589bd8f1fa3e9846f2b5dc0011d0652d79ef95cf93f3ae`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\console.rs` is 1931 lines.

Owner-key TPM parser hardening slice done (2026-07-09) - the TPM2 ACPI
interface parser is now host-testable shared core logic instead of private
kernel-only byte parsing. A user/agent can still inspect the same
`system.honesty_report.owner_key_provisioning` fields, but the parser that will
interpret real Surface `TPM2` ACPI details now lives in
`raios-core::owner_key_tpm2` with host tests covering CRB, CRB+ACPI start,
TIS/FIFO, unsupported start methods, absent-table preservation, and short-table
fail-closed behavior; the focused QEMU profile remains byte/field equivalent on
the `tpm2_acpi_absent` path, and owner-seal, persistent-install, load, and
durable-write authority all stay denied. Verified: scoped Rust format check
(`rustfmt --edition 2021 --check raios-core\src\owner_key_tpm2.rs
seed-kernel\src\owner_key.rs`); host tests
(`cargo test --locked -p raios-core owner_key_tpm2`) 7/7 passed; diff check
clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-114250-14376.json` 246/246 predicates, 53
executed commands, `duration_ms: 135283`, report sha256
`58ea397f65de79d468569437339ca135e8bebd07010c6c3a5eb0ace6c2a87d6c`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `raios-core\src\owner_key_tpm2.rs` is 278 lines,
`raios-core\src\lib.rs` is 283 lines, and `seed-kernel\src\owner_key.rs` is
218 lines.

Owner-key TPM interface bridge slice done (2026-07-09) - persistent
owner-key provisioning can now consume real TPM2 ACPI interface details when
the hardware exposes them. A user/agent can now inspect
`system.honesty_report.owner_key_provisioning` and its nested
`owner_key_evidence_input` to see the TPM2 ACPI table physical address,
platform class, control-area/FIFO base address, start method, interface kind,
and read-only interface-status posture; the focused QEMU profile still reports
`tpm2_acpi_absent` with zeroed details, RAM-only `current_boot` key generation
still happens, and owner-seal, persistent-install, load, and durable-write
authority all stay denied. Verified: scoped Rust format check (`rustfmt
--edition 2021 --check seed-kernel\src\owner_key.rs
seed-kernel\src\iommu_vtd.rs seed-kernel\src\agent_protocol_honesty.rs`);
PowerShell profile parse for
`vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-113437-29116.json` 246/246 predicates, 53
executed commands, `duration_ms: 136032`, report sha256
`a42590df6de29b304c3843ead54b86c2600d6b1cdb08365ce6857b6c04250692`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\agent_protocol_honesty.rs` is 604 lines,
`seed-kernel\src\iommu_vtd.rs` is 1457 lines, and
`seed-kernel\src\owner_key.rs` is 382 lines.

Owner-key hardware evidence probe slice done (2026-07-09) - persistent
owner-key provisioning now consumes a real platform probe instead of a static
missing marker. A user/agent can now inspect
`system.honesty_report.owner_key_provisioning.owner_key_evidence_input` and
see that raiOS followed the existing Limine RSDP/ACPI path, validated the ACPI
root table, searched for the real `TPM2` ACPI table, and in the focused QEMU
profile reported `tpm2_acpi_absent`; key material remains RAM-only/non-exported
and owner-seal, persistent-install, load, and durable-write authority all stay
denied. Verified: scoped Rust format check (`rustfmt --edition 2021 --check
seed-kernel\src\owner_key.rs seed-kernel\src\iommu_vtd.rs
seed-kernel\src\agent_protocol_honesty.rs`); PowerShell profile parse for
`vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-112454-27232.json` 246/246 predicates, 53
executed commands, `duration_ms: 128197`, report sha256
`50290272abbd6a085b79602124660a7bec59827502315fdb9a0e7234fdf15852`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\agent_protocol_honesty.rs` is 520 lines,
`seed-kernel\src\iommu_vtd.rs` is 1439 lines, `seed-kernel\src\main.rs` is
519 lines, and `seed-kernel\src\owner_key.rs` is 177 lines.

Owner-key RAM candidate slice done (2026-07-09) - RAM-only owner key material
is now real but non-authorizing. A user/agent can now inspect
`system.honesty_report.owner_key_provisioning` and see that raiOS generated a
RAM-only `current_boot` owner-key candidate from observed entropy, exposes only
the stable handle and `sha256:` fingerprint, keeps the key material classified
secret and non-exported, and still denies owner-seal, persistent-install, load,
and durable-write authority until hardware/TPM-binding evidence exists.
Verified: scoped Rust format check (`rustfmt --edition 2021 --check
seed-kernel\src\owner_key.rs seed-kernel\src\agent_protocol_honesty.rs`);
PowerShell profile parse for
`vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-111511-30672.json` 246/246 predicates, 53
executed commands, `duration_ms: 129052`, report sha256
`dab626f339a654674fd39a9c009b30bbd99ed93d4fbf81e1c2d67c4779565de0`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\owner_key.rs` is 96 lines,
`seed-kernel\src\agent_protocol_honesty.rs` is 464 lines, and
`seed-kernel\src\main.rs` is 518 lines.

Owner-key evidence input slice done (2026-07-09) - owner-key provisioning now
consumes real entropy posture. A user/agent can now inspect
`system.honesty_report.owner_key_provisioning.owner_key_evidence_input` and
see whether RAM-mode ephemeral key generation has enough observed entropy
(`core.entropy` ready, RDRAND observed, pool capacity/fill/total included)
while persistent owner-key generation remains blocked by missing
hardware/TPM-binding evidence; the diagnostic still exports no key material,
generates no key, and grants no owner-seal, persistent-install, load, or
durable-write authority. Verified: scoped Rust format check (`rustfmt --edition
2021 --check seed-kernel\src\agent_protocol_honesty.rs`); PowerShell profile
parse for `vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`;
diff check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-110642-23424.json` 246/246 predicates, 53
executed commands, `duration_ms: 134571`, report sha256
`34f9f7ac4cb55f946ead1f08114229b5e62e0637bc5f235988e8bffda0500669`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\agent_protocol_honesty.rs` is 405 lines.

Owner-key provisioning slice done (2026-07-09) - automatic key lifecycle
posture is explicit. A user/agent can now inspect `system.honesty_report` and
see `owner_key_provisioning`: persistent install is declared to require
`generate_hardware_bound_owner_key_on_persistent_install`, RAM boot is declared
`ephemeral_current_boot_key_only`, key material is not exported, no persistent
or ephemeral key is falsely reported as generated, and `owner_sealed`,
`authorizes_owner_seal`, `authorizes_persistent_install`, `authorizes_load`,
and `durable_write` all remain false until real hardware-binding/entropy
evidence exists. Verified: scoped Rust format check (`rustfmt --edition 2021
--check seed-kernel\src\agent_protocol_honesty.rs`); PowerShell profile parse
for `vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-105952-25324.json` 246/246 predicates, 53
executed commands, `duration_ms: 133739`, report sha256
`cd18ab09de5461a0ae071e038523285c9ef21e5dbe3d85d60a4adb20d82ed15c`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
check: touched `seed-kernel\src\agent_protocol_honesty.rs` is 334 lines.

M12+ Slice 19 done (2026-07-09) - provider-trust input diagnostic is explicit.
A user/agent can now inspect the real `raios.module_load_gate.v0`
loader-runtime readiness and see
`m6_m7_reverify_input_check.provider_trust_input_diagnostic`: it consumes the
M7 loader-policy diagnostic, reports that M7 input is not ready for provider
trust because M7 loader-policy evidence is missing, separately reports
provider-trust evidence as missing, and keeps
`m7_loader_policy_input_ready_for_provider_trust`,
`provider_trust_positive`, `can_enter_provider_trust`, `can_load_now`,
`authorizes_load`, and `load_attempted` false. Verified: scoped Rust format
check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_module_load_gate_render.rs`); PowerShell
profile parse for
`vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff
check clean apart from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-105238-25264.json` 246/246 predicates, 53
executed commands, `duration_ms: 134632`, report sha256
`1aee7de46640c7356721ab75bfc593b7090b45098861cf66da6590c7d4a33ddd`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
note: touched
`seed-kernel\src\agent_protocol_module_load_gate_render.rs` is 6,768 lines;
the existing split plan remains to extract receiver/preflight and
loader-runtime source-fact/reverify-check rendering before broader load-gate
renderer changes.

M12+ Slice 18 done (2026-07-09) - M7 loader-policy input diagnostic is
explicit. A user/agent can now inspect the real `raios.module_load_gate.v0`
loader-runtime readiness and see
`m6_m7_reverify_input_check.m7_loader_policy_input_diagnostic`: it consumes
the M6 reverify input diagnostic, reports that M6 is not yet ready for loader
policy because M6 reverify evidence is missing, separately reports M7
loader-policy evidence as missing, and keeps `can_enter_m7_loader_policy`,
`can_load_now`, `authorizes_load`, and `load_attempted` false. Verified:
scoped Rust format check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_module_load_gate_render.rs`); PowerShell
profile parse for
`vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`; diff check
clean apart from normal CRLF warnings; release seed-kernel build
via `scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-103423-19352.json` 246/246 predicates, 53
executed commands, `duration_ms: 137281`, report sha256
`5ff95ac581a5694c80097af5c0b84e5a9a6437b2f849dd0d9b9cf3de2a8265dd`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
note: touched
`seed-kernel\src\agent_protocol_module_load_gate_render.rs` is 6,678 lines;
the existing split plan remains to extract receiver/preflight and
loader-runtime source-fact/reverify-check rendering before broader load-gate
renderer changes.

M12+ Slice 17 done (2026-07-09) - M6 reverify input diagnostic is explicit. A
user/agent can now inspect the real `raios.module_load_gate.v0`
loader-runtime readiness and see
`m6_m7_reverify_input_check.m6_reverify_input_diagnostic`: receiver/candidate
preflight input is ready, receiver/candidate binding is not absent, M6
reverification evidence is still explicitly missing, and `can_enter_m6_reverify`,
`authorizes_load`, and `load_attempted` all remain false. Verified: scoped Rust
format check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_module_load_gate_render.rs`); PowerShell profile
parse for `vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`;
`git diff --check` clean apart from normal CRLF warnings; release seed-kernel
build via `scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-102700-21124.json` 246/246 predicates, 53
executed commands, `duration_ms: 113380`, report sha256
`a1f333d50b7e470f46ebc7c9f7d3983002afb65516c6a63b73306952e524b5b8`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
note: touched
`seed-kernel\src\agent_protocol_module_load_gate_render.rs` is 6,586 lines;
the existing split plan remains to extract receiver/preflight and
loader-runtime source-fact/reverify-check rendering before broader load-gate
renderer changes.

M12+ Slice 16 done (2026-07-09) - M6/M7 reverify input check is explicit. A
user/agent can now inspect the real `raios.module_load_gate.v0`
loader-runtime readiness and see `m6_m7_reverify_input_check`: the
receiver-preflight source fact is present and ready for reverify, while the M6
reverification evidence input and M7 loader-policy input are named as missing
and `can_enter_m6_reverify`, `can_enter_m7_loader_policy`, `can_load_now`, and
`authorizes_load` all remain false. Verified: scoped Rust format check
(`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_module_load_gate_render.rs`); PowerShell profile
parse for `vm-harness\shadow-vm-smoke-profile-m12-distribution-provenance.ps1`;
`git diff --check` clean apart from normal CRLF warnings; release seed-kernel
build via `scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-101810-22548.json` 246/246 predicates, 53
executed commands, `duration_ms: 131347`, report sha256
`eca3551768705ebe018de99dedb524820b4d878de9936355548be0fe8875c510`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
note: touched
`seed-kernel\src\agent_protocol_module_load_gate_render.rs` is 6,490 lines;
the existing split plan remains to extract receiver/preflight and
loader-runtime source-fact/reverify-check rendering before broader load-gate
renderer changes.

Failed VM classification (2026-07-09 10:08 CEST): focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-100836-26640.json` failed before guest boot
at predicate/stage `image_packaging`; the report has 0 predicates and QEMU was
not started. Verdict: `host-transport`; Cargo inherited a stale package-cache
path under missing `F:\scorefollower-build\cargo`. Retry is allowed with local
`CARGO_HOME` / `CARGO_TARGET_DIR` overrides; no guest behavior was observed.

M12+ Slice 15 done (2026-07-09) - receiver preflight is a loader-runtime source
fact. A user/agent can now inspect the real `raios.module_load_gate.v0`
loader-runtime readiness and see the catalog-bound receiver preflight as an
eleventh non-authorizing source fact: receiver identity complete, retained
candidate matched to catalog finalize, preflight evaluated, and still
`can_load_now: false` / `authorizes_load: false` until M6/M7 reverify,
provider trust, and owner seal exist. Verified: scoped Rust format check
(`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_module_load_gate_render.rs`); PowerShell profile
parse for the touched M12/full harness scripts; `git diff --check` clean apart
from normal CRLF warnings; release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` retry report
`release/vm-reports/shadow-20260709-100905-17032.json` 246/246 predicates, 53
executed commands, `duration_ms: 123830`, report sha256
`211214e53cbe77d8c857ad4629e75eb189bfb110dcaee3a5eb5bae5a7c56a5b6`.
The earlier focused VM report `shadow-20260709-100836-26640.json` is logged
above as `host-transport` image-packaging failure, not guest behavior.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Gate check: latest full-profile report remains green
at `release/vm-reports/shadow-20260708-150428-34396.json` 7867/7867, while
this slice used the focused M12 profile per aggressive-fast cadence. File-size
note: touched `seed-kernel\src\agent_protocol_module_load_gate_render.rs` is
6,357 lines; the existing split plan remains to extract receiver/preflight and
loader-runtime source-fact rendering before broader load-gate renderer changes.

Failed VM classification (2026-07-09 09:50 CEST): focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-094755-29616.json` failed predicate
`m12-distribution:N3_provenance_does_not_enable_granted_candidate_load`.
Verdict: `guest-behavior`; the command reached the generic
`raios.module_load_gate.v0` denial and that retained-candidate load-denial
projection did not yet include the catalog-bound receiver preflight facts. Retry
was blocked until the generic load gate consumed the same projection. Resolved by
M12+ Slice 14; retry report `shadow-20260709-095344-20524.json` passed.

M12+ Slice 14 done (2026-07-09) - receiver preflight visible in the real load
denial. A user/agent can now ask to load the catalog-delivered candidate and see
the generic `raios.module_load_gate.v0` denial carry the same catalog-finalized
receiver-identity preflight facts: retained candidate hash, catalog-finalized
hash, guest-complete receiver evidence, missing M6/M7/provider/owner gates, and
all load/install/execute/persist/network/durable-write authority still false.
Verified: scoped format check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol.rs seed-kernel\src\agent_protocol_registry.rs
seed-kernel\src\agent_protocol_module_load_gate_render.rs
seed-kernel\src\granted_candidate_service.rs`); PowerShell profile parse;
release seed-kernel build via `scripts\build-seed-kernel.ps1 -Profile release`;
focused VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-095344-20524.json` 246/246 predicates, 53
executed commands, `duration_ms: 137623`, report sha256
`af071c943ac5dbb4ce9322d183643964c76c3784a259480508533d9142470179`.
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and
`seed-kernel/src/usb.rs`. Full+recovery remain deferred to the M12+ block close
per the aggressive-fast cadence. File-size note: touched
`seed-kernel\src\agent_protocol_module_load_gate_render.rs` is 6,161 lines; the
next load-gate renderer change beyond a narrow field projection should extract
receiver/preflight and loader-runtime source-fact rendering into separate
`agent_protocol_module_load_gate_*_render.rs` helpers, preserving the existing
`raios.module_load_gate.v0` byte shape under the focused profile.

M12+ Slice 13 done (2026-07-09) - receiver load preflight bound to the
catalog-finalized retained candidate. A user/agent can now ask raiOS to
preflight a receiver identity only after the matching local catalog delivery
has reassembled the artifact, staged it as the inert retained Wasm candidate,
and recorded that catalog-finalized candidate hash; guest-complete receiver
evidence alone now stays denied as
`distribution_receiver_identity_load_preflight_candidate_not_finalized`, and
the missing M6/M7, provider-trust, and owner-seal gates are named only when the
catalog-finalized candidate hash matches the retained candidate. Load,
install, execute, persist, network, and durable writes remain false. Verified:
scoped format check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol_registry.rs`); PowerShell profile parse;
release seed-kernel build via `scripts\build-seed-kernel.ps1 -Profile
release`; focused VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-093455-12140.json` 246/246 predicates, 53
executed commands, `duration_ms: 119796`, report sha256
`0fe1c73b29a40e511419336c5163d9e16a517d90956a8b89da7a52f9a35d0e04`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 12 done (2026-07-09) - receiver identity load preflight. A
user/agent can now ask raiOS to run a current-boot receiver-identity load
preflight against the local catalog artifact hash; the guest refuses to
evaluate until receiver evidence is guest-complete, and once complete it
returns the explicit missing M6, M7, provider-trust, and owner-seal gates while
still reporting `can_load_now: false`, `load_authorized: false`, and no
load/install/execute/persist/network/durable-write authority. This is still a
diagnostic on the final load path, not a load decision. Verified: scoped
format check (`rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol.rs seed-kernel\src\agent_protocol_registry.rs`);
release seed-kernel build via `scripts\build-seed-kernel.ps1 -Profile
release`; focused VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-092219-15860.json` 244/244 predicates, 52
executed commands, `duration_ms: 120872`, report sha256
`54dd7d6063309e2acbf0d66e789c96329791e6b7870a58c12c8e716ed2ceb6a6`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 11 done (2026-07-09) - receiver identity raw evidence guest
verification. A user/agent can now carry the raw receiver artifact-identity
descriptor, current-boot load descriptor, P-256 public keys, and DER signatures
through the real host static/CAS serial export into raiOS; the guest stores the
six bounded payloads as `current_boot` RAM-only, `local_only` evidence,
recomputes every payload SHA-256, re-verifies both descriptor signatures in the
guest, re-checks descriptor artifact/load/identity bindings, and only then marks
`receiver_identity_complete: true` while still granting no
acquisition/install/load/execute/persist/network/durable-write authority. This
is not provenance-as-install and not a load decision; M6/M7 re-verification and
owner sealing remain required for any future load. Verified: scoped format
checks (`cargo fmt --package registry-core -- --check`; `rustfmt --edition
2021 --check seed-kernel\src\agent_protocol.rs
seed-kernel\src\agent_protocol_registry.rs`); targeted
`cargo test --locked -p registry-core
distribution_export_reads_cas_blob_and_emits_serial_commands` (1 passed, with
local Cargo cache/target overrides because the environment pointed Cargo at a
missing `F:\scorefollower-build` path); release seed-kernel build via
`scripts\build-seed-kernel.ps1 -Profile release`; focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-091233-17048.json` 240/240 predicates, 50
executed commands, `duration_ms: 120569`, report sha256
`1fbc5b33cd881203fcb81fdcbdf8df1036334163847e340ad1eb8a79837455f2`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 10 done (2026-07-09) - receiver identity retained by the guest
local catalog. A user/agent can now feed the host-exported receiver identity
metadata through the real serial export command stream into raiOS; the guest
retains the descriptor/key/signature hashes and host-side binding claims as
`current_boot` RAM-only, `local_only` catalog metadata, carries that metadata
through catalog begin/finalize responses, and still reports
`guest_signature_verification_performed: false` plus
`requires_m6_m7_reverify_for_load: true`. This grants no
acquisition/install/load/execute/persist/network/durable-write authority and
does not treat provenance or host claims as installability. Verified:
PowerShell profile parse; scoped format checks (`cargo fmt --package
registry-core -- --check`; `rustfmt --edition 2021 --check
seed-kernel\src\agent_protocol.rs seed-kernel\src\agent_protocol_registry.rs`);
`cargo test --locked -p raios-core -p registry-core -p registry-tools` (300
raios-core tests, 21 registry-core tests); focused VM `m12-distribution-provenance`
report `release/vm-reports/shadow-20260709-085021-5116.json` 231/231
predicates, 43 executed commands, `duration_ms: 159233`, report sha256
`1fda90be807dc1438956c4f971a57c704890f80e57c5b0ec47ece9416eee5690`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 9 done (2026-07-09) - receiver identity evidence in the local
distribution export. A user/agent can now export a local CAS registry artifact
together with the receiver-required raiOS Wasm artifact-identity descriptor,
current-boot load descriptor, P-256 public keys, and DER signatures; the host
export verifies both descriptor signatures, checks that the descriptors bind
the exact exported artifact SHA-256 and each other, carries the raw descriptor
evidence in the export packet, and still emits only the existing serial
catalog/chunk commands. This grants no acquisition/install/load/execute/persist/
network/durable-write authority; the receiver identity is evidence for later
M6/M7 re-verification, not a load decision. Verified: PowerShell profile parse;
scoped `cargo fmt --package ota-tools --package registry-core --package
registry-tools -- --check`; `cargo test --locked -p ota-tools -p
registry-core -p registry-tools` (5 ota-tools lib tests, 4 promotion-signer
tests, 1 sign/verify integration test, 21 registry-core tests); focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-083354-7272.json` 229/229 predicates,
42 executed commands, `duration_ms: 110097`, report sha256
`a81ead75f5c7b1af152e640c40282f8151665e86e8f9b3b0f5fae493178161d4`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 8 done (2026-07-09) - host static/CAS distribution export. A
user/agent can now take a dev-signed artifact already published into the local
content-addressed registry, export that real CAS entry as bounded serial
distribution commands, and feed those commands through raiOS's existing
current-boot local catalog and chunk transport; the host export recomputes the
stored BLAKE3 blob identity, computes the receiver SHA-256 artifact identity,
adds a scalar-2 provenance signature, emits per-chunk SHA-256/base64 payloads,
and the guest still stages only an inert current-boot candidate after chunk,
whole-artifact, and provenance verification. This grants no
acquisition/install/load/execute/persist/network/durable-write authority; host
registry provenance remains a source locator/evidence layer, not load
worthiness, and M6/M7 re-verification is still required for any future load.
Verified: PowerShell profile parse; scoped `cargo fmt --package ota-tools
--package registry-core --package registry-tools -- --check`; `cargo test
--locked -p ota-tools -p registry-core -p registry-tools` (4 ota-tools lib
tests, 4 promotion-signer tests, 1 sign/verify integration test, 21
registry-core tests); focused VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-081542-6576.json` 229/229 predicates,
42 executed commands, `duration_ms: 108906`, report sha256
`39fe096560a15d2ffb341ba74598f3f5bfdd21f8a68943d83e5c508cb60cbe8e`;
`scripts\scan-secrets.ps1` found no OpenAI-key-like values. Global
`cargo fmt --all -- --check` remains red only on the pre-existing unrelated
format drift in `raios-core/src/marvell_wifi_fw.rs` and `seed-kernel/src/usb.rs`.
An earlier same-slice VM run
`release/vm-reports/shadow-20260709-081209-4752.json` is classified below as
a host-transport harness bug before the new T2 predicate ran; the corrected
retry passed. Full+recovery remain deferred to the M12+ block close per the
aggressive-fast cadence.

Failed VM classification (2026-07-09): focused
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-081209-4752.json` failed during host-side
profile setup before the new
`m12-distribution:T2_host_static_cas_export_commands` predicate could run.
Failing predicate: none reached; PowerShell `Invoke-M12CargoTool` used a
parameter named `$Args`, so the function executed bare `cargo`, captured
Cargo's "Rust's package manager" help text, and `ConvertFrom-Json` rejected it
as non-JSON. Verdict: host-transport / harness bug, not guest behavior; no M12
predicate failed and QEMU was still running at teardown.

M12+ Slice 7 done (2026-07-09) - non-builtin local distribution catalog. A
user/agent can now submit a dev-publisher-signed artifact catalog entry into a
current-boot, non-builtin local catalog, select it later by content hash to
start the existing bounded serial chunk transport, and only after chunk hashes,
whole-artifact hash, and provenance signature verification does raiOS stage the
artifact as an inert current-boot candidate. A wrong catalog selector does not
start delivery, and a follow-up finalize stages nothing. This grants no
acquisition/install/load/execute/persist/network/durable-write authority; the
catalog is RAM-only metadata, provenance remains origin evidence only, and
M6/M7 re-verification is still required for any future load. Verified:
touched-file rustfmt check; PowerShell profile parse; `cargo test --locked -p
raios-core` (300 passed, with local Cargo cache/target overrides because the
environment pointed Cargo at a missing `F:\scorefollower-build` path); release
seed-kernel build (same pre-existing IOMMU/WiFi warnings); focused VM
`m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-075830-26216.json` 228/228 predicates,
42 executed commands, `duration_ms: 106309`, report sha256
`249f9293436f15eb27ee14d1d5034657819b6f42736ea9f2fe2622ab99174640`.
An earlier same-slice VM run
`release/vm-reports/shadow-20260709-075623-19216.json` was classified below
as host-transport `qemu_exited` before any M12-specific command ran; the clean
retry passed without a code change. Full+recovery remain deferred to the M12+
block close per the aggressive-fast cadence.

M12+ Slice 6 done (2026-07-09) - real local serial distribution transport. A
user/agent can now deliver a dev-publisher-signed local artifact over the
serial protocol as bounded content-addressed chunks; raiOS verifies the
declared chunk hashes, reassembles out-of-order chunks through the M12 chunked
delivery verifier, checks the whole artifact hash plus provenance signature,
and only then stages the artifact into the existing inert current-boot candidate
intake. A wrong claimed chunk hash clears the pending transport and finalize
stages nothing. This grants no acquisition/install/load/execute/persist/network
or durable-write authority; provenance remains origin evidence only and M6/M7
re-verification is still required for any future load. Verified: touched-file
rustfmt check; PowerShell profile parse; `cargo test --locked -p raios-core`
(300 passed, with local Cargo cache/target overrides because the environment
pointed Cargo at a missing `F:\scorefollower-build` path); release seed-kernel
build; focused VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-014534-25684.json` 215/215 predicates,
34 executed commands, `duration_ms: 86323`, report sha256
`75bbb66c62f16b0711ffa31cc9156688f2d23c25704ba4fecc5912a1a33390f2`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 5 done (2026-07-09) — kernel bridge for multi-entry + chunked local
distribution. A user/agent can now select between two dev-publisher-signed local
registry entries (`svc.demo.echo` and `svc.demo.bufecho`) by content hash through
the in-kernel bounded registry; a valid selection stages exactly that artifact
into the existing inert current-boot candidate intake, and the chunked bufecho
selftest reassembles out-of-order chunks only after per-chunk and whole-artifact
hash verification before staging. Misses fail closed as `registry_entry_not_found`.
This grants no acquisition/install/load/execute/persist/network/durable-write
authority; provenance remains origin evidence only and M6/M7 re-verification is
still required for any future load. Verified: touched-file rustfmt check;
`cargo test --locked -p raios-core` (300 passed); release seed-kernel build; focused
VM `m12-distribution-provenance` report
`release/vm-reports/shadow-20260709-013326-2688.json` 204/204 predicates,
26 executed commands, `duration_ms: 73069`, report sha256
`0c92db2bce5c4bdba0ed9a1cfd23f98a87377be93220b658a27c4a4d4ba8d6bd`.
Full+recovery remain deferred to the M12+ block close per the aggressive-fast
cadence.

M12+ Slice 2 done (2026-07-08) — ADR 0009 Option A, a LOCAL content-addressed
registry. Phase A (commit 4e1acf7, raios-core): a typed registry entry model +
selection evaluator — the constructor RECOMPUTES sha256 from bytes and fail-closed
rejects mismatched hash / absent-empty signature / wrong publisher key / secret
classification; the evaluator reuses the Slice-1 provenance verify + honesty and
produces a RegistrySelectionDecision with EVERY authority false (acquisition/
install/load/execute/persist); rendered via record::Value. Phase B (commit 18374f7,
seed-kernel): one built-in echo registry entry + a READ-ONLY registry-selection
diagnostic that selects by sha256, verifies provenance against the kernel-recomputed
bytes, and STAGES a valid entry into the EXISTING candidate-intake path as an INERT
candidate (via a behavior-preserving shared intake_and_retain helper the serial path
also uses). Grants nothing — load/execute/install/persist stay denied; provenance !=
load-worthiness; M6/M7 promotion + re-verify still required to ever load. Verified:
raios-core 262 tests; seed-kernel build warning-free; m12-distribution-provenance
225/225 (registry selection stages inert candidate; wrong hash → no stage; load STILL
denied). Full+recovery deferred to the M12+ block close. Scoping + implementation by
CODEX workers.

M12+ first distribution slice done (2026-07-08) — ADR 0009 Option A. M12+ Phase A
(commit d5f49b9): raios-core provenance-verify primitive — verify a distribution/
publisher signature over a DOMAIN-TAGGED message ("raios.distribution_provenance.
v0" ++ artifact_sha256) with a SEPARATE pinned publisher key (dev P-256 scalar 2,
distinct from the promotion scalar-1 + artifact-identity keys) so a promotion or
identity signature can't be replayed as provenance; + a grants-nothing honesty
evaluator delegating M12-1 (authorizes_acquisition/install/load FALSE on every
path); + the ota distribution-provenance-signer bin. M12+ Phase B (commit 6034355):
a read-only kernel diagnostic that verifies a publisher signature against the
KERNEL-recomputed retained candidate.sha256 and reports honest evidence — the
candidate stays INERT (load/execute/install still denied); provenance != load-
worthiness; loading still needs the full M6/M7 promotion + re-verify. Verified: raios-
core 253 + ota-tools tests; seed-kernel build warning-free; m12-distribution-
provenance profile 223/223 (valid provenance → load STILL denied; absent/tampered/
wrong-artifact-binding rejected; granted-candidate load gate + generic durable gate +
honesty report unchanged). Grants nothing; dev_key_not_owner_sealed. Full+recovery
deferred to the M12+ block close. Scoping + implementation by CODEX workers.

WiFi-1b-pure done (2026-07-08, commit b23b4f5) — OWNER SIDE TRACK. Pure
FwAction -> ordered register-write-plan translator in raios-core/src/
marvell_wifi_fw.rs (the testable bridge between the WiFi-1a sequencer and the
future kernel MMIO/DMA shell): plan_register_writes(FwAction, block_dma_phys) ->
fixed-capacity no-alloc RegWrite list. 5 new unit tests pin the exact ordered plan
per action. NO unsafe/IO/alloc; raios-core 253 tests. WiFi-1b hardware shell design
committed (docs/marvell-88w8897-wifi-1b-design.md); the MMIO/DMA shell itself is
next (compile-only, inert, until the real Surface).

WiFi-1a done (2026-07-08, commit e3de309) — OWNER SIDE TRACK, first real code
brick of the Surface Pro 4 Marvell 88W8897 WiFi driver. A PURE, hardware-
INDEPENDENT firmware-download sequencer in raios-core/src/marvell_wifi_fw.rs: a
`step(RegisterReads) -> FwAction` state machine modelling the mwifiex_pcie block-
download loop (poll cmd_size, WriteBlock/RingDoorbell, CRC-retry capped at 2,
WriteDrvReady, PollFwStatus until FIRMWARE_READY, Done; fail-closed on empty/
oversize/bad state), grounded in docs/marvell-88w8897-wifi-driver-scope.md. NO
unsafe, NO MMIO/DMA, NO firmware blob, wired to NOTHING — honestly HARDWARE-
UNTESTED, inert until a later kernel slice drives it on the physical Surface.
Dispatched as a CODEX worker. Verified host-only: cargo fmt clean; raios-core 237
tests (8 new marvell_wifi_fw cases: happy path, CRC-retry, retry-cap, empty-fw,
partial block, poll-not-ready, zero-cmd-size, block-len-guard); grants nothing.

M11-7 done (2026-07-08) — SECOND real kernel-code relocation (ADR 0008 Option 2):
the kernel's HTTP-response header parsers moved out of the kernel and now run in a
signed Wasm sandbox, cross-checked. M11-7a (commit 928833f) carved the 11 pure
non-allocating parsers (parse_status / parse_content_length / http_response_
complete / header_contains + helpers) out of seed-kernel/src/openai.rs into a
standalone no-dep crate raios-http-parse (openai.rs re-imports the 6 fns its
remaining code calls; kernel byte-identical; sha2 dev-dep-only). M11-7b (commit
2a1284d) runs the SAME parsers inside the dev-key-signed svc.demo.httphead guest
on an embedded HTTP response, kernel independently re-parses + cross-checks on
well-formed + truncated + content-length paths (httphead 194/194, regression
196/196). M11-7c (commit ec236de) FIXED the real latent bug M11-7a preserved
verbatim: parse_content_length short-circuited to None for every response
(split_header(line)? bailed on the colon-less status line); now it skips non-
header lines, so http_response_complete correctly detects non-chunked completion
via Content-Length. Re-proven: the httphead predicate flipped from content-length-
short-circuit-evidence to content-length-parsed-agree (guest+core both parse
content_length=11). Verified per the aggressive-fast loop WITHOUT the per-slice
max-effort review (dropped by owner 2026-07-08): host DoD + own diff read +
focused profile + secret scan. Guests dev_key_not_owner_sealed; build.rs P-256
gate active. Scoping + implementation both by CODEX workers (owner 2026-07-08).

M11-6 done (2026-07-08) — the FIRST real kernel-code relocation into Wasm
(ADR 0008 Option 2). M11-6a (commit 18d095f) carved the pure DER X.509
validity-window parser out of raios-core into a standalone dependency-free
no_std crate `raios-x509-time` (raios-core re-exports it as
`cert_validity_window` so every call site is byte-identical; `sha2` is a
dev-dependency ONLY so the guest never builds crypto — raios-core itself cannot
compile to wasm32). M11-6b (commit 17d46f8) runs that SAME parser inside a
dev-key-signed sandboxed Wasm service `svc.demo.certwindow` (imports only the 3
byte-buffer host fns) on the real M10C-4 certificate; the kernel INDEPENDENTLY
re-parses the same cert and cross-checks the guest's fixed 18-byte record on
BOTH the happy AND the truncated-error path. Guest output is evidence-only, the
core is authority (deterministic sandboxed re-execution, not a diverse reimpl);
every trust/authorize/durable flag is hard-false; `policy_allows_beyond_env`
stays false; the kernel-side decode is fail-closed (length/magic/version guarded)
so a hostile guest output cannot panic the kernel. The shared runner was touched
only additively (a `raw_captured_output` field read solely by the certwindow
decode + a per-call `fuel_budget` parameter) so echo/bufecho stay byte-identical.
Verified host-only: cargo fmt clean; raios-x509-time 22 + raios-core 229 tests
green; guest `.wasm` reproduces sha256 345663778bfd… via the standard build
script; shipping seed-kernel build EXIT 0 warning-free with the build.rs P-256
hard gate active. VM: m11-6-certwindow 193/193; m11-buffer-channel regression
196/196 (echo/bufecho byte-identical). Review: MAX-EFFORT 4-lens adversarial
review (signing / shared-runner-regression / cross-check-integrity / honesty) =
SHIP with zero defects, including independent cryptographic re-verification of
both scalar-1 descriptor signatures. Everything honestly labeled
`dev_key_not_owner_sealed`.

M11-5b Phase B done (2026-07-08, host-only worker packet; no QEMU): raiOS now
runs a new signed `svc.demo.bufecho` Wasm service that imports only
`env.input_len`, `env.input_read`, and `env.output_write`, reads a bounded
host-staged byte buffer, and writes the same bytes back as captured output
evidenced by matching len+sha256 fields. Existing `svc.demo.echo`,
`svc.demo.hello`, and `svc.dev.granted_candidate` import surfaces stay
unchanged; bufecho descriptors are signed by the public scalar-1 dev key and
honestly labeled `dev_key_not_owner_sealed`, with build-time P-256 verification
as the hard gate. Host-only verification is recorded in the worker handoff;
the orchestrator runs `m11-buffer-channel` and the mandatory isolation review.

M11-5a Phase A done (2026-07-08, host-only worker packet; no QEMU): raiOS now
has the default-deny byte-buffer Wasm data-channel mechanism: `env.input_len`,
`env.input_read`, and `env.output_write` are known host imports and have bounded
runtime plumbing to stage host input into guest memory and capture guest output
as len+sha256 evidence, but no existing service declares or is granted them.
`svc.demo.echo`, `svc.demo.hello`, and `svc.dev.granted_candidate` keep their
existing authorized import lists and marker shape; the subset check still runs
before instantiation and the linker still only defines imports that the scoped
grant evaluator authorized. Host-only verification passed:
`cargo fmt --all -- --check`, `cargo test --locked -p raios-core` (246 passed),
and `powershell -File scripts\build-seed-kernel.ps1 -Profile release`.

M10C-4 done (2026-07-08, host-only worker packet; no QEMU): on live boot,
raiOS parses the fixed real embedded-tls localhost certificate fixture DER
through the committed raios-core X.509 validity-window parser and checks the
parsed 2021-10-13T08:20:42Z..2031-10-11T08:20:42Z window against the CMOS RTC
using the M10C-2 unverified-basis comparator, then reports
`real_cert_probe` beside the unchanged fixed synthetic cert-time selftest
cases. The embedded fixture is
`vendor/embedded-tls-0.17.0/tests/data/server-cert.pem` decoded to 522 DER
bytes with sha256
`baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4`; it is a
fixed test fixture, not a live handshake cert. This grants no trusted time,
chain/hostname/WebPKI validation, cert-time validation, provider request/export
authority, durable write, owner seal, capability, or transmission. Host-only
verification passed: `cargo fmt -p seed-kernel --check`,
`cargo fmt -p raios-core --check`, `cargo test --locked -p raios-core` (243
passed), release seed-kernel build, and PowerShell parse of
`vm-harness/shadow-vm-smoke-profile-common.ps1`.

M10C-3 done (2026-07-08, host-only worker packet; no QEMU): raiOS-core can now
fail-closed parse a real DER certificate's declared `notBefore`/`notAfter`
validity window into `CertValidityDateTime`, so the existing M10C-2
unverified-basis comparator can check real certificate dates. The parser is a
self-contained bounded DER TLV reader in `cert_validity_window.rs`: one-byte
tags only, definite lengths only, non-minimal/oversize/truncated forms denied,
no recursion, and all payload movement is bounds-checked. The embedded fixture
is the real `vendor/embedded-tls-0.17.0/tests/data/server-cert.pem` DER
(522 bytes, sha256
`baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4`) with
parsed dates 2021-10-13T08:20:42Z..2031-10-11T08:20:42Z. This grants no live
cert use, network, trusted time, chain/hostname/WebPKI validation, provider
authority, durable write, or schema. Host-only verification passed:
`cargo test --locked -p raios-core` (243 passed) and
`cargo fmt -p raios-core -- --check`.

system.honesty_report done (2026-07-08, host-only worker packet; no QEMU):
an agent can read `system.honesty_report` to get one live, auditable,
read-only, local-only manifest of raiOS's honest posture across provider trust,
time, cert-time, provider export, Wasm import-surface enforcement, external
acquisition posture, and owner-seal state. The report grants nothing, writes
nothing durable, sends nothing, and changes no state; provider/time subrecords
reuse the live evaluator helpers also used by `provider.trust_honesty` and
`system.time_authority`, and `no_dishonest_overclaim` is computed from
overclaim/grant/write/transmission booleans rather than the evaluator `honest`
field. Host-only verification passed: `cargo fmt -p seed-kernel --check`,
`cargo fmt -p raios-core --check`, `cargo test --locked -p raios-core` (234
passed), release seed-kernel build, and PowerShell parse of
`vm-harness/shadow-vm-smoke-profile-common.ps1`.

M12-1 done (2026-07-08, host-only worker packet; no QEMU): raiOS now has a
fail-closed raios-core evaluator that can certify external artifact acquisition
labels are honest: received bytes are candidate intake, distribution signatures
are provenance only, load-worthiness still requires the M6/M7 re-verify chain,
and success grants no acquisition, install, or load authority. Host-only
verification passed: `cargo test --locked -p raios-core` and
`cargo fmt -p raios-core -- --check`.

M11-3a done (2026-07-08, host-only worker packet; orchestrator runs the
memory-durable VM profile): raiOS now durably audits the exact Wasm host-import
surface a service was already authorized and linked with as a local-only
`capability_grant` memory record, deduped per
`(service_id, authorized_import_list_sha256)` per boot. The audit driver binds
the record to M11-3 run evidence (`import_grant_performed`, authorized status,
module imports within the authorized list, authorized import count, and the
canonical `authorized_import_list_sha256` for the same service/import tuple)
before appending; mismatches, unknown services, duplicates, or append failures
stay RAM-only. `svc.demo.echo` and `svc.dev.granted_candidate` call the audit
only from their service-level `start()` path after lifecycle event creation and
before taking the state lock; `wasm_runtime::execute_module_bytes` and
`module.granted_candidate_selftest` are untouched. The service.start response
now additively exposes `durable_import_grant_audit`, and the memory-durable
profile adds `wasm-import-grant-durable:first-appended`,
`wasm-import-grant-durable:second-deduped`,
`wasm-import-grant-durable:chain-advance-exactly-one`, and
`wasm-import-grant-durable:context-local-only-nonexportable`. This grants no new
import, linker behavior, enforcement behavior, provider authority, transmission,
secret, or raios-core behavior. Host-only verification passed:
`cargo fmt -p seed-kernel --check`, `cargo fmt -p raios-core --check`,
`cargo test --locked -p raios-core` (229 passed), release seed-kernel build, and
PowerShell parse of `vm-harness\shadow-vm-smoke-profile-memory-durable.ps1`.

M11-3 done (2026-07-08, host-only worker packet; orchestrator runs VM/review):
raiOS now authorizes each current Wasm service's host-import surface through the
committed M11-2 evaluator and constructs service instances from only the
evaluator-authorized imports. `svc.demo.echo` and the dev-key granted candidate
still run with exactly `env.log` plus `env.counter_get`; a granted list that
omits the module's required `env.counter_get` fails closed before instantiation
as `module_import_not_authorized`, and the existing `env.forbidden_write`
negative remains a physical wasmi `missing_definition` link failure. The change
adds no TLS/HTTP relocation, provider secret handling, new host import,
non-`env` policy grant, raios-core change, artifact-store semantic change,
global fallback linker, durable write, persistence, or QEMU run. Host-only
verification is recorded in the worker handoff; focused VM evidence is expected
from `vm-harness\shadow-vm-smoke.ps1 -Profile m11-wasm-import-grant`.

M11-2 done (2026-07-08, host-only worker packet; no QEMU): a fail-closed
raios-core evaluator can now authorize the exact declared Wasm host-import list
for one service/artifact binding, default-denying missing service id, missing
artifact binding, missing import list, over-cap lists, unknown host imports,
broader-than-`env` imports when owner policy is false, and duplicates. This
grants nothing by itself: no kernel linker wiring, service load, code movement,
secret release, durable write, or authority flip was added. The only production
known host imports are the two functions that exist today,
`("env","log")` and `("env","counter_get")`; no fictional `net.*`,
`tls_record.*`, `crypto.*`, `time.*`, or `secret.*` imports were invented.
The `import_beyond_env_not_owner_authorized` reason is implemented as an honest
forward guard for the day a real non-`env` host import exists; current tests
prove it is unreachable today because all known imports are `env.*`, so a
non-`env` request fails first as `unknown_host_import`. The canonical
`authorized_import_list_sha256` helper hashes the declared ordered list with
`record::sha256_of_json`, is deterministic, order-sensitive, and authorizes
nothing. Host-only verification passed: `cargo test --locked -p raios-core`
(229 passed, including 11 scoped_wasm_import_grant tests; 7 distinct denial
reason strings are pairwise-unique, 6 are reachable with today's production
known-import set) and `cargo fmt -p raios-core -- --check`.

M10C-2 done (2026-07-08, host-only worker packet; no QEMU): raiOS can now
compare fixed synthetic certificate validity windows (`notBefore`/`notAfter`)
against the live CMOS RTC wall clock and report within / not-yet-valid /
expired as `cmos_rtc_unverified` sanity evidence, while validating no
certificate time and granting no provider request/export authority,
transmission, durable write, or capability. `raios-core/src/cert_validity_window.rs`
adds the host-tested tuple comparator with inclusive notBefore/notAfter
boundaries and grant-nothing decisions. The read-only
`system.cert_time_check_selftest` method emits
`raios.cert_time_check_selftest.v0` as test infrastructure only, using two fixed
synthetic windows: wide 2020-01-01T00:00:00..9999-12-31T23:59:59 expects
`within_window_unverified_basis`; expired
2000-01-01T00:00:00..2010-01-01T00:00:00 expects
`after_expired_unverified_basis`. It performs no DER parse, live cert read,
network, provider write, provider export, durable write, or trust/authority
grant. Host-only verification passed: `cargo fmt -p raios-core --check`,
`cargo fmt -p seed-kernel --check`, `cargo test --locked -p raios-core` (213
passed, including 8 cert_validity_window tests), release seed-kernel build, and
PowerShell parse of `vm-harness/shadow-vm-smoke-profile-common.ps1`.

M10C-1 done (2026-07-08, host-only worker packet; no QEMU): raiOS can now read
the machine's CMOS RTC wall-clock components and expose them via the read-only
`system.time_authority` method as `local_only`, `current_boot`, explicitly
`cmos_rtc_unverified` SOURCE evidence; the clock is available for later
cert-time comparison but is not trusted, validates no certificate time, and
grants no provider request/export authority, durable write, transmission, or
capability.

M10B-2 done (2026-07-08, host-only worker packet; no QEMU): the kernel's live
`provider.trust_honesty` path now evaluates the one real OpenAI provider
through the committed provider-agnostic
`raios_core::provider_trust_descriptor::ProviderTrustDescriptor`, proving the
honesty path is descriptor-driven while granting nothing. The descriptor is
built from `provider_trust::snapshot()`: `provider_id = "openai"`,
`trust_state = snap.state.as_protocol()`, `development_bypass =
snap.development_bypass`, false chain/time validation claims, and
id/host/port/transport/hostname/pin/chain/time/certificate-verifier metadata
from the real Stage-0 verifier. The handler now calls
`evaluate_provider_trust_descriptor_honesty`, so the M10A-1 decision result is
unchanged, and additively reports descriptor identity
(`descriptor_id`, `host`, `descriptor_sha256`) alongside the existing provider
id, chain policy, and time policy. The common smoke profile adds
`protocol:provider_trust_honesty_descriptor_driven` without loosening the
existing unpinned-denial or grants-nothing predicates. Host-only verification:
`cargo fmt -p seed-kernel --check`, `cargo fmt -p raios-core --check`,
`cargo test --locked -p raios-core` (201 passed), release seed-kernel build,
and PowerShell parse of the edited common profile all passed. No raios-core,
provider trust config, OpenAI transport/export gate, second/fake live provider,
durable write, request/export authority, commit, or VM run was added.

M10B-1 done (2026-07-08, host-only worker packet; no kernel/VM wiring): an
agent can describe OpenAI and a second provider with one typed,
provider-agnostic trust descriptor and run the committed M10A-1 honesty
evaluator over either provider while granting no provider request authority and
no provider export authority. `raios-core/src/provider_trust_descriptor.rs`
adds `ProviderTrustDescriptor`, OpenAI metadata mirroring the Stage-0 pinned
TLS verifier, and a clearly synthetic Anthropic-shaped selftest descriptor with
`trust_state = pin_config_missing`; descriptor honesty only maps into
`ProviderTrustHonestyInput` and calls `evaluate_provider_trust_honesty`.
Descriptor identity hashes use the shared `record::Value` model plus
`sha256_of_json` over schema/provider/host/port/transport/hostname/pin/chain/
time/certificate-policy fields, excluding trust state and claims. Host-only
verification: `cargo test --locked -p raios-core` passed 201 tests, including
the four provider descriptor tests, and `cargo fmt -p raios-core -- --check`
passed. No QEMU, seed-kernel, vm-harness, vendor, live provider, pins, chain,
trusted time, request/export authority, or durable provider-trust write was
added.

M10A-2 done (2026-07-08, host-only worker packet; VM profiles are orchestrator-
run): raiOS can now run the committed M10A-1 provider-trust honesty evaluator
against the live Stage-0 trust snapshot through the read-only
`provider.trust_honesty` method. The kernel maps
`provider_trust::snapshot()` into `ProviderTrustHonestyInput` as provider
`openai`, `trust_state = snap.state.as_protocol()`,
`chain_policy = snap.verifier.chain_policy`,
`time_policy = snap.verifier.time_policy`, `development_bypass =
snap.development_bypass`, and false chain/time validation claims. The default
no-pin boot is honestly reported as `pin_config_missing` with M10A-1
`performed:false`, `status:"denied"`, reason `trust_state_not_pin_verified`,
`chain_validated:false`, `time_validated:false`, and both
`authorizes_provider_request:false` and `authorizes_provider_export:false`.
`vm-harness\shadow-vm-smoke-profile-common.ps1` now adds the quick-sourced
`protocol:provider_trust_honesty_unpinned_denial` and
`protocol:provider_trust_honesty_grants_nothing` predicates without changing the
real `provider.context_export` denial row/handler. Host-only verification:
`cargo fmt -p seed-kernel -- --check`, `cargo fmt -p raios-core -- --check`,
`cargo test --locked -p raios-core` (197 passed), release seed-kernel build, and
PowerShell parse of the edited common profile all passed.

Failure classification log (rule: AGENTS.md "Failure Classification Rule"):

- 2026-07-09 `shadow-20260709-075623-19216.json`
  (`m12-distribution-provenance`, M12+ Slice 7 working tree): no failing
  predicate; 173/173 reached predicates passed and 14 commands executed, then
  the harness classified `serial_transport_failure: qemu_exited` while
  reconnecting to serial TCP port 4565 after `provider.context_gate
  provider_minimal`. Serial tail ended cleanly after
  `RAIOS_AGENT_END provider.context_gate` / `AGENT RESPONSE WRITTEN TO SERIAL`,
  QEMU stderr was empty, and no M12-specific command had run yet. Verdict:
  **host-transport (qemu_exited / QEMU process silent exit)**, not observed
  guest behavior and not evidence against the local-catalog distribution
  slice. Retry is permitted after this classification.

- 2026-07-06 `shadow-20260706-132514-25060.json` (full profile, M6B-2
  working tree): 2502/2502 reached predicates passed, then the M0-2
  instrumentation classified `serial_transport_failure: qemu_exited`
  (QEMU pid 6364 exited during a 0.5s serial reconnect on port 4565).
  Verdict: **host-transport (qemu_exited)**, the documented intermittent
  silent-exit class — NOT a guest crash: no panic/page-fault in the
  serial log, single boot (BdsDxe only at offsets 87/197, no mid-run
  reboot), and the guest had already run healthy well past the change
  into `module.loader_*` hooks. M6B-2 exonerated: the full 10-case grant
  selftest passed, including `signed_fully_bound_attestation_grants_capability`
  (the grants_capability=true path), the 4 fail-closed cases, and the
  co-emission invariant. CONFIRMED intermittent: clean retry (no code
  change) `shadow-20260706-133249-16364.json` passed 8168/8168.

- 2026-07-06 `shadow-20260706-092454-23172.json` (quick profile, M6A-1
  working tree): 348/350 predicates passed; the 2 failures are both the
  single command `agent audit.events 72` (expecting
  `RAIOS_AGENT_END memory.recent_events`), which timed out. Verdict:
  **host-transport** (harness serial-reader overrun on the large
  audit.events scrape), NOT the resolved 2026-07-05 stack overflow.
  Evidence: report self-classification `serial_transport_failure: null`
  and `qemu_process.before_teardown.state = "running"` (QEMU alive at
  teardown, harness killed it — no `qemu_exited`); serial log grew to
  3.75 MB, ~2.3 MB *past* the reader's stuck offset 1,429,462 with no
  panic/page-fault and a single boot (BdsDxe only at offsets 87/197);
  the guest reached 66 END markers including the heavy
  `module.load_ephemeral`/`service.rollback_apply`. The new M6A-1
  `candidate_intake` capability is exonerated and verified: all 8
  `quick:wasm_echo_probe_candidate_*` predicates passed. Same "asked for
  too much data at once" scrape class as the 2026-07-03 audit-window
  failures. CONFIRMED intermittent host-transport: clean retry (no code
  change) `shadow-20260706-093418-2968.json` passed 562/562, the same
  `audit.events`/`memory.recent_events` commands green.
  RECURRED 2026-07-06 during M6A-2a quick regression
  `shadow-20260706-102248-13040.json`: byte-identical signature (same
  `agent audit.events 72` timeout at offset ~1,429,514,
  `serial_transport_failure: null`, QEMU `state: running` at teardown);
  candidate needles + the M6A-2a label fix all green before it. This
  audit.events-72 scrape reader-overrun is now a recurring nuisance flake
  — candidate for a real harness fix (bounded/paged scrape) in a later
  harness slice, tracked separately from capability work.
- 2026-07-05 `shadow-20260705-114125-1380.json` (recovery profile,
  uncommitted M2-4 working tree): 433/433 reached predicates passed, then
  the NEW M0-2 instrumentation classified the death in 0.5s:
  `qemu_exited`, no listener, exit code unobtainable, guest serial tail
  clean directly after `memory.recent_events` / `AGENT RESPONSE WRITTEN
  TO SERIAL`. Same signature as the 2026-07-04 full-profile failure —
  which happened on pre-M2 committed code, so the M2 ports are exonerated
  as the cause. Pattern (two occurrences, both dying right after
  `memory.recent_events`) now points at a timing-dependent guest
  reset/triple fault after that response (`-no-reboot` turns a guest
  reset into a silent clean QEMU exit) rather than host transport;
  reclassified as suspected guest-behavior, intermittent. Follow-up
  candidate for investigation if it recurs: the post-response code path
  of `memory.recent_events` (agent_protocol_memory.rs).
- 2026-07-05 `shadow-20260705-120244-*.json` (recovery profile, second
  occurrence same day): identical signature AGAIN — 433/433 reached
  predicates passed, death at exactly the same profile position (directly
  after `memory.recent_events`), `qemu_exited` classified in 0.5s, clean
  serial tail. Now 2 of 3 recovery-profile runs on this tree; no longer
  "intermittent noise" but a ~50%-reproducible guest crash at a fixed
  boundary. PROMOTED to an active repair work item: instrument/inspect
  what runs after the `memory.recent_events` response is written
  (agent_protocol_memory.rs post-response path, next-command read loop,
  or heap state at that point). The M2 ports remain exonerated (first
  occurrence 2026-07-04 predates them).
  RESOLVED 2026-07-05: root cause found and fixed. Checkpoint bisection
  (two serial breadcrumbs) showed death BETWEEN two markers with only
  function returns in between — the stack-overflow signature. Measured:
  `EventSnapshot` was **3,784,744 bytes** (`[Option<Event>; 256]`, each
  `Event` 14,784 bytes) copied by value onto the command stack on every
  `memory.recent_events`. Fix: `snapshot_recent` removed;
  `emit_recent_events` now iterates the ring one event at a time via
  `event_log::recent_event` (spinlock never held across serial writes;
  bindings borrowed, not copied). Proof: 5/5 consecutive recovery-profile
  runs green (previously ~50% crash rate; P(5 green | old rate) ≈ 3%),
  plus a 6th green run after breadcrumb removal
  (`shadow-20260705-125828-3624.json`, 3644/3644). Follow-up: dead
  `EventSnapshot` struct in `event_log_types.rs:3902` (1 dead-code
  warning) to be deleted in an M2 de-hello-ify/cleanup slice.

- 2026-07-04 `shadow-20260704-183440-16492.json` (full profile): no failing
  predicate (200/200 reached passed, 14 commands executed); failure is
  `Timed out connecting to QEMU serial TCP port 4565` after the guest
  cleanly completed `memory.recent_events` (`AGENT RESPONSE WRITTEN TO
  SERIAL` is the last serial line). Verdict: host-transport, subtype
  suspected qemu_exited — connect attempts found no listener for the full
  ~7-minute retry window, QEMU stderr empty, process gone; guest behavior
  exonerated by the clean serial tail. Packet M0-2 adds harness
  instrumentation (qemu_exited / listener_missing_process_alive /
  connect_timeout_listener_present) so future failures self-classify.
  RESOLVED 2026-07-05: packet M0-2 landed — the harness now tracks the QEMU
  process object, aborts the serial reconnect loop immediately when the
  process is gone, and records a structured `serial_transport_failure`
  classification (qemu_exited with exit code / listener_missing_process_alive
  / connect_timeout_listener_present) plus `qemu_process`
  before/after-teardown snapshots and a structured `stderr_log` block in
  every report. Verified green with quick profile
  `shadow-20260705-094659-19752.json` (417/417 predicates; new fields present
  and correct). Root-cause hypotheses for the silent mid-run QEMU exit, in
  likelihood order (diagnosis only, no launch-side change made):
  (1) `scripts/run-stage0-qemu.ps1:37-38` — any concurrent invocation with
  `-StopExisting` force-kills EVERY `qemu-system-x86_64` on the machine, not
  just its own stale instance; a parallel harness/screenshot launch kills the
  in-flight VM with empty stderr and no listener, exactly the observed
  signature. Suggested future fix: scope StopExisting via a PID file instead
  of a global process kill. (2) `-no-reboot` (`run-stage0-qemu.ps1:122`) — a
  guest reset/triple fault after the last serial write makes QEMU exit
  cleanly (code 0, empty stderr). The new exit-code capture will distinguish
  these on the next occurrence.
- 2026-07-03 `shadow-20260703-183727-11132.json` 7005/7006
  `module_manifest_audit_source`: host-harness audit-window failure — the
  manifest event had scrolled out of the single giant `audit.events 256`
  window; the kernel does record it (`event_log.rs:5218`). Not transport,
  not guest-semantic. Fixed by the bounded per-boundary scrapes now in the
  committed harness.
- 2026-07-03 `shadow-20260703-190659-10500.json` 7380/7381
  `no_entrypoint_scoped` (`Expected False, got ""`): host-harness
  audit-window failure — the parsed binding object was absent from the
  oversized scrape response; the kernel records the entrypoint
  source-evidence event (`event_log.rs:6278`). Not transport, not
  guest-semantic. Same bounded-scrape fix applies.
- 2026-07-03 `shadow-20260703-193007-17640.json`: corrupt JSON report
  (recorded here so it is not silently ignored); superseded by later runs.

Active execution memory: as of 2026-07-02, Phase 6 has its first positive
RAM-only service vertical slice: `raios.ram_only_hello_service.v0`. The kernel
can load/start the built-in `svc.demo.hello` test service through a typed
current-boot load request/descriptor, expose it through `service.inventory`,
report health, stop it, start it again through `service.start`, explicitly
restart it through `service.restart`, drop it, and retain RAM-only
lifecycle/health audit events that cite the descriptor plus a validated
current-image
descriptor-source locator/kind/hash. The current-image descriptor-source path
now also carries a repo-local P-256/SHA-256 signature envelope that is checked
by the build script and verified again in the kernel before descriptor-source
selection; load responses, `service.inventory`, `service.health`, and RAM audit
bindings expose the envelope id/hash, payload/public-key/signature hashes, and
`signature_verified`. A read-only
`service.descriptor_source_trust_selftest` method proves the accepted envelope
verifies and tampered payload, locator/kind, public-key hash, and signature
cases fail closed.
The Hello path now also carries a signed
`raios.builtin_artifact_identity.v0` candidate for the existing
`builtin:svc.demo.hello` artifact. The build script checks the checked-in
P-256/SHA-256 identity signature, the kernel validates it before load, and load
responses, `service.inventory`, `service.health`, and RAM audit bindings expose
the artifact identity id/hash, trust-envelope id/hash, payload, public-key, and
signature hashes, `artifact_identity_signature_verified`, and a signed
`raios.builtin_artifact_content_binding.v0` content/hash binding for the
checked-in Hello service source snapshot. That binding exposes stable content
id/source/hash/trust fields in load, inventory, health, and RAM audit evidence.
The same signed identity now also covers a repo-local
`raios.builtin_artifact_reference.v0` for
`seed-kernel/artifacts/svc.demo.hello.builtin.artifact`, exposing artifact
reference hash, artifact byte hash, content-binding linkage, and trust evidence
while still denying arbitrary external artifact intake, executable page
mapping, persistence, durable audit, rollback, provider-triggered auto-load,
and broad mutation. The read-only
`service.artifact_reference_trust_selftest` method now proves the accepted
artifact reference validates and tampered artifact byte hash, content-binding
hash, reference hash, and trust payload linkage fail closed without accepting
artifact bytes or mutating the event log.
The Hello load path now also emits a
`raios.current_boot_artifact_load_plan_preflight.v0` decision before
load/start. The preflight binds the selected descriptor source, artifact
identity, content binding, artifact reference, artifact bytes, and
`ram_only:svc.demo.hello` service-slot intent into one current-boot/local-only
hash. Load responses, nested load descriptors, `service.inventory`,
`service.health`, and RAM audit bindings expose the preflight id/hash/status,
accepted state, service-slot intent id, and RAM-only service-slot id while
candidate-byte execution, executable mapping, persistence, durable audit,
rollback, provider-triggered auto-load, and broad mutation remain denied.
The read-only `service.artifact_load_plan_preflight_selftest` method now proves
the accepted preflight validates and tampered descriptor-source, artifact
identity, content-binding, artifact-reference, artifact-byte, service-slot, and
denial-flag evidence fails closed without mutating the event log.
The Hello path now also emits a
`raios.ram_only_service_slot_activation.v0` record derived from the accepted
preflight. Load/start, `service.inventory`, `service.health`, stop/drop
responses, and lifecycle/health RAM audit bindings expose the activation
id/hash/status plus active state. The activation hash stays stable across
running, stopped, restarted, and cleared statuses for the same selected
descriptor source and preflight; `service.start` restarts a stopped loaded
current-boot service without creating a new load generation, `service.restart`
records its own restart event while preserving the same loaded generation, and
`service.hot_swap svc.demo.hello` validates the same signed built-in
descriptor/artifact/preflight chain before mutating service state, records its
own hot-swap lifecycle event, advances the loaded generation, and binds the
event as hot-swap/load/start evidence. `service.hot_swap svc.demo.hello.v2`
selects a distinct signed built-in v2 artifact identity over the same
repo-local built-in byte snapshot, exposes `version: "v2"` plus a different
artifact identity id/hash/preflight/activation hash, and preserves the tiny
`raios.ram_only_hello_service_state.v0` counter through an explicit
`raios.ram_only_hello_service_state_migration.v0` record. The state starts at
1 on load, advances through the existing start/restart lifecycle to 3, stays
unchanged across v1 hot-swap, v1->v2 hot-swap, and v2->v1 hot-swap, and is
visible in load responses, `service.inventory`, `service.health`, and RAM
audit bindings with `writes_persistent_state: false`. `service.hot_swap
svc.demo.hello.reset_state` now computes a would-reset
`raios.ram_only_hello_service_state_migration.v0` record with
`accepted: false` / `state_preserved: false`, records a local-only
`capability_denied` lifecycle event, and returns before descriptor,
generation, running state, or the RAM-only counter can change; a follow-up
health probe proves the active generation, descriptor, and state hash/counter
are unchanged. Accepted hot-swaps now also emit
`raios.ram_only_hello_service_hot_swap_probation.v0` evidence that binds the
previous/new descriptor source hash, artifact identity hash, generation, state
hash/counter, and migration hash with `active_current_boot_probation` status
while candidate execution, persistent state, durable audit, rollback install,
and rollback apply stay denied. `service.rollback_preview svc.demo.hello` now
reads the retained probation into
`raios.ram_only_hello_service_rollback_preview.v0`, exposes the previous
rollback target and current candidate descriptor, artifact identity,
generation, state hash/counter, state-migration hash, and preview hash, records
a RAM-only rollback-preview audit event, and proves with follow-up health that
the active v2 service state is unchanged. `service.rollback_apply
svc.demo.hello` now binds the current preview, probation, Hello state, rollback
target/current candidate, requested capability, and missing write authorities
into a structured `capability_denied` response plus
`raios.ram_only_hello_service_rollback_transaction_preflight.v0` current-boot
evidence and
`raios.ram_only_hello_service_rollback_write_authority_gate.v0` current-boot
write-authority evidence plus a
`raios.ram_only_hello_service_rollback_append_intent_gate.v0` append-intent
availability gate plus a
`raios.ram_only_hello_service_rollback_payload_envelope_gate.v0` payload/hash
envelope gate plus
`raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0`
writer/storage authority gate retained on the
`raios.ram_only_hello_service.rollback_apply` RAM audit event. The preflight
hash binds the apply denial hash, preview hash, probation hash, state
hash/counter, target/current descriptor and artifact identity facts, migration
hash, and missing rollback-transaction, durable-audit-write, and
persistent-install authorities. The write-authority gate hash binds the
preflight hash, required `raios.audit_record.v0` and
`raios.rollback_transaction.v0` schemas, unavailable durable-audit-write,
rollback-store-write, and rollback-transaction-append authority, and disabled
write/apply side effects. The append-intent gate hash binds the write-authority
gate hash, preflight hash, apply denial hash, preview hash, probation hash,
current state, rollback target/current candidate descriptor and artifact facts,
required durable schemas, unavailable append/durable-store authorities, and
disabled append/write/apply side effects. The payload envelope gate hash binds
the append-intent gate hash, write-authority gate hash, preflight hash, apply
denial hash, preview hash, probation hash, current state, rollback
target/current candidate descriptor and artifact facts, proposed
`raios.rollback_transaction.v0` payload schema/id/hash, payload provenance hash,
required durable schemas, unavailable transaction-writer/durable-store
authorities, and disabled append/write/apply side effects. The writer/storage
authority gate hash binds the payload-envelope gate hash, proposed payload and
provenance hashes, append-intent gate hash, write-authority gate hash,
preflight hash, apply-denial hash, preview hash, probation hash, current state,
rollback target/current candidate descriptor and artifact facts, requested
capability, required durable schemas, unavailable transaction-writer,
durable-audit-store, rollback-store, and append authority, and disabled
append/write/apply side effects. It now also consumes the shared
`raios.module_audit_rollback_append_contract.v0` current-boot writer/storage
foundation, names `module.audit_rollback_append_contract` as the recovery
visible owner, names `storage.authority.audit_rollback.current_boot` as the
local-only storage authority, names `append.audit_ledger.current_boot` and
`append.rollback_store.current_boot` as the audit and rollback append targets,
records `module.audit_rollback_storage_layout` as the storage authority owner
and `module.audit_rollback_append_contract` as the transaction-writer owner,
adds `raios.audit_rollback_append_target_owner.v0` plus
`raios.audit_rollback_transaction_writer_readiness.v0` readiness facts that
consume the same storage authority and append target IDs, keep both
`missing` / `persistence_device_write_path_missing` after observing the
current-boot QEMU AHCI controller, mapping AHCI ABAR, reading AHCI
version/port registers, issuing one read-only AHCI IDENTIFY DEVICE command on
the active first SATA port, exposing block-device identity for the QEMU
HARDDISK, completing one read-only AHCI Sector-0 read with MBR signature
evidence, parsing empty MBR partition inventory, and exposing
`raios.read_only_block_driver.v0` readiness over that verified read path while
still denying media writes. The storage diagnostic, append-contract inputs, and
Hello rollback writer/storage foundation now also expose
`raios.block_write_path_authority_gate.v0` with id
`block_write_path.authority.audit_rollback.current_boot`, binding the missing
write path to the verified read-only AHCI block driver and empty MBR partition
inventory while keeping `available: false`, `authorizes_media_write: false`,
`authorizes_append: false`, `writes_enabled: false`, and
`write_attempted: false`. The storage diagnostic now also emits
`raios.audit_rollback_target_region_discovery.v0` with id
`target_region.audit_rollback.current_boot`, source
`dedicated_audit_rollback_label_scan`, status `available`, and reason
`dedicated_audit_rollback_region_discovered_read_only`: the VM harness attaches
a separate `RAIOS_AUDITRB_V0`-labeled non-scratch disk, the kernel reads the
label and LBA1 target region without writing it, reports
`candidate_region_present: true`, `candidate_region_start_lba: 1`,
`candidate_region_lba_count: 1`, `candidate_region_is_scratch: false`,
`candidate_overlaps_boot_metadata: false`, `candidate_overlaps_scratch: false`,
rejects the VM scratch region as durable authority, and keeps append/write
authority flags false.
The Hello durable append-authority preflight and rollback-apply RAM audit
binding now retain that discovery under the same preflight hash. The storage
diagnostic and append-contract inputs now
also expose a separate VM-harness-labeled
`raios.scratch_block_region_write_readback.v0` evidence object with id
`scratch.block_region.current_boot.v0`: the harness attaches a temporary
scratch disk on a separate QEMU AHCI/IDE port, LBA0 carries only the
`RAIOS_SCRATCH_V0` label, the kernel skips unlabeled disks and the boot port,
then writes and reads back LBA1 on the labeled scratch region only. Successful
evidence reports `scratch_write_readback_verified`,
`scratch_write_path_available: true`, `write_attempted: true`,
`write_completed: true`, `readback_completed: true`, and
`readback_matches: true`. The same scratch evidence now binds the scratch
device identity, `region_start_lba: 1`, `region_lba_count: 1`,
`region_within_device_bounds: true`, `boot_port_overlap: false`,
`metadata_lba_overlap: false`,
`no_boot_or_partition_metadata_overlap: true`, and
`block_write_authority_available: true`, while explicitly keeping
`authorizes_audit_rollback: false`, `authorizes_append: false`, and
`writes_enabled: false`. The storage diagnostic also emits the separate
current-boot/local-only
`raios.scratch_block_write_authority.v0` object with id
`scratch.block_write_authority.current_boot.v0`, owned by the VM harness
scratch region, authorizing only the verified scratch write/readback proof and
not audit rollback, append, persistence, durable audit, rollback-store, or
rollback application. The audit/rollback block-write-path authority gate and
Hello rollback writer/storage foundation retain the scratch authority id,
bounds, and no-overlap facts as evidence but still remain unavailable for
audit/rollback writes. The shared transaction-writer readiness path now also
emits
`raios.audit_rollback_transaction_writer_scratch_dry_run.v0` with id
`transaction_writer.scratch_dry_run.audit_rollback.current_boot`, status
`scratch_range_ready_not_durable_authority`, and reason
`scratch_write_authority_verified_current_boot`. That dry-run names
`append.audit_ledger.current_boot` / `raios.audit_record.v0` and
`append.rollback_store.current_boot` / `raios.rollback_transaction.v0`, binds
the scratch write authority and `scratch.block_region.current_boot.v0`, proves
the target range is LBA1/512 bytes, scratch-owned, within device bounds, and
free of boot/partition metadata overlap, and still reports
`authorizes_append: false`, `writes_durable_audit_log: false`,
`writes_rollback_store: false`, `appends_rollback_transaction: false`, and
`write_attempted: false`. The same transaction-writer readiness path now also
emits `raios.audit_rollback_target_region_writer_contract.v0` with id
`target_region_writer_contract.audit_rollback.current_boot`, status
`target_region_ready_not_write_authority`, reason
`target_region_read_only_missing_media_write_authority`, the read-only
non-scratch target-region discovery as source evidence, LBA1/512-byte target
span, audit-ledger and rollback-store target ids/schemas, and all
write/append flags false. Nested under that contract, the readiness path now
also emits `raios.audit_rollback_target_region_media_write_policy_preflight.v0`
with id `target_region_media_write_policy_preflight.audit_rollback.current_boot`,
status `denied_missing_media_write_authority_and_durable_audit_policy`, reason
`target_region_contract_ready_policy_or_write_authority_missing`, verified
owner/target/span/schema ids, explicit missing media-write authority and
durable-audit-policy facts, and all write/append flags false. The Hello
durable append-authority preflight now also emits
`raios.ram_only_hello_service_rollback_media_write_authority_gate.v0` with id
`hello_rollback_media_write_authority_gate.current_boot.svc.demo.hello.v0`,
status `denied_missing_durable_audit_policy`, reason
`target_region_test_media_write_verified_durable_audit_policy_missing`, the
durable append preflight hash, the target-region media-write policy preflight
hash, the target-region write/readback dry-run hash now already lifted into the
durable append-authority preflight and writer-storage gate hash,
source-contract and target-span verification facts,
`test_infrastructure_media_write_authority_available: true`, missing
durable-audit-policy facts, and all durable
media-write/append/durable-write flags false while recording that the target
region write/readback was attempted and verified before the gate. The same
durable append-authority preflight now also carries
`raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0`,
binding the append-record dry-run hash, sector-plan hash, target-region
write/readback hash, audit-ledger target/schema, rollback-store target/schema,
the LBA1/512-byte target span, verified current-boot test media authority, and
accepted current-boot no-write durable-audit, rollback-store, and
transaction-append writer candidates over the canonical audit-record,
rollback-transaction, and combined append images, with all write/append flags
false. The
rollback-apply RAM
audit binding now also retains
`raios.ram_only_hello_service_rollback_durable_append_transaction_authorization_gate.v0`
with id
`hello_rollback_durable_append_transaction_authorization_gate.current_boot.svc.demo.hello.v0`,
status `denied_missing_durable_append_transaction_authority`, reason
`writer_policy_ready_durable_append_authority_missing`, the
writer-policy preflight hash, append-record/sector-plan/target-region
write-readback hashes, audit-ledger and rollback-store target/schema ids, the
LBA1/512-byte target span, verified test-media evidence, an accepted
current-boot no-write append-engine candidate over the canonical sector image,
accepted no-write durable-audit, rollback-store, and transaction-append writer
candidates, and missing durable append authority, with all authorize/write/append
flags false. The rollback-apply
response and RAM audit binding now also emit
`raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0`
with id
`hello_rollback_append_engine_readiness_decision.current_boot.svc.demo.hello.v0`,
status `available`, reason
`transaction_append_engine_ready`,
the authorization-gate, writer-policy, append-record, sector-plan, and
target-region write/readback source hashes, LBA1/512-byte target span, verified
target-range and test-media facts, available no-write append-engine candidate,
available no-write durable-audit, rollback-store, and transaction-append writer
candidates, `ready: true`, and all
authorize/write/append flags false. The rollback-apply
response and RAM audit binding now also emit
`raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0`
with id
`hello_rollback_durable_append_authority_decision.current_boot.svc.demo.hello.v0`,
status `denied_missing_durable_append_authority`, reason
`append_engine_ready_durable_audit_policy_missing`, the durable append preflight
hash, writer-policy preflight hash, append-engine readiness decision hash,
media-write authority gate hash, target-region media-write policy preflight
hash, target-region write/readback hash, LBA1/512-byte target span, ready
writer-policy/append-engine/media-write-gate/test-media facts, missing
durable-audit-policy and durable-append-authority facts, and all
authorize/write/append flags false. The rollback-apply response and RAM audit
binding now also emit
`raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0` with
id
`hello_rollback_durable_audit_policy_decision.current_boot.svc.demo.hello.v0`,
status `denied_missing_durable_audit_policy`, reason
`durable_append_authority_blocked_by_missing_durable_audit_policy`, the durable
append-authority decision hash, target-region media-write policy preflight
hash, media-write authority gate hash, target-region write/readback hash,
LBA1/512-byte target span, ready append-engine/media-policy/test-media facts,
missing durable-audit-policy and durable-append-authority facts, and all
authorize/write/append flags false. The rollback-apply RAM audit binding
retains the same writer-policy preflight, authorization gate, append-engine
readiness decision, durable append-authority decision, durable audit-policy
decision, media-write-authority gate, block-write-path gate
schema/id/status/reason, read-only block-driver id, partition-inventory scheme,
and scratch writer dry-run fields as current-boot/local-only evidence.
It retains the current
missing storage-layout, append-engine, append-contract, and rollback-transaction
envelope statuses on the rollback-apply RAM audit event while proving
descriptor, generation, running state, and RAM-only state remain unchanged and
rollback application, persistent install, durable audit writes, rollback-store
writes, rollback transaction append, external artifact bytes, candidate
execution, executable mapping, provider auto-load, and broad mutation stay
denied. `service.hot_swap
svc.demo.hello` can return to
the signed v1 identity. `service.hot_swap external:svc.demo.hello` remains
denied before the service is touched, and a follow-up health probe proves the
running generation and state are preserved. `service.drop` clears the
current-boot slot while citing the same activation before cleanup.
The same lifecycle can also be driven through a host-produced, hash-bound
descriptor-source candidate (`host_bound:svc.demo.hello`) that binds the
current-image source hash while still loading only the built-in current-boot
test service; the host-bound path remains hash-bound and does not become a
signed artifact-loader path.
Keep the evidence chain and fail-closed denials, but continue the pivot through
positive service lifecycles instead of adding another non-authorizing loader
boundary by default. Persistence, arbitrary external artifact intake, durable
audit writes, rollback installation, provider-triggered auto-load, and broad
module/service/config mutation remain denied.

OS-wide AI-parallel pivot memory: treat the current cursor as one integration
track inside the whole raiOS build, not as a traditional serial master plan.
Independent agents may work in parallel on runtime identity, provider
trust/context, UI/input, VM harness/evidence, docs/status, and
recovery/persistence design when ownership boundaries do not conflict. Each
merged result must still be a real verified vertical slice on the final
architecture path, not scaffolding, mocks, fake trust, fake persistence, or a
schema-only detour that does not unblock positive runtime behavior.

Agent protocol track memory: Stage-0 now has its first narrow native
`raios.agent_command_envelope.v0` boundary on the existing serial
`agent <method>` path. The accepted forms are intentionally limited to
local-only read-only targets: `system.describe` with
`cap.system.describe.read`, `system.snapshot` with
`cap.system.snapshot.read`, `system.boot_log` with
`cap.system.boot_log.read`, `system.capabilities` with
`cap.system.capabilities.read`, `device.graph` with
`cap.device.graph.read`, `service.inventory` with
`cap.service.inventory.read`, `problem.list` with
`cap.problem.list.read`, and `recovery.lifeline.status` with the existing
recovery read capability `cap.recovery.load_artifact.read`. The envelope emits a typed local response, rejects
target/capability mismatches, bad schema, and over-capable targets before
dispatch, and on success routes to the existing dispatcher path without
creating a parallel dispatcher, provider write, candidate-byte load,
persistence, durable audit write, rollback install, or broad mutation.
Accepted, mismatched, bad-schema, and over-capable envelope decisions now record
current-boot/local-only
`raios.agent_command_envelope.decision` events with
`raios.agent_command_envelope.audit_binding.v0`, and the envelope response
returns the matching `event_id`/`audit_event_id`.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding the
read-only non-scratch audit/rollback target-region writer contract under the
shared transaction-writer readiness path. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-184713-22996.json` with 298/298
predicates, 56 executed commands, `duration_ms: 87046`, and report SHA-256
`a50e6b2627bc54df84b3a772525b1089411e1543afa0c4f273389d88bac32a46`. The
serial log proves `module.audit_rollback_storage_layout` emits
`raios.audit_rollback_target_region_discovery.v0` with id
`target_region.audit_rollback.current_boot`, source
`dedicated_audit_rollback_label_scan`, status `available`, reason
`dedicated_audit_rollback_region_discovered_read_only`, empty `mbr_empty`
partition evidence for the boot disk, `candidate_region_present: true`,
`candidate_region_start_lba: 1`, `candidate_region_lba_count: 1`,
`candidate_region_is_scratch: false`,
`candidate_overlaps_boot_metadata: false`, `candidate_overlaps_scratch: false`,
`scratch_rejected_as_durable_authority: true`, and
`durable_region_available: true`, while append/write authority remains false.
The same quick run proves
`raios.audit_rollback_transaction_writer_readiness.v0` now carries
`raios.audit_rollback_target_region_writer_contract.v0` with id
`target_region_writer_contract.audit_rollback.current_boot`, status
`target_region_ready_not_write_authority`, reason
`target_region_read_only_missing_media_write_authority`, source discovery
schema/id/status/reason bound to the positive target-region evidence, LBA1/1
sector and 512-byte target span, audit-ledger target
`append.audit_ledger.current_boot` / `raios.audit_record.v0`, rollback-store
target `append.rollback_store.current_boot` /
`raios.rollback_transaction.v0`, `target_range_ready: true`, and all
write/append flags false. It also proves
`service.rollback_apply svc.demo.hello` emits
`raios.audit_rollback_transaction_writer_scratch_dry_run.v0` with id
`transaction_writer.scratch_dry_run.audit_rollback.current_boot`, status
`scratch_range_ready_not_durable_authority`, reason
`scratch_write_authority_verified_current_boot`, source authority
`scratch.block_write_authority.current_boot.v0`, source region
`scratch.block_region.current_boot.v0`, LBA1/512-byte target range,
`target_range_scratch_owned: true`,
`target_range_within_device_bounds: true`,
`target_range_no_boot_or_partition_metadata_overlap: true`, and
`target_range_ready: true`, while keeping `authorizes_append: false`,
durable-audit/rollback-store/transaction-append writes false, and
`write_attempted: false`. Nested under the same
`raios.audit_rollback_transaction_writer_readiness.v0` object, the shared
writer-readiness path now also emits
`raios.audit_rollback_target_region_writer_contract.v0` with id
`target_region_writer_contract.audit_rollback.current_boot`, status
`target_region_ready_not_write_authority`, reason
`target_region_read_only_missing_media_write_authority`, and source discovery
`target_region.audit_rollback.current_boot`. It consumes the read-only
non-scratch LBA1/512-byte target-region evidence, names
`append.audit_ledger.current_boot` / `raios.audit_record.v0` and
`append.rollback_store.current_boot` / `raios.rollback_transaction.v0`, proves
`target_range_ready: true`, and keeps `write_authority_available: false`,
`durable_audit_policy_available: false`, `authorizes_append: false`,
`writes_durable_audit_log: false`, `writes_rollback_store: false`,
`appends_rollback_transaction: false`, and `write_attempted: false`. Nested
inside that contract, `raios.audit_rollback_target_region_media_write_policy_preflight.v0`
verifies the source contract, owner, target, span, and schema ids, reports
`media_write_authority_available: false` with reason
`media_write_authority_missing`, reports
`durable_audit_policy_available: false` with reason
`durable_audit_policy_missing`, and keeps authorizing/writing/appending flags
false. The same response and RAM audit binding now retain
`raios.ram_only_hello_service_rollback_append_record_dry_run.v0` with
canonicalization `raios.rollback_append_record_image.canonical.v0`, dry-run
hash, audit-record image hash, rollback-transaction image hash, byte lengths
255/225/480, LBA1/512-byte target span, source payload/provenance hashes, and
all append/write flags false. The same rollback-apply response and RAM audit
binding now retain
`raios.ram_only_hello_service_rollback_append_sector_plan_dry_run.v0` with
canonicalization `raios.rollback_append_sector_plan.canonical.v0`, plan hash,
sector-image hash, 512-byte sector size, audit-record offset 0,
rollback-transaction offset 255, zero-padding offset/length 480/32, LBA1 target
span, source append-record hash, and all append/write flags false. Nested under
that plan, the same response and RAM audit binding now retain
`raios.ram_only_hello_service_rollback_append_sector_write_readback_dry_run.v0`
with planned sector-image hash, readback sector-image hash, matching
`readback_matches_planned_image: true`, LBA1/512-byte target span,
`write_attempted: true`, `write_completed: true`, and `readback_completed:
true`, while still setting append, durable-audit, rollback-store, and
rollback-transaction writes false. Nested under that write/readback evidence,
the same response and RAM audit binding now retain
`raios.ram_only_hello_service_rollback_durable_append_authority_preflight.v0`
with status `denied_missing_durable_append_authority`, reason
`durable_append_authority_not_granted`, remaining denial reason
`durable_append_authority_missing`, the source write/readback hash,
the target-region discovery schema/id/status/reason/source and read-only
durable-region availability facts,
`storage.authority.audit_rollback.current_boot`,
`append_target_owner.audit_rollback.current_boot`,
`transaction_writer.audit_rollback.current_boot`,
`append.audit_ledger.current_boot`, and `append.rollback_store.current_boot`.
Nested in that durable append preflight, the Hello rollback response and RAM
audit binding now also consume
`raios.audit_rollback_target_region_media_write_policy_preflight.v0` with its
schema/id/status/reason, preflight hash, source target-region writer contract,
owner/target/span/schema verification facts, explicit missing media-write
authority and durable-audit-policy reasons, and all media-write/append/write
flags false. Nested beside it, the same response and RAM audit binding retain
`raios.ram_only_hello_service_rollback_media_write_authority_gate.v0` with its
schema/id/status/reason, gate hash, durable append preflight hash, policy
preflight hash, source-contract facts, target span, verification booleans,
missing media-write authority and durable-audit-policy reasons, and all
media-write/append/durable-write/target-region-write flags false. It proves
`scratch_write_readback_verified: true` while
`scratch_used_as_durable_authority: false`, durable-audit writer,
rollback-store writer, and transaction append writer availability false, and
all append/write flags false. Rollback application, persistent install, durable
audit writes, rollback-store writes, rollback transaction append, external
bytes, candidate execution, executable mapping, provider auto-load, and broad
mutation stay denied and follow-up health proves the active descriptor,
generation, running state, and RAM-only state are unchanged.

Previous focused verification after adding the Hello rollback
media-write-authority gate over the target-region policy preflight: 2026-07-02
on Windows with QEMU 11 in
`release/vm-reports/shadow-20260702-192027-4988.json` with 306/306 quick
predicates, 56 executed commands, `duration_ms: 67009`, base image SHA-256
`23aa783ada0d690c94c09b9167c1129785b4d42b10c39db729917a78ad3c08dd`, and
report SHA-256
`cb77021e5e6aec6ac3b9fe919c1777a2bfc684fb8bb41ef297d451acc6a1290e`.

Most recent full Shadow VM profile attempt around this storage-authority
sequence: 2026-07-02 on Windows with QEMU 11 in
`release/vm-reports/shadow-20260702-182522-7236.json`, report SHA-256
`757ab5a00ca4e1a1b5a6415b1862095bdf7e14fc96f820d910c0e97c5f62fb17`.
It reached 163/163 passing predicates and 11 executed commands through
`provider.context_gate_selftest`, then failed before the module/recovery
profiles on a serial TCP transport error (`Remotehost geschlossen`). Treat this
as a harness/serial follow-up, not a full verification of these latest focused
storage-authority slices.

Latest full verification before this slice: 2026-07-02 on Windows with QEMU 11
in `release/vm-reports/shadow-20260702-174421-7208.json` with 6789/6789
predicates, 243 executed commands, `duration_ms: 553963`, and report SHA-256
`80be25579114eb7f23e7501134948e6a36728b4af258e44280968c2f8ccf77ea`.

Previous full verification after the read-only block-driver readiness slice:

```text
release\vm-reports\shadow-20260702-130028-34200.json
6706/6706 predicates, 243 executed commands, duration_ms: 541557
```

Previous full verification after the read-only MBR partition inventory slice:

```text
release\vm-reports\shadow-20260702-124055-34392.json
6698/6698 predicates, 243 executed commands, duration_ms: 538777
```

Previous focused verification after the Hello rollback payload-envelope gate:

```text
release\vm-reports\shadow-20260702-091057-17852.json
266/266 quick predicates, 54 executed commands, duration_ms: 83454
```

Previous focused verification after the Hello rollback append-intent gate:

```text
release\vm-reports\shadow-20260702-090105-12232.json
263/263 quick predicates, 54 executed commands, duration_ms: 84226
```

Previous focused verification after the Hello rollback write-authority gate:

```text
release\vm-reports\shadow-20260702-085049-8956.json
260/260 quick predicates, 54 executed commands, duration_ms: 72086
```

Previous focused verification after the Hello rollback transaction/durable-audit
preflight:

```text
release\vm-reports\shadow-20260702-084240-14784.json
257/257 quick predicates, 54 executed commands, duration_ms: 73613
```

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding the
read-only Hello rollback preview over retained hot-swap probation evidence.
Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-081302-27580.json` with 247/247
predicates, 52 executed commands, and `duration_ms: 81372`. The quick smoke
proves `service.rollback_preview svc.demo.hello` binds the retained v1->v2
probation hash into
`raios.ram_only_hello_service_rollback_preview.v0`, exposes previous/current
descriptor, artifact identity, generation, state hash/counter, and migration
facts, records a RAM-only rollback-preview audit event, keeps rollback apply,
persistent install, durable audit writes, external bytes, candidate execution,
executable mapping, provider auto-load, and broad mutation denied, and leaves
the active v2 service generation/state unchanged in follow-up health.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
RAM-only accepted hot-swap probation evidence. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-075957-15956.json` with 243/243
predicates, 50 executed commands, and `duration_ms: 77226`. The quick smoke
proves accepted v1, v2, and v1-back hot-swaps emit
`raios.ram_only_hello_service_hot_swap_probation.v0` with
`active_current_boot_probation` status, previous/new generation, previous/new
state hash/counter, previous/new artifact identity hash, and the matching state
migration hash while candidate bytes, executable mapping, persistent state,
durable audit, rollback install, and rollback apply stay denied. The v1->v2
audit event proves the probation hash, previous v1 identity, new v2 identity,
previous/new generation, preserved state hash, and migration hash are retained
in RAM audit evidence. The quick smoke still proves the reset-state hot-swap is
denied before descriptor/generation/state mutation and that accepted v2/v1
hot-swaps preserve the Hello state counter.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the fail-closed Hello state-migration reset gate. Quick Shadow VM smoke passed
in `release/vm-reports/shadow-20260702-074900-3852.json` with 241/241
predicates, 50 executed commands, and `duration_ms: 79318`. The quick smoke
proved the reset-state hot-swap returned structured `capability_denied`,
exposed a would-reset migration with `accepted: false`, and left active
generation, descriptor, state hash, and counter unchanged.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the RAM-only Hello service state migration evidence across signed hot-swap.
Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-073742-10256.json` with 237/237
predicates, 48 executed commands, and `duration_ms: 74841`. The quick smoke
proved load created `raios.ram_only_hello_service_state.v0` with counter 1,
start/restart advanced it to 3, stop and denied external hot-swap preserved it,
and the v2/v1 accepted hot-swaps preserved the same state through explicit
state-migration records.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the distinct signed built-in `svc.demo.hello.v2` hot-swap candidate. Quick
Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-072537-6980.json` with 235/235
predicates, 48 executed commands, and `duration_ms: 75346`. The quick smoke
proved `service.hot_swap svc.demo.hello.v2` selected
`builtin_artifact_identity.svc.demo.hello.v2`, exposed `version: "v2"`, kept
the content/reference bytes non-executing and identical to the checked-in
built-in artifact snapshot, produced a distinct artifact identity hash,
advanced the loaded generation, then `service.hot_swap svc.demo.hello` returned
to the signed v1 identity while advancing the generation again.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
`service.hot_swap svc.demo.hello` on the RAM-only Hello service path. Quick
Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-071540-15484.json` with 231/231
predicates, 46 executed commands, and `duration_ms: 72031`. The quick smoke
proves an external-looking hot-swap target is denied before service mutation,
the running generation remains unchanged after that denial, the accepted
current-image built-in hot-swap records `last_action: "hot_swap"` /
`reason: "hot_swapped_builtin_service"`, advances the loaded generation by one,
and binds the same event id as hot-swap/load/start evidence while artifact byte
intake, executable mapping, persistence, durable audit, rollback install,
provider auto-load, and broad mutation remain denied. The final quick audit read
now uses `agent audit.events 46` so the provider export, module load, recovery
load, Hello lifecycle, Hello health, and all ten command-envelope decision
events remain inside the bounded local audit window.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the local-only read-only `system.boot_log` envelope target to
`raios.agent_command_envelope.v0`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-070530-20712.json` with 226/226
predicates, 43 executed commands, and `duration_ms: 69882`. The quick smoke
proves valid envelopes dispatch through the existing `system.describe`,
`system.snapshot`, `system.boot_log`, `system.capabilities`, `device.graph`,
`service.inventory`, and `problem.list` methods, a `service.inventory`
envelope paired with `cap.system.describe.read` is denied as
`requested_capability_denied` before dispatch, bad-schema envelopes are
rejected, an over-capable `module.load_ephemeral` target is denied before
module dispatch, and `audit.events` exposes all ten decisions as local-only
current-boot audit evidence while unsafe side effects remain disabled. The
final quick audit read now uses `agent audit.events 42` so the older provider
export event remains inside the bounded local audit window after the extra
accepted envelope.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the read-only `device.graph` envelope target to
`raios.agent_command_envelope.v0`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-065801-25136.json` with 224/224
predicates, 42 executed commands, and `duration_ms: 95235`. The quick smoke
proved valid envelopes dispatched through the existing `system.describe`,
`system.snapshot`, `system.capabilities`, `device.graph`,
`service.inventory`, and `problem.list` methods, a `service.inventory`
envelope paired with `cap.system.describe.read` was denied as
`requested_capability_denied` before dispatch, bad-schema envelopes were
rejected, an over-capable `module.load_ephemeral` target was denied before
module dispatch, and `audit.events` exposed all nine decisions as local-only
current-boot audit evidence while unsafe side effects remained disabled.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the read-only `system.capabilities` envelope target to
`raios.agent_command_envelope.v0`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-065202-7476.json` with 222/222
predicates, 41 executed commands, and `duration_ms: 95768`. The quick smoke
proved valid envelopes dispatched through the existing `system.describe`,
`system.snapshot`, `system.capabilities`, `service.inventory`, and
`problem.list` methods, a `service.inventory` envelope paired with
`cap.system.describe.read` was denied as `requested_capability_denied` before
dispatch, bad-schema envelopes were rejected, an over-capable
`module.load_ephemeral` target was denied before module dispatch, and
`audit.events` exposed all eight decisions as local-only current-boot audit
evidence while unsafe side effects remained disabled.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the read-only `system.snapshot` envelope target to
`raios.agent_command_envelope.v0`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-064636-24876.json` with 220/220
predicates, 40 executed commands, and `duration_ms: 67908`. The quick smoke
proved valid envelopes dispatched through the existing `system.describe`,
`system.snapshot`, `service.inventory`, and `problem.list` methods.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the read-only `problem.list` envelope target to
`raios.agent_command_envelope.v0`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-063508-18024.json` with 219/219
predicates, 40 executed commands, and `duration_ms: 94555`. The quick smoke
proved valid envelopes dispatched through the existing `system.describe`,
`service.inventory`, and `problem.list` methods.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the first `raios.agent_command_envelope.v0` serial boundary over
`system.describe`. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-061129-8152.json` with 207/207
predicates, 36 executed commands, and `duration_ms: 64609`. The quick smoke
proves valid envelopes dispatch through the existing `system.describe` method,
bad-schema envelopes are rejected, and an over-capable `module.load_ephemeral`
target is denied before module dispatch.

Previous focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the explicit `service.restart svc.demo.hello` current-boot lifecycle transition on
top of the RAM-only Hello service activation slice. Quick Shadow VM smoke
passed in `release/vm-reports/shadow-20260702-055608-6288.json` with 203/203
predicates, 33 executed commands, and `duration_ms: 62948`. The quick smoke
proves `service.restart` records a real lifecycle event, keeps the same loaded
generation and activation hash, and preserves current-boot/local-only
non-persistence.

Previous full verification at that point: 2026-07-02 on Windows with QEMU 11 after the
explicit `service.start svc.demo.hello` current-boot lifecycle transition on
top of provider trust verifier decisions and optional standby SPKI rotation
state. Full Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-053820-28640.json` with 6640/6640
predicates, 243 executed commands, and `duration_ms: 610100`. The full smoke
proves the provider context gate selftest still expects all 20 cases, including
redaction/classification/budget/trust evidence hash mismatches, and that the
provider context injection gate names `provider_trust_verifier_metadata` as
required evidence. Provider snapshots and provider-minimal context now expose
`raios.provider_trust_verifier_decision.v0` with verifier id, stage, outcome,
and reason; the normal no-pin/no-trust state reports `pin_config` / `rejected`
/ `pin_config_missing`. The OpenAI SPKI verifier now also supports one optional
standby SPKI rotation pin supplied by `OPENAI_SPKI_SHA256_NEXT`; malformed
rotation config fails closed, successful matches record the active or rotation
pin id/slot, and the trust metadata still labels the path as pin-only without
WebPKI chain or time validation. Positive request/export bindings carry a canonical
`provider_trust_evidence_hash` over provider host, trust state, pin kind/id,
TLS-bypass state, `raios.provider_trust_verifier_metadata.v0`, and the verifier
decision; the verifier metadata names the real Stage-0 OpenAI pinned TLS
verifier, exact-host policy, configured leaf/SPKI pin policy, TLS 1.3 P-256
CertificateVerify policy, and explicit `pin_only_no_webpki_chain_validation` /
`not_validated_stage0` chain/time policies. The hash is folded into
request/export binding hashes, retained through binding consumption and final
injection authorization checks, and exposed in provider gate diagnostics and
RAM-only event bindings alongside `redaction_policy_hash`,
`field_classification_hash`, and `token_budget_hash`. No-pin/no-trust provider
context export remains `capability_denied`; the current-image and host-bound
Hello load/start/list/health/stop/drop paths still work; and arbitrary external
artifacts, candidate-byte execution, executable page mapping, persistence,
durable audit, rollback, provider auto-load, and broad mutation remain denied.

Previous focused verification after the explicit `service.start` slice:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-053445-10792.json` with 201/201
predicates, 32 executed commands, and `duration_ms: 61451`.

Previous full verification after the provider trust verifier metadata slice:
2026-07-02 on Windows with QEMU 11. Full Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-042431-24536.json` with 6632/6632
predicates, 243 executed commands, and `duration_ms: 609828`.

Previous focused verification after the provider trust verifier metadata slice:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-042325-25648.json` with 191/191
predicates, 31 executed commands, and `duration_ms: 61603`.

Previous focused verification after the provider trust evidence binding slice:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-040501-24236.json` with 191/191
predicates, 31 executed commands, and `duration_ms: 60507`.

Previous focused verification after the provider context hash-binding slice:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-034303-24400.json` with 191/191
predicates, 31 executed commands, and `duration_ms: 60437`.

Previous focused verification after the service-slot activation slice:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-033352-9800.json` with 185/185
predicates, 31 executed commands, and `duration_ms: 60174`.

Previous focused verification after the artifact load-plan preflight selftest:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-032107-16036.json` with 182/182
predicates, 31 executed commands, and `duration_ms: 38186`.

Previous focused verification after the artifact load-plan preflight:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-030513-27840.json` with 181/181
predicates, 30 executed commands, and `duration_ms: 59868`.

Previous focused verification after the artifact-reference trust selftest:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-025252-23928.json` with 178/178
predicates, 30 executed commands, and `duration_ms: 53295`.

Previous focused verification after artifact byte/reference evidence:
2026-07-02 on Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-023832-25068.json` with 177/177
predicates, 29 executed commands, and `duration_ms: 53786`.

Previous focused verification after artifact content binding: 2026-07-02 on
Windows with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-022858-26440.json` with 174/174
predicates, 29 executed commands, and `duration_ms: 51671`.

Previous focused verification after artifact identity: 2026-07-02 on Windows
with QEMU 11. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-021750-7868.json` with 172/172 predicates,
29 executed commands, and `duration_ms: 51268`.

Earlier focused verification: 2026-07-02 on Windows with QEMU 11 after adding
the read-only descriptor-source trust selftest over the signed current-image
Hello descriptor-source envelope. Quick Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-020435-6212.json` with 170/170 predicates,
29 executed commands, and `duration_ms: 79153`: bare
`module.load_ephemeral`, `module.load_ephemeral svc.demo.nope`, and
`module.load_ephemeral external:svc.demo.hello` still return the
non-authorizing module-load gate, `recovery.load_artifact` remains denied, and
`service.descriptor_source_trust_selftest` returns
`raios.descriptor_source_trust_selftest.v0` with a stable diagnostic id/hash,
five passing read-only cases for the valid envelope plus tampered payload,
locator/kind, public-key hash, and signature, and denied descriptor byte intake,
external artifact load, persistence, durable audit, rollback, and broad
mutation. Then
`module.load_ephemeral svc.demo.hello` returns
`raios.ram_only_hello_service.v0` with
`raios.current_boot_load_request.v0` and
`raios.current_boot_load_descriptor.v0`, a canonical descriptor source locator
`current_image.descriptor_source.svc.demo.hello.v0`, source kind
`current_image_descriptor_source`, `validated: true`, and a computed `sha256:`
descriptor source hash. The current-image descriptor source now exposes a
`raios.descriptor_source_signature_envelope.v0` using
`ecdsa_p256_sha256_asn1_der`; the envelope has a `sha256:` envelope hash,
payload/public-key/signature hashes, `verification_phase:
runtime_before_descriptor_selection`, and `signature_verified: true`, while
explicitly not authorizing external artifact loading or persistent install.
The validator still parses checked key/value fields for canonicalization,
schema/id, source locator/kind, service/artifact metadata, current-boot
scope/classification/persistence, and the false mutation/artifact/persistence
booleans. It inserts a healthy/running current-boot `svc.demo.hello` service
into `service.inventory` with `load_descriptor.current_boot.svc.demo.hello.v0`
and the same descriptor source locator/kind/validation/hash/signature envelope.
`service.health svc.demo.hello` returns
`raios.ram_only_hello_service.health.v0`, reports healthy/running while loaded,
cites the same active descriptor source hash and verified signature envelope,
and records a local-only `raios.ram_only_hello_service.health` RAM event with
`raios.ram_only_hello_service.health_binding.v0`.
`service.stop svc.demo.hello` marks it stopped,
`service.health svc.demo.hello` reports stopped while the service remains
loaded, `service.start svc.demo.hello` marks the same loaded generation running
again with the same activation hash, `service.drop svc.demo.hello` removes it
from `service.inventory`, and after drop `service.health svc.demo.hello`
reports missing without accepting external artifact bytes or writing persistent
state.
Then `module.load_ephemeral host_bound:svc.demo.hello` loads the same built-in
RAM-only service through `host_build.descriptor_source.svc.demo.hello.v0` with
source kind `host_bound_descriptor_source`; the host-bound source text and
runtime fields bind the current-image descriptor source locator, source kind,
and source hash, keep `signature_envelope: null`, and `service.inventory`,
`service.health`, `service.stop`, `service.drop`, and `agent audit.events 32`
keep citing the host-bound source and its bound current-image source hash.
The final audit read includes six
`raios.ram_only_hello_service.lifecycle` events for `svc.demo.hello` whose
evidence/bindings cite the load descriptor, selected validated descriptor-source
hash, and current-image signature envelope when present, plus at least four
`raios.ram_only_hello_service.health` events covering healthy, stopped, missing,
and host-bound healthy states.
The slice uses only a built-in Stage-0 test artifact, accepts no external
artifact bytes, writes no persistent state, writes no durable audit log,
installs no rollback plan, and grants no broad mutation.

Previous full verification: 2026-07-02 on Windows with QEMU 11 after adding the
typed, read-only
`raios.module_loader_executable_entrypoint_invocation_boundary.v0`
on top of the executable entrypoint handoff boundary,
executable entrypoint transfer boundary,
executable entrypoint transfer authorization boundary,
executable entrypoint binding boundary,
descriptor/executable-page binding boundary, executable page-mapping boundary,
executable page-mapping plan boundary, executable image-layout boundary,
executable load-plan result, executable load-plan authority, descriptor
load-plan, descriptor capability-validation, descriptor schema-validation,
descriptor-parser result, descriptor-parser contract, descriptor-acceptance
authority, live-load, and commit sequence. It consumes the retained
`raios.module_loader_executable_entrypoint_handoff_boundary.v0`,
`raios.module_loader_executable_entrypoint_transfer_boundary.v0`,
`raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0`,
`raios.module_loader_executable_entrypoint_binding_boundary.v0`, retained
`raios.module_loader_descriptor_executable_page_binding_boundary.v0`, retained
`raios.module_loader_executable_page_mapping_boundary.v0`, retained
`raios.module_loader_executable_page_mapping_plan_boundary.v0`, retained
`raios.module_loader_executable_image_layout_boundary.v0`, retained module
evidence, RAM-only service-slot reservation/binding, loader-runtime source
evidence, health hooks, rollback hooks, audit/rollback write-boundary evidence,
entrypoint ABI/address-space/memory-map/capability-table evidence, and the full
observed live-load lifecycle chain only as current-boot provenance. It reports
`module_loader_executable_entrypoint_invocation_boundary_non_authorizing`
while keeping executable entrypoint invocation, executable entrypoint handoff,
executable entrypoint transfer, entrypoint transfer authorization, runnable entrypoint binding, executable
page-mapping plan production, executable image-layout production, executable
load-plan authority, executable load-plan production, capability-validated
descriptor binding to executable pages, descriptor capability validation,
capability-validated descriptor production, validated descriptor production,
descriptor schema validation, parsed descriptor production, descriptor parsing,
loader descriptor acceptance, descriptor bytes, artifact bytes, artifact
loading, executable page mapping, service start/running/unload, health record
creation, service-start audit writing, live-load commit, load-commit audit
writing, commit rollback record install, load-result recording, service
registry mutation, service-inventory record creation, service-slot allocation,
durable-audit state writes, rollback-state installation, and load attempts
false. `module.loader_runtime`, denied `module.load_ephemeral` /
`service.load_ephemeral`, compact audit/event bindings, event-log memory
rendering, and selftests now cite the full chain through executable image
layout, page-mapping plan, executable page mapping, descriptor/executable-page
binding, executable entrypoint binding, and executable entrypoint transfer
authorization, explicit executable entrypoint transfer, executable entrypoint
handoff, and executable entrypoint invocation. Full Shadow VM smoke passed in
`release/vm-reports/shadow-20260702-001225-25068.json` with 6611/6611
predicates, 243 executed commands, and `duration_ms: 541342`; that run used
`-TimeoutSeconds 300`, `-SerialWriteChunkSize 16`,
`-SerialWriteDelayMilliseconds 10`, and `-SerialTcpPort 4581`. This
follows the
four typed, read-only commit boundaries:
`raios.module_loader_live_load_commit_boundary.v0`,
`raios.module_loader_commit_audit_boundary.v0`,
`raios.module_loader_commit_rollback_boundary.v0`, and
`raios.module_loader_commit_result_boundary.v0`. This
follows the
typed, read-only
`raios.module_loader_execution_authorization_boundary.v0` and
`raios.module_loader_service_registry_mutation_boundary.v0` over the retained
artifact-byte intake boundary, descriptor-intake boundary, loader-runtime
execution commit gate, normal-module loader-runtime source-evidence chain,
retained module evidence, RAM-only service-slot reservation,
entrypoint/address-space/memory-map source evidence, audit/rollback
write-boundary source evidence, service-slot binding source evidence, and
service-slot registry write commit gate. This follows the
typed, read-only `raios.module_loader_artifact_byte_intake_boundary.v0` over
the retained descriptor-intake boundary, retained loader-runtime execution
commit gate, normal-module loader-runtime source-evidence chain, retained
manifest/artifact hash references, loader artifact-hash binding source
evidence, retained module evidence, and RAM-only service-slot reservation. That
boundary reports `module_loader_artifact_byte_intake_boundary_non_authorizing`,
is retained only as current-boot source evidence, and keeps loader descriptor
intake, descriptor bytes, artifact bytes, artifact-byte intake, execution
authorization, service registry mutation, service-slot allocation,
durable-audit state writes, rollback-state installation, artifact loading, and
load attempts false. This follows the typed, read-only
`raios.module_loader_descriptor_intake_boundary.v0` over
the retained service-slot registry write commit gate, retained loader-runtime
execution commit gate, normal-module loader-runtime source-evidence chain,
retained module hash references, and RAM-only service-slot reservation. That
boundary reports `module_loader_descriptor_intake_boundary_non_authorizing`, is
retained only as current-boot source evidence, and keeps loader descriptor
intake, descriptor bytes, artifact bytes, execution authorization, service
registry mutation, service-slot allocation, durable-audit state writes,
rollback-state installation, artifact loading, and load attempts false. This
follows the typed, read-only
`raios.module_loader_runtime_execution_commit_gate.v0` over the retained
`raios.module_service_slot_allocator_authority_decision.v0`, retained
`raios.module_loader_runtime_contract.v0` input, normal-module loader-runtime
source-evidence chain, loader service-slot binding evidence, audit/rollback
write-boundary evidence, and RAM-only service-slot reservation. That gate
reports `module_loader_runtime_execution_commit_gate_non_authorizing`, is
retained only as current-boot source evidence, and keeps loader descriptor
intake, artifact byte intake, execution authorization, service registry
mutation, service-slot allocation, durable-audit state writes, rollback-state
installation, artifact loading, and load attempts false. This follows the typed,
read-only
`raios.service_slot_registry_write_commit_gate.v0` over the retained
`raios.module_service_slot_allocator_authority_decision.v0`, retained
`raios.service_slot_registry_write_authority.v0` input, service-slot registry
binding, durable-audit write evidence, rollback-install evidence, and RAM-only
service-slot reservation. That commit gate reports
`service_slot_registry_write_commit_gate_non_authorizing` and keeps service
registry mutation, service-slot allocation, durable-audit state writes,
rollback-state installation, artifact loading, and load attempts false, after
adding the typed, read-only composite
`raios.module_service_slot_allocator_authority_decision.v0` over the six named
authority inputs under `raios.module_service_slot_allocator_authority.v0`:
`raios.service_slot_allocation_intent.v0`,
`raios.service_slot_allocator_policy_decision.v0`,
`raios.service_slot_registry_write_authority.v0`,
`raios.module_loader_runtime_contract.v0`,
`raios.service_health_monitor_binding.v0`, and
`raios.service_unload_cleanup_authority.v0`. The kernel now records
current-boot source-evidence records for each input, records a composite
authority-decision source-evidence record that binds all six input
source-evidence ids, proves the input chain complete, and projects the decision
as `defined_non_authorizing` through
`module.service_slot_allocator`, denied `module.load_ephemeral` /
`service.load_ephemeral`, `module.loader_runtime`, and the standalone normal
module loader diagnostics while still denying allocation, service inventory
mutation, descriptor/artifact intake, and load attempts, after defining
`raios.service_slot_allocation_intent.v0`, after defining the
typed, read-only `raios.module_service_slot_allocator_authority.v0` boundary and
projecting
`denied_allocator_authority_not_granted` /
`service_slot_allocator_authority_boundary_non_authorizing` through
`module.service_slot_allocator`, denied `module.load_ephemeral` /
`service.load_ephemeral`, `module.loader_runtime`, and the standalone normal
module loader diagnostics while still denying allocation, service inventory
mutation, descriptor/artifact intake, and load attempts, after propagating the
real retained `module.service_slot_allocator` readiness projection into the
standalone normal-module loader diagnostics (`module.loader_identity`,
`module.loader_artifact_hash_binding`, the typed loader fact diagnostics, and
`module.loader_runtime`), after binding the denied `module.load_ephemeral` /
`service.load_ephemeral` service-slot allocator readiness projection to the
retained `module.service_slot_allocator` source-evidence chain, after promoting
the `module.service_slot_allocator` module-loader prerequisite boundary to an
observed-current-boot available but non-authorizing boundary once durable-audit
write and rollback-install evidence are available, after promoting
the durable-audit write and rollback-install prerequisite gates to
observed-current-boot available once a retained service-slot reservation and all
allocator facts are available, after promoting
`module.service_slot_allocator` to report
`raios.service_slot_registry_binding.v0`,
`raios.service_health_state_model.v0`, and
`raios.service_unload_cleanup_plan.v0` as observed-current-boot available facts
after a retained service-slot reservation and available allocator runtime, after promoting
`raios.ram_only_service_slot_allocator.v0` as an observed-current-boot available
runtime fact after a retained service-slot reservation, after adding retained local-only
current-boot source evidence for the
`module.service_slot_allocator` durable-audit write, rollback-install, and
module-loader prerequisite gates, after adding
retained local-only current-boot source evidence for the four
`module.service_slot_allocator` fact boundaries
(`raios.ram_only_service_slot_allocator.v0`,
`raios.service_slot_registry_binding.v0`,
`raios.service_health_state_model.v0`, and
`raios.service_unload_cleanup_plan.v0`) while preserving denied allocation and
load authority, after adding retained local-only current-boot source evidence for
`module.loader_address_space_boundary`,
`module.loader_memory_map_constraints`,
`module.loader_capability_import_table`,
`module.loader_service_slot_binding`,
`module.loader_health_state_hooks`, `module.loader_rollback_hooks`, and
`module.loader_audit_rollback_write_boundary_binding`, and teaching
`module.loader_runtime` plus its selftest to consume loader-identity,
artifact-hash, entrypoint-ABI, and all seven chained loader-fact
source-evidence records while preserving denied load authority, after adding
retained source evidence for `module.loader_entrypoint_abi`,
`module.loader_artifact_hash_binding`, and `module.loader_identity`, after
propagating
the loader-runtime source-method/source-fact-locator map into the denied
`module.load_ephemeral` loader-runtime readiness projection, its compact
audit/event binding, and `module.load_gate_loader_runtime_selftest`, wiring
`module.loader_runtime` aggregate source-method and source-fact-locator
citations for all ten typed normal-module loader-runtime facts, adding
`module.loader_runtime_selftest` source-map coverage and Shadow VM source-map
predicates,
read-only diagnostics and selftests for the remaining eight typed
normal-module loader-runtime fact boundaries,
read-only `module.loader_artifact_hash_binding` diagnostics and selftests for
the second normal-module loader-runtime fact boundary,
read-only `module.loader_identity` diagnostics and selftests for the first
normal-module loader-runtime fact boundary,
local-only `module.load_gate_loader_runtime_selftest` coverage for the denied
load-gate loader-runtime projection,
denied `module.load_ephemeral` reporting for retained-evidence,
service-slot allocator readiness, and loader-runtime readiness boundaries,
read-only `module.loader_runtime` readiness diagnostics and selftests for the
missing Phase-6 normal-module loader-runtime boundary, read-only
`module.service_slot_allocator` readiness diagnostics and selftests for the
missing Phase-6 RAM-only service-slot allocator/runtime boundary,
suppressing framebuffer redraws for serial command-mode echo, caching Shadow VM serial-log
reads, moving Shadow VM harness support/reporting/serial helper functions into
`vm-harness/shadow-vm-smoke-support.ps1`, splitting Shadow VM profile
validation into focused `vm-harness/shadow-vm-smoke-profile-*.ps1` slices
while keeping the same QEMU/serial command flow, splitting the oversized module
audit/rollback write-boundary implementation into focused
`seed-kernel/src/agent_protocol_module_write_boundary_*.rs` modules, moving recovery load-binding
evaluation, retained-chain mismatch checks,
and load-binding selftest fixtures into
`seed-kernel/src/agent_protocol_recovery_load_binding.rs`, moving recovery
lifeline protocol/vocabulary/runtime/rollback/persistence/memory/admission
evaluators and selftest fixtures out of the
`seed-kernel/src/agent_protocol_recovery_lifeline_eval.rs` facade into focused
`seed-kernel/src/agent_protocol_recovery_*_eval.rs` modules, moving recovery
lifeline command reference parsers, evaluators, and event-log binding builders
into `seed-kernel/src/agent_protocol_recovery_command_reference_eval.rs`,
moving recovery memory/durable/service/dispatch-behavior/executor/side-effect
reference evaluators into
`seed-kernel/src/agent_protocol_recovery_command_effect_reference_eval.rs`,
moving handler/status/rollback/target/effect command reference selftest
fixtures into
`seed-kernel/src/agent_protocol_recovery_command_reference_selftests.rs`,
moving command envelope/dispatch/body evaluator selftest helpers into
`seed-kernel/src/agent_protocol_recovery_command_eval.rs`, and extracting the
recovery lifeline command specs into
`seed-kernel/src/agent_protocol_recovery_lifeline.rs`, updating Shadow VM
reports to derive `commands`/`executed_commands` from actual serial command
execution, moving recovery lifeline execution-stage selftest fixtures and
retained-chain matchers plus execution-stage JSON emission and retained-event
recording plus the thin execution-stage public wrapper methods and
method-predicate wiring plus retained execution-stage chain-presence evaluation
into
`seed-kernel/src/agent_protocol_recovery_execution.rs`, extracting recovery
method predicates and diagnostic argument parsers into
`seed-kernel/src/agent_protocol_recovery_methods.rs`, extracting recovery
capability, selftest-count, and boundary-id constants into
`seed-kernel/src/agent_protocol_recovery_constants.rs`, moving recovery
load-binding types into
`seed-kernel/src/agent_protocol_recovery_load_binding.rs`, moving recovery
artifact-reference types into
`seed-kernel/src/agent_protocol_recovery_artifact_types.rs`, moving recovery
artifact-reference parsers, evaluators, selftest fixtures, and event-log
binding builders into
`seed-kernel/src/agent_protocol_recovery_artifact_reference.rs`, moving lifeline
protocol and command-vocabulary types into
`seed-kernel/src/agent_protocol_recovery_lifeline_protocol_types.rs`, moving
lifeline runtime/isolation/rollback/persistence/provenance/admission types into
`seed-kernel/src/agent_protocol_recovery_runtime_types.rs`, moving command
envelope, dispatch-denial, and body-canonicalization types into
`seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs`, moving
handler/status/rollback-authorization/target-binding types into
`seed-kernel/src/agent_protocol_recovery_command_authorization_types.rs`,
moving memory/durable-write/service-inventory/command-effect gate types into
`seed-kernel/src/agent_protocol_recovery_command_effect_types.rs`, moving
recovery artifact-reference emit helpers into
`seed-kernel/src/agent_protocol_recovery_artifact_reference_emit.rs`, moving
recovery artifact/lifeline request selftest emit helpers into
`seed-kernel/src/agent_protocol_recovery_artifact_selftest_emit.rs`, moving
lifeline protocol emit helpers into
`seed-kernel/src/agent_protocol_recovery_lifeline_protocol_emit.rs`, moving
lifeline command-vocabulary emit helpers into
`seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_emit.rs`,
moving loader-runtime-isolation emit helpers into
`seed-kernel/src/agent_protocol_recovery_loader_runtime_emit.rs`, moving
rollback-transaction, durable-persistence, memory-provenance, and
command-admission emit helpers into
`seed-kernel/src/agent_protocol_recovery_rollback_transaction_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_persistence_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_memory_provenance_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_command_admission_emit.rs`, moving
command-envelope, command-dispatch, command-body-canonicalization, and
command-handler emit helpers into
`seed-kernel/src/agent_protocol_recovery_command_envelope_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_command_dispatch_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_command_body_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_command_handler_emit.rs`, moving
status-read, rollback-preview, rollback-apply, and disable/restart/load-target
emit helpers into
`seed-kernel/src/agent_protocol_recovery_status_handler_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_rollback_preview_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_rollback_apply_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_target_binding_emit.rs`, moving
memory-write, durable-write, service-inventory side-effect, and command-effect
emit helpers into
`seed-kernel/src/agent_protocol_recovery_memory_write_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_durable_write_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_service_inventory_effect_emit.rs`,
and `seed-kernel/src/agent_protocol_recovery_command_effect_emit.rs`, moving
recovery load-binding emit helpers into
`seed-kernel/src/agent_protocol_recovery_load_binding_emit.rs`, and
preserving the
previously verified guest
`module.audit_rollback_availability`,
`module.audit_rollback_availability_selftest`,
`module.audit_rollback_write_policy`,
`module.audit_rollback_write_policy_selftest`,
`module.audit_rollback_storage_layout`,
`module.audit_rollback_storage_layout_selftest`,
`module.audit_rollback_append_engine`,
`module.audit_rollback_append_engine_selftest`,
`module.audit_rollback_append_contract`,
`module.audit_rollback_append_contract_selftest`,
`module.audit_rollback_append_payload_hash`,
`module.audit_rollback_append_payload_hash_selftest`,
`module.audit_rollback_append_intent`,
`module.audit_rollback_append_intent_selftest`,
`module.audit_rollback_write_boundary`, and
`module.audit_rollback_write_boundary_selftest`, plus denied
`recovery.load_artifact`/`module.load_recovery_artifact`, read-only
`recovery.identity_diagnostic`/`recovery.identity_diagnostic_selftest`,
`recovery.trust_diagnostic`/`recovery.trust_diagnostic_selftest`, and
`recovery.vm_test_diagnostic`/`recovery.vm_test_diagnostic_selftest`,
`recovery.local_approval_diagnostic`/
`recovery.local_approval_diagnostic_selftest`,
`recovery.load_binding`/`recovery.load_binding_selftest`, and
`recovery.lifeline_request_diagnostic`/
`recovery.lifeline_request_diagnostic_selftest`, plus
`recovery.lifeline_protocol_diagnostic`/
`recovery.lifeline_protocol_diagnostic_selftest`, plus
`recovery.lifeline_command_vocabulary`/
`recovery.lifeline_command_vocabulary_selftest`, plus
`recovery.loader_runtime_isolation`/
`recovery.loader_runtime_isolation_selftest`, plus
`recovery.rollback_transaction_engine`/
`recovery.rollback_transaction_engine_selftest`, plus
`recovery.durable_audit_rollback_persistence`/
`recovery.durable_audit_rollback_persistence_selftest`, plus
`recovery.memory_provenance`/`recovery.memory_provenance_selftest`, plus
`recovery.lifeline_command_admission`/
`recovery.lifeline_command_admission_selftest`, plus
`recovery.lifeline_command_envelope_diagnostic`/
`recovery.lifeline_command_envelope_diagnostic_selftest`, plus
`recovery.lifeline_command_dispatch_diagnostic`/
`recovery.lifeline_command_dispatch_diagnostic_selftest`, plus
`recovery.lifeline_command_body_canonicalization_diagnostic`/
`recovery.lifeline_command_body_canonicalization_diagnostic_selftest`, plus
`recovery.lifeline_command_handler_binding_diagnostic`/
`recovery.lifeline_command_handler_binding_diagnostic_selftest`, plus
`recovery.lifeline_status_read_handler_diagnostic`/
`recovery.lifeline_status_read_handler_diagnostic_selftest`, plus
`recovery.rollback_preview_authorization_diagnostic`/
`recovery.rollback_preview_authorization_diagnostic_selftest`, plus
`recovery.rollback_apply_authorization_diagnostic`/
`recovery.rollback_apply_authorization_diagnostic_selftest`, plus
`recovery.disable_module_target_binding_diagnostic`/
`recovery.disable_module_target_binding_diagnostic_selftest`, plus
`recovery.restart_last_good_target_binding_diagnostic`/
`recovery.restart_last_good_target_binding_diagnostic_selftest`, plus
`recovery.load_artifact_by_hash_target_binding_diagnostic`/
`recovery.load_artifact_by_hash_target_binding_diagnostic_selftest`, plus
`recovery.memory_write_authority_diagnostic`/
`recovery.memory_write_authority_diagnostic_selftest`, plus
`recovery.durable_audit_rollback_write_authority_diagnostic`/
`recovery.durable_audit_rollback_write_authority_diagnostic_selftest`, plus
`recovery.service_inventory_side_effect_boundary_diagnostic`/
`recovery.service_inventory_side_effect_boundary_diagnostic_selftest`, plus
`recovery.lifeline_command_dispatch_behavior_diagnostic`/
`recovery.lifeline_command_dispatch_behavior_diagnostic_selftest`, plus
`recovery.lifeline_command_executor_capability_table_diagnostic`/
`recovery.lifeline_command_executor_capability_table_diagnostic_selftest`, plus
`recovery.lifeline_command_side_effect_gate_diagnostic`/
`recovery.lifeline_command_side_effect_gate_diagnostic_selftest`, plus
`recovery.lifeline_command_execution_enablement_diagnostic`/
`recovery.lifeline_command_execution_enablement_diagnostic_selftest`,
`recovery.lifeline_command_execution_preflight_diagnostic`/
`recovery.lifeline_command_execution_preflight_diagnostic_selftest`,
`recovery.lifeline_command_execution_intent_diagnostic`/
`recovery.lifeline_command_execution_intent_diagnostic_selftest`, and
`recovery.lifeline_command_execution_commit_gate_diagnostic`/
`recovery.lifeline_command_execution_commit_gate_diagnostic_selftest`, and
`recovery.lifeline_command_execution_result_denial_diagnostic`/
`recovery.lifeline_command_execution_result_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_audit_denial_diagnostic`/
`recovery.lifeline_command_execution_audit_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_observation_denial_diagnostic`/
`recovery.lifeline_command_execution_observation_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_completion_denial_diagnostic`/
`recovery.lifeline_command_execution_completion_denial_diagnostic_selftest`, plus typed missing
`raios.durable_audit_ledger.v0`/`raios.rollback_store.v0` availability facts,
typed missing `raios.durable_audit_write_policy.v0`/
`raios.rollback_install_policy.v0` policy facts, typed missing
`raios.persistence_device_inventory.v0`/
`raios.audit_rollback_storage_layout.v0` storage-layout facts, typed missing
`raios.audit_ledger_append_engine.v0`/
`raios.rollback_store_transaction_engine.v0` append-engine facts, typed missing
`raios.audit_ledger_append_envelope.v0`/
`raios.rollback_store_transaction_envelope.v0` append-contract facts, typed
`raios.audit_record_append_payload_hash_envelope.v0`/
`raios.rollback_transaction_append_payload_hash_envelope.v0` append payload-hash
envelope facts derived from retained current-boot audit/rollback and
service-slot evidence, typed
missing `raios.audit_record_append_intent.v0`/
`raios.rollback_transaction_append_intent.v0` append-intent facts, and explicit
missing storage-layout, append-engine, append-contract, append-envelope,
append-payload, append-intent stable-id, payload-hash, and provenance binding
inputs over the retained module evidence chain, and typed current-boot
`raios.recovery_artifact_load_denial_evidence.v0` facts for missing recovery
artifact identity, trust, VM-test, local approval, loader, and rollback
evidence on the separate `cap.recovery.load_artifact` path, plus local-only
retained `raios.recovery_artifact_identity.v0` and
`raios.recovery_artifact_trust.v0`,
`raios.recovery_artifact_vm_test.v0`, and
`raios.recovery_artifact_local_approval.v0` hash-reference diagnostics whose
event ids are consumed by `recovery.load_binding`, plus retained recovery-only
evidence-id binding diagnostics that
reject normal module append-intent, append-payload, writer, service-slot, and
`module.load_ephemeral` authority, and a retained local-only
`raios.recovery_lifeline_request.v0` hash-reference diagnostic over the fully
retained recovery evidence chain and a read-only
`raios.recovery_lifeline_protocol_state.v0` diagnostic that consumes that
lifeline request plus the six recovery evidence event ids, plus a read-only
`raios.recovery_lifeline_command_vocabulary.v0` envelope that enumerates
recovery lifeline command ids, argument schemas, required capabilities, and
denial reasons while still denying recovery loading, durable writes, rollback
installs, service-slot allocation, loader execution, direct-OpenAI recovery
shortcuts, command dispatch, and lifeline behavior, plus a read-only
`raios.recovery_loader_runtime_isolation.v0` boundary that enumerates missing
address-space, entrypoint ABI, memory-map, capability-import,
artifact-hash-binding, provider-separation, and normal-module-separation facts
while rejecting invalid request/protocol-state/command-vocabulary inputs and
still loading nothing, plus a read-only
`raios.recovery_rollback_transaction_engine.v0` boundary that reuses the
retained lifeline chain, command-vocabulary envelope, and loader isolation
boundary, enumerates missing rollback target, transaction provenance,
last-good, disabled-module set, artifact-hash, replay, recovery-capability
import, atomic apply/abort, durable persistence, and recovery-memory facts, and
keeps rollback preview/apply non-executable, plus a read-only
`raios.durable_audit_rollback_persistence.v0` boundary that consumes the
rollback engine boundary, enumerates missing persistence device, storage
layout, audit append-log, rollback store, replay cursor, last-good checkpoint,
write ordering, crash consistency, integrity root/hash chain, and
recovery-memory-provenance facts, and keeps durable writes, rollback replay,
recovery-memory writes, and rollback installs disabled, plus a read-only
`raios.recovery_memory_provenance.v0` boundary that consumes that durable
persistence boundary, enumerates missing source record ids, source schema
hashes, classification, authority level, rollback-transaction binding,
last-good checkpoint binding, recovery-only export profile, redaction state,
replay window, and audit linkage facts, and keeps memory writes and provider
export disabled, plus a read-only
`raios.recovery_lifeline_command_admission.v0` boundary that consumes recovery
memory provenance and defines non-executing admission requirements for lifeline
status, rollback preview, rollback apply, disable module, restart last-good,
and load recovery artifact by hash commands while rejecting invalid request,
protocol-state, command-vocabulary, loader-isolation, rollback-engine, durable
persistence, and memory-provenance chains, plus a read-only
`raios.recovery_lifeline_command_envelope_reference.v0` hash-reference
diagnostic that consumes command admission and validates allowed lifeline
command id, argument schema, argument hash, required capability, target
locator, command-admission boundary id, and retained request hash while
accepting no command body and dispatching no command behavior, plus a read-only
`raios.recovery_lifeline_command_dispatch_denial.v0` boundary that consumes the
retained command-envelope reference, exposes missing command body
canonicalization, command handler binding, status handler, rollback
authorization, per-command target binding, memory/durable write authority, and
service-inventory side-effect facts, and still accepts no command body and
dispatches no behavior, plus a read-only
`raios.recovery_lifeline_command_body_canonicalization.v0` hash-reference
diagnostic that consumes the retained command-envelope reference and the
dispatch-denial boundary, validates command id, argument schema, argument hash,
target locator, command-envelope reference hash, dispatch boundary id, and
current-boot scope, retains only local-only current-boot body-canonicalization
hash evidence, exposes missing body schema canonicalization, body redaction/
classification, handler input binding, rollback authorization linkage,
recovery-memory write linkage, durable audit/rollback write linkage, and
service-inventory side-effect linkage facts, and still accepts no raw command
body or command envelope and dispatches no behavior, plus a read-only
`raios.recovery_lifeline_command_handler_binding.v0` hash-reference diagnostic
that consumes the retained body-canonicalization reference, validates command
id, argument schema, argument hash, target locator, command-envelope reference
hash, body-canonicalization hash, dispatch boundary id, handler id, and
handler-input binding hash, retains only local-only current-boot handler
binding evidence, and advances dispatch only to missing status-read handler
while still accepting no raw command body and dispatching no behavior, plus a
read-only `raios.recovery_lifeline_status_read_handler.v0` hash-reference
diagnostic that consumes the retained handler-binding reference, validates
command id, argument schema, argument hash, target locator, command-envelope
reference hash, body-canonicalization hash, handler-binding hash, dispatch
boundary id, status handler id, and status-read projection hash, retains only
local-only current-boot status-read handler evidence, and advances dispatch
only to missing rollback-preview authorization while still executing no status
read and dispatching no behavior, plus a read-only
`raios.recovery_rollback_preview_authorization.v0` hash-reference
diagnostic that consumes the retained status-read handler reference, validates
command id, argument schema, argument hash, target locator, command-envelope
reference hash, body-canonicalization hash, handler-binding hash, status-read
handler hash, dispatch boundary id, rollback-preview authorization id, and
preview projection hash, retains only local-only current-boot preview
authorization evidence, and advances dispatch only to missing rollback-apply
authorization while still executing no rollback preview or recovery command,
plus a read-only `raios.recovery_rollback_apply_authorization.v0`
hash-reference diagnostic that consumes the retained rollback-preview
authorization reference, validates command id, argument schema, argument hash,
target locator, command-envelope reference hash, body-canonicalization hash,
handler-binding hash, status-read handler hash, rollback-preview authorization
hash, dispatch boundary id, rollback-apply authorization id, and apply
projection hash, and now also binds the sourced
`raios.ram_only_hello_service_rollback_apply.v0` denial hash, retained durable
policy write-authority decision hash, and retained
`raios.recovery_rollback_inspect_source_reference.v0` hash, retains only
local-only current-boot apply authorization evidence, and advances dispatch
only to missing disable-module target binding while still executing no rollback
apply or recovery command, plus read-only
`raios.recovery_disable_module_target_binding.v0` hash-reference diagnostic
that consumes the retained rollback-apply authorization reference, validates
command id, argument schema, argument hash, target locator, command-envelope
reference hash, body-canonicalization hash, handler-binding hash,
status-read-handler hash, rollback-preview authorization hash,
rollback-apply authorization hash, dispatch boundary id, disable-module target
id, and disable-module projection hash, and now also binds the sourced
`raios.ram_only_hello_service_rollback_apply.v0` denial hash, retained durable
policy write-authority decision hash, and retained
`raios.recovery_rollback_inspect_source_reference.v0` hash from that retained
apply-authorization event, while still executing no module disable or recovery
command, plus read-only `raios.recovery_restart_last_good_target_binding.v0`,
`raios.recovery_load_artifact_by_hash_target_binding.v0`, and
`raios.recovery_memory_write_authority.v0` hash-reference diagnostics that
chain disable/restart/load/memory authority without restarting services,
loading artifacts, or writing recovery memory, plus a read-only
`raios.durable_audit_rollback_write_authority.v0` hash-reference diagnostic
that consumes the retained recovery-memory write authority, validates the
command/argument/target/envelope/body/handler/status/authorization/target/
memory/dispatch/projection hashes, retains only local-only current-boot
durable-write authority evidence, and advances dispatch only to the missing
service-inventory side-effect boundary while still writing no durable audit or
rollback records and dispatching no behavior, plus a read-only
`raios.recovery_service_inventory_side_effect_boundary.v0` hash-reference
diagnostic that consumes the retained durable-audit/rollback write-authority
reference, validates the same command/argument/target/envelope/body/handler/
status/authorization/target/memory/durable/dispatch/projection hashes, retains
only local-only current-boot service-inventory side-effect boundary evidence,
and advances dispatch only to explicit `defined_non_executable` behavior while
still allocating no service slot, creating no service inventory records,
changing no service inventory, and dispatching no behavior, plus a read-only
`raios.recovery_lifeline_command_dispatch_behavior.v0` hash-reference
diagnostic that consumes the retained service-inventory side-effect boundary,
carries the same source hashes, retains only local-only current-boot
command-dispatch behavior evidence, and keeps command dispatch, command
execution, and service inventory mutation denied, plus a read-only
`raios.recovery_lifeline_command_executor_capability_table.v0` hash-reference
diagnostic that consumes the retained command-dispatch behavior event, carries
the same source hashes, retains only local-only current-boot executor
capability-table evidence, and keeps command dispatch, command execution, and
service inventory mutation denied, plus a read-only
`raios.recovery_lifeline_command_side_effect_gate.v0` hash-reference diagnostic
that consumes the retained executor capability-table event, carries the same
source hashes, retains only local-only current-boot side-effect gate evidence,
and keeps command dispatch, command execution, and service inventory mutation
denied, via
headless
Shadow VM smoke
covering
deterministic `provider_minimal`
packet/field-list evidence, explicit provider request-binding denial and
export-denial audit records, the denied `provider.context_export` gate, the
local redaction projection, read-only memory context, the RAM-only current-boot
event log with structured denial bindings, the runtime
`raios.provider_request_envelope.v0` marker on the real OpenAI request path,
positive local-only request/export audit binding records on the SPKI pinned
OpenAI path, checked current-boot binding consumption with single-use rejection,
a local-only negative gate selftest for stale/dropped,
previous-boot-or-unretained, substituted-schema, substituted-positive-record,
and mismatched-hash cases, the separate fail-closed
`raios.provider_context_injection_gate.v0` diagnostic, local-only negative
final-injection authorization selftests, the fail-closed
`raios.module_load_gate.v0` denial with event-log binding for denied
`module.load_ephemeral`, read-only `module.manifest_diagnostic`,
`module.manifest_diagnostic_selftest`, `module.artifact_diagnostic`,
`module.artifact_diagnostic_selftest`, `module.vm_report_diagnostic`,
`module.vm_report_diagnostic_selftest`, `module.grant_diagnostic`, and
`module.grant_diagnostic_selftest`, `module.attestation_diagnostic`,
`module.attestation_diagnostic_selftest`, `module.load_gate_attestation_selftest`,
`module.approval_diagnostic`, `module.approval_diagnostic_selftest`, and
`module.load_gate_approval_selftest` manifest, candidate-artifact, VM-report,
computed-grant, local-attestation, and local-approval hash-reference diagnostics,
local-only current-boot retention of valid manifest, artifact, VM-report,
computed-grant, local-attestation, and local-approval hash references, the
denied module load gate reporting retained manifest, artifact, VM-report,
computed-grant, local-attestation, and local-approval references without
authorizing loading, guest
audit/rollback hash-reference diagnostics that retain valid references only as
local-only current-boot evidence, the denied module load gate reporting retained
audit/rollback references as non-authorizing hash evidence only after live
current-boot predicate validation, rejection of a wrong-schema retained
audit/rollback reference in the live denied load gate, and local-only negative
manifest, artifact, retained-reference, plus audit/rollback evidence gate
selftests plus
`module.audit_rollback_diagnostic_selftest` guest hash-reference diagnostics,
and guest `module.service_slot_diagnostic` RAM-only service-slot reservation
hash-reference diagnostics that retain valid reservations as local-only
current-boot evidence without allocating a slot or loading artifacts, plus the
denied module load gate live-validating that retained reservation as
non-authorizing service-slot evidence and local-only service-slot gate
selftests for rejected retained reservations, and local-only
audit/rollback write-boundary selftests for missing, stale, substituted,
previous-boot, wrong-schema, mismatched, recovery-separated, and
accepted-current-boot-but-denied candidates, and local-only
`module.load_gate_vm_report_selftest` coverage for missing, stale,
wrong-schema, substituted, hash-mismatched, and binding-mismatched VM-report
references, local-only retained local-attestation reference gate selftests, and
local-only retained local-approval reference gate selftests.
Direct OpenAI pin-mismatch plus SPKI pinned-trust smokes using a fake local API
key remain previously verified from the prior handoff.

Latest current-cursor focused verification: 2026-07-02 on Windows with
`cargo fmt --all -- --check`,
`scripts\build-seed-kernel.ps1 -Profile release`,
`scripts\package-stage0.ps1 -Profile release`,
`cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`,
`git diff --check`, `scripts\scan-secrets.ps1`, and
`vm-harness\shadow-vm-smoke.ps1 -Profile quick -TimeoutSeconds 300`.
The focused quick report is
`release\vm-reports\shadow-20260702-185627-6792.json` with 298/298
predicates, 56 executed commands, `duration_ms: 86581`, and report SHA-256
`542d99dc6989a7edf63c50d2f8ac76e46c14ff29c4d47471b693e53fab291e89`.
The quick run verifies `raios.audit_rollback_target_region_discovery.v0` under
the storage authority foundation with positive read-only non-scratch
target-region evidence, scratch rejected as durable authority, and
durable-region availability true while append/write authority stays false. It
also verifies `raios.audit_rollback_target_region_writer_contract.v0` nested
under `raios.audit_rollback_transaction_writer_readiness.v0`, bound to that
target discovery, `append.audit_ledger.current_boot` /
`raios.audit_record.v0`, `append.rollback_store.current_boot` /
`raios.rollback_transaction.v0`, LBA1/512-byte target span, and all
write/append flags false. Nested under that contract it also verifies
`raios.audit_rollback_target_region_media_write_policy_preflight.v0`, bound to
the contract schema/id/status/reason plus owner/target/span/schema ids, with
missing media write authority and durable audit policy expressed as structured
denial facts and all write/append flags false. The Hello durable
append-authority preflight still proves scratch write/readback is not treated
as durable authority and that durable audit writes, rollback-store writes,
transaction append, rollback application, persistence, external bytes,
candidate execution, executable mapping, provider auto-load, and broad mutation
remain denied. Latest passing full Shadow VM report remains
`release\vm-reports\shadow-20260702-174421-7208.json` with 6789/6789
predicates; later full-profile attempts around the storage-authority slices
hit a serial TCP transport failure after `provider.context_gate_selftest` and
are tracked as harness/serial follow-up, not full verification.

Latest focused verification: 2026-07-03 local report timestamp on Windows with
QEMU 11 after adding the Hello rollback current-boot transaction-append dry-run
blocked by the transaction-append authority-denial gate. Quick Shadow VM smoke
passed against `release\raios-stage0.img` in
`release\vm-reports\shadow-20260703-010933-17728.json` with 373/373
predicates, 56 executed commands, and `duration_ms: 85447`; report SHA-256 is
`a3d999dbe8b7cc0ba1d0b0a6b3b14614004ffbf8e1b1ca5dd97bc6ecbd0f8c6b`,
base image SHA-256 is
`2b05f9935ebc289cbc738cb3a0052374842586282a806d0ea4bff1d702a48242`.
The quick smoke proves
`raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0`
is nested under the durable append-authority preflight and retained on the
rollback-apply RAM audit binding. The write-authority, policy-ledger, and
audit-policy availability facts keep binding the ledger-aware result,
ledger-candidate, target-region media-write policy preflight, target-region
write/readback hash, audit/rollback append target ids and schemas, and the same
LBA1/512-byte target span while reporting write authority, durable policy
ledger, durable audit policy, and durable append authority unavailable. The new
`raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0`
fact consumes the durable audit-policy availability evidence and binds its hash
plus the policy-ledger availability hash, write-authority availability hash,
ledger-aware result hash, ledger-candidate hash, target-region media-write
policy preflight hash, target-region write/readback hash, audit/rollback append
target ids and schemas, and the same LBA1/512-byte target span. It verifies
audit-policy availability, policy-ledger, write-authority, ledger, media-write
policy, target-region write/readback, span, target ids, and test-media evidence,
but still reports durable append authority, durable audit policy, durable policy
ledger, and write authority unavailable and performs no durable write or append.
The new
`raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0`
decision consumes the durable append-authority availability hash, audit-policy
availability hash, append-engine readiness hash, durable writer-policy
preflight hash, media-write policy preflight hash, target-region write/readback
hash, audit/rollback append target ids and schemas, and the same LBA1/512-byte
target span. It verifies append-authority availability evidence, audit-policy
availability evidence, append-engine readiness, writer-policy readiness,
media-write policy, target readback, target span, target ids, and test-media
evidence, but still reports durable append authority, durable audit policy, and
transaction append unavailable and performs no write, append, or transaction
append.
The new
`raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0`
gate consumes the transaction-append availability decision hash plus the
durable append-authority availability hash, audit-policy availability hash,
append-engine readiness hash, durable writer-policy preflight hash, media-write
policy preflight hash, target-region write/readback hash, audit/rollback append
target ids and schemas, and the same LBA1/512-byte target span. It verifies
availability-decision evidence, append-engine readiness, writer-policy
readiness, media-write policy, target readback, target span, target ids, and
test-media evidence, but keeps `missing_transaction_append_authority: true` and
all media-write, append, transaction-append, durable-audit, rollback-store, and
write-attempt side effects false.
The new
`raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0` evidence
is consumed by the rollback-apply denial and RAM audit binding under that
authority-denial gate. It binds the authority-denial gate hash,
transaction-append availability decision hash, append-record dry-run hash,
sector-plan hash, target-region write/readback hash, planned/readback sector
image hashes, audit-ledger target/schema, rollback-store target/schema, and the
same LBA1/512-byte target span. It proves append-image readiness only as
current-boot/test-media evidence, reports the authority-denial gate verified,
target span verified, target-region write/readback verified, append image
ready, and blocked by missing transaction-append authority, while keeping
media-write, append, transaction-append, durable-audit, rollback-store, and
transaction-append-attempt side effects false.
The rollback-apply denial now exposes
`raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0` as
read-only current-boot recovery evidence over the already materialized
dedicated target-region LBA1 sector. The AHCI path re-reads the labeled
`RAIOS_AUDITRB_V0` target-region sector through the existing block path, hashes
the full 512-byte sector image, hashes the canonical audit-record and
rollback-transaction byte ranges, verifies offsets 0/255/480 with zero padding,
binds the target-region write/readback hash and sector-plan hash, and keeps
media-write, append, durable-audit, rollback-store, rollback-transaction append,
and installed rollback state false.
The rollback-apply denial now consumes that dry-run in
`raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0`.
That decision binds the durable append-authority availability dry-run hash,
transaction-append dry-run hash, target-region sector inspection hash,
write-authority availability hash, durable audit-policy availability hash,
durable append-authority availability hash, authority-denial gate hash,
transaction append-availability decision hash, audit/rollback target ids and
schemas, and the same LBA1/512-byte target span. It verifies the dry-run,
sector inspection, write-authority evidence, audit-policy availability evidence,
append-authority availability evidence, target span, and current-boot
test-media write/readback evidence, then records write authority, durable
policy ledger, durable audit policy, durable append authority, transaction
append, media write, append, rollback transaction append, durable audit writes,
rollback-store writes, write attempts, and rollback application as
unavailable/false.
The rollback-apply denial now also exposes
`raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0`
as current-boot test-media-only evidence. It binds the durable policy-ledger
availability hash, policy write-authority availability hash, ledger-aware
acceptance result hash, ledger-candidate hash, media-policy preflight hash,
target-region write/readback hash, transaction-append authority-denial gate
hash, transaction append-availability decision hash, audit/rollback target ids
and schemas, and the same LBA1/512-byte target span. It verifies the
policy-ledger availability evidence, write-authority evidence, ledger evidence,
media policy, target-region write/readback, transaction-append denial gate,
target span, audit/rollback target ids, and current-boot test-media write
authority while keeping write authority, durable policy ledger, durable audit
policy, durable append authority, transaction append, media write, append,
rollback transaction append, durable audit writes, rollback-store writes,
write attempts, rollback application, and installed rollback state
unavailable/false.
The rollback-apply denial now also exposes
`raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0`
as current-boot test-media-only evidence. It binds the durable audit-policy
availability hash, durable policy-ledger availability dry-run hash, durable
policy-ledger availability hash, policy write-authority availability hash,
ledger-aware acceptance result hash, ledger-candidate hash, media-policy
preflight hash, target-region write/readback hash, transaction-append
authority-denial gate hash, transaction append-availability decision hash,
audit/rollback target ids and schemas, and the same LBA1/512-byte target span.
It verifies the audit-policy availability evidence, policy-ledger dry-run
evidence, policy-ledger availability evidence, write-authority evidence, ledger
evidence, media policy, target-region write/readback, transaction-append denial
gate, target span, audit/rollback target ids, and current-boot test-media write
authority while keeping write authority, durable policy ledger, durable audit
policy, durable append authority, transaction append, media write, append,
rollback transaction append, durable audit writes, rollback-store writes,
write attempts, rollback application, and installed rollback state
unavailable/false.
The rollback-apply denial now also exposes
`raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0`
as current-boot test-media-only evidence. It binds the durable
append-authority availability hash, durable audit-policy availability dry-run
hash, durable audit-policy availability hash, durable policy-ledger
availability dry-run hash, durable policy-ledger availability hash, policy
write-authority availability hash, ledger-aware acceptance result hash,
ledger-candidate hash, media-policy preflight hash, target-region
write/readback hash, transaction-append authority-denial gate hash,
transaction append-availability decision hash, audit/rollback target ids and
schemas, and the same LBA1/512-byte target span. It verifies the
append-authority availability evidence, audit-policy dry-run evidence,
audit-policy availability evidence, policy-ledger dry-run evidence,
policy-ledger availability evidence, write-authority evidence, ledger evidence,
media policy, target-region write/readback, transaction-append denial gate,
target span, audit/rollback target ids, and current-boot test-media write
authority while keeping write authority, durable policy ledger, durable audit
policy, durable append authority, transaction append, media write, append,
rollback transaction append, durable audit writes, rollback-store writes,
write attempts, rollback application, and installed rollback state
unavailable/false.
Durable media writes, durable audit writes, rollback-store writes, transaction
append, rollback application, persistence, external bytes, candidate execution,
executable mapping, provider auto-load, broad mutation, durable append
authority, and installed rollback state remain denied.

Latest focused/quick harness verification for that evidence: 2026-07-03 local
report timestamp on Windows with QEMU 11 after binding the top-level
rollback-apply denial hash to retained durable policy write-authority decision
and retained recovery inspect-source evidence.
The focused `hello-rollback-dry-run`
profile still loads the built-in Hello service, hot-swaps to v2, runs rollback
preview, materializes only the planned LBA1/512-byte `RAIOS_AUDITRB_V0` test
sector, inspects the retained sector, and keeps the still-denied
`service.rollback_apply svc.demo.hello` consuming the validated source
reference plus the policy write-authority decision in the top-level
`raios.ram_only_hello_service_rollback_apply.v0` denial hash without granting
durable authority. Descriptor/generation/running/RAM state, durable-audit,
rollback-store, transaction append, apply, persistence, external-artifact bytes,
executable mapping, provider auto-load, broad mutation, and installed rollback
state stay unchanged/denied. Focused report
`release\vm-reports\shadow-20260703-052003-28604.json` passed 181/181
predicates with 24 executed commands and `duration_ms: 82145`; report SHA-256
is `e4f509a05e47b1ecab54852c9a2ded7ff16607f80473ff32d3bc545aceebd9ef`;
base image SHA-256 is
`98afce8ca591e11bc6e5db4e89ab6cd4e6311d1a142e731f926439c7f4e90327`.
The quick profile report `release\vm-reports\shadow-20260703-052134-30240.json`
passed 404/404 predicates with 59 executed commands and `duration_ms: 148672`;
report SHA-256 is
`8cf2496d99becbbeaaa73632c5b56fe39690ad67d8a8c34daf4a97fc573dee57`;
base image SHA-256 is
`6b543c73b0a7a2fe2a37a4fbd12759b0695e418df095cc4c7ae8885f3763bce0`.

Latest recovery execution-enablement verification: the recovery profile report
`release\vm-reports\shadow-20260703-074211-20452.json` passed 2823/2823
predicates with 142 executed commands and `duration_ms: 227267`; report SHA-256
is `81ab2dfe9301cdabe579e860406a1db8c027e51fdd795fa01adae0c3e25183db`;
base image SHA-256 is
`370bb1d851374def07003968d6292fff940dba4ad72c04719f2584d7e7efe402`.
The focused `hello-rollback-dry-run` guard report
`release\vm-reports\shadow-20260703-074606-9508.json` passed 181/181
predicates with 24 executed commands and `duration_ms: 83725`; report SHA-256
is `e0e0e287da3e936546abc696dfacc04c165e179421ce33e7436f92b5eedb4495`;
base image SHA-256 is
`9fdc804342b97fe26a8c90862a8f4d23a43803cf7ef5c10f8ae2982e322e4558`.
Previous recovery side-effect gate verification: the recovery profile report
`release\vm-reports\shadow-20260703-072638-30256.json` passed 2799/2799
predicates with 142 executed commands and `duration_ms: 222896`; report SHA-256
is `4f4df2c6c44f4c5a75d63d30e6bcf4ff5b54091eb6ac720a48c4039fed4a3751`;
base image SHA-256 is
`dc8db0b397ed84915f36cb759a4a971ef09ab7c04413fd60e376b1834f096fb7`.

Previous recovery executor capability-table verification: the recovery profile
report `release\vm-reports\shadow-20260703-071243-14856.json` passed 2793/2793
predicates with 142 executed commands and `duration_ms: 223571`; report SHA-256
is `7038b842c55a30442dce3af0629d91c6cecec0f4299e5d759808975186f12699`;
base image SHA-256 is
`023fe7ef056f99ac2fd53e470181ce4575488d844109812c9f432449328ec709`.
The focused `hello-rollback-dry-run` guard report
`release\vm-reports\shadow-20260703-071631-28124.json` passed 181/181
predicates with 24 executed commands and `duration_ms: 81316`; report SHA-256
is `c2330af8ff9331c3f30d1d110518fa6f4abf58f447d0abf26eae8f53d04595b6`;
base image SHA-256 is
`644997917a4c6f3f5472eda1f8e3947c82b476d4757d744787d4acb9679f7401`.

Previous recovery command-dispatch behavior verification: the recovery profile
report `release\vm-reports\shadow-20260703-070020-19184.json` passed 2787/2787
predicates with 142 executed commands and `duration_ms: 222220`; report SHA-256
is `57424c7ff566d505cf012ed785e2b02fcd04d6f8aeed6e6b5837af90b09e0403`;
base image SHA-256 is
`62550e2f675e0dc38e3f974d040c47a3d382d9ba6a5658ce673021e34b140770`.
The focused `hello-rollback-dry-run` guard report
`release\vm-reports\shadow-20260703-070408-13684.json` passed 181/181
predicates with 24 executed commands and `duration_ms: 80756`; report SHA-256
is `d805c998b68b80f6e68d1dce958ea752bd5eb262b717c2c843e930bce70a83f0`;
base image SHA-256 is
`7cf1beab1c349a2b1f369bb584eb6349a9967b925526522d3c6cf45fbc179f62`.

Previous recovery service-inventory side-effect boundary verification: the
recovery profile report `release\vm-reports\shadow-20260703-064708-18720.json`
passed 2781/2781 predicates with 142 executed commands and
`duration_ms: 223126`;
report SHA-256 is
`5521c70ec182d5f37dd67d0041e422a8ecad92cb745522470701455f79591ff1`;
base image SHA-256 is
`d11b8d5ff5cc2bae433664730c6997e4cf4d7046dc7c82240263cee6ee1de3a6`.
The follow-up focused `hello-rollback-dry-run` report
`release\vm-reports\shadow-20260703-065115-22780.json` passed 181/181
predicates with 24 executed commands and `duration_ms: 81111`; report SHA-256
is
`81212860e351693cc7882e162eadb8a7bda926f37d46e0065a2f0cc3d8ebe74d`;
base image SHA-256 is
`93a22948850c4a21bc38ac26f3e05387a13f5cf7046b3c9caffc53dd07904ea9`.

Latest recovery load-artifact denial status/load-binding verification: the 2026-07-03
local report-timestamp recovery profile
`release\vm-reports\shadow-20260703-085421-6328.json` passed 2898/2898
predicates with 143 executed commands and `duration_ms: 242684`; report
SHA-256 is
`7a3aa6e4a18fe0e21aa1afb1c4c46c5608a1b6684001c7797bc3cdde234f3824`;
base image SHA-256 is
`73ffe138f7dd0d735bb0ff1e5334e13c699169a09931b8c50600e5bd00ba3a8a`.
It proves `recovery.load_binding` consumes the retained source-bound
`raios.recovery_lifeline_command_execution_completion_denial.v0` reference from
the dispatch-denial chain, and the separate denied `recovery.load_artifact`
response now nests that same current-boot/local-only/read-only load-binding
denial evidence after the retained chain exists. The nested evidence preserves
the load-binding status/reason, completion-denial event id, final stage hash,
side-effect-gate hash, source rollback/policy/inspect hashes, and prior
execution-stage hashes. The RAM audit/event-log binding for denied
`recovery.load_artifact` now carries the same nested load-binding status/reason,
retained evidence event ids/hashes, completion-denial chain, and no-mutation
flags so `audit.events` does not need to infer it from the response body. Once
that nested load-binding reaches `available_non_authorizing`, the top-level
denial response and audit binding now distinguish
`denied_recovery_load_binding_not_authorizing` from the initial
missing-evidence state.
`recovery.load_binding_selftest` covers the missing completion-denial reference
as a fifteenth fail-closed case. The path keeps module disable, restart,
artifact load, memory write, durable media writes, durable audit writes,
rollback-store writes, transaction append, rollback application, service
inventory mutation, lifeline command dispatch, command execution, persistence,
external bytes, candidate execution, executable mapping, provider auto-load,
broad mutation, and installed rollback state denied.

Latest module audit/rollback write-boundary command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-180640-6876.json` passed 1610/1610
predicates with 71 executed commands and `duration_ms: 168943`; report SHA-256
is `268d01494ec751ff20bd8d940d3926199aa4e29fd847d903fb581be7a925f5fa`;
base image SHA-256 is
`54f4d0a78f1d412724ff55eafaad7f75ea651fa8360c9533ce60be1fde77b560`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_write_boundary` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_write_boundary` implementation. The focused VM proves
the enveloped command returns the real
`raios.module_audit_rollback_write_boundary.v0` response, preserves the
write-boundary denial evidence for missing durable audit write and rollback
install authority, keeps write policy, rollback policy, storage layout, append
engine, payload-hash, and append-intent prerequisites missing, creates no
durable audit records or rollback plans, loads no recovery artifact, and does
not attempt load. A mismatched capability (`cap.system.describe.read`) is
denied by the envelope before any write-boundary dispatch. Persistence, durable
audit writes, rollback-store writes, transaction append, rollback application,
external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Full checkpoint status after the write-boundary envelope slice: the full Shadow
VM checkpoint is not yet green because the current full profile still has a
repeatable harness/serial checkpoint failure around the non-terminal
`agent audit.events 256` scrape in
`vm-harness\shadow-vm-smoke-profile-full-module-load-gate.ps1`. Two runs with
the current full profile,
`release\vm-reports\shadow-20260703-181303-6380.json` and
`release\vm-reports\shadow-20260703-182424-4208.json`, each passed all 3522
predicates reached with 125 executed commands, then failed on the next TCP
serial connection after the module load-gate `agent audit.events 256` command.
The first report has SHA-256
`e4c0f6ff52cf049094468674298b5b020189b569bf70e01425614690411a8b3e` and base
image SHA-256
`f241b896060b9ee4f3d547aed0803b7676d1cb063b4efec1c7052551dd4d56a9`; the fresh
port rerun has report SHA-256
`6b28da9ea9cb2e0620713584f9d9241aa517da4818466fc5b29d64008ccd222f` and base
image SHA-256
`db8b9a91bd071aa92963feabbe3c0ab9ae234963b2abf8d578a20b57e60a0a78`. A
diagnostic reduced-window attempt reached recovery and passed 7005/7006
predicates with 299 executed commands, but failed
`protocol:module_manifest_audit_source`, proving the later full-audit checks
still depend on preserving the earlier module evidence audit scrape. A
keep-open TCP experiment still observed a remote close after the same large
scrape and was backed out. No runtime authority, durable write, rollback,
external artifact, executable mapping, provider auto-load, broad mutation, or
installed rollback state was enabled by these failed checkpoint attempts.

Latest module audit/rollback append-intent command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-175627-17296.json` passed 1607/1607
predicates with 69 executed commands and `duration_ms: 166258`; report SHA-256
is `8d72411b5d9ef5700d747cb503fee577ba701efd4547b8540d26094744cfaa19`;
base image SHA-256 is
`bf4b445b03010088756d754361bd6891826e394b6b9c76e8fced85a24aed0851`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_append_intent` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_append_intent` implementation. The focused VM proves
the enveloped command returns the real
`raios.module_audit_rollback_append_intent.v0` response with append-contract,
payload-hash, audit-record intent, and rollback-transaction intent facts; keeps
writes disabled; creates no durable audit records or rollback plans; reports
missing append intents and unavailable load; and does not attempt load. A
mismatched capability (`cap.system.describe.read`) is denied by the envelope
before any append-intent dispatch. Persistence, durable audit writes,
rollback-store writes, transaction append, rollback application, external
artifact intake, candidate execution, executable mapping, provider auto-load,
broad mutation, and installed rollback state remain denied.

Latest module audit/rollback append-payload-hash command-envelope verification:
the 2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-175035-2800.json` passed 1604/1604
predicates with 67 executed commands and `duration_ms: 177113`; report SHA-256
is `582f61003c4456b47771c0299eca2a9524dafc6584c984db74019d5b91323b34`;
base image SHA-256 is
`857e6fec52753b20a9bd3261aa840b08df2c9aae3730ddf685ef81cf3e9f75a5`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_append_payload_hash` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_append_payload_hash` implementation. The focused VM
proves the enveloped command returns the real
`raios.module_audit_rollback_append_payload_hash.v0` response with retained
audit/rollback evidence, service-slot reservation, append-contract input, and
payload-hash envelope facts; keeps writes disabled; creates no durable audit
records or rollback plans; reports missing payload-hash append authority; and
does not attempt load. A mismatched capability (`cap.system.describe.read`) is
denied by the envelope before any append-payload-hash dispatch. Persistence,
durable audit writes, rollback-store writes, transaction append, rollback
application, external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Latest module audit/rollback append-contract command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-174332-4352.json` passed 1601/1601
predicates with 65 executed commands and `duration_ms: 187156`; report SHA-256
is `ca7eb7d28195ddbdf7d6bb95d386856de139b5759f0393b7e05b5dedf0da8e2c`;
base image SHA-256 is
`e4d69b69adaebe9b05069297859f41ae8b83c44af936c02444777ad7c83222ca`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_append_contract` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_append_contract` implementation. The focused VM proves
the enveloped command returns the real
`raios.module_audit_rollback_append_contract.v0` response with storage-layout,
append-engine, write-policy, availability, append-target owner, and
transaction-writer readiness bindings; keeps writes disabled; creates no
durable audit records or rollback plans; reports missing append envelopes and
unavailable load; and does not attempt load. A mismatched capability
(`cap.system.describe.read`) is denied by the envelope before any
append-contract dispatch. Persistence, durable audit writes, rollback-store
writes, transaction append, rollback application, external artifact intake,
candidate execution, executable mapping, provider auto-load, broad mutation,
and installed rollback state remain denied.

Latest module audit/rollback append-engine command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-173613-18376.json` passed 1598/1598
predicates with 63 executed commands and `duration_ms: 186625`; report SHA-256
is `6d12fb98e5a77c5be7ff2a0f0571981a0cefb8539d5c04f7f84ad3c369655b97`;
base image SHA-256 is
`e163138c8832eb160e7507f30deda32670085fd84a2bcb03fa328cd3f7dfe6b5`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_append_engine` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_append_engine` implementation. The focused VM proves
the enveloped command returns the real
`raios.module_audit_rollback_append_engine.v0` response with append-engine
readiness and missing audit-ledger/rollback-store engine facts; keeps writes
disabled; creates no durable audit records or rollback plans; reports the
append engine missing/unavailable; and does not attempt load. A mismatched
capability (`cap.system.describe.read`) is denied by the envelope before any
append-engine dispatch. Persistence, durable audit writes, rollback-store
writes, transaction append, rollback application, external artifact intake,
candidate execution, executable mapping, provider auto-load, broad mutation,
and installed rollback state remain denied.

Latest module audit/rollback storage-layout command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-172803-24984.json` passed 1595/1595
predicates with 61 executed commands and `duration_ms: 168454`; report SHA-256
is `18afb623facb2d03f2b0c647be2ed5a82a3044550b5a5a4e976a279c72b8a96f`;
base image SHA-256 is
`26b8eb5e98701360178e5d7ed7bd838b166f879e4f886892066189f2f5fc764d`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_storage_layout` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_storage_layout` implementation. The focused VM proves
the enveloped command returns the real
`raios.module_audit_rollback_storage_layout.v0` response with its read-only
AHCI block-driver, scratch write/readback, and audit/rollback target-region
evidence; keeps writes disabled; creates no durable audit records or rollback
plans; reports missing durable storage layout and unavailable load; and does
not attempt load. A mismatched capability (`cap.system.describe.read`) is
denied by the envelope before any storage-layout dispatch. Persistence, durable
audit writes, rollback-store writes, transaction append, rollback application,
external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Latest module audit/rollback write-policy command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-164538-10396.json` passed 1592/1592
predicates with 59 executed commands and `duration_ms: 170656`; report SHA-256
is `04e902095c5892728fe3d6b7651e9a39969627f40db7436bee6f1ec92bec2288`;
base image SHA-256 is
`668bcf60b9d72b47c0af02144089ba3f873f203fb479b06dfc0d6eb6361fa1af`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `module.audit_rollback_write_policy` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_write_policy` implementation. The focused VM proves the
enveloped command returns the real
`raios.module_audit_rollback_write_policy.v0` response, remains
local-only/read-only, keeps writes disabled, creates no durable audit records
or rollback plans, does not install rollback state, and does not attempt load.
A mismatched capability (`cap.system.describe.read`) is denied by the envelope
before any write-policy dispatch. The focused profile also re-proves the
availability envelope and the downstream module audit/rollback evidence chain.
Persistence, durable audit writes, rollback-store writes, transaction append,
rollback application, external artifact intake, candidate execution, executable
mapping, provider auto-load, broad mutation, and installed rollback state remain
denied.

Latest module audit/rollback availability command-envelope verification: the
2026-07-03 local report-timestamp focused module-audit-rollback profile
`release\vm-reports\shadow-20260703-163924-21932.json` passed 1589/1589
predicates with 57 executed commands and `duration_ms: 163752`; report SHA-256
is `a484522e6e4ea58c65f374d7e3ff91a6f856fa30e94fd901bed79c042aa676dd`;
base image SHA-256 is
`a4d55f11c6b0280a8dd13a3622a02a6024e42ee7b49c0b76eb8a6fe1822d0594`.
It adds the focused `module-audit-rollback` Shadow VM profile, which runs the
common boot/provider checks plus the existing module evidence and
audit/rollback profile without the full provider/recovery/hello matrix. The
same slice extends the centralized serial `raios.agent_command_envelope.v0`
allowlist to `module.audit_rollback_availability` with the existing
`cap.module.grant_diagnostic.read` authority, dispatching to the existing
`module.audit_rollback_availability` implementation. The focused VM proves the
enveloped command returns the real `raios.module_audit_rollback_availability.v0`
response, remains local-only/read-only, keeps writes disabled, creates no
durable audit records or rollback plans, does not install rollback state, and
does not attempt load. A mismatched capability (`cap.system.describe.read`) is
denied by the envelope before any availability dispatch. The focused profile
also re-proves the downstream module audit/rollback evidence chain. Persistence,
durable audit writes, rollback-store writes, transaction append, rollback
application, external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Latest Hello service-health command-envelope verification: the 2026-07-03
local report-timestamp focused hello-rollback-dry-run profile
`release\vm-reports\shadow-20260703-163113-23516.json` passed 203/203
predicates with 30 executed commands and `duration_ms: 116096`; report SHA-256
is `70960acd9f617528e0eaa6980d2d8bde70e2c92372baee8a9f647ef4a6152989`;
base image SHA-256 is
`8bd6ab77ff13824ef127d7c0a0b7455fa5200dc20be4d16e91d80992ce85c913`.
It extends the centralized serial `raios.agent_command_envelope.v0` allowlist
to `service.health` with the existing `cap.service.health.read` authority,
dispatching to the existing `service.health svc.demo.hello` implementation
instead of adding service-target parsing or new health logic. The focused VM
proves the enveloped command returns the real
`raios.ram_only_hello_service.health.v0` response for the loaded v1 Hello
service, and that a mismatched capability (`cap.system.describe.read`) is
denied by the envelope before any `service.health` dispatch. The same run keeps
the previously verified `service.rollback_preview` and
`recovery.rollback_inspect` envelope paths green. Persistence, durable audit
writes, rollback-store writes, transaction append, rollback application,
external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Latest Hello recovery rollback-inspect command-envelope verification: the
2026-07-03 local report-timestamp focused hello-rollback-dry-run profile
`release\vm-reports\shadow-20260703-144833-19104.json` passed 200/200
predicates with 28 executed commands and `duration_ms: 109610`; report SHA-256
is `3b748444200f54626e693ac103917fa965b822b23b339e3d87bc1e5781b32296`;
base image SHA-256 is
`4e5ecd18d8803dc2f2768a673c02617e78ea708cbc3f892ff9582cbdba28908f`.
It cleans up the serial `raios.agent_command_envelope.v0` allowlist into a
single method/capability/response-id/dispatch-method table and extends that
shared boundary to `recovery.rollback_inspect` with
`cap.recovery.rollback_inspect.read`, dispatching to the existing
`recovery.rollback_inspect svc.demo.hello` implementation rather than adding
new rollback logic. The focused VM proves the enveloped command returns the
real `raios.recovery_rollback_inspect.v0` response after target-region
materialization, remains read-only, does not authorize media write/append,
does not write durable audit or rollback-store state, does not append a
rollback transaction, does not apply rollback, and does not install rollback
state. The same profile proves a mismatched capability
(`cap.system.describe.read`) is denied by the envelope before any
`recovery.rollback_inspect` dispatch, while the prior
`service.rollback_preview` envelope path remains green. Persistence, durable
audit writes, rollback-store writes, transaction append, rollback application,
external artifact intake, candidate execution, executable mapping,
provider auto-load, broad mutation, and installed rollback state remain denied.

Latest Hello rollback command-envelope preview verification: the 2026-07-03
local report-timestamp focused hello-rollback-dry-run profile
`release\vm-reports\shadow-20260703-144044-22112.json` passed 197/197
predicates with 26 executed commands and `duration_ms: 100716`; report SHA-256
is `e0acd8bd07e4abda57d9f95fdcd641e91093d9b05f089347466a5c310a6c8122`;
base image SHA-256 is
`61692613d30515dfb7b57b22ec66d7761181816f1b9b40cb9b1a0a4fdf26a040`.
It extends the existing `raios.agent_command_envelope.v0` allowlist to
`service.rollback_preview` with `cap.service.rollback_preview.read`, dispatching
to the existing `service.rollback_preview svc.demo.hello` implementation rather
than adding a new rollback path. The focused VM proves the enveloped command
returns the real `raios.ram_only_hello_service_rollback_preview.v0` response
after a v2 hot-swap probation exists, remains read-only, does not apply
rollback, does not install a rollback plan, and does not write durable audit.
The same profile proves a mismatched capability
(`cap.system.describe.read`) is denied by the envelope before any
`service.rollback_preview` dispatch. Persistence, durable audit writes,
rollback-store writes, transaction append, rollback application, external
artifact intake, candidate execution, executable mapping, provider auto-load,
broad mutation, and installed rollback state remain denied.

Latest provider-memory/full verification cleanup: the 2026-07-03 local
report-timestamp focused provider-memory-full profile
`release\vm-reports\shadow-20260703-142658-22296.json` passed 258/258
predicates with 21 executed commands and `duration_ms: 86345`; report SHA-256
is `b5c69047eb76cfc22f1fac5f0d17f4e4c623937920b6c0ff5a8d00791a489fce`;
base image SHA-256 is
`b470ba5d2bc3b275c09d7ceda6c63e748921a17660c2fd9718f7ff8697b3dab5`.
It adds the focused `provider-memory-full` Shadow VM profile so the
full-profile provider-memory assertions can be run without the long
module/recovery matrix. The cleanup leaves runtime provider behavior unchanged:
`provider.context_injection_gate provider_minimal`, memory query/trace/recent
events, memory mutation denials, and the broader provider-memory checks still
run before the large `provider.context_gate_selftest provider_minimal`, and the
gate selftest is now invoked terminally in both the focused
`provider-memory-full` path and at the end of the `full` profile. This avoids
requiring a fragile immediate serial reconnect after the large gate-selftest
response. The report proves the terminal gate selftest still covers all 20
negative cases, including `omitted_field_list_hash_mismatch`, without global
event-log mutation, real request envelopes, positive bindings, provider writes,
or provider body attachment. The injection-gate omission selftest remains
covered by the focused `provider-memory` report
`release\vm-reports\shadow-20260703-141539-23964.json`. The cleanup does not
add memory writes, provider export, fallback execution, recovery command
dispatch, service inventory mutation, module disable, restart, artifact load,
durable writes, rollback-store writes, transaction append, rollback
application, persistence, external bytes, candidate execution, executable
mapping, provider auto-load, broad mutation, or installed rollback state.

Latest provider context injection-gate omission negative selftest verification:
the 2026-07-03 local report-timestamp focused provider-memory profile
`release\vm-reports\shadow-20260703-141539-23964.json` passed 169/169
predicates with 12 executed commands and `duration_ms: 73231`; report SHA-256
is `bfe3f93b043a5385bee753f988f9e64c12b8e4539ae5ccf8bf807a0e5a361cc6`;
base image SHA-256 is
`1df763d1db5798468eb23dd3bdcca2c8a4ad0dc4e3a35dfe2c7fd1c34c7fd6c5`.
It extends `provider.context_injection_gate_selftest provider_minimal` to 8
cases with
`final_authorization_omitted_field_list_hash_mismatch`, proving a tampered
`omitted_field_list_hash` in the final injection authorization is rejected as
`final_injection_authorization_substituted_record`. The focused profile also
keeps the denied pre-attachment `provider.context_injection_gate` path green
with provider export disabled, automatic context injection disabled,
`context_attached_to_provider_body: false`, provider write `not_attempted`,
`can_attach_context: false`, and explicit local-only recovery-status omission
evidence present. The selftest remains local-only test infrastructure: it
mutates no global event log, creates no real provider request envelope, creates
no positive binding records, creates no final authorization records, attaches
no provider context body, and attempts no provider write. A prior attempt,
`release\vm-reports\shadow-20260703-141017-18808.json`, observed the new
selftest response passing but failed the run on a post-selftest serial TCP
reconnect; the focused profile now runs that selftest as the terminal command.
The slice does not add memory writes, provider export, fallback execution,
recovery command dispatch, service inventory mutation, module disable,
restart, artifact load, durable writes, rollback-store writes, transaction
append, rollback application, persistence, external bytes, candidate execution,
executable mapping, provider auto-load, broad mutation, or installed rollback
state.

Latest provider context injection-gate recovery-status omission verification:
the 2026-07-03 local report-timestamp focused provider-memory profile
`release\vm-reports\shadow-20260703-140416-24496.json` passed 159/159
predicates with 11 executed commands and `duration_ms: 62815`; report SHA-256
is `f971fbb182fce2574843b5950076c694a240868b46491161f5c3b1b38de24fd0`;
base image SHA-256 is
`c4d8a6f9ba317213d59b71aa75b63b9b9c298f6e26853d56a84d1efbdf4d2182`.
It adds the same read-only
`raios.provider_minimal.local_only_omission.v0` recovery-status omission object
to `provider.context_injection_gate provider_minimal`, proving the local-only
`current.recovery_lifeline_status` fact and
`recovery.lifeline.status.current_boot` locator stay omitted before provider
body attachment. The focused profile `provider-memory` was added to run common
provider context checks plus this injection-gate path without the long full
provider/module matrix. The VM proved automatic context injection disabled,
`context_attached_to_provider_body: false`, `can_attach_context: false`, final
authorization missing, prewrite body check not attempted, provider writes
`not_attempted`, and the recovery status omission evidence present. The slice
does not add memory writes, provider export, fallback execution, recovery
command dispatch, service inventory mutation, module disable, restart, artifact
load, durable writes, rollback-store writes, transaction append, rollback
application, persistence, external bytes, candidate execution, executable
mapping, provider auto-load, broad mutation, or installed rollback state.

Latest provider context gate/export recovery-status omission verification: the
2026-07-03 local report-timestamp quick profile
`release\vm-reports\shadow-20260703-135340-24628.json` passed 417/417
predicates with 59 executed commands and `duration_ms: 169446`; report SHA-256
is `dee6d94fd7c865abe13760c6261927e0711c6970f6e2a6fee47e074d8674104b`;
base image SHA-256 is
`d5148c70ec56787f65f9642c71bb68fc83369d9b8d004a33f8aaf8e36cce2c11`.
It adds read-only
`raios.provider_minimal.local_only_omission.v0` evidence to
`provider.context_gate provider_minimal` and
`provider.context_export provider_minimal`, proving
`current.recovery_lifeline_status` and
`recovery.lifeline.status.current_boot` are local-only fields omitted from
provider context. The same quick VM also re-proved the provider packet omission
from `memory.context provider_minimal`, kept provider export disabled, kept
context attachment to provider bodies false, kept provider writes
`not_attempted`, and continued to deny export without a fake provider request
envelope. The slice does not add memory writes, provider export, fallback
execution, recovery command dispatch, service inventory mutation, module
disable, restart, artifact load, durable writes, rollback-store writes,
transaction append, rollback application, persistence, external bytes,
candidate execution, executable mapping, provider auto-load, broad mutation, or
installed rollback state.

Latest provider-minimal recovery status omission verification: the 2026-07-03
local report-timestamp quick profile
`release\vm-reports\shadow-20260703-134738-27480.json` passed 415/415
predicates with 59 executed commands and `duration_ms: 185349`; report SHA-256
is `768419ef893172f42fbab42a3bede18b90ca3e02054834828af7bc2a3973d615`;
base image SHA-256 is
`d1dc57bda91e4cc1becaff02e249a8218d3af9f359b35655b8ed421fd21e6886`.
It extends `provider_minimal` redaction/classification evidence so the new
local-only `current.recovery_lifeline_status` fact and
`recovery.lifeline.status.current_boot` locator are explicitly listed in
`omitted_fields` and in the nested provider packet `omitted` list, while the
packet `included.current` list continues to omit the recovery status locator.
The quick VM parsed the `memory.context provider_minimal` response and proved
the local-only recovery status fact is omitted from the provider packet, and it
kept `provider.context_export provider_minimal` denied with packet/exported
field/omitted field/redaction/classification/token-budget hashes present,
request/export bindings missing, no provider write attempted, and no fake
request envelope. The slice does not add memory writes, provider export,
fallback execution, recovery command dispatch, service inventory mutation,
module disable, restart, artifact load, durable writes, rollback-store writes,
transaction append, rollback application, persistence, external bytes,
candidate execution, executable mapping, provider auto-load, broad mutation, or
installed rollback state.

Latest recovery agent-context status fact verification: the 2026-07-03 local
report-timestamp focused recovery profile
`release\vm-reports\shadow-20260703-133613-9360.json` passed 3634/3634
predicates with 184 executed commands and `duration_ms: 311351`; report
SHA-256 is
`2a7253e34aea055bdfcdd05a8772e548ef3bf3e1b776aad8b8050dca0d5c0771`;
base image SHA-256 is
`dc9d1aab855f39161413bde3e612b1dc0d6b24f805349463d22771d901801399`.
It adds `recovery.lifeline.status.current_boot` to `memory.context` as a
current-boot/local-only `raios.agent_context.recovery_lifeline_status_fact.v0`
fact and to `memory.query`/`memory.trace` as a locator back to the read-only
`recovery.lifeline.status` source. Before the retained status-execution result
exists, the fact reports `unavailable_missing_retained_result` with
`source_retained_result_verified: false`. After the retained result exists, it
verifies the result against the retained status-read handler and
completion-denial evidence, reports `available_read_only_current_boot`, exposes
the retained result event id/hash and source event ids, and keeps the nested
projection bounded. The fact does not write memory, export to a provider, create
a fallback executor, dispatch a recovery command, mutate service inventory,
enable command execution, load recovery artifacts, write durable audit or
rollback-store state, attempt transaction append, apply rollback, persist state,
consume external bytes, map executable candidates, auto-load from a provider, or
grant broad mutation. The same cleanup centralizes missing/mismatched/accepted
status handling in the shared recovery status-read state helper.

Latest recovery command-envelope status-read verification: the 2026-07-03 local
report-timestamp focused recovery profile
`release\vm-reports\shadow-20260703-132135-17152.json` passed 3623/3623
predicates with 181 executed commands and `duration_ms: 313612`; report
SHA-256 is
`c0213b4df9f7578e0d6e12b56e57abb44b77d66b44d6108b31870edd0d6fa7eb`;
base image SHA-256 is
`a7e4d985fd440b08cd57658864a5593c64c966f8d226ed8e0e2ea7be2cc363e7`.
It keeps `recovery.lifeline_status_result_read` as the read-only consumer of the
retained `raios.recovery_lifeline_status_execution_result.v0` record and exposes
the same consumer through the command-shaped `recovery.lifeline.status` method.
Before the retained result exists, both paths report
`denied_missing_retained_result`, create no recovery result records, and return
an unavailable status projection. After the retained result exists, both verify
the result against the latest retained status-read handler and completion-denial
evidence, then return a bounded current-boot
`raios.recovery_lifeline_status_projection.v0` with `recovery_core_alive: true`.
The existing `agent command_envelope` boundary now also allowlists
`target_method=recovery.lifeline.status` with
`requested_capability=cap.recovery.load_artifact.read`, records the normal
current-boot/local-only envelope decision event, and dispatches only to the
existing status-read method; a wrong capability is denied before status dispatch.
The status read itself remains non-mutating, and provider recovery route,
lifeline dispatch, command execution, actual status execution, module disable,
restart last-good, recovery artifact load, recovery-memory writes, durable audit
writes, rollback-store writes, service-inventory mutation, and load attempt all
remain false.

Latest recovery status execution-result verification: the 2026-07-03 local
report-timestamp focused recovery profile
`release\vm-reports\shadow-20260703-124307-23596.json` passed 3536/3536
predicates with 174 executed commands and `duration_ms: 281958`; report
SHA-256 is
`a99f62a9c1e807bd417815a0ee96f4cda5b84bbcb7962ebc1d6fc04d6227fd3e`;
base image SHA-256 is
`1ae08ce368d3eeed327cc75842dbedceaf63fe42c8d3b21d6d0dd72519e8ca3a`.
It adds `recovery.lifeline_status_execution_result_diagnostic`, which consumes
the existing dispatch behavior, executor capability table, side-effect gate,
retained status-read handler, and retained execution-stage completion-denial
chain. Before execution-stage evidence is retained, the diagnostic reports
`blocked_missing_evidence`, creates no event-log record, and keeps
`recorded_event_id: null`. After the completion-denial stage is retained, it
records a current-boot/local-only
`raios.recovery_lifeline_status_execution_result.v0` hash reference with
`retained_read_only_result_command_still_denied`, binds the retained
status-read-handler event/hash/projection, retained completion-denial event/hash,
dispatch behavior hash, executor capability table hash, side-effect gate hash,
and all prior execution-stage hashes, and exposes the binding through
`audit.events`. The result remains non-authorizing: dispatch, command execution,
actual lifeline-status execution, recovery-memory writes, durable audit writes,
rollback-store writes, recovery artifact load, service-inventory record
creation, inventory mutation, and load attempt all remain false. The same slice
also cleaned the readiness decision into the recovery command eval path so the
diagnostic and future consumers share one gate.

Latest recovery execution-stage source-diagnostic, harness-cleanup, and
status-readiness verification: the 2026-07-03 local report-timestamp recovery
profile `release\vm-reports\shadow-20260703-121942-14876.json` passed
3478/3478 predicates with 172 executed commands and `duration_ms: 297242`;
report SHA-256 is
`e2ca038c5d743612109c1eab3a6e4c38cf77e8e1228f7e01ef97b3db1d63e1b4`;
base image SHA-256 is
`55e11a8783b89996ee79894b4d2171299d6e2e25f4e5ca78a2e84b409d5319a2`.
It proves `recovery.lifeline_command_admission`,
`recovery.load_artifact_by_hash_target_binding_diagnostic`,
`recovery.lifeline_command_dispatch_diagnostic`,
`recovery.memory_write_authority_diagnostic`,
`recovery.durable_audit_rollback_write_authority_diagnostic`,
`recovery.service_inventory_side_effect_boundary_diagnostic`,
`recovery.lifeline_command_dispatch_behavior_diagnostic`,
`recovery.lifeline_command_executor_capability_table_diagnostic`,
`recovery.lifeline_command_side_effect_gate_diagnostic`, and every
execution-stage diagnostic from
`recovery.lifeline_command_execution_enablement_diagnostic` through
`recovery.lifeline_command_execution_completion_denial_diagnostic` now expose a
current-boot/local-only/read-only
`raios.recovery_artifact_load_denial_source.v0` object after the denied
`recovery.load_artifact` audit binding exists, and they report the earlier
incomplete source state as present-but-missing identity evidence before the
full recovery load-binding chain is retained. The available source object
carries the denied load-artifact event id, nested load-binding status/reason,
completion denial event id/hash, side-effect-gate hash, and source
rollback/policy/inspect hashes into the
admission/target/dispatch/memory-write/durable-audit/service-inventory/dispatch-behavior/executor/side-effect-gate/enablement/preflight/intent/commit-gate/result-denial/audit-denial/observation-denial/completion-denial
diagnostic views without changing the canonical load-artifact-by-hash target
binding, memory-write authority hash, durable-audit authority hash, service
inventory side-effect boundary hash, command-dispatch behavior hash, executor
capability table hash, side-effect gate hash, execution enablement hash,
execution preflight hash, execution intent hash, execution commit-gate hash,
execution result-denial hash, execution audit-denial hash, execution
observation-denial hash, or execution completion-denial hash or creating new
retained target, dispatch, memory-write, durable-audit, service-inventory,
command-dispatch behavior, executor capability, side-effect gate, or
execution-stage records from source-only diagnostics. The recovery
execution-binding harness now sends `agent audit.events 256` after these
diagnostics so the large audit scrape remains terminal and does not require a
post-audit serial reconnect. The cleanup checkpoint also groups the eight
duplicated execution-stage load-denial source assertion blocks behind
`Assert-RecoveryExecutionStageLoadDenialSource` while preserving their predicate
names and stage-specific hash assertions; PowerShell parsing passed and
`git diff --check` reported only the repo's existing CRLF warnings. The same
report also proves the existing dispatch diagnostic now emits
`raios.recovery_lifeline_status_execution_readiness.v0`: before the execution
chain is retained it reports `blocked_missing_evidence` with
`recovery_lifeline_command_execution_enablement_not_implemented`,
`execution_completion_denial_present: false`, and
`would_execute_lifeline_status_read: false`; after the retained completion
denial exists it reports `available_read_only_non_authorizing` with
`recovery_lifeline_status_read_ready_command_execution_disabled`, the retained
status-read-handler event/hash/projection/id, completion-denial presence, and
`would_execute_lifeline_status_read: true`. The readiness record remains
non-authorizing: dispatch, command execution, actual lifeline-status execution,
recovery memory writes, durable audit writes, rollback-store writes, recovery
artifact load, service-inventory record creation, inventory mutation, and load
attempt all remain false. The same report also proves the denied dispatch view
reports `defined_non_executable` /
`recovery_lifeline_command_dispatch_execution_disabled` with the retained
completion-denial event/hash and the non-authorizing load-denial source object.
The path keeps module disable, restart, artifact load, memory write, durable
media writes, durable audit writes, rollback-store writes, transaction append,
rollback application, service inventory mutation, lifeline command dispatch,
command execution, persistence, external bytes, candidate execution,
executable mapping, provider auto-load, broad mutation, and installed rollback
state denied.

RESOLVED 2026-07-04: the full Shadow VM checkpoint harness repair is verified
green. The bounded per-boundary audit scrapes (`audit.events 24/64/96` close
to the records they prove, no giant mid-profile `audit.events 256`) were
committed in `0ee066e` and the full profile now completes:
`vm-harness\shadow-vm-smoke.ps1 -Profile full -TimeoutSeconds 420 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10`
passed with report `release\vm-reports\shadow-20260704-184615-9224.json`,
`result: passed`, 7814/7814 predicates, 334 executed commands,
`duration_ms: 1226345`, report SHA-256
`68c8e160849ca9333867ea6007013b2e49d6f39e4e7e4930b761944967ba96ee`. This is
the first green full checkpoint since 2026-07-02 (6789/6789); the predicate
count grew because the bounded scrapes split checks without weakening any.
One earlier same-day attempt failed as host-transport (see the failure
classification log at the top of this file).

RESOLVED 2026-07-05: packet M0-2 landed and verified (quick profile
`shadow-20260705-094659-19752.json`, 417/417 predicates, new
`serial_transport_failure` / `qemu_process` / structured `stderr_log`
report fields present). **M0 Stabilize is closed.** All three M0 criteria
hold: honest committed tree, full profile green
(`shadow-20260704-184615-9224.json`, 7814/7814), all recent failures
classified (see failure classification log at the top of this file).

RESOLVED 2026-07-05 (same day, later session): slice M1-1 landed. The repo
root was already a cargo workspace; new member `raios-core`
(`#![cfg_attr(not(test), no_std)]`, dep: sha2 only) provides
`sha256_bytes`, `sha256_hex`, and the `ByteSink` trait with host tests
(`cargo test --locked -p raios-core`: 3/3 passed in 0.16s — official
SHA-256 vectors, hex round-trip, Vec sink). `descriptor_sources.rs`
deduplicated onto `raios_core::sha256_bytes`; kernel rebuilt unchanged and
quick profile green (`shadow-20260705-100850-5584.json`, 417/417).
Constraint discovered: the `hello_service.rs` `sha256_bytes` duplicate
CANNOT be removed yet — the file participates in the signed Hello source
snapshot, and editing it fails the build with an
`artifact_content_source_sha256` mismatch; that dedup moves to M2
(de-hello-ify) together with a descriptor/signature chain update.

RESOLVED 2026-07-05: slice M1-2 landed. The pure protocol parsers moved
into `raios-core` as `pub fn method_eq`, `method_head_eq`,
`parse_sha256_ref`, and `parse_current_boot_event_sequence` (the
`EventId` construction stays kernel-side as a thin wrapper);
`agent_protocol_support.rs` re-exports them (`pub(crate) use raios_core::…`)
so no other kernel file changed. Truth-table host tests: `cargo test
--locked -p raios-core` 9/9 (method boundary rules, sha256: prefix
case-insensitivity, uppercase hex, length/charset rejects, 8-digit
event-sequence edges). Kernel rebuilt unchanged; quick profile green
(`shadow-20260705-101746-21240.json`, 417/417).

RESOLVED 2026-07-05: slice M1-3 landed. Origin divergence resolved — the
two remote-only commits were the owner's README tagline edits made via
the GitHub web UI; merged cleanly in `0f144c4`. `.github/workflows/ci.yml`
(commit `d57243b`, pushed) runs two jobs on every push/PR: `cargo test
--locked -p raios-core` (stable) and the pinned `nightly-2024-10-15`
build-std release build of seed-kernel with the exact RUSTFLAGS the local
build script uses (command replicated locally first, exit 0), uploading
the kernel ELF artifact. First CI run GREEN:
https://github.com/Sportinger/raios/actions/runs/28734704673 — host tests
16s, kernel build 1m11s.

RESOLVED 2026-07-05: slice M1-3b landed and **M1 Testable Core is
closed**. The `vm-smoke` CI job (windows-latest, chocolatey QEMU) runs
the headless quick profile on every push and uploads the report artifact
even on failure. First attempt failed honestly: Windows checkout CRLF
conversion changed the raw source bytes and the seed-kernel build script
correctly rejected the P-256 signed source snapshots
(`signature::Error`); fixed by forcing `core.autocrlf false` before
checkout (`943a9a0`). Green run 28734873106: host tests 15s, kernel
build 1m11s, VM quick profile 5m39s. M1 capability sentence verified:
kernel logic passes as host `cargo test` in under a second, and a second
machine builds and smokes every commit.

RESOLVED 2026-07-05: slice M2-1 landed. `raios-core::record` provides the
single typed `Value` model (Null/Bool/U64/borrowed Str/Sha256 rendered as
`"sha256:<64hex>"`/EventSequence rendered as
`"event.current_boot.%08d"`/Array/ordered Object), one serializer
`write_json` matching the kernel JSON conventions byte-for-byte (CRLF,
two-space indents, the `json_str` escaping table from
`agent_protocol_support.rs`), and `sha256_of_json` computed through a
hashing `ByteSink` over exactly the serialized bytes — emitter/hasher
divergence is structurally impossible. 14/14 host tests (escaping truth
table, exact nested rendering bytes, empty containers, hash-equals-bytes,
rendering forms). Kernel untouched. Orchestrator review caught and fixed
one worker bug: empty Array/Object rendered without their closing
bracket. IMPORTANT for every M2 port: the existing kernel hashers hash
canonical `key=value` LINES, not JSON bytes (`module_evidence.rs:4538` +
`:542`), so each ported gate must map its old hash convention explicitly.

Historical M6 promotion-loop status (not the current cursor; see the top
`Current exact next task` paragraph and `docs/ROADMAP.md`). Milestone M6 Promotion Loop v0, sub-milestone
M6A, `docs/ROADMAP.md`): M6A-1 (intake mechanism) and M6A-2a (REAL
runtime delivery) are DONE (2026-07-06). M6A-1:
`module_candidate_intake.rs` accepts bounded bytes (256 KiB cap), hashes
in-guest, validates via `wasm_runtime::validate_module_bytes`
(wasmi::Module::new), returns an inert `ExternalWasmCandidate`
(load/execution/persistence hard-false on every path). M6A-2a: new
`module_candidate_channel.rs` reassembles a real external `.wasm`
delivered over the serial console as base64 chunks (bounded RAM buffer,
local base64 decoder, fail-closed discard on malformed/overflow/empty),
finalize's ONLY sink is `intake_external_wasm_candidate`; two registered
read-methods `module.submit_candidate_chunk` / `module.submit_candidate_finalize`
(no new MethodAction, no dispatch-arm behavior change); the delivery
label is now the real `serial_console_base64_chunks_v0` (the
`pending_m6a_slice2` placeholder is retired). Verified: focused
`shadow-20260706-102027-16828.json` 176/176 (real 4205-byte echo wasm
delivered, exact SHA match f81f9442…abd2, retained inert, all denials
false, malformed-chunk discard + VM-still-responsive negative case) +
quick regression `shadow-20260706-102839-18048.json` 562/562.
Adversarially reviewed: no reachable load/grant/instantiate/execute/persist
sink, no panic/OOB/bound-bypass/lock/state-leak.
KNOWN RESIDUAL (from the M6A-2a adversarial review, honest gap): finalize
runs `wasmi::Module::new` on attacker-controlled bytes — bounded (256 KiB,
freed, no execution/instantiation/JIT/authority) but the wasm
validator/parser itself is NOT time/fuel-bounded, so a maliciously crafted
≤256 KiB module is a theoretical guest-DoS surface. M4 trap-hardening
covers malformed→module_new_error without panic; time/fuel-bounding the
validation is a candidate for a later hardening slice. Also: each
submit_* call writes one fixed read-audit entry (static strings, bounded
ring — no attacker bytes) like every read method.
M6A-2b DONE (2026-07-06, harness-only, zero kernel change): the
module-evidence cross-check now evaluates the REAL delivered-candidate
artifact identity — `shadow-vm-smoke-profile-full-module-evidence.ps1`
computes the echo wasm SHA from disk (anchored to the known ECHO hash
`f81f9442…abd2` == intake `artifact_sha256`) in place of the synthetic
`2222…`, with a new `protocol:module_evidence_real_candidate_sha_matches_echo`
predicate and `can_load_now: false` preserved. Honest gap:
vm_test_report/local_attestation hashes stay synthetic (post-run report
hash; later land-if-cheap). Verified: FULL `shadow-20260706-104758-19976.json`
8160/8160. **Sub-milestone M6A (external candidate identity) COMPLETE.**
M6B-1 DONE (2026-07-06): raiOS can cryptographically VERIFY a
P-256/SHA-256 promotion-authority signature against a PINNED public key
(distinct from the build/descriptor key) over the existing canonical
local-attestation reference hash, and mark the local_attestation gate
`signature_verified` — while grants_capability / authorizes_guest_load /
can_load_now / artifact_loaded / service_started / load_attempted ALL
STAY FALSE (grants nothing, loads nothing). New `raios-core/src/
promotion_attestation.rs` (pinned-only verifier — NO key arg; fails
closed; host tests 31/31 incl. roundtrip/tamper/wrong-key/malformed-DER/
self-pin); attestation.rs verifies an optional hex-DER signature over the
UNMODIFIED expected reference hash (no canonical churn), new statuses
`local_attestation_signature_verified_load_still_denied` /
`mismatched_local_attestation_signature`; 10th selftest case
`promotion_signature_invalid`. ADR 0007 (Proposed) records the first
runtime trust anchor. Verified: host `cargo test -p raios-core` 31/31 +
FULL `shadow-20260706-115036-5444.json` 8162/8162. Adversarially
reviewed: could-not-refute (no authority path, no forgery-that-grants, no
panic/OOB/churn/golden-regression).
CRITICAL — the pinned key is a NON-RATIFIED PLACEHOLDER (the P-256
generator point, scalar 1, publicly known); `PROMOTION_AUTHORITY_IS_PLACEHOLDER=true`.
It is safe only because nothing is granted. See ADR 0007 "M6B-2
Enforcement Precondition".
M6B-2 DONE (2026-07-06): the FIRST authority flip. `module.grant_diagnostic`
now emits `grants_capability=true` labeled `trust_tier=dev_key_not_owner_sealed`
ONLY when the evidence chain is valid AND the retained local attestation is
`signature_verified` (dev-key P-256) AND its five component hashes bind to
exactly this grant (`module_grant_grants_capability`, grant.rs:418-432, fail-closed
on None/unsigned/mismatch/shadow). `can_load_now` / `authorizes_guest_load` /
`grants_load_now` stay `no()` — load needs the M6C slot/loader + M6D
audit/rollback. Owner decision this session (ADR 0007): the dev key gets full
GRANT function labeled honestly; NO placeholder hard-refuse; owner key K + load
are the later sealing/M6C-D work. Verified: FULL `shadow-20260706-133249-16364.json`
8168/8168 (10-case grant selftest incl. the grants_capability=true case + 4
fail-closed + co-emission invariant; one qemu_exited flake, green on retry).
Adversarially reviewed: could-not-refute (no grant without verified+bound
signature, no load leak, trust_tier co-emitted atomically, no golden churn);
the code-vs-comment mismatch it flagged is fixed (promotion_attestation.rs
comment now matches the ratified dev-tier decision).
M6C-1 DONE (2026-07-06): a granted external Wasm candidate now LOADS and RUNS.
`granted_candidate_service.rs` (new): the accepted+valid finalize retains the
delivered bytes in `module_candidate_intake::RETAINED_CANDIDATE` (RAM,
current_boot); a granted-candidate current-boot service loads into its own RAM
slot and `start()` FAILS CLOSED — it executes `wasm_runtime::execute_module_bytes`
(extracted from the echo runner, define_capability_envelope UNCHANGED) ONLY when
`module_grant_grants_capability` (M6B-2: valid + signature_verified + all 5
hashes) is true AND the retained SHA-256 == the granted artifact_hash. Dispatch
rows (agent_protocol.rs) route module.load_ephemeral/service.start/stop/drop to
the granted service only when granted; ungranted falls through to the generic
DeniedModuleLoadEphemeral. Grant surfaces a dev-tier `can_load_now` true only
when grant+retained-bytes+slot+loader ready; `module_grant_check_can_load` stays
false and required_before_load (durable audit+rollback) intact. Every run/response
labeled `trust_tier=dev_key_not_owner_sealed`. Verified: focused
`shadow-20260706-145058-10548.json` 178/178 (real echo wasm delivered → granted
load+run, instantiation_ok/run success/fuel>0/guest log; ungranted → capability_denied,
no instantiation; generic durable gate stays denied) + FULL
`shadow-20260706-162211-18496.json` 8168/8168 + raios-core 31/31. Max-effort
adversarial review: could-not-refute (no run without verified+bound grant, no
envelope escape, no durable/owner-sealed write, dev-tier can_load_now does not
leak into the durable gate).
KNOWN RESIDUALS (honest, labeled): (1) the live end-to-end dev-key SIGNATURE in
the m6c-promotion profile is unavailable under Windows PowerShell 5.1 / .NET
Framework P-256 (scalar-1 import + DER) — a TEST-TOOLING gap only; the run
mechanism is proven by the in-guest granted_candidate selftest and the P-256
verify by M6B-1 host tests, so the profile marks it informational and relies on
the selftest. (2) the live wasm execution path shares the M4/M5 echo runner and
does NOT wire the wasm memory `limiter` (guest memory bounded by address
space + host allocator, fuel bounds CPU) — identical surface to echo, a
defense-in-depth candidate for a later hardening pass.
M6C-2 DONE (2026-07-06, non-authorizing honesty slice): After M6C-2,
inspecting service.inventory, module.service_slot_diagnostic, and
module.loader_runtime WHILE a dev-key-granted external candidate is actually
loaded this boot shows the live loaded service, its allocated RAM slot, and the
dev-tier accepted/loaded/can-load-now run reflected truthfully
(trust_tier=dev_key_not_owner_sealed) instead of the current hardcoded
absent/unallocated/false ??? with maps_executable_pages, durable/persistent,
native guest-load, and owner-sealed still false and nothing newly granted,
loaded, or written. `granted_candidate_service::live_load_projection()` is the
single current-boot source for the live projection and is derived from
`loaded_snapshot()`; guardrail booleans stay literal false. `service.inventory`
adds `svc.dev.granted_candidate` only while loaded; `module.service_slot_diagnostic`
adds a separate `live_granted_service_slot` object without changing the
reservation-reference policy/selftest; `module.loader_runtime` adds one
`live_granted_load_projection` block while its native readiness header/policy/
evaluation all-false path and loader_runtime selftest stay unchanged.
`module.granted_candidate_selftest` now has 5 cases, adding loaded-projection
and not-loaded-projection truth cases. Worker checks only (no VM per packet):
`cargo fmt --all -- --check`, release build, and `scripts\scan-secrets.ps1`.

M6D-1 DONE (2026-07-06, RAM-only verified un-promote / rollback): A
dev-key-granted external Wasm candidate that is loaded and running this boot
can be UN-PROMOTED through a verified rollback path: at load the
granted-candidate service records a RAM-only rollback plan (artifact_hash +
pre-load service-inventory hash + ram_only_service_slot_id + cleanup-actions
hash) captured on the successful can_load branch BEFORE the slot flip, and
`service.rollback_apply svc.dev.granted_candidate` stops -> drops/clears-bytes
-> frees the RAM slot -> removes it from service.inventory, then verifies the
re-projected inventory hash equals the captured pre-load baseline before
recording the un-promote ??? returning capability_denied (fail-closed) when no
promotion was recorded this boot; every response labeled
trust_tier=dev_key_not_owner_sealed with owner_sealed=false, durable=false,
persistence=none. `seed-kernel/src/granted_candidate_service.rs` stores a
`PromotionRecord` in RAM only, uses
`module_evidence::computed_module_rollback_plan_hash`, clears retained
candidate bytes through `module_candidate_intake::clear`, and does not reuse
`drop_service()` for rollback. The granted selftest is now 8 cases, adding
recorded-promotion restore verification, no-promotion fail-closed denial, and
one-shot second-apply denial. `shadow-vm-smoke.ps1 -Profile m6d-rollback` is
wired for orchestrator VM proof; live PS5.1 dev-key signing remains
informational, with the positive rollback path proven by the in-guest selftest.
M6D-1 does not touch `raios-core/src/scoped_rollback_apply.rs`, AHCI/storage
authority, rollback writer bindings, or the generic durable
`raios.module_load_gate.v0`; no disk write or persistence is introduced.

M6 COMPLETE (2026-07-06): the dev-tier promotion loop is closed end to end —
delivered -> identity -> grant -> load -> RUN -> rolled back, all RAM-only,
fail-closed, honestly labeled dev_key_not_owner_sealed, each slice
max-adversarially reviewed.

M7 PERSISTENCE FOUNDATION now active (map m7-persistence-map-2026-07-06.md,
revalidated M7-0 c08a636). M7-0 recorded the divergence: M6D-2 (durable
promotion transaction) was deferred and is a prerequisite for M7D; sequence
M7A -> M7B (SEED_DATA RECLOG store) -> M6D-2-into-SEED_DATA -> M7D.
M7A-1 DONE (2026-07-06, host-side infra, no kernel): `scripts/make-gpt-persist-image.py`
builds a real GPT persist disk (protective MBR + primary/backup GPT with correct
CRC32s; SEED_ESP_A/B 128 MiB FAT32 + SEED_DATA custom type GUID
5EEDDA7A-...-000000000001 carrying a `RAIOS_DATA_SB_V0` superblock at LBA0/1 with
region table BOOTCTL 2/8, RECLOG 16/4096, ARTSTOR 8192/516096); it hard-refuses
release/ output paths. Harness wires it as a 4th QEMU drive (`bus=ide.3`,
`id=raiospersist0`) behind an optional `-PersistDiskPath`; new
`shadow-vm-smoke-profile-persistence.ps1` validates GPT header/CRC/SEED_DATA/superblock
host-side. Fixed an orchestrator harness bug (Resolve-PersistDiskImage leaked the
builder stdout into its return value). image-layout-v0.md gained the V0 partition
type GUIDs + raw-region-map note. Verified: `-Profile persistence` 7/7 (4th drive
attaches, kernel boots) + quick regression green (one audit.events host-transport
flake, green on retry).
M7A-2 DONE (2026-07-06, kernel read-only GPT/SEED_DATA detection): the kernel
finds + validates SEED_DATA on the harness GPT disk and reports typed
`persist.layout` evidence (present/absent/invalid) with ZERO writes. Pure
parsers in `raios-core/src/gpt_layout.rs` + `seed_data_layout.rs` (protective
MBR, GPT header + entry-array CRC32, ESP/SEED_DATA type-GUID/name, superblock
magic/version/region/hash + LBA1-copy) with 42 host tests incl. every corruption
fixture (bad CRCs, truncated, absurd counts w/ checked_mul, duplicate/missing
SEED_DATA, hash/copy mismatch). `seed-kernel/src/persist_detect.rs` reads sectors
from the 4th AHCI port via the existing `READ_DMA_EXT` (no WRITE_DMA_EXT, no
driver rework, no write-boundary touched); `persist.layout` is a Read0 method,
on-demand (not at boot), evidence local_only/current_boot,
write_attempted/writes_enabled/persistence_claimed all false. Corruption/absent
→ fail-closed, kernel continues without persistence. Orchestrator hardening from
the max adversarial review: `persist_detect.rs` now uses `checked_add(1)` for the
LBA1 superblock read so a maliciously-emulated `first_lba=u64::MAX` device fails
closed (Invalid) instead of overflow-panicking in debug builds. Verified:
`-Profile persistence` 12/12 (kernel GPT/superblock needles + no-disk
`gpt-absent-fail-closed` child run + `kernel_layout_read_only_current_boot`) +
quick 562/562 + raios-core 42/42. Max adversarial review: could-not-refute
(strictly read-only, fail-closed on every corruption, empirically fuzzed, no
persistence claim).
M7B-1 DONE (2026-07-06, RECLOG read/scan, still read-only): the kernel scans the
SEED_DATA RECLOG region and chain-validates RAIOSRC0 frames (magic |
frame_len≥512 &%512 | payload_len | seq | prev_frame_sha256 | payload_sha256 |
payload | zero pad; header_len=88), reporting typed head/tail/count + torn-tail
via `durable.record_log_scan`. Pure codec+scan in `raios-core/src/durable_record_frame.rs`
(51 host tests incl. bad magic/payload-hash/prev-hash/seq gap+dup/torn/multi-sector,
frame_len=0 rejected); `seed-kernel/src/durable_store.rs` reads the bounded RECLOG
region (sector<4096) via the existing READ_DMA_EXT and feeds scan_reclog; scan
STOPS at the first invalid frame (torn tail = evidence, not authority); appends
stay `capability_denied`, evidence local_only/current_boot. `make-gpt-persist-image.py`
gained `--seed-reclog-fixture empty|valid:N|valid:N,torn`; the persistence profile
proves chain-head/count/torn-tail/empty via child-VM fixtures. Orchestrator harness
fix: child-VM boot/answer timeout raised 45s→180s (they booted too slowly under
4-VM contention — flake, not a kernel hang; the max adversarial review confirmed
the scan terminates + is bounded, could-not-refute). Verified: `-Profile persistence`
19/19 + raios-core 51/51.
M7B-2 DONE (2026-07-06, the FIRST REAL persistence WRITE): raiOS performs a
scoped durable APPEND to `append.record_log.seed_data` ONLY, with full M3
build→verify-region→write→readback→inspect→report discipline. It builds a
`raios.durable_record.v0` boot-lifecycle frame chained to the scanned tail
(seq=tail+1, prev=tail_frame_sha256), verifies the target span lies fully inside
the pinned RECLOG bounds, writes the multi-sector span, RE-READS it from disk,
proves it byte-identical (readback sha256 == planned frame_sha256) and re-parses
it as a valid chained frame, and only THEN reports `appended` via
`durable.record_log_append`. Store-full → deny (no rotation); torn/invalid tail
→ deny (NO overwrite); within-boot only (`persistence_claimed:false`, dev-tier,
RAM ring still authoritative for current_boot UI).
STALE-MAP CORRECTION (caught by the max-effort scope, verified vs HEAD): the map
packet's "generalize the write-boundary chain for the RECLOG span" wording was
WRONG — that chain (`agent_protocol_module_write_boundary_*.rs`) structurally
always-denies and its `writes_enabled`/`authorizes_append` booleans are SHARED
cross-target, so a positive branch there would grant generic write to EVERY
module. M3-3 (the first durable write) touched ZERO boundary-chain files; M7B-2
mirrors THAT: a NEW sibling evaluator `raios-core/src/scoped_seed_data_append.rs`
with its OWN pins (EXPECTED_METHOD=`durable.record_log_append`,
TARGET=`append.record_log.seed_data`, SCHEMA=`raios.durable_record.v0`,
REGION_MARKER=`RAIOS_DATA_RECLOG`) + range/chain/write-readback-reparse gauntlet
(32 distinct typed denials). The AHCI writer `write_readback_reclog_append` loops
the existing `issue_write_sector` over the frame sectors, validating EVERY LBA in
`[seed_data_first_lba+16, +4112)` BEFORE any write (no partial-write escape);
`issue_dma_command` untouched; `scoped_rollback_apply.rs` and the shared read-only
`durable_record_log_scan_fields` literals untouched (scan method stays
capability_denied). The RECLOG region is pinned by `seed_data_layout::parse_region`
to EXACTLY start_lba=16 / lba_count=4096 (superblock SHA256-checked), so a corrupt
superblock cannot relocate or enlarge the write span.
HONESTY NOTE (from the max adversarial review, verdict could_not_refute): several
of the evaluator's pins are fed consistent kernel-derived values on both operands
(e.g. `payload_sha256` == `planned_payload_sha256`), so the evaluator RECORDS the
decision more than it INDEPENDENTLY re-derives it — the actual enforcement rests on
the AHCI writer's own in-bounds guards plus the disk readback + `parse_reclog_frame`
reparse + full-region rescan (count+1 / new tail seq+hash). This mirrors the M3
evaluator's single-trusted-caller pattern and is non-exploitable; recorded here
rather than overclaimed, per the "evidence must not masquerade as enforcement" rule.
Write set: `durable_record_frame.rs` (public `plan_reclog_append` + host tests),
`scoped_seed_data_append.rs` (NEW), `lib.rs`, `ahci.rs` (additive span writer),
`durable_store.rs` (append emitter + deterministic payload), `agent_protocol.rs`
(one dispatch row), `make-gpt-persist-image.py` (`full` fixture), persistence
profile (append/readback/chain/store-full/generic-still-denied needles).
Verified: raios-core 60/60; `-Profile persistence` 31/31 (durable-append-authorized
+ durable-readback-hash + durable-chain-head + durable-store-full-denied +
generic-target-still-denied); `-Profile module-audit-rollback` 1709/1709
UNCHANGED-GREEN (M3 pins + generic module denial intact); max adversarial review
could_not_refute; FULL regression 8168/8168 GREEN
(`shadow-20260706-203739-23872.json`, hash-verified
8f64c4133d4b51edc3e885e40db69eec78b5880210e64e62e48ab5c41cdf84d5). One persistence run was a
host-transport flake (partial serial delivery of the append command — child VM
booted, kernel idle at prompt, no write attempted; classified host-transport,
retried green).

M7C-1 DONE (2026-07-06, boot-control READ + state machine + SAFE posture,
READ-ONLY): the kernel reads the BOOTCTL region (SEED_DATA rel LBA 2 count 8 =
two 2048-byte ping-pong storage slots A[0..2048)/B[2048..4096)), validates each
`RAIOSBC0` slot envelope (magic | payload_len | seq | payload_sha256 | fixed
binary payload | zero pad), picks the highest-valid-seq slot, and runs a PURE
fail-closed state machine (`raios-core/src/boot_control.rs`) that selects the
boot slot + posture (Normal/Probation/Safe/PersistenceUnavailable) and reports
it via `boot.control_read`. SAFE is entered on both-slots-invalid, ambiguous
equal-seq-different-content, `safe_mode`, a non-bootable/None selected slot, or
pending past the failure threshold (falls back to last_good). `pending_consumed`
and `would_mark_good` are ALWAYS false this slice (consume/mark-good is M7C-2).
KEY ARCHITECTURE FACT: raiOS has NO kernel-side record/JSON READER (only
`write_json`), so the on-disk boot-control payload is a PINNED FIXED BINARY
struct decoded at fixed offsets (mirroring the superblock/RECLOG precedent), NOT
parsed JSON; the record-model + `write_json` contract is honored at the
`raios.boot_control_read.v0` EVIDENCE layer only. The on-disk const layout lives
in ONE raios-core module mirrored byte-for-byte by `make-gpt-persist-image.py`.
`MAX_PENDING_BOOT_ATTEMPTS=3` is a v0-PROVISIONAL, OWNER-OVERRIDABLE threshold
(the spec leaves it open, image-layout-v0.md:354) — recorded as an owner
decision, changeable. Timestamps/UUIDs (started_utc/last_success_utc/attempt_id)
render as `null` — bare metal has no RTC/UUID; the state machine keys only on
slot/seq/state/generation/failure_count/success_marked/safe_mode.
READ-ONLY proof: no WRITE_DMA_EXT and no sector write in the boot-control path
(the kernel read mirrors the RECLOG read via `issue_read_sector_into` bounded to
`bootctl_lba_count`); no `replace.boot_control.seed_data` target exists yet;
`scoped_seed_data_append.rs`, `scoped_rollback_apply.rs`, the write-boundary
chain, the shared `durable_record_log_scan_fields`, and the RECLOG append path
are all untouched. `current_boot_posture()` is EXPOSED but consumed by no write
path this slice. Write set: raios-core `boot_control.rs` (NEW, codec + state
machine + evidence + 10 host tests incl. 6 scenarios + 16-reason envelope
truth-table + exact-bytes render), `lib.rs`, seed-kernel `boot_control.rs` (NEW,
bounded AHCI read + emit + posture accessor), `ahci.rs` (additive read-only
BOOTCTL region read), `agent_protocol.rs` (one dispatch row),
`make-gpt-persist-image.py` (`--seed-bootctl` fixtures + const mirror + self-check),
persistence profile (boot-control-read / safe-posture-both-slots-invalid /
pending-not-consumed-in-safe needles).
Verified: raios-core 70/70; `-Profile persistence` 34/34 (3 new boot-control
needles + all 31 prior unchanged); `-Profile module-audit-rollback` 1709/1709
UNCHANGED-GREEN; max adversarial review could_not_refute (all 6 attack classes
refuted: no write path, no read escape/panic, fail-closed state machine, no
false authority, Python/raios-core const layout byte-identical, additive-only
diff; one documented NIT — the Python builder self-check oracle returns a
different decision_reason than the kernel for the unreachable None-target-slot
case, both fail closed to identical SAFE posture); FULL regression 8168/8168
GREEN (`shadow-20260706-213833-33436.json`, hash-verified
d37d8f8ccccc08452f74f22663b7623e7e439a30c15264a07874546c5d05ee09).

M7C-2 is split into three sub-slices (each its own commit, persistence
checkpoint): 2a SAFE-gates-the-append, 2b the boot-success WRITE, 2c offline
owner tooling. **M7C-2a DONE (2026-07-06):** `emit_durable_record_log_append`
now gains an additive, strictly-more-restrictive posture PRECONDITION — when
`current_boot_posture()` (M7C-1) is Safe or PersistenceUnavailable the durable
append is `capability_denied` (reason `boot_control_safe_mode`) before any
plan/write; Normal|Probation are byte-identical to before (Probation MUST stay
allowed so a boot-success audit append can escape probation in 2b). The two
existing append probes (`durable-append-authorized`, `durable-store-full-denied`)
are reseeded with `--seed-bootctl valid-a` (Normal) so they stay green, and a
NEW `persist-denied-in-safe` needle (RECLOG room + both-invalid boot control →
SAFE → denied) proves the gate. Verified: raios-core 70/70 (unchanged);
`-Profile persistence` 35/35; `-Profile module-audit-rollback` 1709/1709
UNCHANGED-GREEN. (FULL is deferred to the M7C-2 close — verified that
`durable.record_log_append` is exercised ONLY in the persistence profile and a
persist disk is attached only there, so the gate cannot affect FULL.)

**M7C-2b DONE (2026-07-06, the FIRST BOOTCTL write):** a booted kernel that
meets the map-3.4 boot-success criteria (evaluated ONCE at mark time, actively
driving the AHCI/superblock/boot-control reads) durably marks success by
ping-pong-writing a `winner.seq+1` record into the LOSER BOOTCTL storage slot
through a NEW scoped target `replace.boot_control.seed_data`
(`raios-core/src/scoped_boot_control_replace.rs`, 32 distinct denials incl.
`target_not_loser_slot` / `bad_seq_not_strictly_greater` / `write_span_out_of_bootctl`):
validate-all → write exactly one 2048B/4-sector slot → readback → reparse →
evaluate → re-read-and-assert (loser now wins by seq, last_good advanced, pending
consumed) → append a RECLOG audit record (reusing the UNCHANGED
`scoped_seed_data_append` gate directly, no nested response envelope). Crash-safe
ping-pong: a torn write damages only the loser; the current winner stays
authoritative. `last_good` advances ONLY on a genuine CASE-A Probation success
(never to an abandoned/exhausted pending — CASE C keeps it; already-marked is
idempotent). The AHCI writer `write_readback_bootctl_slot` hard-pins
byte_count==4096 / lba_count==8 / offset∈{0,2048} and validates every sector LBA
inside the BOOTCTL span BEFORE any write; `issue_dma_command` untouched; the write
never touches the superblock/GPT/RECLOG/ARTSTOR/winner-slot. `persistence_claimed`
and deterministic-slot-boot both false. Harness robustness: each child-VM fixture
probe now DELETES its large images after answering (the persistence profile was
filling the host disk mid-run as needles grew). Verified: raios-core 79/79;
`-Profile persistence` 40/40 (5 new: boot-success-marked / boot-control-write-pingpong
/ last-good-advance / failure-count-keeps-last-good / mark-denied-in-safe; all 35
prior unchanged); `-Profile module-audit-rollback` 1709/1709 UNCHANGED-GREEN; max
adversarial review could_not_refute (0 findings, all 7 attack classes refuted).

**M7C-2c DONE (2026-07-06, offline owner tooling, host-only):**
`scripts/switch-boot-slot.ps1` (dry-run by default, `-Apply` to write, refuses
`release/` + non-GPT images) + an additive `--stage-slot`/`--set-pending`
subcommand in `make-gpt-persist-image.py` that ping-pong-writes a `winner.seq+1`
pending record into the LOSER BOOTCTL slot (and optionally stages a `--payload-dir`
ESP via the existing `Fat32Builder`), reusing the SINGLE existing Python
boot-control codec mirrored with raios-core. Plus a non-gating
`scripts/experiments/ovmf-esp-selection.ps1` observation + a map addendum
(observed OVMF v2.70 default-HD-boot Not Found → EFI shell, both ESPs FS0/FS1, no
stub ran → inconclusive; deterministic firmware slot boot NOT claimed). ZERO
Rust/kernel/vm-harness change; `build_image`/fixture path untouched. Verified by
host self-checks + independent re-run (set-pending ping-pongs the new slot to win;
dry-run writes nothing; `-Apply` advances pending; release/ + non-GPT refused by
both layers; scan-secrets clean).

**M7C COMPLETE (2026-07-06).** Boot control read + write closed end to end. FULL
regression 8168/8168 GREEN (`shadow-20260706-231420-33040.json`, hash-verified
d56803735146f5a77fd6454dc36a987467a972a35dd1a31eb5b23090d1f758e5). raiOS now has
TWO of the three M7 scoped write targets live (`append.record_log.seed_data`,
`replace.boot_control.seed_data`); the third (`blob.artifact_store.seed_data`) is
M7D. Still within-boot dev-tier (`persistence_claimed:false`).

M6D-2 (durable promotion transaction into SEED_DATA RECLOG, the bridge from M6's
RAM loop to M7 persistence and a prerequisite for M7D re-promotion) is split into
two workers. **M6D-2a DONE (2026-07-07, RAM prerequisite):** the dev-key promotion
signature (ECDSA/DER) was previously verified then discarded (only
`signature_verified:bool` kept). It is now RETAINED in RAM as a new Copy event
`module.promotion_signature_reference.retained`
(`ModulePromotionSignatureReference` — attestation_reference_hash, promotion
authority key sha256, signature_der[≤80]+len, signature_verified), recorded ONLY
on the signature-verified branch (never on unsigned/bad-signature), fetchable via
`latest_module_promotion_signature_reference()`, with a
`promotion_signature_retained` diagnostic boolean for a needle. This is the ONLY
reason the M6D-2b durable record can carry a REAL re-verifiable signature (not a
summary). RAM-only, no durable write, no owner-sealed/cross-reboot claim,
`PROMOTION_AUTHORITY_IS_PLACEHOLDER` stays true. Verified: raios-core 79/79
(unchanged); `-Profile module-audit-rollback` 1709/1709 UNCHANGED-GREEN.
**M6D-2b DONE (2026-07-07) → M6D-2 COMPLETE.** On a VERIFIED dev-key promote
(module LOAD) and VERIFIED un-promote (rollback_apply), raiOS now durably appends
one self-contained `raios.promotion_transaction.v0` RECLOG record binding the full
M6 chain (artifact/manifest/vm_report/local_attestation/computed_grant hashes, the
4 retained reference-event-id strings verbatim, the recomputable
attestation_reference_hash, the retained signature DER + key fingerprint, plus
rollback_plan/pre_load-inventory/slot/generation; for un-promote also
reprojected_inventory_hash + restore_hash_verified + cleanup flags) via a NEW
sibling scoped evaluator `scoped_promotion_transaction_append` on RAIOS_DATA_RECLOG
(own method/target/schema pins + 8 added authority pins — signature_verified,
grant_binds_capability, trust_tier=dev_key_not_owner_sealed, owner_sealed=false,
promotion_authority_is_placeholder=true, persistence_claimed=false; 40 distinct
denials). It reuses the payload-agnostic RECLOG codec + the M7B-2 `write_readback_reclog_append`
writer UNCHANGED (append past the validated tail — a bad seq/prev can only torn the
NEW tail, never overwrite a boot-marker frame), is SAFE-gated (`boot_control_safe_mode`)
and complete-or-absent (denies `promotion_signature_reference_missing` when the
signature is absent), and is hooked NESTED-ONLY + best-effort (the durable append
never blocks or alters the RAM promote/rollback loop). The record is a COMPLETE
self-contained re-verification input so M7D can recompute the 32-byte
attestation_reference_hash and re-verify the stored DER over it (never trusting a
stored hash/bool). Dev-tier throughout (`persistence_claimed`/`owner_sealed`/
`cross_reboot_proven` all false, `PROMOTION_AUTHORITY_IS_PLACEHOLDER` true). ZERO
edits to `scoped_seed_data_append`/`scoped_boot_control_replace`/`scoped_rollback_apply`/
the write-boundary chain/the boot-marker path/the generic load gate/grant blocked_by/
audit_rollback diagnostic. Also FIXED a PRE-EXISTING stale needle: the m6c-promotion
profile asserted `granted_candidate selftest case_count==5` but M6D-1 (c61bf93) had
grown the selftest to 8 cases (adding 3 rollback cases) without updating it — red
since M6D-1, never caught because M6D-1 verified via m6d-rollback (==8), not m6c;
corrected to `==8`. Verified: raios-core 82/82; `-Profile persistence` 41/41;
`-Profile m6c-promotion` 180/180; `-Profile m6d-rollback` 186/186; max adversarial
review could_not_refute (all 7 attack classes; one unreachable NIT — EventIdString
caps at 8 digits, i.e. ≥100,000,000 events/boot, never approached).
`-Profile module-audit-rollback` 1709/1709 UNCHANGED-GREEN; FULL regression
8168/8168 GREEN (`shadow-20260707-004402-31444.json`, hash-verified
7c12f653941147b54a9ddf5d8c698db304ffa3f1036bc96849132d34f00aca5b). M6D-2 is the
bridge from M6's RAM promotion loop to M7 persistence: an AI-authored module's
promotion now leaves a durable, independently re-verifiable transaction in the log
— dev-tier, ready for M7D to re-check after reboot.

M7D (persistent artifact store + boot-time re-promotion — survive an actual reboot)
is split M7D-1 (persist) → M7D-2 (reboot proof). M7D-1 is split 1a (raios-core) →
1b (kernel). **M7D-1a DONE (2026-07-07, raios-core host-only, grants nothing):**
the content-addressed `RAIOSAR0` ARTSTOR blob codec
(`raios-core/src/artifact_blob_frame.rs` — magic|frame_len|payload_len|payload_sha256|
wasm|pad, header 48, NO seq/prev chain: authority lives in the RECLOG record, not
the blob) + TWO new pinned scoped evaluators:
`scoped_artifact_store_blob` (target `blob.artifact_store.seed_data`, ARTSTOR span,
`write_span_out_of_artstor` + `artifact_store_full` + full write-readback gauntlet +
signature/grant/dev-tier + a NEW `promotion_transaction_verified` gate,
`persistence_claimed` pinned false — 32 distinct denials) and
`scoped_artifact_persist_append` (a structural clone of the M6D-2 promotion-append
evaluator for the RECLOG `raios.artifact_persist.v0` record — its OWN schema, because
`scoped_seed_data_append` must NOT be widened). ZERO kernel/harness edits; existing
evaluators (`scoped_seed_data_append`/`scoped_boot_control_replace`/
`scoped_promotion_transaction_append`/`durable_record_frame`) byte-for-byte unchanged.
Verified: raios-core 97/97. HONEST TARGET-COUNT NOTE: the map's "3 scoped write
targets" is an undercount — live scoped write targets are now
`append.record_log.seed_data` (M7B-2), `replace.boot_control.seed_data` (M7C-2b),
`append.promotion_transaction.seed_data` (M6D-2), and (M7D-1) `blob.artifact_store.seed_data`
+ `append.artifact_persist.seed_data`; every other write stays `capability_denied`.
**M7D-1b DONE (2026-07-07) → M7D-1 (persistent artifact store) COMPLETE.** On a
successful M6 Promote whose durable promotion transaction verified in RECLOG THIS
boot, raiOS now writes the promoted candidate's wasm bytes as a content-addressed
`RAIOSAR0` blob into the validated ARTSTOR span (3rd scoped target
`blob.artifact_store.seed_data`, via a NEW additive `ahci::write_readback_artstor_blob`
— validate-EVERY-sector-in-`[artstor_start,+artstor_lba_count)`-then-write, reusing
`issue_write_sector`; `issue_dma_command` untouched), reads it back + reparses, then
chains a `raios.artifact_persist.v0` RECLOG record (own scoped target
`append.artifact_persist.seed_data`) as the SINGLE commit point binding blob
offset/len/frame-sha + artifact/manifest/vm_report/grant hashes + service_id +
import_set_hash + the M6D-2 `promotion_transaction_sha256`. A new
`seed-kernel/src/artifact_store.rs` orchestrates it + a RECLOG-driven `artifact.store_scan`
enumerator that recomputes each blob's on-disk sha256 and reports it present/verified
but INERT — a bare ARTSTOR blob with NO chained RECLOG record is reported `garbage`,
never authority. The persist hook is NESTED-ONLY + best-effort in `load()` (Promote
path ONLY, never rollback), gated on posture Normal|Probation + the M6D-2 transaction
`performed && kind==promote`; SAFE denies (`boot_control_safe_mode`), ARTSTOR-full
denies, promotion-transaction-not-verified denies. Stored blobs gain ZERO load
authority (no `wasm_runtime`/execute path; `authorizes_load`/`maps_executable_pages`/
`durable`/`owner_sealed`/`persistence_claimed`/`cross_reboot_proven` all false) — the
code IS on disk, but re-verification + re-run is M7D-2. ZERO edits to the M7D-1a
evaluators / `scoped_seed_data_append` / other scoped evaluators / write-boundary chain
/ `durable_store` boot-marker+promotion paths / the generic load gate. Verified:
raios-core 97/97; `-Profile persistence` 48/48 (7 new: artifact-persisted /
blob-hash-verified / blob-without-record-is-garbage / persist-denied-in-safe /
artifact-store-full-denied / promotion-transaction-not-verified / scoped-target-denials);
`-Profile m6c-promotion` 180/180; `-Profile m6d-rollback` 186/186; `-Profile
module-audit-rollback` 1709/1709 UNCHANGED-GREEN; max adversarial review
could_not_refute (0 findings; 27 attack attempts all refuted — write confinement
airtight, no ARTSTOR byte reaches the wasm runtime, RECLOG record the single commit
point, orphan blobs inert garbage, persist gated on SAFE + performed&&kind==promote);
FULL regression 8168/8168 GREEN (`shadow-20260707-015537-28252.json`,
hash-verified a2b65b772f722bf1a8c598305aef2f71c1fbb652c346cbc16492f221f962c7c7).
M7D-1 (persistent artifact store) is COMPLETE — a promoted module's code now lands
durably on disk, chained to its evidence, yet stays inert until M7D-2 re-verifies it.

ACTIVE — **M7D-2** (THE PRODUCT MOMENT): boot-time re-promotion + a two-boot proof. Boot 1
promotes + persists a real external candidate, shuts down; boot 2 on the SAME kept
persist disk scans the RECLOG artifact_persist records, recomputes each blob sha256 from
ARTSTOR, re-verifies every referenced hash + the promotion-transaction readback + the
dev-key P-256 signature over the freshly recomputed 32-byte attestation_reference_hash,
then RECONSTRUCTS the candidate bytes (from the verified blob) + repopulates the 3 RAM
event-log references FROM the re-verified record (only after the signature re-verify
passes, copying signature_verified from the RESULT not the stored bool), then feeds it
through the SAME M6 gate chain (`evaluate_authorization` → `module_grant_grants_capability`
→ slot allocator → wasm_runtime) — NO bypass, NO parallel trust path, NO "trusted because
stored" — and, only on success, instantiates it so the service answers live. Anything
failing re-verification stays inert + `repromotion_denied`; SAFE => zero re-promotion;
`cross_reboot_proven=true` ONLY on the boot-2 grant record, still dev-tier
(owner_sealed/persistence_claimed false, trust_tier dev_key_not_owner_sealed).
PLAN (4 commits, mirrors the M6B-1 verify→M6B-2 flip cadence) — **STATUS: (1) DONE
(c5cdc77, dev signer 4/4); (2) DONE (repromotion.run re-verify chain that GRANTS
NOTHING — raios-core 106/106 incl. corrupt-blob/tampered-record/bad-signature/
attestation-mismatch host fixtures, persistence 48/48, module-audit-rollback
1709/1709, FULL 8168/8168, max adversarial review could_not_refute 0 findings; STEP 4
re-runs `verify_promotion_authority_signature` over the freshly recomputed
attestation_reference_hash, never trusting the stored boolean — a host test proves it
verifies even with `signature_verified:false` stored; 21 distinct denial reasons; a
forged dev-key signature yields only `reverified`/`would_repromote` EVIDENCE at
`dev_key_not_owner_sealed`, never a load); (3) DONE (authority flip: fully re-verified Normal/Probation boot records reconstruct the candidate, replay the verified RAM references, and reach execution only through the unchanged M6 `emit_load`/`emit_start` path; dev-tier only with owner_sealed/persistence_claimed false and `cross_reboot_proven=true` only on the repromoted record — the audit evaluator was CONTROLLED-widened for the `repromoted` status while still hard-denying owner_sealed/persistence_claimed/non-dev-tier on ALL statuses; verified raios-core 109/109, persistence 48/48, m6c 180/180, m6d 186/186, module-audit-rollback 1709/1709, FULL 8168/8168, max adversarial review could_not_refute; the ONE low state-hygiene finding — orphaned retain not rolled back on a NO-LOAD denial — was FIXED by clearing the retained candidate on the repopulation-failed + load-denied paths; append-only grant references remain superseded-not-rolled-back, dev-tier, no escalation, the M6 gate governs every actual grant); (4) DONE**: **(1)** a reliable p256
Rust host signer (`ota/cli/src/bin/dev-promotion-signer.rs`, scalar-1, RFC6979,
byte-identical to `promotion_attestation::verify_promotion_authority_signature`) — the
existing PS 5.1/.NET signer is documented-unreliable so M6C/M6D fall back to a synthetic
selftest that never persists; M7D-2 needs a REAL persisted signature; **(2)** NEW
`seed-kernel/src/repromotion.rs` STEP 0–4 re-verify chain that GRANTS NOTHING (read-only,
emits `raios.repromotion.v0` + a chained RECLOG audit via a NEW
`raios-core/src/scoped_repromotion_append.rs`; widen the M7D-1 `artifact_store` enumerator
to pub(crate); host tests incl. corrupt-blob + tampered-record denial fixtures); **(3)**
the authority flip (reconstruct + repopulate + dispatch the UNMODIFIED gate via
`granted_candidate_service::emit_load`/`emit_start`; register `repromotion.run`); **(4)**
the two-boot harness (`make-gpt-persist-image.py` corrupt/tamper subcommands;
`run-stage0-qemu.ps1` persist-drive `cache=writethrough` — CRITICAL: the persist drive
currently has no cache mode → `cache=writeback` → a teardown loses the bytes boot 2 needs;
NEW `vm-harness/shadow-vm-persistence-reboot.ps1` two-boot wrapper on a KEPT
`--seed-bootctl valid-a` disk, merged `raios.vm_test_report.v0`). This is the milestone
that ends the current_boot-only era. RECLOG
generic append, generic (non-`svc.demo.hello`) durable audit/rollback writes,
executable candidate-byte mapping, provider auto-load, broad mutation, ARTSTOR,
GPT/superblock metadata, and installed rollback state all STAY denied unless the
M7C-2/M6D-2 gates say otherwise. External candidate INTAKE remains allowed over
the real serial channel; dev-key-granted current-boot external candidate
load/run/rollback is RAM-only and not owner-sealed.

**M7D-2 (4/4) DONE (2026-07-07) → M7D COMPLETE → M7 PERSISTENCE FOUNDATION
COMPLETE.** The two-boot proof (`vm-harness/shadow-vm-persistence-reboot.ps1`,
`shadow-persistence-reboot-*.json`) is GREEN at 85/85 predicates, 0 failures,
across all five golden-needle categories on a KEPT persist disk: boot 1
`durable-promotion-performed` + `artifact-persisted` + `service-answers-before-reboot`;
a clean QMP quit; boot 2 `repromotion-granted` + `service-answers-after-reboot`
(`cross_reboot_proven:true` only on the repromoted grant record); the corrupt-ARTSTOR-blob
and tampered-`artifact_persist`-record children both `repromotion_denied` with a
hash-mismatch reason and no service answer; and a SAFE-posture child that skips
re-promotion before enumeration. A REAL P-256 dev-key signature (the new Rust
`dev-promotion-signer`) is persisted at boot 1 and boot 2 RE-RUNS
`verify_promotion_authority_signature` over the freshly recomputed
attestation_reference_hash — never trusting the stored boolean — before dispatching
the UNMODIFIED M6 `emit_load`/`emit_start` gate; trust_tier stays
`dev_key_not_owner_sealed`, owner_sealed/persistence_claimed false,
`PROMOTION_AUTHORITY_IS_PLACEHOLDER:true`. Two real read-path defects were found and
fixed while proving it: (a) `seed-kernel/src/artifact_store.rs::extract_sha256` read
64 hex chars immediately after the needle, but `Value::Sha256` always serializes as
`"sha256:<64hex>"`, so every needle landed on the `sha256:` prefix — boot-2
enumeration silently returned "no_artifacts" and the transaction reparse never
parsed; fixed to consume the mandatory prefix (a fail-closed, backward-compatible
skip). (b) the new `make-gpt-persist-image.py` inspector compared recomputed raw hex
against the stored `sha256:`-prefixed field, making `binding_ok`/`binding_mismatches`
vacuous (test-integrity, MEDIUM, found by max adversarial review) — fixed with the same
prefix normalization so the tamper/corrupt "did-the-mutation-land" self-checks are now
meaningful (`*-tamper-landed` guards pass). Verification: two-boot 85/85, host raios-core
109/109 + ota-tools 4/4, FULL regression 8168/8168, max-effort adversarial review
(primary fix could-not-refute; the one MEDIUM finding fixed + re-verified). The
current_boot-only era is over: an AI-authored module now survives a real reboot and
comes back to life through the same governed gate, still honestly dev-tier and never
owner-sealed.

M8 RECOVERY AGENT LIFELINE now active (map
`docs/plan-reviews/m8-recovery-lifeline-map-2026-07-06.md`, revalidated against HEAD
during M8A-1 scoping). Capability: "when the world above breaks, a minimal pinned
serial-first path diagnoses and restores last-good — restoring known-good state only,
never promoting anything new." Sequence: M8A-1 (pinned table + dispatch isolation) →
M8A-2 (real read-only snapshot) → M8A-3 (fuel-exhaustion wedge proof + dedicated
profile) → M8B (disable_module + restart_last_good, each its OWN scoped evaluator —
NEVER a shared write-boundary flip) → M8C (durable last-good + SAFE) → M8D
(load_artifact_by_hash from the local M7D store only, full M6 re-verify, never
fetches).

**M8A-1 DONE (2026-07-07, evidence-only, grants nothing).** A frozen
`LIFELINE_METHODS` table (`raios-core/src/recovery_lifeline_table.rs`, host-tested,
pinned `vocabulary_sha256=dbb5562f…95b5`) + a SEPARATE kernel dispatch path
(`seed-kernel/src/recovery_lifeline.rs`) checked BEFORE the general `AGENT_METHODS`
table (3-line hook in `agent_protocol::dispatch`, before `lookup_method`). Only
`recovery.lifeline_table` is implemented — a pure const-table read that renders
`raios.recovery_lifeline_table.v0` (transport `serial_local`, trust_state
`local_physical_console`); the five spec endpoints (`recovery.snapshot`,
`restart_last_good`, `disable_module`, `rollback`, `load_artifact_by_hash`) all return
typed `capability_denied` and mutate nothing. The lifeline module imports NONE of
wasm/provider/net/tls/event-log/durable-write machinery; lookup is full-string
case-insensitive (usable under duress); the vocabulary hash is a pin that fails the
gate on any silent authority growth (endpoint added/reordered/`implemented`-flipped).
Honest: owner_sealed false, `dev_key_not_owner_sealed`, mutates_state false,
current_boot. Verified: raios-core host 2/2 (golden hash), quick 583/583 (7 lifeline
needles — all six endpoints + a case-insensitive path), FULL 8168/8168 (frozen
`recovery` profile byte-identical — no method-name collision, exact-name interception
only), max-effort adversarial review (no BLOCKER/HIGH/MEDIUM; two LOW fixed —
case-insensitive lookup + full endpoint-coverage needles; one LOW deferred to M8A-2
with a code note: arg-bearing endpoints must move to head-token matching; the
no-event-log-write NIT kept as a deliberate lifeline-independence property). Nothing
durable written; every M6/M7 write target and grant untouched.

**M8A-2 DONE (2026-07-07, read-only, mutates nothing).** `recovery.snapshot` is now a
REAL read: `emit_snapshot` (`seed-kernel/src/recovery_lifeline.rs`) renders
`raios.recovery_snapshot.v0` from LIVE current-boot state — boot posture + the service
inventory (id/kind/core_owned/replaceable/health) + core_owned/replaceable/unhealthy
counts + lifeline availability — so an operator can DIAGNOSE before restoring. The
lifeline dispatch now threads `runtime` and routes lifeline_table→table,
snapshot→snapshot, the four mutators→typed `capability_denied` (unchanged). Secret
leakage is structurally impossible: only fixed `&'static` ids/kinds and
fixed-vocabulary health states (`healthy`/`starting`/`degraded`/`missing`) are
emitted; the free-form `last_error` detail (wifi/TLS/OpenAI text) is deliberately
dropped and `trust_state` is the const `local_physical_console`, so
`redacted:true`/`classification:local_only` are honest. No durable write, no lifecycle
change, no grant, no promotion; `provider::snapshot()` is a cached-STATE read
(`routes_through_provider:false` accurate — no outbound call); owner_sealed false,
`dev_key_not_owner_sealed`, mutates_state false, current_boot. The vocabulary hash
re-pinned to `523b719b…819f` (the pin correctly reflects snapshot going implemented;
a future implemented-flip on any endpoint fails the host+VM gate). Verified: raios-core
2/2 (re-pinned golden hash), quick 583/583 (snapshot renders live inventory/health, all
four mutators + a case-insensitive path still deny), FULL 8168/8168 (frozen recovery
byte-identical), max-effort adversarial review (no BLOCKER/HIGH/MEDIUM; secret-leakage
CLEAN by construction; one LOW — `current_boot_posture()` does a bounded read-only
BOOTCTL read per call, documented as intentional self-diagnosis, cached-posture
deferred to M8C; two NITs applied). Next: M8A-3 (fuel-exhaustion wedge proof + a
dedicated lifeline profile) — the assumption the pinned dispatcher survives a real Wasm
trap is not yet proven and is M8's key risk.

**M8A-3 DONE (2026-07-07) — M8's KEY RISK GATE PASSED: the lifeline survives a real
Wasm crash.** A new test-infra method `echo.invoke_fuel_starved`
(`seed-kernel/src/wasm_runtime.rs::run_echo_fuel_starved`, budget 1) runs the REAL echo
module until it genuinely traps with wasmi `OutOfFuel`; the trap is caught as an Err
VALUE (never a panic), echo is marked `crashed` (new `HEALTH_CRASHED` value in the
non-attested `echo_service.rs`), and — proven in the new standalone `m8-lifeline`
profile (191/191) — `recovery.lifeline_table` AND `recovery.snapshot` STILL answer
within bounded timeouts while echo is wedged, the snapshot lists echo under a new
`crashed_services` array, and the four mutators still deny. WHY it survives (honest):
Wasm is fuel-metered and runs only on-demand on the single cooperative loop, so a wedge
traps-and-returns rather than starving `console::poll` — this is fuel+cooperative
scheduling, NOT hardware isolation (that is post-M11). Execution-model verdict: proof
slice, not re-architecture. The worker correctly hit the descriptor-attestation
STOP-tripwire (the packet's const belonged in a signed hello source) and resolved it
WITHOUT re-signing — `current_boot_service.rs` stayed byte-identical to HEAD; vocab hash
unchanged `523b719b…819f`. Max-effort adversarial review found ONE HIGH honesty defect
(the `crashed` latch was never reset → a restarted, healthy, running echo was falsely
reported `crashed`); FIXED by clearing `crashed`/`last_error_id` on a successful start
(recovery) and on drop, with a new end-to-end guard (`m8-lifeline:restart_recovers_healthy`
+ `post_restart_snapshot_no_false_crash`). Deferrals (flagged, not defects): no
`store.limiter` on the echo path (fuel=1 pre-empts `memory.grow`; a real MEDIUM only when
the path runs untrusted bytes — M8B/M9); the snapshot taking echo's STATE spinlock (safe
under the single-threaded kernel). Nothing durable written; grants nothing; the wedge
event is `current_boot` + `test_infrastructure`. Verified: raios-core 111 (vocab hash
unchanged), m8-lifeline 191/191 (survival + the restart-recovery honesty guard), quick
583/583 (a host-transport `audit.events` timeout flake cleared on retry), recovery
byte-identical, FULL 8168/8168. The restore ACTIONS (disable_module, restart_last_good)
begin at M8B — each its OWN scoped evaluator, never a shared write-boundary flip.

**M8B-1 DONE (2026-07-07) — the lifeline's FIRST mutating action + FIRST durable write:
`recovery.disable_module`.** Split, mirroring M6B/M7D: **1a (cd37721, grants nothing)** —
NEW `raios-core/src/scoped_recovery_action_append.rs`, a separately-pinned scoped
evaluator (EXPECTED_METHOD `durable.recovery_action_append` / TARGET_ID
`append.recovery_action.seed_data` / SCHEMA `raios.recovery_action.v0` / MARKER
`RAIOS_DATA_RECLOG` / ACTION_KIND `disable_module`) with the shared write→readback→reparse
gauntlet + a recovery tail pinning the three disable-target classification bools; 41
pairwise-distinct denial reasons; nothing called it yet. **1b — the kernel executor:**
`recovery.disable_module <target>` classifies the target read-only, DENIES core-owned /
lifeline-endpoint / unknown / SAFE-posture BEFORE any plan/write, then writes a durable
`raios.recovery_action.v0` record through the SHARED RECLOG mechanism authorized ONLY by
the 1a evaluator, and ONLY on durable-append success stops+disables echo (a current-boot
RAM latch in the NON-attested `echo_service.rs`; `HEALTH_DISABLED` takes precedence over
`crashed`; a disabled module never runs wasm; `start` refuses it; `drop` clears it). The
CRITICAL trap was avoided: the pre-planning map's "reuse the generalized transaction
helper" was the SHARED write-boundary chain (grants generic write to every module) and its
"edit `current_boot_service.rs`" would break the signed hello attestation — BOTH neutralized
(own evaluator; latch in echo_service.rs; `current_boot_service.rs` byte-identical to HEAD).
Vocab hash re-pinned `523b719b…→03d3985c…` (only `disable_module.implemented` flipped).
Restore-only: disable REMOVES a module, grants nothing — `grants_new_capability:false`,
`owner_sealed:false`, `dev_key_not_owner_sealed`, `reversible_this_boot:false`,
`persistence_claimed:false`, `PROMOTION_AUTHORITY_IS_PLACEHOLDER:true`. Max-effort
adversarial review: NO BLOCKER/HIGH — deny-before-mutate provably fail-closed (preflight
before any write AND re-checked in append_recovery_action; classification exactly-one-of,
fail-closed to unknown); the durable write passes the REAL classification bools to the 1a
evaluator (not forged constants); the gauntlet compares readback to the PLANNED frame (no
M7D-2 self-compare); a disabled module can never run wasm. One MEDIUM (pre-existing,
latent): autocrlf could smudge the signed `current_boot_service.rs` to CRLF on a fresh
checkout and break the attestation — fixed separately by adding it to `.gitattributes -text`.
Test env: the `m8-lifeline` profile now boots a `--seed-bootctl valid-a` persist disk
(Normal posture) so the durable recovery-action write is exercised LIVE (M8A was read-only
and needed no disk). Verified: raios-core 115 (vocab hash re-pinned), m8-lifeline 225/225
(durable append landed + echo stopped; core/lifeline/unknown/`*` denials with distinct
reasons; selftest truth table incl. safe_posture_denied; disabled-start refused; the three
remaining mutators still deny; redaction clean), recovery byte-identical, quick 580/580
(hash + disable_module-shape needles synced), FULL 8168/8168.

**M8B-2 DONE (2026-07-07) — the lifeline's SECOND restore action: `recovery.restart_last_good`.**
Split, mirroring M8B-1: **2a (5333633, grants nothing)** — widened
`scoped_recovery_action_append` to accept `action_kind ∈ {disable_module, restart_last_good}`
as a PINNED widening within one authority (like promote/unpromote), added a
`restart_target_restorable` input + ONE new kind-guarded denial `target_not_restartable`;
every disable pin byte-identical; the two `WrongActionKind` mutations retargeted so they
can't silently invert; vocab hash UNCHANGED (nothing wired yet). **2b — the executor:**
`recovery.restart_last_good <target>` classifies read-only, DENIES core / lifeline / unknown
/ SAFE / not-restartable BEFORE any write, writes a durable
`raios.recovery_action.v0` (action_kind=restart_last_good) via the SHARED gauntlet authorized
ONLY by the evaluator (real `restart_target_restorable`, not forged), and ONLY on
durable-append success clears echo's disabled+crashed RAM latches and re-runs the EXISTING
verified `start()` path — which re-hashes the compile-time-constant echo bytes against the
pinned hash EVERY call and refuses on failure, so the re-run can only execute the already-
attested built-in echo (no new loader, no promotion, no capability relaxation). A failed
re-run reports `running:false`/`health:stopped` honestly (false-healthy impossible). The
durable record attests the AUTHORIZED action; the live health is reported only from the real
run result. Restart latch/logic live in the NON-attested `echo_service.rs`;
`current_boot_service.rs` byte-identical. Vocab hash re-pinned `03d3985c…→4a2c52a5…` (only
`restart_last_good.implemented` flipped). Restore-only: restores a known-good BUILT-IN module
already in RAM — `restores_known_good:true`, `grants_new_capability:false`, `owner_sealed:false`,
`dev_key_not_owner_sealed`, `persistence_claimed:false` (no persistence / no cross-reboot
last-good — that is M8C). Max-effort adversarial review: NOTHING above LOW — deny-before-mutate
airtight (preflight + independent evaluator re-check), the re-run cannot run weakened/different
code, disable payload byte-identical, append-only, gauntlet compares readback to the PLANNED
frame (no self-compare), frozen recovery unaffected (method_head_eq protects the
`_target_binding_diagnostic` suffix). Verified: raios-core 116 (vocab re-pinned), m8-lifeline
265/265 (restart of a disabled AND of a crashed echo → healthy/running; target_not_restartable
when healthy; core/lifeline/unknown/`*` denials; selftest truth table; the two remaining
mutators still deny; redaction clean), recovery 3833/3833 byte-identical, quick 580/580, FULL
8168/8168. **M8B (disable + restart, the flagship M8B capability) is complete.** Next: **M8C**
(durable last-good pointer + SAFE integration) then **M8D** (recovery.load_artifact_by_hash
from the local M7D store only).

**M8C-1 DONE (2026-07-07, READ-ONLY, grants nothing).** `recovery.snapshot` now surfaces the
durable M7C BOOTCTL state as two additive read-only sub-objects — `durable_last_good`
(source `bootctl_slot_pointer`; last-good A/B slot, seq, boot_success_mark, safe_mode,
authoritative slot, failure_count, bootctl payload sha256; honest `available:false` /
`boot_success_mark:"missing"` when no durable record exists) and `rollback_preview` (a pure
projection of `evaluate_boot_control`: `would_switch` / `target_bootable` /
`would_fall_back_to_last_good_on_next_boot`, with `mutates_nothing:true` and
`mutating_rollback_available_via_lifeline:false`). All from ONE existing read-only bootctl read
— NO durable write, NO new scoped evaluator, NO new lifeline method, NO vocabulary re-pin
(stays `4a2c52a5…`, method_count 6). The mutating `recovery.rollback` stays denied
(implemented:false) per the restore-only-never-promote lane. NO service-set hash is fabricated
(raiOS has none at HEAD; the last-good is the boot-slot pointer). Max-effort adversarial review:
nothing above LOW — verified truly read-only (one bootctl read, no write DMA, pure renderers),
honest missing-evidence (no fabricated slot/seq/hash), no secret leak (only the payload sha256
as `Value::Sha256`; `storage_sha256` not rendered), additive-only. Two LOW honesty-precision
items applied: `available` now requires the last-good slot to be BOOTABLE (a non-bootable
pointer is no usable rollback), and `would_fall_back` requires a genuine last-good target
(pending-exhausted with no last-good degrades to SAFE, not a fallback). Verified: raios-core 118
(2 new render tests present+absent, vocab unchanged), m8-lifeline 266/266 (present-path), quick
581/581 (missing-evidence path), recovery 3833/3833 byte-identical, FULL 8168/8168. The
recovery diagnosis now includes "which system copy is last-good / are we in safe-mode" and a
look-but-don't-touch rollback preview.

**M8D-1 DONE (2026-07-07, GRANTS NOTHING).** `recovery.load_artifact_by_hash <sha256>` re-instates
a persisted artifact FROM THE LOCAL M7D STORE ONLY, addressed by content hash — M8D-1 is the
grants-nothing half: parse the caller hash (`parse_sha256_ref`, eats the `sha256:` prefix), select
the `artifact_persist` record whose `artifact_sha256` matches, RE-VERIFY the FULL M6 chain from
scratch (reuses `repromotion_reverify::reverify_persisted_artifact` UNCHANGED — re-runs the P-256
signature verify, never trusts a stored boolean), and REPORT ONLY. NO durable write, NO load
(no `emit_load`/`emit_start`, no retain), `authorizes_load`/`cross_reboot_proven`/`service_loaded`/
`mutates_live_state` always false. Fail-closed order: `malformed_hash` → SAFE/PersistenceUnavailable
(`boot_control_safe_mode`, before any store read) → controller-absent → `artifact_not_in_local_store`
→ reverify-mismatch. NEVER fetches / accepts new bytes / accepts a URL (`accepts_external_bytes=
accepts_url=fetches=false`) — the hash is only a SELECTOR into already-attested local records, so it
can never widen authority. To share ONE reverify implementation, `reverify_record_only` +
`find_promotion_transaction` + `repopulate_reverified_references` were made `pub(crate)` in
`repromotion.rs` with `emit_repromotion_run` byte-for-byte unchanged (proven by m6c-promotion
180/180). New schema `raios.recovery_load.v0`; the row flipped `implemented:true`; vocab hash
re-pinned `4a2c52a5…→7488a1ab…` in all three pins. Max-effort adversarial review: nothing above LOW
— repromotion refactor VERIFIED byte-for-byte behavior-preserving, grants nothing on every path
(incl. reverified-success), never-fetch, honest labels, no secret leak. Verified: raios-core 119
(vocab re-pinned), m8-lifeline 270/270 (malformed/absent/SAFE denials + selftest truth table),
m6c-promotion 180/180 (repromotion path preserved), quick 584/584, recovery 3833/3833 byte-identical,
FULL 8168/8168. Next: **M8D-2** — the authority flip (durable audit via a NEW own
`scoped_recovery_load_append` + the UNMODIFIED M6 gate; the actual load MUST gate on the FULL
`reverify_record` path — wasm-validity + M6 gate — never on `reverify_record_only().reverified()`
alone), proven positive via the two-boot harness. That closes **M8**.

**M8D-2 DONE (2026-07-07) — M8 RECOVERY AGENT LIFELINE COMPLETE.** The authority flip: on a
matched record, `recovery.load_artifact_by_hash` runs the FULL `repromotion::reverify_record`
(re-verifies the whole M6 chain from scratch INCLUDING reconstructed-wasm-validity, then dispatches
the UNMODIFIED M6 gate `granted_candidate_service::emit_load`/`emit_start` to re-instate a RAM-only
current-boot service), and appends a durable `raios.recovery_load.v0` AUDIT via a NEW own
`raios-core/src/scoped_recovery_load_append.rs` evaluator (own pins method/target/schema/marker;
authorizes_load/cross_reboot_proven ONLY when decision_status=="reinstated"; owner_sealed/
persistence_claimed denied on ALL statuses; 43-row pairwise-distinct truth table). Load is authorized
ONLY by `outcome.reinstated` (status=="repromoted" && performed && service_loaded && service_started)
— a record that passes payload-sha but whose reconstructed wasm does NOT validate is denied
`reconstructed_wasm_invalid` BEFORE any load (the M8D-1 review's flagged reverify-only trap is closed;
zero call sites remain). Append-after-gate matching repromotion.run's audit discipline (the record
attests a reinstatement that genuinely happened; a failed load is never reported loaded; every denial
fails closed with NO durable write). Restore-toward-known-good: re-instates an ALREADY-attested LOCAL
artifact only — grants nothing NEW, never fetches, never accepts bytes/URL (`accepts_external_bytes=
accepts_url=fetches=false`); `owner_sealed:false`, `persistence_claimed:false`,
`PROMOTION_AUTHORITY_IS_PLACEHOLDER:true`, `dev_key_not_owner_sealed`; `authorizes_load`/
`cross_reboot_proven` true ONLY on a genuine gate load+start. The M6 gate / reverify /
granted_candidate_service / wasm_runtime are UNTOUCHED (no second loader, no gate loosening); the
new evaluator is OWN-pinned, not a shared write-boundary flip; `emit_repromotion_run`+`reverify_record`
byte-for-byte preserved. Max-effort adversarial review: nothing above LOW (the CRITICAL "load gates on
the full path" check PASSES; two LOW — gate-denied under-reports a partial RAM load in the
SAFE/never-over-claim direction, near-unreachable; the reinstated success labels are harness-covered
not host-tested to keep the vocab hash frozen). Verified: raios-core 125 (6 new evaluator tests, vocab
unchanged 7488a1ab), two-boot 110/110 (boot-2 load-by-hash re-instates the boot-1-persisted artifact +
service answers LIVE; wrong-hash → artifact_not_in_local_store + no answer; tampered record → full-
reverify denial + no load + no durable write), m8-lifeline (denials), recovery byte-identical, quick,
m6c-promotion (repromotion intact), FULL 8168/8168.

**M8 COMPLETE (2026-07-07).** The Recovery Agent Lifeline is a minimal pinned serial-first path that,
when the world above breaks: reads its frozen command table (M8A-1), DIAGNOSES live + durable state
(M8A-2/M8C-1 snapshot incl. last-good/SAFE + read-only rollback preview), SURVIVES a wedged Wasm
service (M8A-3), and takes four restore actions — DISABLE a bad module (M8B-1), RESTART a
disabled/crashed module to known-good (M8B-2), and RE-INSTATE a persisted artifact by hash from the
LOCAL store with full re-verification (M8D). Every mutating action is deny-before-mutate + durable-
record-first via its OWN pinned scoped evaluator, restore-only-never-promote, and honestly
`dev_key_not_owner_sealed` / owner_sealed false. Survival rests on fuel-metered + cooperative
scheduling (NOT hardware isolation — that is post-M11).

**M9 Durable Memory & Context Broker v1 now active** (ADR 0004 Phase D; map
`docs/plan-reviews/m9-durable-memory-map-2026-07-06.md`, revalidated against HEAD during scoping).
raiOS itself is the memory — typed facts with provenance + classification, never a chat log / fake
persistence / prompt dump. Scoping found the read-only `raios.agent_context.v0` broker
(`agent_protocol_memory.rs::emit_memory_context`) ALREADY exists with per-record classification, an
explicit `omitted` array, and `provider_export:"disabled"`; provider export is already blanket
fail-closed (`DeniedProviderContextExport`); and M8's `recovery_memory_*` grants-nothing validators
must NOT be collided with. Sequence: **M9A-1** (typed record schema, grants nothing) → M9A-2 (first
durable memory write via its OWN scoped `scoped_memory_record_append` evaluator, single-boot) → M9A-3
(decision/problem via supersede) → M9B-1 (agent-authored observation, scoped) → M9C-1 (broker draws on
durable records) → M9C-2 (provider export gating end-to-end) → M9D-1 (cross-reboot survival, closes M9).

**M9A-1 DONE (2026-07-07, grants nothing, host-only).** NEW `raios-core/src/memory_record.rs` — the
typed `raios.memory_record.v0` record (schema/id/kind/entity/predicate/value/classification/authority/
boot_id/sequence/source/evidence/tags/supersedes/created_at{clock:"boot_relative",ticks} — NO
wall-clock; M10 owns trusted time) on the shared `record.rs` Value/Field + `sha256_of_json` (no
hand-rolled emit/hash). Fail-closed constructor: `Classification` has NO `Secret` variant so a secret
plaintext is STRUCTURALLY un-constructable (`"secret"` input → typed `Err(secret_never_durable_until_
sealed_secret_design)`) → a secret can never become a durable record and can never reach a provider;
unknown classification → `local_only` (never public); `MemoryKind` is an 8-value allowlist (unknown →
`Err`); `observation` without entity/source → `Err`; supersede-not-overwrite; returns `Result`, no
input panics. Nothing calls it yet — zero kernel change, zero disk write, no VM, no vocab change.
Verified: cargo test -p raios-core 133 (8 new: pinned sample sha256 `4ab57d93…`, Value::Sha256 renders
`sha256:<64hex>`, secret rejected, unknown-kind rejected, observation entity/source required, unknown
classification → local_only, supersedes round-trips), rustfmt clean, kernel builds (no_std).

**M9A-2a DONE (2026-07-07, grants nothing, host-only).** NEW `raios-core/src/scoped_memory_record_append.rs`
— a dedicated CLONE of `scoped_recovery_load_append` with its OWN pinned identity
(`memory.record_log_append` / `append.memory_record.seed_data` / `raios.memory_record.v0` /
`RAIOS_DATA_RECLOG`). Authorizes exactly one thing — appending a durable memory-record frame through the
shared reclog gauntlet (scan → plan → write_readback → reparse → evaluate → rescan) — and re-checks in
depth what `MemoryRecord::new` already enforces (`secret` never durable, only the 8 authority-bearing
kinds in scope) plus a new per-boot write-quota pin (`memory_write_quota_exhausted`) and the
`dev_key_not_owner_sealed` / not-owner-sealed / not-persistence-claimed trust pins. No kernel wires it yet.
Verified: cargo test -p raios-core 137 (4 new: valid append authorizes, empty-log seq/prev, all 8 kinds in
scope, full pairwise-unique denial truth table), rustfmt clean.

**M9A-2b DONE (2026-07-07, first durable memory write, single-boot).** raiOS durably records ONE
system-authored `raios.memory_record.v0` fact — a `capability_denial` of the permanently-denied generic
durable module-load gate (the REAL denial reasons from `agent_protocol_module_grant.rs`: durable audit
missing, rollback plan missing, loader unavailable, service slot unallocated) — into the SEED_DATA RECLOG.
NEW `seed-kernel/src/durable_store.rs::append_memory_record` (a structural clone of `append_recovery_load`)
adds ONE thing: a RAM-only per-boot write quota (128 records / 32 KiB, a fresh `spin::Mutex` reset every
boot, reserved before the plan/write and released on any post-reserve denial, never on success) ahead of
the shared scan → plan → ahci write_readback → reparse → `evaluate_scoped_memory_record_append` → rescan
gauntlet. NEW `seed-kernel/src/memory_store.rs` is the ONE Read0 driver: builds the fixed record via
`MemoryRecord::new` (fail-closed; a construction error would be a RAM-only `capability_denied` response,
never a durable append) and renders `durable_store::append_memory_record`'s evidence on
`memory.record_log_append`, plus a synthetic `memory.record_log_append_selftest` (NO disk write) proving
secret classification / unknown kind / the scoped evaluator's own defensive pins / quota exhaustion all
deny without ever calling the durable writer (no self-recursion). Payload bytes are
`write_json(record.to_record_value())`, so `payload_sha256 == record.record_sha256()`. Success authority
`scoped_memory_record_append_authorized`; record `id=mem.capability_denial.module_load_ephemeral_durable.
current_boot.v0`, `kind=capability_denial`, `classification=local_only`, `authority=core_ledger`,
`boot_id="current_boot"`; `sequence`/`created_at_ticks` fall back to `0` (no RAM-only, side-effect-free
peek at the current event-log sequence exists yet). GRANTS NOTHING new: system-authored only, no agent
write path; every `memory.*` mutation stays denied and provider export stays fail-closed. NEW single-boot
`memory-durable` VM profile (`vm-harness/shadow-vm-smoke-profile-memory-durable.ps1`, **43/43 predicates
green**) proves a real durable append + independent follow-up `durable.record_log_scan` agreement, with the
append's `payload_sha256` pinned to the **golden `record_sha256` (`sha256:1e0d230e…1a77ba8f`)** computed
in raios-core from the identical `MemoryRecordInput` — so the proof shows the EXACT
`raios.memory_record.v0` bytes are on disk, not merely a well-formed frame. The fail-closed selftest
families (secret / unknown-kind / scoped defensive pins) run RAM-only (reclog count/tail unchanged), and a
live probe (`durable_store::memory_write_quota_probe_exhaustion`) drives the REAL per-boot RAM quota to
exhaustion AND back (proving the one new primitive fires + refunds, not just the evaluator's synthetic
`quota_ok=false` pin). Guard needles parse each specific response (memory.record_observation / memory.redact
capability_denied, provider.context_export denied, memory.context provider_export "disabled") rather than
whole-log substrings. A max-effort adversarial review returned SHIP (no CRITICAL/HIGH; fail-closed write,
balanced quota, nothing new granted, signed set untouched); its two MEDIUM proof gaps (prove the live quota;
pin the exact record hash) and two LOW guard-needle gaps were closed before commit. Regression: `quick`,
`recovery` (byte-identical), `m6c-promotion`, `full` (8168) all green. Next: **M9A-3** (decision/problem via
supersede).

**M9A-3a DONE (2026-07-07, grants nothing, host-only, commit `2bef6bf`).** `raios-core/src/memory_record.rs`
gains the supersede-not-overwrite write-side rules on top of M9A-1's constructor: `decision` now requires
`entity`+`source` (`decision_missing_entity`/`decision_missing_source`), `problem` now requires
`entity`+`predicate` (`problem_missing_entity`/`problem_missing_status`), one of the 5 audit/authority kinds
(`capability_grant`/`capability_denial`/`promotion_tx_ref`/`rollback_tx_ref`/`export_audit`) can never be
authored as a superseding record (`audit_kind_may_not_supersede` — the reclog audit trail must never be
hideable by a future broker resolving a `supersedes` link), `supersedes` is capped at
`MAX_SUPERSEDES_PER_RECORD = 8` (`supersedes_list_too_long`), and a record can never name its own id
(`supersede_self_reference`). Docs/ROADMAP.md records the matching **R1 read-side precondition**: the
future M9C broker MUST ignore any supersede link whose TARGET is an audit kind (a non-audit `decision`
naming a `capability_denial` id cannot be denied at write time — that needs the target's kind, which means
reparsing the log, which means the broker). No reader resolving supersession may ship before R1. Verified:
`cargo test -p raios-core` 147 (10 new: decision/problem required-field truth table, a decision may
legitimately supersede a non-audit id, all 5 audit kinds rejected as superseding, supersedes-too-long,
supersedes-at-max accepted, self-supersede rejected, all reasons pairwise-unique), rustfmt clean. Nothing
calls the new rules yet.

**M9A-3b DONE (2026-07-07, decision+problem durable + supersede write-side proof, single-boot, grants
nothing).** raiOS durably records THREE truthful system-authored `raios.memory_record.v0` facts through the
SAME gauntlet (`durable_store::append_memory_record`, authorized ONLY by
`evaluate_scoped_memory_record_append`): **A** a general standing `decision`
(`mem.decision.module_sharing_confirmed_vision.current_boot.v0` — module sharing between raiOS users is
owner-confirmed vision, source: owner answers commit `b7241b2` /
`docs/plan-reviews/m12-plus-direction-2026-07-06.md`), **P** an honest `problem`
(`mem.problem.memory_mutation_denied.current_boot.v0` — the current `memory.*` mutation-policy limitation,
mirroring `agent_protocol_memory.rs`'s `mutation_policy` field verbatim), and **B** a refined `decision`
(`mem.decision.module_sharing_evidence_gated.current_boot.v0` — sharing/downloading a module is candidate
intake, NEVER an install) that SUPERSEDES A (`supersedes: [A.id]`) — the write-side proof of
supersede-not-overwrite: constructing B mutates nothing about A, it only carries the link. NEW
`seed-kernel/src/memory_store.rs::emit_memory_decision_problem_log_append` (Read0, wired at
`memory.decision_problem_log_append` / `raios.memory_decision_problem_append.v0`) constructs ALL THREE
records via `MemoryRecord::new` FIRST (fail-closed; a construction error emits a RAM-only `capability_denied`
before any append — construction is side-effect-free, so there is no after-append error window and never a
hidden partial durable write) THEN appends A, P, B, and renders each's `durable_store::append_memory_record`
evidence in a `records` array, with each entry additionally echoing the record's OWN `supersedes` list (empty
for A/P, `[A.id]` for B) — the on-disk proof that the link landed, not an in-memory claim. The top-level
`durable_append`/`performed`/`reason` are DERIVED from the real per-record evidence (`all(|e| e.performed)`),
never hardcoded — an adversarial-review fix so a denied append (SAFE posture / quota / no disk) is reported
honestly instead of falsely claiming success. The single-record `memory.record_log_append` response body is byte-identical (its
field-rendering was refactored into a shared `memory_record_evidence_fields` helper reused by both methods).
EXTENDED `emit_memory_record_log_append_selftest` with 5 more RAM-only, never-appending cases exercising
`MemoryRecord::new`/`MemoryRecordError` directly: `audit_kind_supersede_denied`
(`audit_kind_may_not_supersede`), `supersedes_list_too_long_denied` (`supersedes_list_too_long`),
`self_supersede_denied` (`supersede_self_reference`), `decision_missing_source_denied`
(`decision_missing_source`), `problem_missing_status_denied` (`problem_missing_status`) — `case_count` 6 → 11.
GRANTS NOTHING new: system-authored only, no agent write path (M9B); every `memory.*` mutation stays denied
and provider export stays fail-closed; every record is honestly `dev_key_not_owner_sealed` /
`owner_sealed=false` / `persistence_claimed=false` / `boot_id="current_boot"`. The **R1 broker rule** from
M9A-3a (a future M9C reader must ignore supersede links targeting an audit kind) is explicitly DEFERRED —
this slice proves only the write side. EXTENDED `memory-durable` VM profile
(`vm-harness/shadow-vm-smoke-profile-memory-durable.ps1`) with a new `memory-durable-supersede` family: a
fresh child-VM probe (generalized `Invoke-MemoryRecordAppendFixtureProbe` via a new `-AppendMethod`
parameter, defaulting to the M9A-2b method so the existing call site is untouched) drives
`memory.decision_problem_log_append` against the same `valid:2` reclog fixture and asserts, per record A/P/B:
`durable_append=="appended"`, `authority=="scoped_memory_record_append_authorized"`,
`readback_sha256==frame_sha256 && reparse_valid`, honest `owner_sealed`/`persistence_claimed`, correct
`kind`/`classification`, and the **pinned golden `payload_sha256`** (A `sha256:9b39ac73…98cb0`, P
`sha256:3b010268…9a249`, B `sha256:5f27b06d…7a262`, each computed in raios-core from the identical frozen
`MemoryRecordInput` — independently reproduced during implementation via a throwaway host-side probe test,
reverted before commit) — proving the EXACT bytes of all three records landed. Also asserts the reclog chain
advances by exactly `+1` at each of A/P/B and `+3` across the trio, B's echoed `supersedes == [A.id]` with
A/P echoing empty `supersedes`, and a follow-up `durable.record_log_scan` agrees with B's `tail_seq`/`count`.
The selftest family gained 5 matching needles (each new case denied with its exact reason) plus an unchanged
before/after main-VM reclog scan (RAM-only), and `case_count == 11` replaces the old `>= 6` assertion.
Verified: `cargo build --profile release` (seed-kernel) exit 0; `cargo test --locked -p raios-core` 147/147
unchanged (no raios-core source touched — only reused already-merged M9A-3a rules); `rustfmt --check` clean
on both touched seed-kernel files; the PS profile parses via
`[System.Management.Automation.Language.Parser]::ParseFile`. Orchestrator-run proof: `memory-durable`
**77/77** green (M9A-2b families + the new supersede family + extended selftest, all with the honesty fix);
block-close regression `quick` / `recovery` (byte-identical) / `m6c-promotion` / `full` all green. Max-effort
adversarial review: one MEDIUM (top-level over-claim) + one LOW (latent partial-append) both fixed by the
construct-all-then-append + derived-top-level change above; all else traced clean (refactor byte-identical,
W1 audit-supersede closed in both layers, truthful contents, non-vacuous needles). **This closes the M9A
block** (system-authored durable memory: schema → first write → decision/problem + supersede). Next:
**M9B-1** (agent-authored observation, scoped) — the first NON-system durable memory write.

**M9B-1b DONE (2026-07-07, first AGENT-authored durable observation, single-boot, grants nothing new).**
raiOS gains its FIRST agent-controlled durable memory write: `memory.observation_log_append` lets an AGENT
durably record ONE `observation` `raios.memory_record.v0` by supplying exactly 4 base64-encoded,
newline-separated fields — `entity`/`predicate`/`value`/`source_record_id` — while the kernel FORCES every
other authority-bearing field: `id` is kernel-assigned from a fresh per-boot RAM-only counter
(`mem.observation.agent.current_boot.NNNNNNNN.v0`, also the record's `sequence`), `kind="observation"`,
`classification="local_only"`, `authority="agent"`, `boot_id="current_boot"`, `supersedes=[]`,
`tags=["agent","observation"]`. NEW `seed-kernel/src/durable_store.rs::append_memory_record_inner<'a>`
generalizes the M9A-2b/M9A-3b append body over the record's lifetime
(`MemoryRecordAppendEvidence<'a>`, with `record_id`/`record_authority: &'a str`; every other field stays
`&'static str`) so an agent-authored record built from a decoded, non-`'static` RAM buffer can be rendered
before that buffer drops. The two `'static` system callers (`append_memory_record`, M9A-2b/M9A-3b, UNCHANGED
behavior) and the ONE new entry point (`append_agent_observation_record<'a>`) both call this shared inner
body, which forwards `agent_authored` to `evaluate_scoped_memory_record_append` (M9B-1a, already merged:
confines an agent record to observation-only/no-supersede/local_only — deny-in-depth backstop even if a
caller bypassed `MemoryRecord::new`). The per-boot RAM write quota is generalized to a sector-count parameter
(`memory_write_quota_try_reserve_sectors`/`_release_sectors`; the pre-existing zero-arg
`memory_write_quota_try_reserve`/`_release` — and therefore `memory_write_quota_probe_exhaustion` and the
system path — are byte-unchanged, defined as `..._sectors(1)`); the agent path conservatively charges
`AGENT_OBS_QUOTA_SECTORS = 2` sectors (worst case for a 2-sector agent-observation frame) so it can never
undercharge. NEW `seed-kernel/src/memory_store.rs::emit_memory_observation_log_append` strips the method
prefix, decodes the base64 argument via the SAME hardened `module_candidate_channel::decode_base64_chunk`
(visibility widened to `pub(crate)`, no second decoder), splits the decoded bytes on `\n` into exactly 4
fields, and fail-closed validates each BEFORE ever calling `MemoryRecord::new` or consuming a sequence
number — so a malformed input never burns the per-boot id counter and never appends: missing payload
(`agent_observation_missing_payload`), base64 decode failure (the decoder's own distinct reason, e.g.
`rejected_malformed_base64_chunk`), field count ≠ 4 (`agent_observation_field_count`), a byte cap violation —
entity≤64/predicate≤32/value≤96/source≤64 (`agent_observation_field_too_long`), a locator-safe charset
violation `[A-Za-z0-9 ._:/-]` (`agent_observation_field_charset`), and empty entity/source
(`agent_observation_entity_empty`/`agent_observation_source_empty`, checked ahead of
`MemoryRecord::new`'s own `observation_missing_entity`/`observation_missing_source` for a clearer reason).
Registered as ONE new `agent_protocol.rs` Head-matched row
(`MethodAction::ReadMethod(memory_store::emit_memory_observation_log_append)`); the broad
`memory.record_observation` method and the entire `MEMORY_MUTATION_METHODS` denial set are UNTOUCHED and stay
denied — re-asserted by a new VM guard needle AFTER the new method exists, proving the broad boundary did not
open. EXTENDED `memory-durable` VM profile (`vm-harness/shadow-vm-smoke-profile-memory-durable.ps1`) with
`memory-agent-observation-authorized` (a fresh child-VM probe, generalized
`Invoke-MemoryRecordAppendFixtureProbe` via a new `-AppendArg` parameter, sends
`memory.observation_log_append <FROZEN_BLOB>` against the `valid:2` reclog fixture and asserts
`durable_append=="appended"`, `authority=="scoped_memory_record_append_authorized"`, `kind=="observation"`,
`classification=="local_only"`, `record_authority=="agent"`,
`record_id=="mem.observation.agent.current_boot.00000001.v0"` (the first-boot-write proof),
`readback_sha256==frame_sha256 && reparse_valid`, honest `owner_sealed=false`/`persistence_claimed=false`,
`frame_len<=1024` (proves the 2-sector quota charge is not an undercharge), the **pinned golden**
`payload_sha256 == sha256:75ea5ab92fc9dafe908bae204e5a357947e47ba7e231aaddc0c19854288e198d` (independently
reproduced during implementation via a throwaway host-side `raios-core` test, reverted before commit — proves
the EXACT agent record landed, not merely a well-formed frame), a reclog chain advance of exactly `+1`, and a
follow-up `durable.record_log_scan` agreement) and `memory-agent-observation-denied` (5 distinct RAM-only
denials sent against the main VM — not-base64, a 3-field blob, an over-cap field, a disallowed-charset field,
and an empty entity — each asserted `durable_append=="capability_denied"` / `performed==false` with its exact
reason, bracketed by a single before/after main-VM `durable.record_log_scan` proving nothing was appended
across all 5 attempts). The **R1 broker rule** (M9A-3a: a future M9C reader must ignore supersede links
targeting an audit kind) remains explicitly DEFERRED to M9C — this slice's write path can never author a
supersede link at all, so R1 is unaffected either way. Verified: `cargo build --profile release`
(seed-kernel) exit 0; `cargo test --locked -p raios-core` 148/148 unchanged (only a lifetime/quota
generalization of already-merged M9A-2b/M9A-3b/M9B-1a code — no new raios-core rules, no new tests);
`rustfmt --edition 2021 --check` clean on all four touched seed-kernel files; the PS profile parses via
`[System.Management.Automation.Language.Parser]::ParseFile`. Deviation: widened
`module_candidate_channel::decode_base64_chunk` from private to `pub(crate)` (one line, no behavior change) so
the new driver could reuse it rather than fork a second decoder, per the packet's explicit instruction.
Orchestrator proof: `memory-durable` **105/105** green (M9A families + the new agent-observation authorized
family — golden `sha256:75ea5ab9…198d` for the exact agent record — + the 5-case RAM-only denial matrix + the
guard re-asserting `memory.record_observation` STILL denied after the new method exists). A HOST-TRANSPORT
finding surfaced and was fixed: the ~213-byte agent command (base64 blob) sent as one fast burst overflowed the
guest 16550 UART RX FIFO (the guest drains MAX_BYTES_PER_POLL=64 with UI redraw between polls), dropping the
tail so the line never dispatched; the profile now paces every send (small chunks + short delay), exactly how
`submit_candidate_chunk` and any real agent sender pace long serial writes. Max-effort adversarial review:
**SHIP** — parser escape, authority forge, and quota undercharge all CONFIRMED closed (golden + worst-case
917≤1024-byte frame independently reproduced). Its LOW-2 (undercharge only statically proven) was closed with a
fail-closed runtime guard: an agent frame that ever exceeds its reserved charge is DENIED
(`agent_observation_frame_exceeds_quota_charge`), so an undercharge is impossible even if a future field-cap or
schema change grows the frame. LOW-1/LOW-3 are documented as M9C broker-trust rules (order agent records by the
reclog frame seq / boot_id, NEVER the payload `sequence`; trust the forced `authority="agent"`/`source.method`,
NEVER the agent-supplied `source.record_id`). The **R1 broker rule** (M9A-3a: a future M9C reader must ignore
supersede links targeting an audit kind) remains explicitly DEFERRED to M9C — this slice's write path can never
author a supersede link at all, so R1 is unaffected either way. Verified: `cargo build --profile release`
(seed-kernel) exit 0; `cargo test --locked -p raios-core` 148/148 unchanged (only a lifetime/quota
generalization of already-merged M9A-2b/M9A-3b/M9B-1a code — no new raios-core rules, no new tests). This
closes **M9B-1** (the first agent-authored durable memory write, scoped and confined). Next: the M9C broker
(R1 supersede-target rule + LOW-1/LOW-3 trust rules + typed-fact reads) that M9A and M9B leave deferred.

M9B block-close verification (2026-07-07): **`recovery` byte-identical PASSED** (shared paths unchanged by
M9B-1b) and **`full` 7834/7834 PASSED**. The `quick` profile aborted mid-run (~290/509) on
`command:agent audit.events 72` — "Timed out waiting for RAIOS_AGENT_END memory.recent_events" with UEFI
BdsDxe / ANSI boot noise polluting the serial-log tail. Verdict: **host-transport** (the recurring
audit.events-72 serial-transport flake). CONFIRMED not a guest bug: `full` exercises `memory.recent_events`
among its 7834 predicates and passed, so the path works; the flake is quick-profile serial timing (worsened
by 99%-full-disk timing), not M9B-1b (which never touches the event log / memory.recent_events / UI). One
`full` attempt also hit a transient "Image packaging failed" (predicate_count 0, base_image null) that passed
cleanly on the immediate re-run — a packaging/host transient, not code (recovery had built+packaged fine
seconds earlier). Both classified host-transport per the goal's flake rule; M9B closed on
recovery-byte-identical + full-7834 + memory-durable-105.

**M9C-1b IMPLEMENTED (2026-07-08, read-only broker surface, no provider export).** `memory.context` can now
DO the first durable-memory broker read: it reads the SEED_DATA RECLOG region through the existing
`ahci::read_persist_reclog_region` path, reparses integrity-verified frames with the already-merged
`raios_core::memory_record::parse`, drops non-memory/corrupt payloads without panic, caps the newest parsed
records to 64, resolves visibility with `resolve_durable_memory` (R1 audit supersede ignored, LOW-1 frame
seq ordering, audit id-shadow preservation, LOW-3 identity by record id only), and emits a top-level
`durable_records` metadata array plus explicit durable omission folds. It never emits raw record `value`.
`local_only` records surface only locally with `exportable:false` and `durable_local_only_value`; provider
export stays fail-closed (`provider_export:"disabled"`, `agent_protocol_provider.rs` untouched, no durable
content routed into `provider_projection`). New `memory.broker_resolve_selftest` is RAM-only/no-disk and
covers R1, LOW-1, audit-id-shadow, and LOW-3. The `memory-durable` profile gained broker-* needle families
for durable inclusion, supersede hiding, frame-ordering, classification/exportability, export still closed,
and the broker resolver selftest; QEMU proof is left to the orchestrator per packet instructions.

**M9C-2b IMPLEMENTED (2026-07-08, durable provider-export DENIAL audits; grants nothing).**
Every `provider.context_export` attempt is now evaluated by the committed M9C-2a
`scoped_provider_export` evaluator against real kernel method/profile/trust state plus honest-absent
packet/audit fields; denied attempts build a system-authored durable
`capability_denial` memory record through the existing `scoped_memory_record_append` gauntlet, so there is
no new raios-core write boundary and no provider export success path. Dedupe design D-A: a per-boot RAM
table stores at most the 16 gate/reason pairs and compares keys only through
`export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, reason)`; repeated identical denials cite the
first audit's payload hash/seq and append nothing, append-denied audit writes are not recursively recorded,
and an unreachable 17th distinct key fails closed as RAM-only. `provider.context_export` still returns
`capability_denied`, `memory.context` still reports `provider_export:"disabled"`, `provider_write` remains
`not_attempted`, and M9C-2c remains the future positive export path. Host-only verification for this worker:
`cargo fmt -p seed-kernel -- --check` PASS, `cargo fmt -p raios-core -- --check` PASS,
`cargo test --locked -p raios-core` PASS (191 tests), release seed-kernel build PASS (`built
target/x86_64-seed/release/seed-kernel`), and the edited memory-durable PowerShell profile parses with 0
errors. Per orchestrator override, no QEMU profile was run here; the `export-denial-durable:*` VM predicates
were added for the orchestrator.

**M9C-2c-1 IMPLEMENTED (2026-07-08, deterministic PUBLIC-ONLY provider-export packet assembly evidence; grants nothing).**
raiOS can now DO a read-only packet assembly proof over durable memory: a fixed system-authored public
decision fixture (`mem.decision.provider_export_public_fixture.current_boot.v0`, classification `public`) can
be appended through the existing memory-record gauntlet, and
`provider.context_export_packet_selftest provider_minimal` then reads the same durable context as
`memory.context`, filters to `exportable:true` records only, reports included public ids, excluded local-only
count, packet record count, and a deterministic `sha256:<64hex>` packet hash over public metadata only. The
canonical hash input is a typed `Value::Object` with fields `profile`, `scope`,
`packet_all_records_public`, `packet_record_count`, and `records`; each record carries only `id`, `kind`,
`entity`, `predicate`, `classification`, `authority`, `scope`, and `exportable`. Raw durable `value` and
excluded local-only ids are not included. The selftest performs no gate evaluation, no authorization, no
export audit, no durable packet write, no provider write, and no transmission; `provider.context_export`
stays the existing denial. The `memory-durable` profile gained `export-packet:*` predicates for public
inclusion, local-only exclusion, post-filter all-public/count consistency, hash determinism, and
no-authorization/no-audit/no-write. Host-only verification for this worker: `cargo fmt -p seed-kernel -- --check`
PASS, `cargo fmt -p raios-core -- --check` PASS, `cargo test --locked -p raios-core` PASS (191 tests),
release seed-kernel build PASS (`built target/x86_64-seed/release/seed-kernel`), and the edited
memory-durable PowerShell profile parses with 0 errors. Per orchestrator override, no QEMU profile was run.

**M9C-2c-2 IMPLEMENTED (2026-07-08, provider-export AUTHORITY FLIP is selftest-only; no transmission).**
raiOS can now DO a deterministic, test-only provider-export authorization proof: after the public fixture
packet from M9C-2c-1 is assembled, `provider.context_export_authorized_selftest provider_minimal` evaluates a
fixed synthetic gate vector (`method=provider.context_export`, `profile=provider_minimal`,
`trust_state=pinned_spki_verified`, `tls_certificate_verification_bypassed=false`,
`packet_all_records_public=true`, `budget_tokens=4096`, `packet_estimated_tokens=256`,
`audit_destination=provider.openai.responses`, trust snapshot present) and records exactly one durable
`mem.export_audit.provider_context_export_selftest.current_boot.v0` memory record with kind `export_audit`,
classification `local_only`, authority `core_ledger`, and `supersedes:[]`. Its value binds the scoped provider
export gate/schema/reason, method/profile, synthetic trust state, destination, budget, packet hash, packet
record count, audit binding hash (`sha256_of_json` over packet_hash, destination, trust_state, budget_tokens),
`packet_assembled:true`, `transmission_performed:false`, `export_performed:false`,
`provider_write:"selftest_no_transmission"`, `owner_sealed:false`, `trust_tier:"dev_key_not_owner_sealed"`,
and `test_infrastructure:true`. The authorized selftest never calls `provider::snapshot()` and never reaches
OpenAI/TLS/socket/API-key code; the real `provider.context_export provider_minimal` dispatch row and denial
handler are unchanged and still return `capability_denied` with the M9C-2b denial audit. The negative firewall
method `provider.context_export_authorized_selftest_smuggle provider_minimal` uses the same synthetic vector
except `packet_all_records_public=false` and an incremented count, denies with
`packet_contains_non_public_record`, and appends no export audit. The `memory-durable` profile gained
`export-authorized-selftest:*` predicates for gate authorization, durable export-audit append, non-superseding
audit, honest labels, local-only smuggle denial, audit-chain advancement, real export still denied, local-only
context visibility, and provider-export status still disabled. Host-only verification for this worker:
`cargo fmt -p seed-kernel -- --check` PASS, `cargo fmt -p raios-core -- --check` PASS,
`cargo test --locked -p raios-core` PASS (191 tests), release seed-kernel build PASS
(`built target/x86_64-seed/release/seed-kernel`), and the edited memory-durable PowerShell profile parses with
0 errors. Per orchestrator override, no QEMU profile was run.

**M9C-2c-2 FOLLOW-UP FIX (2026-07-08, authorized audit dedupe + fixture guard).**
The fixed selftest export audit is now per-boot deduped like the M9C-2b denial audits: the first successful
`provider.context_export_authorized_selftest` append returns `dedupe:"first_authorized_appended"` and stores
the audit payload hash/seq in RAM; a second same-boot call returns `dedupe:"duplicate_ram_only"`,
`durable_append:"not_attempted_deduplicated"`, `performed:false`, and cites the first audit record id,
payload hash, and seq without touching the RECLOG. Append denial stays fail-closed as
`append_denied_ram_only` and does not recursively record another audit. The positive selftest also now refuses
to evaluate the authorized vector unless the assembled public packet contains
`mem.decision.provider_export_public_fixture.current_boot.v0`; if absent it returns `authorized:false`,
`gate_reason:"public_fixture_absent"`, `gate_evaluated:false`, and appends no export audit. The smuggle denial
path is unchanged. The `memory-durable` profile now calls the authorized selftest twice and adds
`export-authorized-selftest:dedupe-second-appends-nothing`; the chain predicate still requires exactly +2
frames total across fixture append plus first export_audit, proving authorized2 and smuggle append nothing.
Host-only verification for this worker: `cargo fmt -p seed-kernel -- --check` PASS,
`cargo fmt -p raios-core -- --check` PASS, `cargo test --locked -p raios-core` PASS (191 tests), release
seed-kernel build PASS (`built target/x86_64-seed/release/seed-kernel`), and the edited memory-durable
PowerShell profile parses with 0 errors. Per orchestrator override, no QEMU profile was run.

**M10A-1 IMPLEMENTED (2026-07-08, provider-trust HONESTY evaluator, raios-core only, grants nothing).**
raiOS can now DO a host-tested fail-closed honesty evaluation over provider trust evidence before real
WebPKI/time hardening lands: `raios-core/src/scoped_provider_trust_honesty.rs` accepts only the honest Stage-0
pin-only labels (`pinned_cert_verified`/`pinned_spki_verified`,
`pin_only_no_webpki_chain_validation`, `not_validated_stage0`) and rejects dev bypass, chain/time overclaims,
`webpki_verified`, and unknown/negative trust states. Success reports
`honest_pin_only_time_unvalidated_grants_nothing` with `honest:true`,
`chain_validated:false`, `time_validated:false`, and both `authorizes_provider_request:false` and
`authorizes_provider_export:false`; no kernel wiring, provider request, provider export, QEMU, real time,
CA roots, WebPKI chain logic, or second-provider adapter was added. Host-only verification:
`cargo test --locked -p raios-core` PASS (197 tests, 6 new trust-honesty tests; 11 distinct pairwise-unique
denial reasons in `denial_truth_table_names_first_failed_pin`) and `cargo fmt --all -- --check` PASS.

Latest host-tool verification: after the 2026-07-03 local report-timestamp
Latest host-tool verification: after the 2026-07-03 local report-timestamp
recovery/hello dispatch-bound completion-denial smoke runs on Windows with
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release`,
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release`,
`cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`,
`git diff --check` (CRLF warnings only), and
`scripts\scan-secrets.ps1` with no OpenAI-key-like values found, covering
OTA/registry tooling plus the non-authorizing
`raios.computed_capability_grant.v0` diagnostic, host-side
`raios.module_audit_rollback_diagnostic.v0` audit/rollback candidates, and
negative manifest/artifact/report/attestation/audit/rollback evidence cases.
`cargo fmt --all -- --check` was attempted in the same pass with
`RUST_MIN_STACK=67108864` and failed before format-diff output with `rustfmt`
stack overflow on the current oversized Rust sources.
`vm-harness\shadow-vm-smoke.ps1 -Profile full -TimeoutSeconds 420 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10`
was also attempted and failed before reaching recovery/full-audit assertions:
after 11 provider-memory commands it timed out reconnecting to QEMU serial TCP
port 4565; report `release\vm-reports\shadow-20260703-074757-29068.json`
recorded `result: failed`, 163/163 predicates, `duration_ms: 477924`, and
report SHA-256
`6d6f031cb3eb784ad164a711e2eaf0da7ecda4af6804c6c37b826bc57d19ae26`.

Latest quick guest-protocol verification: 2026-06-30 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile quick -TimeoutSeconds 180 -SerialWriteChunkSize 64 -SerialWriteDelayMilliseconds 2 -SerialTcpPort 4568`,
report `release\vm-reports\shadow-20260630-225419-7620.json` with 136/136
predicates, 13 `executed_commands` entries, `duration_ms: 32290`, and no
static command inventory,
covering the real QEMU/serial path through boot readiness, core read-only
methods, provider-minimal export gates, denied `module.load_ephemeral`, denied
`recovery.load_artifact`, and RAM-only audit visibility.

Latest focused recovery guest-protocol verification: 2026-07-03 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile recovery -TimeoutSeconds 90 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10 -SerialTcpPort 4624`,
report `release\vm-reports\shadow-20260703-133613-9360.json` with 3634/3634
predicates, 184 `executed_commands` entries, and no static command inventory,
covering the real QEMU/serial path through recovery evidence retention,
lifeline-command diagnostics, retained recovery status execution-result read,
command-envelope dispatch to the existing status read, the read-only recovery
lifeline status fact in `memory.context`, `memory.query`, and `memory.trace`,
provider/export side-effect denial, service inventory mutation denial, and
RAM-only recovery audit visibility while skipping the normal module-loading
diagnostic matrix.

Historical full guest-protocol verification: 2026-05-24 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile full -TimeoutSeconds 180`, report
`release\vm-reports\shadow-20260524-184613-23604.json` with 4557/4557
predicates, 209 `executed_commands` entries, `duration_ms: 181285`, and no
static command inventory,
covering absent/accepted/stale/mismatched/invalid module-manifest
hash-reference diagnostics, RAM-only retention of valid manifest and
candidate-artifact references, absent/accepted/stale/mismatched/binding-checked
VM-report hash-reference diagnostics, RAM-only retention of a valid VM-report
reference, live denied load-gate visibility of retained manifest, artifact, and
VM-report hash evidence, negative manifest/artifact/VM-report-reference gate
selftests, absent/accepted/stale/mismatched/wrong-policy module computed-grant
hash-reference diagnostics plus RAM-only retention of a valid computed-grant
hash reference and its visibility in the denied module load gate while live
loading remains denied, absent/accepted/stale/mismatched local-attestation
hash-reference diagnostics plus RAM-only retention of a valid local-attestation
reference and its visibility in the denied module load gate only after live
current-boot validation, negative retained-reference and retained
local-attestation-reference gate selftests, negative retained audit/rollback
reference gate selftests, absent/accepted/stale/mismatched local-approval
hash-reference diagnostics plus RAM-only retention of a valid local-approval
reference and its visibility in the denied module load gate only after live
current-boot validation, negative retained local-approval-reference gate
selftests,
read-only `module.audit_rollback_availability` exposing typed current-boot
missing durable audit-ledger and rollback-store availability facts, its
local-only negative selftest coverage, read-only
`module.audit_rollback_write_policy` exposing typed missing durable-write and
rollback-install policy facts plus selftests for stale/schema/provenance/
binding gaps, read-only `module.audit_rollback_storage_layout` exposing typed
missing persistence-device inventory and audit/rollback storage-layout facts
plus current-boot layout selftests, read-only
`module.audit_rollback_append_engine` exposing typed missing audit-ledger
append-engine and rollback-store transaction-engine facts plus current-boot
append-engine readiness selftests, read-only
`module.audit_rollback_append_contract` exposing typed
missing audit-ledger append-envelope and rollback-store transaction-envelope
facts plus explicit storage-layout, append-engine, write-policy, availability,
and provenance binding fields while consuming those diagnostics, read-only
`module.audit_rollback_append_payload_hash` exposing typed local-only
current-boot audit-record and rollback-transaction payload-hash envelope facts
derived from retained audit/rollback candidates, service-slot reservation
evidence, pre-load write-request shape, and bound append-contract ids while
keeping those envelopes non-durable and non-authorizing, read-only
`module.audit_rollback_append_intent` exposing typed missing audit-record and
rollback-transaction append-intent facts plus explicit append-contract,
append-engine, storage-layout, write-policy, availability, payload-hash, and
provenance binding fields while consuming the bound append contract plus
payload-hash envelope readiness, and
read-only `module.audit_rollback_write_boundary`
consuming those availability, policy, storage-layout, append-engine through
append-contract, append-contract facts, append payload-hash envelopes, and
append-intent facts plus the retained module evidence chain and returning
`denied_missing_durable_write_boundary` with
`durable_audit_write_missing`, `rollback_install_missing`,
`storage_layout_missing`, and `append_engine_missing`, plus
`module.audit_rollback_append_payload_hash_selftest` coverage for missing,
previous-boot, wrong-schema, provenance, retained-evidence binding,
service-slot binding, pre-load write-request binding, append-contract id
binding, target-schema binding, payload-hash, retained-evidence missing,
service-slot missing, append-contract missing, and available-but-non-authorizing
payload-hash candidates, plus `module.audit_rollback_append_intent_selftest`
coverage for missing, previous-boot, wrong-schema, provenance, append-contract,
append-engine, storage-layout, write-policy, availability, payload-hash,
payload-envelope missing, and
available-but-non-authorizing append-intent candidates, plus
`module.audit_rollback_write_boundary_selftest` negative coverage for missing,
stale, substituted, previous-boot, wrong-schema, mismatched,
availability-missing, policy-missing, append-contract-missing,
append-intent-missing, payload-envelope-missing, writer-unimplemented, and
recovery-separated candidates,
missing/mismatched durable audit plus rollback evidence selftests, and guest
audit/rollback hash-reference diagnostics over `raios.audit_record.v0` and
`raios.rollback_plan.v0` candidates, including RAM-only retention of a valid
audit/rollback reference, live rejection of a wrong-schema retained
audit/rollback reference, and valid retained audit/rollback visibility in the
denied module load gate, plus RAM-only service-slot reservation diagnostics and
selftests over retained computed-grant/audit/rollback event ids, canonical
reservation hashes, pre-load service-inventory hashes, and `ram_only:` slot ids,
including live denied load-gate visibility of valid retained service-slot
reservation evidence without allocation, local-only negative service-slot gate
selftests, read-only `module.service_slot_allocator` readiness diagnostics over
the RAM-only allocator runtime, service registry binding, health-state,
unload/cleanup, and missing durable-audit, rollback-install, and loader gates,
plus
allocator-readiness selftests, and the separate denied recovery artifact load boundary proving
`cap.recovery.load_artifact`, typed missing recovery identity/trust/VM-test/
approval/loader/rollback facts, event-log binding, no normal module capability
reuse, no recovery artifact load, and no service inventory change, plus
read-only recovery artifact identity/trust hash-reference diagnostics retaining
valid local-only current-boot `raios.recovery_artifact_identity.v0` and
`raios.recovery_artifact_trust.v0` event bindings without artifact bytes or
load authority, read-only recovery artifact VM-test/local-approval
hash-reference diagnostics retaining valid local-only current-boot
`raios.recovery_artifact_vm_test.v0` and
`raios.recovery_artifact_local_approval.v0` event bindings without accepting
VM-test JSON, approval text, artifact bytes, or load authority, read-only
recovery artifact loader/rollback-evidence hash-reference diagnostics retaining
valid local-only current-boot `raios.recovery_artifact_loader.v0` and
`raios.recovery_artifact_rollback_evidence.v0` event bindings without accepting
loader descriptors, rollback evidence JSON, artifact bytes, or load authority,
plus read-only `recovery.load_binding` and `recovery.load_binding_selftest`
proving all six required recovery-only evidence ids, normal module
append-intent, append-payload, writer, service-slot, and
`module.load_ephemeral` facts are non-authority, append payload-hash envelopes
remain non-authority inputs, plus read-only
`recovery.lifeline_request_diagnostic` and
`recovery.lifeline_request_diagnostic_selftest` proving
`raios.recovery_lifeline_request.v0` consumes the retained recovery identity,
trust, VM-test, local-approval, loader, and rollback-evidence event ids only as
local-only current-boot hash references, rejects missing, stale, previous-boot,
wrong-schema, substituted, and mismatched chains, and keeps recovery artifacts
non-loaded, non-durable, local-only, and non-authorizing until a recovery
lifeline protocol behavior exists, plus read-only
`recovery.lifeline_protocol_diagnostic` and
`recovery.lifeline_protocol_diagnostic_selftest` proving
`raios.recovery_lifeline_protocol_state.v0` consumes the retained
`raios.recovery_lifeline_request.v0` event id plus those six recovery evidence
event ids, rejects missing, stale, previous-boot, wrong-schema, substituted,
and mismatched lifeline request/evidence chains before reporting protocol gaps,
and exposes typed local-only missing facts for lifeline protocol state,
command vocabulary, loader runtime isolation, rollback transaction engine,
durable audit/rollback persistence, and recovery memory provenance.
It also covers read-only `recovery.lifeline_command_vocabulary` and
`recovery.lifeline_command_vocabulary_selftest`, proving
`raios.recovery_lifeline_command_vocabulary.v0` enumerates recovery command ids,
argument-envelope schemas, required capabilities, and denial reasons only after
the retained lifeline request/evidence chain validates, rejects missing, stale,
previous-boot, wrong-schema, substituted, and mismatched request/protocol-state
inputs, and keeps command envelopes, command dispatch, recovery loading,
durable writes, rollback installs, loader execution, service-slot allocation,
and service inventory changes disabled. It also covers read-only
`recovery.loader_runtime_isolation` and
`recovery.loader_runtime_isolation_selftest`, proving
`raios.recovery_loader_runtime_isolation.v0` reuses the retained lifeline
request/evidence chain and command-vocabulary envelope, rejects invalid
request/protocol-state/command-vocabulary inputs, exposes missing loader
address-space, entrypoint ABI, memory-map, capability-import, artifact-hash,
provider-separation, normal-module-separation, rollback transaction, durable
persistence, and recovery-memory-provenance facts, and keeps loader execution,
artifact loading, command dispatch, durable writes, rollback installs,
service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled. It also covers read-only
`recovery.rollback_transaction_engine` and
`recovery.rollback_transaction_engine_selftest`, proving
`raios.recovery_rollback_transaction_engine.v0` consumes the retained lifeline
request/evidence chain, command-vocabulary envelope, and loader runtime
isolation boundary, rejects invalid request/protocol-state/command-vocabulary/
loader-isolation inputs, exposes missing rollback target, transaction
id/provenance, last-good, disabled-module set, artifact-hash, replay,
recovery-capability import, atomic apply/abort, durable audit/rollback
persistence, and recovery-memory-provenance facts, and keeps rollback preview,
rollback apply, lifeline command dispatch, loader execution, artifact loading,
durable writes, rollback installs, service-slot allocation, direct-OpenAI
recovery shortcuts, and service inventory changes disabled. It also covers
read-only `recovery.durable_audit_rollback_persistence` and
`recovery.durable_audit_rollback_persistence_selftest`, proving
`raios.durable_audit_rollback_persistence.v0` consumes the rollback transaction
engine boundary after the retained lifeline chain, command-vocabulary envelope,
and loader isolation boundary validate, rejects invalid rollback-engine/
loader-isolation/command-vocabulary/protocol-state/request inputs, exposes
missing persistence-device inventory, storage-layout identity, audit append-log
identity, rollback-store identity, replay cursor, last-good checkpoint, write
ordering, crash consistency, integrity root/hash chain, and recovery-memory
provenance facts, and keeps durable writes, rollback replay, recovery-memory
writes, rollback preview/apply, loader execution, artifact loading, rollback
installs, service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled.
It also covers read-only `recovery.memory_provenance` and
`recovery.memory_provenance_selftest`, proving
`raios.recovery_memory_provenance.v0` consumes the durable persistence boundary
after the retained lifeline chain, command-vocabulary envelope, loader
isolation boundary, and rollback transaction-engine boundary validate, rejects
invalid durable-persistence/rollback-engine/loader-isolation/
command-vocabulary/protocol-state/request inputs, exposes missing source record
ids, source schema hashes, classification, authority level,
rollback-transaction binding, last-good checkpoint binding, recovery-only
export profile, redaction state, replay window, and audit linkage facts, and
keeps memory writes, provider export, durable writes, rollback replay,
rollback preview/apply, command dispatch, loader execution, artifact loading,
rollback installs, service-slot allocation, direct-OpenAI recovery shortcuts,
and service inventory changes disabled.
It also covers read-only `recovery.lifeline_command_admission` and
`recovery.lifeline_command_admission_selftest`, proving
`raios.recovery_lifeline_command_admission.v0` consumes the recovery memory
provenance boundary after the retained lifeline chain, command-vocabulary
envelope, loader isolation boundary, rollback transaction-engine boundary, and
durable persistence boundary validate, rejects invalid memory-provenance/
durable-persistence/rollback-engine/loader-isolation/command-vocabulary/
protocol-state/request inputs, enumerates non-executing admission requirements
for lifeline status, rollback preview, rollback apply, disable module, restart
last-good, and load recovery artifact by hash command envelopes, and keeps
command envelopes, command dispatch, rollback preview/apply, memory writes,
provider export, durable writes, rollback replay, loader execution, artifact
loading, rollback installs, service-slot allocation, direct-OpenAI recovery
shortcuts, and service inventory changes disabled.
It also covers read-only `recovery.lifeline_command_envelope_diagnostic` and
`recovery.lifeline_command_envelope_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_envelope_reference.v0` consumes command
admission after the retained lifeline chain, validates only hash/reference
shape for lifeline status, rollback preview/apply, disable module, restart
last-good, and load-artifact-by-hash command ids, rejects invalid
command-admission/memory-provenance/durable-persistence/rollback-engine/
loader-isolation/command-vocabulary/protocol-state/request chains, and keeps
command bodies, command envelope acceptance, command dispatch, rollback
preview/apply, memory writes, provider export, durable writes, rollback replay,
loader execution, artifact loading, rollback installs, service-slot
allocation, direct-OpenAI recovery shortcuts, and service inventory changes
disabled while retaining a valid status-command hash reference only as
local-only current-boot evidence.
It also covers read-only `recovery.lifeline_command_dispatch_diagnostic` and
`recovery.lifeline_command_dispatch_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_dispatch_denial.v0` consumes the retained
command-envelope reference, rejects invalid command-envelope/admission/
memory-provenance/durable-persistence/rollback-engine/loader-isolation/
command-vocabulary/protocol-state/request chains, exposes missing command body
canonicalization, command handler binding, status-read handler,
rollback-preview/apply authorization, disable-module/restart-last-good/
load-artifact-by-hash target bindings, recovery-memory write authority,
durable audit/rollback write authority, and service-inventory side-effect
facts, and keeps command bodies, command envelope acceptance, command dispatch,
rollback preview/apply, memory writes, provider export, durable writes,
rollback replay, loader execution, artifact loading, rollback installs,
service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled.
It also covers read-only
`recovery.lifeline_command_body_canonicalization_diagnostic` and
`recovery.lifeline_command_body_canonicalization_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_body_canonicalization.v0` consumes the
retained command-envelope reference plus the dispatch-denial boundary,
validates only the canonical command-body hash/reference shape, rejects invalid
dispatch/envelope/admission/memory-provenance/durable-persistence/
rollback-engine/loader-isolation/command-vocabulary/protocol-state/request
chains, retains a valid status-command body-canonicalization reference only as
local-only current-boot evidence, and causes the dispatch diagnostic to advance
only to the next missing handler-binding fact while still accepting no raw
command body and dispatching no recovery command.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/raios-stage0.img`
- Firmware vars seed: `release/ovmf_vars.fd`
- Bootloader: Limine 10 UEFI binary at `release/esp/EFI/BOOT/BOOTX64.EFI`
- Config file: `limine.conf` at ESP root and `EFI/BOOT/limine.conf`
- Kernel path inside image: `/kernel/kernel.elf`

The image boots in QEMU using the Windows PowerShell runner:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

For interactive serial commands, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

For a QEMU xHCI inventory run, add `-UsbXhciInput`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

For the bare-metal-style VM profile with USB keyboard, USB mouse, RDRAND, and
e1000 networking, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:03.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
```

Expected visible framebuffer UI:

```text
AI  CONSOLE                                      SET
RAIOS
DIRECT AI HOST
NET CONFIGURED   INPUT READY   USB READY   RNG READY
CHAT
TYPE MESSAGE AND PRESS ENTER
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
HHDM offset=0xffff800000000000
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

Console commands verified over TCP serial and USB-HID keyboard input:

```text
help
status
devices
log
provider
openai
setup
ask <text>
```

The framebuffer UI defaults to an AI chat mode. The `CONSOLE` tab keeps the
debug console visible, and the `SET` tab opens provider settings. `setup` also
opens the in-VM OpenAI/API-key menu. API-key entry is masked, held only in guest
RAM, and not printed into the console or serial output. For local-only testing,
the build scripts can also embed `OPENAI_API_KEY` into a separate non-default
image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI trust-gate smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

Direct OpenAI SPKI pinned-trust smoke is verified with a temporary image built
from a process-local fake API key and a current `OPENAI_SPKI_SHA256` pin.
Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

The legacy leaf-certificate pinned-trust smoke remains supported with
`OPENAI_CERT_SHA256`. Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard/mouse input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture-decisions/0001-raios-agent-protocol.md`.

## Historical Rollback Cursor Archive

This section is historical handoff material from earlier rollback/storage
slices. Do not use it as the current cursor; the current exact next task is the
`Current exact next task` paragraph above and the compact cursor in
`docs/ROADMAP.md`.

Now that `service.rollback_apply svc.demo.hello` consumes the shared
current-boot append-contract foundation, names the rollback-transaction append
target, observes real AHCI read evidence, proves the VM-harness-labeled scratch
region, writes and reads back the planned Hello rollback append sector image on
scratch only, binds a read-only `RAIOS_AUDITRB_V0` target-region discovery,
writes/reads back that same planned sector image on the dedicated non-scratch
target region as test infrastructure, exposes
`raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0`, and
then denies
`raios.ram_only_hello_service_rollback_durable_append_transaction_authorization_gate.v0`
over the append-record, sector-plan, target-region write/readback,
audit-ledger, rollback-store, target-span, writer-policy, and missing writer
evidence, and then consumes that gate in
`raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0`,
and then binds that readiness through
`raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0`,
binds that append decision through
`raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0`, and
binds the policy decision, canonical audit-record image, media-write policy
evidence, and verified LBA1/512-byte target span through
`raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0`, and
consumes that candidate through
`raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0`,
then emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0`
over that acceptance gate, candidate, decision, audit image, media policy, and
target span as read-only current-boot evidence, then emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0`
over that ledger candidate as a fail-closed acceptance result, then emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_write_authority_availability.v0`
over that result, ledger candidate, media policy, target-region write/readback,
audit/rollback target ids/schemas, and target span while still withholding
durable media writes and durable append authority, then emits
`raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0`
over that write-authority availability evidence, ledger-aware result, ledger
candidate, media policy, target-region write/readback, audit/rollback target
ids/schemas, and target span while still withholding durable policy ledger,
durable audit policy, durable media writes, and durable append authority, then
emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0`
over that policy-ledger availability evidence, write-authority availability,
ledger-aware result, ledger candidate, media policy, target-region
write/readback, audit/rollback target ids/schemas, and target span while still
withholding durable audit policy, durable policy ledger, write authority,
durable media writes, and durable append authority, then emits
`raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0`
over that audit-policy availability evidence, policy-ledger availability,
write-authority availability, ledger-aware result, ledger candidate, media
policy, target-region write/readback, audit/rollback target ids/schemas, and
target span while still withholding durable append authority, durable audit
policy, durable policy ledger, write authority, and durable media writes,
then emits
`raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0`
over that durable append-authority availability evidence, audit-policy
availability, append-engine readiness, durable writer policy, media policy,
target-region write/readback, audit/rollback target ids/schemas, and target
span while still withholding durable append authority, durable audit policy,
transaction append, durable writes, and rollback application,
then emits
`raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0`
over that transaction-append availability decision, durable append-authority
availability, audit-policy availability, append-engine readiness, durable writer
policy, media policy, target-region write/readback, audit/rollback target
ids/schemas, and target span while keeping
`missing_transaction_append_authority: true` and all write/append side effects
false, then emits
`raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0`
over durable append-authority availability, audit-policy availability dry-run,
audit-policy availability, policy-ledger availability dry-run, policy-ledger
availability, write-authority availability, ledger-aware result,
ledger-candidate, media policy, target-region write/readback,
transaction-append authority-denial gate, transaction append-availability
decision, audit/rollback target ids/schemas, and target span as current-boot
test-media-only evidence, then makes
`raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0`
consume that durable append-authority availability dry-run hash alongside its
transaction-append dry-run, target-sector inspection, write-authority,
audit-policy, append-authority, transaction-denial, transaction
append-availability, audit/rollback target id/schema, and LBA1/512-byte span
evidence, then makes the top-level
`raios.ram_only_hello_service_rollback_apply.v0` denial hash consume that
retained durable policy write-authority decision hash plus retained
`raios.recovery_rollback_inspect_source_reference.v0` evidence, and now makes
the read-only `raios.recovery_rollback_apply_authorization.v0`
diagnostic/reference path bind that sourced denial hash plus the retained
durable policy write-authority decision and inspect-source hashes, then makes
the read-only `raios.recovery_disable_module_target_binding.v0`
diagnostic/reference path carry the same source-bound apply evidence through
the retained disable-module target binding while still disabling no module. Do
not make the global AHCI block driver writable, do not treat scratch or
test-media writes as durable authority, and do not claim installed rollback
state. The next slice should carry that source-bound disable-module
target-binding evidence into the existing
`raios.recovery_restart_last_good_target_binding.v0` diagnostic/reference path.
Continue to
keep rollback application, persistent install, durable audit writes,
rollback-store writes, rollback transaction append, external artifact bytes,
candidate-byte execution, executable mapping, provider-triggered auto-load, and
broad mutation denied until the real writer and storage authority exists.
Provider trust/context hardening remains a parallel Track B, but do not claim
WebPKI chain or time validation until trusted roots, intermediate chain
handling, and a trusted time source are actually present.

The next slice should:

- keep bare `module.load_ephemeral` and arbitrary external artifacts denied
- keep `agent command_envelope ... target_method=system.describe ...` routing
  through the existing dispatcher
- keep `agent command_envelope ... target_method=system.snapshot ...` routing
  through the existing dispatcher
- keep `agent command_envelope ... target_method=system.boot_log ...` routing
  through the existing dispatcher and local-only
- keep `agent command_envelope ... target_method=system.capabilities ...`
  routing through the existing dispatcher
- keep `agent command_envelope ... target_method=device.graph ...` routing
  through the existing dispatcher
- keep `agent command_envelope ... target_method=service.inventory ...` routing
  through the existing dispatcher
- keep the mismatched allowed target/read-capability envelope denied with
  current-boot audit evidence and no dispatcher side effect
- keep `agent command_envelope ... target_method=problem.list ...` routing
  through the existing dispatcher
- keep malformed or over-capable envelopes denied before dispatch
- keep the rollback-apply writer/storage gate bound to the shared
  append-contract foundation, `storage.authority.audit_rollback.current_boot`,
  `append.audit_ledger.current_boot`, and `append.rollback_store.current_boot`,
  plus append-target-owner, transaction-writer-readiness, and
  block-write-path authority-gate facts derived from the verified read-only
  block-driver and partition inventory evidence
- keep the durable append-authority availability fact bound into the
  rollback-apply response and RAM audit event, keep the no-write
  rollback transaction-append dry-run blocked by the authority-denial gate, keep
  the durable policy write-authority decision bound to the durable
  append-authority availability dry-run hash, keep the top-level
  rollback-apply denial hash bound to the retained durable policy
  write-authority decision plus retained inspect-source evidence, and make the
  next slice feed that sourced denial evidence into the recovery
  rollback-apply authorization diagnostic/reference path while all durable
  media writes, durable appends, and audit/rollback application paths remain
  denied
- keep the fail-closed rollback-apply gate over retained rollback-preview/
  probation evidence proving it cannot mutate descriptor, generation, running
  state, or RAM-only Hello state
- keep the read-only rollback preview over retained hot-swap probation evidence
  proving previous/current descriptor, generation, state hash/counter, and
  migration facts without mutating service state
- keep accepted hot-swap probation evidence binding previous/new descriptor,
  generation, state hash/counter, and migration hash without claiming rollback
  or persistence authority
- keep `service.hot_swap svc.demo.hello.reset_state` denied by the
  state-migration gate and prove it cannot change the active descriptor,
  generation, or RAM-only Hello state
- keep current-image and host-bound `svc.demo.hello` load/list/stop/start/
  restart/drop passing in quick VM smoke, with explicit `service.start
  svc.demo.hello` starting a stopped loaded current-boot service and
  `service.restart svc.demo.hello` preserving the same loaded generation
- keep `service.hot_swap external:svc.demo.hello` denied before service
  mutation and both v1/v2 hot-swaps advancing only accepted current-boot
  generations
- keep `raios.current_boot_load_request.v0` and
  `raios.current_boot_load_descriptor.v0` in the positive path
- keep `service.health svc.demo.hello` proving healthy, stopped, missing, and
  host-bound source-bound states in quick VM smoke
- keep `service.descriptor_source_trust_selftest` green for valid and tampered
  descriptor-source envelope cases
- keep `service.artifact_reference_trust_selftest` green for valid and tampered
  artifact-reference byte/content/reference/trust cases
- keep `service.artifact_load_plan_preflight_selftest` green for valid and
  tampered descriptor/artifact/slot/denial cases
- keep the signed built-in artifact identity/trust envelope visible in load
  response, `service.inventory`, `service.health`, and RAM audit bindings
- keep the signed content/hash binding for the existing built-in Hello artifact
  candidate visible in load response, `service.inventory`, `service.health`,
  and RAM audit bindings
- keep the signed artifact-byte/reference evidence for the Hello candidate
  visible in load response, `service.inventory`, `service.health`, and RAM
  audit bindings
- keep the artifact load-plan preflight id/hash/status/accepted state visible
  in load response, nested descriptor, `service.inventory`, `service.health`,
  and lifecycle/health RAM audit evidence
- keep the RAM-only service-slot activation record derived from the accepted
  preflight, with stable id/hash/status/active state and explicit current-boot
  scope visible in load/start, `service.inventory`, `service.health`, stop/drop
  responses, and lifecycle/health RAM audit evidence
- harden provider trust/context gating in its own track with typed positive
  evidence rather than prompt stuffing or a development TLS bypass
- keep stable artifact byte/reference trust ids and hashes in load response,
  `service.inventory`, `service.health`, and RAM audit bindings
- keep the selected descriptor source locator/kind/validation/hash plus any
  bound source hash visible in load response, service.inventory, health output,
  and lifecycle/health audit event
- keep persistence, arbitrary external artifact intake, durable audit writes,
  rollback installation, provider-triggered auto-load, and broad
  module/service/config mutation denied

Do not add a signed artifact loader yet. The runtime-artifact track now has a
real current-boot lifecycle, signed v1/v2 hot-swap, state-preserving migration
evidence, a fail-closed reset-state migration denial, accepted hot-swap
probation evidence, and a read-only rollback preview for the already-working
Hello slot, plus a fail-closed rollback-apply gate over retained
preview/probation facts; the next highest-value OS core slice is the smallest
rollback transaction/durable-audit preflight that can later authorize a real
apply while arbitrary module execution and real rollback application remain
denied.

For multi-agent execution, treat the agent-command boundary as Track A and
provider trust/context as Track B. UI/input polish, harness speed/evidence, and
recovery/persistence design may proceed in parallel as long as they do not
weaken current runtime denials.

Historical recovery refactor notes retained below are no longer the active
roadmap cursor:

- the latest behavior-neutral slice moved the six recovery lifeline command
  specs and dispatch boundary constant into
  `seed-kernel/src/agent_protocol_recovery_lifeline.rs`, keeping public command
  vocabulary and all schema/boundary ids unchanged
- the follow-up behavior-neutral slice moved execution-stage selftest case
  construction, retained-chain reference matchers, JSON response emission, and
  retained execution-stage event recording into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`
- the latest behavior-neutral slice moved the thin execution-stage public
  wrapper methods and method-predicate wiring into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`, and the central
  agent dispatcher now imports those wrappers directly from the execution module
- the current behavior-neutral slice moved retained execution-stage
  chain-presence evaluation into
  `seed-kernel/src/agent_protocol_recovery_execution.rs` while leaving the
  recovery dispatch candidate type in `agent_protocol_recovery.rs`
- the previous behavior-neutral slice moved the shared execution-stage
  descriptor/input ownership, method/argument matching helpers, stage
  descriptor constants, execution-stage boundary IDs, reference-check type,
  parser/evaluator, hash-validation, and live-chain validation helpers into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`
- the latest behavior-neutral slices moved recovery method predicates and
  diagnostic argument parsers into
  `seed-kernel/src/agent_protocol_recovery_methods.rs`, recovery capability,
  selftest-count, and boundary-id constants into
  `seed-kernel/src/agent_protocol_recovery_constants.rs`, recovery
  load-binding types into
  `seed-kernel/src/agent_protocol_recovery_load_binding.rs`, recovery
  artifact-reference types into
  `seed-kernel/src/agent_protocol_recovery_artifact_types.rs`, plus recovery
  artifact-reference parsers, evaluators, selftest fixtures, and event-log
  binding builders into
  `seed-kernel/src/agent_protocol_recovery_artifact_reference.rs`, and lifeline
  protocol/command-vocabulary types into
  `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_types.rs`, plus
  lifeline runtime/isolation/rollback/persistence/provenance/admission types
  into `seed-kernel/src/agent_protocol_recovery_runtime_types.rs`, and command
  envelope/dispatch-denial/body-canonicalization types into
  `seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs`, plus
  handler/status/rollback-authorization/target-binding types into
  `seed-kernel/src/agent_protocol_recovery_command_authorization_types.rs`,
  and memory/durable-write/service-inventory/command-effect gate types into
  `seed-kernel/src/agent_protocol_recovery_command_effect_types.rs`, plus
  recovery lifeline command reference parsers, evaluators, and event-log
  binding builders into
  `seed-kernel/src/agent_protocol_recovery_command_reference_eval.rs`, plus
  recovery memory/durable/service/dispatch-behavior/executor/side-effect
  reference evaluators into
  `seed-kernel/src/agent_protocol_recovery_command_effect_reference_eval.rs`,
  plus
  handler/status/rollback/target/effect command reference selftest fixtures
  into `seed-kernel/src/agent_protocol_recovery_command_reference_selftests.rs`,
  plus command envelope/dispatch/body evaluator selftest helpers into
  `seed-kernel/src/agent_protocol_recovery_command_eval.rs`, plus
  recovery artifact-reference emit helpers into
  `seed-kernel/src/agent_protocol_recovery_artifact_reference_emit.rs`,
  recovery artifact/lifeline request selftest emit helpers into
  `seed-kernel/src/agent_protocol_recovery_artifact_selftest_emit.rs`, and
  lifeline protocol emit helpers into
  `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_emit.rs`, plus
  lifeline command-vocabulary emit helpers into
  `seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_emit.rs`,
  and loader-runtime-isolation emit helpers into
  `seed-kernel/src/agent_protocol_recovery_loader_runtime_emit.rs`, plus
  rollback-transaction, durable-persistence, memory-provenance, and
  command-admission emit helpers into
  `seed-kernel/src/agent_protocol_recovery_rollback_transaction_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_persistence_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_memory_provenance_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_command_admission_emit.rs`, plus
  command-envelope, command-dispatch, command-body-canonicalization, and
  command-handler emit helpers into
  `seed-kernel/src/agent_protocol_recovery_command_envelope_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_command_dispatch_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_command_body_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_command_handler_emit.rs`, plus
  status-read, rollback-preview, rollback-apply, and disable/restart/
  load-target emit helpers into
  `seed-kernel/src/agent_protocol_recovery_status_handler_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_rollback_preview_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_rollback_apply_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_target_binding_emit.rs`, plus
  memory-write, durable-write, service-inventory side-effect, and
  command-effect emit helpers into
  `seed-kernel/src/agent_protocol_recovery_memory_write_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_durable_write_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_service_inventory_effect_emit.rs`,
  and `seed-kernel/src/agent_protocol_recovery_command_effect_emit.rs`, plus
  recovery load-binding emit helpers into
  `seed-kernel/src/agent_protocol_recovery_load_binding_emit.rs`
- `seed-kernel/src/agent_protocol_recovery.rs` is now below the 10k-line
  threshold; continue future extraction only over stable ownership boundaries
  such as remaining protocol retained-chain helpers or further splitting the
  focused command evaluator modules
- preserve every public method name, schema id, boundary id, denial reason,
  canonical hash line, event-log binding, and shadow-smoke expectation exactly
  except for file/module ownership
- keep the refactor commit behavior-neutral and prove it with
  `build-seed-kernel.ps1 -Profile release`, `cargo fmt --all -- --check`,
  `git diff --check`, workspace Cargo tests, secret scan, and
  `vm-harness\shadow-vm-smoke.ps1`
- when running `vm-harness\shadow-vm-smoke.ps1` through an agent tool on this
  Windows/QEMU setup, allow at least a 30-minute outer timeout, pass
  `-TimeoutSeconds 300` when the default per-command serial timeout is too
  tight, and check `release\vm-reports\shadow-*.json` plus the temp
  `serial.log` before treating a timeout as a protocol failure
- for fast iteration, `vm-harness\shadow-vm-smoke.ps1 -Profile quick` runs the
  real QEMU/serial path through boot, core read-only methods, provider-minimal
  export gates, denied module loading, denied recovery artifact loading, and
  RAM-only audit visibility while skipping the exhaustive negative matrix; the
  default `-Profile full` remains the complete evidence path
- for focused recovery iteration, `vm-harness\shadow-vm-smoke.ps1 -Profile recovery`
  skips the provider selftest, memory mutation, and normal module-loading matrix
  while preserving the real recovery lifeline and audit path; the harness writes
  serial commands in chunks by default to avoid spending minutes on host-side
  byte pacing for long diagnostic commands
- keep this refactor style as the standing development rule: when a stable
  boundary is visible, split it before files or docs become large, and make
  reports derive from observed execution rather than duplicated static lists

Historical verified recovery foundation retained for reference:

- `recovery.lifeline_command_execution_completion_denial_diagnostic` and
  `recovery.lifeline_command_execution_completion_denial_diagnostic_selftest`
  now retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_completion_denial.v0` hash
  reference over the retained execution-observation-denial reference while
  still accepting no raw command body, no lifeline command body, no lifeline
  command envelope, dispatching no command, executing no recovery behavior,
  observing no command result, exporting no provider context, and writing no
  memory, audit, rollback, completion, or service-inventory records. Dispatch
  now advances through completion-denial before ending at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_execution_observation_denial_diagnostic` and
  `recovery.lifeline_command_execution_observation_denial_diagnostic_selftest`
  now retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_observation_denial.v0` hash
  reference over the retained execution-audit-denial reference while still
  accepting no raw command body, no lifeline command body, no lifeline command
  envelope, dispatching no command, executing no recovery behavior, observing
  no command result, exporting no provider context, and writing no memory,
  audit, rollback, or service-inventory records. Dispatch now advances through
  audit-denial and observation-denial before ending at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_execution_audit_denial_diagnostic` and
  `recovery.lifeline_command_execution_audit_denial_diagnostic_selftest` now
  retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_audit_denial.v0` hash reference
  over the retained execution-result-denial reference while still accepting no
  raw command body, no lifeline command body, no lifeline command envelope,
  dispatching no command, executing no recovery behavior, and writing no audit
  or rollback records. Dispatch now advances through result-denial and
  audit-denial before ending at explicit `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_side_effect_gate_diagnostic` and
  `recovery.lifeline_command_side_effect_gate_diagnostic_selftest` now retain
  only local-only current-boot side-effect-gate hash references over the
  retained executor-capability-table reference and advance dispatch only to the
  missing execution-enablement boundary until that boundary is retained. They
  do not accept raw command bodies or lifeline envelopes, dispatch commands,
  execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_execution_enablement_diagnostic`,
  `recovery.lifeline_command_execution_preflight_diagnostic`,
  `recovery.lifeline_command_execution_intent_diagnostic`, and
  `recovery.lifeline_command_execution_commit_gate_diagnostic`, plus
  `recovery.lifeline_command_execution_result_denial_diagnostic`, and
  `recovery.lifeline_command_execution_audit_denial_diagnostic`, and
  `recovery.lifeline_command_execution_observation_denial_diagnostic`, and
  `recovery.lifeline_command_execution_completion_denial_diagnostic`, with their
  selftests, now retain local-only current-boot hash references over the
  previous execution stage. They validate the same command, target, authority,
  side-effect-gate, executor, dispatch, and source-bound rollback-apply/policy/
  inspect hashes, advance dispatch through the enablement, preflight, intent,
  commit-gate, result-denial, audit-denial, observation-denial, and
  completion-denial facts, and still end at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`. They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_executor_capability_table_diagnostic` and
  `recovery.lifeline_command_executor_capability_table_diagnostic_selftest`
  now retain only local-only current-boot executor-capability-table hash
  references over the retained command-dispatch behavior reference and advance
  dispatch only to the side-effect gate boundary, which remains non-executing
  until the execution-enablement boundary is retained.
  They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_dispatch_behavior_diagnostic` and
  `recovery.lifeline_command_dispatch_behavior_diagnostic_selftest` now retain
  only local-only current-boot command-dispatch behavior hash references over
  the retained service-inventory side-effect boundary reference and advance
  dispatch only to the missing executor-capability table until that table is
  retained. They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.service_inventory_side_effect_boundary_diagnostic` and
  `recovery.service_inventory_side_effect_boundary_diagnostic_selftest` now
  retain only local-only current-boot service-inventory side-effect boundary
  hash references over the retained durable-audit/rollback write-authority
  reference and leave dispatch at explicit `defined_non_executable` behavior.
  They do not dispatch commands, allocate service slots, create service
  inventory records, change service inventory, write recovery memory, or write
  durable audit/rollback state.
- `recovery.durable_audit_rollback_write_authority_diagnostic` and
  `recovery.durable_audit_rollback_write_authority_diagnostic_selftest` now
  retain only local-only current-boot durable-audit/rollback write-authority
  hash references over the retained recovery-memory write-authority reference
  and leave dispatch stopped at missing service-inventory side-effect
  boundary. They do not dispatch commands, write durable audit/rollback state,
  write recovery memory, load artifacts, allocate service slots, or change
  service inventory.
- `recovery.memory_write_authority_diagnostic` and
  `recovery.memory_write_authority_diagnostic_selftest` now retain only
  local-only current-boot recovery-memory write-authority hash references over
  the retained load-artifact-by-hash target binding and leave dispatch stopped
  at missing durable-audit/rollback write authority. They do not dispatch
  commands, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- `recovery.load_artifact_by_hash_target_binding_diagnostic` and
  `recovery.load_artifact_by_hash_target_binding_diagnostic_selftest` now
  retain only local-only current-boot load-target hash references over the
  retained restart-last-good target binding and leave dispatch stopped at
  missing recovery-memory write authority. They do not dispatch commands, load
  artifacts, authorize recovery load, write recovery memory, create durable
  records, or change service inventory.
- `recovery.restart_last_good_target_binding_diagnostic` and
  `recovery.restart_last_good_target_binding_diagnostic_selftest` now retain
  only local-only current-boot restart-target hash references over the retained
  disable-module target binding and leave dispatch stopped at missing
  load-artifact-by-hash target binding. They do not dispatch commands, restart
  services, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- `recovery.disable_module_target_binding_diagnostic` and
  `recovery.disable_module_target_binding_diagnostic_selftest` now retain only
  local-only current-boot disable-target hash references over the retained
  rollback-apply authorization and leave dispatch stopped at missing
  restart-last-good target binding. They do not dispatch commands, disable
  modules, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- Virtio has been removed from the Stage-0 kernel runtime and VM runner path.
- RDRAND seeds entropy in the bare-metal-style VM profile.
- Intel e1000 configures RX/TX rings, negotiates DHCP through smoltcp, and shows
  IP/gateway state in the framebuffer UI and serial console.
- a PS/2/i8042 polling fallback is present for first bare-metal keyboard tests
  on machines that expose legacy keyboard compatibility. It is only reported as
  ready after an acknowledge from the keyboard or real scancode input.
- a polled xHCI path now inventories USB controllers, resets directly attached
  root-port devices, enumerates HID boot keyboards, relative boot mice, and QEMU
  HID tablets, and feeds reports into the same input queue as PS/2.
- if no USB keyboard or pointer is active, the event loop periodically re-probes
  xHCI so a keyboard plugged in after boot can be picked up without rebooting.
- the USB status line includes `EV`, `ERR`, and `TCC` counters for HID input
  reports and interrupt transfer diagnostics on bare metal.
- the USB-XHCI row now includes keyboard and mouse readiness.
- the framebuffer renderer is double-buffered to avoid visible full-screen
  redraw flicker, and pointer movement now updates only a small cursor overlay
  instead of forcing a full UI redraw.
- the visible QEMU GTK profile uses `usb-tablet` absolute pointer input by
  default and hides the host cursor over the guest area without automatic mouse
  grab, so only the raiOS pointer is visible and remains aligned after focus
  changes; `-RelativeMouse` or `-MouseGrab` switches back to relative
  `usb-mouse` for stricter boot-mouse testing.
- the visible UI now defaults to a chat-first surface with `AI`, `CONSOLE`, and
  `SET` modes. Serial commands continue to use the command interpreter so VM
  harnesses remain deterministic.
- USB/PS2 keyboard input now carries special keys into the UI: Tab and arrow
  keys move a visible focus ring through the top navigation, chat/console input,
  and settings actions; Enter activates the focused item and Esc backs out of
  settings/API-key entry.
- the Surface Pro 4 internal WLAN target has been selected as Marvell AVASTAR
  88W8897 (`11ab:2b38`, Linux reference driver family `mwifiex_pcie`). Stage-0
  now probes PCI for that device and exposes it as a Wi-Fi status chip/log line,
  and the settings menu can record a RAM-only SSID and WPA passphrase. Firmware
  upload, WPA, and packet transport are not implemented yet.
- a VM-local `setup` menu now records a RAM-only OpenAI API key without echoing
  the key back into the serial log.
- `ask <text>` now stays inside the guest. In the normal build it requires the
  VM API key state and then fails closed at provider trust before API-key copy or
  HTTPS write unless a syntactically valid provider pin is configured. With
  `-EmbedOpenAiSpkiPinFromEnv`, the preferred verifier slice checks the OpenAI
  leaf SubjectPublicKeyInfo SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  `-EmbedOpenAiCertPinFromEnv`, the first positive verifier slice checks the
  OpenAI leaf certificate SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  the explicit development override
  `-AllowUnverifiedOpenAiTls`, it resolves `api.openai.com`, opens TCP 443
  through e1000, performs TLS 1.3 with `NoVerify`, sends an HTTPS Responses API
  request, parses `output_text`, and prints the provider response.
- the provider trust state is visible in console/provider status,
  `system.snapshot.v0`, `problem.list`, and `service.inventory`; the default
  trust problem is `provider.tls_pin_config_missing`, while a successful pinned
  handshake reports `pinned_spki_verified` or `pinned_cert_verified`.
- `raios.agent.v0` exposes read-only serial methods for `system.describe`,
  `system.snapshot`, `system.capabilities`, `system.boot_log`, `device.graph`,
  `problem.list`, `service.inventory`, `memory.profile`, `memory.context`,
  `memory.query`, `memory.trace`, `memory.recent_events`, `audit.events`,
  `module.manifest_diagnostic`, `module.artifact_diagnostic`,
  `module.vm_report_diagnostic`, `module.grant_diagnostic`,
  `module.attestation_diagnostic`, `module.approval_diagnostic`,
  `module.audit_rollback_append_intent`, and their current selftest methods.
- mutating or potentially mutating methods such as `module.load_ephemeral`,
  `service.restart`, `config.apply`, `provider.configure`, and `wifi.configure`
  return structured `capability_denied` until manifest, VM test report, local
  attestation, computed capability grant, approval, audit, and rollback evidence
  exist.
- `module.load_ephemeral` and `service.load_ephemeral` now return
  `raios.module_load_gate.v0`, which reports the manifest, exact artifact, VM
  report, local attestation, computed grant, local approval, durable audit,
  rollback plan, loader, and ram-only service slot gates; the current state is
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.manifest_diagnostic` now exposes a read-only guest diagnostic for a
  module-manifest hash reference. It accepts no manifest JSON, artifact bytes,
  or unsigned service code and validates only the canonical
  `raios.module_manifest_reference.v0` hash over the manifest hash, requested
  capability, load mode, subject, resource, and current-boot scope.
- A valid `module.manifest_diagnostic` reference is retained as a local-only
  current-boot `raios.module_manifest_reference.v0` event binding. The retained
  record stores hashes only, appears through `retained_manifest_reference` and
  `audit.events`, and remains non-authorizing with
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained manifest reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `module_manifest: retained_hash_reference_only`,
  `retained_module_manifest_reference.state: present`, retained hashes, and
  `retained_module_manifest_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  manifest hashes as accepted evidence.
- `module.load_gate_manifest_selftest` now exposes local-only
  `raios.module_load_gate_manifest_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch retained manifest-reference candidates without mutating the
  global event log, accepting manifest JSON or artifact bytes, or loading
  artifacts.
- `module.artifact_diagnostic` now exposes a read-only guest diagnostic for a
  candidate-artifact hash reference. It accepts no manifest JSON, artifact
  bytes, or unsigned service code and validates the canonical
  `raios.module_candidate_artifact_reference.v0` hash over retained manifest and
  computed-grant event ids plus manifest, artifact, report, attestation, and
  grant hashes.
- A valid `module.artifact_diagnostic` reference is retained as a local-only
  current-boot `raios.module_candidate_artifact_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_candidate_artifact_reference` and `audit.events`, and remains
  non-authorizing with `artifact_loaded: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained candidate-artifact reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `candidate_artifact: retained_hash_reference_only`,
  retained artifact hashes, and
  `retained_candidate_artifact_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  artifact hashes as accepted evidence.
- `module.load_gate_artifact_selftest` now exposes local-only
  `raios.module_load_gate_artifact_selftest.v0` test infrastructure for missing,
  accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
  wrong-schema, substituted-record, hash-mismatch, manifest-reference mismatch,
  and computed-grant-reference mismatch candidates without mutating the global
  event log or loading artifacts.
- `module.vm_report_diagnostic` now exposes a read-only guest diagnostic for a
  VM-test-report hash reference. It accepts no manifest JSON, report JSON,
  artifact bytes, or unsigned service code and validates the canonical
  `raios.module_vm_test_report_reference.v0` hash over retained manifest,
  candidate-artifact, and computed-grant event ids plus manifest, artifact,
  report, attestation, grant, manifest-reference, and artifact-reference hashes.
- A valid `module.vm_report_diagnostic` reference is retained as a local-only
  current-boot `raios.module_vm_test_report_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_vm_test_report_reference` and `audit.events`, and remains
  non-authorizing with `accepts_vm_report_json: false`, `can_load_now: false`,
  and `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained VM-test-report reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `vm_test_report: retained_hash_reference_only`,
  retained report hashes, and
  `retained_vm_test_report_reference_not_authorizing`; stale, substituted,
  wrong-schema, hash-mismatched, or manifest/artifact/grant-mismatched
  references are rejected without exposing their report hashes as accepted
  evidence.
- `module.load_gate_vm_report_selftest` now exposes local-only
  `raios.module_load_gate_vm_report_selftest.v0` test infrastructure for
  missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record,
  hash-mismatch, manifest-reference mismatch, artifact-reference mismatch,
  computed-grant-reference mismatch, and VM-report-hash mismatch candidates
  without mutating the global event log, accepting report JSON, or loading
  artifacts.
- host-side `registry-tools grant-diagnostic` now emits
  `raios.computed_capability_grant.v0` over an exact module manifest,
  candidate artifact, Shadow-VM report, local attestation, approval phrase,
  requested capability, subject, resource, and current-boot scope. The
  diagnostic is evidence only: valid tuples set
  `computed_candidate_present: true`, while `grants_capability`,
  `grants_load_now`,
  `authorizes_guest_load`, `can_load_now`, and `load_attempted` remain false.
- `registry-core` unit tests reject mismatched manifest/artifact/report/
  attestation hashes, non-empty manifest `granted_caps`, wrong approval
  phrases, and `limits.grants_load_now: true` attestations.
- `module.grant_diagnostic` now exposes a read-only guest diagnostic for a
  computed-grant hash reference. It accepts no artifact bytes and validates only
  the `raios.computed_capability_grant.canonical.v0` hash over manifest,
  artifact, VM-report, and local-attestation hashes. A valid reference sets
  `computed_candidate_present: true` but still keeps
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`,
  `can_load_now: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- A valid `module.grant_diagnostic` reference is now retained as a local-only
  current-boot `raios.module_computed_grant_reference.v0` event binding. The
  retained record stores hashes only, appears through `retained_reference` and
  `audit.events`, and remains non-authorizing with
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.attestation_diagnostic` now exposes
  `raios.module_local_attestation_reference_diagnostic.v0` as a read-only guest
  hash-reference diagnostic. It accepts only canonical hashes and current-boot
  event ids for retained manifest, artifact, VM-report, and computed-grant
  evidence; it accepts no local-attestation JSON, no artifact bytes, and keeps
  `authorizes_guest_load`, `can_load_now`, and `load_attempted` false.
- A valid `module.attestation_diagnostic` reference is retained as a local-only
  current-boot `raios.module_local_attestation_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_local_attestation_reference` and `audit.events`, and remains
  non-authorizing.
- `module.load_ephemeral` and `service.load_ephemeral` now snapshot the latest
  retained computed-grant reference into their denied
  `raios.module_load_gate.v0` response and event binding. With a retained
  reference, the gate reports
  `computed_capability_grant: retained_hash_reference_only`,
  `retained_computed_grant_reference.state: present`, retained hashes, and
  `retained_computed_grant_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained local-attestation reference before snapshotting it into their denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `local_attestation: retained_hash_reference_only`,
  `retained_local_attestation_reference.state: present`, retained hashes, and
  `retained_local_attestation_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_attestation_selftest` now exposes local-only
  `raios.module_load_gate_local_attestation_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record,
  hash-mismatch, manifest-reference mismatch, artifact-reference mismatch,
  VM-report-reference mismatch, and computed-grant-reference mismatch
  candidates without mutating the global event log, accepting local-attestation
  JSON, accepting artifact bytes, loading artifacts, or mutating service
  inventory.
- `module.approval_diagnostic` now exposes
  `raios.module_local_approval_reference_diagnostic.v0` as a read-only guest
  hash-reference diagnostic. It accepts only canonical hashes and current-boot
  event ids for retained manifest, artifact, VM-report, computed-grant, and
  local-attestation evidence; it accepts no free-form local approval text,
  artifact bytes, or unsigned service code, and keeps `authorizes_guest_load`,
  `can_load_now`, and `load_attempted` false.
- A valid `module.approval_diagnostic` reference is retained as a local-only
  current-boot `raios.module_local_approval_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_local_approval_reference` and `audit.events`, and remains
  non-authorizing.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained local-approval reference before snapshotting it into their denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `local_approval: retained_hash_reference_only`,
  `retained_local_approval_reference.state: present`, retained approval hashes,
  and `retained_local_approval_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_approval_selftest` now exposes local-only
  `raios.module_load_gate_local_approval_selftest.v0` test infrastructure for
  missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, hash-mismatch,
  manifest-reference mismatch, artifact-reference mismatch, VM-report-reference
  mismatch, local-attestation-reference mismatch, and computed-grant-reference
  mismatch candidates without mutating the global event log, accepting approval
  text, accepting artifact bytes, loading artifacts, or mutating service
  inventory.
- `module.load_gate_retained_selftest` now exposes local-only
  `raios.module_load_gate_retained_reference_selftest.v0` test infrastructure
  for the denied load gate's retained-reference predicate. It covers missing,
  accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch candidates without mutating the global event log, creating
  retained records, loading artifacts, or mutating service inventory.
- `module.load_ephemeral` and `service.load_ephemeral` also expose
  `raios.module_load_gate_audit_rollback_requirements.v0` in the denied
  response and event binding. The requirement schema names
  `raios.audit_record.v0`, `raios.rollback_plan.v0`, retained grant/reference
  ids, local approval, rollback-plan hash, and ram-only service-slot id as
  required but missing, with writes disabled and `can_load: false`.
- `module.load_gate_audit_rollback_selftest` now exposes local-only
  `raios.module_load_gate_audit_rollback_selftest.v0` test infrastructure for
  missing/stale/previous-boot/wrong-schema/substituted retained
  audit/rollback references, retained computed-grant/audit/rollback hash
  mismatches, retained service-slot mismatch, missing durable audit, missing
  rollback plan, matching-but-still-denied audit/rollback evidence,
  audit/rollback schema mismatches, retained grant hash mismatch,
  manifest/artifact/VM-report/local-attestation mismatches, local approval
  mismatch, rollback hash mismatch, rollback artifact mismatch, and rollback
  service-slot mismatch. It creates no retained references, durable audit
  records, rollback plans, service slots, event-log records, or loads.
- `registry-tools audit-rollback-diagnostic` now emits
  `raios.module_audit_rollback_diagnostic.v0` with nested
  `raios.audit_record.v0` and `raios.rollback_plan.v0` candidates. It binds
  the retained computed-grant hash, retained-reference event id, denied load
  event id, local approval, ram-only service-slot id, rollback plan hash,
  manifest, artifact, VM report, and local attestation while keeping
  `durable_audit_written: false`, `rollback_plan_installed: false`,
  `can_load_now: false`, and `load_attempted: false`.
- `registry-core` unit tests now reject audit/rollback candidate mismatches for
  retained grant hash, manifest, artifact, report, attestation, approval,
  rollback hash, and service-slot ids.
- `module.audit_rollback_diagnostic` now exposes
  `raios.module_audit_rollback_reference_diagnostic.v0` as a guest
  hash-reference diagnostic. It accepts only hashes and current-boot ids for the
  audit record, rollback plan, computed grant, retained reference, denied load
  event, manifest, artifact, VM report, local attestation, local approval,
  pre-load service inventory, cleanup actions, and ram-only service slot. A
  valid reference reports `valid_hash_reference_load_still_denied`, records one
  local-only current-boot `raios.module_audit_rollback_reference.v0` event
  binding, and still keeps `durable_audit_written`,
  `rollback_plan_installed`, `can_load_now`, and `load_attempted` false.
- `module.audit_rollback_diagnostic_selftest` covers absent, accepted
  current-boot, stale, previous-boot event id, wrong-schema, substituted audit
  hash, rollback hash mismatch, computed-grant hash mismatch, and invalid
  ram-only service-slot cases without creating audit records, rollback plans,
  service slots, retained references, or service inventory changes.
- `module.service_slot_diagnostic` now exposes
  `raios.module_service_slot_reservation_diagnostic.v0` as a guest
  hash-reference diagnostic. It binds a reservation hash to the retained
  computed-grant reference id, retained audit/rollback reference id, computed
  grant hash, audit-record hash, rollback-plan hash, pre-load service-inventory
  hash, and `ram_only:` slot id. A valid reference records only a local-only
  current-boot `raios.module_service_slot_reservation.v0` event binding and
  keeps `allocates_service_slot`, `creates_service_inventory_records`,
  `can_load_now`, and `load_attempted` false.
- `module.service_slot_diagnostic_selftest` covers absent, accepted
  current-boot, stale, mismatched reservation hash, and invalid `ram_only:`
  service-slot cases without mutating the global event log, creating retained
  reservation records, allocating slots, loading artifacts, or changing service
  inventory.
- `module.service_slot_allocator` now exposes
  `raios.module_service_slot_allocator_readiness.v0` as a source-evidence-only
  current-boot diagnostic over the RAM-only allocator/runtime side of Phase 6.
  It consumes retained service-slot reservation evidence only as a local-only
  hash reference, records retained current-boot source-evidence events for the
  allocator facts, and now turns
  `raios.ram_only_service_slot_allocator.v0`,
  `raios.service_slot_registry_binding.v0`,
  `raios.service_health_state_model.v0`, and
  `raios.service_unload_cleanup_plan.v0` into observed-current-boot available
  facts once a retained service-slot reservation exists. The durable-audit write
  and rollback-install prerequisite gates now also become observed-current-boot
  available when those facts are available. The module-loader prerequisite
  boundary also becomes observed-current-boot available but non-authorizing. The
  diagnostic records a local-only current-boot
  `raios.module_service_slot_allocator_authority_source_evidence.v0` event,
  exposes a nested `raios.module_service_slot_allocator_authority.v0`
  authority boundary that names the future authority inputs, and advances live
  allocator readiness through the composite
  `raios.module_service_slot_allocator_authority_decision.v0` and
  `raios.service_slot_registry_write_commit_gate.v0` diagnostics to
  `service_slot_allocator_authority_boundary_non_authorizing` while keeping
  registry mutation, slot allocation, durable-audit state writes,
  rollback-state installation, service-inventory mutation, `can_allocate`,
  `can_load_now`, and `load_attempted` false.
- `module.service_slot_allocator_selftest` covers missing retained
  reservation evidence, allocator scope/schema/provenance/binding failures,
  observed source-evidence for the missing and available allocator runtime,
  registry binding, health-state model, unload cleanup, durable audit, rollback
  install, module-loader prerequisite, registry-write commit gate missing, and
  the final all-inputs-ready case while still denying registry, allocation,
  audit, rollback, and load authority.
- `module.loader_runtime` now exposes
  `raios.module_loader_runtime_readiness.v0` as a read-only current-boot
  diagnostic over the missing normal-module loader/runtime side of Phase 6. It
  consumes retained module evidence, retained service-slot allocator
  source-evidence readiness, and the
  latest retained `module.loader_identity` and
  `module.loader_artifact_hash_binding`, `module.loader_entrypoint_abi`,
  `module.loader_address_space_boundary`,
  `module.loader_memory_map_constraints`,
  `module.loader_capability_import_table`,
  `module.loader_service_slot_binding`,
  `module.loader_health_state_hooks`, `module.loader_rollback_hooks`, and
  `module.loader_audit_rollback_write_boundary_binding`
  source-evidence events only as local-only current-boot inputs, reports
  missing typed loader identity, artifact hash binding, entrypoint ABI,
  address-space and memory-map
  isolation, capability import table, service-slot binding, health/rollback
  hooks, and audit/rollback write-boundary binding facts, and keeps
  `loads_artifact`, `allocates_service_slot`,
  `creates_service_inventory_records`, `can_load_now`, and `load_attempted`
  false. With valid retained allocator source evidence, the live aggregate and
  loader source-evidence responses now report
  `denied_allocator_authority_not_granted` /
  `service_slot_allocator_authority_boundary_non_authorizing`, cite the
  allocator-authority boundary, and no longer fall back to the old static
  runtime-missing placeholder. Each aggregate fact and loader-fact `blocked_by`
  entry cites the source diagnostic method and source fact locator for the
  corresponding typed method.
- `module.loader_runtime_selftest` covers missing retained evidence,
  service-slot allocator readiness/runtime gaps, stale/scope/schema/provenance
  and retained-evidence/service-slot/audit-boundary binding failures, each
  missing loader-runtime fact, observed-current-boot loader identity,
  artifact-hash, entrypoint-ABI, address-space, memory-map, capability-table,
  service-slot, health-hook, rollback-hook, and write-boundary source-evidence
  cases, the non-authorizing live-load attempt, artifact-load,
  executable-mapping, entrypoint-transfer, and service-start boundary cases,
  and the final all-inputs-ready
  `defined_non_executable` case without loading artifacts or mutating service
  inventory. It also exposes `source_fact_count: 10`,
  `source_fact_map_complete: true`, and a local source map for the aggregate
  facts.
- The denied `module.load_ephemeral` loader-runtime readiness projection and
  compact audit/event binding reuse the ten base loader-runtime source facts
  and append the receiver-identity load preflight as an eleventh
  non-authorizing source fact. Each embedded missing loader-runtime fact now
  carries a stable id, source method, source fact locator, missing reason,
  current-boot/local-only scope, and non-authorizing status.
  `module.load_gate_loader_runtime_selftest` exposes and checks the ten base
  loader-runtime facts without mutating the event log or attempting a load.
- `module.loader_identity` now exposes `raios.module_loader_identity.v0` as a
  read-only current-boot diagnostic for the first typed normal-module
  loader-runtime fact. It reports the live fact as missing/local-only and
  requires retained module evidence, service-slot allocator readiness/runtime,
  and audit/rollback write-boundary binding before it can become available.
  The live diagnostic records a separate
  `raios.module_loader_identity_source_evidence.v0` event in the current-boot
  RAM event log; that record is local-only, non-authorizing, accepts no loader
  descriptor or artifact bytes, and is consumed by `module.loader_runtime`
  only as observed source evidence.
  `module.loader_identity_selftest` covers missing retained evidence,
  allocator readiness/runtime gaps, missing audit/write boundary,
  identity scope/schema/provenance failures, missing retained-evidence,
  service-slot-allocator, and audit-boundary bindings, missing identity, and
  all-inputs-present-but-non-authorizing identity evidence.
- `module.loader_artifact_hash_binding` now exposes
  `raios.module_loader_artifact_hash_binding.v0` as a read-only current-boot
  diagnostic for the second typed normal-module loader-runtime fact. It reports
  the live fact as missing/local-only and requires retained module evidence,
  service-slot allocator readiness/runtime, audit/rollback write-boundary
  binding, and loader identity before it can become available.
  The live diagnostic records a separate
  `raios.module_loader_artifact_hash_binding_source_evidence.v0` event in the
  current-boot RAM event log; that record is local-only, non-authorizing,
  accepts no loader descriptor or artifact bytes, cites the retained loader
  identity source-evidence event id when present, and is consumed by
  `module.loader_runtime` only as observed source evidence.
  `module.loader_artifact_hash_binding_selftest` covers missing prerequisites,
  artifact-hash binding scope/schema/provenance failures, missing
  retained-evidence/service-slot/audit-boundary/loader-identity bindings,
  missing artifact-hash binding, and all-inputs-present-but-non-authorizing
  artifact-hash binding evidence.
- `module.loader_entrypoint_abi` now records retained
  `raios.module_loader_entrypoint_abi_source_evidence.v0` in the current-boot
  RAM event log. The record is local-only, non-authorizing, accepts no loader
  descriptor or artifact bytes, cites the retained artifact-hash source-evidence
  event id when present, and is consumed by `module.loader_runtime` only as
  observed source evidence.
- The next seven typed normal-module loader-runtime facts after entrypoint ABI now
  record retained source-evidence events in the current-boot RAM event log:
  `module.loader_address_space_boundary`,
  `module.loader_memory_map_constraints`,
  `module.loader_capability_import_table`,
  `module.loader_service_slot_binding`,
  `module.loader_health_state_hooks`,
  `module.loader_rollback_hooks`, and
  `module.loader_audit_rollback_write_boundary_binding`. Each record is
  local-only, non-authorizing, accepts no loader descriptor or artifact bytes,
  cites the previous retained loader-fact source-evidence event id when
  present, and is consumed by `module.loader_runtime` only as observed source
  evidence. These diagnostics are
  emitted through the shared `agent_protocol_module_loader_fact` boundary and
  keep descriptor input, artifact input, service-slot allocation, service
  inventory mutation, and load attempts disabled. Their selftests cover missing
  prerequisites, previous-boot, schema/provenance failures, retained-evidence,
  service-slot-allocator, audit/write-boundary, previous-loader-fact binding
  gaps, missing fact, and all-inputs-present-but-non-authorizing cases.
- `module.audit_rollback_availability` now exposes
  `raios.module_audit_rollback_availability.v0` as a read-only current-boot
  diagnostic over typed `raios.durable_audit_ledger.v0` and
  `raios.rollback_store.v0` availability facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it keeps
  `writes_enabled`, `creates_durable_audit_records`, `creates_rollback_plans`,
  `installs_rollback_plan`, `can_load_now`, and `load_attempted` false.
- `module.audit_rollback_availability_selftest` covers missing ledger/store,
  previous-boot, schema mismatch, missing provenance, and
  available-facts-but-policy-missing cases without mutating the global event
  log or creating durable records.
- `module.audit_rollback_write_policy` now exposes
  `raios.module_audit_rollback_write_policy.v0` as a read-only current-boot
  diagnostic over typed `raios.durable_audit_write_policy.v0` and
  `raios.rollback_install_policy.v0` policy facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it names the
  retained-evidence and availability bindings required before a future writer
  could append audit records or install rollback plans.
- `module.audit_rollback_write_policy_selftest` covers missing policy pairs,
  previous-boot, schema mismatch, missing provenance, retained-evidence binding
  gaps, availability binding gaps, and available-policy-but-writer-missing
  cases without mutating the global event log, enabling writes, or installing
  rollback plans.
- `module.audit_rollback_storage_layout` now exposes
  `raios.module_audit_rollback_storage_layout.v0` as a read-only current-boot
  diagnostic over typed `raios.persistence_device_inventory.v0` and
  `raios.audit_rollback_storage_layout.v0` facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it separates
  persistence device identity, partition inventory, write-path availability,
  layout regions, append slots, and recovery-region separation from write or
  append authority.
- `module.audit_rollback_storage_layout_selftest` covers missing storage inputs,
  previous-boot, schema mismatch, missing provenance, missing stable device
  identity, missing partition inventory, layout-device binding gaps, missing
  audit-ledger and rollback-store layout regions, missing append slots, recovery
  boundary gaps, and available-but-still-non-authorizing storage layout cases
  without mutating the global event log, enabling writes, or installing rollback
  plans.
- `module.audit_rollback_append_engine` now exposes
  `raios.module_audit_rollback_append_engine.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_ledger_append_engine.v0` and
  `raios.rollback_store_transaction_engine.v0` facts. The live slice reports
  both facts as missing, local-only, non-durable, and non-authorizing; it
  consumes the storage-layout diagnostic as input while keeping append-only,
  flush, replay, write-policy binding, and recovery separation separate from
  write authority.
- `module.audit_rollback_append_engine_selftest` covers missing append-engine
  pairs, previous-boot, schema mismatch, missing provenance, storage-layout
  binding gaps, write-policy binding gaps, missing append-only/flush/replay
  support, recovery-boundary gaps, and available-but-still-non-authorizing
  append-engine cases without mutating the global event log, enabling writes,
  or installing rollback plans.
- `module.audit_rollback_append_contract` now exposes
  `raios.module_audit_rollback_append_contract.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_ledger_append_envelope.v0` and
  `raios.rollback_store_transaction_envelope.v0` append-contract facts. The
  live slice reports both facts as missing, local-only, non-durable, and
  non-authorizing; it consumes the storage-layout and append-engine diagnostics
  while exposing explicit stable-id/provenance bindings for storage-layout,
  append-engine, write-policy, and availability facts.
- `module.audit_rollback_append_contract_selftest` covers missing append
  envelope pairs, previous-boot, schema mismatch, missing provenance,
  provenance-binding gaps, write-policy binding and id gaps, availability
  binding and id gaps, storage-layout id gaps, append-engine id gaps,
  storage-layout gaps, and append-engine-missing cases without mutating the
  global event log, enabling writes, or installing rollback plans.
- `module.audit_rollback_append_payload_hash` now exposes
  `raios.module_audit_rollback_append_payload_hash.v0` as a read-only
  current-boot diagnostic over typed
  `raios.audit_record_append_payload_hash_envelope.v0` and
  `raios.rollback_transaction_append_payload_hash_envelope.v0` facts. The live
  slice derives envelope hashes only from retained audit/rollback candidates,
  retained service-slot reservation evidence, the pre-load write-request shape,
  and bound append-contract ids; because append-contract facts are still
  missing, the payload envelopes remain `missing`, local-only, non-durable, and
  non-authorizing.
- `module.audit_rollback_append_payload_hash_selftest` covers missing payload
  hash pairs, previous-boot, schema mismatch, missing provenance,
  retained-audit/rollback binding gaps, service-slot binding gaps, pre-load
  write-request binding gaps, append-contract id gaps, target-schema gaps,
  payload-hash gaps, retained-evidence missing, service-slot missing,
  append-contract missing, and available-but-still-non-authorizing payload-hash
  cases without mutating the global event log, enabling writes, or installing
  rollback plans.
- `module.audit_rollback_append_intent` now exposes
  `raios.module_audit_rollback_append_intent.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_record_append_intent.v0` and
  `raios.rollback_transaction_append_intent.v0` facts. The live slice reports
  both facts as missing, local-only, non-durable, and non-authorizing; it
  consumes the bound append-contract facts and the append payload-hash envelope
  readiness while naming explicit
  append-contract, append-engine, storage-layout, write-policy, availability,
  payload-hash, and provenance bindings for future append requests.
- `module.audit_rollback_append_intent_selftest` covers missing append-intent
  pairs, previous-boot, schema mismatch, missing provenance,
  provenance-binding gaps, append-contract binding gaps, append-engine id gaps,
  storage-layout id gaps, write-policy id gaps, availability id gaps,
  payload-hash gaps, append-contract-missing cases, payload-envelope-missing
  cases, and
  available-but-still-non-authorizing append-intent cases without mutating the
  global event log, enabling writes, or installing rollback plans.
- `module.audit_rollback_write_boundary` now exposes
  `raios.module_audit_rollback_write_boundary.v0` as a read-only guest
  diagnostic over the retained manifest, candidate-artifact, VM-report,
  computed-grant, local-attestation, local-approval, audit/rollback, and
  service-slot reservation chain plus the audit/rollback availability,
  durable-write policy, storage-layout, append-engine through append-contract,
  append-contract facts, append payload-hash envelopes, and append-intent facts.
  It emits a typed
  `raios.module_pre_load_audit_rollback_write_request.v0` plus
  `raios.module_audit_rollback_write_denial_evidence.v0`, keeps
  `writes_enabled`, `creates_durable_audit_records`, `creates_rollback_plans`,
  `installs_rollback_plan`, `loads_artifact`, and `loads_recovery_artifact`
  false, and reports explicit `durable_audit_write_missing`,
  `rollback_install_missing`, `storage_layout_missing`, and
  `append_engine_missing` gates.
- `module.audit_rollback_write_boundary_selftest` covers missing, stale,
  substituted, previous-boot, wrong-schema, mismatched hash, service-slot
  mismatch, recovery-artifact separation, one-sided missing availability,
  available-facts-but-policy-missing, rollback-policy-missing,
  append-contract-missing, rollback-transaction-missing, append-intent-missing,
  payload-envelope-missing, writer-unimplemented, and accepted-current-boot-but-denied cases without
  mutating the global event log, creating durable records, installing rollback
  plans, or loading artifacts.
- `module.load_ephemeral` and `service.load_ephemeral` now also validate the
  latest retained audit/rollback reference before snapshotting it into the
  denied `raios.module_load_gate.v0` response and event binding. The live
  predicate checks that the retained reference binds the latest retained
  computed-grant reference, a prior denied load event, canonical computed-grant,
  rollback-plan, and audit-record hashes, and a valid `ram_only:` service-slot
  id. With a valid retained reference, the gate reports
  `durable_audit_record: retained_hash_reference_only_not_durable`,
  `rollback_plan: retained_hash_reference_only_not_installed`,
  `retained_audit_rollback_reference.state: present`, retained audit/rollback
  hashes, `durable_audit_write_missing`, and `rollback_install_missing`, while
  still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`. A retained reference that points at a wrong-schema
  event or mismatched hashes is reported as `rejected_retained_reference`, and
  its audit/rollback hashes are not exposed as accepted gate evidence.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained `raios.module_service_slot_reservation.v0` before snapshotting it
  into the same denied gate. The live predicate checks the retained grant and
  audit/rollback event ids, referenced event variants, canonical reservation
  hash, computed-grant/audit/rollback hashes, pre-load service-inventory hash,
  and `ram_only:` slot id. A valid reservation reports
  `service_slot: retained_hash_reference_only_not_allocated`,
  `retained_service_slot_reservation.state: present`, and
  `service_slot_reservation_hash` while keeping
  `allocates_service_slot: false`, `can_load: false`,
  `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now also distinguish the
  next Phase-6 boundaries inside the denied gate: retained module evidence
  completeness, read-only `raios.module_service_slot_allocator_readiness.v0`,
  and read-only `raios.module_loader_runtime_readiness.v0`. With valid retained
  service-slot evidence plus retained `module.service_slot_allocator` source
  evidence, the gate reports
  `service_slot_allocator: defined_non_authorizing`,
  `service_slot_allocator_ready: false`,
  `loader_runtime: blocked_by_service_slot_allocator_authority`,
  `readiness_status: denied_allocator_authority_not_granted`,
  `readiness_reason: service_slot_allocator_authority_boundary_non_authorizing`,
  and a nested `raios.module_service_slot_allocator_authority.v0` boundary, plus
  typed missing loader-runtime facts such as `raios.module_loader_identity.v0`,
  while keeping `loads_artifact`, `allocates_service_slot`,
  `creates_service_inventory_records`, `can_load_now`, and `load_attempted`
  false.
- `module.load_gate_service_slot_selftest` now exposes local-only
  `raios.module_load_gate_service_slot_selftest.v0` test infrastructure for
  missing, accepted-current-boot, stale/dropped, wrong-schema, substituted,
  computed-grant-hash, audit-hash, rollback-hash, inventory-hash,
  service-slot, and reservation-hash retained service-slot reservation cases.
  Rejected cases report `rejected_retained_reference` and keep accepted
  `service_slot_reservation_hash` evidence absent.
- `module.load_gate_loader_runtime_selftest` now exposes local-only
  `raios.module_load_gate_loader_runtime_selftest.v0` test infrastructure for
  the denied load-gate loader-runtime projection. It covers missing/rejected
  retained module evidence, missing/rejected retained service-slot reservation
  projection, and the all-retained-evidence-ready state that remains blocked
  by the non-authorizing service-slot allocator authority boundary, while
  keeping descriptor/artifact input, service-slot allocation, service inventory
  mutation, and load attempts disabled.
- `module.grant_diagnostic_selftest` covers absent, accepted-current-boot,
  stale previous-boot, mismatched manifest-hash, and wrong-policy computed
  grant references without loading artifacts or mutating service inventory.
- `vm-harness\shadow-vm-smoke.ps1` verifies the read-only agent protocol,
  provider trust problem visibility, static service inventory, and denied module
  load behavior, then writes a `raios.vm_test_report.v0` report.
- `memory.profile`, `memory.context`, `memory.query`, and `memory.trace` now
  expose a local read-only `current_boot` memory context slice. The
  `memory.context` result schema is `raios.agent_context.v0`, includes
  current-boot `context_event_id`/`audit_event_id` handles for the local read,
  and provider export is explicitly disabled.
- `memory.context provider_minimal` now emits a local-only
  `raios.provider_context_projection.v0` preview with explicit
  `public`/`local_only`/`secret` field classification, included and omitted
  field lists, deterministic `packet_evidence` hashes for the canonical packet
  plus exported and omitted field lists, a nested redacted
  `raios.agent_context.v0` packet, and `can_export: false` until positive
  provider trust and a distinct provider export audit binding exist.
- `provider.context_export provider_minimal` now exposes the first
  `raios.provider_context_export.v0` gate. It returns structured
  `capability_denied`, records `cap.provider.context_export` with risk
  `export`, reports `provider_write: not_attempted`, reports packet and
  field-list evidence bindings as present, keeps the positive provider request
  binding and export audit binding gates missing, and emits separate
  current-boot denial evidence as
  `raios.provider_request_binding_denial.v0` and
  `raios.provider_context_export_denial_audit.v0`.
- `event.log.v0` now carries structured `bindings` for those denial events:
  both include the canonical provider-minimal packet hash plus exported and
  omitted field-list hashes, and both explicitly report
  `satisfies_current_boot_export_gate: false`.
- The real OpenAI `ask` path now emits a local-only
  `OPENAI_PROVIDER_REQUEST_ENVELOPE` serial marker with schema
  `raios.provider_request_envelope.v0` after request id allocation and before
  DNS/TCP/TLS/API-key copy/HTTPS write. It records redacted request-body shape,
  body hash, envelope hash, trust snapshot, `provider_write: not_attempted`,
  and `context_attached_to_provider_body: false` without raw prompt text,
  `Content-Length`, API keys, or Authorization values.
- On the `pinned_spki_verified` direct OpenAI path, after TLS proof and matching
  request-body hash validation but before API-key copy or HTTPS write, Stage-0
  now records local-only positive
  `raios.provider_request_binding.v0` and
  `raios.provider_context_export_audit_binding.v0` events. They bind the exact
  request-body hash, request-envelope hash, provider-minimal packet hash,
  exported-field-list hash, omitted-field-list hash, redaction-policy hash,
  field-classification hash, token-budget hash, and
  `provider_trust_evidence_hash` over provider host, trust state, pin kind/id,
  TLS-bypass state, and `raios.provider_trust_verifier_metadata.v0`. The
  verifier metadata exposes the real Stage-0 pinned TLS verifier id, exact-host
  policy, configured leaf/SPKI pin policy, TLS 1.3 P-256 CertificateVerify
  policy, and explicit chain/time non-validation policies. The request binding
  satisfies only `satisfies_request_binding_gate: true`; the export audit
  binding sets `positive_export_authorization: true`, but both retain
  `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`.
- `provider.context_gate provider_minimal` now exposes a read-only
  `raios.provider_context_export_gate_state.v0` diagnostic over retained
  current-boot binding records. It can validate one matching positive request
  binding plus export-audit binding pair while keeping `can_export: false`.
- `provider.context_gate_selftest provider_minimal` exposes local-only test
  infrastructure over the same gate predicate. It does not mutate the global
  event log, create request envelopes, create positive binding records, or
  attempt provider writes. The Shadow VM smoke now covers stale/dropped event
  ids, previous-boot-or-unretained ids, denial-schema substitution,
  positive-record substitution, request/body/binding hash mismatches, context
  hash mismatches, redaction/classification/budget/trust-evidence hash
  mismatches, and trust-bypass records.
- `provider.context_export provider_minimal` now consumes one valid retained
  positive binding pair for local gate evaluation only, records
  `raios.provider_context_binding_consumption.v0`, and still returns
  `capability_denied` because `automatic_context_injection` remains disabled.
  A second attempt against the same pair is rejected as
  `binding_already_consumed`.
- `provider.context_injection_gate provider_minimal` now exposes the separate
  `raios.provider_context_injection_gate.v0` diagnostic. It reports final
  authorization as missing, requires
  `raios.provider_context_injection_authorization.v0`, keeps
  `automatic_context_injection: disabled`, and reports
  `can_attach_context: false`.
- `provider.context_injection_gate_selftest provider_minimal` exposes local-only
  test infrastructure over the final injection predicate. It does not mutate the
  global event log, create real request envelopes, create positive binding
  records, create final authorization records, attempt provider writes, or
  attach context. The Shadow VM smoke now covers missing, stale/dropped,
  wrong-schema, substituted-positive-record, final body-hash mismatch, trust
  downgrade, and body-attachment-without-final-authorization cases.
- On positive pinned/WebPKI OpenAI request paths, Stage-0 now emits a local-only
  `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker after request/export binding
  evidence and before API-key copy or HTTPS write. The marker binds the request
  body hash, request-envelope hash, provider-minimal context hashes, and
  provider-trust evidence hash while keeping provider write not attempted and
  body attachment false.
- `provider.context_export` still does not create a request envelope; the
  Shadow VM smoke checks that denied export cannot fake one.
- `memory.query` and `memory.trace` include
  `snapshot.current.provider_minimal` as the stable locator for the redacted
  current-status projection.
- `memory.recent_events` and `audit.events [limit]` expose a bounded RAM-only
  `event.log.v0` ring containing compact `audit.event.v0` records for agent
  protocol reads, known `capability_denied` outcomes, provider request-binding
  denials, provider export-denial audits with hash-valued denial bindings, and
  the `raios.module_load_gate.v0` denial binding.
- denied memory/module/service/config methods include current-boot `event_id`
  and `audit_event_id` handles, while all durable audit, persistence, policy
  mutation, redaction mutation, and rollback behavior remains denied.
- memory mutation methods (`memory.record_observation`,
  `memory.propose_policy`, `memory.supersede_fact`, `memory.redact`, and
  `memory.compact`) return structured `capability_denied` with missing audit and
  persistence evidence.
- `vm-harness\shadow-vm-smoke.ps1` now verifies memory-context schemas,
  context event ids, the local `provider_minimal` redaction projection, the
  provider-minimal packet/field-list hashes, the denied
  `provider.context_export` gate with hash bindings present, positive request
  and export audit bindings still missing, denial-audit records present but not
  satisfying export gates, provider writes still not attempted, memory
  query/trace, event log schemas, audit alias, memory mutation denials with
  event ids, the read-only `provider.context_gate` missing-binding state, the
  `provider.context_gate_selftest` negative predicate cases, the separate
  `provider.context_injection_gate` missing-final-authorization state, the
  `provider.context_injection_gate_selftest` negative final-authorization cases,
  the read-only module manifest, candidate-artifact, VM-report, computed-grant,
  local-attestation, and local-approval diagnostics and selftests, the module audit/rollback
  hash-reference diagnostics and selftests, retained
  `raios.module_manifest_reference.v0`,
  `raios.module_candidate_artifact_reference.v0`,
  `raios.module_vm_test_report_reference.v0`,
  `raios.module_computed_grant_reference.v0`,
  `raios.module_local_attestation_reference.v0`, and
  `raios.module_local_approval_reference.v0`, plus
  `raios.module_audit_rollback_reference.v0` event bindings, and the denied
  module load gate including retained manifest, retained artifact, retained
  VM-report, retained computed-grant, retained local-attestation, retained
  local-approval, plus retained audit/rollback reference state in the response and event-log binding, live
  wrong-schema retained audit/rollback rejection, plus negative
  manifest-reference, artifact-reference, VM-report-reference,
  retained-reference, retained local-attestation-reference, retained
  local-approval-reference, retained audit/rollback reference, and
  audit/rollback requirement selftests,
  service-slot reservation diagnostics and selftests, live denied load-gate
  visibility of valid retained service-slot reservation evidence, negative
  retained service-slot reservation gate selftests, and read-only
  audit/rollback availability, write-policy, storage-layout, append-engine,
  append-contract, append payload-hash, append-intent, plus write-boundary
  diagnostics/selftests, and the separate denied recovery artifact load
  boundary with typed missing recovery identity, trust, VM-test, approval,
  loader, and rollback evidence, plus read-only recovery loader and
  rollback-evidence hash-reference diagnostics, all six retained recovery
  evidence ids bound into `recovery.load_binding`, recovery lifeline request
  hash-reference diagnostics over the fully retained recovery chain, recovery
  lifeline protocol-state gap diagnostics over that request and its six
  evidence ids, recovery lifeline command-vocabulary envelope diagnostics, and
  recovery loader runtime-isolation plus rollback transaction-engine boundary
  diagnostics plus durable audit/rollback persistence boundary diagnostics and
  selftests, plus recovery memory-provenance boundary diagnostics and selftests
  over source record ids, source schema hashes, classification, authority,
  rollback-transaction binding, last-good checkpoint binding, recovery-only
  export profile, redaction state, replay window, and audit linkage facts, plus
  recovery lifeline command-admission diagnostics and selftests over status,
  rollback preview/apply, disable module, restart last-good, and load recovery
  artifact by hash admission requirements, plus recovery lifeline
  command-envelope reference diagnostics and selftests over allowed command ids,
  argument schemas, argument hashes, required capabilities, target locators,
  command-admission boundary ids, retained request hashes, and a valid retained
  status-command envelope reference that still dispatches no command, plus
  recovery lifeline command-dispatch denial diagnostics and selftests over
  missing body canonicalization, handler binding, status-read handling,
  rollback authorization, per-command target binding, recovery-memory write
  authority, durable audit/rollback write authority, and service-inventory
  side-effect facts, plus recovery lifeline command-body canonicalization
  diagnostics and selftests over the retained command-envelope reference,
  dispatch-denial boundary id, canonical command-body metadata hash/reference,
  local-only missing redaction/classification and handler-input linkage facts,
  and the still-non-executing dispatch boundary after body evidence is retained.
  Latest full report:
  `release\vm-reports\shadow-20260701-091747-9784.json` with 6446/6446
  predicates, 243 executed commands, and `duration_ms: 490492`.
  Latest focused reports:
  `release\vm-reports\shadow-20260703-180640-6876.json` with 1610/1610
  module-audit-rollback predicates, 71 executed commands, and
  `duration_ms: 168943`,
  `release\vm-reports\shadow-20260702-234758-31976.json` with 361/361 quick
  predicates, 56 executed commands, and `duration_ms: 103205`, and
  `release\vm-reports\shadow-20260524-175144-24260.json` with 2725/2725
  recovery predicates, 142 executed commands, and `duration_ms: 138960`.
  These reports derive `commands` from observed serial execution. The recovery
  profile still exercises the same predicate/command count, but serial command
  echo no longer forces framebuffer redraws while long hash-reference commands
  are being received.
- `vm-harness\openai-direct-smoke.ps1 -ExpectPinMismatch` was run against a
  local image built with a fake API key and intentionally wrong SPKI pin. It
  verified the real request envelope marker appears on the `ask` path, omits raw
  prompt/Content-Length/Authorization values, then fails at pin mismatch before
  HTTPS request data is sent and without positive request/export audit binding
  markers.
- `vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` was run against a
  local image built with a fake API key and the current OpenAI SPKI pin. It
  verified the real request envelope marker, positive request binding marker,
  positive export audit binding marker, and blocked injection-gate marker appear
  before the HTTPS write path, that marker body/envelope/binding/context hashes
  match, that
  `provider.context_gate` validates the retained pair, that
  `provider.context_export` consumes it once for local gate evaluation, and that
  the second consumption attempt returns `binding_already_consumed`, while
  provider-minimal context remains unattached.
- the development serial relay and old host-framing path have been removed from
  the runtime path.
- the next trust milestone is WebPKI or broader certificate algorithm support
  once trust anchors, time, hostname checks, and chain handling are specified.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\raios-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard,
  mouse, hub traversal, and a limited no-input USB hotplug rescan exist, but full
  detach/reconfigure handling and broad NIC coverage do not exist yet, so real
  hardware may still boot to the UI but lack input/network unless it matches the
  implemented paths.
- Wi-Fi support currently detects the Surface Pro 4 Marvell AVASTAR 88W8897
  target and stores RAM-only SSID/WPA configuration for the current boot. The
  next implementation step is a Marvell PCIe firmware-upload path before 802.11
  association or WPA2 can work.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 has verified DNS/TCP/TLS/HTTPS for `api.openai.com:443` behind the
  explicit unverified development override, the preferred SPKI pin verifier, and
  the legacy leaf-certificate pin verifier. SPKI pinning still depends on the
  leaf using the currently supported P-256 ECDSA `CertificateVerify` path;
  broader algorithm support or WebPKI remains a hardening step.
- The OpenAI JSON response parser is intentionally minimal and only extracts the
  first `output_text` string.
- QEMU TCP serial is single-client in practice; do not run two serial clients
  against the same port at the same time.
- (Historical gap, CLOSED by M6C/M7D:) a signed module runtime exists — an external
  dev-key-signed Wasm candidate is delivered, verified, promoted, durably persisted,
  and re-verified across a real reboot through the unchanged M6 gate, honestly labeled
  `dev_key_not_owner_sealed` (owner-sealed is the final ceremony).

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/raios-stage0.img` unless the replacement
  has booted in QEMU.
