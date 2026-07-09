# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now use a prepared persistence stick as real xHCI
USB Mass Storage, validate GPT `SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA`, validate
`RAIOS_DATA_SB_V0`, append one local-only diagnostic frame to
`SEED_DATA/RECLOG` with SCSI `WRITE(10)`, read it back, reparse it, and show
`MSC LOG seq<N> lba<N>` only after verification.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick prep: Disk 2 (`USB SanDisk 3.2Gen1`) was written with the real GPT
persist layout: `SEED_ESP_A`, `SEED_ESP_B`, and `SEED_DATA` validated. The
next write should refresh it with the RECLOG append image for Surface testing.

WiFi status: Surface Marvell firmware bring-up and `SCAN_EXT` command are real,
but RX-PFU is parked because it froze the Surface and made MMIO read all-ones.
Do not re-enable RX-PFU while adding stick logging.

Hub/input status: the Surface confirmed the hub-mouse rearm path fires, but the
mouse can still break again. Stick RECLOG diagnostics now give us host-readable
evidence instead of phone photos.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: focused USB-storage VM serial observed
`usb-msc: reclog append seq=1 lba=526352 verified` and
`status USB-XHCI ... MSC LOG seq1 lba526352`; host inspect reported RECLOG
`valid`, `count=1`; quick Shadow VM `shadow-20260709-174348-9768.json` passed
542/542.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: write the new image to Disk 2, boot the Surface, reproduce the
mouse/WiFi issue, then inspect `SEED_DATA/RECLOG` on the host.
