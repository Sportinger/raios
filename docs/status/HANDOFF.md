# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~14:35, loop paused for budget)

rustc runs the FULL compile frontend on-device: reads+parses /src/hello.rs,
resolves std from the sysroot, type-checks, reaches output emission, fails
only creating /out/rmeta<rand>/lib.rmeta (os76, instance.rs:229 — temp dir fd
lacks PATH_CREATE_FILE; STATUS has the precise fix). Reached via 4 WASI
calibrations (symlink/rights/O_DIRECTORY/O_TRUNC, all committed+pushed).
Earlier today: the rustc "spin" solved (fuel starvation + grow), --version
completes exit 0; §1–3 reframe + both escape needles permanent; §4 JSON-diag
box; floor doc; unsafe inventory. Tree clean, all pushed (HEAD 1166ca6).

## Next step

§6 rustc-compile (buildable, NOT owner-blocked): fix rights_inheriting
propagation from the writable /out preopen to opened subdirs so the temp dir
carries PATH_CREATE_FILE; instrument the temp-dir fd rights first, keep ROFS
+ write denials. Then rerun wasi.rustcbuild on persist-combined-rustcbuild.img
(-GuestMemoryMB 8192 -KeepImage) → expect hello.wasm. Also queued: §3
rollback-isolation profile (rewrite mirroring m6d-rollback verbatim), §4
device-graph IRQ fields, §2 storage-negative. Owner-gated (blocking "all
boxes"): §5/§6 pre-ADR-0005 wording reframe; bare-metal escape run (Surface);
unattended-loop hardware (money).

## Recently (exactly 3, newest first)

### 2026-07-19 — rustc compiles real source on-device up to output write
wasi.rustcbuild + 4 WASI file calibrations walked rustc from "can't read
source" to running the whole frontend (std resolved, type-check done),
blocked only at artifact create (os76). Precise next lane in STATUS. §6 open.

### 2026-07-19 — rustc --version completes inside raiOS
After the escrow top-up fix, stderr capture revealed 'LLVM ERROR: out of
memory': prepare_rustcrun pre-grew the guest to max, so its first allocator
grow was denied. Guest now keeps 399 initial pages; grows 399→401 approved;
`RAIOS_RUSTCSTDOUT text=rustc 1.83.0-dev.`, exit 0 (4716732).

### 2026-07-19 — Starvation fix verified: rustc executes for real
E3 top-up fix + conformance starvation test; decisive rerun: 4 rounds to
real stderr I/O + trap vs 200k dead rounds before. The on-device compiler
now runs and fails ordinarily; stderr capture (E4) is the next evidence.
