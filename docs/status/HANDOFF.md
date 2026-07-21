# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~14:10, root orchestrator active)

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

ADR 0036 selected the observation-only H24 Associate-doorbell ACK probe; the
post-PMK canary remains fallback. Commit `37a2b15` reads
`PCIE_CPU_INT_STATUS` exactly once on the real Associate timeout before
quarantine and appends one secret-free value: `0xD2010000` cleared,
`0xD2010001` still set, or `0xD2010002` unavailable. ACK is never completion or
DMA-consumption proof. Extractor/mutations, 61 Marvell tests, 16 DMA tests,
ephemeral boundary, release build, and final read-only ACCEPT are green; pushed
on `main`.

## Next step

Package and write H24 from `37a2b15`, verify both slots and readback, then perform
one cold Surface boot and start Wi-Fi once. No same-boot Retry after quarantine;
on failure power down and extract RECLOG before rewrite.

## Recently (exactly 3, newest first)

### 2026-07-21 — `37a2b15` adds the H24 doorbell ACK probe
One pre-quarantine read; three bounded outcomes; no behavior change.

### 2026-07-21 — H23 response stayed untouched
Correct `0x0012` header; verified cleanup; no response or CMD_DONE.

### 2026-07-21 — H23 Surface stick prepared
Policy, firmware, A/B, SEED_DATA, and readback green.
