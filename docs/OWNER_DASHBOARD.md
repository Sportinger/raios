# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-10.

Current capability: on the real Surface, a user can click `WiFi DETECTED`, scan
live networks, select a secured SSID, and enter its password while keyboard and
mouse remain responsive without unplug/replug.

What is still denied: broad USB disk mutation, writes outside `SEED_DATA/RECLOG`,
durable secret storage, owner-sealed persistence, and WLAN link authority.

Stick evidence: host inspection was valid 84/84 with no torn tail; the successful
Surface screen now reports verified `MSC LOG seq85`.

WiFi status: Surface-proven `FW_STATUS`, `HW_SPEC`, and real legacy-scan SSIDs
are green. The new guided UI uses that exact path; RX-PFU, association, DHCP,
and link authority remain parked.

Hub/input status: Surface-proven stable through the full guided flow at 65.7s
uptime with `KBD READY`, `MOUSE READY`, `ERR 0`, and no replug.

Owner-key status: RAM boot still creates only a secret RAM-only
`current_boot` owner-key candidate. Persistent owner seal/install/load/durable
authority remains denied until the real sealing ceremony.

Latest proof: positive Surface screenshot/owner confirmation plus quick Shadow VM
`shadow-20260709-235209-16668.json` 542/542. Disk 2 kernel SHA is `1AAB49E2...`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used focused USB VM
evidence plus quick profile per aggressive-fast cadence.

Next task: implement the first real bounded Marvell association step; keep
`connected`, link, DHCP, and provider access denied until full evidence exists.
