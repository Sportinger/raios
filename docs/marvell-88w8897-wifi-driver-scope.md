# Marvell Avastar 88W8897 (Surface Pro 4 WiFi) — native raiOS driver scoping

> STATUS: research/scoping only (2026-07-08). NO driver code exists; NO working WiFi is claimed.
> Owner decision (2026-07-08): pursue the built-in Marvell 88W8897 (Option 3) as a **side track**,
> in parallel with finishing the core ROADMAP milestones. Honestly labeled: any driver ships as an
> M11-sandboxed, M12-signed, owner-sealed high-privilege component; the vendor firmware blob is a
> closed, unaudited binary loaded verbatim and labeled public+unaudited.
>
> COVERAGE NOTE: this pass grounded the CHIP+FIRMWARE-download protocol (register-level, from the
> Linux mwifiex_pcie source) and the raiOS PCIe/DMA transport plumbing. The 802.11 SCAN command
> protocol and a complete raiOS-capability inventory were NOT completed in this pass (2 research
> agents failed a schema cap) and need a follow-up scoping pass before the scan milestone is planned.

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
