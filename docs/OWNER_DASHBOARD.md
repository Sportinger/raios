# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: raiOS can now request legacy 2.4 GHz scan results directly
through the bounded command-response buffer and feed real SSID/channel/security
facts into the existing `[LIVE]` network list without event/RX rings.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and WLAN link authority.

Stick evidence: RECLOG is valid 66/66 with no torn tail. New frames show
`reports=179` remained frozen through endpoint rearm, hub-port reset, and more
rearms; no xHCI transfer error was recorded.

WiFi status: Surface-proven `FW_STATUS`, `HW_SPEC`, and `SCAN_EXT` are green.
The new image uses legacy scan response data to avoid the event ring entirely;
RX-PFU and link authority remain parked pending the next hardware test.

Hub/input status: mouse input still stalled after WiFi start. Soft rearm and a
successful port reset did not restore reports; physical unplug/replug did.
Full child re-enumeration recovery remains open.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-225029-33012.json` passed 542/542
and 32/32 focused Marvell tests passed. Disk 2 now has kernel SHA
`E46FC0FB...`; positive legacy-result evidence still requires the Surface.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: start WiFi, recover the mouse by replug if needed, click Scan once,
and confirm `SCAN: done result=live_results_ready len=106` plus real `[LIVE]`
SSIDs. Then read RECLOG again for the input timeline.
