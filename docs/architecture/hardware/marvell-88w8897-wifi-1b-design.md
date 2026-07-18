# WiFi-1b Design: Marvell 88W8897 PCIe Firmware-Download Hardware Layer

Read-only scoping note: `git status --short` shows unrelated `?? release/raios-stage0-preview.img`; preserve it.

## 1. MODULE SHAPE

Add a new kernel module: `seed-kernel/src/marvell_wifi_pcie.rs`.

Do not grow `seed-kernel/src/wifi.rs` into the driver. `wifi.rs` is currently probe/config/status only: target IDs at `seed-kernel/src/wifi.rs:8-10`, probe at `seed-kernel/src/wifi.rs:112-158`, SSID/passphrase storage at `seed-kernel/src/wifi.rs:164-201`, BAR0-only helper at `seed-kernel/src/wifi.rs:210-227`. Keep it as the owner-facing WiFi status/config surface.

`marvell_wifi_pcie.rs` should be the hardware engine, declared near the existing modules in `seed-kernel/src/main.rs:104-144`.

Split it into two layers:

- Thin unsafe shell: `read_reg`, `write_reg`, `dma_block_phys`, `dma_write_block`, `map_register_bar`. Mirror e1000’s volatile helpers at `seed-kernel/src/e1000.rs:409-415`, BAR/bus-master/MMIO flow at `seed-kernel/src/e1000.rs:159-176`, static DMA buffers at `seed-kernel/src/e1000.rs:122-125`, and `virt_to_phys` use at `seed-kernel/src/e1000.rs:268,276,294,303`.

- Small driver loop: call `raios_core::marvell_wifi_fw::FirmwareDownload::step(RegisterReads)` and translate `FwAction` to the shell. The pure sequencer is already exported at `raios-core/src/lib.rs:15`, is explicitly hardware-independent at `raios-core/src/marvell_wifi_fw.rs:1-6`, and exposes `step()` at `raios-core/src/marvell_wifi_fw.rs:153-177`.

Keep hardware I/O untestable and tiny. If adding software tests, add at most one pure helper in `raios-core/src/marvell_wifi_fw.rs` that converts `FwAction` plus a DMA physical address into an ordered register-write plan. No trait/interface unless a second implementation exists.

## 2. BRING-UP FLOW

First hard gate: chip present. `wifi::probe()` already calls `pci::find_device(0x11ab, 0x2b38)` and returns `Missing` when absent at `seed-kernel/src/wifi.rs:118-121`. QEMU will take this path.

Second hard gate: firmware available. For WiFi-1b first slice, this must return false, so no BAR mapping, no bus-master enable, and no MMIO/DMA happens.

When the blob gate is later true, sequence:

1. Map register BAR2, not current BAR0. The scope doc says Linux register I/O is through BAR2/config `0x18`, while `wifi.rs` only reads BAR0 today: `docs/marvell-88w8897-wifi-driver-scope.md:72`, `docs/marvell-88w8897-wifi-driver-scope.md:428`. Use `pci::read_bar_info(address, 2)`, whose BAR decoder handles 64-bit BARs at `seed-kernel/src/pci.rs:92-174`, especially `seed-kernel/src/pci.rs:144-167`.

2. Enable bus master with `pci::enable_bus_master`, which sets I/O, memory, and bus-master bits at `seed-kernel/src/pci.rs:86-90`. e1000 does this before mapping at `seed-kernel/src/e1000.rs:169-176`.

3. Map MMIO with `memory::map_mmio`, which uses the fixed MMIO VA window and no-cache/write-through flags at `seed-kernel/src/memory.rs:111-139` and `seed-kernel/src/memory.rs:159-162`.

4. Initialize `FirmwareDownload::new(firmware)` from `raios-core/src/marvell_wifi_fw.rs:130-139`.

5. Each iteration reads:
   - `CMD_SIZE = 0xC40` from `raios-core/src/marvell_wifi_fw.rs:12-13`
   - `FW_STATUS = 0xC44` from `raios-core/src/marvell_wifi_fw.rs:14-15`
   - `PCIE_CPU_INT_STATUS = 0xC1C` from `raios-core/src/marvell_wifi_fw.rs:26-27`

   Feed those into `RegisterReads`, defined at `raios-core/src/marvell_wifi_fw.rs:37-42`.

6. Map `FwAction`:
   - `WriteBlock { image_offset, len }` (`raios-core/src/marvell_wifi_fw.rs:80-84`): copy that firmware slice into a DMA bounce buffer, zero any unused tail, write `CMD_ADDR_LO = 0xC10`, `CMD_ADDR_HI = 0xC14`, and `CMD_SIZE = 0xC40` from `raios-core/src/marvell_wifi_fw.rs:8-13`.
   - `RingDoorbell` (`raios-core/src/marvell_wifi_fw.rs:85`): write `CPU_INTR_DOOR_BELL` to `PCIE_CPU_INT_EVENT = 0xC18`, constants at `raios-core/src/marvell_wifi_fw.rs:24-29`; then poll `PCIE_CPU_INT_STATUS` until the bit clears before the next `step()`.
   - `WriteDrvReady { value }` (`raios-core/src/marvell_wifi_fw.rs:86-88`): write `DRV_READY = 0xCF0`, constant at `raios-core/src/marvell_wifi_fw.rs:20-21`; value should be `FIRMWARE_READY_PCIE = 0xfedcba00` from `raios-core/src/marvell_wifi_fw.rs:30-31`.
   - `PollFwStatus` (`raios-core/src/marvell_wifi_fw.rs:89-91`): read again after a bounded delay/timeout.
   - `Retry { image_offset }` (`raios-core/src/marvell_wifi_fw.rs:92-94`): no DMA write yet; continue polling at the same offset. Retry cap is in WiFi-1a at `raios-core/src/marvell_wifi_fw.rs:34-35` and enforced at `raios-core/src/marvell_wifi_fw.rs:234-242`.
   - `Done` / `Fail` (`raios-core/src/marvell_wifi_fw.rs:95-107`): record honest status; no link claim.

The register-level protocol is already summarized in the scope doc at `docs/marvell-88w8897-wifi-driver-scope.md:89`.

Do not add HostCmd scan, event rings, `WifiPhy`, association, WPA, TxPD/RxPD, or smoltcp integration in WiFi-1b. Those are explicitly later work at `docs/marvell-88w8897-wifi-driver-scope.md:92-98` and missing at `docs/marvell-88w8897-wifi-driver-scope.md:474-481`.

## 3. THE FIRMWARE BLOB PROBLEM

raiOS has no filesystem loader, and the blob is closed firmware. The scope doc records `pcie8897_uapsta.bin` as 803,884 bytes, about 785 KB, version 15.68.19.p21 at `docs/marvell-88w8897-wifi-driver-scope.md:418`.

Options:

- `include_bytes!`: simplest for physical bring-up, but commits/bundles a large closed unaudited blob. Existing pattern is only build-generated Wasm artifact embedding, cited at `docs/marvell-88w8897-wifi-driver-scope.md:467`.

- M12+ signed candidate/artifact path: best final architecture for provenance/classification, cited at `docs/marvell-88w8897-wifi-driver-scope.md:468`.

- Known disk region: not first choice; persistence and broad durable artifact intake remain gated by roadmap/status, and README still says external unsigned artifact intake and executable candidate-byte mapping remain denied at `README.md:291-294`.

Recommendation for WiFi-1b first inert slice: no blob. Implement a runtime `firmware_blob() -> Option<&'static [u8]>` that returns `None`, so the full hardware code compiles but cannot run. Report `firmware_blob_unavailable` and `hardware_untested`.

For later hardware bring-up, either temporary `include_bytes!` with explicit `public + unaudited closed firmware` labeling, or the M12 signed channel. The scope doc requires that label and a denial for independent integrity verification at `docs/marvell-88w8897-wifi-driver-scope.md:470`.

## 4. DMA REQUIREMENTS

WiFi-1b first slice needs one physically-addressable firmware block bounce buffer. WiFi-1a caps firmware block size at 256 bytes with `FW_BLOCK_SIZE` at `raios-core/src/marvell_wifi_fw.rs:32-33`.

Use the e1000 pattern first: aligned static buffer plus `memory::virt_to_phys`. e1000’s static DMA storage is at `seed-kernel/src/e1000.rs:122-125`; descriptors/buffers are programmed via `memory::virt_to_phys` at `seed-kernel/src/e1000.rs:268,276,294,303`. The translation function is `seed-kernel/src/memory.rs:92-105`.

The allocator gap: there is no typed DMA allocator returning `(virt, phys)`. The scope doc calls that out at `docs/marvell-88w8897-wifi-driver-scope.md:436-443` and lists it as must-build at `docs/marvell-88w8897-wifi-driver-scope.md:476`. For WiFi-1b, static bounce buffer is enough. Later TX/RX/event rings and PFU descriptors need either a typed DMA handle or more static buffers; those later requirements are listed at `docs/marvell-88w8897-wifi-driver-scope.md:67-71`.

Hardware-only unknown: whether Surface silicon accepts these physical addresses for DMA, and there is no IOMMU protection; see `docs/marvell-88w8897-wifi-driver-scope.md:443`.

## 5. INERT-GATING + HONESTY

Keep WiFi-1b inert by requiring both:

- chip present: `wifi::probe()` returns missing in QEMU at `seed-kernel/src/wifi.rs:118-121`
- firmware available: first slice returns `None`, so no MMIO/DMA/bus-master path runs

Do not hide the module behind `#[cfg]`; compile the code. Runtime gate it.

Status should stay honest:

- Current UI says detected WiFi is still `FW TODO` at `seed-kernel/src/system_status.rs:303-321`.
- Console says `FIRMWARE/SCAN TODO` at `seed-kernel/src/console.rs:1714-1724`.
- Agent problems say firmware upload and WPA are not implemented at `seed-kernel/src/agent_protocol_system.rs:1241-1248`.

WiFi-1b should update those strings only as far as truth allows: `firmware uploader compiled, blob unavailable, hardware-untested`. Never report firmware ready, scan, link, or network capability without physical Surface evidence.

## 6. SOFTWARE-VERIFIABLE vs HARDWARE-ONLY

Software-verifiable:

- `cargo test --locked -p raios-core`, preserving WiFi-1a tests at `raios-core/src/marvell_wifi_fw.rs:250-434`.
- Kernel release build via documented command at `docs/agents/DEBUGGING.md:8-12`.
- `cargo fmt --all -- --check`, documented at `docs/agents/DEBUGGING.md:2076`.
- Pure action-to-register-write helper tests, only if the worker adds a tiny helper in `raios-core/src/marvell_wifi_fw.rs`. Test `WriteBlock`, `RingDoorbell`, and `WriteDrvReady`; do not fake MMIO.

Hardware-only:

- BAR2 is really the register BAR on Surface silicon. The doc marks this unknown at `docs/marvell-88w8897-wifi-driver-scope.md:428`.
- DMA addresses are accepted by the 88W8897.
- Doorbell clear timing and firmware block requests behave as Linux expects.
- `FW_STATUS` reaches `FIRMWARE_READY_PCIE`.
- Poll-only operation works without MSI. This is explicitly unknown at `docs/marvell-88w8897-wifi-driver-scope.md:44` and `docs/marvell-88w8897-wifi-driver-scope.md:450`.

No QEMU result can prove the chip path; QEMU only proves inert behavior and build structure.

## 7. CONCRETE WiFi-1b WORKER PACKET

Capability sentence: raiOS can compile a gated Marvell 88W8897 PCIe firmware-download hardware shell that would drive the WiFi-1a sequencer on real hardware, while granting no WiFi capability and performing no MMIO/DMA unless both the chip and firmware blob are present.

Allowed write set:

- `seed-kernel/src/marvell_wifi_pcie.rs` new
- `seed-kernel/src/main.rs` module declaration only
- `seed-kernel/src/wifi.rs` minimal hook/status plumbing only
- optionally `seed-kernel/src/system_status.rs` and `seed-kernel/src/console.rs` for honest status text
- optionally `raios-core/src/marvell_wifi_fw.rs` only for a tiny pure action-plan helper/test

Ordered changes:

1. Add `marvell_wifi_pcie.rs`.
2. Define an inert `firmware_blob() -> Option<&'static [u8]>` returning `None`.
3. Implement `try_start(address: PciAddress) -> status`; first branch returns `firmware_blob_unavailable_hardware_untested`.
4. After that gate, implement but do not execute in QEMU: BAR2 decode, memory check, bus-master enable, MMIO map.
5. Add static aligned 256-byte firmware bounce buffer and physical-address lookup.
6. Add unsafe shell methods: `read_reg`, `write_reg`, `read_fw_registers`, `dma_write_block`.
7. Add the firmware loop matching on WiFi-1a `FwAction`.
8. Hook `wifi.rs` after successful chip detection so QEMU still reports Missing and Surface-without-blob reports gated/unavailable.
9. Update status text only enough to say compiled/gated/hardware-untested.
10. Do not add scan, HostCmd, rings, interrupts, WPA, `WifiPhy`, persistence, blob commit, or external artifact loading.

Forbidden:

- No firmware blob in repo.
- No `include_bytes!` yet.
- No fake success path.
- No fallback that marks WiFi ready.
- No MSI/APIC work.
- No 802.11 stack.
- No durable writes or artifact intake changes.

Host-only DoD:

- `cargo fmt --all -- --check`
- `cargo test --locked -p raios-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release`
- `scripts\scan-secrets.ps1`
- Confirm QEMU/chip-absent path remains inert by code inspection; do not claim runtime WiFi verification.