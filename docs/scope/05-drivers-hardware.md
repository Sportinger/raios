# 05 — Drivers & Hardware (agent-built, in domains)

> Breakdown of `docs/SCOPE.md` §5. Every driver is a userspace domain with
> exactly the grants it needs — no driver code in the kernel, ever. Exploratory
> lane mode: crashes are cheap, the watchdog restarts you.

## Common bar for every driver domain (applies to each item below)
- [ ] Runs with a minimal explicit grant set (its BAR, its IRQ, its DMA region)
- [ ] Kill + restart of the driver leaves the system healthy (device re-init works)
- [ ] Negative test: driver domain cannot reach any foreign device or region
- [ ] Emits RECLOG diagnostics; failure states are machine-readable

Evidence 2026-07-21: H21 retained HID and left three valid chained RECLOG
frames: USB `errors=0`, a pre-BME PCI Command checkpoint at `0x0402`, and
terminal `0xD1030001` at 42.757 seconds. That secret-free code identifies an
all-ones read of the firmware-owned RX-WR/TX-RD register during ring
construction. The diagnostic path therefore ran on hardware, but one failure
record does not prove the common driver-domain logging contract; the box stays
open.

## Wi-Fi (Marvell 88W8897)
- [ ] Firmware load + handshake from inside the domain
- [ ] Own DMA region, IOMMU-fenced; scan + connect + traffic
- [ ] Survives repeated kill/restart mid-traffic

Evidence 2026-07-20/21, Surface Pro 4: cold-boot kernel runs proved 88W8897
firmware upload/readiness and a live 2.4-GHz scan, then reached WPA2 selection
and physical passphrase entry. H20 used
`PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL` before scan, but
`Starting WiFi` coincided with loss of keyboard and mouse and produced no WiFi
failure trace. H21 retained HID, proved the persisted pre-BME checkpoint, and
localized the next failure to a premature `0xC08C` read before descriptor
registration. Commit `c787320` now seeds only that construction-time shadow to
zero, matching upstream 88W8897 pointer ownership; real RX/TX reads still
reject all ones before data use. Its focused predicates, mutation negatives,
16 pointer tests, full typecheck, release build, and final read-only review are
green. It is not hardware-proven yet. The path is still in-kernel; domain
execution, IOMMU fencing, connection, traffic, and safe kill/restart remain
open.

## USB stack
- [ ] Host controller domain; hotplug events surface as typed events
- [ ] Input devices (HID) usable by other domains via capability, not raw bus access

## Network stack
- [ ] Ethernet/Wi-Fi backed; DNS/TCP/TLS path as domain service, out of the kernel
- [ ] A misbehaving network stack is killable without kernel harm (negative test)

## Storage driver
- [ ] Block device domain honoring range-scoped storage capabilities
- [ ] Power-loss torture: no corruption of ranges it was never granted

## GPU
- [ ] Framebuffer region as a capability (already the display path)
- [ ] Long-term: command submission / 3D as its own domain — kept as vision,
      not blocking any other checkbox
