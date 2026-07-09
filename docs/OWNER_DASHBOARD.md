# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now see a prepared persistence stick as real xHCI
USB Mass Storage, configure BOT bulk endpoints, read sectors with SCSI
`READ(10)`, validate GPT `SEED_ESP_A`/`SEED_ESP_B`/`SEED_DATA`, validate the
`RAIOS_DATA_SB_V0` superblock copy, and show `MSC SEED` in USB status.

What is still denied: USB `WRITE(10)`, RECLOG append, durable WiFi logs, broad
disk mutation, owner-sealed persistence, and live WLAN result/link authority.

Stick prep: Disk 2 (`USB SanDisk 3.2Gen1`) was written with the real GPT
persist layout: `SEED_ESP_A`, `SEED_ESP_B`, and `SEED_DATA` validated. The
kernel now has the read-only half needed before appending logs to that stick.

WiFi status: Surface Marvell firmware bring-up and `SCAN_EXT` command are real,
but RX-PFU is parked because it froze the Surface and made MMIO read all-ones.
Do not re-enable RX-PFU while adding stick logging.

Hub/input status: a test image now re-arms a silent mouse interrupt endpoint
behind a USB hub after it has worked once and then stopped. Needs real Surface
confirmation; QEMU does not reproduce that stall.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: real Disk 2 write log
`raios-usb-write-disk2-20260709-172741.log` ended with
`SEED_DATA superblock valid: True`; quick Shadow VM
`shadow-20260709-172354-28840.json` passed 542/542. Focused USB-storage VM
serial log observed
`usb-msc: ... seed_data=present seed_data_superblock_validated` and
`status USB-XHCI ... MSC SEED`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: add the first scoped USB `WRITE(10)` path that can append exactly one
typed WiFi diagnostic frame into `SEED_DATA/RECLOG`, then read back and reparse
it.
