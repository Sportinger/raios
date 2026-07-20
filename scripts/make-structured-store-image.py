#!/usr/bin/env python3
"""Create and verify the disposable raiOS M13 structured-store QEMU disk."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import json
import os
import stat
import struct
import sys
import tempfile
import uuid
from pathlib import Path


SECTOR_SIZE = 512
GPT_HEADER_SIZE = 92
GPT_ENTRY_SIZE = 128
GPT_ENTRY_COUNT = 128
GPT_ENTRY_ARRAY_BYTES = GPT_ENTRY_SIZE * GPT_ENTRY_COUNT
GPT_ENTRY_ARRAY_LBAS = GPT_ENTRY_ARRAY_BYTES // SECTOR_SIZE
PRIMARY_HEADER_LBA = 1
PRIMARY_ENTRIES_LBA = 2
FIRST_USABLE_LBA = 34
FIRST_PARTITION_LBA = 2048
MIN_SIZE_MIB = 16
MAX_SIZE_MIB = 4096
DEFAULT_SIZE_MIB = 64
READ_CHUNK_SIZE = 1024 * 1024

# The type and label are the generic M13 structured-store contract. Real media
# keep their own fingerprint-pinned unique GUID; these unique IDs are QEMU-only.
PARTITION_TYPE_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000013")
PARTITION_LABEL = "RAIOS_STRUCTURED_STORE"
TEST_DISK_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000001300")
TEST_PARTITION_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000001301")
MUTATED_PARTITION_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-ffffffff1301")

STORE_BLOCK_LEN = 512
STORE_SUPERBLOCK_MAGIC = b"RAIOSSB1"
STORE_FRAME_MAGIC = b"RAIOSFR1"
STORE_SUPERBLOCK_HASH_OFFSET = 88
STORE_SUPERBLOCK_HASH_END = 120
STORE_FRAME_HEADER_LEN = 208
STORE_FRAME_HEADER_HASH_OFFSET = 168
STORE_FRAME_HEADER_HASH_END = 200
STORE_FRAME_CRC32_OFFSET = 200
STORE_UUID = b"c1-test-store-v1"


def crc32(data: bytes | bytearray) -> int:
    return binascii.crc32(data) & 0xFFFFFFFF


def geometry(size_bytes: int) -> dict[str, int]:
    if size_bytes % SECTOR_SIZE:
        raise ValueError("image size is not sector-aligned")
    total_lbas = size_bytes // SECTOR_SIZE
    backup_header_lba = total_lbas - 1
    backup_entries_lba = backup_header_lba - GPT_ENTRY_ARRAY_LBAS
    last_usable_lba = backup_entries_lba - 1
    if FIRST_PARTITION_LBA > last_usable_lba:
        raise ValueError("image is too small for the aligned structured-store partition")
    return {
        "total_lbas": total_lbas,
        "backup_header_lba": backup_header_lba,
        "backup_entries_lba": backup_entries_lba,
        "last_usable_lba": last_usable_lba,
    }


def partition_entry(last_lba: int) -> bytes:
    name = PARTITION_LABEL.encode("utf-16-le")
    if len(name) > 72:
        raise ValueError("partition label exceeds the GPT limit")
    return struct.pack(
        "<16s16sQQQ72s",
        PARTITION_TYPE_GUID.bytes_le,
        TEST_PARTITION_GUID.bytes_le,
        FIRST_PARTITION_LBA,
        last_lba,
        0,
        name.ljust(72, b"\0"),
    )


def partition_entries(last_lba: int) -> bytes:
    return partition_entry(last_lba).ljust(GPT_ENTRY_ARRAY_BYTES, b"\0")


def gpt_header(
    *,
    current_lba: int,
    backup_lba: int,
    entries_lba: int,
    entries_crc: int,
    last_usable_lba: int,
) -> bytes:
    header = bytearray(SECTOR_SIZE)
    struct.pack_into(
        "<8sIIIIQQQQ16sQIII",
        header,
        0,
        b"EFI PART",
        0x00010000,
        GPT_HEADER_SIZE,
        0,
        0,
        current_lba,
        backup_lba,
        FIRST_USABLE_LBA,
        last_usable_lba,
        TEST_DISK_GUID.bytes_le,
        entries_lba,
        GPT_ENTRY_COUNT,
        GPT_ENTRY_SIZE,
        entries_crc,
    )
    struct.pack_into("<I", header, 16, crc32(header[:GPT_HEADER_SIZE]))
    return bytes(header)


def protective_mbr(total_lbas: int) -> bytes:
    mbr = bytearray(SECTOR_SIZE)
    sector_count = min(total_lbas - 1, 0xFFFFFFFF)
    mbr[446:462] = struct.pack(
        "<B3sB3sII", 0, b"\x00\x02\x00", 0xEE, b"\xff\xff\xff", 1, sector_count
    )
    mbr[510:512] = b"\x55\xaa"
    return bytes(mbr)


def write_image(handle, size_bytes: int) -> None:
    geo = geometry(size_bytes)
    entries = partition_entries(geo["last_usable_lba"])
    entries_crc = crc32(entries)
    primary_header = gpt_header(
        current_lba=PRIMARY_HEADER_LBA,
        backup_lba=geo["backup_header_lba"],
        entries_lba=PRIMARY_ENTRIES_LBA,
        entries_crc=entries_crc,
        last_usable_lba=geo["last_usable_lba"],
    )
    backup_header = gpt_header(
        current_lba=geo["backup_header_lba"],
        backup_lba=PRIMARY_HEADER_LBA,
        entries_lba=geo["backup_entries_lba"],
        entries_crc=entries_crc,
        last_usable_lba=geo["last_usable_lba"],
    )

    handle.truncate(size_bytes)
    for offset, data in (
        (0, protective_mbr(geo["total_lbas"])),
        (PRIMARY_HEADER_LBA * SECTOR_SIZE, primary_header),
        (PRIMARY_ENTRIES_LBA * SECTOR_SIZE, entries),
        (geo["backup_entries_lba"] * SECTOR_SIZE, entries),
        (geo["backup_header_lba"] * SECTOR_SIZE, backup_header),
    ):
        handle.seek(offset)
        handle.write(data)
    handle.flush()
    os.fsync(handle.fileno())


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def is_beneath(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def reject_device_path(path: Path) -> Path:
    raw = os.fspath(path)
    windows = raw.replace("/", "\\").lower()
    if windows.startswith(("\\\\.\\", "\\\\?\\", "\\device\\")):
        raise ValueError("device paths are forbidden; use a disposable regular image file")
    expanded = path.expanduser()
    if expanded.is_symlink():
        raise ValueError("symbolic-link images are forbidden")
    resolved = expanded.resolve(strict=False)
    posix = resolved.as_posix()
    if posix == "/dev" or posix.startswith(("/dev/", "/proc/", "/sys/")):
        raise ValueError("device and kernel pseudo-filesystems are forbidden")
    return resolved


def validate_image_path(path: Path, *, output: bool) -> Path:
    resolved = reject_device_path(path)
    if resolved.suffix.lower() not in {".img", ".raw"}:
        raise ValueError("test disk path must end in .img or .raw")
    if output:
        if "structured-store" not in resolved.name.lower():
            raise ValueError("output filename must contain 'structured-store'")
        forbidden_names = ("stage0", "seed_data", "seed-data", "reclog", "esp-a", "esp-b")
        if any(name in resolved.name.lower() for name in forbidden_names):
            raise ValueError("output filename resembles a boot or existing persistence artifact")
        allowed_roots = (
            (repo_root() / "target").resolve(),
            Path(tempfile.gettempdir()).resolve(),
        )
        if not any(is_beneath(resolved, root) for root in allowed_roots):
            raise ValueError("output must stay under repo target/ or the host temp directory")
        if not resolved.parent.is_dir():
            raise ValueError(f"output parent does not exist: {resolved.parent}")
    return resolved


def require_regular_file(path: Path) -> os.stat_result:
    if path.is_symlink():
        raise ValueError("symbolic-link images are forbidden")
    result = path.stat()
    if not stat.S_ISREG(result.st_mode):
        raise ValueError("image is not a regular file")
    return result


def parse_header(raw: bytes, *, expected_lba: int) -> dict[str, object]:
    if len(raw) != SECTOR_SIZE:
        raise ValueError("short GPT header read")
    fields = struct.unpack_from("<8sIIIIQQQQ16sQIII", raw)
    (
        signature,
        revision,
        header_size,
        stored_crc,
        reserved,
        current_lba,
        backup_lba,
        first_usable_lba,
        last_usable_lba,
        disk_guid,
        entries_lba,
        entry_count,
        entry_size,
        entries_crc,
    ) = fields
    if signature != b"EFI PART":
        raise ValueError(f"GPT header at LBA {expected_lba} has bad signature")
    if revision != 0x00010000 or header_size != GPT_HEADER_SIZE or reserved != 0:
        raise ValueError(f"GPT header at LBA {expected_lba} has unsupported metadata")
    if any(raw[GPT_HEADER_SIZE:]):
        raise ValueError(f"GPT header at LBA {expected_lba} has nonzero reserved bytes")
    if current_lba != expected_lba:
        raise ValueError(f"GPT header at LBA {expected_lba} has wrong current LBA")
    checked = bytearray(raw[:header_size])
    checked[16:20] = b"\0" * 4
    if crc32(checked) != stored_crc:
        raise ValueError(f"GPT header CRC mismatch at LBA {expected_lba}")
    return {
        "current_lba": current_lba,
        "backup_lba": backup_lba,
        "first_usable_lba": first_usable_lba,
        "last_usable_lba": last_usable_lba,
        "disk_guid": str(uuid.UUID(bytes_le=disk_guid)),
        "entries_lba": entries_lba,
        "entry_count": entry_count,
        "entry_size": entry_size,
        "entries_crc32": entries_crc,
        "header_crc32": stored_crc,
    }


def read_exact(handle, offset: int, length: int) -> bytes:
    handle.seek(offset)
    data = handle.read(length)
    if len(data) != length:
        raise ValueError(f"short image read at byte {offset}")
    return data


def verify_protective_mbr(raw: bytes, total_lbas: int) -> None:
    if len(raw) != SECTOR_SIZE or raw[510:512] != b"\x55\xaa":
        raise ValueError("protective MBR signature is missing")
    if raw != protective_mbr(total_lbas):
        raise ValueError("protective MBR is not the exact non-bootable test-disk marker")
    if raw[446 + 4] != 0xEE:
        raise ValueError("protective MBR does not contain the 0xEE partition")
    start_lba, sector_count = struct.unpack_from("<II", raw, 446 + 8)
    if start_lba != 1 or sector_count != min(total_lbas - 1, 0xFFFFFFFF):
        raise ValueError("protective MBR bounds do not cover the disk")
    if any(raw[462:510]):
        raise ValueError("protective MBR contains unexpected additional partitions")


def decode_partition(entries: bytes) -> dict[str, object]:
    if any(entries[GPT_ENTRY_SIZE:]):
        raise ValueError("GPT must contain exactly one partition")
    type_bytes, unique_bytes, first_lba, last_lba, attributes, name_bytes = struct.unpack_from(
        "<16s16sQQQ72s", entries
    )
    try:
        name = name_bytes.decode("utf-16-le").split("\0", 1)[0]
    except UnicodeDecodeError as exc:
        raise ValueError("partition label is not valid UTF-16LE") from exc
    return {
        "type_guid": str(uuid.UUID(bytes_le=type_bytes)),
        "unique_guid": str(uuid.UUID(bytes_le=unique_bytes)),
        "first_lba": first_lba,
        "last_lba": last_lba,
        "attributes": attributes,
        "label": name,
    }


def partition_is_zero(handle, first_lba: int, last_lba: int) -> bool:
    remaining = (last_lba - first_lba + 1) * SECTOR_SIZE
    handle.seek(first_lba * SECTOR_SIZE)
    while remaining:
        chunk = handle.read(min(remaining, READ_CHUNK_SIZE))
        if not chunk or any(chunk):
            return False
        remaining -= len(chunk)
    return True


def verify_image(path: Path, *, require_empty: bool = True) -> dict[str, object]:
    resolved = validate_image_path(path, output=False)
    result = require_regular_file(resolved)
    size_bytes = result.st_size
    if size_bytes < MIN_SIZE_MIB * 1024 * 1024 or size_bytes > MAX_SIZE_MIB * 1024 * 1024:
        raise ValueError("image size is outside the disposable test-disk limits")
    geo = geometry(size_bytes)

    with resolved.open("rb") as handle:
        verify_protective_mbr(read_exact(handle, 0, SECTOR_SIZE), geo["total_lbas"])
        primary = parse_header(
            read_exact(handle, PRIMARY_HEADER_LBA * SECTOR_SIZE, SECTOR_SIZE),
            expected_lba=PRIMARY_HEADER_LBA,
        )
        backup = parse_header(
            read_exact(handle, geo["backup_header_lba"] * SECTOR_SIZE, SECTOR_SIZE),
            expected_lba=geo["backup_header_lba"],
        )
        expected_common = {
            "first_usable_lba": FIRST_USABLE_LBA,
            "last_usable_lba": geo["last_usable_lba"],
            "disk_guid": str(TEST_DISK_GUID),
            "entry_count": GPT_ENTRY_COUNT,
            "entry_size": GPT_ENTRY_SIZE,
        }
        for name, header, expected_backup, expected_entries in (
            ("primary", primary, geo["backup_header_lba"], PRIMARY_ENTRIES_LBA),
            ("backup", backup, PRIMARY_HEADER_LBA, geo["backup_entries_lba"]),
        ):
            for field, expected in expected_common.items():
                if header[field] != expected:
                    raise ValueError(f"{name} GPT {field} identity mismatch")
            if header["backup_lba"] != expected_backup or header["entries_lba"] != expected_entries:
                raise ValueError(f"{name} GPT redundant-copy geometry mismatch")

        primary_entries = read_exact(
            handle, PRIMARY_ENTRIES_LBA * SECTOR_SIZE, GPT_ENTRY_ARRAY_BYTES
        )
        backup_entries = read_exact(
            handle, geo["backup_entries_lba"] * SECTOR_SIZE, GPT_ENTRY_ARRAY_BYTES
        )
        if crc32(primary_entries) != primary["entries_crc32"]:
            raise ValueError("primary GPT partition-array CRC mismatch")
        if crc32(backup_entries) != backup["entries_crc32"]:
            raise ValueError("backup GPT partition-array CRC mismatch")
        if primary_entries != backup_entries:
            raise ValueError("primary and backup GPT partition arrays differ")
        partition = decode_partition(primary_entries)
        if not (
            primary["first_usable_lba"]
            <= partition["first_lba"]
            <= partition["last_lba"]
            <= primary["last_usable_lba"]
        ):
            raise ValueError("structured-store partition is out of GPT bounds")
        expected_partition = {
            "type_guid": str(PARTITION_TYPE_GUID),
            "unique_guid": str(TEST_PARTITION_GUID),
            "first_lba": FIRST_PARTITION_LBA,
            "last_lba": geo["last_usable_lba"],
            "attributes": 0,
            "label": PARTITION_LABEL,
        }
        for field, expected in expected_partition.items():
            if partition[field] != expected:
                raise ValueError(f"structured-store partition {field} identity mismatch")
        if primary_entries[:GPT_ENTRY_SIZE] != partition_entry(geo["last_usable_lba"]):
            raise ValueError("structured-store partition encoding identity mismatch")
        if not partition_is_zero(handle, FIRST_USABLE_LBA, FIRST_PARTITION_LBA - 1):
            raise ValueError("pre-partition test-disk gap is not empty")
        store_empty = partition_is_zero(handle, partition["first_lba"], partition["last_lba"])
        if require_empty and not store_empty:
            raise ValueError("structured-store partition is not empty/unformatted")

    return {
        "schema": "raios.structured_store_test_image.v1",
        "valid": True,
        "disposable_qemu_only": True,
        "path": str(resolved),
        "size_bytes": size_bytes,
        "sector_size": SECTOR_SIZE,
        "disk_guid": str(TEST_DISK_GUID),
        "partition": partition,
        "store_state": "empty_unformatted" if store_empty else "initialized_or_nonempty",
        "checks": [
            "regular_file_not_device",
            "protective_mbr",
            "primary_gpt_header_crc",
            "backup_gpt_header_crc",
            "primary_backup_entry_array_crc",
            "redundant_gpt_match",
            "exact_test_disk_identity",
            "exact_partition_type_label_identity",
            "partition_bounds",
            "empty_pre_partition_gap",
            "empty_unformatted_partition" if store_empty else "partition_content_not_validated",
        ],
    }


def create_image(path: Path, *, size_mib: int, replace: bool) -> dict[str, object]:
    if not MIN_SIZE_MIB <= size_mib <= MAX_SIZE_MIB:
        raise ValueError(f"--size-mib must be between {MIN_SIZE_MIB} and {MAX_SIZE_MIB}")
    resolved = validate_image_path(path, output=True)
    size_bytes = size_mib * 1024 * 1024
    geometry(size_bytes)

    if resolved.exists():
        if not replace:
            raise ValueError("output exists; use --replace-test-image only for this exact test image")
        verify_image(resolved, require_empty=False)
        temporary = resolved.with_name(
            f".{resolved.stem}.{os.getpid()}.tmp{resolved.suffix}"
        )
        if temporary.exists():
            raise ValueError(f"temporary output already exists: {temporary}")
        try:
            with temporary.open("xb+") as handle:
                write_image(handle, size_bytes)
            verify_image(temporary)
            verify_image(resolved, require_empty=False)
            os.replace(temporary, resolved)
        finally:
            if temporary.exists():
                temporary.unlink()
    else:
        created = False
        try:
            with resolved.open("xb+") as handle:
                created = True
                write_image(handle, size_bytes)
            verify_image(resolved)
        except Exception:
            if created and resolved.exists():
                resolved.unlink()
            raise
    return verify_image(resolved)


def rewrite_header_crc(handle, header_lba: int) -> None:
    offset = header_lba * SECTOR_SIZE
    header = bytearray(read_exact(handle, offset, SECTOR_SIZE))
    header[16:20] = b"\0" * 4
    struct.pack_into("<I", header, 16, crc32(header[:GPT_HEADER_SIZE]))
    handle.seek(offset)
    handle.write(header)


def resign_entry_arrays(path: Path, mutation) -> None:
    size_bytes = path.stat().st_size
    geo = geometry(size_bytes)
    with path.open("r+b") as handle:
        for entries_lba, header_lba in (
            (PRIMARY_ENTRIES_LBA, PRIMARY_HEADER_LBA),
            (geo["backup_entries_lba"], geo["backup_header_lba"]),
        ):
            offset = entries_lba * SECTOR_SIZE
            entries = bytearray(read_exact(handle, offset, GPT_ENTRY_ARRAY_BYTES))
            mutation(entries, geo)
            handle.seek(offset)
            handle.write(entries)
            header_offset = header_lba * SECTOR_SIZE
            header = bytearray(read_exact(handle, header_offset, SECTOR_SIZE))
            struct.pack_into("<I", header, 88, crc32(entries))
            handle.seek(header_offset)
            handle.write(header)
            rewrite_header_crc(handle, header_lba)
        handle.flush()
        os.fsync(handle.fileno())


def verify_store_superblock(block: bytes, copy_index: int, partition: dict[str, object]) -> None:
    partition_lba_count = partition["last_lba"] - partition["first_lba"] + 1
    if (
        block[:8] != STORE_SUPERBLOCK_MAGIC
        or struct.unpack_from("<H", block, 8)[0] != 1
        or struct.unpack_from("<H", block, 10)[0] != STORE_SUPERBLOCK_HASH_END
        or block[12] != copy_index
        or block[13] != copy_index
        or any(block[14:16])
        or struct.unpack_from("<I", block, 16)[0] != STORE_BLOCK_LEN
        or any(block[20:24])
        or block[24:40] != STORE_UUID
        or struct.unpack_from("<Q", block, 40)[0] != 1
        or struct.unpack_from("<Q", block, 56)[0] != partition["first_lba"]
        or struct.unpack_from("<Q", block, 64)[0] != partition_lba_count
        or struct.unpack_from("<Q", block, 72)[0] != 2
        or struct.unpack_from("<Q", block, 80)[0] != partition_lba_count - 2
        or any(block[STORE_SUPERBLOCK_HASH_END:])
    ):
        raise ValueError("structured-store superblock identity mismatch")
    canonical = bytearray(block)
    canonical[STORE_SUPERBLOCK_HASH_OFFSET:STORE_SUPERBLOCK_HASH_END] = b"\0" * 32
    if hashlib.sha256(canonical).digest() != block[STORE_SUPERBLOCK_HASH_OFFSET:STORE_SUPERBLOCK_HASH_END]:
        raise ValueError("structured-store superblock hash mismatch")


def inspect_initialized_store(path: Path, inspection: dict[str, object]) -> dict[str, int]:
    if inspection["store_state"] != "initialized_or_nonempty":
        raise ValueError("mutation requires an initialized, nonempty structured-store image")
    partition = inspection["partition"]
    partition_offset = partition["first_lba"] * SECTOR_SIZE
    partition_blocks = partition["last_lba"] - partition["first_lba"] + 1
    previous_hash = bytes(32)
    expected_sequence = 1
    last_frame_offset = None
    zero_tail_started = False

    with path.open("rb") as handle:
        for copy_index in range(2):
            verify_store_superblock(
                read_exact(handle, partition_offset + copy_index * STORE_BLOCK_LEN, STORE_BLOCK_LEN),
                copy_index,
                partition,
            )
        for relative_lba in range(2, partition_blocks):
            offset = partition_offset + relative_lba * STORE_BLOCK_LEN
            frame = read_exact(handle, offset, STORE_BLOCK_LEN)
            if not any(frame):
                zero_tail_started = True
                continue
            if zero_tail_started:
                raise ValueError("structured-store log contains data after its zero tail")
            payload_len = struct.unpack_from("<I", frame, 20)[0]
            if (
                frame[:8] != STORE_FRAME_MAGIC
                or struct.unpack_from("<H", frame, 8)[0] != 1
                or frame[10] not in {1, 2, 3, 4}
                or frame[11] != 0
                or struct.unpack_from("<H", frame, 12)[0] != STORE_FRAME_HEADER_LEN
                or struct.unpack_from("<H", frame, 14)[0] != 0
                or struct.unpack_from("<I", frame, 16)[0] != STORE_BLOCK_LEN
                or payload_len > STORE_BLOCK_LEN - STORE_FRAME_HEADER_LEN
                or struct.unpack_from("<Q", frame, 24)[0] != expected_sequence
                or struct.unpack_from("<Q", frame, 48)[0] != 1
                or frame[56:72] != STORE_UUID
                or frame[104:136] != previous_hash
                or any(frame[STORE_FRAME_HEADER_LEN + payload_len :])
                or struct.unpack_from("<I", frame, 204)[0] != 0
            ):
                raise ValueError("structured-store log frame identity mismatch")
            payload = frame[STORE_FRAME_HEADER_LEN : STORE_FRAME_HEADER_LEN + payload_len]
            if hashlib.sha256(payload).digest() != frame[136:168]:
                raise ValueError("structured-store log frame payload hash mismatch")
            header = bytearray(frame[:STORE_FRAME_HEADER_LEN])
            header[STORE_FRAME_HEADER_HASH_OFFSET:STORE_FRAME_HEADER_HASH_END] = b"\0" * 32
            header[STORE_FRAME_CRC32_OFFSET:STORE_FRAME_CRC32_OFFSET + 4] = b"\0" * 4
            if hashlib.sha256(header).digest() != frame[STORE_FRAME_HEADER_HASH_OFFSET:STORE_FRAME_HEADER_HASH_END]:
                raise ValueError("structured-store log frame header hash mismatch")
            checked = bytearray(frame)
            checked[STORE_FRAME_CRC32_OFFSET:STORE_FRAME_CRC32_OFFSET + 4] = b"\0" * 4
            if crc32(checked) != struct.unpack_from("<I", frame, STORE_FRAME_CRC32_OFFSET)[0]:
                raise ValueError("structured-store log frame CRC mismatch")
            previous_hash = hashlib.sha256(frame).digest()
            expected_sequence += 1
            last_frame_offset = offset

    if last_frame_offset is None:
        raise ValueError("mutation requires initialized structured-store log content")
    return {
        "partition_offset": partition_offset,
        "last_frame_offset": last_frame_offset,
    }


def copy_initialized_test_image(source: Path, output: Path) -> tuple[Path, Path, dict[str, object], dict[str, int]]:
    source = validate_image_path(source, output=False)
    if "structured-store" not in source.name.lower():
        raise ValueError("source filename must contain 'structured-store'")
    inspection = verify_image(source, require_empty=False)
    store = inspect_initialized_store(source, inspection)
    output = validate_image_path(output, output=True)
    if source == output:
        raise ValueError("mutation output must be a new copy, not the source image")
    if output.exists():
        raise ValueError("mutation output already exists; refusing to replace it")
    created = False
    try:
        with source.open("rb") as reader, output.open("xb") as writer:
            created = True
            while chunk := reader.read(READ_CHUNK_SIZE):
                writer.write(chunk)
            writer.flush()
            os.fsync(writer.fileno())
        verify_image(output, require_empty=False)
    except Exception:
        if created and output.exists():
            output.unlink()
        raise
    return source, output, inspection, store


def mutate_partition_guid(source: Path, output: Path) -> dict[str, object]:
    _, output, _, _ = copy_initialized_test_image(source, output)
    geo = geometry(output.stat().st_size)
    try:
        resign_entry_arrays(
            output,
            lambda entries, _geo: entries.__setitem__(
                slice(16, 32), MUTATED_PARTITION_GUID.bytes_le
            ),
        )
        expect_rejected(output, "unique_guid identity")
    except Exception:
        output.unlink(missing_ok=True)
        raise
    return {
        "operation": "change_partition_unique_guid",
        "primary_entry_byte_offset": PRIMARY_ENTRIES_LBA * SECTOR_SIZE + 16,
        "backup_entry_byte_offset": geo["backup_entries_lba"] * SECTOR_SIZE + 16,
        "source_exact_identity": True,
        "mutated_exact_identity": False,
        "gpt_crcs_recomputed": True,
        "initialized_content_preserved": True,
    }


def corrupt_last_log_frame(source: Path, output: Path) -> dict[str, object]:
    _, output, _, store = copy_initialized_test_image(source, output)
    mutation_offset = store["last_frame_offset"] + STORE_FRAME_CRC32_OFFSET
    try:
        with output.open("r+b") as handle:
            original = read_exact(handle, mutation_offset, 1)
            handle.seek(mutation_offset)
            handle.write(bytes([original[0] ^ 1]))
            handle.flush()
            os.fsync(handle.fileno())
        identity = verify_image(output, require_empty=False)
        if identity["store_state"] != "initialized_or_nonempty":
            raise ValueError("corrupted copy lost initialized structured-store content")
    except Exception:
        output.unlink(missing_ok=True)
        raise
    return {
        "operation": "corrupt_last_log_frame",
        "frame_byte_offset": store["last_frame_offset"],
        "mutated_byte_offset": mutation_offset,
        "exact_test_image_identity": True,
        "gpt_unchanged": True,
        "initialized_content_present": True,
    }


def expect_rejected(path: Path, expected: str) -> None:
    try:
        verify_image(path)
    except ValueError as exc:
        if expected.lower() not in str(exc).lower():
            raise ValueError(f"self-test expected {expected!r}, got: {exc}") from exc
        return
    raise ValueError(f"self-test mutation was accepted: {expected}")


def write_selftest_initialized_store(path: Path) -> None:
    inspection = verify_image(path)
    partition = inspection["partition"]
    partition_lba_count = partition["last_lba"] - partition["first_lba"] + 1
    partition_offset = partition["first_lba"] * SECTOR_SIZE

    def superblock(copy_index: int, selection_epoch: int) -> bytes:
        block = bytearray(STORE_BLOCK_LEN)
        block[:8] = STORE_SUPERBLOCK_MAGIC
        struct.pack_into("<HH", block, 8, 1, STORE_SUPERBLOCK_HASH_END)
        block[12] = copy_index
        block[13] = copy_index
        struct.pack_into("<I", block, 16, STORE_BLOCK_LEN)
        block[24:40] = STORE_UUID
        struct.pack_into(
            "<QQQQQQ",
            block,
            40,
            1,
            selection_epoch,
            partition["first_lba"],
            partition_lba_count,
            2,
            partition_lba_count - 2,
        )
        block[STORE_SUPERBLOCK_HASH_OFFSET:STORE_SUPERBLOCK_HASH_END] = hashlib.sha256(block).digest()
        return bytes(block)

    payload = b"nonsecret-selftest"
    frame = bytearray(STORE_BLOCK_LEN)
    frame[:8] = STORE_FRAME_MAGIC
    struct.pack_into("<HBBHHIIQQQ", frame, 8, 1, 1, 0, STORE_FRAME_HEADER_LEN, 0, STORE_BLOCK_LEN, len(payload), 1, 1, 1)
    struct.pack_into("<Q", frame, 48, 1)
    frame[56:72] = STORE_UUID
    frame[72:88] = b"selftest-spacev1"
    frame[88:104] = b"selftest-record!"
    frame[136:168] = hashlib.sha256(payload).digest()
    frame[STORE_FRAME_HEADER_LEN:STORE_FRAME_HEADER_LEN + len(payload)] = payload
    header = bytearray(frame[:STORE_FRAME_HEADER_LEN])
    header[STORE_FRAME_HEADER_HASH_OFFSET:STORE_FRAME_HEADER_HASH_END] = b"\0" * 32
    header[STORE_FRAME_CRC32_OFFSET:STORE_FRAME_CRC32_OFFSET + 4] = b"\0" * 4
    frame[STORE_FRAME_HEADER_HASH_OFFSET:STORE_FRAME_HEADER_HASH_END] = hashlib.sha256(header).digest()
    checked = bytearray(frame)
    checked[STORE_FRAME_CRC32_OFFSET:STORE_FRAME_CRC32_OFFSET + 4] = b"\0" * 4
    struct.pack_into("<I", frame, STORE_FRAME_CRC32_OFFSET, crc32(checked))

    with path.open("r+b") as handle:
        for offset, data in (
            (partition_offset, superblock(0, 1)),
            (partition_offset + STORE_BLOCK_LEN, superblock(1, 2)),
            (partition_offset + 2 * STORE_BLOCK_LEN, frame),
        ):
            handle.seek(offset)
            handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())


def run_self_test() -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="raios-structured-store-") as directory:
        image = Path(directory) / "structured-store-selftest.img"

        create_image(image, size_mib=MIN_SIZE_MIB, replace=False)
        with image.open("r+b") as handle:
            handle.seek(PRIMARY_HEADER_LBA * SECTOR_SIZE + 16)
            handle.write(b"\0\0\0\0")
        expect_rejected(image, "header CRC")

        image.unlink()
        create_image(image, size_mib=MIN_SIZE_MIB, replace=False)
        with image.open("r+b") as handle:
            handle.seek((geometry(image.stat().st_size)["backup_header_lba"]) * SECTOR_SIZE)
            handle.write(b"BROKEN!!")
        expect_rejected(image, "bad signature")

        image.unlink()
        create_image(image, size_mib=MIN_SIZE_MIB, replace=False)
        resign_entry_arrays(
            image,
            lambda entries, _geo: entries.__setitem__(
                slice(16, 32), uuid.UUID("5eedda7a-c0de-4a55-9a15-ffffffffffff").bytes_le
            ),
        )
        expect_rejected(image, "unique_guid identity")

        image.unlink()
        create_image(image, size_mib=MIN_SIZE_MIB, replace=False)
        resign_entry_arrays(
            image,
            lambda entries, geo: struct.pack_into(
                "<Q", entries, 40, geo["last_usable_lba"] + 1
            ),
        )
        expect_rejected(image, "out of GPT bounds")

        image.unlink()
        create_image(image, size_mib=MIN_SIZE_MIB, replace=False)
        with image.open("r+b") as handle:
            handle.seek(FIRST_PARTITION_LBA * SECTOR_SIZE)
            handle.write(b"not-empty")
        expect_rejected(image, "not empty/unformatted")
        inspected = verify_image(image, require_empty=False)
        if inspected["store_state"] != "initialized_or_nonempty":
            raise ValueError("self-test failed to report initialized partition content")
        create_image(image, size_mib=MIN_SIZE_MIB, replace=True)

        refused_output = Path(directory) / "structured-store-empty-refused.img"
        try:
            corrupt_last_log_frame(image, refused_output)
        except ValueError as exc:
            if "initialized, nonempty" not in str(exc):
                raise
        else:
            raise ValueError("empty structured-store mutation source was accepted")

        write_selftest_initialized_store(image)
        initialized = verify_image(image, require_empty=False)
        inspect_initialized_store(image, initialized)

        guid_output = Path(directory) / "structured-store-guid-mutated.img"
        guid_result = mutate_partition_guid(image, guid_output)
        if guid_result["mutated_exact_identity"] is not False:
            raise ValueError("partition-GUID mutation did not reject exact identity")

        frame_output = Path(directory) / "structured-store-frame-corrupt.img"
        frame_result = corrupt_last_log_frame(image, frame_output)
        if frame_result["exact_test_image_identity"] is not True:
            raise ValueError("frame corruption changed the exact GPT identity")
        try:
            inspect_initialized_store(
                frame_output, verify_image(frame_output, require_empty=False)
            )
        except ValueError as exc:
            if "frame CRC mismatch" not in str(exc):
                raise
        else:
            raise ValueError("corrupted structured-store frame passed integrity inspection")

    return {
        "self_test": "passed",
        "checks": [
            "valid_deterministic_image",
            "primary_header_crc_rejection",
            "backup_gpt_rejection",
            "partition_identity_rejection",
            "partition_bounds_rejection",
            "nonempty_store_rejection",
            "initialized_store_inspection",
            "explicit_test_image_replacement",
            "empty_mutation_source_rejection",
            "partition_guid_copy_mutation",
            "last_log_frame_copy_corruption",
        ],
    }


def print_result(result: dict[str, object], *, as_json: bool) -> None:
    if as_json or result.get("operation"):
        print(json.dumps(result, indent=2, sort_keys=True))
        return
    if result.get("self_test"):
        print("structured-store image self-test: passed")
    else:
        print(f"structured-store image: {result['path']}")
        print(f"size: {result['size_bytes']} bytes")
        print(f"disk GUID: {result['disk_guid']}")
        partition = result["partition"]
        print(
            "partition: {label} type={type_guid} unique={unique_guid} "
            "first_lba={first_lba} last_lba={last_lba}".format(**partition)
        )
        print(f"store state: {result['store_state']}")
        print("verification: passed")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Create or verify the deterministic disposable QEMU disk for the raiOS "
            "M13 structured store. Outputs are limited to repo target/ or host temp; "
            "device paths are refused."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser("create", help="create an empty test disk")
    create.add_argument("output", type=Path)
    create.add_argument("--size-mib", type=int, default=DEFAULT_SIZE_MIB)
    create.add_argument(
        "--replace-test-image",
        action="store_true",
        help="replace only an existing image that passes the exact test-image verifier",
    )
    create.add_argument("--json", action="store_true")

    for command, help_text in (
        ("inspect", "inspect GPT identity and report whether partition content is empty"),
        ("verify", "verify the exact empty/unformatted bootstrap test disk"),
    ):
        inspect = subparsers.add_parser(command, help=help_text)
        inspect.add_argument("image", type=Path)
        inspect.add_argument("--json", action="store_true")

    self_test = subparsers.add_parser("selftest", help="run disposable positive/negative checks")
    self_test.add_argument("--json", action="store_true")

    for command, help_text in (
        (
            "mutate-partition-guid",
            "copy an initialized exact test image and change both partition unique GUIDs",
        ),
        (
            "corrupt-last-log-frame",
            "copy an initialized exact test image and corrupt its last log-frame CRC byte",
        ),
    ):
        mutation = subparsers.add_parser(command, help=help_text)
        mutation.add_argument("source", type=Path)
        mutation.add_argument("output", type=Path)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.command == "create":
            result = create_image(
                args.output, size_mib=args.size_mib, replace=args.replace_test_image
            )
        elif args.command in {"inspect", "verify"}:
            result = verify_image(args.image, require_empty=args.command == "verify")
        elif args.command == "mutate-partition-guid":
            result = mutate_partition_guid(args.source, args.output)
        elif args.command == "corrupt-last-log-frame":
            result = corrupt_last_log_frame(args.source, args.output)
        else:
            result = run_self_test()
        print_result(result, as_json=getattr(args, "json", False))
        return 0
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
