# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, Surface capture-stage USB ready)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`f3850bb`. The detached old root `C:\Users\admin\Documents\raios2` remains
foreign WIP; never clean, reset, merge, or integrate it. Worktree is clean.

The firmware-bearing `f330e82` hardware run advanced through EB1 and EB1P,
then stayed visibly at gray EB1C for more than two minutes. This localizes the
stall to `surface_fact_capture::capture()`: provider configuration and console
initialization returned, while EB2 before `usb::init` was never reached.

After power-off, read-only extraction revalidated the expected SanDisk USB
disk 1, serial `0101d57ec458c24f1b93`, its GPT and SEED_DATA, but again found
RECLOG `valid_frame_count=0`, `tail_status=zero_tail`, no USB diagnostics and
no Surface Fact candidate. The stall is still before the first durable append.

`f3850bb` adds five static, value-free progress checkpoints inside capture:
SC immediately before CPUID, SS before SMBIOS, SM before MemoryMap, SP before
PCI enumeration, and SV before finalization; EB2 remains after capture returns.
The exact-order predicate and missing/reorder mutations pass, the freestanding
release build passes, and a fresh independent read-only review reports ACCEPT.
Capture work and ordering are otherwise unchanged.

The firmware-bearing payload uses kernel SHA-256
`dfc4d63acb637ec3f735c1b6084fc834d1f12411587ee97f924f50d9baf2b1ef`;
Core Policy A/generation 1 and the Marvell firmware verified. The image was
written to the revalidated SanDisk USB disk 1. Physical post-write inspection
confirmed GPT with exact ESP A, ESP B and SEED_DATA partition sizes.

## Next step

Cold-boot the prepared USB once. Record the last visible code among
SC/SS/SM/SP/SV/EB2, power off, and return the stick for read-only extraction;
do not repeat the boot before extraction.

## Recently (exactly 3, newest first)

### 2026-07-22 - Capture stages accepted
`f3850bb`: SC/SS/SM/SP/SV instrumentation, mutation predicate, release build,
and independent read-only ACCEPT are green.

### 2026-07-22 - Hardware run stayed at EB1C
Provider and console returned; capture did not return. Verified extraction
again showed zero RECLOG frames and zero tail.

### 2026-07-22 - EB1 initialization split accepted
`f330e82`: EB1/EB1P/EB1C/EB2 order, six mutations, release build and fresh
read-only ACCEPT are green.
