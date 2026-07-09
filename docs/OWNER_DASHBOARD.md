# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: Surface WiFi firmware bring-up now yields much sooner
between Marvell poll bursts: each firmware/HW_SPEC/SCAN_EXT poll pass is capped
at 16 actions with 10us waits instead of 128 actions with 20us waits. That
should reduce mouse/UI lag during Start WiFi FW while preserving the same
fail-closed scan/link authority posture.

After firmware and HW_SPEC are ready, `Scan networks` still issues the real
mwifiex `SCAN_EXT` 2.4GHz wildcard command and reports command status. Live
network results still wait on event/Rx-ring parsing; no association/link
authority is claimed.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. `ownerkey` also reports the next TPM register raiOS would read.
Persistent install remains policy-only; no persistent key, owner seal, load
authority, or durable-write authority is granted.

Latest focused proof: `quick` `shadow-20260709-133204-7396.json` passed
542/542 for the Marvell poll-budget image and host smoke timeout update.
Latest owner-key image proof remains `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` 253/253 against the exact default image.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused quick
profile per aggressive-fast cadence.

Next owner action: boot the refreshed USB on the Surface Pro 4, press Start
WiFi FW, check whether the pointer is less laggy during the ~5s firmware phase,
then press Scan networks and send a photo/log of the `SCAN_EXT` line. In
parallel, `ownerkey` capture still gives the next TPM status-read evidence.
