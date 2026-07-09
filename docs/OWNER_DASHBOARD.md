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

Stick prep: `scripts\write-stage0-usb.ps1 -UsePersistLayout` writes the real
GPT stick layout. The kernel now has the read-only half needed before appending
logs to that stick.

WiFi status: Surface Marvell firmware bring-up and `SCAN_EXT` command are real,
but RX-PFU is parked because it froze the Surface and made MMIO read all-ones.
Do not re-enable RX-PFU while adding stick logging.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: focused USB-storage VM serial log observed
`usb-msc: ... seed_data=present seed_data_superblock_validated` and
`status USB-XHCI ... MSC SEED`. Quick Shadow VM
`shadow-20260709-163357-10832.json` passed 542/542, 79 commands,
report sha256 `ec6133b2e609fb81a1b5375f6b2599c16a59b5af338b878479a250580dffb3c6`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: add the first scoped USB `WRITE(10)` path that can append exactly one
typed WiFi diagnostic frame into `SEED_DATA/RECLOG`, then read back and reparse
it.
