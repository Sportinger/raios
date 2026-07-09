# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now attempt Marvell `DRV_READY` with the actual
host-interrupt source masked and pending status cleared using Linux mwifiex
register semantics, while WiFi DMA/INTx remain pre-quarantined.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: Disk 2 RECLOG is a valid 57-frame chain. Its latest frame is
`hub_mouse_port_reset_failed` with `reports=179`, `errors=0`, `last_xfer_cc=1`,
`m_port=259`, and `m_ep=1`: mouse reports stopped without a recorded xHCI
transfer error.

WiFi status: the old quarantine used `0xC3C` as though it were the host
interrupt-enable register and cleared status with the wrong polarity. The new
image uses `HOST_INT_MASK=0xC34`, Linux's status-mask value, and Linux's
write-zero-to-clear behavior before `DRV_READY`. Firmware-ready, `GET_HW_SPEC`,
scan/link authority, and RX-PFU are still denied.

Hub/input status: new images no longer poll hub child ports through EP0 after
the first real hub-mouse report, which targets the owner's "same short time
after movement" freeze pattern. Root-port hotplug and the targeted recovery
reset still remain.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-213525-25680.json` passed
542/542; report sha256 starts `1aa7f61f...`. Disk 2 `SEED_ESP_A` is refreshed
without reformatting with kernel SHA `EFB49EF8...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot the Surface, click `Start WiFi FW` once, and see whether the hub
mouse survives. If it still fails, inspect MSI/MSI-X and upstream bridge state
before trying another `DRV_READY` variant.
