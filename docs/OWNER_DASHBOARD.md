# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-10.

Current capability: the normal release image boots into calm Genesis with
Conversation, typed Context, a Composer, RAM-only masked AI/WiFi setup, and an
openable cached Recovery view. A deliberate signed proof can enter the bounded
personal surface, accept sanitized input, leave by F12, and automatically fall back
to Genesis after trap/fuel; it is not installed as the default shell.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, unsupported WiFi security,
and provider access before positive Surface link/DHCP evidence.

Stick status: currently unplugged by the owner. No physical write is permitted
or attempted; the prior `SEED_ESP_A` evidence remains historical only.

WiFi status: Surface-proven firmware, HW_SPEC, live SSIDs, selection, and input
are green. Association, WPA2 `PORT_RELEASE`, PFU data, and DHCP are implemented;
their first positive bare-metal proof is the current test.

Hub/input status: Surface-proven stable through the full guided flow at 65.7s
uptime with `KBD READY`, `MOUSE READY`, `ERR 0`, and no replug.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Genesis status: A0/B0, A1, A2, I2/G3 and AB/G4 are verified. The signed proof runs
only on its exact six bounded UI imports in a fresh metered Wasm instance; F12,
trap/fuel fallback, dynamic inventory removal, and the byte-identical secure strip
are proven by `shadow-20260710-124838-24564.json` (206/206, five captures). The
release still says Personal shell not created and Vault not configured.

Vault/store status: C1 is now proven end-to-end on a fresh dedicated QEMU image:
exact GPT admission, blank format, dual superblock readback, real ATA flush,
committed append and a second-boot replay (`shadow-20260710-032738-34812`, 9/9).
C3 still reports TPM `NotProven`; no physical or durable secret claim is open.

Vault crypto status: C2's exact AES-GCM/HKDF envelope, recovery-wrapper, zeroizing
owners, and two-consumer evaluator pass 379 core tests. C4's unarmed composition
foundation adds readback-verified keyring restore, typed ciphertext records, and
complete-history-only nonce reconstruction; no Vault set/unlock/decrypt, plaintext
lease, WiFi/provider use, audit, or physical-target path is wired.

Latest proof: focused Genesis UI `shadow-20260710-124838-24564.json` passed 206/206
with signed personal-shell entry/input/F12/trap/fuel evidence and five bound PNG
captures. C1's structured-store QEMU proof remains 9/9. The latest quick Shadow VM
remains the unchanged `shadow-20260710-024402-2584.json` 542/542 baseline.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used its focused
Genesis UI evidence; the full baseline is unchanged per aggressive-fast cadence.

Latest C4 proof: release build plus focused structured-store regression
`shadow-20260710-040559-24348.json` passed 9/9 with zero failures; it does not arm
Vault authority.

Next task: I3/G5.4 performs the separately gated Vault/Broker review and joins only
the named durable identity-bound secret paths. Disk 2 stays untouched; do not claim
provider access or durable secrets before evidence.
