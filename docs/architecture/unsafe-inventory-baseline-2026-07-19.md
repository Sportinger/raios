# Unsafe inventory baseline — 2026-07-19

## Command

Run from the repository root with Python 3.14:

```text
python scripts/unsafe-inventory.py --summary
```

## Counts

| Crate | Total | Tagged | Untagged |
|---|---:|---:|---:|
| `buildfs-pack` | 0 | 0 | 0 |
| `core-policy-sign` | 0 | 0 | 0 |
| `descriptor-resign` | 0 | 0 | 0 |
| `fake-cloud-server` | 0 | 0 | 0 |
| `ota-tools` | 0 | 0 | 0 |
| `raios-core` | 0 | 0 | 0 |
| `raios-dns-parse` | 0 | 0 | 0 |
| `raios-http-parse` | 0 | 0 | 0 |
| `raios-lang` | 0 | 0 | 0 |
| `raios-w7-acquire-logic` | 0 | 0 | 0 |
| `raios-wasi-preview1` | 0 | 0 | 0 |
| `raios-wasm-ir` | 0 | 0 | 0 |
| `raios-wasmi-conformance` | 0 | 0 | 0 |
| `raios-x509-spki` | 0 | 0 | 0 |
| `raios-x509-time` | 0 | 0 | 0 |
| `registry-core` | 0 | 0 | 0 |
| `registry-tools` | 0 | 0 | 0 |
| `seed-kernel` | 346 | 4 | 342 |
| `svc-build-assembler` | 4 | 0 | 4 |
| `svc-demo-bufecho` | 2 | 0 | 2 |
| `svc-demo-certspki` | 2 | 0 | 2 |
| `svc-demo-certwindow` | 2 | 0 | 2 |
| `svc-demo-dnsparse` | 3 | 0 | 3 |
| `svc-demo-echo` | 2 | 0 | 2 |
| `svc-demo-httphead` | 2 | 0 | 2 |
| `svc-net-acquire-w7` | 17 | 0 | 17 |
| `svc-personal-shell-proof` | 9 | 0 | 9 |
| `wasm-import-inventory` | 0 | 0 | 0 |
| **Total** | **389** | **4** | **385** |

`Tagged` means that a comment contains the exact marker `SAFETY` on the site
line or one of the three preceding physical lines.

## Hand-verified samples

The following JSON entries were compared with the named source files and line
numbers:

1. `seed-kernel/src/memory.rs:150` — `kind: block`, source
   `let cached_virt_start = unsafe {`, `safety_comment: true`. Line 149 is
   `// SAFETY: every MMIO_CACHE access is serialized by PAGE_TABLE_LOCK.`
2. `seed-kernel/src/ahci.rs:372` — `kind: fn`, source
   `pub(crate) unsafe fn identify(self) -> Result<AhciBlockDeviceIdentity, &'static str> {`,
   `safety_comment: false`. Lines 369–371 contain a `# Safety` doc section but
   not the exact uppercase marker `SAFETY`.
3. `seed-kernel/src/e1000.rs:151` — `kind: impl`, source
   `unsafe impl Send for E1000 {}`, `safety_comment: false`. Lines 148–151
   contain no comment with the marker `SAFETY`.

## Scope and limits

- Workspace crates come from the root `Cargo.toml` `[workspace].members` list.
  All `.rs` files below those member roots are scanned; `.git`, `target`, and
  `vendor` directories are excluded.
- The scanner masks line comments, nested block comments, normal strings, raw
  strings, byte strings, C strings, and character literals before matching
  unsafe syntax.
- This is a lexical source inventory. It does not expand macros or evaluate
  `cfg` conditions. Unsafe syntax written in a macro definition is inventoried
  as source; unsafe syntax produced only by macro expansion is not visible.
- A `SAFETY` marker is a proximity tag only. The scanner does not assess the
  comment's justification. In a multiline comment, proximity is measured from
  the physical line containing `SAFETY`.
