#!/usr/bin/env python3
"""Create the small raiOS FAT32 boot image without Linux mtools."""

from __future__ import annotations

import argparse
import math
import struct
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

SECTOR_SIZE = 512
SECTORS_PER_CLUSTER = 1
RESERVED_SECTORS = 32
FAT_COUNT = 2
ROOT_CLUSTER = 2
END_OF_CHAIN = 0x0FFFFFFF
MEDIA_DESCRIPTOR = 0xF8


@dataclass
class FileNode:
    name: str
    data: bytes
    first_cluster: int = 0


@dataclass
class DirNode:
    name: str
    first_cluster: int
    children: list[DirNode | FileNode] = field(default_factory=list)
    parent: DirNode | None = None


class Fat32Builder:
    def __init__(self, size_bytes: int) -> None:
        if size_bytes % SECTOR_SIZE != 0:
            raise ValueError("image size must be sector-aligned")
        self.size_bytes = size_bytes
        self.total_sectors = size_bytes // SECTOR_SIZE
        self.fat_sectors = self._compute_fat_sectors()
        self.data_start_sector = RESERVED_SECTORS + FAT_COUNT * self.fat_sectors
        self.cluster_count = (
            self.total_sectors - self.data_start_sector
        ) // SECTORS_PER_CLUSTER
        self.fat = [0] * (self.cluster_count + 2)
        self.fat[0] = 0x0FFFFFF0 | MEDIA_DESCRIPTOR
        self.fat[1] = END_OF_CHAIN
        self.fat[ROOT_CLUSTER] = END_OF_CHAIN
        self.next_cluster = ROOT_CLUSTER + 1
        self.image = bytearray(size_bytes)
        self.root = DirNode("", ROOT_CLUSTER)

    def _compute_fat_sectors(self) -> int:
        fat_sectors = 1
        while True:
            data_sectors = self.total_sectors - RESERVED_SECTORS - FAT_COUNT * fat_sectors
            cluster_count = data_sectors // SECTORS_PER_CLUSTER
            required = math.ceil((cluster_count + 2) * 4 / SECTOR_SIZE)
            if required <= fat_sectors:
                return fat_sectors
            fat_sectors = required

    def add_file(self, path: str, data: bytes) -> None:
        parts = [part for part in path.replace("\\", "/").split("/") if part]
        if not parts:
            raise ValueError("empty path")
        directory = self.root
        for part in parts[:-1]:
            directory = self._ensure_dir(directory, part)
        directory.children.append(FileNode(parts[-1], data))

    def _ensure_dir(self, parent: DirNode, name: str) -> DirNode:
        for child in parent.children:
            if isinstance(child, DirNode) and child.name.upper() == name.upper():
                return child
        node = DirNode(name, self._alloc_cluster(), parent=parent)
        parent.children.append(node)
        return node

    def _alloc_cluster(self) -> int:
        if self.next_cluster >= len(self.fat):
            raise RuntimeError("FAT image is out of clusters")
        cluster = self.next_cluster
        self.next_cluster += 1
        self.fat[cluster] = END_OF_CHAIN
        return cluster

    def _alloc_chain(self, length: int) -> int:
        clusters = math.ceil(length / (SECTOR_SIZE * SECTORS_PER_CLUSTER))
        if clusters == 0:
            return 0
        first = self._alloc_cluster()
        previous = first
        for _ in range(1, clusters):
            current = self._alloc_cluster()
            self.fat[previous] = current
            previous = current
        self.fat[previous] = END_OF_CHAIN
        return first

    def build(self) -> bytes:
        self._assign_file_clusters(self.root)
        self._write_boot_region()
        self._write_node_data(self.root)
        self._write_fats()
        return bytes(self.image)

    def _assign_file_clusters(self, directory: DirNode) -> None:
        for child in directory.children:
            if isinstance(child, FileNode):
                child.first_cluster = self._alloc_chain(len(child.data))
            else:
                self._assign_file_clusters(child)

    def _write_boot_region(self) -> None:
        boot = bytearray(SECTOR_SIZE)
        boot[0:3] = b"\xEB\x58\x90"
        boot[3:11] = b"RAIOS  "
        struct.pack_into("<H", boot, 11, SECTOR_SIZE)
        boot[13] = SECTORS_PER_CLUSTER
        struct.pack_into("<H", boot, 14, RESERVED_SECTORS)
        boot[16] = FAT_COUNT
        struct.pack_into("<H", boot, 17, 0)
        struct.pack_into("<H", boot, 19, 0)
        boot[21] = MEDIA_DESCRIPTOR
        struct.pack_into("<H", boot, 22, 0)
        struct.pack_into("<H", boot, 24, 32)
        struct.pack_into("<H", boot, 26, 64)
        struct.pack_into("<I", boot, 28, 0)
        struct.pack_into("<I", boot, 32, self.total_sectors)
        struct.pack_into("<I", boot, 36, self.fat_sectors)
        struct.pack_into("<H", boot, 40, 0)
        struct.pack_into("<H", boot, 42, 0)
        struct.pack_into("<I", boot, 44, ROOT_CLUSTER)
        struct.pack_into("<H", boot, 48, 1)
        struct.pack_into("<H", boot, 50, 6)
        boot[64] = 0x80
        boot[66] = 0x29
        struct.pack_into("<I", boot, 67, 0x53454544)
        boot[71:82] = b"RAIOS     "
        boot[82:90] = b"FAT32   "
        boot[510:512] = b"\x55\xAA"
        self._write_sector(0, boot)
        self._write_sector(6, boot)

        fsinfo = bytearray(SECTOR_SIZE)
        struct.pack_into("<I", fsinfo, 0, 0x41615252)
        struct.pack_into("<I", fsinfo, 484, 0x61417272)
        struct.pack_into("<I", fsinfo, 488, 0xFFFFFFFF)
        struct.pack_into("<I", fsinfo, 492, self.next_cluster)
        fsinfo[510:512] = b"\x55\xAA"
        self._write_sector(1, fsinfo)
        self._write_sector(7, fsinfo)

    def _write_fats(self) -> None:
        fat_bytes = bytearray(self.fat_sectors * SECTOR_SIZE)
        for index, value in enumerate(self.fat):
            struct.pack_into("<I", fat_bytes, index * 4, value)
        for fat_index in range(FAT_COUNT):
            start = (RESERVED_SECTORS + fat_index * self.fat_sectors) * SECTOR_SIZE
            self.image[start : start + len(fat_bytes)] = fat_bytes

    def _write_node_data(self, directory: DirNode) -> None:
        self._write_directory(directory)
        for child in directory.children:
            if isinstance(child, FileNode):
                self._write_file(child)
            else:
                self._write_node_data(child)

    def _write_directory(self, directory: DirNode) -> None:
        cluster_bytes = SECTOR_SIZE * SECTORS_PER_CLUSTER
        data = bytearray(cluster_bytes)
        entries: list[bytes] = []
        if directory.parent is not None:
            entries.append(directory_entry(".", 0x10, directory.first_cluster, 0))
            parent_cluster = directory.parent.first_cluster
            entries.append(directory_entry("..", 0x10, parent_cluster, 0))
        for child in directory.children:
            attr = 0x10 if isinstance(child, DirNode) else 0x20
            size = 0 if isinstance(child, DirNode) else len(child.data)
            entries.extend(directory_entries(child.name, attr, child.first_cluster, size))
        if len(entries) * 32 > cluster_bytes:
            raise RuntimeError(f"directory {directory.name or '/'} is too large")
        for index, entry in enumerate(entries):
            data[index * 32 : index * 32 + 32] = entry
        self._write_cluster(directory.first_cluster, data)

    def _write_file(self, node: FileNode) -> None:
        if node.first_cluster == 0:
            return
        remaining = node.data
        cluster = node.first_cluster
        cluster_bytes = SECTOR_SIZE * SECTORS_PER_CLUSTER
        while cluster < END_OF_CHAIN:
            chunk = remaining[:cluster_bytes]
            self._write_cluster(cluster, chunk.ljust(cluster_bytes, b"\0"))
            remaining = remaining[cluster_bytes:]
            next_cluster = self.fat[cluster]
            if next_cluster >= 0x0FFFFFF8:
                break
            cluster = next_cluster

    def _write_sector(self, sector: int, data: bytes | bytearray) -> None:
        start = sector * SECTOR_SIZE
        self.image[start : start + len(data)] = data

    def _write_cluster(self, cluster: int, data: bytes | bytearray) -> None:
        sector = self.data_start_sector + (cluster - 2) * SECTORS_PER_CLUSTER
        self._write_sector(sector, data)


def directory_entry(name: str, attr: int, first_cluster: int, size: int) -> bytes:
    return directory_entry_from_short(short_name(name), attr, first_cluster, size)


def directory_entries(name: str, attr: int, first_cluster: int, size: int) -> list[bytes]:
    try:
        short = short_name(name)
        return [directory_entry_from_short(short, attr, first_cluster, size)]
    except ValueError:
        short = short_alias(name)
        return lfn_entries(name, short) + [directory_entry_from_short(short, attr, first_cluster, size)]


def directory_entry_from_short(short: bytes, attr: int, first_cluster: int, size: int) -> bytes:
    now = datetime.now()
    fat_time = (now.hour << 11) | (now.minute << 5) | (now.second // 2)
    fat_date = ((max(now.year, 1980) - 1980) << 9) | (now.month << 5) | now.day
    entry = bytearray(32)
    entry[0:11] = short
    entry[11] = attr
    struct.pack_into("<H", entry, 14, fat_time)
    struct.pack_into("<H", entry, 16, fat_date)
    struct.pack_into("<H", entry, 18, fat_date)
    struct.pack_into("<H", entry, 20, (first_cluster >> 16) & 0xFFFF)
    struct.pack_into("<H", entry, 22, fat_time)
    struct.pack_into("<H", entry, 24, fat_date)
    struct.pack_into("<H", entry, 26, first_cluster & 0xFFFF)
    struct.pack_into("<I", entry, 28, size)
    return bytes(entry)


def lfn_entries(name: str, short: bytes) -> list[bytes]:
    chars = [ord(ch) for ch in name]
    chunks = [chars[index : index + 13] for index in range(0, len(chars), 13)]
    checksum = lfn_checksum(short)
    entries: list[bytes] = []
    for idx, chunk in enumerate(reversed(chunks), start=1):
        sequence = len(chunks) - idx + 1
        if idx == 1:
            sequence |= 0x40
        padded = chunk + ([0x0000] if len(chunk) < 13 else [])
        padded += [0xFFFF] * (13 - len(padded))
        entry = bytearray(32)
        entry[0] = sequence
        write_utf16_slots(entry, 1, padded[0:5])
        entry[11] = 0x0F
        entry[12] = 0
        entry[13] = checksum
        write_utf16_slots(entry, 14, padded[5:11])
        struct.pack_into("<H", entry, 26, 0)
        write_utf16_slots(entry, 28, padded[11:13])
        entries.append(bytes(entry))
    return entries


def write_utf16_slots(entry: bytearray, offset: int, values: list[int]) -> None:
    for index, value in enumerate(values):
        struct.pack_into("<H", entry, offset + index * 2, value)


def lfn_checksum(short: bytes) -> int:
    value = 0
    for byte in short:
        value = (((value & 1) << 7) + (value >> 1) + byte) & 0xFF
    return value


def short_name(name: str) -> bytes:
    if name in {".", ".."}:
        return name.encode("ascii").ljust(11, b" ")
    upper = name.upper()
    if "." in upper:
        stem, ext = upper.rsplit(".", 1)
    else:
        stem, ext = upper, ""
    allowed = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$%'-_@~`!(){}^#&")
    if not stem or len(stem) > 8 or len(ext) > 3:
        raise ValueError(f"{name!r} is not an 8.3 FAT name")
    if any(ch not in allowed for ch in stem + ext):
        raise ValueError(f"{name!r} contains unsupported FAT characters")
    return stem.encode("ascii").ljust(8, b" ") + ext.encode("ascii").ljust(3, b" ")


def short_alias(name: str) -> bytes:
    upper = name.upper()
    if "." in upper:
        stem, ext = upper.rsplit(".", 1)
    else:
        stem, ext = upper, ""
    allowed = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$%'-_@~`!(){}^#&")
    stem = "".join(ch for ch in stem if ch in allowed)[:6] or "FILE"
    ext = "".join(ch for ch in ext if ch in allowed)[:3]
    return (stem + "~1").encode("ascii").ljust(8, b" ") + ext.encode("ascii").ljust(3, b" ")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--size", default=64 * 1024 * 1024, type=int)
    args = parser.parse_args()

    root = args.root.resolve()
    files = [
        ("limine.conf", root / "limine.conf"),
        ("EFI/BOOT/BOOTX64.EFI", root / "EFI" / "BOOT" / "BOOTX64.EFI"),
        ("EFI/BOOT/limine.conf", root / "EFI" / "BOOT" / "limine.conf"),
        ("kernel/kernel.elf", root / "kernel" / "kernel.elf"),
    ]

    builder = Fat32Builder(args.size)
    for image_path, source in files:
        if not source.exists():
            raise FileNotFoundError(source)
        builder.add_file(image_path, source.read_bytes())

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(builder.build())
    print(f"wrote {args.output} ({args.size} bytes)")


if __name__ == "__main__":
    main()
