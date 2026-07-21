# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~14:40, root orchestrator active)

H24 hardware-proved commit `37a2b15`. Read-only extraction at
`%TEMP%\raios-h24-reclog-after-37a2b15.json` found three valid chained frames,
a clean zero tail, and USB `errors=0`. At 118.495 seconds Associate `0x0012`
still had the expected 132-byte request header, host interrupt status zero,
verified quiesce/cleanup, an untouched response, and no `CMD_DONE`. The new
pre-quarantine observation decoded `associate_doorbell_ack=cleared`: firmware
acknowledged notification, but payload consumption and command execution remain
unproven.

Pinned extraction recipe: gate Disk 2 to `USB SanDisk 3.2Gen1`, serial
`0101d57ec458c24f1b93`, USB, size `30784094208`, and not Boot/System. First
1 MiB SHA-256 is
`515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`.
Run elevated `scripts/extract-hw-failure-trace.ps1 -DiskNumber 2
-ExpectedFriendlyName "USB SanDisk 3.2Gen1" -ExpectedImagePrefixSha256 <hash>`;
never pass erase confirmation.

ADR 0037 selects H25: after successful Profile and PMK, replace Associate once
with one read-only `GET_HW_SPEC` canary using the same fresh-low `CMD_DONE`,
timeout, and quarantine contract. A valid expected completion proves post-PMK
mailbox liveness and moves next to Associate/BSS state; timeout with cleared
doorbell favors a generic post-PMK mailbox/firmware stall. One independent
read-only Codex review recommended this. Owner commit `a516824` changed the
uncertainty-review rule from two opinions to one.

## Next step

Implement and verify the bounded H25 canary lane, then package and write the
next Surface stick. Preserve fail-closed completion, no response read before
current `CMD_DONE`, no retry after quarantine, and secret-free RECLOG.

## Recently (exactly 3, newest first)

### 2026-07-21 — H24 doorbell was cleared
Notification reached firmware; Associate still produced no response/completion.

### 2026-07-21 — H24 Surface stick prepared
Pinned policy/firmware, A/B, SEED_DATA, write/readback, and prefix gate were green.

### 2026-07-21 — `37a2b15` added the H24 probe
One pre-quarantine read; three bounded outcomes; no behavior change.
