# ADR 0010: Shareable Driver Modules (and DMA confinement via IOMMU)

## Status

Status: **PROPOSED** (2026-07-08) — pending owner decision. Recommendation:
**Option A now** (the "brain + hand" split: a signed, sandboxable driver-logic
module + a tiny trusted core actuator confined to an owner-approved per-device
hardware-capability grant, distributed through the existing ADR 0009 signed
registry) with **IOMMU/VT-d as the accepted DMA-confinement endgame, built
incrementally** (it is a large, hardware-blind subsystem — not a one-shot).

The owner has expressed intent to build IOMMU/VT-d and a module system. This ADR
records the design + the honest sequencing so the two do not get conflated: the
module system delivers shareable drivers soon and is VM-testable; IOMMU is the
last-mile that makes an *untrusted-source* DMA driver *safe*, and is its own
multi-slice milestone.

## Context

ADR 0005 puts untrusted, live-built code in Wasm sandboxes with no host access
beyond granted imports; ADR 0008 makes those imports per-service and keeps the
CORE the trust authority; ADR 0009 distributes signed modules through a local
content-addressed registry where a signature is *provenance, not
load-worthiness*. This works cleanly for **parsers** (pure bytes→bytes: X.509,
HTTP, SPKI — all now relocated into signed cross-checked Wasm guests).

**Drivers are the opposite of parsers.** A driver needs raw hardware: memory-
mapped I/O registers (MMIO), device DMA into host RAM, precise timing, and
sometimes interrupts. Handing a Wasm sandbox "arbitrary MMIO + DMA" hands it the
whole machine — it defeats the isolation thesis. So drivers are the case ADR
0005/0008 did not yet answer: how does raiOS accept a driver that many people
build "chip by chip" and share, without trusting each author with the kernel?

raiOS already has the seed of the answer in the Marvell 88W8897 WiFi work:
`raios-core/src/marvell_wifi_fw.rs` is a **pure, host-tested driver "brain"** (a
register-write PLAN / state machine — no hardware), and
`seed-kernel/src/marvell_wifi_pcie.rs` is the **thin trusted "hand"** that does
the real MMIO/DMA. This ADR generalises that split into the driver-module model.

## Options considered

**Option A — Brain (shareable, sandboxed) + Hand (tiny, trusted), capability-
scoped.** The shareable unit is a signed Wasm "brain" module (pure protocol/
state-machine logic that emits register-op intents / consumes register reads) plus
a **hardware-capability manifest** ("may touch PCI 11ab:2b38, BAR2, these DMA
buffers, these register ranges — nothing else"). A small trusted core "hand"
executes the brain's ops against real MMIO/DMA, and the CORE is the reference
monitor: it validates every MMIO address and every DMA target against the grant.
Fits the existing guest=evidence / core=authority model exactly. A malicious or
buggy brain can only reach the hardware it was granted. MMIO is fully core-
mediated. **DMA is the gap** (see below).

**Option B — Native signed driver modules (Linux-kernel-module style).** Ship the
driver as native code, signed, with a manifest. Fast + flexible, but native code
has full machine access — the manifest is advisory unless enforced by hardware
(per-module MMU + IOMMU). Without that, a native driver module is *full trust* —
it abandons the raiOS isolation thesis and makes every third-party driver a
kernel-level blob.

**Option C — Fully-Wasm driver with narrow core-mediated hardware imports.** The
driver runs entirely in Wasm and calls `mmio_read/mmio_write/dma_map/wait_bit`
imports that the core validates per-call against the grant. More flexible than A
(dynamic control flow, timing loops in the guest) while keeping confinement, but
per-op boundary crossings cost latency, µs-precise timing is harder across the
boundary, and the core must become a full per-call hardware reference monitor.

**The DMA problem (applies to all options).** MMIO the core can mediate (check
each register address). DMA it cannot: the *device itself* reads/writes host RAM
at physical addresses the driver programs into it. A bad driver programs a kernel
address → the device corrupts the kernel, bypassing Wasm and the MMU entirely.
The only real confinement is an **IOMMU (Intel VT-d)** that restricts, per device,
which physical addresses it may touch. Therefore: a *truly safe, untrusted-source*
DMA driver requires IOMMU support. This is the honest ceiling, and the endgame —
not the first step.

## Decision (recommended)

1. **Adopt Option A as the raiOS driver-module standard.** A shareable driver =
   a signed Wasm "brain" module + a hardware-capability manifest, distributed via
   the ADR 0009 signed registry (signature = provenance; load-worthiness is re-
   verified through the M6 gate + M7 persistence re-verify each load). The owner
   approves the hardware-capability grant (which device / BAR / DMA region /
   register ranges) — this is a real, owner-gated capability, honestly
   `dev_key_not_owner_sealed` until the sealing ceremony. The trusted "hand" is
   kernel code; the core validates every MMIO access against the grant.

2. **MMIO is core-mediated from day one.** DMA is, until IOMMU lands, **owner-
   trusted per driver** and labelled exactly that — a driver granted DMA is
   trusted not to point the device at kernel memory. No dishonest overclaim: the
   capability report states DMA is not hardware-confined yet.

3. **IOMMU/VT-d is the accepted DMA-confinement endgame, built incrementally** as
   its own milestone, not a prerequisite for the module system:
   - Slice 1: parse the DMAR ACPI table, detect the remapping unit(s), map their
     registers, read + report capability (grants nothing; VM-testable with QEMU
     `-device intel-iommu`).
   - Slice 2: per-device domains + second-level I/O page tables for a single
     device's granted DMA buffers (no enforcement yet).
   - Slice 3: enable DMA remapping + fault handling; flip a driver's DMA grant
     from "owner-trusted" to "IOMMU-confined".
   Once Slice 3 lands for a device class, even a wild-shared driver cannot corrupt
   the kernel via DMA.

4. **Distribution + lifecycle** ride the existing machinery: an external AI ports
   a Linux driver (e.g. mwifiex) → emits the raiOS brain module → signs it
   (publisher key, domain-tagged) → publishes to the registry → the owner reviews
   the hardware-capability grant → raiOS re-verifies + loads it confined. No new
   distribution channel; provenance ≠ load-worthiness stays the hard invariant.

## Consequences

- Drivers become first-class shareable modules under the same trust model as
  parsers, without trusting each author with the kernel — for the MMIO-mediated
  case immediately, and for DMA once IOMMU Slice 3 lands.
- The module system is buildable + VM-testable now (raios-core capability model +
  evaluators + the ADR 0009 registry + owner-gated grants). It is not blocked on
  IOMMU.
- IOMMU/VT-d is honestly a large, hardware-blind subsystem (DMAR ACPI + I/O page
  tables + remapping hardware + fault handling), debuggable mainly on real silicon
  (the Surface) or a finicky emulated IOMMU. It must be scoped as a multi-slice
  milestone with its own state-ladder telemetry, not implied as a quick win.
- Every capability grant and confinement claim stays honestly labelled
  (`dev_key_not_owner_sealed`, DMA "owner-trusted / not IOMMU-confined" until
  proven). Grants-nothing until an explicit owner-gated authority-flip slice.
