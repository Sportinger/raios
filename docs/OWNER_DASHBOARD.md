# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-10.

Current capability: the normal release image now boots directly into the calm
Genesis shell: Conversation, Context, safe AI/WiFi setup and a Composer. The
former technical mode renderer is gone; serial chat and the real guided WiFi
path remain connected through their shared action/state path.

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

Genesis status: A0/B0 plus A1 are live locally. The verified no-secret 1280x800
capture is `target/captures/genesis-shell-a1.png`; it honestly says Personal
shell not created and Vault not configured. The proof personal shell remains
unlinked test infrastructure.

Vault/store status: C1 now also validates the exact redundant GPT/type/label/
GUID admission boundary. C3 adds bounded TPM command/transport and RR1 wrapper
custody, still `NotProven`. No physical or durable secret claim is open until
the QEMU AHCI flush/format/append/reboot evidence lands.

Vault crypto status: C2's exact AES-GCM/HKDF envelope, recovery-wrapper, zeroizing
owners, and two-consumer evaluator pass 379 core tests; no plaintext lease or use
path is wired into WiFi/provider yet.

Latest proof: 386/386 raios-core host tests, release package, and an inspected
safe no-secret Genesis capture. The latest quick Shadow VM remains the unchanged
`shadow-20260710-024402-2584.json` 542/542 baseline.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: execute the QEMU-only C1 AHCI structured-store vertical slice with
real flush/format/open/append/reboot proof. Disk 2 stays untouched until the
owner returns; do not claim provider access or durable secrets before evidence.
