# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-10.

Current capability: the normal release image boots into calm Genesis with
Conversation, typed Context, a Composer, RAM-only masked AI/WiFi setup, and an
openable cached Recovery view. Genesis Recovery can restart or disable the fixed
demo echo through the same typed Lifeline executor as serial; no second authority
or visual fallback was added.

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

Genesis status: A0/B0, A1 and A2 core interaction are verified. The inspected
no-secret 1280x800 capture is `target/captures/genesis-shell-a2.png`; the release
still says Personal shell not created and Vault not configured. The proof personal
shell remains unlinked test infrastructure.

Vault/store status: C1 is now proven end-to-end on a fresh dedicated QEMU image:
exact GPT admission, blank format, dual superblock readback, real ATA flush,
committed append and a second-boot replay (`shadow-20260710-032738-34812`, 9/9).
C3 still reports TPM `NotProven`; no physical or durable secret claim is open.

Vault crypto status: C2's exact AES-GCM/HKDF envelope, recovery-wrapper, zeroizing
owners, and two-consumer evaluator pass 379 core tests. C4's unarmed composition
foundation adds readback-verified keyring restore, typed ciphertext records, and
complete-history-only nonce reconstruction; no Vault set/unlock/decrypt, plaintext
lease, WiFi/provider use, audit, or physical-target path is wired.

Latest proof: focused Genesis UI `shadow-20260710-034302-30252.json` 181/181,
release package, and inspected safe no-secret Genesis capture. C1's structured-
store QEMU proof remains 9/9. The latest quick Shadow VM remains the unchanged
`shadow-20260710-024402-2584.json` 542/542 baseline.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used its focused
Genesis UI evidence; the full baseline is unchanged per aggressive-fast cadence.

Latest C4 proof: release build plus focused structured-store regression
`shadow-20260710-040559-24348.json` passed 9/9 with zero failures; it does not arm
Vault authority.

Next task: ADR 0013 restores a small tracked descriptor signer after the ignored
local helper was lost. Its host test accepts exact bytes and rejects altered bytes;
it is local `dev_key_not_owner_sealed` provenance only, not OTA or runtime
authority. I2/G3 can now sign its two proof descriptors and integrate the six
approved UI imports. The Vault/Broker join remains separately Sol-review-gated.
Disk 2 stays untouched until the owner returns; do not claim provider access or
durable secrets before evidence.
