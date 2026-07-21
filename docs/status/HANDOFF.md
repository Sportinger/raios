# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, paused cleanly at owner request)

H25 hardware-proved commit `d617efd`. The saved read-only extraction
`E:\raios-build\h25-reclog-after-d617efd.json` has six valid chained frames, a
clean zero tail, USB `errors=0`, and an expected current-epoch completion for
the post-PMK `GET_HW_SPEC` canary at 120.707 seconds. Network remained denied
and cold reboot remained required. Generic post-PMK mailbox/firmware liveness
is proven; the fault is Associate/BSS-specific.

Linux `mwifiex` comparison selected H26: retain the scan firmware TSF and AP
beacon timestamp, append `TLV_TYPE_TSFTIMESTAMP` (`0x0113`, firmware then AP)
to Associate, and restore PMK -> Associate. This is a functional correction,
not another diagnostic-only boot.

Stick identity is Disk 2, `USB SanDisk 3.2Gen1`, serial
`0101d57ec458c24f1b93`, USB, 30784094208 bytes, never Boot/System. Gate the
elevated read-only extractor on that identity and first-1-MiB SHA-256
`515a68c5ce3337112d0513b7d95524b836813f112c6751d912423454f0d702cb`; never
pass erase confirmation. Writer evidence is under `E:\raios-build\h25-usb-*`.

No H26 worker/build is active. Its clean empty worktree and branch were removed;
the inserted stick was not rewritten. A final non-elevated extraction was denied
by Windows raw-disk access, so the saved H25 JSON remains authoritative.

## Next step

Create one H26 scan-TSF -> Associate lane, run focused builder/parser negatives
and one read-only Codex review, then package and write exactly one Surface stick.
Keep current-completion gating, no same-boot retry/grant, and secret-free RECLOG.

## Recently (exactly 3, newest first)

### 2026-07-21 — H26 restart point selected and workspace closed
Linux `mwifiex` comparison selected the missing scan-TSF Associate TLV; no H26
product work was started, and the empty temporary branch/worktree was removed.

### 2026-07-21 — H25 proved post-PMK mailbox liveness
The canary completed as expected; the remaining fault is Associate/BSS-specific.

### 2026-07-21 — `d617efd` added the post-PMK canary
Bounded completion/timeout classification; no network grant, retry, or secret log.
