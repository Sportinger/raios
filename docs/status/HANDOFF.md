# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~11:22, root orchestrator active)

H22 is packaged and written to the serial-pinned SanDisk. The release kernel is
`89c8dfee816288a057d4fb9d999d86526350bfeef41a4acc9badc8610128a6e0`;
Core Policy slot A generation 1 is verified against that kernel, and the pinned
8897 firmware remains `cf4f51f41bd7ef4d7fe65fb76b8a2a0897bc70a0742bc4aea13d93b03fffd03a`.
Source validation, both ESP slots, Boot Control, SEED_DATA, and writer readback
are green. The transcript is `%TEMP%\raios-h22-usb-write-df2c71c.transcript.log`.

A second read-only extraction before the rewrite reproduced H21 exactly: three
valid chained RECLOG frames, clean zero tail, USB `errors=0`, PCI Command
`0x0402`, and terminal `MarvellPublicationStep=0xD1030001` at 42.757 seconds.
There was no newer boundary. Commit `c787320` removes only that premature
firmware-owned RX pointer read; runtime raw decoding and fail-closed quarantine
remain. ADR 0035 records the narrow decision. No Wi-Fi, traffic, domain, IOMMU,
or isolation checkbox closes until the physical H22 result exists.

Code, decision, and status through `df2c71c` are pushed on `main`. The prepared
stick boots that exact tree plus the kernel identity above.

## Next step

Perform exactly one cold Surface boot and start Wi-Fi once; do not use same-boot
Retry after quarantine. Record the first visible result. On any failure, power
down, return the stick without rewriting it, and extract RECLOG first.

## Recently (exactly 3, newest first)

### 2026-07-21 — H22 persistent Surface stick prepared
Kernel, signed policy, firmware, A/B layout, SEED_DATA, and readback are green.

### 2026-07-21 — `c787320` defers the premature RX pointer read
RX construction matches upstream ownership; runtime all-ones rejection remains.

### 2026-07-21 — H21 persisted exact `0xD1030001`
USB stayed clean and the Marvell failure localized to pre-registration `0xC08C`.
