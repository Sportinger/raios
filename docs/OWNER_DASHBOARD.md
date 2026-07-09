# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: USB keyboard and mouse may now remain normally idle without
raiOS forcibly rearming endpoints, resetting a hub port, or synchronously
writing recovery records through the same hub.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and WLAN link authority.

Stick evidence: RECLOG is valid 84/84 with no torn tail. Forced recovery frames
showed report progress but `errors=0`, `ICC=0`, successful transfer `CC=1`;
the watchdog actions were false positives, not device errors.

WiFi status: Surface-proven `FW_STATUS`, `HW_SPEC`, and real legacy-scan SSIDs
are green. The new guided UI uses that exact path; RX-PFU, association, DHCP,
and link authority remain parked.

Hub/input status: the one-second mouse-idle watchdog and its repeated synchronous
RECLOG writes are deleted. Real transfer errors still recover; Surface keyboard
and mouse stability now need one fresh guided-flow test.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: quick Shadow VM `shadow-20260709-235209-16668.json` passed 542/542.
The previous direct-scan slice also passed 32/32 focused Marvell tests. Disk 2
now has final kernel SHA `357906AB...` without reformatting `SEED_DATA`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: refresh/boot Disk 2, run `WiFi DETECTED`, type the password, and
leave the mouse idle for over ten seconds; neither input device should drop.
