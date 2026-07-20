# 05 — Drivers & Hardware (agent-built, in domains)

> Breakdown of `docs/SCOPE.md` §5. Every driver is a userspace domain with
> exactly the grants it needs — no driver code in the kernel, ever. Exploratory
> lane mode: crashes are cheap, the watchdog restarts you.

## Common bar for every driver domain (applies to each item below)
- [ ] Runs with a minimal explicit grant set (its BAR, its IRQ, its DMA region)
- [ ] Kill + restart of the driver leaves the system healthy (device re-init works)
- [ ] Negative test: driver domain cannot reach any foreign device or region
- [ ] Emits RECLOG diagnostics; failure states are machine-readable

Evidence 2026-07-20: the H20 Surface freeze left only one valid
`raios.usb_diag.v0` `boot_probe` record (`errors=0`, `recoveries=0`) and no
`raios.hw_failure_trace.v0`. The hard failure was therefore not captured as a
machine-readable WiFi diagnostic, so the common RECLOG box remains open.

## Wi-Fi (Marvell 88W8897)
- [ ] Firmware load + handshake from inside the domain
- [ ] Own DMA region, IOMMU-fenced; scan + connect + traffic
- [ ] Survives repeated kill/restart mid-traffic

Evidence 2026-07-20, Surface Pro 4: cold-boot kernel runs proved 88W8897
firmware upload/readiness and a live 2.4-GHz scan, then reached WPA2 selection
and physical passphrase entry. H20 used
`PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL` before scan, but
`Starting WiFi` coincided with loss of keyboard and mouse and produced no WiFi
failure trace. This is evidence for firmware upload and scan only. The path is
still in-kernel; domain execution, IOMMU fencing, connection, traffic, and safe
kill/restart remain open. The freeze is evidence against closing any WiFi box.

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
