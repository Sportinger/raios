# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~10:35, root orchestrator active)

The first post-K2 Surface cold boot materially narrowed H20. The GUI and HID
remained usable. The recovered RECLOG contained three valid chained frames:
USB `errors=0`, the persisted pre-BME K2 checkpoint with PCI Command `0x0402`
(Memory Space on, BME off, interrupt-disable on), and a terminal HardwareSpec
`data_ring_unavailable` with host interrupt status zero. No network state was
granted. This is evidence that the K2/USB safety path survived that boot, not
proof of DMA drain, IOMMU containment, connection, or traffic.

`d8d8f34` is accepted ring telemetry for the next boot. Every event/RX arm and
host-pointer publication failure now emits one bounded, secret-free
`MarvellPublicationStep` cause code before quarantine; the generic HWSPEC trace
cannot replace it. The focused predicate and mutation negative are green, the
whole seed-kernel host typecheck is green, and two fresh read-only Codex reviews
returned ACCEPT. `4c748ab` separately repaired GPT GUID decoding on Windows
PowerShell 5.1 and its extractor selftest is green.

The Marvell path is still in-kernel and the isolation brake remains open.
`docs/SCOPE.md:155` and all mapped Wi-Fi/isolation boxes remain unchecked.

The next-test SanDisk is ready. Release packaging from main `b6fb30c` produced
kernel SHA-256 `0652ec58…f4cfe8`, signed/verified Core Policy slot A generation
1, and image SHA-256 `ea9240a6…d46813`. Source validation and the Windows 5.1
extractor selftest are green. The serial-pinned writer completed on
`0101d57ec458c24f1b93`; Windows re-observed both 128 MiB ESPs and the 256 MiB
SEED_DATA partition at the exact expected offsets.

## Next step

Perform one cold Surface boot from the prepared SanDisk and start Wi-Fi once.
No same-boot retry after quarantine. If networking still fails, shut down,
return the stick to Windows, and extract RECLOG: the new `0xD1KK_DDDD` value
identifies the exact ring operation plus decoder class or DMA index.

## Recently (exactly 3, newest first)

### 2026-07-21 — H21 serial-pinned Surface stick ready
Release build, policy binding, source validation, raw write, and GPT layout passed.

### 2026-07-21 — `d8d8f34` exact data-ring cause accepted twice
Specific ring failures persist before fail-closed quarantine; no gate was relaxed.

### 2026-07-21 — `4c748ab` PowerShell 5.1 GPT GUID parsing fixed
The read-only extractor now accepts exact 16-byte GUIDs and rejects wrong lengths.
