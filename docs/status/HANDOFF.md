# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, EB1-split USB ready for hardware boot)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`f330e82`. The detached old root `C:\Users\admin\Documents\raios2` remains
foreign WIP; never clean, reset, merge, or integrate it. Worktree is clean.

The first Surface capture boot showed the raiOS/Limine loader and then a black
display. Read-only extraction proved the USB GPT and SEED_DATA valid but found
RECLOG `valid_frame_count=0`, `tail_status=zero_tail`, no USB diagnostics and
no Surface Fact candidate. The kernel did not reach its first durable append.

K4 is accepted and pushed. It draws bounded static framebuffer checkpoints:
EB1 before Surface measurement, EB2 immediately before `usb::init`, EB3 after
USB, EB4P only after successful capture append, EB4E only after append error,
and EB4F only after measurement error. Small framebuffers retain distinct
color fallbacks. Predicate bounds plus missing/duplicate/reorder/err-collapsed
source mutations pass, the freestanding release build passes, and a fresh
independent read-only review reports no findings and `VERDICT: ACCEPT`.

The `f84c224` hardware run stayed visibly at EB1. A second read-only extraction
again proved valid GPT/SEED_DATA with RECLOG `valid_frame_count=0`, zero tail,
no USB diagnostics and no Surface facts. `f330e82` now splits only that window:
EB1 before provider config, EB1P after provider return, EB1C after console
return, then EB2 after Surface capture. Predicate bounds and six mutations,
freestanding release build, and fresh read-only review are green with no
findings. No provider/console/capture/USB call count or semantics changed.

The firmware-bearing `f330e82` payload was packaged with kernel SHA-256
`8c6181673e18778ee1e1890cd3ee139acaecc646317c4e10028a8f9490d29900`;
Core Policy A/generation 1 and the embedded Marvell payload verified. It was
written to the revalidated SanDisk USB disk 1, serial
`0101d57ec458c24f1b93`; post-write GPT A/B, BOOTCTL and SEED_DATA passed.

## Next step

Cold-boot once and record whether the last code is EB1, EB1P, EB1C or EB2.
Then power off and return the stick for read-only extraction; do not repeat
the boot before extraction.

## Recently (exactly 3, newest first)

### 2026-07-22 - EB1 initialization window split accepted
`f330e82`: EB1/EB1P/EB1C/EB2 order, six mutations, release build and fresh
read-only ACCEPT are green.

### 2026-07-22 - Hardware run stayed at EB1 with empty RECLOG
Loader then EB1 visible; second verified extraction again returned zero frames
and zero tail on the expected SanDisk USB device.

### 2026-07-22 - Reviewed Surface capture path prepared
`a17c18b` provides bounded SMBIOS/CPUID/memory/PCI capture with verified durable
readback; the first hardware run did not reach its USB append point.
