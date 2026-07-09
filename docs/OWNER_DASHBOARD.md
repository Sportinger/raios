# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now keep the Surface mouse stable through the
Linux-matched Marvell `DRV_READY` transition and continue into a bounded
firmware-ready MMIO poll while WiFi DMA/INTx remain disabled.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: Disk 2 RECLOG is a valid 57-frame chain. Its latest frame is
`hub_mouse_port_reset_failed` with `reports=179`, `errors=0`, `last_xfer_cc=1`,
`m_port=259`, and `m_ep=1`: mouse reports stopped without a recorded xHCI
transfer error.

WiFi status: the old quarantine used `0xC3C` as though it were the host
interrupt-enable register and cleared status with the wrong polarity. The new
image keeps that corrected sequence, leaves BAR memory readable, and accepts
firmware-ready only when `FW_STATUS=0xFEDCBA00`. `GET_HW_SPEC`, scan/link DMA,
and RX-PFU remain parked.

Hub/input status: new images no longer poll hub child ports through EP0 after
the first real hub-mouse report, which targets the owner's "same short time
after movement" freeze pattern. Root-port hotplug and the targeted recovery
reset still remain.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: the owner confirmed the corrected interrupt-mask image preserves
the hub mouse after `Start WiFi FW`. Quick Shadow VM
`shadow-20260709-214736-29904.json` passed 542/542; Disk 2 `SEED_ESP_A` now has
the DMA-off firmware-status image with kernel SHA `6EE48EEF...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot the Surface, click `Start WiFi FW` once, and confirm
`ready@ready`, `FW_STATUS=0xFEDCBA00`, and a responsive mouse. That evidence
unlocks one bounded `GET_HW_SPEC` DMA window before live scanning.
