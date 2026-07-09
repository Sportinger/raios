# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now write Marvell `DRV_READY` only inside an
even earlier quarantine boundary that disables WiFi DMA/INTx before the ready
write and then quiesces the WiFi PCI function.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: Disk 2 RECLOG readback worked on host. The latest run reached
`seq15=hub_mouse_port_reset` with `reports=11`, `m_port=259`, `m_chg=0`, and
`m_ep=1`, showing the hub port and xHCI endpoint still looked healthy while
reports had stopped.

WiFi status: Surface Marvell firmware block download is still real, and the
next image disables WiFi bus mastering and INTx before writing `DRV_READY`, then
turns off I/O space and memory space. Firmware-ready, `GET_HW_SPEC`, scan/link
authority, and RX-PFU are still denied.

Hub/input status: new images no longer poll hub child ports through EP0 after
the first real hub-mouse report, which targets the owner's "same short time
after movement" freeze pattern. Root-port hotplug and the targeted recovery
reset still remain.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-210923-18096.json` passed
542/542 after moving DMA/INTx quarantine before `DRV_READY`; report sha256
starts `39e03c5b...`. Disk 2 `SEED_ESP_A` is refreshed with kernel SHA
`F5CBFFFC...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: boot the Surface, click `Start WiFi FW`, confirm
`DRV_READY written after DMA/INTx pre-quarantine`, and see whether the hub mouse
survives the pre-quarantined ready transition.
