# Marvell Avastar 88W8897 (Surface Pro 4 WiFi) — native raiOS driver scoping

> STATUS: research/scoping only (2026-07-08). NO driver code exists; NO working WiFi is claimed.
> Owner decision (2026-07-08): pursue the built-in Marvell 88W8897 (Option 3) as a **side track**,
> in parallel with finishing the core ROADMAP milestones. Honestly labeled: any driver ships as an
> M11-sandboxed, M12-signed, owner-sealed high-privilege component; the vendor firmware blob is a
> closed, unaudited binary loaded verbatim and labeled public+unaudited.
>
> COVERAGE: this doc grounds (a) the CHIP+FIRMWARE-download protocol register-level from the Linux
> mwifiex_pcie source, (b) the raiOS PCIe/DMA transport plumbing, (c) the 802.11 SCAN command
> protocol (HostCmd envelope, ext-scan TLVs, async event/BSS-descriptor parsing — appended
> 2026-07-08 follow-up), and (d) a raiOS capability inventory (reuse vs must-build — appended
> follow-up). The scan section corrects an earlier assumption: mwifiex is a FULL-MAC driver, so
> scan-only is smaller than it looked (firmware does the radio work) but associate + WPA2 4-way
> handshake is LARGER (host-side MLME raiOS lacks). Remaining unknowns are silicon-only and flagged
> inline.

---

I have enough grounded context. Here is the synthesized scoping document.

---

# Scoping Plan — Native raiOS Driver for the Marvell Avastar 88W8897 (Surface Pro 4 WiFi)

Status: research/plan pass only. NO driver code is written here and NO claim of working WiFi is made. Target: PCIe device VID `0x11ab` / DID `0x2b38`, Microsoft subsystem vendor `0x045e` (Surface Pro 4 built-in WiFi). This document distinguishes what is KNOWN/documented from what is UNCERTAIN and requires hardware bring-up.

Primary specification source, because no public Marvell datasheet exists: the Linux `mwifiex_pcie` driver — `pcie.h`, `pcie.c`, `fw.h` under `drivers/net/wireless/marvell/mwifiex/`.

---

## 1. HONEST VERDICT

**Feasible in stages, but a usable WiFi link is a large, hardware-gated effort — and one milestone (firmware download → "firmware running") is genuinely bounded and worth doing first.**

The transport is not the hard part. raiOS already has a proven PCIe-DMA NIC (`e1000.rs`) that does everything the Marvell chip's *wire* needs: BAR mapping, uncached MMIO, DMA descriptor rings, physical-address translation, ordered volatile register I/O, and a poll loop. `wifi.rs` already correctly probes this exact chip. The 88W8897 firmware-download handshake is documented register-for-register in `mwifiex/pcie.c`. So "detect → map registers → stream the firmware blob → read back `FIRMWARE_READY_PCIE=0xfedcba00`" is a concrete, source-grounded target that reuses existing raiOS plumbing.

**The single biggest risk is the combination of a flaky closed firmware and a blind debug loop on the Surface.** Three compounding facts make this risk real, not theoretical:

1. **The firmware is a closed, unaudited ~785 KB vendor blob with a documented reputation for random crashes and resets** — upstream Linux carries a dedicated "Work around firmware bugs on 88W8897" patch series ([lore.kernel.org verdre series](https://lore.kernel.org/lkml/20210830123704.221494-1-verdre@v0yd.nl/T/)). When it wedges, the *documented* Linux recovery is a D3cold power-cycle of the device **and its parent PCI bridge** ([pali thread](https://lkml.kernel.org/netdev/20211012090037.v3w4za5hshtm253f@pali/T/)). raiOS has **no ACPI, no PCI power management, and no D-state facility** — grep of `seed-kernel/src` finds no power-management module. So a from-scratch driver may be able to *start* the chip but not *rescue* it, meaning every firmware crash could force a full cold reboot.

2. **There is no serial port and no wired Ethernet on the Surface Pro 4.** raiOS's existing drivers log via `serial::write_line` (e.g. `wifi.rs:119,143`), which works in the QEMU shadow VM but produces **nothing observable on real Surface hardware**. On the metal, the *only* built-in observability channel is the framebuffer (`framebuffer.rs` double-buffered `set_pixel`/`fill_rect`/`present`; text via `ui.rs`/`text.rs`). Debugging a DMA/doorbell race on a flaky closed firmware, blind except for pixels you paint yourself, is the dominant practical cost.

3. **Poll-only operation is unproven for this chip.** raiOS has no IDT/APIC/MSI (only `cli`/`hlt` in `main.rs:188,468,475`); `e1000` masks all interrupts and polls. The firmware *download* protocol is pure doorbell/poll and needs no interrupts — good. But whether the 8897 updates `PCIE_HOST_INT_STATUS` and completes ring DMA **without an enabled MSI vector** is UNKNOWN. If it silently requires MSI, the "small first milestone" grows an MSI/interrupt-controller prerequisite before anything works.

Bottom line: pursue the firmware-ready milestone because it is bounded and de-risks the biggest single unknown early. Do **not** promise WiFi. The full-MAC 802.11 command layer (scan/associate/WPA) above firmware-ready is a separate, much larger body of work that raiOS does not implement at all today (`wifi.rs` only *stores* an SSID and passphrase, `wifi.rs:164-201`).

---

## 2. WHAT raiOS ALREADY HAS vs WHAT'S MISSING

### Already has (reusable, proven)
- **PCI enumeration + config I/O**: `pci::find_device`, `read_bar_info` (incl. 64-bit BARs), `enable_bus_master` — `pci.rs:61-174,176-194`. Legacy `0xCF8/0xCFC` mechanism `pci.rs:264-314`.
- **Correct probe of the exact target**: `pci::find_device(0x11ab, 0x2b38)`, subsystem read at config `0x2c`, Microsoft-Surface tag when subsystem vendor `== 0x045e`, `Missing` when absent — `wifi.rs:112-158,210-227`.
- **Uncached MMIO mapping for device registers**: `memory::map_mmio` maps with `NO_CACHE | WRITE_THROUGH` into a fixed MMIO window — `memory.rs:111-139,159-162`. Exactly right for the register BAR.
- **DMA physical-address translation**: `memory::virt_to_phys` — `memory.rs:92-105`.
- **A complete working PCIe-DMA NIC as the template**: BAR read → bus-master → MMIO map → static-mut descriptor rings → volatile `read32`/`write32` → `compiler_fence`-ordered tail-bump → polled `receive`/`transmit` — `e1000.rs:120-125,159-176,232,264-384,409-415`.
- **Timing primitives for the poll/timeout loops**: `rdtsc()` + PIT-calibrated `tsc_per_ms()` — `time.rs:63-127`. Enough to build the µs busy-waits (`usleep_range(10,20)`) and ms-paced (`msleep(100)`) loops the firmware download needs. (No blocking-sleep helper is exposed yet; the ingredients exist.)
- **The whole stack above Ethernet**: smoltcp DHCP/DNS/TCP/TLS terminates on a `phy::Device` (`E1000Phy` `receive`/`transmit`, `net.rs:513-541`). A WiFi phy plugs in identically.
- **A framebuffer to observe results without serial**: `framebuffer.rs` (`set_pixel`, `fill_rect`, double-buffered `present`), text via `ui.rs`/`text.rs`.
- **A poll-only main loop** the WiFi status-register poll fits into — `net.rs:525-527`.

### Missing (must be built)
- **Firmware-blob delivery.** e1000 needs no firmware, so raiOS has **no** path to get a ~785 KB binary into the kernel. No filesystem loader; no `include_bytes!` usage anywhere in `seed-kernel/src`. Must choose: embed via `include_bytes!` (bloats the kernel image by ~785 KB) or ship through the agent-protocol module/provider channel. Either way needs an ADR-0004 classification decision (see §8).
- **The firmware downloader state machine** (256-byte block loop, CRC-retry, doorbell, `FIRMWARE_READY_PCIE` poll). No analog exists.
- **Bit-packed shared ring-index registers.** e1000 uses separate RDT/RDH, TDT/TDH. The 8897 packs host TX-write + RX-read indices into one 32-bit register (`PCIE_WR_DATA_PTR_Q0_Q1=0xC05C`) and firmware indices into another (`PCIE_RD_DATA_PTR_Q0_Q1=0xC08C`), with `tx_start_ptr=16`, `tx_mask=0x03FF0000`, `rx_mask=0x000003FF`, wrap masks `0x07FF0000`/`0x000007FF`. New logic.
- **PFU per-buffer descriptors**: `struct mwifiex_pfu_buf_desc { u16 flags; u16 offset; u16 frag_len; u16 len; u64 paddr; u32 reserved }` packed, with SOP/EOP/rollover flags — vs e1000's status-byte descriptor.
- **The command/response mailbox** (write cmd + cmd-response buffer phys addrs into scratch regs, ring doorbell, read completion in `PCIE_HOST_INT_STATUS`).
- **The TxPD/RxPD data-path header.** HONESTY CORRECTION to the task framing: mwifiex data frames are **not** bare Ethernet — every frame carries a driver-added `TxPD`/`RxPD` packet descriptor that the driver must prepend on TX and strip on RX before handing 802.3 to smoltcp. `E1000Phy` (`net.rs:513-541`) consumes the *resulting* 802.3 frame, but the WiFi phy must do the PD wrapping.
- **The full-MAC 802.11 command vocabulary**: `HostCmd_CMD_*` (e.g. `FUNC_INIT`, `GET_HW_SPEC`, scan, `802_11_ASSOCIATE`, key/WPA plumbing) with TLV payloads and a connect state machine. Absent entirely.
- **Larger, differently-sized DMA memory**: e1000's static-mut arrays are sized for 2 KB frames; the Marvell path needs 32×~4 KB RX (~128 KB) + TX + 8 event buffers + cmd/cmd-rsp/sleep-cookie + a staging path for the ~785 KB image. No general DMA allocator exists (static-mut + `virt_to_phys` only).
- **BAR-window correction (concrete gotcha).** Linux maps BAR0 (larger window, `pci_mmap`) *and* BAR2 (`pci_mmap1`) but does **all register I/O through BAR2**. raiOS `wifi.rs` currently reads only BAR0 base (`wifi.rs:210-227`). The driver must map the **register BAR (Linux resource index 2 = config offset `0x18`)**, and must account for BAR0 being 64-bit (it consumes config `0x10`+`0x14`), which shifts how `read_bar_info`'s index maps to config offset. Verify on silicon.
- **Firmware-crash recovery / power management** (D3cold + FLR-on-bridge). No facility.
- **Optionally MSI/IDT/APIC**, only if poll-only proves insufficient (see §7 unknown #1).

---

## 3. DRIVER ARCHITECTURE

Add one new module (proposed `seed-kernel/src/mwifiex.rs`), structured as layers that mirror `mwifiex_pcie`. Keep `wifi.rs` as the owner-facing config/probe surface; `mwifiex.rs` is the hardware engine it drives.

**Layer 0 — Transport bring-up** *(reuses e1000 plumbing almost verbatim)*
- Map the register BAR (BAR2 / config `0x18`), `enable_bus_master`, uncached MMIO map. Reuse `pci::read_bar_info` + `memory::map_mmio` + `pci::enable_bus_master` (`e1000.rs:159-176`). Provide `read32`/`write32` volatile helpers (`e1000.rs:409-415`).

**Layer 1 — DMA region + ring allocation** *(reuses e1000 static-mut + virt_to_phys pattern)*
- Static-mut, aligned regions for: firmware block buffer (`MWIFIEX_UPLD_SIZE=2312`), cmd + cmd-response + sleep-cookie buffers, and the TX(32)/RX(32)/event(8) rings with their PFU descriptors and data buffers. Phys addresses via `memory::virt_to_phys` (`memory.rs:92-105`). Program ring-index registers `0xC05C`/`0xC08C`. Same *shape* as `e1000.rs:277-311`, but packed indices + PFU descriptors are new.

**Layer 2 — Firmware loader** *(no analog in raiOS — the biggest new piece)*
- Read `SCRATCH_13 (0xCF4)`; if `== MWIFIEX_PCIE_FLR_HAPPENS (0xFEDCBABA)` take the combo-firmware WiFi-extract path. Block loop: poll `cmd_size (SCRATCH_2=0xC40)` for the firmware-requested length; if LSB set → CRC retry (cap `MAX_WRITE_IOMEM_RETRY=2`); DMA-map a `≤256`-byte-padded block, write its phys addr to `cmd_addr_lo/hi (SCRATCH_0/1=0xC10/0xC14)`, length to `SCRATCH_2`, ring `CPU_INTR_DOOR_BELL (BIT1)` into `PCIE_CPU_INT_EVENT (0xC18)`, poll `PCIE_CPU_INT_STATUS (0xC1C)` until the doorbell clears; advance. After the last block, write `FIRMWARE_READY_PCIE (0xfedcba00)` to `drv_rdy (SCRATCH_12=0xCF0)` and poll `fw_status (SCRATCH_3=0xC44)` until it reads back `0xfedcba00`. Grounded in `pcie.c` `mwifiex_prog_fw_w_helper`/`mwifiex_check_fw_status`; constants in `pcie.h`/`fw.h`.

**Layer 3 — Command/event + interrupt-or-poll** *(reuses e1000's poll loop model)*
- Poll `PCIE_HOST_INT_STATUS (0xC30)` in the main loop; decode `HOST_INTR_DNLD_DONE(BIT0)`/`UPLD_RDY(BIT1)`/`CMD_DONE(BIT2)`/`EVENT_RDY(BIT3)`; write-clear with the bitwise complement (Linux ISR behavior). Command send = write cmd + cmd-rsp buffer phys addrs to scratch regs, ring doorbell. This is the same poll-and-clear discipline as `e1000.rs` descriptor polling. **First cut is polling; MSI is deferred** (see §7 #1).

**Layer 4 — 802.11 scan** *(new; first *functional* WiFi step)*
- Issue `GET_HW_SPEC` and a scan command over the mailbox, collect scan-result events from the event ring, decode SSIDs. This is the first milestone's "real WiFi did something" proof (§4).

**Layer 5 — Associate + WPA + data path** *(new; large; deferred — §5)*
- Associate command, WPA2/WPA3 key handshake (**runs in firmware**, not in raiOS), TxPD/RxPD data-path wrapping, then a `WifiPhy` mirroring `E1000Phy` (`net.rs:513-541`) so smoltcp DHCP/DNS/TCP/TLS runs unchanged.

Reuse map at a glance: BAR/bus-master/MMIO → `e1000.rs:159-176` + `memory.rs:111-139`; DMA phys → `memory.rs:92-105`; ring setup/tail-bump discipline + volatile I/O + fences → `e1000.rs:264-415`; timing/timeouts → `time.rs:63-127`; smoltcp handoff → `net.rs:513-572`. Genuinely new: the firmware loader, packed shared-index registers, PFU descriptors, TxPD/RxPD, and the HostCmd vocabulary.

---

## 4. FIRST MILESTONE (hardware-verifiable, no serial)

**Goal:** the smallest *real* win — get the closed firmware running and prove the chip is alive and talking, observed entirely on the framebuffer.

**Scope (in order):**
1. Detect chip (already done, `wifi.rs:112-158`).
2. Map the **register BAR (BAR2)**, `enable_bus_master`, read a stable register (chip revision `0x1100`/`0x1200`, or a known scratch value) to prove MMIO reads work.
3. Allocate DMA regions; program ring-index registers; prove `virt_to_phys` yields addresses the chip accepts.
4. Run the firmware block-download state machine (Layer 2).
5. Poll `fw_status (0xC44)` to `FIRMWARE_READY_PCIE=0xfedcba00`; write the magic to `drv_rdy (0xCF0)`.
6. **Stretch, same milestone:** issue one benign command (`GET_HW_SPEC`) and read its response via polled `HOST_INT_STATUS` — this proves the mailbox and confirms the poll-only assumption. A scan → SSID list is the *next* step; keep it out of the "done" bar unless #1–6 land cleanly.

**Definition of "done":** on real Surface Pro 4 hardware, from cold boot, the driver streams the embedded firmware and `fw_status` reads back `0xfedcba00` within the multi-second timeout, and (stretch) `GET_HW_SPEC` returns a well-formed response with a plausible MAC/hw-spec.

**How it's observed with NO serial port:** a dedicated framebuffer status panel (`framebuffer.rs` + `ui.rs`/`text.rs`), painted as a **state ladder** so a blind failure still tells you *where* it stopped:
- `PROBE OK  vid:did subsys` → green
- `BAR2 MAP  base=… size=…` → green
- `FW DNLD   block N / total, retries R` → live counter (proves the block loop is advancing, not hung)
- `FW READY  fw_status=0x…` → green on `0xfedcba00`, red + the actual value on timeout
- `HWSPEC    mac=… ` (stretch)

The last value shown *is* the diagnosis. Use distinct colors/positions per stage so a photo of the screen is a complete bug report. In the QEMU shadow VM the same run also logs to serial (existing `serial::write_line` path) — serial is the rich channel for the VM dev loop; the framebuffer ladder is the *only* channel that survives on the metal.

---

## 5. LATER MILESTONES (scoped, not first)

- **L5a — Scan → SSID list on framebuffer.** Scan command + event-ring result parsing. First visible "WiFi works" moment for the owner.
- **L5b — Associate (open network).** `HostCmd_CMD_802_11_ASSOCIATE` + connect state machine; verify link-up event. Prove the data path with an open AP before adding crypto.
- **L5c — WPA2/WPA3 key handshake.** The 4-way EAPOL handshake runs **in firmware**; raiOS drives it via key commands. Feed the passphrase already stored in `wifi.rs`.
- **L5d — Data path + smoltcp.** TxPD/RxPD wrap/unwrap; add `WifiPhy` mirroring `E1000Phy`; run DHCP → DNS → TCP → TLS over WiFi unchanged (`net.rs:513-572`).
- **L5e — Crash recovery.** Detect firmware wedge; attempt FLR (`SCRATCH_13=0xFEDCBABA` re-download path) since raiOS cannot do D3cold. Honestly may cap out at "reboot to recover" until power management exists.
- **L5f — Throughput/stability hardening.** Only meaningful once poll-only is proven adequate (or MSI is added).

---

## 6. DEBUG STRATEGY (no serial, no wired Ethernet on the Surface)

- **Two-tier loop.** Do *all* logic bring-up in the **QEMU shadow VM first**, where `serial::write_line` gives full traces and the harness captures reports. QEMU cannot emulate the real 8897, so VM proves *code structure/state-machine* correctness only — never firmware behavior. Real silicon is the gate for anything firmware-touching.
- **Framebuffer state ladder as primary metal telemetry** (§4). Make every stage paint a distinct region so a phone photo is a full status snapshot. Add a scrolling hex mini-console for the last N register writes/reads.
- **On-screen register dump on failure.** On any timeout, paint the raw scratch registers (`0xC40/0xC44/0xCF0/0xCF4`) and the `HOST_INT_STATUS`. These four values distinguish "firmware never started," "download stalled mid-image," and "ready but mailbox dead."
- **USB-tether as a bootstrap dev network (later).** For iterating faster than reflash-and-photograph: once raiOS can drive a USB CDC-Ethernet dongle (separate effort), a tethered link becomes a real log/telemetry channel independent of the WiFi chip under test — the pragmatic way to get bytes off the Surface without serial. Until then, the framebuffer is it.
- **Cold-power discipline.** Because there is no D3cold recovery, treat **full power-off between attempts** as the reset primitive; do not assume a warm reboot clears a wedged chip.
- **Blob integrity check on-screen.** Paint the embedded firmware's length + a checksum at boot, so a truncated/corrupt `include_bytes!` blob is caught before it looks like a chip fault.

---

## 7. HONEST EFFORT ESTIMATE

Ranges, not promises. Assumptions: single AI-agent-driven workflow with cheap Codex workers and Claude scope/review; physical reflash-and-photograph loop on one Surface Pro 4; no serial; no MSI initially; firmware delivered via `include_bytes!`.

| Milestone | Effort (engineering-weeks, wide) | Confidence | Dominant cost |
|---|---|---|---|
| Firmware delivery decision + embed + blob integrity | 0.5–1 | High | ADR/classification, not code |
| L0–L1 transport + DMA regions + BAR2 map | 1–2 | Med-High | BAR-window/64-bit-BAR correctness on silicon |
| L2 firmware downloader → **FIRMWARE_READY** | 2–5 | **Medium** | Blind hardware bring-up; poll timing; DMA phys correctness |
| L3 + `GET_HW_SPEC` round-trip | 1–3 | Medium | Confirms poll-only actually works |
| **First milestone total (§4)** | **~5–11** | **Medium** | Hardware unknowns, not the code volume |
| L5a scan → SSIDs | 2–4 | Medium | HostCmd/TLV + event parsing |
| L5b–L5d assoc + WPA + data path + smoltcp | 6–14 | **Low** | Large command vocabulary; WPA state; TxPD/RxPD; stability |
| L5e crash recovery without power mgmt | 2–? (open) | **Very Low** | May be blocked by missing D3cold |

The variance is dominated by hardware unknowns and the blind debug loop, not by lines of code. If poll-only fails (see below), add an MSI/IDT/APIC prerequisite worth several more weeks before *anything* past L1 works.

**Top 3 unknowns that only real hardware resolves:**
1. **Does the 8897 work poll-only (no MSI)?** Whether it updates `PCIE_HOST_INT_STATUS` and completes ring/cmd DMA without an enabled MSI vector. If not, the whole plan gains an interrupt-controller prerequisite. *(This is the single most schedule-defining unknown.)*
2. **Can raiOS recover a wedged firmware at all** without D3cold power-cycling the device and its parent bridge? If every crash needs a cold reboot, "usable WiFi" on this famously-flaky chip is severely capped until power management exists.
3. **Are the BAR layout and DMA bus addresses correct on real Surface silicon** — which config BAR is the register window, whether BAR0's 64-bit-ness shifts raiOS's BAR indexing, and whether `virt_to_phys` yields addresses the chip's DMA engine actually honors for all the new buffers plus the ~785 KB staging path.

Secondary source unknowns to close by direct grep *before* coding (cheap, not hardware): exact `MAX_POLL_TRIES`/`MWIFIEX_MAX_FW_POLL_TRIES`, exact `MWIFIEX_RX_DATA_BUF_SIZE`/`tx_buf_size` for 8897, the full `TxPD`/`RxPD` layouts in `fw.h`, and **`LICENCE.Marvell`'s exact reverse-engineering/modification/redistribution terms** (WHENCE only says "Redistributable; see LICENCE.Marvell").

---

## 8. HOW IT FITS raiOS SAFETY

The roadmap already names this exact item — **"Bare-metal Wi-Fi vs USB-Ethernet"** — as an M12+ direction (`ROADMAP.md:1020`), so this plan slots into existing governance rather than inventing new policy.

- **Firmware blob = `public` + explicitly `unaudited closed firmware` (ADR-0004 Memory Rule).** It is redistributable but not open, not readable, and not verifiable by raiOS. It must be a labeled fact with provenance (origin `git.marvell.com/mwifiex-firmware.git` via linux-firmware, version, size, sha256), classified as unaudited before it is embedded or exported. Never launder it into "trusted." Record `capability_denied` for "firmware integrity independently verified," because raiOS cannot.
- **Signed delivery (M12).** If the blob (or the driver module itself) ships through the module/candidate channel rather than being baked into the kernel image, it rides the existing signed-artifact path (ADR-0009 external artifact distribution; the M11-5/M12 signed-service machinery). Until the owner-key sealing ceremony, every label stays honestly `dev_key_not_owner_sealed` / `owner_sealed:false` — consistent with the current M10/M11/M12 posture in `ROADMAP.md:55-76`.
- **The driver is an owner-sealed high-privilege component, not a sandboxable Wasm service.** A native PCIe driver needs raw MMIO, bus-master DMA, and physical-memory access — it **cannot** live behind the M11 Wasm capability-grant boundary (ADR-0008 per-service Wasm import grants), which is precisely for the *replaceable, capability-scoped* parts of the system. So the honest safety model is: the WiFi driver is a **kernel-resident, owner-sealed, high-trust** component whose privilege is acknowledged, minimized, and audited — the opposite of the "everything-else-as-a-service" M11 direction. It belongs in the *slim permanent core* category (boot + network bring-up), not the service layer (`ROADMAP.md:1013-1015`).
- **Least authority within the kernel.** Even as a privileged component, scope it: it should only ever touch its own mapped BAR and its own DMA regions, expose a narrow typed interface to `net.rs` (frames in/out) and to `wifi.rs` (config/status), and surface a durable capability fact — e.g. `wifi.link = capability_denied` until associate+key succeed on hardware — so the owner dashboard never overclaims WiFi that isn't real. This matches the repo's honesty discipline (`no_dishonest_overclaim`, `capability_denied` when evidence is missing).
- **Blast-radius honesty.** Document up front that a bus-master-capable driver for a crash-prone closed firmware is a real attack/faults surface, and that raiOS today lacks an IOMMU and PCI power management to fully contain or recover it. That limitation is a *stated* fact, sealed with the driver, not a silent gap.

---

*Grounding: in-repo — `wifi.rs:8-10,112-158,164-201,210-227`; `e1000.rs:120-125,159-176,232,264-415`; `pci.rs:61-174,176-194,264-314`; `memory.rs:92-105,111-177`; `net.rs:513-572`; `time.rs:63-127`; `main.rs:188,468,475`; `framebuffer.rs`; `ROADMAP.md:1009-1023,1020`; ADR-0004/0008/0009. External — mwifiex [`pcie.h`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/pcie.h), [`pcie.c`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/pcie.c), [`fw.h`](https://github.com/torvalds/linux/blob/master/drivers/net/wireless/marvell/mwifiex/fw.h); firmware [WHENCE](https://raw.githubusercontent.com/jakeday/mwifiex-firmware/master/WHENCE); firmware-bug [patch series](https://lore.kernel.org/lkml/20210830123704.221494-1-verdre@v0yd.nl/T/); D3cold/FLR [pali thread](https://lkml.kernel.org/netdev/20211012090037.v3w4za5hshtm253f@pali/T/) and [pci patch](https://patchwork.kernel.org/project/linux-pci/patch/20210709145831.6123-3-verdre@v0yd.nl/); Surface [linux-surface wiki](https://github.com/linux-surface/linux-surface/wiki). Any value not traceable to one of these is marked UNKNOWN and gated on datasheet/hardware.*

---

## 802.11 SCAN COMMAND PROTOCOL (grounded)

> STATUS: research/scoping only (2026-07-08). Follow-up pass closing the SCAN facet that the first pass left open. NO driver code; NO working scan is claimed. This section is grounded in the Linux `mwifiex` full-MAC driver (`drivers/net/wireless/marvell/mwifiex/`) and in current raiOS source. Every constant is quoted from `fw.h`; anything not traceable to source is marked **UNKNOWN**.

This assumes the firmware-ready milestone (§4) has already succeeded: the closed blob is running, `fw_status` read back `0xfedcba00`, and the command/response + event mailbox over the PCIe ring works (prior doc §3 Layer 3). Everything below is the layer that turns "firmware alive" into "a printable list of SSIDs in range," **without associating**.

Key honesty note up front: `mwifiex` is a **full-MAC** driver. The 802.11 MLME (scanning state machine, join, auth, 4-way handshake sequencing) lives partly in firmware and partly in Linux userspace (`wpa_supplicant`) — **not** in the driver. That cuts both ways: scan-only is *smaller* than it looks (the firmware does the radio work; the host just issues one command and parses TLVs), but associate+WPA is *larger* than the prior doc implied (the 4-way handshake is **host** work in the mainline path, see §F).

---

### A. The host-command model — the `HostCmd_DS_COMMAND` envelope

Every command and every command-response shares one 8-byte generic header, then a per-command body. From `fw.h`:

```c
struct host_cmd_ds_command {
    __le16 command;   // opcode, e.g. HostCmd_CMD_802_11_SCAN_EXT
    __le16 size;      // total bytes incl. this header + body
    __le16 seq_num;   // host-assigned; echoed in the response
    __le16 result;    // 0 = success in the response
    union { ... } params;   // per-command body
} __packed;
```

`#define S_DS_GEN sizeof(struct host_cmd_ds_gen)` = the 8-byte header (`command`+`size`+`seq_num`+`result`). Command bodies are sized as `S_DS_GEN + sizeof(body) + tlv_len`.

Opcodes needed for scan-only, quoted from `fw.h`:

| Constant | Value | Role |
|---|---|---|
| `HostCmd_CMD_FUNC_INIT` | `0x00a9` | kick firmware into functional state |
| `HostCmd_CMD_GET_HW_SPEC` | `0x0003` | read MAC, region, `fw_cap_info` |
| `HostCmd_CMD_MAC_CONTROL` | `0x0028` | enable RX/TX |
| `HostCmd_CMD_CFG_DATA` | `0x008f` | RF calibration data (see §E note) |
| `HostCmd_CMD_802_11_SCAN` | `0x0006` | legacy scan (sync, results in cmd-resp) |
| `HostCmd_CMD_802_11_SCAN_EXT` | `0x0107` | **ext scan (async, results via event) — the 8897 path** |

Response matching (`cmdevt.c :: mwifiex_process_cmdresp`): the firmware ORs `HostCmd_RET_BIT` (`0x8000`) into the echoed opcode; the driver checks `orig_cmdresp_no & HostCmd_RET_BIT`, masks the id with `HostCmd_CMD_ID_MASK` (`0x0fff`), matches it against the outstanding command, and reads `le16_to_cpu(resp->result)`. So a scan-ext response arrives as `command == 0x8107`.

Sequence numbering (`cmdevt.c :: mwifiex_dnld_cmd_to_fw`): `host_cmd->seq_num = cpu_to_le16(HostCmd_SET_SEQ_NO_BSS_INFO(adapter->seq_num, bss_num, bss_type))` — for a single-interface raiOS first cut, `bss_num`/`bss_type` are 0 and `seq_num` is just a monotonic counter echoed back.

---

### B. Issuing a scan over the PCIe command ring

The command goes out through the same PCIe mailbox the prior doc grounded (`if_ops.host_to_card(adapter, MWIFIEX_TYPE_CMD, skb, ...)` in `cmdevt.c`). Concretely on the PCIe interface (`pcie.c`, prior doc §3 Layer 3): the driver writes the command-buffer physical address into the PCIe command scratch registers, rings the doorbell, and later reads completion from `PCIE_HOST_INT_STATUS (0xC30)` — `CMD_DONE(BIT2)` for the command-response, `EVENT_RDY(BIT3)` for asynchronous events. No new transport is needed for scan beyond what firmware-download + `GET_HW_SPEC` already exercise; scan is "just another HostCmd" on that mailbox.

The scan **command body** (`scan.c :: mwifiex_cmd_802_11_scan_ext`) is deliberately thin — the ext-scan body is only a reserved dword plus a TLV buffer:

```c
struct host_cmd_ds_802_11_scan_ext {
    u32 reserved;
    u8  tlv_buffer[];   // the whole scan config is TLVs
} __packed;
cmd->size = cpu_to_le16(sizeof(ext_scan->reserved) + scan_cfg->tlv_buf_len + S_DS_GEN);
```

The TLVs are built in `scan.c :: mwifiex_config_scan`. Every TLV starts with `struct mwifiex_ie_types_header { __le16 type; __le16 len; }`. The proprietary TLV ids are `PROPRIETARY_TLV_BASE_ID (0x0100)` + offset. For a **minimal "list every SSID on every channel"** scan you need only a few:

| TLV | Type value | Purpose in a minimal scan |
|---|---|---|
| `TLV_TYPE_WILDCARDSSID` | `0x0100+18 = 0x0112` | one entry with `ssid_len = 0` = match any SSID (broadcast probe) |
| `TLV_TYPE_NUMPROBES` | `0x0100+2 = 0x0102` | probes per channel (e.g. 2) |
| `TLV_TYPE_CHANLIST` | `0x0100+1 = 0x0101` | the channels to sweep |
| `TLV_TYPE_BSS_MODE` | `0x0100+206 = 0x01ce` | infra/any (ext-scan only) |
| `TLV_TYPE_BSSID` | `0x0100+35 = 0x0123` | omit / all-zero = don't filter by BSSID |

The channel list is an array of `mwifiex_chan_scan_param_set` inside `mwifiex_ie_types_chan_list_param_set`:

```c
struct mwifiex_chan_scan_param_set {
    u8     radio_type;              // 2.4GHz vs 5GHz band
    u8     chan_number;             // e.g. 1..14 for 2.4GHz
    u8     chan_scan_mode_bitmap;   // MWIFIEX_PASSIVE_SCAN / HIDDEN_SSID_REPORT / DISABLE_CHAN_FILT
    __le16 min_scan_time;
    __le16 max_scan_time;
} __packed;
```

Firmware limits (`fw.h`): `MWIFIEX_MAX_CHANNELS_PER_SPECIFIC_SCAN = 14`, `MWIFIEX_DEF_CHANNELS_PER_SCAN_CMD = 4`, `MWIFIEX_MAX_SSID_LIST_LENGTH = 10`, `MWIFIEX_USER_SCAN_CHAN_MAX = 50`. Linux splits a large sweep into several scan commands of ≤`MWIFIEX_DEF_CHANNELS_PER_SCAN_CMD` channels each (`scan.c :: mwifiex_scan_channel_list`) and issues them back-to-back. A raiOS first cut can start with a single command covering the 2.4GHz channels 1–14 (`radio_type = 0`, `chan_scan_mode_bitmap = 0` = active scan) to keep it to one command round-trip.

Legacy-vs-ext selection is a single branch (`scan.c :: mwifiex_scan_channel_list`):
```c
if (priv->adapter->ext_scan)
    cmd_no = HostCmd_CMD_802_11_SCAN_EXT;
else
    cmd_no = HostCmd_CMD_802_11_SCAN;
```
`adapter->ext_scan` defaults on in mainline mwifiex, so the 8897/PCIe path uses **ext scan**. (Whether the specific Surface 8897 firmware advertises ext-scan capability in `fw_cap_info` from `GET_HW_SPEC` is worth confirming on silicon — marked in §G. The legacy `0x0006` path is a documented fallback if not.)

---

### C. Async result / event delivery

There are **two different delivery models**, and this is the single most important thing to get right:

**Legacy scan (`HostCmd_CMD_802_11_SCAN`, 0x0006) — synchronous.** Results come back **in the command response itself** (`scan.c :: mwifiex_ret_802_11_scan`). The response body is:
```c
struct host_cmd_ds_802_11_scan_rsp {
    __le16 bss_descript_size;
    u8     number_of_sets;               // number of BSS entries
    u8     bss_desc_and_tlv_buffer[];     // packed BSS descriptors
} __packed;
```
There is **no "EVENT_SCAN_REPORT"** in this path — completion *is* the cmd-response. (`scan.c :: mwifiex_check_next_scan_command`: `if (!adapter->ext_scan) mwifiex_complete_scan(priv);` — i.e. legacy completes inline.) The prior task framing's assumption of a scan-report event applies only to ext scan; correcting that here.

**Ext scan (`HostCmd_CMD_802_11_SCAN_EXT`, 0x0107) — asynchronous.** The command-response (`scan.c :: mwifiex_ret_802_11_scan_ext`) carries only channel **statistics** TLVs (`TLV_TYPE_CHANNEL_STATS = 0x0100+198 = 0x01c6`), not BSS results. The actual BSS results are pushed later as one or more **events**:

- Event delivery (`cmdevt.c :: mwifiex_process_event`): the first dword of the event buffer is `event_cause`; the driver masks it with `EVENT_ID_MASK (0xffff)` and dispatches via `mwifiex_process_sta_event`.
- Event id (`fw.h`): `EVENT_EXT_SCAN_REPORT = 0x00000058`. (Related: `EVENT_BG_SCAN_REPORT = 0x00000018`; BSS num/type are packed in the high bits via `EVENT_GET_BSS_NUM`/`EVENT_GET_BSS_TYPE`.)
- Dispatch (`sta_event.c :: mwifiex_process_sta_event`):
  ```c
  case EVENT_EXT_SCAN_REPORT:
      ret = mwifiex_handle_event_ext_scan_report(priv, adapter->event_skb->data);
  ```

So for the 8897 the flow is: **issue `SCAN_EXT` → poll `HOST_INT_STATUS` → on `CMD_DONE` read the (stats-only) response → on `EVENT_RDY` read event(s) with cause `0x58` → parse the BSS TLVs out of the event body.** A raiOS scan-only loop must therefore drive both the command-response and the event ring; results arrive on the event ring, not the command-response.

---

### D. Parsing scan results — where SSID / BSSID / channel / RSSI live

**Ext-scan event body** (`scan.c :: mwifiex_handle_event_ext_scan_report`) is a stream of TLVs. Two matter per BSS:

```c
struct mwifiex_ie_types_bss_scan_rsp {           // TLV type 0x0100+86 = 0x0156
    struct mwifiex_ie_types_header header;
    u8 bssid[ETH_ALEN];        // <-- BSSID (6 bytes)
    u8 frame_body[];           // <-- the 802.11 mgmt frame body (fixed params + IEs)
} __packed;

struct mwifiex_ie_types_bss_scan_info {          // TLV type 0x0100+87 = 0x0157
    struct mwifiex_ie_types_header header;
    __le16 rssi;               // <-- RSSI (signed), see units note
    __le16 anpi;
    u8     cca_busy_fraction;
    u8     radio_type;
    u8     channel;            // <-- channel (also derivable from DS-Param IE)
    u8     reserved;
    __le64 tsf;
} __packed;
```

The loop pairs them: `TLV_TYPE_BSS_SCAN_RSP` gives **BSSID** + the frame body; `TLV_TYPE_BSS_SCAN_INFO` gives **RSSI** (`rssi = (s32)(s16)le16_to_cpu(scan_info_tlv->rssi)`), **channel**, and **TSF**. The frame body is then handed to `mwifiex_parse_single_response_buf`.

**The per-BSS fixed header + IE walk** (`scan.c :: mwifiex_parse_single_response_buf`), identical shape for both scan types, reads fields in this order:
1. `memcpy(bssid, current_ptr, ETH_ALEN)` — 6-byte **BSSID** (legacy path; for ext it came from the RSP TLV).
2. **legacy only:** `rssi = (s32)*current_ptr` (1 byte), then `rssi = (-rssi)*100` (magnitude of −dBm, scaled).
3. `struct mwifiex_fixed_bcn_param` = `__le64 timestamp; __le16 beacon_period; __le16 cap_info_bitmap;` — the standard 802.11 beacon fixed params.
4. The remainder is the **tagged IE buffer**, walked element-by-element.

**IE walk** (`scan.c :: mwifiex_update_bss_desc_with_ie`) — the loop is the generic 802.11 `element_id, element_len, value` TLV walk:
```c
while (bytes_left >= 2) {
    element_id  = *current_ptr;
    element_len = *(current_ptr + 1);
    total_ie_len = element_len + sizeof(struct ieee_types_header);  // header = 2 bytes
    // switch(element_id) ...
    current_ptr += total_ie_len;
    bytes_left  -= total_ie_len;
}
```
For a **first cut that prints just SSIDs**, only two element ids matter:

| Element id | Constant | What to read |
|---|---|---|
| **0** | `WLAN_EID_SSID` | `ssid_len = element_len` (0..32); the SSID bytes follow the 2-byte header. `element_len == 0` = hidden SSID. |
| 4 | `WLAN_EID_DS_PARAMS` | `bss_entry->channel = ds_param_set->current_chan` (1 byte) — the operating channel, when not taken from the scan-info TLV. |

(Others the full driver also pulls but a scan-only cut can skip: `WLAN_EID_SUPP_RATES` (1), `WLAN_EID_RSN` (48) / `WLAN_EID_VENDOR_SPECIFIC` (221, WPA OUI `00:50:f2:01`) → *these tell you the security type* if you later want to show "locked" networks, `WLAN_EID_HT_CAPABILITY` (45), `WLAN_EID_VHT_CAPABILITY` (191).) `WLAN_EID_*` come from Linux's generic `ieee80211.h`, not `fw.h` — note that when porting.

So the **absolute minimum extraction** for "SSIDs in range" is: for each `TLV_TYPE_BSS_SCAN_RSP`, take `bssid[6]`, skip the 12-byte fixed beacon params in `frame_body`, walk the IEs, and print the value of element id 0. Channel and RSSI (for a nicer list) come from the paired `TLV_TYPE_BSS_SCAN_INFO`.

---

### E. Absolute minimum command set: cold-firmware-ready → printable SSID list

Grounded in the init order in `sta_cmd.c :: mwifiex_sta_init_cmd` (full list is 17 commands; below is the honest *minimum subset* to reach a scan, with the rest called out as "mainline does it, likely skippable for scan-only"):

**Required:**
1. **`HostCmd_CMD_PCIE_DESC_DETAILS`** — PCIe-only, sent first in the mainline init. Hands firmware the host TX/RX/event/cmd ring descriptor layout. On PCIe this is almost certainly mandatory before firmware will DMA anything back. *(Confirm the body layout against `pcie.c` before coding — marked in §G.)*
2. **`HostCmd_CMD_FUNC_INIT` (0x00a9)** — transition firmware to functional state.
3. **`HostCmd_CMD_GET_HW_SPEC` (0x0003)** — returns permanent MAC, `region_code`, `fw_cap_info`, `fw_release_number` (`struct host_cmd_ds_get_hw_spec`). Needed to (a) confirm the mailbox works, (b) learn whether ext-scan/region gating apply. This is also the natural "stretch" of the firmware-ready milestone.
4. **`HostCmd_CMD_MAC_CONTROL` (0x0028)** — `struct host_cmd_ds_mac_control { __le32 action; }`. Enable the MAC with `HostCmd_ACT_MAC_RX_ON (BIT0) | HostCmd_ACT_MAC_TX_ON (BIT1) | HostCmd_ACT_MAC_ETHERNETII_ENABLE (BIT4)` = `0x13`. *(`mwifiex_cmd_mac_control` just writes the caller's `*action`; the actual default `curr_pkt_filter` is composed in mwifiex `init.c` — exact value to confirm, §G. TX must be on because active scan transmits probe requests.)*
5. **`HostCmd_CMD_802_11_SCAN_EXT` (0x0107)** with the TLV set from §B (wildcard-SSID len 0 + num-probes + channel-list + bss-mode).
6. **Drain events:** poll `HOST_INT_STATUS`, read every `EVENT_EXT_SCAN_REPORT (0x58)`, parse per §D, print SSIDs.

**Probably required for the radio to actually hear anything on real Surface silicon (not just build a well-formed command):**
- **`HostCmd_CMD_CFG_DATA` (0x008f)** — RF/calibration data. Whether the 8897 needs host-supplied cal data (vs self-contained in the firmware image) to produce a usable RF path is **UNKNOWN** and hardware-gated. If scans come back empty on metal, this is the first suspect.
- **Regulatory/region** — `GET_HW_SPEC.region_code` plus mainline's `HostCmd_CMD_CHAN_REGION_CFG`. If the region defaults to a restrictive domain, some channels may be silently skipped. A raiOS scan-only cut can start with 2.4GHz ch 1–11 (universally allowed) to sidestep this.

**Mainline sends but scan-only can likely skip:** `RECONFIGURE_TX_BUFF`, `802_11_PS_MODE_ENH` (power save), `TX_RATE_CFG`, `RF_TX_PWR`, `AMSDU_AGGR_CTRL`, `802_11_SNMP_MIB` (11D), `11N_CFG`, `PACKET_AGGR_CTRL` — these tune the data path / power / aggregation, none of which a passive-ish first scan strictly needs. Keep them out of the first cut; add back only if scans misbehave.

**Minimal happy path, end to end:** `PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL(0x13) → SCAN_EXT(ch1–11, wildcard) → {EVENT 0x58}* → print element-id-0 values`. That is on the order of **5 commands + an event drain** — genuinely small *if* the mailbox and event ring from the firmware-ready milestone already work.

---

### F. Brief scope (NOT first milestone): associate + WPA2 4-way + one data frame

This is called out only to size it honestly; it is **much** larger than scan and is deferred.

- **Associate** — `HostCmd_CMD_802_11_ASSOCIATE (0x0012)`. Body carries the target BSSID + a TLV stack (SSID, PHY/rate params, channel, and for WPA the RSN/WPA IE echoed from the scan result, plus auth params). Firmware performs the actual 802.11 auth+assoc exchange and reports link-up via an event. This needs an SME-style "pick a BSS from scan results, then join" state machine that raiOS does not have.
- **WPA2 4-way handshake — HONEST CORRECTION to the prior doc.** In the **mainline mwifiex path the 4-way handshake runs in the HOST** (`wpa_supplicant`, over EAPOL data frames), **not in firmware.** The driver only *installs the resulting keys* via `HostCmd_CMD_802_11_KEY_MATERIAL (0x005e)`. So raiOS would have to implement the supplicant itself: PMK derivation (PBKDF2-HMAC-SHA1 over the passphrase already stored in `wifi.rs`), the 4-way EAPOL-Key exchange (nonce exchange, PTK derivation via the PRF, MIC with HMAC-SHA1/AES-CMAC), and GTK handling — then push PTK/GTK down with `KEY_MATERIAL`. That is a real crypto + state-machine body of work with no analog in raiOS today. (Some Marvell firmware supports an *embedded* supplicant offload, but that path is not what mainline uses and its availability on this blob is UNKNOWN.)
- **One data frame** — requires the `TxPD`/`RxPD` packet-descriptor wrapping the prior doc already flagged (mwifiex frames are not bare 802.3), plus a `WifiPhy` mirroring `E1000Phy` so smoltcp runs unchanged. Only meaningful after assoc+keys succeed.
- **The cfg80211 / mac80211 layers raiOS lacks.** `mwifiex` is full-MAC and binds to **cfg80211** (not mac80211 — the MLME is in firmware). For **scan-only, raiOS needs none of this** — it issues the HostCmd and parses TLVs directly, which is exactly why scan-only is the right first functional milestone. For **associate+WPA**, raiOS must replicate the roles cfg80211 + wpa_supplicant play in Linux: BSS selection / SME, the regulatory domain, the netlink control plane, and the EAPOL supplicant. That replication — not the HostCmds themselves — is the bulk of the "real WiFi link" cost.

---

### G. UNKNOWNS for scan-only (honest list)

1. **Ext-scan vs legacy on this exact blob.** Mainline defaults `adapter->ext_scan = true`, so the plan assumes `SCAN_EXT (0x0107)` + `EVENT_EXT_SCAN_REPORT (0x58)`. Whether the specific Surface Pro 4 8897 firmware advertises ext-scan in `GET_HW_SPEC.fw_cap_info` (and thus which path actually works) is **UNKNOWN — verify on hardware**; keep the legacy `0x0006` synchronous path as a fallback.
2. **Does scan work poll-only (no MSI)?** Inherits the prior doc's #1 unknown. Ext-scan *depends* on the event ring delivering `EVENT_RDY`/`0x58` asynchronously. If the 8897 won't raise event-ring completions without an enabled MSI vector, scan-only is blocked behind an interrupt-controller prerequisite even though the command itself is trivial. **UNKNOWN — the schedule-defining risk.**
3. **`PCIE_DESC_DETAILS` body layout.** The exact structure raiOS must hand firmware to describe its rings is in `pcie.c`, not yet transcribed. **Needs a direct `pcie.c` read before coding.**
4. **Whether `CFG_DATA` / region config gate a *usable* RF path.** A well-formed `SCAN_EXT` may still return zero BSSes if the firmware needs host cal data or the region code disables the scanned channels. **UNKNOWN — hardware-gated; mitigate by starting on ch 1–11.**
5. **Exact `MAC_CONTROL` default filter** (`curr_pkt_filter`) composed in mwifiex `init.c` — the `RX_ON|TX_ON|ETHERNETII_ENABLE` = `0x13` guess is from the flag definitions, not the composed constant. **Confirm in `init.c`.**
6. **Legacy per-BSS framing detail.** The exact byte layout that delimits successive BSS descriptors in the legacy `bss_desc_and_tlv_buffer` (the per-entry IE-length field) was not transcribed field-for-field; only the ext-scan TLV layout is fully pinned here. **Needs a direct `mwifiex_ret_802_11_scan` read if the legacy fallback is used.**
7. **RSSI units for display.** Ext: signed `s16` in `bss_scan_info.rssi`; legacy: `(-byte)*100`. Both need a unit/normalization decision if the SSID list shows signal strength. Not blocking for an SSID-only list. **Secondary.**
8. **`WLAN_EID_*` provenance.** SSID (0), DS-Params (4), RSN (48) element ids come from Linux's generic `ieee80211.h`, **not** `fw.h`. raiOS must define these itself; trivial but easy to get subtly wrong (e.g. RSN vs vendor-WPA). **Note when porting.**

*Grounding: mwifiex — [`fw.h`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/fw.h) (all opcodes/structs/TLV ids/event codes quoted above), [`scan.c`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/scan.c) (`mwifiex_cmd_802_11_scan_ext`, `mwifiex_config_scan`, `mwifiex_scan_channel_list`, `mwifiex_ret_802_11_scan_ext`, `mwifiex_handle_event_ext_scan_report`, `mwifiex_parse_single_response_buf`, `mwifiex_update_bss_desc_with_ie`), [`cmdevt.c`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/cmdevt.c) (`mwifiex_dnld_cmd_to_fw`, `mwifiex_process_cmdresp`, `mwifiex_process_event`), [`sta_cmd.c`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/sta_cmd.c) (`mwifiex_sta_init_cmd` order), [`sta_event.c`](https://raw.githubusercontent.com/torvalds/linux/master/drivers/net/wireless/marvell/mwifiex/sta_event.c) (`EVENT_EXT_SCAN_REPORT` dispatch). In-repo — `wifi.rs:112-208` (probe + SSID/passphrase config only; no `HostCmd`/scan logic exists, confirmed by grep), prior §3 Layer 3 PCIe mailbox. Anything not traceable to these is marked UNKNOWN and gated on `pcie.c`/`init.c` read or hardware bring-up.*

---

## raiOS CAPABILITY INVENTORY (what exists vs missing)

> This is a precise, file:line inventory of the raiOS `seed-kernel` primitives a DMA-driven PCIe WiFi driver depends on, produced by reading the actual source. Each item states KNOWN (grounded in code) vs UNKNOWN (needs datasheet/hardware). Grounding for the firmware size: `pcie8897_uapsta.bin` is **803,884 bytes (~785 KB)** ([kernel.googlesource.com mrvl tree](https://kernel.googlesource.com/pub/scm/linux/kernel/git/vkoul/firmware/+/byt_new/mrvl/pcie8897_uapsta.bin?autodive=0%2F%2F), version 15.68.19.p21 per [Debian firmware-libertas](https://packages.debian.org/bullseye/firmware-libertas)).

### 1. PCI config-space access + device/BAR discovery — **EXISTS, reusable**
`seed-kernel/src/pci.rs` is a complete legacy-mechanism (`0xCF8`/`0xCFC`) PCI layer:
- Device scan: `pci::find_device(vendor, device)` brute-forces every bus/dev/func — `pci.rs:176-194`; class scan `find_by_class` — `pci.rs:196-220`.
- Config read: `PciAddress::read_u32/read_u16/read_u8` — `pci.rs:61-75`; write: `write_u16/write_u32` — `pci.rs:77-83`; low-level port I/O `pci_config_read_u32` / `config_address` / `outl` / `inl` — `pci.rs:264-314`.
- BAR decode with size-probing: `read_bar_info(address, index)` — `pci.rs:92-174`. Handles I/O, `Memory32`, and **64-bit `Memory64`** BARs, correctly consuming two config dwords for the 64-bit case — `pci.rs:144-167`. Returns `PciBar { index, kind, base, size }`.
- Bus-master enable: `enable_bus_master` sets command-register `0x04` bits `0|1|2` (I/O, memory, **bus master**) — `pci.rs:86-90`.
- The exact target chip is already probed: `find_device(0x11ab, 0x2b38)` + subsystem read at config `0x2c` + Microsoft-Surface tag — `wifi.rs:118-158`.

**Gap (not a missing capability, a wiring fix):** `wifi.rs` currently reads only BAR0 via a hand-rolled `read_bar0_base` — `wifi.rs:210-227`. mwifiex does all register I/O through **BAR2** (Linux resource index 2 = config offset `0x18`). The driver must call `pci::read_bar_info(address, 2)`; the generic decoder already supports this. Because BAR0 is 64-bit it consumes config `0x10`+`0x14`, so index→offset mapping must be verified on silicon. **UNKNOWN — needs hardware** to confirm which BAR index is the register window on the real Surface part.

### 2. MMIO mapping of a BAR into CPU memory — **EXISTS, reusable, correct flags**
- `memory::map_mmio(phys, len) -> MmioMapping` — `memory.rs:111-139`. Reserves virtual space in a fixed 16 MB MMIO window (`MMIO_WINDOW_BASE = 0xffff_ffff_c000_0000`, `MMIO_WINDOW_SIZE = 16 MiB`) — `memory.rs:18-19`, and maps pages with **`PRESENT | WRITABLE | NO_CACHE | WRITE_THROUGH`** — `memory.rs:159-162` — exactly right for a device register BAR. `MmioMapping::as_ptr::<T>()` — `memory.rs:44-52`.
- Requires HHDM (`mmio_ready`) — `memory.rs:107-109,115-117`.
- Page-table frames for these mappings come from a **static pool of only 64 pages** (`PAGE_TABLE_POOL_PAGES = 64`, `StaticFrameAllocator`) — `memory.rs:20,35-36,190-208`. This caps how many distinct MMIO regions/page-tables can be built, not the mapping size itself.
- Proven callers: e1000 — `e1000.rs:170-176`; xHCI (maps `min(size, 1 MiB)`) — `usb.rs:374-377`.

### 3. DMA-capable, physically-contiguous, phys-addressable memory — **CAPABILITY EXISTS (two proven mechanisms); only a typed allocator is missing**
There is **no** DMA allocator that hands back a `(virt, phys)` handle, but two working mechanisms already produce exactly the memory a WiFi driver needs:
- **`memory::virt_to_phys(ptr)`** — `memory.rs:92-105`: for any kernel-image address (`virt >= virtual_base`) it returns `virt - virtual_base + physical_base` (a linear offset), else identity-maps low addresses. It relies on Limine loading the kernel image at a **contiguous** physical range.
- **Static-mut `.bss` buffers + per-buffer `virt_to_phys`** — the pattern e1000 uses: `RX_DESCS/TX_DESCS/RX_BUFFERS/TX_BUFFERS` — `e1000.rs:122-125` (RX_BUFFERS = 32×2048 = **64 KB** of contiguous DMA target), programmed via `virt_to_phys` — `e1000.rs:268,276,294,303`. xHCI does the same at larger scale (command/event rings, DCBAA, 64 scratch pages, device contexts) — `usb.rs:548-567`.
- **A 64 MiB kernel heap exists**: `linked_list_allocator::LockedHeap` over `static mut HEAP: [u8; 64 MiB]` — `main.rs:17,174-182,207-208`. Because `HEAP` is a static array **inside the linearly-mapped kernel image**, heap allocations are themselves physically contiguous and `virt_to_phys`-translatable — the same property e1000's multi-page 64 KB rings already depend on and prove in the VM.

**Verdict for the WiFi driver:** the ~785 KB firmware blob, the TX(32)/RX(32)/event(8) rings, PFU descriptors, and cmd/cmd-rsp/sleep-cookie buffers all fit inside the 64 MiB heap **or** as static-mut arrays. Note the download is streamed in ≤`MWIFIEX_UPLD_SIZE` (2312-byte) blocks, so the *DMA staging* buffer per block is tiny — only the source blob must be resident, which the heap absorbs trivially.
**Caveats / UNKNOWN — needs hardware:** (a) physical contiguity across the full ~785 KB spans ~192 pages and depends on Limine's contiguous kernel load — high-confidence because e1000's working multi-page rings already rely on it, but confirm on Surface silicon; (b) there is **no IOMMU** — bus-master DMA is entirely unguarded (a stated blast-radius fact, not a fixable gap here).

### 4. Interrupt handling vs polling — **POLL-ONLY MODEL EXISTS (matches the firmware handshake); MSI is UNKNOWN**
- raiOS has **no IDT, no APIC, no MSI** anywhere in `seed-kernel/src` (a grep for `InterruptDescriptorTable`/`set_handler_fn`/`lidt`/`apic`/`MSI` returns only the unrelated xHCI `TRB_IDT` immediate-data flag — `usb.rs:75`). The only CPU-interrupt use is `cli` at boot — `main.rs:188` — and `cli; hlt` in the panic/halt paths — `main.rs:467-475`.
- e1000 is **pure polling**: it masks *all* NIC interrupts (`write32(REG_IMC, 0xFFFF_FFFF)` — `e1000.rs:232`) and its `receive()`/`transmit()` are driven from the main loop.
- The main loop is a cooperative TSC-gated poll loop: `loop` — `main.rs:274`; `PeriodicTask::try_run` gates each subsystem by a TSC interval — `scheduler.rs:16-35`; `net::poll()` is called on its cadence — `main.rs:375`, alongside USB rescan/input/provider polls — `main.rs:354-382`.

**Verdict:** the WiFi driver can use the *identical* model — register a `wifi::poll()` `PeriodicTask` that polls `PCIE_HOST_INT_STATUS` and clears bits. The firmware **download** protocol is pure doorbell/poll and needs no interrupts, so the first milestone fits perfectly. **UNKNOWN — needs hardware:** whether the 8897 completes ring/command DMA and updates `HOST_INT_STATUS` with **no MSI vector enabled**. If it silently requires MSI, an IDT/APIC/MSI subsystem (which does not exist) becomes a prerequisite — the single most schedule-defining unknown.

### 5. Timers / delays for firmware-handshake waits — **EXISTS (rdtsc + calibrated tsc_per_ms); one thin helper missing**
- `time::rdtsc()` — `time.rs:63-76`; `calibrate_tsc()` (PIT channel-0 rate-generator calibration) — `time.rs:78-123`; `tsc_per_ms()` — `time.rs:125-127`.
- Monotonic milliseconds = `rdtsc()/tsc_per_ms` — `net.rs:737-740` (also `openai.rs:412`, `tls_io.rs:102`).
- Busy-wait building blocks already exist: `spin_delay(iterations)` — `usb.rs:2096-2102`; poll-with-timeout `wait_for(limit, cond)` — `usb.rs:2477`; ahci `wait_until`. These are iteration-count spins, not calibrated µs/ms.

**Verdict:** the ingredients for mwifiex's `usleep_range(10,20)` / `msleep(100)` waits exist — a calibrated delay is `poll until rdtsc() delta >= n * tsc_per_ms`. **Missing (trivial):** a `time::delay_us/delay_ms` helper; none is exposed today. **UNKNOWN — needs hardware:** PIT-based TSC calibration accuracy on real Surface silicon vs QEMU (affects timeout margins, not feasibility).

### 6. smoltcp `phy::Device` frame seam — **EXISTS, reusable**
- The smoltcp `Device` impl is `E1000Phy` — `net.rs:513-541`: `receive()` pulls a frame via `e1000::receive()` — `net.rs:525-528`; `transmit()` yields `E1000TxToken` — `net.rs:530-532`; `capabilities()` sets `Medium::Ethernet`, MTU `MAX_FRAME_SIZE = 1536` — `net.rs:534-540`. Tokens `E1000RxToken`/`E1000TxToken` — `net.rs:543-572`. Driven by `iface.poll(instant, &mut phy, &mut sockets)` — `net.rs:115`.
- Underlying frame I/O: `e1000::receive() -> RxPacket` (802.3 frame) — `e1000.rs:316-340`; `e1000::transmit(&[u8])` — `e1000.rs:354-384`.

**Verdict:** a `WifiPhy` mirroring `E1000Phy` plugs in identically, and the entire smoltcp DHCP/DNS/TCP/TLS stack runs unchanged. **Gap (new logic, not a missing seam):** mwifiex data frames are **not** bare 802.3 — each carries a driver-added `TxPD`/`RxPD` packet descriptor. `E1000Phy` consumes the *resulting* 802.3 frame; the WifiPhy must **prepend TxPD on TX and strip RxPD on RX** before handing 802.3 to smoltcp. That wrapping is new code.

### 7. The firmware-blob problem (no filesystem) — **DELIVERY MECHANISM EXISTS (proven `include_bytes!` + M12 signed channel); embed decision missing**
raiOS has **no filesystem loader**, but the embed pattern is proven and used today:
- `include_bytes!` is used **only** in `build.rs`-generated code for signed Wasm service artifacts: `ECHO_WASM_ARTIFACT_BYTES` — `build.rs:627`; `BUFECHO` — `build.rs:882`; `CERTWINDOW` — `build.rs:1137`, each `include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/*.wasm"))`. The 803,884-byte firmware could be embedded the same way (adds ~785 KB to the kernel image; the blob then lives in the linearly-mapped image and is directly DMA-source-addressable via `virt_to_phys`).
- Alternatively, the **M12 signed-candidate channel** is live in-tree — `module_candidate_channel.rs`, `module_candidate_intake.rs`, `granted_candidate_service.rs`, `agent_protocol_module_grant.rs`, with build-time P-256 verification (`build.rs`) and runtime gating (`descriptor_sources.rs`). A ~785 KB firmware blob could ride this as an external signed artifact, keeping it out of the kernel image and giving it provenance/classification.

**Verdict:** mechanism exists. **Missing:** the embed-vs-channel decision plus an ADR-0004 classification — the blob must be labeled `public` + `unaudited closed firmware` with provenance (origin, version 15.68.19.p21, size 803884, sha256) and a `capability_denied` for "integrity independently verified," and a boot-time blob-length/checksum self-check so a truncated `include_bytes!` is caught before it looks like a chip fault.

---

### MUST BUILD (missing)
- **Register-BAR selection for Marvell** — replace `wifi.rs:210-227`'s BAR0-only read with `pci::read_bar_info(address, 2)` (BAR2 register window); verify 64-bit-BAR index shift on silicon.
- **Typed DMA-handle allocator** returning `(virt, phys)` — today it is ad-hoc static-mut + per-buffer `virt_to_phys`; the raw capability exists, the ergonomic allocator does not.
- **Firmware downloader state machine** — block loop over SCRATCH regs, `CPU_INTR_DOOR_BELL`, `FIRMWARE_READY_PCIE=0xfedcba00` poll. No analog in raiOS.
- **Packed shared ring-index register logic + PFU descriptors** — vs e1000's separate RDT/RDH and status-byte descriptors (`e1000.rs:277-311`).
- **Command/event mailbox** — write cmd/cmd-rsp phys addrs to scratch regs, doorbell, poll/clear `PCIE_HOST_INT_STATUS`.
- **TxPD/RxPD wrap/strip** inside a new `WifiPhy` (mirrors `E1000Phy`, `net.rs:513-541`).
- **802.11 `HostCmd_*` vocabulary** — `GET_HW_SPEC`, scan, associate, WPA key plumbing with TLVs. Absent entirely.
- **Calibrated delay helper** (`time::delay_us/ms`) — trivial; ingredients in `time.rs:63-127` exist, the wrapper does not.
- **Firmware embed + ADR-0004 classification + boot-time blob-integrity self-check.**
- **(Contingent) MSI/IDT/APIC** — only if poll-only proves insufficient (§4 unknown). Nothing exists today.
- **Firmware-crash recovery / PCI power management** (D3cold + FLR-on-bridge) — no facility; likely caps recovery at "cold reboot."

### REUSE (exists, proven)
- **PCI enumeration + config I/O + BAR decode (incl. 64-bit) + bus-master enable** — `pci.rs:61-194,264-314`.
- **Correct probe of the exact chip** (`0x11ab:0x2b38`, subsystem) — `wifi.rs:118-158`.
- **Uncached MMIO mapping** (`NO_CACHE|WRITE_THROUGH`, 16 MiB window) — `memory.rs:111-162`; callers `e1000.rs:170-176`, `usb.rs:374-377`.
- **DMA phys translation + contiguous buffers** — `memory::virt_to_phys` `memory.rs:92-105`; proven by e1000's multi-page 64 KB rings `e1000.rs:122-125,264-311`.
- **64 MiB contiguous, translatable kernel heap** — `main.rs:174-208`.
- **Poll-only main loop + TSC-gated `PeriodicTask` scheduler** — `main.rs:274-384`; `scheduler.rs:16-35`.
- **e1000 as the full DMA-NIC template** (rings, volatile `read32`/`write32`, `compiler_fence` tail-bump, polled rx/tx) — `e1000.rs:230-416`.
- **Timing** — `rdtsc` + PIT-calibrated `tsc_per_ms` + monotonic `now_ms` — `time.rs:63-127`, `net.rs:737-740`.
- **Busy-wait / poll-with-timeout primitives** — `usb.rs:2096-2102,2477`.
- **smoltcp `phy::Device` seam** (a `WifiPhy` drops in unchanged) — `net.rs:513-572`, `iface.poll` `net.rs:115`.
- **`include_bytes!` artifact-embed pattern** — `build.rs:627,882,1137`.
- **M12 signed-candidate channel** for external artifact delivery — `module_candidate_channel.rs` / `module_candidate_intake.rs` / `granted_candidate_service.rs` / `agent_protocol_module_grant.rs` / `descriptor_sources.rs`.

**Net:** the entire *transport substrate* a DMA PCIe WiFi driver needs (PCI, bus-master, uncached MMIO, contiguous phys-addressable DMA memory, poll loop, timing, smoltcp seam, blob-embed) already exists and is proven by e1000/xHCI. Everything missing is **Marvell-specific protocol logic** (firmware download, packed ring indices, PFU/TxPD/RxPD, HostCmd vocabulary) plus two contingent unknowns that only real Surface hardware resolves: **poll-only viability (no MSI)** and **firmware-crash recovery without D3cold**.
