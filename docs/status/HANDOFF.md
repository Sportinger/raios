# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~13:20, root orchestrator active)

H23 hardware-proved the timeout fingerprint. SanDisk `0101d57ec458c24f1b93`
has three valid chained frames, a clean zero tail, and USB `errors=0`. At 60.857
seconds the terminal frame is
`Associate(5)/CommandTimeout(100)`, host interrupt status zero, followed by
fingerprint `0xa6c4884c`: command `0x0012`, length 132, expected request header,
verified terminal quiesce/cleanup, and response class `untouched_zero`. The
firmware wrote no response header and raised no `CMD_DONE`; no timeout or
completion relaxation is allowed.

Read-only extraction recipe: first gate `Get-Disk -Number 2` to friendly name
`USB SanDisk 3.2Gen1`, serial above, USB, size `30784094208`, and not Boot/System.
In elevated PowerShell read only the first 1 MiB of `\\.\PhysicalDrive2`; its
SHA-256 is `515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`.
Then run `scripts/extract-hw-failure-trace.ps1 -DiskNumber 2
-ExpectedFriendlyName "USB SanDisk 3.2Gen1" -ExpectedImagePrefixSha256 <hash>`.
Never pass an erase confirmation. Latest JSON is
`%TEMP%\raios-h23-reclog-after-f77ca05-20260721T1313.json`.

Two reviews returned `PROBE_REQUIRED`. ADR 0036 selects the observation-only H24
Associate-doorbell ACK probe; the post-PMK canary remains fallback. Its exact
order is in isolated worktree `raios-h24-doorbell-ack-d3ea68d`; no product diff.

H24 is owner-blocked: Codex `gpt-5.6-sol` and `gpt-5.5` both hit the account
limit before touching files. Reset: 2026-07-27 02:41; credits may unblock
earlier. Claude substitution and root implementation are forbidden.

## Next step

Owner: restore Codex worker capacity, then rerun
`target/orchestrator-orders/h24-associate-doorbell-ack.md` in the isolated H24
worktree. Verify, commit, push, package, then perform one cold boot. No same-boot
Retry after quarantine.

## Recently (exactly 3, newest first)

### 2026-07-21 — H23 response stayed untouched
Correct `0x0012` header; verified cleanup; no response or CMD_DONE.

### 2026-07-21 — H23 Surface stick prepared
Policy, firmware, A/B, SEED_DATA, and readback green.

### 2026-07-21 — `f77ca05` adds the H23 timeout fingerprint
The bounded trace records header class only after verified terminal cleanup.
