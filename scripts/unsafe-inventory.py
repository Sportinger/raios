#!/usr/bin/env python3
"""Inventory explicit unsafe sites in first-party Rust workspace crates."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


SITE_PATTERNS = (
    ("block", re.compile(r"\bunsafe\s*\{")),
    ("fn", re.compile(r"\bunsafe\s+fn\b")),
    ("impl", re.compile(r"\bunsafe\s+impl\b")),
    ("trait", re.compile(r"\bunsafe\s+(?:auto\s+)?trait\b")),
    ("extern", re.compile(r"\bunsafe\s+extern\b")),
)
EXCLUDED_DIRECTORY_NAMES = frozenset({".git", "target", "vendor"})
RAW_STRING_START = re.compile(r"(?:br|cr|r)(#{0,255})\"")


@dataclass(frozen=True)
class WorkspaceCrate:
    name: str
    root: Path
    relative_root: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inventory explicit unsafe sites in first-party Rust workspace crates."
    )
    output = parser.add_mutually_exclusive_group()
    output.add_argument("--out", type=Path, help="write stable JSON to this path")
    output.add_argument(
        "--summary",
        action="store_true",
        help="print per-crate and tagged/untagged counts instead of JSON",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def load_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def workspace_crates(root: Path) -> list[WorkspaceCrate]:
    workspace_manifest = load_toml(root / "Cargo.toml")
    workspace = workspace_manifest.get("workspace")
    if not isinstance(workspace, dict):
        raise ValueError(f"{root / 'Cargo.toml'} has no [workspace] table")

    members = workspace.get("members")
    if not isinstance(members, list) or not all(isinstance(item, str) for item in members):
        raise ValueError("workspace.members must be an array of paths")

    member_roots: set[Path] = set()
    for member_pattern in members:
        matches = sorted(root.glob(member_pattern), key=lambda path: path.as_posix())
        if not matches:
            raise ValueError(f"workspace member pattern matched nothing: {member_pattern}")
        for match in matches:
            member_root = match.resolve()
            relative_parts = member_root.relative_to(root).parts
            if EXCLUDED_DIRECTORY_NAMES.intersection(relative_parts):
                continue
            member_roots.add(member_root)

    crates: list[WorkspaceCrate] = []
    for member_root in member_roots:
        manifest_path = member_root / "Cargo.toml"
        manifest = load_toml(manifest_path)
        package = manifest.get("package")
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            raise ValueError(f"{manifest_path} has no package.name")
        crates.append(
            WorkspaceCrate(
                name=package["name"],
                root=member_root,
                relative_root=member_root.relative_to(root).as_posix(),
            )
        )

    duplicate_names = [name for name, count in Counter(c.name for c in crates).items() if count > 1]
    if duplicate_names:
        raise ValueError(f"duplicate workspace package names: {', '.join(sorted(duplicate_names))}")
    return sorted(crates, key=lambda crate: (crate.name, crate.relative_root))


def _mask_range(characters: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if characters[index] not in "\r\n":
            characters[index] = " "


def _record_safety_lines(source: str, start: int, end: int, safety_lines: set[int]) -> None:
    comment = source[start:end]
    for match in re.finditer(r"SAFETY", comment):
        safety_lines.add(source.count("\n", 0, start + match.start()) + 1)


def _raw_string_end(source: str, start: int) -> int | None:
    prefix_match = RAW_STRING_START.match(source, start)
    if prefix_match is None:
        return None
    hashes = prefix_match.group(1)
    terminator = '"' + hashes
    content_start = prefix_match.end()
    terminator_start = source.find(terminator, content_start)
    return len(source) if terminator_start < 0 else terminator_start + len(terminator)


def _quoted_string_end(source: str, quote_index: int, quote: str) -> int:
    index = quote_index + 1
    while index < len(source):
        character = source[index]
        if character == "\\":
            index += 2
            continue
        if character == quote:
            return index + 1
        index += 1
    return len(source)


def _char_literal_end(source: str, start: int) -> int | None:
    quote_index = start + 1 if source.startswith("b'", start) else start
    if quote_index >= len(source) or source[quote_index] != "'":
        return None
    content_index = quote_index + 1
    if content_index >= len(source) or source[content_index] in "\r\n'":
        return None
    if source[content_index] == "\\":
        end = _quoted_string_end(source, quote_index, "'")
        if source[end - 1] != "'" or "\n" in source[quote_index:end]:
            return None
        return end
    if content_index + 1 < len(source) and source[content_index + 1] == "'":
        return content_index + 2
    return None


def mask_non_code(source: str) -> tuple[str, set[int]]:
    """Replace comments and literals with spaces while preserving line breaks."""
    masked = list(source)
    safety_lines: set[int] = set()
    index = 0
    length = len(source)

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = length if end < 0 else end
            _record_safety_lines(source, index, end, safety_lines)
            _mask_range(masked, index, end)
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < length and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            _record_safety_lines(source, index, end, safety_lines)
            _mask_range(masked, index, end)
            index = end
            continue

        raw_end = _raw_string_end(source, index)
        if raw_end is not None:
            _mask_range(masked, index, raw_end)
            index = raw_end
            continue

        if source[index] == '"':
            end = _quoted_string_end(source, index, '"')
            _mask_range(masked, index, end)
            index = end
            continue

        if source.startswith(('b"', 'c"'), index):
            end = _quoted_string_end(source, index + 1, '"')
            _mask_range(masked, index, end)
            index = end
            continue

        char_end = _char_literal_end(source, index)
        if char_end is not None:
            _mask_range(masked, index, char_end)
            index = char_end
            continue

        index += 1

    return "".join(masked), safety_lines


def scan_source(source: str, crate_name: str, relative_path: str) -> list[dict[str, object]]:
    masked, safety_lines = mask_non_code(source)
    source_lines = source.splitlines()
    sites: list[tuple[int, int, dict[str, object]]] = []
    for kind, pattern in SITE_PATTERNS:
        for match in pattern.finditer(masked):
            line_number = masked.count("\n", 0, match.start()) + 1
            line_text = source_lines[line_number - 1].strip() if source_lines else ""
            tagged = any(line in safety_lines for line in range(max(1, line_number - 3), line_number + 1))
            sites.append(
                (
                    line_number,
                    match.start(),
                    {
                        "crate": crate_name,
                        "file": relative_path,
                        "kind": kind,
                        "line": line_number,
                        "safety_comment": tagged,
                        "source": line_text,
                    },
                )
            )
    sites.sort(key=lambda item: (item[0], item[1], str(item[2]["kind"])))
    return [site for _, _, site in sites]


def rust_files(crate: WorkspaceCrate) -> Iterable[Path]:
    files: list[Path] = []
    for directory, directory_names, file_names in os.walk(crate.root):
        directory_names[:] = sorted(
            name for name in directory_names if name not in EXCLUDED_DIRECTORY_NAMES
        )
        for file_name in sorted(file_names):
            if file_name.endswith(".rs"):
                files.append(Path(directory) / file_name)
    return files


def build_inventory(root: Path) -> dict[str, object]:
    crates = workspace_crates(root)
    sites: list[dict[str, object]] = []
    for crate in crates:
        for path in rust_files(crate):
            relative_path = path.relative_to(root).as_posix()
            source = path.read_text(encoding="utf-8")
            sites.extend(scan_source(source, crate.name, relative_path))

    sites.sort(key=lambda site: (str(site["file"]), int(site["line"]), str(site["kind"])))
    by_crate: dict[str, dict[str, int]] = {}
    for crate in crates:
        crate_sites = [site for site in sites if site["crate"] == crate.name]
        tagged = sum(bool(site["safety_comment"]) for site in crate_sites)
        by_crate[crate.name] = {
            "tagged": tagged,
            "total": len(crate_sites),
            "untagged": len(crate_sites) - tagged,
        }

    tagged_total = sum(bool(site["safety_comment"]) for site in sites)
    return {
        "crates": by_crate,
        "schema_version": 1,
        "sites": sites,
        "totals": {
            "tagged": tagged_total,
            "total": len(sites),
            "untagged": len(sites) - tagged_total,
        },
    }


def render_json(inventory: dict[str, object]) -> str:
    return json.dumps(inventory, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def render_summary(inventory: dict[str, object]) -> str:
    crates = inventory["crates"]
    assert isinstance(crates, dict)
    totals = inventory["totals"]
    assert isinstance(totals, dict)
    name_width = max([len("crate"), *(len(str(name)) for name in crates)])
    lines = [
        f"{'crate':<{name_width}}  {'total':>5}  {'tagged':>6}  {'untagged':>8}",
        f"{'-' * name_width}  {'-' * 5}  {'-' * 6}  {'-' * 8}",
    ]
    for name in sorted(crates):
        counts = crates[name]
        assert isinstance(counts, dict)
        lines.append(
            f"{name:<{name_width}}  {counts['total']:>5}  "
            f"{counts['tagged']:>6}  {counts['untagged']:>8}"
        )
    lines.extend(
        [
            f"{'-' * name_width}  {'-' * 5}  {'-' * 6}  {'-' * 8}",
            f"{'TOTAL':<{name_width}}  {totals['total']:>5}  "
            f"{totals['tagged']:>6}  {totals['untagged']:>8}",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    try:
        inventory = build_inventory(repo_root())
        if args.summary:
            sys.stdout.write(render_summary(inventory))
        else:
            output = render_json(inventory)
            if args.out is None:
                sys.stdout.write(output)
            else:
                args.out.write_text(output, encoding="utf-8", newline="\n")
    except (OSError, tomllib.TOMLDecodeError, ValueError) as error:
        print(f"unsafe-inventory: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
