# Current Status

Current extracts retained during the 2026-07-18 docs split. The complete, unchanged histories are in _archive/2026-07-18_ROADMAP.md and _archive/2026-07-18_PROJECT_STATUS_history.md.

## Current Capability Cursor

**ON-DEVICE FACTORY LAYER — WASI BUILD WORLD + BAUPLATZ + SYSROOT IMPORT LIVE
(2026-07-19).** The kernel now hosts a capability-gated Wasm build world end to
end. Slice-6 glue: an opaque `AuthorizedBuildJob` → exact-30 WASI import linker
(no fallback) → checked guest memory → fuel-bounded runner; storage per ADR 0020
= a two-stage authority (core `BuildStorageAuthority` binds manifest hashes to
the ticket; kernel materializes a per-job chunk table with per-read sha256
rehash) + a pre-I/O commit gate + single-use output write handle. T2 threads
closed (deterministic pump, wasi thread-spawn/proc_exit/cap-48, futex deadlock →
deterministic JobDeadlocked). Bauplatz: the full 1-GiB shared-memory window
(399/16384 pages) instantiates and grows on both RAM profiles — `pages_max=16384`
live at 8 GiB — enabled by ADR 0021 park-before-charge bulk fuel (vendored wasmi;
41/41 conformance incl. pacing invariance) + a Vec-doubling-aware grow limiter +
4-GiB heap cap; an over-class memory shape is denied pre-instantiation. Sysroot
import: the real 71-MB rustc sysroot (pin `13daf6f9`, 1161 CAS chunks) is seeded
offline into ARTSTOR and read live through the granted rehashed reader — a
single-pass sha256 index (1163 reads, not the old O(n²) ~1.35M) resolves it.
Evidence: QEMU quick 501–502/502 with permanent needles RAIOS_WASI / RAIOS_THREADS
/ RAIOS_WASIMEM; seeded sysimport `RAIOS_SYSIMPORT selftest=pass manifest=ok
chunks=32` (shadow-20260719-030038). Commits 4e17c10 (gate), 9028e61 (core
authority), 30fb378 (kernel storage), 3f2a64a (deadlock), 98b2955 (fuel),
15331a3 (limiter), 917174b (index). ADRs 0018–0021.
**The real 91-MB rustc compiler now LOADS and INSTANTIATES in-kernel from the
store** (27fa7f6): wasi.compilerload reads the pinned compiler BuildFS
(1b9214df, 1457 chunks) through the granted reader, reassembles all 95_427_808
bytes verified to sha c6dccf3e, `Module::new` parses the real rustc-wasm, its
30 imports match the gate, it authorizes, and the linker instantiates it with
the 399/16384 shared memory — `RAIOS_COMPILERLOAD stage=instantiated
file_sha=ok imports=30 mem_pages=399` on the 8-GiB profile (shadow-20260719-
040901, 502/502). Enabled by the idempotent-MMIO fix (86fe9b9): map_mmio
cached identical device mappings, ending a VA leak that failed multi-thousand-
read reassembly. Honest boundary: running the module's start section traps
even with a full fuel budget (needs real threads + mounted files) — that is
the execution milestone, not load.
Repro (seeded, needs external images): pack with `buildfs-pack`, seed
`make-gpt-persist-image.py --seed-buildfs <dir> --expect-manifest-sha256 <pin>
out.img`, then `shadow-vm-smoke.ps1 -Profile quick -PersistDiskPath out.img
-GuestMemoryMB 8192 -TimeoutSeconds 600` with the `wasi.sysimport` (sysroot
pin 13daf6f9) or `wasi.compilerload` (compiler pin 1b9214df) needle. **The WASI world and the deterministic thread pump are now MARRIED on one
store** (ADR 0022, 6e3886a): WasiHostState gains ThreadHostMode
{Deny|Scheduled}; the merged pump (wasi_thread_pump.rs) queues thread-spawn
and materializes workers at pump points through the same exact-30 linker +
shared instance/reader; per-thread fuel escrow uses a vendored raw-remaining
swap (19fde8e, no clock inflation, no cross-TID funding); shared memory is
reserved at class max to purify the limiter; a WASI-effect digest pins
interleaved determinism. Proven: a multi-thread WASI fixture spawns a worker
that fd_writes on the shared instance, double-run trace+effect digests equal —
`RAIOS_WASITHREAD selftest=pass spawns=1 trace_det=1 effect_det=1` (8192,
shadow-20260719-051453), all frozen selftests still green. The combined
sysroot+compiler image is built + kernel-verified. Next: run the real rustc
through this pump with the mounted sysroot (args rustc --version), then
hello.rs double-build through the commit gate = the W5 factory proof.
Open follow-ups (flagged, not blocking): map_mmio has no unmap path (VA leak,
harmless at ~1163 mappings, matters when a full rustc run reads far more chunks);
offline-seeded ARTSTOR frames are reclog-less and read as reserved to the store
scan; the T2 multi-thread pump must enforce the ADR-0021 bank-monotonicity
invariant when it later pumps >1 bulk-heavy thread.

**B1.3 RUIP PROGRAM PERSISTENCE VERIFIED-CLOSED — B1 BLOCK CLOSED (2026-07-17).**
An owner-approved RUIP program (the editor) installs durably through the SAME W6
machinery (ARTSTOR-backed) and survives reboot: boot-2 autoload re-verifies the
W6 signature + canonical bytes and restores an INERT Source::Durable workspace
entry before any command (shell not started; a fresh click then runs it),
rollback tombstone survives boot 3, negatives fail closed; the signed guest is
unchanged. `genesis-ui` shadow-20260715-145046-7640.json (282/282 same-boot),
m6c-promotion 188/188 regression (granted bytes byte-identical), persistence-
reboot -ProgramPersistence shadow-20260717-114259-19696.json (60/60, three
boots). Honest scope: persists the program DEFINITION, not typed text. Parallel
design lane landed too: Genesis approval labels [INSTALL]/[PERSIST]/[RUN]/
[RUN+PERSIST] + a program-persistence truth line (durable survives reboot vs
current-boot), and present_rect partial-present (no full-FHD copy on editor
keystrokes) — serial output + click geometry byte-identical.
**B2 ACTIVE — B2.1a + B2.2a + B2.1b LIVE + B2.2 LIVE VERIFIED (2026-07-17):** a
fixed key-free agent answer becomes inert source files, one system-owned preflight
failure drives an exact immutable child that re-passes without build/run/install/
export authority (`project-workspace` `shadow-20260717-142445-27836.json`,
654/654); a REAL OpenAI answer over positive `pinned_cert` TLS
(`development_tls_bypass=false`) became two inert source files committed
`answer_origin=live`/`local_only`/`untrusted_agent_candidate` with zero
executable effect (`openai-direct-smoke.ps1 -ExpectProjectWorkspaceAnswer`, 6/6,
exit 0); AND the SCOPED FEEDBACK EXPORT is live-proven — a classified four-field
packet (check id + revision/tree sha256 + `build_cargo_lock_missing`, all public)
left to api.openai.com under a single-use positive gate + durable export audit,
`context_attached_to_provider_body:true`, no bypass, host-verified body hash
proving no source/secret leak; the provider's non-conforming answer was honestly
rejected, revision 1 intact (`-ExpectScopedFeedbackExport`, 8/8, exit 0). Serial
lanes: `project.ask` (source) + `agent project.{verify_revision,feedback_packet,
feedback_submit}`. Single use is per-authorization; the retained packet stays for
owner transparency. The live image needs the disposable C1 structured store +
valid-a BOOTCTL persist disk; the leaf cert pin is computed live and rotates.
The Genesis SOURCE status panel is LANDED and proven: `draw_source_status` shows
phase/revision/origin/verifier/feedback from `agent_build_loop::snapshot()`
(read-only, honest fallbacks, export row deliberately omitted until the provider
layer exposes it); serial output byte-identical by diff, and the focused
`genesis-ui` profile is green on the current HEAD
(`shadow-20260717-223146-26440.json`, 282/282 — approval click geometry and the
full W6 program flow unregressed).
**B3.0 SPIKE CLOSED + B3A-1 FIRST ON-DEVICE BUILD PROVEN (2026-07-17):** the
GO/NO-GO report (`docs/plans/b3-plan.md`) parks rustc-as-Wasm with measurable
reopen criteria (no Cranelift Wasm backend; 200-800 MiB artifact and 512 MiB-2
GiB working set vs the 64 MiB kernel heap / 512 MiB VM; interpreter slowdown)
and starts the staged ladder. Stage 1 is LIVE-PROVEN in-VM: `raios-wasm-ir`
(no-dep crate, 8 host tests, hand-written golden bytes, runs under wasmi
=0.31.2) assembles the bounded `RAIOS_WASM_IR_V1` into canonical Wasm, and the
signed `svc.build.assembler` guest reproduced it inside the sandbox with the
independent in-kernel recompute byte-identical (`build.assemble_probe`
`probe_outcome=passed`, sha256:37b6dae3, wasmi-valid 52-byte module, fully
inert, W5/W6 untouched).
**B3A-1c VERIFIED-CLOSED (2026-07-18):** the on-device build loop is closed in
miniature — a `main.rwir` source revision (fixture per the B2.1a discipline)
was assembled IN-SYSTEM twice deterministically (run1==run2==independent kernel
recompute==host-hardcoded golden hash, zero imports, exact entrypoint), bound
into a W5 preview, serial approve denied, and ONE physical Genesis click ran
the self-built module once returning 42 with zero install/persistence effect;
prepare-less restart and replay prepare deny, negative table and byte-identical
drift scans green (`build-assemble` `shadow-20260718-082526-6872.json`, 33/33).
Vision station 4 (build on device) is now real at assembler scale. Honest
scope: same-boot double instantiation, not yet a cross-reboot fresh-store
double build; the serial-log combine fix after the report is verified by the
next profile run.
**OWNER GOAL RE-CENTERED (2026-07-18, binding — see docs/plans/plan-personal-rust-playground.md):**
the on-device factory is the main road (agents build+test big software via
Genesis jobs, no workshop). B3A-2a (rlang crate + typed emitter, 21 host
tests green) is COMMITTED and rlang is PAUSED — spare tool + reusable
encoder, not critical path.
**WORKSHOP PROBE GREEN (2026-07-18, commit 37929ba):** the unmodified public
threaded rustc-as-Wasm artifact builds and runs under pinned wasmtime 46
(hello 1.6 s, medium `-O` 1.2 s, ~670 MB peak, real guest threads ~26-32,
rust-lld embedded so no separate linker job); full report
`docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md`. The former
"threads-free cloud fork" step is DROPPED (owner 2026-07-18: no fork, no
compiler modification); a later cloud rebake is provenance-check only.
**NEXT DELIBERATE SLICE:** bring the tool into the system — threads in the
cage (T1 atomics/shared memory in the vendored wasmi, host-testable; then T2
round-robin pump) plus Bauplatz heap and the WASI subset; slices and budgets
in `docs/plans/plan-rust-kernel.md` §7, then Bauplatz scoping with the
measured budgets.
`/program` stays the explicit RUIP fast lane. Honest gap: the scoped feedback carries no
source, so the provider rarely returns a conforming child — richer
(still-classified) evidence and the B3 build/test producer close the iterate
loop.


## Latest Detailed Evidence

# Project Status

B2.2a VERIFIED (2026-07-17): focused `project-workspace` report
`shadow-20260717-142445-27836.json` passed 654/654 predicates in 715 s; report
SHA-256 `da93b159bdf356faef2bdb80e333538dce54c9bba9dc5b60e391cfa3751c12ea`.
The first key-free proof-carrying iteration is real: system-owned source
preflight rejects revision 1 (the B2.1a fixture without `Cargo.lock`) as
`build_cargo_lock_missing`; a bounded `local_only` feedback packet contains
only check id, revision hash, tree hash, and reason; an exact child of revision
1 adds `Cargo.lock` and re-passes as `source_preflight_ok`. Revision 1 is fully
reparsed and hash-checked before the child write; a replay/wrong-parent child
is denied before storage, and the loop compiles, tests, runs, loads, installs,
promotes, and exports nothing.
This is a deterministic source-shape preflight, not a compiler or test runner;
on-device compilation remains B3 and live scoped feedback export remains a
B2.2 follow-up.

B2.2a FAILED-RUN ROOT CLASSIFICATION (2026-07-17, corrected after timeline
correlation): all four failed attempts were **host-transport/orchestration**,
not guest behavior. Reports `shadow-20260717-134940-30228.json`,
`shadow-20260717-135324-17428.json`, `shadow-20260717-135530-29560.json`, and
`shadow-20260717-135820-21776.json` formed one overlapping chain: each newer run
started before the preceding report finished; every report records TCP port
4565 plus `-StopExisting`. The newer harness therefore terminated/replaced the
older QEMU on the same fixed port. The observed signatures follow directly:
remote-host TCP close in the first run, then marker waits against replaced
guests at `w2b_stale_control:commit`, `w3_no_lock_source:commit`, and
`w3_file_hash_mismatch:chunk`. The next non-overlapping run passed all those
unchanged paths and every B2.2a predicate. Root repair: the shadow harness now
owns a named per-port mutex and rejects a concurrent same-port smoke before
packaging/QEMU startup, so `-StopExisting` cannot kill another active smoke.

END-OF-SESSION RED-GATE NOTE (2026-07-17): this session repaired the host
orchestration failure above and closed B2.2a with its mandatory focused profile.
Per the owner's aggressive-fast cadence, `full` was not repeated for this
sub-slice; the latest block-close full report remains
`shadow-20260714-234313-21680.json`, passed 2,685 predicates, while the changed
workspace/parent-readback path is covered by the newer focused 654/654 report.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/raios-stage0.img`
- Firmware vars seed: `release/ovmf_vars.fd`
- Bootloader: Limine 10 UEFI binary at `release/esp/EFI/BOOT/BOOTX64.EFI`
- Config file: `limine.conf` at ESP root and `EFI/BOOT/limine.conf`
- Kernel path inside image: `/kernel/kernel.elf`

The image boots in QEMU using the Windows PowerShell runner:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

For interactive serial commands, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

For a QEMU xHCI inventory run, add `-UsbXhciInput`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

For the bare-metal-style VM profile with USB keyboard, USB mouse, RDRAND, and
e1000 networking, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:03.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
```

Expected visible framebuffer UI:

```text
AI  CONSOLE                                      SET
RAIOS
DIRECT AI HOST
NET CONFIGURED   INPUT READY   USB READY   RNG READY
CHAT
TYPE MESSAGE AND PRESS ENTER
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
HHDM offset=0xffff800000000000
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

Console commands verified over TCP serial and USB-HID keyboard input:

```text
help
status
devices
log
provider
openai
setup
ask <text>
```

The framebuffer UI defaults to an AI chat mode. The `CONSOLE` tab keeps the
debug console visible, and the `SET` tab opens provider settings. `setup` also
opens the in-VM OpenAI/API-key menu. API-key entry is masked, held only in guest
RAM, and not printed into the console or serial output. For local-only testing,
the build scripts can also embed `OPENAI_API_KEY` into a separate non-default
image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI trust-gate smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

Direct OpenAI SPKI pinned-trust smoke is verified with a temporary image built
from a process-local fake API key and a current `OPENAI_SPKI_SHA256` pin.
Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

The legacy leaf-certificate pinned-trust smoke remains supported with
`OPENAI_CERT_SHA256`. Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard/mouse input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture/decisions/0001-raios-agent-protocol.md`.


## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\raios-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input has a current-boot US/German-ASCII picker and German QWERTZ,
  punctuation, and normal-host AltGr mapping. Unicode/umlaut input, personal-app
  AltGr forwarding, layout persistence, full modifier/dead-key handling, and
  text editing beyond Backspace remain.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard,
  mouse, hub traversal, and a limited no-input USB hotplug rescan exist, but full
  detach/reconfigure handling and broad NIC coverage do not exist yet, so real
  hardware may still boot to the UI but lack input/network unless it matches the
  implemented paths.
- Wi-Fi support currently detects the Surface Pro 4 Marvell AVASTAR 88W8897
  target and stores RAM-only SSID/WPA configuration for the current boot. The
  next implementation step is a Marvell PCIe firmware-upload path before 802.11
  association or WPA2 can work.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 has verified DNS/TCP/TLS/HTTPS for `api.openai.com:443` behind the
  explicit unverified development override, the preferred SPKI pin verifier, and
  the legacy leaf-certificate pin verifier. SPKI pinning still depends on the
  leaf using the currently supported P-256 ECDSA `CertificateVerify` path;
  broader algorithm support or WebPKI remains a hardening step.
- The OpenAI JSON response parser is intentionally minimal and only extracts the
  first `output_text` string.
- QEMU TCP serial is single-client in practice; do not run two serial clients
  against the same port at the same time.
- (Historical gap, CLOSED by M6C/M7D:) a signed module runtime exists — an external
  dev-key-signed Wasm candidate is delivered, verified, promoted, durably persisted,
  and re-verified across a real reboot through the unchanged M6 gate, honestly labeled
  `dev_key_not_owner_sealed` (owner-sealed is the final ceremony).

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/raios-stage0.img` unless the replacement
  has booted in QEMU.
