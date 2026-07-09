# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: Surface WiFi firmware bring-up now uses a larger bounded
firmware-download burst instead of throttling every helper block through the
1ms UI scheduler. The firmware-ready and scan paths are event-ring-only again:
the RX-PFU images froze the Surface, and the scan-time attempt made MMIO read
back all-ones (`HOST_INT=0xffffffff`), so that path is parked for now.

Stick persistence prep: the USB writer now has `-UsePersistLayout`, which writes
the real GPT `SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA` image with valid empty RECLOG
to a USB disk. Kernel USB Mass Storage read/write is still the next missing
piece before raiOS can append WiFi diagnostics to that stick itself.

After firmware and HW_SPEC are ready, `Scan networks` still issues the real
mwifiex `SCAN_EXT` 2.4GHz wildcard command and reports command status. If a
firmware event appears, the UI now reports the raw `EVENT_RING` rd/wr/type/
cause/len state. Empty buffers are amber diagnostics, not fake live results.
Live network names still wait on real frame parsing; no association/link
authority is claimed.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. `ownerkey` also reports the next TPM register raiOS would read.
Persistent install remains policy-only; no persistent key, owner seal, load
authority, or durable-write authority is granted.

Latest focused proof: `quick` `shadow-20260709-155505-32448.json` passed
542/542 for the event-only RX-PFU rollback image.
Latest owner-key image proof remains `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` 253/253 against the exact default image.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused quick
profile per aggressive-fast cadence.

Next owner action: keep testing WiFi with the current event-only USB unless we
explicitly rewrite the stick with `-UsePersistLayout`; send `SCAN_EXT` and
`EVENT_RING`. Kernel USB-MSC block support is the next persistence slice.
