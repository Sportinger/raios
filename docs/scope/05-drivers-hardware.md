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
Commit `37a2b15` added one Associate-timeout observation without altering
publication, timeout, completion, or DMA ownership. H24 hardware-proved it:
three valid chained frames and a clean zero tail show USB `errors=0`, the same
correct untouched Associate request at 118.495 seconds, and a cleared HostCmd
doorbell. Firmware acknowledged notification but produced no response or
`CMD_DONE`. ADR 0037 therefore selects one post-PMK `GET_HW_SPEC` liveness
canary in place of Associate for H25. Commit `d617efd` implements that canary:
only a current-epoch expected completion is accepted, its response is discarded,
one secret-free result is retained, and every outcome quarantines without a
network grant or same-boot retry. Focused predicates, mutation negatives, 61
Marvell tests, 16 DMA-safety tests, unsafe-inventory verification, release build,
and one independent read-only review are green. The GPT A/B + SEED_DATA H25
Surface run produced six valid chained frames, a clean zero tail, and USB
`errors=0`. At 120.707 seconds the post-PMK canary completed with the expected
current-epoch `CMD_DONE`; network state remained denied and reboot-required as
designed. Generic post-PMK HostCmd mailbox liveness is therefore hardware-proven,
leaving Associate/BSS-specific setup or semantics as the next discriminator.
Linux `mwifiex` comparison identified one concrete unimplemented input: its
Associate builder appends `TLV_TYPE_TSFTIMESTAMP` (`0x0113`) with the firmware
scan TSF and AP beacon timestamp, while raiOS currently retains/appends neither.
H26 is therefore scoped to carry those two scan values into Associate and to
restore the real PMK -> Associate path. Three bounded implementation attempts
remain unaccepted: the final R3 review proved a Ready-replacement TOCTOU in
which a losing concurrent start can quiesce or erase the published winner, and
both executable models omit that Ready-origin interleaving. ADR 0045 parks H26
blocked with the Owner; no H26 image or USB write is authorized.
The path is still in-kernel; connection, traffic, domain execution, IOMMU
fencing, and safe kill/restart remain open.

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
