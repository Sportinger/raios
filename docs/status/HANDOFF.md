# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, K3 repair architecture accepted; K3a active)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`ed6ec7d`. The detached old root `C:\Users\admin\Documents\raios2` contains
foreign WIP; never clean, reset, merge, or integrate it.

The five inherited dirty files remain the rejected K3 Surface-capture slice:
unsafe baseline, `main.rs`, `usb.rs`, the K3 predicate, and
`surface_fact_capture.rs`. Two fresh read-only reviews rejected an Owner boot.
Local Limine 0.5.0 proves SMBIOS entry pointers are physical, but K3 lacks a
Memory-map-authorized interval reader, complete SMBIOS/CPUID rules, fail-closed
PCI snapshot semantics, and a production RECLOG failure seam. The real unsafe
baseline check is red (398 recorded vs 404 current sites).

ADR 0043 is accepted and pushed. It retains Wire V1, fails the whole capture on
uncertain facts, and splits repair into three secured slices. K3a is the only
active worker and owns exactly clean `pci.rs` plus `test-pci-bar-sizing.ps1`;
the five inherited dirty files are taboo. No physical stick write occurred.

## Next step

Verify K3a's capture-specific PCI Result enumeration with the full existing
runtime/emitted-code predicate and rejection mutation, then obtain an
independent read-only review and secure only its exact two files. Continue with
K3b bounded SMBIOS/CPUID/ordering and K3c production RECLOG state machine.
After final predicates, release build and two fresh K3 accepts, package the
capture image. Never write the physical stick without an explicit final disk
number confirmation from the Owner.

## Recently (exactly 3, newest first)

### 2026-07-22 - K3 repair boundary accepted
`ed6ec7d`: ADR 0043 records two neutral opinions, resolves SMBIOS pointer and
parser disagreements, and selects three bounded fail-closed repair slices.

### 2026-07-22 - K3 review rejected Owner boot
Both reviews found unbounded SMBIOS reads, incomplete facts, ambiguous PCI
probe failures, model-only RECLOG tests, and a stale unsafe baseline.

### 2026-07-22 - Header-bounded PCI proof accepted
`e926f06`: 21/21 runtime tests, two transport mutation negatives, and fresh
independent reviews accepted the exact two-file PCI BAR sizing slice.
