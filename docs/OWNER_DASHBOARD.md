# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now issue a real 2.4 GHz wildcard `SCAN_EXT`
command with PCI bus mastering open only from the doorbell through first
`CMD_DONE` (or timeout), then return to DMA quarantine.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: after the successful HW_SPEC boot, Disk 2 RECLOG is a valid
58-frame chain with no torn tail. The new tail is `seq=58 reason=boot_probe`;
WiFi completion remains visible UI evidence rather than a RECLOG record.

WiFi status: firmware-ready and `GET_HW_SPEC` are Surface-proven with responsive
input. The new image extends the same DMA bound to `SCAN_EXT`; event/RX rings,
RX-PFU, live SSIDs, and link authority remain parked pending this hardware test.

Hub/input status: new images no longer poll hub child ports through EP0 after
the first real hub-mouse report, which targets the owner's "same short time
after movement" freeze pattern. Root-port hotplug and the targeted recovery
reset still remain.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-221452-16344.json` passed 542/542
and 28/28 focused Marvell tests passed. Disk 2 now has the bounded SCAN_EXT
kernel SHA `41CD3F08...`; positive scan evidence still requires the Surface.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: on the Surface start WiFi, wait for green HW_SPEC, click Scan once,
and confirm green `SCAN_EXT: done ... len=113` plus a responsive mouse. Do not
expect SSIDs until the following event-result slice.
