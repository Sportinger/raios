# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~15:30, root orchestrator active)

H25 commit `d617efd` is accepted, pushed to `main`, packaged, and written to
Surface test Disk 2. After WPA2 Profile and PMK it replaces Associate exactly
once with a read-only `GET_HW_SPEC` canary on the same connection mailbox. It
accepts only a current-epoch expected `CMD_DONE`, discards the returned HW spec,
emits one bounded secret-free result, then quarantines with no network grant or
same-boot retry. One independent read-only Codex review accepted the diff.

Release kernel SHA-256 is
`9cab38c0e41f2c029dde4e5b4de65caa89ed708c4e9e9994539d0a377ee0da0d`;
package image SHA-256 is
`4e54344dcb9ad2e4e1903c1c31437cc84af9181f2d2f6438766ba7155c92c11a`.
The GPT A/B + SEED_DATA write reported 537936384 bytes and a valid superblock.
First 1 MiB SHA-256 is
`515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`.
Writer evidence: `E:\raios-build\h25-usb-write-d617efd.transcript.log` and
`E:\raios-build\h25-usb-prefix-d617efd.json`.

Pinned extraction: gate Disk 2 to `USB SanDisk 3.2Gen1`, serial
`0101d57ec458c24f1b93`, USB, size `30784094208`, and not Boot/System. Run
elevated `scripts/extract-hw-failure-trace.ps1 -DiskNumber 2
-ExpectedFriendlyName "USB SanDisk 3.2Gen1" -ExpectedImagePrefixSha256 <hash>`;
never pass erase confirmation.

## Next step

Cold-boot H25 once, select WPA2 and enter the passphrase once. Do not Retry
after quarantine. Power off, return the stick, and extract RECLOG read-only.
Expected discriminator: canary completion means mailbox lives and Associate/BSS
state is next; cleared-doorbell timeout means generic post-PMK firmware stall.

## Recently (exactly 3, newest first)

### 2026-07-21 — H25 Surface stick prepared
Pinned firmware/policy, source validation, A/B, SEED_DATA, and readback are green.

### 2026-07-21 — `d617efd` added the post-PMK canary
Bounded completion/timeout classification; no network grant, retry, or secret log.

### 2026-07-21 — H24 hardware cleared the doorbell
Firmware acknowledged Associate notification but produced no response or CMD_DONE.
