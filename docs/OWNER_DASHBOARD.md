# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: Surface WiFi firmware bring-up now arms the Marvell
firmware event ring before `DRV_READY` and exposes a real `EVENT_RING` status
line with rd/wr pointers, transfer type, event cause, length, and host
interrupt status.

After firmware and HW_SPEC are ready, `Scan networks` still issues the real
mwifiex `SCAN_EXT` 2.4GHz wildcard command and reports command status. If a
firmware event appears, the UI now advances honestly to "scan event observed;
rx ring not implemented". Live network names still wait on Rx-ring parsing; no
association/link authority is claimed.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. `ownerkey` also reports the next TPM register raiOS would read.
Persistent install remains policy-only; no persistent key, owner seal, load
authority, or durable-write authority is granted.

Latest focused proof: `quick` `shadow-20260709-141710-13408.json` passed
542/542 for the Marvell event-ring observation image.
Latest owner-key image proof remains `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` 253/253 against the exact default image.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused quick
profile per aggressive-fast cadence.

Next owner action: boot the refreshed USB on the Surface Pro 4, press Start
WiFi FW, press Scan networks, and send a photo/log of the `SCAN_EXT` plus
`EVENT_RING` lines. In parallel, `ownerkey` capture still gives the next TPM
status-read evidence.
