# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now reach firmware-ready and complete one real
Surface `GET_HW_SPEC` request while PCI bus mastering is enabled only from the
doorbell through first `CMD_DONE` (or the three-second timeout).

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and live WLAN result/link
authority.

Stick evidence: after the successful HW_SPEC boot, Disk 2 RECLOG is a valid
58-frame chain with no torn tail. The new tail is `seq=58 reason=boot_probe`;
WiFi completion remains visible UI evidence rather than a RECLOG record.

WiFi status: the owner confirmed firmware-ready, green `HW_SPEC`, and responsive
input on the real Surface. DMA is closed before parsing CMD_DONE. Scan/link DMA,
event/RX rings, and RX-PFU remain parked.

Hub/input status: new images no longer poll hub child ports through EP0 after
the first real hub-mouse report, which targets the owner's "same short time
after movement" freeze pattern. Root-port hotplug and the targeted recovery
reset still remain.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: the owner confirmed the bounded HW_SPEC image works on the real
Surface with responsive input. Quick Shadow VM
`shadow-20260709-215913-22068.json` passed 542/542; Disk 2 kernel SHA is
`6D79D5CC...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: issue one real `SCAN_EXT` command through the same bounded DMA window
while event/RX rings remain parked, then prove input stability and inspect the
first command response before enabling any result ring.
