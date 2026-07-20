#!/usr/bin/env python3
"""Inventory explicit unsafe syntax in first-party Rust sources.

Schema v2 deliberately separates discovery from enforcement.  Untagged sites are
reportable during migration, while ``--require-all-tagged`` and ``--check`` are
fail-closed gates for the eventual fully tagged tree.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
import tomllib
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


EXCLUDED_DIRECTORY_NAMES = frozenset({".git", "target", "vendor", "generated"})
RAW_STRING_START = re.compile(r"(?:br|cr|r)(#{0,255})\"")
SITE_PATTERN = re.compile(
    r"\bunsafe\s*(?:"
    r"(?P<block>\{)|"
    r"(?P<extern>extern(?:\s+(?:\"[^\"\r\n]*\"|r#[^\r\n]*?#))?\s+fn\b)|"
    r"(?P<extern_block>extern(?:\s+(?:\"[^\"\r\n]*\"|r#[^\r\n]*?#))?\s*\{)|"
    r"(?P<fn>fn\b)|"
    r"(?P<impl>impl\b)|"
    r"(?P<trait>(?:auto\s+)?trait\b)"
    r")"
)
SITE_KINDS = ("block", "fn", "extern", "extern_block", "impl", "trait")
TAG_MARKER = re.compile(r"\bSAFETY\s*\[")
TAG_GRAMMAR = re.compile(
    r"^\s*(?://+|/\*+|\*+)?\s*SAFETY\["
    r"(?P<id>unsafe\.[a-z0-9][a-z0-9-]*(?:\.[a-z0-9][a-z0-9-]*)+)"
    r"\]\s+Reason:\s*(?P<reason>.+?)\s+Invariant:\s*(?P<invariant>.+?)"
    r"\s*(?:\*/)?\s*$"
)
DECLARATION_PREFIX = re.compile(
    r"^\s*(?:(?:#\s*!?\s*\[[^\]]*\]|pub(?:\s*\([^)]*\))?|async|const|extern\s+\"[^\"]*\")\s*)*$",
    re.DOTALL,
)
ITEM_PATTERN = re.compile(
    r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?"
    r"(?:unsafe\s+)?"
    r"(?:(?P<kind>fn|struct|enum|union|trait|impl|mod)\s+)"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*|[^\s{]+)"
)


@dataclass(frozen=True)
class WorkspaceCrate:
    name: str
    root: Path
    relative_root: str


@dataclass(frozen=True)
class Comment:
    start: int
    end: int
    text: str


@dataclass(frozen=True)
class RawSite:
    start: int
    token_end: int
    end: int
    kind: str


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, help=argparse.SUPPRESS)
    output = parser.add_mutually_exclusive_group()
    output.add_argument("--out", type=Path, help="write canonical schema-v2 JSON")
    output.add_argument("--summary", action="store_true", help="print count summary")
    output.add_argument("--check", type=Path, metavar="INVENTORY", help="compare to inventory")
    parser.add_argument("--build-evidence", action="store_true", help="check inventory and emit kernel-bound evidence")
    parser.add_argument("--artifact", type=Path, help="kernel artifact for build evidence")
    parser.add_argument("--profile", choices=("debug", "release"), help="kernel build profile")
    parser.add_argument("--evidence-out", type=Path, help="atomic build evidence destination")
    parser.add_argument("--require-all-tagged", action="store_true")
    parser.add_argument("--self-test", action="store_true", help="run focused scanner tests")
    return parser.parse_args(argv)


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _sha256(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


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
            if EXCLUDED_DIRECTORY_NAMES.intersection(member_root.relative_to(root).parts):
                continue
            member_roots.add(member_root)
    crates: list[WorkspaceCrate] = []
    for member_root in member_roots:
        manifest = load_toml(member_root / "Cargo.toml")
        package = manifest.get("package")
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            raise ValueError(f"{member_root / 'Cargo.toml'} has no package.name")
        crates.append(WorkspaceCrate(package["name"], member_root, member_root.relative_to(root).as_posix()))
    duplicate_names = [name for name, count in Counter(c.name for c in crates).items() if count > 1]
    if duplicate_names:
        raise ValueError(f"duplicate workspace package names: {', '.join(sorted(duplicate_names))}")
    return sorted(crates, key=lambda crate: (crate.name, crate.relative_root))


def _mask_range(characters: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if characters[index] not in "\r\n":
            characters[index] = " "


def _raw_string_end(source: str, start: int) -> int | None:
    match = RAW_STRING_START.match(source, start)
    if match is None:
        return None
    terminator = '"' + match.group(1)
    found = source.find(terminator, match.end())
    return len(source) if found < 0 else found + len(terminator)


def _quoted_string_end(source: str, quote_index: int, quote: str) -> int:
    index = quote_index + 1
    while index < len(source):
        if source[index] == "\\":
            index += 2
        elif source[index] == quote:
            return index + 1
        else:
            index += 1
    return len(source)


def _char_literal_end(source: str, start: int) -> int | None:
    quote = start + 1 if source.startswith("b'", start) else start
    if quote >= len(source) or source[quote] != "'":
        return None
    content = quote + 1
    if content >= len(source) or source[content] in "\r\n'":
        return None
    if source[content] == "\\":
        end = _quoted_string_end(source, quote, "'")
        return end if end and source[end - 1:end] == "'" and "\n" not in source[quote:end] else None
    return content + 2 if content + 1 < len(source) and source[content + 1] == "'" else None


def lex(source: str) -> tuple[str, str, list[Comment]]:
    """Return code-only mask, comment-free syntax, and exact comments."""
    code = list(source)
    syntax = list(source)
    comments: list[Comment] = []
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end < 0 else end
            comments.append(Comment(index, end, source[index:end]))
            _mask_range(code, index, end)
            _mask_range(syntax, index, end)
            index = end
            continue
        if source.startswith("/*", index):
            depth, end = 1, index + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth, end = depth + 1, end + 2
                elif source.startswith("*/", end):
                    depth, end = depth - 1, end + 2
                else:
                    end += 1
            comments.append(Comment(index, end, source[index:end]))
            _mask_range(code, index, end)
            _mask_range(syntax, index, end)
            index = end
            continue
        raw_end = _raw_string_end(source, index)
        if raw_end is not None:
            _mask_range(code, index, raw_end)
            index = raw_end
            continue
        if source[index] == '"':
            end = _quoted_string_end(source, index, '"')
            _mask_range(code, index, end)
            index = end
            continue
        if source.startswith(('b"', 'c"'), index):
            end = _quoted_string_end(source, index + 1, '"')
            _mask_range(code, index, end)
            index = end
            continue
        char_end = _char_literal_end(source, index)
        if char_end is not None:
            _mask_range(code, index, char_end)
            index = char_end
            continue
        index += 1
    return "".join(code), "".join(syntax), comments


def _balanced_end(masked: str, opening: int) -> int:
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return index + 1
    return len(masked)


def discover_sites(masked: str) -> list[RawSite]:
    sites: list[RawSite] = []
    for match in SITE_PATTERN.finditer(masked):
        kind = next(name for name in SITE_KINDS if match.group(name))
        brace = masked.find("{", match.end())
        semicolon = masked.find(";", match.end())
        if kind == "block":
            brace = match.start("block")
        elif kind == "extern_block":
            brace = match.end() - 1
        if brace >= 0 and (semicolon < 0 or brace < semicolon):
            end = _balanced_end(masked, brace)
        else:
            end = semicolon + 1 if semicolon >= 0 else match.end()
        sites.append(RawSite(match.start(), match.end(), end, kind))
    extern_blocks = [site for site in sites if site.kind == "extern_block"]
    sites = [site for site in sites if site.kind == "extern_block" or not any(block.start < site.start < block.end for block in extern_blocks)]
    return sorted(sites, key=lambda site: (site.start, site.kind))


def _normalize_syntax(value: str) -> str:
    return re.sub(r"\s+", " ", value).strip()


def _enclosing_item(masked: str, offset: int) -> str:
    candidates = [match for match in ITEM_PATTERN.finditer(masked) if match.start() <= offset]
    if not candidates:
        return "<crate>"
    match = candidates[-1]
    return f"{match.group('kind')} {match.group('name')}"


def _diagnostic(code: str, path: str, site: RawSite | None = None, **extra: object) -> dict[str, object]:
    result: dict[str, object] = {"code": code, "path": path}
    if site is not None:
        result["kind"] = site.kind
    result.update(extra)
    return result


def scan_source(source: str, crate_name: str, relative_path: str) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    masked, syntax_source, comments = lex(source)
    raw_sites = discover_sites(masked)
    diagnostics: list[dict[str, object]] = []
    parsed_tags: list[tuple[Comment, re.Match[str]]] = []
    for comment in comments:
        if not TAG_MARKER.search(comment.text):
            continue
        match = TAG_GRAMMAR.match(comment.text)
        if match is None or not match.group("reason").strip() or not match.group("invariant").strip():
            diagnostics.append(_diagnostic("invalid_tag", relative_path, tag_text=comment.text.strip()))
        else:
            parsed_tags.append((comment, match))

    tag_counts = Counter(match.group("id") for _, match in parsed_tags)
    for site_id, count in sorted(tag_counts.items()):
        if count > 1:
            diagnostics.append(_diagnostic("duplicate_tag", relative_path, id=site_id, occurrences=count))

    associations: dict[int, list[tuple[Comment, re.Match[str]]]] = {i: [] for i in range(len(raw_sites))}
    used_comments: set[int] = set()
    block_ranges = [(i, s) for i, s in enumerate(raw_sites) if s.kind == "block"]
    for comment, tag in parsed_tags:
        containing = [(i, site) for i, site in block_ranges if site.token_end <= comment.start < site.end]
        if containing:
            # A tag inside unsafe code belongs only to the innermost containing
            # opening, and only when it is the first non-trivia content.
            index, site = min(containing, key=lambda pair: pair[1].end - pair[1].start)
            if not masked[site.token_end:comment.start].strip():
                associations[index].append((comment, tag))
                used_comments.add(comment.start)
            continue
        following = [
            (i, site) for i, site in enumerate(raw_sites)
            if comment.end <= site.start and DECLARATION_PREFIX.fullmatch(source[comment.end:site.start])
        ]
        if following:
            index, _ = min(following, key=lambda pair: pair[1].start)
            associations[index].append((comment, tag))
            used_comments.add(comment.start)

    for comment, tag in parsed_tags:
        if comment.start not in used_comments:
            diagnostics.append(_diagnostic("orphan_tag", relative_path, id=tag.group("id")))

    sites: list[dict[str, object]] = []
    for index, raw in enumerate(raw_sites):
        tags = associations[index]
        normalized = _normalize_syntax(syntax_source[raw.start:raw.end])
        enclosing = _enclosing_item(masked, raw.start)
        base = {
            "crate": crate_name,
            "path": relative_path,
            "enclosing_item": enclosing,
            "kind": raw.kind,
            "syntax_hash": _sha256(normalized),
        }
        if len(tags) != 1:
            code = "untagged_site" if not tags else "multiple_tags_for_site"
            diagnostics.append(_diagnostic(code, relative_path, raw, enclosing_item=enclosing))
            sites.append({**base, "id": None, "reason": None, "invariant": None, "rationale_hash": None, "association_hash": None})
            continue
        _, tag = tags[0]
        reason, invariant = tag.group("reason").strip(), tag.group("invariant").strip()
        rationale = _sha256(f"Reason:{reason}\nInvariant:{invariant}")
        site_id = tag.group("id")
        sites.append({
            **base,
            "id": site_id,
            "reason": reason,
            "invariant": invariant,
            "rationale_hash": rationale,
            "association_hash": _sha256(f"{raw.kind}\n{normalized}\n{reason}\n{invariant}"),
        })

    ids: dict[str, list[dict[str, object]]] = {}
    for site in sites:
        if site["id"] is not None:
            ids.setdefault(str(site["id"]), []).append(site)
    for site_id, matches in sorted(ids.items()):
        if len(matches) > 1:
            diagnostics.append(_diagnostic("duplicate_id", relative_path, id=site_id, occurrences=len(matches)))
    return sites, diagnostics


def rust_files(crate: WorkspaceCrate) -> Iterable[Path]:
    files: list[Path] = []
    for directory, directory_names, file_names in os.walk(crate.root):
        directory_names[:] = sorted(name for name in directory_names if name not in EXCLUDED_DIRECTORY_NAMES)
        files.extend(Path(directory) / name for name in sorted(file_names) if name.endswith(".rs"))
    return files


def build_inventory(root: Path) -> dict[str, object]:
    crates = workspace_crates(root)
    sites: list[dict[str, object]] = []
    diagnostics: list[dict[str, object]] = []
    for crate in crates:
        for path in rust_files(crate):
            relative = path.relative_to(root).as_posix()
            found, errors = scan_source(path.read_text(encoding="utf-8"), crate.name, relative)
            sites.extend(found)
            diagnostics.extend(errors)
    sites.sort(key=lambda site: (str(site["path"]), str(site["enclosing_item"]), str(site["kind"]), str(site["id"]), str(site["syntax_hash"])))
    diagnostics.sort(key=lambda item: json.dumps(item, sort_keys=True))
    global_ids: dict[str, int] = Counter(str(s["id"]) for s in sites if s["id"] is not None)
    existing_duplicate_diagnostics = {(d.get("code"), d.get("id")) for d in diagnostics}
    for site_id, count in sorted(global_ids.items()):
        if count > 1 and ("duplicate_id", site_id) not in existing_duplicate_diagnostics:
            diagnostics.append({"code": "duplicate_id", "id": site_id, "occurrences": count, "path": "<workspace>"})
    by_kind = {kind: sum(s["kind"] == kind for s in sites) for kind in SITE_KINDS}
    tagged = sum(s["id"] is not None for s in sites)
    by_crate: dict[str, dict[str, int]] = {}
    for crate in crates:
        selected = [site for site in sites if site["crate"] == crate.name]
        crate_tagged = sum(site["id"] is not None for site in selected)
        by_crate[crate.name] = {"tagged": crate_tagged, "total": len(selected), "untagged": len(selected) - crate_tagged}
    return {
        "schema_version": 2,
        "crates": by_crate,
        "sites": sites,
        "diagnostics": diagnostics,
        "totals": {"by_kind": by_kind, "tagged": tagged, "total": len(sites), "untagged": len(sites) - tagged},
    }


def render_json(inventory: dict[str, object]) -> str:
    return json.dumps(inventory, ensure_ascii=False, separators=(",", ":"), sort_keys=True) + "\n"


def render_summary(inventory: dict[str, object]) -> str:
    totals = inventory["totals"]
    assert isinstance(totals, dict)
    kinds = totals["by_kind"]
    assert isinstance(kinds, dict)
    return (
        f"total={totals['total']} tagged={totals['tagged']} untagged={totals['untagged']} "
        + " ".join(f"{kind}={kinds[kind]}" for kind in SITE_KINDS)
        + "\n"
    )


def compare_inventory(expected: dict[str, object], actual: dict[str, object]) -> list[dict[str, object]]:
    differences: list[dict[str, object]] = []
    if expected.get("schema_version") != 2:
        return [{"code": "unsupported_inventory_schema", "expected": 2, "found": expected.get("schema_version")}]
    expected_sites = expected.get("sites")
    actual_sites = actual.get("sites")
    if not isinstance(expected_sites, list) or not isinstance(actual_sites, list):
        return [{"code": "invalid_inventory_sites"}]
    old = {str(s.get("id")): s for s in expected_sites if isinstance(s, dict) and s.get("id") is not None}
    new = {str(s.get("id")): s for s in actual_sites if isinstance(s, dict) and s.get("id") is not None}
    for site_id in sorted(old.keys() - new.keys()):
        differences.append({"code": "removed_site", "id": site_id})
    for site_id in sorted(new.keys() - old.keys()):
        differences.append({"code": "added_site", "id": site_id})
    fields = {
        "kind": "kind_drift", "path": "path_drift", "enclosing_item": "item_drift",
        "reason": "rationale_drift", "invariant": "rationale_drift",
        "rationale_hash": "rationale_hash_drift", "syntax_hash": "syntax_hash_drift",
        "association_hash": "association_hash_drift",
    }
    for site_id in sorted(old.keys() & new.keys()):
        emitted: set[str] = set()
        for field, code in fields.items():
            if old[site_id].get(field) != new[site_id].get(field) and code not in emitted:
                differences.append({"code": code, "id": site_id, "field": field})
                emitted.add(code)
    old_untagged = Counter((s.get("path"), s.get("kind"), s.get("syntax_hash")) for s in expected_sites if isinstance(s, dict) and s.get("id") is None)
    new_untagged = Counter((s.get("path"), s.get("kind"), s.get("syntax_hash")) for s in actual_sites if isinstance(s, dict) and s.get("id") is None)
    if old_untagged != new_untagged:
        differences.append({"code": "untagged_set_drift"})
    return differences


def run_self_test() -> int:
    import unittest
    suite_path = Path(__file__).resolve().parent / "tests" / "unsafe-inventory"
    suite = unittest.defaultTestLoader.discover(str(suite_path), pattern="test_*.py")
    return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1


def _file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return "sha256:" + digest.hexdigest()


def emit_build_evidence(root: Path, baseline: Path, artifact: Path, profile: str, output: Path) -> None:
    baseline_bytes = baseline.read_bytes()
    expected = json.loads(baseline_bytes)
    current = build_inventory(root)
    differences = compare_inventory(expected, current)
    if differences:
        raise ValueError("stale baseline: " + json.dumps(differences, sort_keys=True))
    invalid = [item for item in current["diagnostics"] if item["code"] != "untagged_site"]
    if invalid:
        raise ValueError("invalid inventory tags: " + json.dumps(invalid, sort_keys=True))
    artifact = artifact.resolve(strict=True)
    if not artifact.is_file():
        raise ValueError(f"kernel artifact is not a file: {artifact}")
    canonical = render_json(current).encode("utf-8")
    totals = current["totals"]
    evidence = {
        "schema_version": 1,
        "status": "accepted",
        "profile": profile,
        "artifact": {"path": artifact.as_posix(), "sha256": _file_sha256(artifact)},
        "inventory": current,
        "inventory_sha256": "sha256:" + hashlib.sha256(canonical).hexdigest(),
        "baseline_sha256": "sha256:" + hashlib.sha256(baseline_bytes).hexdigest(),
        "counts": {"total": totals["total"], "tagged": totals["tagged"], "untagged": totals["untagged"], "by_kind": totals["by_kind"], "by_crate": current["crates"]},
    }
    encoded = (json.dumps(evidence, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")
    output = output.resolve()
    output.parent.mkdir(parents=False, exist_ok=True)
    temporary: Path | None = None
    try:
        descriptor, name = tempfile.mkstemp(prefix=f".{output.name}.", suffix=".tmp", dir=output.parent)
        temporary = Path(name)
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(encoded)
            handle.flush()
            os.fsync(handle.fileno())
        parsed = json.loads(temporary.read_bytes())
        if parsed != evidence:
            raise ValueError("post-write evidence verification failed")
        os.replace(temporary, output)
        temporary = None
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        return run_self_test()
    root = (args.root or repo_root()).resolve()
    try:
        inventory = build_inventory(root)
        exit_code = 0
        if args.build_evidence:
            if args.check is None or args.artifact is None or args.profile is None or args.evidence_out is None:
                raise ValueError("--build-evidence requires --check, --artifact, --profile, and --evidence-out")
            emit_build_evidence(root, args.check, args.artifact, args.profile, args.evidence_out)
            return 0
        if args.artifact is not None or args.profile is not None or args.evidence_out is not None:
            raise ValueError("evidence options require --build-evidence")
        if args.check is not None:
            expected = json.loads(args.check.read_text(encoding="utf-8"))
            differences = compare_inventory(expected, inventory)
            if differences:
                sys.stderr.write(json.dumps({"status": "stale", "differences": differences}, sort_keys=True) + "\n")
                exit_code = 1
        elif args.summary:
            sys.stdout.write(render_summary(inventory))
        else:
            output = render_json(inventory)
            if args.out is None:
                sys.stdout.write(output)
            else:
                args.out.write_text(output, encoding="utf-8", newline="\n")
        if args.require_all_tagged and inventory["totals"]["untagged"]:
            sys.stderr.write(json.dumps({"code": "untagged_sites", "count": inventory["totals"]["untagged"]}, sort_keys=True) + "\n")
            exit_code = 1
        invalid = [d for d in inventory["diagnostics"] if d["code"] != "untagged_site"]
        if invalid:
            sys.stderr.write(json.dumps({"code": "invalid_tags", "diagnostics": invalid}, sort_keys=True) + "\n")
            exit_code = 1
        return exit_code
    except (OSError, json.JSONDecodeError, tomllib.TOMLDecodeError, ValueError) as error:
        print(f"unsafe-inventory: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
