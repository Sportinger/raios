#!/usr/bin/env python3
r"""Map WebAssembly function indices to name-section names or code offsets.

Pinned rustc examples (PowerShell, from the repository root):

    python scripts/map-wasm-function.py 8657
    python scripts/map-wasm-function.py 8657 114040 113803
    python scripts/map-wasm-function.py --artifact C:\path\to\module.wasm 31

The default artifact is the owner-pinned rustc_opt.wasm. If its name section
is stripped, `function_offset` is the function record and `code_offset` is the
first instruction after local declarations; feed that range to a DWARF-aware
or symbolized disassembler.
"""

from __future__ import annotations

import argparse
import hashlib
import mmap
from dataclasses import dataclass
from pathlib import Path
from typing import BinaryIO


PINNED_ARTIFACT = Path(r"C:\Users\admin\raios-artifacts\rustc-wasm\rustc_opt.wasm")
PINNED_SHA256 = "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791"


class WasmFormatError(ValueError):
    """Raised when the input is not a supported core WebAssembly module."""


def read_uleb(data: mmap.mmap, offset: int, limit: int) -> tuple[int, int]:
    value = 0
    shift = 0
    while offset < limit and shift < 70:
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte & 0x80 == 0:
            return value, offset
        shift += 7
    raise WasmFormatError(f"invalid unsigned LEB128 at 0x{offset:x}")


def read_name(data: mmap.mmap, offset: int, limit: int) -> tuple[str, int]:
    length, offset = read_uleb(data, offset, limit)
    end = offset + length
    if end > limit:
        raise WasmFormatError("name extends past its containing section")
    return bytes(data[offset:end]).decode("utf-8", errors="replace"), end


def skip_limits(data: mmap.mmap, offset: int, limit: int) -> int:
    flags, offset = read_uleb(data, offset, limit)
    _, offset = read_uleb(data, offset, limit)
    if flags & 0x01:
        _, offset = read_uleb(data, offset, limit)
    return offset


def skip_ref_type(data: mmap.mmap, offset: int, limit: int) -> int:
    if offset >= limit:
        raise WasmFormatError("missing reference type")
    form = data[offset]
    offset += 1
    if form in (0x63, 0x64):  # ref null / ref followed by a heap type
        _, offset = read_uleb(data, offset, limit)
    return offset


@dataclass(frozen=True)
class FunctionBody:
    function_offset: int
    body_offset: int
    code_offset: int
    end_offset: int


@dataclass
class ModuleIndex:
    imported_names: list[str]
    bodies: list[FunctionBody]
    function_names: dict[int, str]
    exported_names: dict[int, str]
    has_name_section: bool


def parse_imports(
    data: mmap.mmap, start: int, end: int
) -> list[str]:
    count, offset = read_uleb(data, start, end)
    functions: list[str] = []
    for _ in range(count):
        module, offset = read_name(data, offset, end)
        field, offset = read_name(data, offset, end)
        if offset >= end:
            raise WasmFormatError("missing import descriptor")
        kind = data[offset]
        offset += 1
        if kind == 0:  # function
            _, offset = read_uleb(data, offset, end)
            functions.append(f"{module}.{field}")
        elif kind == 1:  # table
            offset = skip_ref_type(data, offset, end)
            offset = skip_limits(data, offset, end)
        elif kind == 2:  # memory
            offset = skip_limits(data, offset, end)
        elif kind == 3:  # global
            offset = skip_ref_type(data, offset, end)
            offset += 1  # mutability
        elif kind == 4:  # tag
            _, offset = read_uleb(data, offset, end)
            _, offset = read_uleb(data, offset, end)
        else:
            raise WasmFormatError(f"unknown import kind {kind}")
        if offset > end:
            raise WasmFormatError("import descriptor extends past section")
    return functions


def parse_code(data: mmap.mmap, start: int, end: int) -> list[FunctionBody]:
    count, offset = read_uleb(data, start, end)
    bodies: list[FunctionBody] = []
    for _ in range(count):
        function_offset = offset
        body_size, body_offset = read_uleb(data, offset, end)
        body_end = body_offset + body_size
        if body_end > end:
            raise WasmFormatError("function body extends past code section")
        local_groups, code_offset = read_uleb(data, body_offset, body_end)
        for _ in range(local_groups):
            _, code_offset = read_uleb(data, code_offset, body_end)
            code_offset = skip_ref_type(data, code_offset, body_end)
        bodies.append(FunctionBody(function_offset, body_offset, code_offset, body_end))
        offset = body_end
    return bodies


def parse_exports(data: mmap.mmap, start: int, end: int) -> dict[int, str]:
    count, offset = read_uleb(data, start, end)
    functions: dict[int, str] = {}
    for _ in range(count):
        name, offset = read_name(data, offset, end)
        if offset >= end:
            raise WasmFormatError("missing export descriptor")
        kind = data[offset]
        offset += 1
        index, offset = read_uleb(data, offset, end)
        if kind == 0:
            functions[index] = name
    return functions


def parse_function_names(
    data: mmap.mmap, start: int, end: int
) -> dict[int, str]:
    names: dict[int, str] = {}
    offset = start
    while offset < end:
        subsection = data[offset]
        offset += 1
        size, payload = read_uleb(data, offset, end)
        payload_end = payload + size
        if payload_end > end:
            raise WasmFormatError("name subsection extends past custom section")
        if subsection == 1:
            count, cursor = read_uleb(data, payload, payload_end)
            for _ in range(count):
                function_index, cursor = read_uleb(data, cursor, payload_end)
                name, cursor = read_name(data, cursor, payload_end)
                names[function_index] = name
        offset = payload_end
    return names


def parse_module(data: mmap.mmap) -> ModuleIndex:
    if len(data) < 8 or data[:8] != b"\x00asm\x01\x00\x00\x00":
        raise WasmFormatError("not a WebAssembly core module version 1")
    imported_names: list[str] = []
    bodies: list[FunctionBody] = []
    function_names: dict[int, str] = {}
    exported_names: dict[int, str] = {}
    has_name_section = False
    offset = 8
    while offset < len(data):
        section_id = data[offset]
        offset += 1
        size, start = read_uleb(data, offset, len(data))
        end = start + size
        if end > len(data):
            raise WasmFormatError("section extends past end of module")
        if section_id == 0:
            custom_name, payload = read_name(data, start, end)
            if custom_name == "name":
                has_name_section = True
                function_names.update(parse_function_names(data, payload, end))
        elif section_id == 2:
            imported_names = parse_imports(data, start, end)
        elif section_id == 7:
            exported_names = parse_exports(data, start, end)
        elif section_id == 10:
            bodies = parse_code(data, start, end)
        offset = end
    return ModuleIndex(
        imported_names,
        bodies,
        function_names,
        exported_names,
        has_name_section,
    )


def sha256(stream: BinaryIO) -> str:
    digest = hashlib.sha256()
    while chunk := stream.read(1024 * 1024):
        digest.update(chunk)
    stream.seek(0)
    return digest.hexdigest()


def parse_index(token: str) -> int:
    # Accept both a bare RAIOS_RUSTCPC index and an `idx:count` entry.
    value = token.rstrip(",").split(":", 1)[0]
    return int(value, 0)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("indices", nargs="+", type=parse_index, help="function index/indices")
    parser.add_argument(
        "--artifact",
        type=Path,
        default=PINNED_ARTIFACT,
        help=f"Wasm module (default: {PINNED_ARTIFACT})",
    )
    args = parser.parse_args()

    artifact = args.artifact.resolve()
    with artifact.open("rb") as stream:
        artifact_sha = sha256(stream)
        with mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as data:
            module = parse_module(data)

    print(f"artifact={artifact}")
    print(f"sha256={artifact_sha}")
    print(f"pinned_sha_match={str(artifact_sha == PINNED_SHA256).lower()}")
    print(f"name_section={str(module.has_name_section).lower()}")
    print(f"imported_functions={len(module.imported_names)}")
    print(f"defined_functions={len(module.bodies)}")
    total_functions = len(module.imported_names) + len(module.bodies)
    for function_index in args.indices:
        print(f"function_index={function_index}")
        if function_index < 0 or function_index >= total_functions:
            print(f"  error=out_of_range total_functions={total_functions}")
            continue
        name = module.function_names.get(function_index)
        exported_name = module.exported_names.get(function_index)
        if function_index < len(module.imported_names):
            print("  kind=imported")
            print(f"  name={name or exported_name or module.imported_names[function_index]}")
            continue
        body = module.bodies[function_index - len(module.imported_names)]
        print("  kind=defined")
        print(f"  name={name or exported_name or 'none'}")
        if name is not None:
            print("  name_source=name_section")
        elif exported_name is not None:
            print("  name_source=export")
        print(f"  function_offset=0x{body.function_offset:x}")
        print(f"  body_offset=0x{body.body_offset:x}")
        print(f"  code_offset=0x{body.code_offset:x}")
        print(f"  end_offset=0x{body.end_offset:x}")
        if name is None and exported_name is None:
            print("  note=name_section_missing_or_unnamed; symbolize code_offset with DWARF-aware disassembly")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
