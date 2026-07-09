# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: on the Surface path, after Marvell firmware and HW_SPEC are
ready, `Scan networks` now issues a real mwifiex `SCAN_EXT` 2.4GHz wildcard
command through the PCIe command mailbox and reports its command status. Live
network results still wait on event/Rx-ring parsing; no association/link
authority is claimed.

Owner-key capture still shows the next TPM register raiOS would read.
`ownerkey` and `system.honesty_report.owner_key_provisioning` report whether a
read-only TPM status-register plan exists, the register kind, physical address,
width, and reason.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. Persistent install remains policy-only; no persistent key, owner
seal, load authority, or durable-write authority is granted.

Latest focused proof: `quick` `shadow-20260709-125408-23664.json` passed
542/542 for the new boot image after the WiFi scan mailbox change. Latest
owner-key image proof remains `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` 253/253 against the exact default image.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused quick
profile per aggressive-fast cadence.

Next owner action: boot the refreshed USB on the Surface Pro 4, press Start
WiFi FW, wait for HW_SPEC ready, then press Scan networks and send a photo/log
of the `SCAN_EXT` line. In parallel, `ownerkey` capture still gives the next TPM
status-read evidence.
