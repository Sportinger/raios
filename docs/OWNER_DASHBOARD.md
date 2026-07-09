# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now use the prepared persistence stick as a real
xHCI USB Mass Storage diagnostic sink, then escalate the Surface hub-mouse
watchdog from endpoint rearm to one targeted hub-port reset/re-enumeration when
the rearm does not restore reports.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: Disk 2 RECLOG readback worked on host. It contained 11 valid
frames: `seq1=boot_probe`, then `seq2..11=hub_mouse_rearm`; reports stayed
stuck at 23, proving the old endpoint-only rearm fired but did not recover the
mouse.

WiFi status: Surface Marvell firmware bring-up and `SCAN_EXT` command are real,
but RX-PFU is parked because it froze the Surface and made MMIO read all-ones.
Do not re-enable RX-PFU while adding stick logging.

Hub/input status: new images now log `hub_mouse_port_reset` or
`hub_mouse_port_reset_failed` after two unsuccessful endpoint rearms, using the
existing hub hotplug enumeration path for exactly that mouse port.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: host inspect of the real stick reported RECLOG `valid`,
`count=11`; quick Shadow VM `shadow-20260709-184742-22644.json` passed 542/542.
Disk 2 `SEED_ESP_A` is refreshed with kernel SHA `C0E4DA64...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot the Surface from Disk 2, reproduce the mouse loss, then inspect
`SEED_DATA/RECLOG` for `hub_mouse_port_reset` and whether mouse reports advance
again.
