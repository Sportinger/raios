# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~13:20, root orchestrator active)

H23 hardware-proved the secret-free timeout fingerprint. The returned SanDisk
`0101d57ec458c24f1b93` contains three valid chained frames and a clean zero tail.
USB has `errors=0`; the terminal frame at 60.857 seconds is
`Associate(5)/CommandTimeout(100)`, host interrupt status zero, followed by
fingerprint `0xa6c4884c`: command `0x0012`, length 132, expected request header,
verified terminal quiesce/cleanup, and response class `untouched_zero`. The
firmware neither wrote a response header nor raised `CMD_DONE`; extending the
timeout or accepting a header as completion remains forbidden.

Read-only extraction recipe: first gate `Get-Disk -Number 2` to friendly name
`USB SanDisk 3.2Gen1`, serial above, USB, size `30784094208`, and not Boot/System.
In elevated PowerShell read only the first 1 MiB of `\\.\PhysicalDrive2`; its
SHA-256 is `515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`.
Then run `scripts/extract-hw-failure-trace.ps1 -DiskNumber 2
-ExpectedFriendlyName "USB SanDisk 3.2Gen1" -ExpectedImagePrefixSha256 <hash>`.
Never pass an erase confirmation. Latest JSON is
`%TEMP%\raios-h23-reclog-after-f77ca05-20260721T1313.json`.

## Next step

Obtain two fresh neutral read-only Codex opinions on the untouched-response
boundary, choose the narrow H24 repair or probe, then verify, commit, push,
package, and perform one cold boot. No same-boot Retry after quarantine.

## Recently (exactly 3, newest first)

### 2026-07-21 — H23 response buffer stayed untouched
Correct 132-byte `0x0012` request; verified cleanup; no response and no CMD_DONE.

### 2026-07-21 — H23 persistent Surface stick prepared
Kernel-bound policy, pinned firmware, A/B layout, SEED_DATA, and readback green.

### 2026-07-21 — `f77ca05` adds the H23 timeout fingerprint
The bounded trace records header class only after verified terminal cleanup.
