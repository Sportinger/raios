# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, Genesis-unblock USB ready)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`b18f272`. The detached old root `C:\Users\admin\Documents\raios2` remains
foreign WIP; never clean, reset, merge, or integrate it. Worktree is clean.

The `ec1aea1` hardware run stayed visibly at yellow SI. This proves CPUID and
the SS checkpoint completed, then the first physical SMBIOS entry-point slice
faulted before ST/SR/MemoryMap/PCI/USB. Repeated earlier returned sticks had
zero-tail RECLOG because USB was never reached; another empty extraction was
not useful for this deterministic pre-USB fault.

`b18f272` adds fieldless `PhysicalSmbiosAccessPolicy::{Reject, Allow}` and makes
the normal boot select Reject. After SS, capture returns the static error before
SI, Limine option access, `capture_smbios` or any physical slice. The entire
capture is discarded; main then retains its existing EB2 -> `usb::init` -> EB3
-> EB4F route and continues toward Genesis/WLAN. No partial facts can append;
the bounded Allow implementation remains unchanged for future explicit use.

Reject-to-Allow and late-Reject source mutations, early-boot and capture
predicates, freestanding release build, `git diff --check`, unsafe inventory,
and fresh independent read-only review are green; review verdict is ACCEPT.

The firmware-bearing payload uses kernel SHA-256
`803e4c0df649cfc3435358b9db440a4ee2996308c26ede05cc9a30f6d6bf0f6c`;
Core Policy A/generation 1 and Marvell firmware verified. It was written to the
revalidated SanDisk disk 1; physical GPT ESP A/B and SEED_DATA sizes verified.

## Next step

Cold-boot this USB once. Expected transient codes are SS, EB2, EB3 and EB4F,
then normal Genesis/WLAN UI. Report the final screen or last persistent code.
If Genesis appears, continue directly with Wi-Fi connect and agent test.

## Recently (exactly 3, newest first)

### 2026-07-22 - Unsafe SMBIOS access rejected during boot
`b18f272`: fail-closed boot policy, mutation predicates, release build and
independent ACCEPT are green; physical USB layout verified.

### 2026-07-22 - Hardware run stayed at SI
The first physical SMBIOS entry-point slice is the deterministic pre-USB fault.

### 2026-07-22 - SMBIOS access stages accepted
`ec1aea1`: SI/ST/SR instrumentation, release build and independent ACCEPT were
green and provided the decisive hardware boundary.
