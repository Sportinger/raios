# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-10.

Current capability: the guided Surface flow now continues from a selected live
open or WPA2-PSK/CCMP network through real Marvell association, secure-port
release, Ethernet RX/TX, and DHCP instead of stopping after password entry.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, unsupported WiFi security,
and provider access before positive Surface link/DHCP evidence.

Stick evidence: host inspection was valid 84/84 with no torn tail; Disk 2
`SEED_ESP_A` now carries kernel SHA `9871578A46BDE75F...` without reformatting.

WiFi status: Surface-proven firmware, HW_SPEC, live SSIDs, selection, and input
are green. Association, WPA2 `PORT_RELEASE`, PFU data, and DHCP are implemented;
their first positive bare-metal proof is the current test.

Hub/input status: Surface-proven stable through the full guided flow at 65.7s
uptime with `KBD READY`, `MOUSE READY`, `ERR 0`, and no replug.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Genesis status: the architecture and exact personal-shell boundary are owner-
approved in ADR 0011. A0/B0 now supplies responsive Genesis geometry, an
atomic display-list validator, exact evidence-bound future `ui.*` grants, and
a stateless proof-guest source; the new visible shell/runtime is not verified yet.

Vault/store status: C1's identity-bound PREPARE/DATA/COMMIT/TOMBSTONE codec,
readback port, and disposable GPT fixture are ready; no physical or durable secret
claim is open until the AHCI join and focused VM evidence land.

Latest proof: 351/351 raios-core host tests, release build, secret scan, quick
Shadow VM `shadow-20260710-020632-29720.json` 542/542, and verified Disk 2 copy hash.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot Disk 2 and capture association status/AID, WPA2 `PORT_RELEASE`,
RX/TX pointer movement, and DHCP. Do not claim provider access before that proof.
