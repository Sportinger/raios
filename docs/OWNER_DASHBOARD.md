# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: clicking `WiFi DETECTED` now automatically starts real
firmware/HW_SPEC/scan progress, opens the live SSID list, and accepts a selected
network plus masked RAM-only password without console commands.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and WLAN link authority.

Stick evidence: RECLOG is valid 66/66 with no torn tail. New frames show
`reports=179` remained frozen through endpoint rearm, hub-port reset, and more
rearms; no xHCI transfer error was recorded.

WiFi status: Surface-proven `FW_STATUS`, `HW_SPEC`, and real legacy-scan SSIDs
are green. The new guided UI uses that exact path; RX-PFU, association, DHCP,
and link authority remain parked.

Hub/input status: mouse input still stalled after WiFi start. Soft rearm and a
successful port reset did not restore reports; physical unplug/replug did.
Full child re-enumeration recovery remains open.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-232827-11108.json` passed 542/542.
The previous direct-scan slice also passed 32/32 focused Marvell tests.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: refresh Disk 2 and click `WiFi DETECTED`; verify progress, live list,
SSID selection, masked password, and the honest `Connection not established`
result. Then read RECLOG again if hub input stalls.
