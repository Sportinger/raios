# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~14:19, root orchestrator active)

H24 is accepted, pushed, and written to SanDisk `0101d57ec458c24f1b93`.
Commit `37a2b15` reads `PCIE_CPU_INT_STATUS` exactly once on the real Associate
timeout before quarantine and persists one secret-free value: `0xD2010000`
cleared, `0xD2010001` still set, or `0xD2010002` unavailable. ACK is never
completion or DMA-consumption proof. Extractor/mutations, 61 Marvell tests, 16
DMA tests, ephemeral boundary, release build, and independent read-only ACCEPT
are green.

H24 package: image SHA-256
`b73387857aac26d9386038a088ee06262ee21e2a615c57620d0289fa3a46047e`;
kernel SHA-256
`4b69af33f083b90471973ea90cda37921862452910a4e5d5a05ae2b00ae15df9`;
firmware SHA-256
`cf4f51f41bd7ef4d7fe65fb76b8a2a0897bc70a0742bc4aea13d93b03fffd03a`.
Writer transcript `%TEMP%\raios-h24-usb-write-37a2b15.transcript.log` proves
GPT `SEED_ESP_A + SEED_ESP_B + SEED_DATA`, 537936384 image bytes, and a valid
SEED_DATA superblock. Post-write identity/layout readback is green.

Pinned read-only extraction recipe: gate Disk 2 to friendly name
`USB SanDisk 3.2Gen1`, serial above, USB, size `30784094208`, and not
Boot/System. The first 1 MiB SHA-256 is
`515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`.
Run elevated `scripts/extract-hw-failure-trace.ps1 -DiskNumber 2
-ExpectedFriendlyName "USB SanDisk 3.2Gen1" -ExpectedImagePrefixSha256 <hash>`.
Never pass an erase confirmation during extraction.

## Next step

Cold-boot the Surface from H24, start Wi-Fi exactly once, and enter the password.
Do not Retry after quarantine. On failure power down, return the stick, and
extract RECLOG read-only. Use `associate_doorbell_ack.classification` to choose
the next bounded probe; if network state is granted, switch to network debug.

## Recently (exactly 3, newest first)

### 2026-07-21 — H24 Surface stick prepared
Pinned policy/firmware, A/B, SEED_DATA, write/readback, and prefix gate are green.

### 2026-07-21 — `37a2b15` adds the H24 doorbell ACK probe
One pre-quarantine read; three bounded outcomes; no behavior change.

### 2026-07-21 — H23 response stayed untouched
Correct `0x0012` header; verified cleanup; no response or CMD_DONE.
