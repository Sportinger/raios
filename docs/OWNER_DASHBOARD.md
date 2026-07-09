# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: Surface WiFi firmware bring-up now yields much sooner
between Marvell poll bursts: each firmware/HW_SPEC/SCAN_EXT poll pass is capped
at 32 actions every 1ms with 10us waits instead of the old 128-action bursts.
This restores firmware-copy throughput after the too-slow 16-action build while
still reducing long mouse/UI stalls during Start WiFi FW.

After firmware and HW_SPEC are ready, `Scan networks` still issues the real
mwifiex `SCAN_EXT` 2.4GHz wildcard command and reports command status. Live
network results still wait on event/Rx-ring parsing; no association/link
authority is claimed.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. `ownerkey` also reports the next TPM register raiOS would read.
Persistent install remains policy-only; no persistent key, owner seal, load
authority, or durable-write authority is granted.

Latest focused proof: `quick` `shadow-20260709-134429-25500.json` passed
542/542 for the corrected 1ms/32-action Marvell poll-budget image.
Latest owner-key image proof remains `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` 253/253 against the exact default image.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused quick
profile per aggressive-fast cadence.

Next owner action: boot the refreshed USB on the Surface Pro 4, press Start
WiFi FW, check whether the pointer is less laggy during the ~5s firmware phase,
then press Scan networks and send a photo/log of the `SCAN_EXT` line. In
parallel, `ownerkey` capture still gives the next TPM status-read evidence.
