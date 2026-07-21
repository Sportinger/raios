# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~16:15, root orchestrator active)

H25 hardware-proved commit `d617efd`. Read-only extraction at
`E:\raios-build\h25-reclog-after-d617efd.json` found six valid chained frames,
a clean zero tail, and USB `errors=0`. At 120.707 seconds the one post-PMK
`GET_HW_SPEC` canary completed with the expected current-epoch `CMD_DONE` and
reported `post_pmk_hw_spec_canary.outcome=expected_completion`. Network state
remained denied and cold reboot remained required as designed.

This rules out a generic post-PMK HostCmd mailbox/firmware stall: Profile, PMK,
and a subsequent command all complete on the same connection mailbox. The
remaining failure is Associate/BSS-specific. H24 already proved Associate's
correct 132-byte request, cleared doorbell, untouched response, and absent
`CMD_DONE`; H26 must now discriminate missing/incorrect pre-Associate BSS state
or command semantics without weakening fail-closed completion.

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

Obtain one independent read-only Codex review of the Marvell pre-Associate/BSS
sequence and select one bounded H26 discriminator. Then implement, verify,
package, and write the next Surface stick. Preserve no response read before a
current completion, no same-boot retry, no network grant, and secret-free RECLOG.

## Recently (exactly 3, newest first)

### 2026-07-21 — H25 proved post-PMK mailbox liveness
The canary completed as expected; the remaining fault is Associate/BSS-specific.

### 2026-07-21 — `d617efd` added the post-PMK canary
Bounded completion/timeout classification; no network grant, retry, or secret log.

### 2026-07-21 — H25 Surface stick prepared
Pinned firmware/policy, source validation, A/B, SEED_DATA, and readback were green.
