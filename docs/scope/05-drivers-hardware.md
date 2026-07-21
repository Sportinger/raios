# 05 — Drivers & Hardware (agent-built, in domains)

> Breakdown of `docs/SCOPE.md` §5. Every driver is a userspace domain with
> exactly the grants it needs — no driver code in the kernel, ever. Exploratory
> lane mode: crashes are cheap, the watchdog restarts you.

## Common bar for every driver domain (applies to each item below)
- [ ] Runs with a minimal explicit grant set (its BAR, its IRQ, its DMA region)
- [ ] Kill + restart of the driver leaves the system healthy (device re-init works)
- [ ] Negative test: driver domain cannot reach any foreign device or region
- [ ] Emits RECLOG diagnostics; failure states are machine-readable

Evidence 2026-07-21: H23 retained HID and left three valid chained RECLOG frames
with a clean zero tail and USB `errors=0`. The terminal frame contains the host
status plus a bounded association-timeout fingerprint after verified cleanup.
The diagnostic path therefore ran on hardware, but one in-kernel failure record
does not prove the common driver-domain logging contract; the box stays open.

## Wi-Fi (Marvell 88W8897)
- [ ] Firmware load + handshake from inside the domain
- [ ] Own DMA region, IOMMU-fenced; scan + connect + traffic
- [ ] Survives repeated kill/restart mid-traffic

Evidence 2026-07-20/21, Surface Pro 4: cold-boot kernel runs proved 88W8897
firmware upload/readiness and a live 2.4-GHz scan, then reached WPA2 selection
and physical passphrase entry. H20 used
`PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL` before scan, but
`Starting WiFi` coincided with loss of keyboard and mouse and produced no WiFi
failure trace. H21 localized the next failure to a premature `0xC08C` read
before descriptor registration. H22 hardware-proved commit `c787320`: that
failure disappeared, HID remained usable, PCI Command advanced `0x0402>0x0406`,
firmware stayed `0xfedcba00`, Supplicant Profile and PMK completed, and the
third command `ASSOCIATE_CMD 0x0012` timed out without `CMD_DONE`. Commit
`f77ca05` adds a bounded secret-free timeout fingerprint after verified cleanup;
focused tests, mutations, extractor selftest, release build, and independent
read-only acceptance are green. H23 hardware-proved it: command `0x0012`, length
132, and the expected request header were published; quiesce/cleanup succeeded,
but the response header remained `untouched_zero` and no `CMD_DONE` arrived.
Commit `37a2b15` now adds one Associate-timeout observation of whether firmware
cleared the HostCmd doorbell before quarantine; it does not alter publication,
timeout, completion, or DMA ownership. Focused tests, release build, and final
review are green, but the probe is not hardware-proven. The path is still
in-kernel; connection, traffic, domain execution, IOMMU fencing, and safe
kill/restart remain open.

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
