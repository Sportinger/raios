# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, SMBIOS-substage USB ready)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`ec1aea1`. The detached old root `C:\Users\admin\Documents\raios2` remains
foreign WIP; never clean, reset, merge, or integrate it. Worktree is clean.

The `f3850bb` Surface boot reached SC and then SS, proving CPUID completed and
the stall is inside `capture_smbios()` before MemoryMap/PCI/USB. After power-off,
read-only extraction revalidated the expected SanDisk USB disk 1, serial
`0101d57ec458c24f1b93`, GPT and SEED_DATA, but again found RECLOG
`valid_frame_count=0`, `tail_status=zero_tail`, no diagnostics or facts.

`ec1aea1` adds value-free SMBIOS substages: SI immediately before the first
entry-point slice in either exclusive 32/64-bit branch, ST immediately before
the table slice, and SR immediately before the parser. Existing SMBIOS work,
five slice calls and validation remain unchanged. Missing/reorder mutations,
the freestanding release build, `git diff --check`, and fresh independent
read-only review are green; review verdict is ACCEPT.

The firmware-bearing payload uses kernel SHA-256
`dd970b8aafbd614cc988895bca933b59a1e130ed2967d24423261856d694d194`;
Core Policy A/generation 1 and Marvell firmware verified. It was written to the
revalidated SanDisk disk 1. Physical post-write inspection confirms exact GPT
ESP A, ESP B and SEED_DATA sizes.

## Next step

Cold-boot the prepared USB exactly once. Record the last visible code, expected
SI, ST, SR or a later existing checkpoint. Power off and return the stick for
read-only extraction; do not repeat the boot before extraction.

## Recently (exactly 3, newest first)

### 2026-07-22 - SMBIOS access stages accepted and written
`ec1aea1`: SI/ST/SR, mutation predicate, release build and independent ACCEPT
are green; firmware-bearing physical USB layout verified.

### 2026-07-22 - Hardware run stayed at SS
CPUID completed; capture stalled inside SMBIOS. Returned RECLOG remained empty
with zero tail.

### 2026-07-22 - Capture stages accepted
`f3850bb`: SC/SS/SM/SP/SV instrumentation, mutation predicate, release build,
and independent read-only ACCEPT are green.
