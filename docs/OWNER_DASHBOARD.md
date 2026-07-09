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

Stick evidence: host inspection was valid 84/84 with no torn tail; the successful
Surface screen now reports verified `MSC LOG seq85`.

WiFi status: Surface-proven firmware, HW_SPEC, live SSIDs, selection, and input
are green. Association, WPA2 `PORT_RELEASE`, PFU data, and DHCP are implemented;
their first positive bare-metal proof is the current test.

Hub/input status: Surface-proven stable through the full guided flow at 65.7s
uptime with `KBD READY`, `MOUSE READY`, `ERR 0`, and no replug.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: 335/335 core host tests and quick Shadow VM
`shadow-20260710-010658-31684.json` 542/542. The new Disk 2 handoff is pending.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot Disk 2 and capture association status/AID, WPA2 `PORT_RELEASE`,
RX/TX pointer movement, and DHCP. Do not claim provider access before that proof.
