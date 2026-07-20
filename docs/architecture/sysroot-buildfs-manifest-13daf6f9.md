# Sysroot BuildFS pin — manifest 13daf6f9

Date: 2026-07-18 · Produced by the orchestrator on the workstation (E: was
detached; artifacts re-downloaded and verified instead — the pinned SHAs
made that a safe substitution).

## Inputs (pinned)

| Part | Value |
|---|---|
| Compiler | oligamiq/rust_wasm v0.3.0-release `rustc_opt.wasm.tar.gz` → `rustc_opt.wasm` (95,427,808 B) |
| Compiler sha256 | `c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791` — **byte-identical** to the probe artifact and the import-inventory pin |
| Sysroot tar | `wasm32-wasip1-threads.tar.gz` (same release), 27 rlibs + self-contained |
| Emulation libs | `libwasi-emulated-{mman,signal,process-clocks,getpid}.a` from WebAssembly/wasi-sdk **wasi-sdk-33** `wasi-sysroot-33.0+m.tar.gz` (non-LTO variants under `lib/wasm32-wasip1-threads/`) |
| Layout | probe recipe: tar content → `sysroot/lib/rustlib/wasm32-wasip1-threads/lib/` (+`self-contained/`), the four libs copied into `self-contained/` |

## Output (buildfs-pack, tool at c20f8ad)

| Field | Value |
|---|---|
| Manifest sha256 | `13daf6f9042d07c4d698d60ea16869ed85e2035f762f4b5a048e71e7523b7b15` |
| Content | 72 MiB, 1163 files under `chunks/` + `manifest.bin` + `manifest.sha256` |
| Determinism | two independent pack runs over the same tree → identical manifest sha256 |

## Location

`C:\Users\admin\raios-artifacts\rustc-wasm\` (workstation, outside the
repo): source tars, verified `rustc_opt.wasm`, arranged `sysroot/`, packed
`buildfs-run1/` (+ `buildfs-run2/` as the determinism witness — safe to
delete). This manifest hash is the value the sysroot mount grant of a real
rustc build job must pin.
