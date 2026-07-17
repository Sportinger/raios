# B3 implementation notes

## B3A-1a assembler crate

Capability: host and future signed-guest code can turn one bounded,
inspectable `RAIOS_WASM_IR_V1` source into deterministic final WebAssembly
bytes without runtime dependencies.

### Exact v1 grammar

The source is ASCII, uses LF (`0a`) line endings, ends with LF, contains no
blank lines or comments, and is at most `MAX_IR_SOURCE_BYTES = 4096` bytes.
Spaces shown below are exactly one ASCII space.

```text
RAIOS_WASM_IR_V1
func <name> -> i32
const.i32 <i32>
return
end
[memory <min-pages> <max-pages>]
[data <offset> <hex-bytes>]*
```

- `<name>` is 1-64 bytes. It starts with `a-z`; following bytes are `a-z`,
  `0-9`, `_`, `.`, or `-`. The sole function is exported under this exact
  name.
- `<i32>`, page counts, and offsets use canonical decimal: `0`, an unsigned
  value beginning `1-9`, or (for `<i32>` only) `-` followed by `1-9` and
  digits. `+`, leading zeroes, `-0`, and out-of-range values are rejected.
- The body is exactly one `i32.const`, `return`, and the function `end`. There
  are no parameters, locals, imports, calls, tables, or other control flow.
- `memory` is optional, unique, and must precede all `data` lines. It declares
  min and max limits with `min <= max <= MAX_MEMORY_PAGES = 4`.
- `data` requires memory. Each line is one active memory-0 segment. Offset is
  bounded by the declared minimum memory, hex is non-empty, lowercase,
  even-length, and has no prefix or separators. At most
  `MAX_DATA_SEGMENTS = 8` segments and `MAX_DATA_BYTES = 2000` decoded bytes
  are accepted.
- Any line after the optional memory/data suffix is trailing content. V1 has
  no separate export directive: the one function export is mandatory.

### Canonical WebAssembly emission

`assemble` returns a fixed-capacity `WasmModuleBytes`; it does not allocate.
The encoded module is capped at `MAX_WASM_OUTPUT_BYTES = 4096` bytes and emits
only, in order: magic/version, type, function, optional memory, export, code,
and optional data sections. There are no custom/name sections.

All indices are zero, every vector/section/body length uses shortest unsigned
LEB128, and `i32.const` plus data offsets use shortest signed LEB128. Memory
uses explicit min/max limits. The code body has an empty locals vector and one
exact `end` opcode. Data uses the active memory-0 segment form. Identical input
therefore has one byte-for-byte output independent of clock, randomness,
allocation state, or iteration order.

Three literal byte goldens independently pin the encoder, including the
`raios_service_main` return-42 module. Dev-only wasmi 0.31.2 tests validate and
execute that module; wasmi is not a runtime dependency. Boundary tests pin
signed LEB128 at 63/64 and 8191/8192 and unsigned section sizes at 127/128.

### Exact failure reasons

- `oversize_source`: input exceeds 4096 bytes (checked first).
- `non_ascii_source`: any input byte has its high bit set.
- `wrong_version_line`: the first line is not exactly `RAIOS_WASM_IR_V1`.
- `missing_terminator`: final LF or a required body/end line is missing.
- `duplicate_section`: a second version, function, or memory declaration.
- `unknown_directive`: a required declaration/instruction has unknown shape.
- `invalid_function_name`: the function name violates its alphabet or cap.
- `non_canonical_integer_text`: decimal spelling or range is not canonical.
- `non_canonical_hex_text`: data hex is empty, odd, uppercase, separated, or
  contains a non-hex byte.
- `quota_exceeded`: memory geometry, segment count, or data-to-memory bounds
  exceed the v1 limits.
- `oversize_output`: decoded data exceeds 2000 bytes, size arithmetic
  overflows, or encoded output would exceed 4096 bytes.
- `trailing_content`: content remains after the permitted suffix or a memory
  declaration appears after data.

Reason precedence follows the checks above where conditions overlap; most
notably source size wins before content inspection.

### Packet boundary and deviations

This packet adds only the pure `raios-wasm-ir` crate, its host tests, the root
workspace-member entry, and these notes. It does not change the kernel, guest
wrapper, other crates, or VM harness. The packet's suggested allocator-backed
output was not needed: the existing relocation-crate style and 4 KiB ABI are
served by one fixed-capacity buffer. The selected concrete small quotas are
four memory pages, eight data segments, and 2000 decoded data bytes; there are
no other deviations.
