# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, Surface checkpoint USB ready for hardware boot)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`f84c224`. The detached old root `C:\Users\admin\Documents\raios2` remains
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

The firmware-bearing `f84c224` payload was packaged with kernel SHA-256
`5539cabd6363c9b427fd6e94dfa42c1952282a15eaca698bef50d6b16ac67f76`;
Core Policy A/generation 1 and the single embedded Marvell firmware payload
verified. It was written to the revalidated SanDisk USB disk 1 (serial
`0101d57ec458c24f1b93`). Post-write GPT A/B, BOOTCTL and SEED_DATA inspection
passed; the internal NVMe was excluded.

## Next step

Cold-boot the Surface once from this USB and record the last visible code/photo.
Wait briefly if it stops, power off, then return the stick for read-only RECLOG
extraction. Do not repeat the boot before extraction.

## Recently (exactly 3, newest first)

### 2026-07-22 - Bounded exclusive boot checkpoints accepted
`f84c224`: EB1..EB4P/E/F, five negative/bounds gates, release build and fresh
read-only ACCEPT are green.

### 2026-07-22 - Hardware boot produced an empty RECLOG
Loader visible, then black screen; verified read-only extraction returned zero
frames and zero tail on the expected SanDisk USB device.

### 2026-07-22 - Reviewed Surface capture path prepared
`a17c18b` provides bounded SMBIOS/CPUID/memory/PCI capture with verified durable
readback; the first hardware run did not reach its USB append point.
