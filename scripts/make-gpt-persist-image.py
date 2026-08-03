#!/usr/bin/env python3
"""Build and validate the raiOS M7 GPT persistence test disk."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import struct
import sys
import tempfile
import uuid
from dataclasses import dataclass
from pathlib import Path


def _load_rollback_security():
    helper_path = Path(__file__).with_name("rollback_image_security.py")
    spec = importlib.util.spec_from_file_location("rollback_image_security", helper_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load rollback image security helper: {helper_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ROLLBACK_SECURITY = _load_rollback_security()

SECTOR_SIZE = 512
GPT_ENTRY_SIZE = 128
GPT_ENTRY_COUNT = 128
GPT_ENTRY_ARRAY_BYTES = GPT_ENTRY_SIZE * GPT_ENTRY_COUNT
GPT_ENTRY_ARRAY_LBAS = GPT_ENTRY_ARRAY_BYTES // SECTOR_SIZE
PRIMARY_GPT_HEADER_LBA = 1
PRIMARY_GPT_ENTRIES_LBA = 2
FIRST_USABLE_LBA = 34
FIRST_PARTITION_LBA = 2048

ESP_SLOT_BYTES = 128 * 1024 * 1024
ESP_SLOT_LBA_COUNT = ESP_SLOT_BYTES // SECTOR_SIZE
SEED_DATA_LBA_COUNT = (256 * 1024 * 1024) // SECTOR_SIZE

ESP_A_START_LBA = FIRST_PARTITION_LBA
ESP_B_START_LBA = ESP_A_START_LBA + ESP_SLOT_LBA_COUNT
SEED_DATA_START_LBA = ESP_B_START_LBA + ESP_SLOT_LBA_COUNT
TOTAL_LBA_COUNT = SEED_DATA_START_LBA + SEED_DATA_LBA_COUNT + GPT_ENTRY_ARRAY_LBAS + 1
BACKUP_GPT_HEADER_LBA = TOTAL_LBA_COUNT - 1
BACKUP_GPT_ENTRIES_LBA = BACKUP_GPT_HEADER_LBA - GPT_ENTRY_ARRAY_LBAS
LAST_USABLE_LBA = BACKUP_GPT_ENTRIES_LBA - 1
TOTAL_BYTES = TOTAL_LBA_COUNT * SECTOR_SIZE

ESP_TYPE_GUID = uuid.UUID("c12a7328-f81f-11d2-ba4b-00a0c93ec93b")
SEED_DATA_TYPE_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000001")
DISK_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000000")

SUPERBLOCK_MAGIC = b"RAIOS_DATA_SB_V0"
SUPERBLOCK_VERSION = 0
SUPERBLOCK_HEADER_LEN = 128
SUPERBLOCK_HASH_OFFSET = SUPERBLOCK_HEADER_LEN
SUPERBLOCK_HASH_LEN = 32
SUPERBLOCK_REGION_ENTRY_LEN = 24
SUPERBLOCK_REGION_TABLE_OFFSET = 48

BOOTCTL_START_LBA = 2
BOOTCTL_LBA_COUNT = 8
RECLOG_START_LBA = 16
RECLOG_LBA_COUNT = 4096
ARTSTOR_START_LBA = 8192
ARTSTOR_LBA_COUNT = SEED_DATA_LBA_COUNT - ARTSTOR_START_LBA

BOOTCTL_MAGIC = b"RAIOSBC0"
BOOTCTL_PAYLOAD_MAGIC = b"RAIOSBP0"
BOOT_CONTROL_VERSION = 0
BOOTCTL_SLOT_SIZE = 2048
BOOTCTL_SLOT_HEADER_LEN = 52
BOOTCTL_SLOT_COUNT = 2
BOOTCTL_STORAGE_SLOT_SECTORS = BOOTCTL_SLOT_SIZE // SECTOR_SIZE
BOOTCTL_SLOT_MAX_PAYLOAD_LEN = BOOTCTL_SLOT_SIZE - BOOTCTL_SLOT_HEADER_LEN
BOOTCTL_PAYLOAD_SHA256_OFFSET = 20
BOOTCTL_PAYLOAD_OFFSET = 52
BOOTCTL_PAYLOAD_STRUCT = "<8sIBBBBBBQQBIQBI"
BOOTCTL_PAYLOAD_LEN = struct.calcsize(BOOTCTL_PAYLOAD_STRUCT)
BOOTCTL_PAYLOAD_MAGIC_OFFSET = 0
BOOTCTL_PAYLOAD_VERSION_OFFSET = 8
BOOTCTL_ACTIVE_SLOT_OFFSET = 12
BOOTCTL_LAST_GOOD_SLOT_OFFSET = 13
BOOTCTL_PENDING_SLOT_OFFSET = 14
BOOTCTL_SAFE_MODE_OFFSET = 15
BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET = 16
BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET = 17
BOOTCTL_BOOT_ATTEMPT_GENERATION_OFFSET = 18
BOOTCTL_SLOT_A_GENERATION_OFFSET = 26
BOOTCTL_SLOT_A_STATE_OFFSET = 34
BOOTCTL_SLOT_A_FAILURE_COUNT_OFFSET = 35
BOOTCTL_SLOT_B_GENERATION_OFFSET = 39
BOOTCTL_SLOT_B_STATE_OFFSET = 47
BOOTCTL_SLOT_B_FAILURE_COUNT_OFFSET = 48
MAX_PENDING_BOOT_ATTEMPTS = 3

SLOT_ID_NONE = 0
SLOT_ID_A = 1
SLOT_ID_B = 2

SLOT_STATE_EMPTY = 0
SLOT_STATE_CANDIDATE = 1
SLOT_STATE_PENDING = 2
SLOT_STATE_GOOD = 3
SLOT_STATE_BAD = 4

RECLOG_MAGIC = b"RAIOSRC0"
RECLOG_HEADER_LEN = 88
RECLOG_PREV_FRAME_SHA256_OFFSET = 24
RECLOG_PAYLOAD_SHA256_OFFSET = 56
ARTIFACT_BLOB_MAGIC = b"RAIOSAR0"
ARTIFACT_BLOB_HEADER_LEN = 48
ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET = 16
BUILD_FS_MANIFEST_V1 = b"raios.buildfs_manifest.v1"
BUILD_FS_CHUNK_SIZE = 64 * 1024

REGIONS = (
    ("BOOTCTL", BOOTCTL_START_LBA, BOOTCTL_LBA_COUNT),
    ("RECLOG", RECLOG_START_LBA, RECLOG_LBA_COUNT),
    ("ARTSTOR", ARTSTOR_START_LBA, ARTSTOR_LBA_COUNT),
)


@dataclass(frozen=True)
class Partition:
    name: str
    type_guid: uuid.UUID
    unique_guid: uuid.UUID
    first_lba: int
    lba_count: int

    @property
    def last_lba(self) -> int:
        return self.first_lba + self.lba_count - 1


@dataclass(frozen=True)
class ReclogFixture:
    frame_count: int = 0
    torn_tail: bool = False


@dataclass(frozen=True)
class BuildFsSeedPayload:
    label: str
    sha256: bytes
    payload: bytes


@dataclass(frozen=True)
class BuildFsSeed:
    manifest_sha256: bytes
    payloads: tuple[BuildFsSeedPayload, ...]
    frame_bytes: int


@dataclass(frozen=True)
class BuildFsChunkReference:
    manifest_index: int
    file_index: int
    file_path: str
    chunk_index: int
    sha256: bytes
    length: int


@dataclass(frozen=True)
class BuildFsManifestIndex:
    directory_count: int
    file_count: int
    chunk_references: tuple[BuildFsChunkReference, ...]
    chunk_requirements: dict[bytes, int]


@dataclass(frozen=True)
class DevRollbackSeed:
    artifact: bytes
    reclog: bytes
    artstor: bytes
    metadata: dict[str, object]


@dataclass(frozen=True)
class ResolvedBuildFsChunkFrame:
    frame_offset: int
    frame_len: int
    payload_len: int
    payload_sha256: bytes


PARTITIONS = (
    Partition(
        "SEED_ESP_A",
        ESP_TYPE_GUID,
        uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000a"),
        ESP_A_START_LBA,
        ESP_SLOT_LBA_COUNT,
    ),
    Partition(
        "SEED_ESP_B",
        ESP_TYPE_GUID,
        uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000b"),
        ESP_B_START_LBA,
        ESP_SLOT_LBA_COUNT,
    ),
    Partition(
        "SEED_DATA",
        SEED_DATA_TYPE_GUID,
        uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000d"),
        SEED_DATA_START_LBA,
        SEED_DATA_LBA_COUNT,
    ),
)


def crc32(data: bytes | bytearray) -> int:
    return binascii.crc32(data) & 0xFFFFFFFF


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def assert_not_release_output(path: Path) -> None:
    resolved = path.resolve()
    release = (repo_root() / "release").resolve()
    if is_relative_to(resolved, release):
        raise ValueError(f"refusing to write GPT persist image under release/: {resolved}")


def load_fat32_builder():
    fat32_path = Path(__file__).with_name("make-fat32-image.py")
    spec = importlib.util.spec_from_file_location("make_fat32_image", fat32_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {fat32_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module.Fat32Builder


def parse_reclog_fixture(spec: str | None) -> ReclogFixture:
    if spec is None or spec == "" or spec == "empty":
        return ReclogFixture()

    frame_count: int | None = None
    torn_tail = False
    for raw_part in spec.replace("+", ",").split(","):
        part = raw_part.strip().lower()
        if not part:
            continue
        if part == "torn":
            torn_tail = True
        elif part == "full":
            frame_count = RECLOG_LBA_COUNT
        elif part.isdigit():
            frame_count = int(part)
        elif part.startswith("valid:"):
            frame_count = int(part.split(":", 1)[1])
        elif part.startswith("frames:"):
            frame_count = int(part.split(":", 1)[1])
        else:
            raise ValueError(f"unsupported RECLOG fixture spec: {spec}")

    if frame_count is None:
        raise ValueError(f"RECLOG fixture spec needs a frame count: {spec}")
    if frame_count < 0:
        raise ValueError("RECLOG fixture frame count must not be negative")
    if frame_count > RECLOG_LBA_COUNT:
        raise ValueError(f"RECLOG fixture frame count exceeds region sectors: {frame_count}")
    if torn_tail and frame_count == 0:
        raise ValueError("torn RECLOG fixture needs at least one valid frame")
    if torn_tail and frame_count >= RECLOG_LBA_COUNT:
        raise ValueError("torn RECLOG fixture needs room for the torn sector")
    return ReclogFixture(frame_count=frame_count, torn_tail=torn_tail)


def slot_name(value: int) -> str | None:
    if value == SLOT_ID_NONE:
        return None
    if value == SLOT_ID_A:
        return "A"
    if value == SLOT_ID_B:
        return "B"
    raise ValueError(f"bad BOOTCTL slot id: {value}")


def slot_id(value: str | None) -> int:
    if value is None:
        return SLOT_ID_NONE
    normalized = value.upper()
    if normalized == "A":
        return SLOT_ID_A
    if normalized == "B":
        return SLOT_ID_B
    raise ValueError(f"bad BOOTCTL slot id: {value}")


def slot_state_name(value: int) -> str:
    names = {
        SLOT_STATE_EMPTY: "empty",
        SLOT_STATE_CANDIDATE: "candidate",
        SLOT_STATE_PENDING: "pending",
        SLOT_STATE_GOOD: "good",
        SLOT_STATE_BAD: "bad",
    }
    if value not in names:
        raise ValueError(f"bad BOOTCTL slot state: {value}")
    return names[value]


def slot_state_id(value: str) -> int:
    normalized = value.lower()
    names = {
        "empty": SLOT_STATE_EMPTY,
        "candidate": SLOT_STATE_CANDIDATE,
        "pending": SLOT_STATE_PENDING,
        "good": SLOT_STATE_GOOD,
        "bad": SLOT_STATE_BAD,
    }
    if normalized not in names:
        raise ValueError(f"bad BOOTCTL slot state: {value}")
    return names[normalized]


def normal_boot_control_fields() -> dict[str, object]:
    return {
        "active": "A",
        "last_good": "A",
        "pending": None,
        "safe_mode": False,
        "boot_attempt_slot": "A",
        "boot_attempt_generation": 1,
        "success_marked": True,
        "slot_a_generation": 1,
        "slot_a_state": SLOT_STATE_GOOD,
        "slot_a_failure_count": 0,
        "slot_b_generation": 2,
        "slot_b_state": SLOT_STATE_EMPTY,
        "slot_b_failure_count": 0,
    }


def build_boot_control_payload(fields: dict[str, object]) -> bytes:
    return struct.pack(
        BOOTCTL_PAYLOAD_STRUCT,
        BOOTCTL_PAYLOAD_MAGIC,
        BOOT_CONTROL_VERSION,
        slot_id(fields["active"]),
        slot_id(fields["last_good"]),
        slot_id(fields["pending"]),
        1 if fields["safe_mode"] else 0,
        slot_id(fields["boot_attempt_slot"]),
        1 if fields["success_marked"] else 0,
        int(fields["boot_attempt_generation"]),
        int(fields["slot_a_generation"]),
        int(fields["slot_a_state"]),
        int(fields["slot_a_failure_count"]),
        int(fields["slot_b_generation"]),
        int(fields["slot_b_state"]),
        int(fields["slot_b_failure_count"]),
    )


def build_boot_control_slot(seq: int, fields: dict[str, object]) -> bytes:
    payload = build_boot_control_payload(fields)
    if len(payload) != BOOTCTL_PAYLOAD_LEN:
        raise ValueError(f"BOOTCTL payload length drift: {len(payload)}")
    if len(payload) > BOOTCTL_SLOT_MAX_PAYLOAD_LEN:
        raise ValueError("BOOTCTL payload does not fit in storage slot")
    slot = bytearray(BOOTCTL_SLOT_SIZE)
    slot[0:8] = BOOTCTL_MAGIC
    struct.pack_into("<I", slot, 8, len(payload))
    struct.pack_into("<Q", slot, 12, seq)
    slot[BOOTCTL_PAYLOAD_SHA256_OFFSET : BOOTCTL_PAYLOAD_SHA256_OFFSET + 32] = hashlib.sha256(payload).digest()
    slot[BOOTCTL_PAYLOAD_OFFSET : BOOTCTL_PAYLOAD_OFFSET + len(payload)] = payload
    return bytes(slot)


def boot_control_fields_for_fixture(spec: str) -> tuple[bytes, bytes]:
    normalized = spec.strip().lower()
    empty = b"\0" * BOOTCTL_SLOT_SIZE
    if normalized == "valid-a":
        return build_boot_control_slot(1, normal_boot_control_fields()), empty
    if normalized == "ab":
        fields_a = normal_boot_control_fields()
        fields_b = normal_boot_control_fields()
        fields_b.update(
            {
                "active": "B",
                "last_good": "B",
                "boot_attempt_slot": "B",
                "slot_b_state": SLOT_STATE_GOOD,
            }
        )
        return build_boot_control_slot(5, fields_a), build_boot_control_slot(9, fields_b)
    if normalized == "both-invalid":
        bad_magic = bytearray(build_boot_control_slot(1, normal_boot_control_fields()))
        bad_magic[0] = ord("X")
        bad_hash = bytearray(build_boot_control_slot(2, normal_boot_control_fields()))
        bad_hash[BOOTCTL_PAYLOAD_SHA256_OFFSET] ^= 1
        return bytes(bad_magic), bytes(bad_hash)
    if normalized in {"pending", "pending-safe", "pending-exhausted"}:
        fields = normal_boot_control_fields()
        fields.update(
            {
                "pending": "B",
                "success_marked": False,
                "slot_b_state": SLOT_STATE_PENDING,
            }
        )
        if normalized == "pending-safe":
            fields["safe_mode"] = True
        if normalized == "pending-exhausted":
            fields["slot_b_failure_count"] = MAX_PENDING_BOOT_ATTEMPTS
        return build_boot_control_slot(1, fields), empty
    raise ValueError(f"unsupported BOOTCTL fixture spec: {spec}")


def protective_mbr() -> bytes:
    mbr = bytearray(SECTOR_SIZE)
    mbr[446 + 4] = 0xEE
    struct.pack_into("<I", mbr, 446 + 8, 1)
    struct.pack_into("<I", mbr, 446 + 12, min(TOTAL_LBA_COUNT - 1, 0xFFFFFFFF))
    mbr[510:512] = b"\x55\xAA"
    return bytes(mbr)


def gpt_partition_name(name: str) -> bytes:
    encoded = name.encode("utf-16le")
    if len(encoded) > 72:
        raise ValueError(f"GPT partition name too long: {name}")
    return encoded.ljust(72, b"\0")


def gpt_entry(partition: Partition) -> bytes:
    entry = bytearray(GPT_ENTRY_SIZE)
    entry[0:16] = partition.type_guid.bytes_le
    entry[16:32] = partition.unique_guid.bytes_le
    struct.pack_into("<QQQ", entry, 32, partition.first_lba, partition.last_lba, 0)
    entry[56:128] = gpt_partition_name(partition.name)
    return bytes(entry)


def gpt_entries() -> bytes:
    entries = bytearray(GPT_ENTRY_ARRAY_BYTES)
    for index, partition in enumerate(PARTITIONS):
        start = index * GPT_ENTRY_SIZE
        entries[start : start + GPT_ENTRY_SIZE] = gpt_entry(partition)
    return bytes(entries)


def gpt_header(current_lba: int, backup_lba: int, entries_lba: int, entries_crc: int) -> bytes:
    header_size = 92
    header = bytearray(SECTOR_SIZE)
    header[0:8] = b"EFI PART"
    struct.pack_into("<I", header, 8, 0x00010000)
    struct.pack_into("<I", header, 12, header_size)
    struct.pack_into("<QQQQ", header, 24, current_lba, backup_lba, FIRST_USABLE_LBA, LAST_USABLE_LBA)
    header[56:72] = DISK_GUID.bytes_le
    struct.pack_into("<QIII", header, 72, entries_lba, GPT_ENTRY_COUNT, GPT_ENTRY_SIZE, entries_crc)
    checksum = crc32(header[:header_size])
    struct.pack_into("<I", header, 16, checksum)
    return bytes(header)


def superblock_sector() -> bytes:
    header = bytearray(SUPERBLOCK_HEADER_LEN)
    header[0:16] = SUPERBLOCK_MAGIC
    struct.pack_into(
        "<IIIIQ",
        header,
        16,
        SUPERBLOCK_VERSION,
        SUPERBLOCK_HEADER_LEN,
        len(REGIONS),
        SUPERBLOCK_REGION_ENTRY_LEN,
        SEED_DATA_LBA_COUNT,
    )
    offset = SUPERBLOCK_REGION_TABLE_OFFSET
    for tag, start_lba, lba_count in REGIONS:
        header[offset : offset + 8] = tag.encode("ascii").ljust(8, b"\0")
        struct.pack_into("<QQ", header, offset + 8, start_lba, lba_count)
        offset += SUPERBLOCK_REGION_ENTRY_LEN

    sector = bytearray(SECTOR_SIZE)
    sector[:SUPERBLOCK_HEADER_LEN] = header
    sector[SUPERBLOCK_HASH_OFFSET : SUPERBLOCK_HASH_OFFSET + SUPERBLOCK_HASH_LEN] = hashlib.sha256(
        header
    ).digest()
    return bytes(sector)


def write_at(handle, lba: int, data: bytes) -> None:
    handle.seek(lba * SECTOR_SIZE)
    handle.write(data)


def reclog_payload(seq: int) -> bytes:
    return f'{{\r\n  "fixture": "m7b-reclog",\r\n  "seq": {seq}\r\n}}'.encode("ascii")


def build_reclog_frame(seq: int, prev_frame_sha256: bytes, payload: bytes, frame_len: int = SECTOR_SIZE) -> bytes:
    if len(prev_frame_sha256) != 32:
        raise ValueError("prev_frame_sha256 must be 32 bytes")
    if frame_len % SECTOR_SIZE != 0 or frame_len < SECTOR_SIZE:
        raise ValueError("RECLOG frame_len must be a non-zero sector multiple")
    if len(payload) > frame_len - RECLOG_HEADER_LEN:
        raise ValueError("RECLOG payload does not fit in frame")

    frame = bytearray(frame_len)
    frame[0:8] = RECLOG_MAGIC
    struct.pack_into("<IIQ", frame, 8, frame_len, len(payload), seq)
    frame[RECLOG_PREV_FRAME_SHA256_OFFSET : RECLOG_PREV_FRAME_SHA256_OFFSET + 32] = prev_frame_sha256
    frame[RECLOG_PAYLOAD_SHA256_OFFSET : RECLOG_PAYLOAD_SHA256_OFFSET + 32] = hashlib.sha256(payload).digest()
    frame[RECLOG_HEADER_LEN : RECLOG_HEADER_LEN + len(payload)] = payload
    return bytes(frame)


def build_artifact_blob_frame(payload: bytes, frame_len: int = SECTOR_SIZE) -> bytes:
    if frame_len % SECTOR_SIZE != 0 or frame_len < SECTOR_SIZE:
        raise ValueError("ARTSTOR blob frame_len must be a non-zero sector multiple")
    if len(payload) > frame_len - ARTIFACT_BLOB_HEADER_LEN:
        raise ValueError("ARTSTOR payload does not fit in frame")
    frame = bytearray(frame_len)
    frame[0:8] = ARTIFACT_BLOB_MAGIC
    struct.pack_into("<II", frame, 8, frame_len, len(payload))
    frame[ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET : ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET + 32] = hashlib.sha256(payload).digest()
    frame[ARTIFACT_BLOB_HEADER_LEN : ARTIFACT_BLOB_HEADER_LEN + len(payload)] = payload
    return bytes(frame)


def parse_required_sha256(value: str, option: str) -> bytes:
    if len(value) != 64 or any(character not in "0123456789abcdefABCDEF" for character in value):
        raise ValueError(f"{option} must be exactly 64 hexadecimal digits")
    return bytes.fromhex(value)


def artifact_blob_frame_len(payload_len: int) -> int:
    if payload_len < 0:
        raise ValueError("ARTSTOR payload length must not be negative")
    unaligned = ARTIFACT_BLOB_HEADER_LEN + payload_len
    return ((unaligned + SECTOR_SIZE - 1) // SECTOR_SIZE) * SECTOR_SIZE


def parse_buildfs_manifest_index(manifest: bytes) -> BuildFsManifestIndex:
    offset = 0

    def take(length: int, field: str) -> bytes:
        nonlocal offset
        if length < 0 or offset + length > len(manifest):
            raise ValueError(f"BuildFS manifest truncated at {field}")
        value = manifest[offset : offset + length]
        offset += length
        return value

    def take_u64(field: str) -> int:
        return struct.unpack("<Q", take(8, field))[0]

    def take_bytes(field: str) -> bytes:
        length = take_u64(f"{field}.len")
        if length > len(manifest) - offset:
            raise ValueError(f"BuildFS manifest {field} length exceeds remaining bytes")
        return take(length, field)

    def take_path(field: str) -> str:
        raw = take_bytes(field)
        try:
            path = raw.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError(f"BuildFS manifest {field} is not UTF-8") from exc
        if path.startswith("/") or not path:
            raise ValueError(f"BuildFS manifest {field} has an invalid path")
        components = path.split("/")
        if any(component in {"", ".", ".."} for component in components):
            raise ValueError(f"BuildFS manifest {field} has an invalid component")
        return path

    domain = take_bytes("domain")
    if domain != BUILD_FS_MANIFEST_V1:
        raise ValueError("BuildFS manifest domain mismatch")
    if take_u64("chunk_size") != BUILD_FS_CHUNK_SIZE:
        raise ValueError("BuildFS manifest chunk size mismatch")

    directory_count = take_u64("directory_count")
    if directory_count > len(manifest):
        raise ValueError("BuildFS manifest directory count is impossible")
    directories = [take_path(f"directory[{index}]") for index in range(directory_count)]
    if directories != sorted(directories, key=lambda path: path.encode("utf-8")):
        raise ValueError("BuildFS manifest directories are not canonically ordered")
    if len(set(directories)) != len(directories):
        raise ValueError("BuildFS manifest contains duplicate directories")
    directory_set = set(directories)
    for path in directories:
        if "/" in path and path.rsplit("/", 1)[0] not in directory_set:
            raise ValueError(f"BuildFS manifest directory parent is missing: {path}")

    file_count = take_u64("file_count")
    if file_count > len(manifest):
        raise ValueError("BuildFS manifest file count is impossible")
    file_paths: list[str] = []
    chunk_references: list[BuildFsChunkReference] = []
    chunk_requirements: dict[bytes, int] = {}
    for file_index in range(file_count):
        path = take_path(f"file[{file_index}].path")
        if path in directory_set:
            raise ValueError(f"BuildFS manifest path is both file and directory: {path}")
        if "/" in path and path.rsplit("/", 1)[0] not in directory_set:
            raise ValueError(f"BuildFS manifest file parent is missing: {path}")
        file_paths.append(path)
        file_len = take_u64(f"file[{file_index}].len")
        take(32, f"file[{file_index}].sha256")
        chunk_count = take_u64(f"file[{file_index}].chunk_count")
        expected_chunk_count = 0 if file_len == 0 else ((file_len - 1) // BUILD_FS_CHUNK_SIZE) + 1
        if chunk_count != expected_chunk_count:
            raise ValueError(f"BuildFS manifest chunk count mismatch for {path}")
        chunk_total = 0
        for chunk_index in range(chunk_count):
            chunk_len = take_u64(f"file[{file_index}].chunk[{chunk_index}].len")
            chunk_sha256 = take(32, f"file[{file_index}].chunk[{chunk_index}].sha256")
            is_last = chunk_index + 1 == chunk_count
            expected_len = (
                file_len % BUILD_FS_CHUNK_SIZE or BUILD_FS_CHUNK_SIZE
                if is_last
                else BUILD_FS_CHUNK_SIZE
            )
            if chunk_len != expected_len:
                raise ValueError(f"BuildFS manifest chunk length mismatch for {path}")
            chunk_total += chunk_len
            previous_len = chunk_requirements.setdefault(chunk_sha256, chunk_len)
            if previous_len != chunk_len:
                raise ValueError("BuildFS manifest reuses a chunk digest with a different length")
            chunk_references.append(
                BuildFsChunkReference(
                    manifest_index=len(chunk_references),
                    file_index=file_index,
                    file_path=path,
                    chunk_index=chunk_index,
                    sha256=chunk_sha256,
                    length=chunk_len,
                )
            )
        if chunk_total != file_len:
            raise ValueError(f"BuildFS manifest file length mismatch for {path}")

    if file_paths != sorted(file_paths, key=lambda path: path.encode("utf-8")):
        raise ValueError("BuildFS manifest files are not canonically ordered")
    if len(set(file_paths)) != len(file_paths):
        raise ValueError("BuildFS manifest contains duplicate files")
    if offset != len(manifest):
        raise ValueError("BuildFS manifest has trailing bytes")
    return BuildFsManifestIndex(
        directory_count=directory_count,
        file_count=file_count,
        chunk_references=tuple(chunk_references),
        chunk_requirements=chunk_requirements,
    )


def parse_buildfs_manifest(manifest: bytes) -> dict[bytes, int]:
    return parse_buildfs_manifest_index(manifest).chunk_requirements


def ensure_artstor_seed_fits(frame_bytes: int) -> None:
    region_bytes = ARTSTOR_LBA_COUNT * SECTOR_SIZE
    required_bytes = frame_bytes + SECTOR_SIZE
    if required_bytes > region_bytes:
        raise ValueError(
            "BuildFS frames plus trailing zero sector exceed ARTSTOR region: "
            f"need {required_bytes} bytes ({frame_bytes} frame bytes + {SECTOR_SIZE} terminator), "
            f"have {region_bytes}"
        )


def prepare_buildfs_seed(buildfs_directory: Path, expected_manifest_sha256: str) -> BuildFsSeed:
    expected = parse_required_sha256(expected_manifest_sha256, "--expect-manifest-sha256")
    if buildfs_directory.is_symlink() or not buildfs_directory.is_dir():
        raise ValueError(f"BuildFS seed path is not a plain directory: {buildfs_directory}")
    manifest_path = buildfs_directory / "manifest.bin"
    chunks_directory = buildfs_directory / "chunks"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ValueError(f"BuildFS manifest.bin is missing: {manifest_path}")
    if chunks_directory.is_symlink() or not chunks_directory.is_dir():
        raise ValueError(f"BuildFS chunks directory is missing: {chunks_directory}")

    manifest = manifest_path.read_bytes()
    actual_manifest_sha256 = hashlib.sha256(manifest).digest()
    if actual_manifest_sha256 != expected:
        raise ValueError(
            "BuildFS manifest SHA-256 mismatch: "
            f"expected {expected.hex()}, got {actual_manifest_sha256.hex()}"
        )
    chunk_requirements = parse_buildfs_manifest(manifest)

    manifest_sha_path = buildfs_directory / "manifest.sha256"
    if manifest_sha_path.exists():
        if manifest_sha_path.is_symlink() or not manifest_sha_path.is_file():
            raise ValueError("BuildFS manifest.sha256 is not a plain file")
        recorded_manifest_sha = manifest_sha_path.read_text(encoding="ascii").strip()
        if recorded_manifest_sha.lower() != expected.hex():
            raise ValueError("BuildFS manifest.sha256 does not match the required pin")

    chunk_entries = list(chunks_directory.iterdir())
    expected_names = {digest.hex() for digest in chunk_requirements}
    actual_names = {entry.name for entry in chunk_entries}
    if actual_names != expected_names:
        missing = sorted(expected_names - actual_names)
        extra = sorted(actual_names - expected_names)
        raise ValueError(f"BuildFS chunk set mismatch: missing={missing[:3]} extra={extra[:3]}")

    payloads: list[BuildFsSeedPayload] = []
    for digest in sorted(chunk_requirements):
        chunk_path = chunks_directory / digest.hex()
        if chunk_path.is_symlink() or not chunk_path.is_file():
            raise ValueError(f"BuildFS chunk is not a plain file: {chunk_path}")
        payload = chunk_path.read_bytes()
        if len(payload) != chunk_requirements[digest]:
            raise ValueError(f"BuildFS chunk length mismatch: {chunk_path.name}")
        actual = hashlib.sha256(payload).digest()
        if actual != digest:
            raise ValueError(
                f"BuildFS chunk SHA-256 mismatch for {chunk_path.name}: got {actual.hex()}"
            )
        payloads.append(BuildFsSeedPayload(f"chunk:{digest.hex()}", digest, payload))

    payloads.append(BuildFsSeedPayload("manifest.bin", expected, manifest))
    frame_bytes = sum(artifact_blob_frame_len(len(entry.payload)) for entry in payloads)
    ensure_artstor_seed_fits(frame_bytes)
    return BuildFsSeed(expected, tuple(payloads), frame_bytes)


def prepare_buildfs_seeds(
    seed_pairs: tuple[tuple[Path, str], ...],
) -> tuple[BuildFsSeed, ...]:
    if not seed_pairs:
        raise ValueError("at least one paired BuildFS seed is required")
    seeds = tuple(
        prepare_buildfs_seed(buildfs_directory, expected_manifest_sha256)
        for buildfs_directory, expected_manifest_sha256 in seed_pairs
    )
    ensure_artstor_seed_fits(sum(seed.frame_bytes for seed in seeds))
    return seeds


def seed_buildfs(handle, seed: BuildFsSeed) -> None:
    seed_buildfs_trees(handle, (seed,))


def seed_buildfs_trees(handle, seeds: tuple[BuildFsSeed, ...]) -> None:
    if not seeds:
        raise ValueError("at least one prepared BuildFS seed is required")
    ensure_artstor_seed_fits(sum(seed.frame_bytes for seed in seeds))
    handle.seek((SEED_DATA_START_LBA + ARTSTOR_START_LBA) * SECTOR_SIZE)
    for tree_index, seed in enumerate(seeds):
        for entry in seed.payloads:
            if hashlib.sha256(entry.payload).digest() != entry.sha256:
                raise ValueError(
                    f"BuildFS tree {tree_index + 1} payload changed after preflight: {entry.label}"
                )
            handle.write(
                build_artifact_blob_frame(entry.payload, artifact_blob_frame_len(len(entry.payload)))
            )
    handle.write(b"\0" * SECTOR_SIZE)


def validate_buildfs_seed_image(image: Path, seed: BuildFsSeed) -> None:
    validate_buildfs_seed_trees_image(image, (seed,))


def validate_buildfs_seed_trees_image(image: Path, seeds: tuple[BuildFsSeed, ...]) -> None:
    if not seeds:
        raise ValueError("at least one prepared BuildFS seed is required")
    expected_frame_bytes = sum(seed.frame_bytes for seed in seeds)
    ensure_artstor_seed_fits(expected_frame_bytes)
    offset = 0
    with image.open("rb") as handle:
        for tree_index, seed in enumerate(seeds):
            tree_start = offset
            for entry in seed.payloads:
                frame_len = artifact_blob_frame_len(len(entry.payload))
                frame = read_artstor_blob(handle, SEED_DATA_START_LBA, offset, frame_len)
                parsed, reason = parse_artifact_blob_frame(frame, 0)
                if parsed is None:
                    raise ValueError(
                        f"BuildFS tree {tree_index + 1} ARTSTOR frame failed offline parse "
                        f"at {offset}: {reason}"
                    )
                payload = frame[
                    ARTIFACT_BLOB_HEADER_LEN : ARTIFACT_BLOB_HEADER_LEN + len(entry.payload)
                ]
                if (
                    int(parsed["frame_len"]) != frame_len
                    or int(parsed["payload_len"]) != len(entry.payload)
                    or parsed["payload_sha256"] != entry.sha256.hex()
                    or hashlib.sha256(payload).digest() != entry.sha256
                ):
                    raise ValueError(
                        f"BuildFS tree {tree_index + 1} ARTSTOR frame binding mismatch: "
                        f"{entry.label}"
                    )
                offset += frame_len
            if offset - tree_start != seed.frame_bytes:
                raise ValueError(f"BuildFS tree {tree_index + 1} ARTSTOR frame span drift")
        if offset != expected_frame_bytes:
            raise ValueError("BuildFS ARTSTOR frame span drift")
        handle.seek((SEED_DATA_START_LBA + ARTSTOR_START_LBA) * SECTOR_SIZE + offset)
        terminator = handle.read(SECTOR_SIZE)
        if len(terminator) != SECTOR_SIZE or not all_zero(terminator):
            raise ValueError("BuildFS ARTSTOR frames are not followed by one zero sector")


def seed_reclog_fixture(handle, fixture: ReclogFixture) -> None:
    if fixture.frame_count == 0 and not fixture.torn_tail:
        return

    handle.seek((SEED_DATA_START_LBA + RECLOG_START_LBA) * SECTOR_SIZE)
    prev_hash = b"\0" * 32
    for seq in range(1, fixture.frame_count + 1):
        frame = build_reclog_frame(seq, prev_hash, reclog_payload(seq))
        handle.write(frame)
        prev_hash = hashlib.sha256(frame).digest()
    if fixture.torn_tail:
        handle.write(b"TORNTAIL" + (b"\xA5" * (SECTOR_SIZE - 8)))


def seed_artstor_garbage_blob(handle) -> None:
    frame = build_artifact_blob_frame(b"bare-artstor-blob-without-reclog-record")
    write_at(handle, SEED_DATA_START_LBA + ARTSTOR_START_LBA, frame)


def validate_dev_rollback_bundle(seed: DevRollbackSeed, expected_artifact: bytes) -> DevRollbackSeed:
    try:
        verified = ROLLBACK_SECURITY.verify_fixture(
            expected_artifact, seed.reclog, seed.artstor
        )
    except ROLLBACK_SECURITY.SecurityError as exc:
        raise ValueError(f"dev rollback fixture rejected: {exc.code}") from exc
    if seed.artifact != expected_artifact:
        raise ValueError("dev rollback fixture artifact differs from requested artifact")
    trusted = {
        "artifact_sha256": f"sha256:{verified.artifact_sha256.hex()}",
        "reclog_tail_sha256": f"sha256:{verified.reclog_tail_sha256.hex()}",
        "artstor_frame_sha256": f"sha256:{verified.artstor_frame_sha256.hex()}",
    }
    return DevRollbackSeed(seed.artifact, seed.reclog, seed.artstor, trusted)

def prepare_dev_rollback_seed(tool: Path, artifact: Path) -> DevRollbackSeed:
    if not tool.is_file():
        raise ValueError(f"dev rollback fixture tool is missing: {tool}")
    if not artifact.is_file():
        raise ValueError(f"dev rollback fixture artifact is missing: {artifact}")
    expected_artifact = artifact.read_bytes()
    if len(expected_artifact) != 121 or hashlib.sha256(expected_artifact).hexdigest() != "33ea9dc8f8ecd039236673fafdffcf63e8691a1d352cffee4ddf47531cb5756c":
        raise ValueError("dev rollback fixture input is not the pinned 121-byte Wasm")
    root = Path(tempfile.mkdtemp(prefix="raios-dev-rollback-seed-"))
    try:
        generated = subprocess.run(
            [str(tool), "generate", str(artifact), str(root)],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if generated.returncode != 0:
            raise ValueError(f"dev rollback fixture generation failed: exit {generated.returncode}")
        sizes = {name: (root / name).stat().st_size for name in ("artifact.wasm", "reclog.bin", "artstor.bin")}
        if sizes["artifact.wasm"] != 121 or not (0 < sizes["reclog.bin"] <= 64 * 1024) or sizes["artstor.bin"] != SECTOR_SIZE:
            raise ValueError("dev rollback fixture producer output exceeds exact bounded sizes")
        seed = DevRollbackSeed(
            artifact=(root / "artifact.wasm").read_bytes(),
            reclog=(root / "reclog.bin").read_bytes(),
            artstor=(root / "artstor.bin").read_bytes(),
            metadata={},
        )
    finally:
        shutil.rmtree(root)
    return validate_dev_rollback_bundle(seed, expected_artifact)


def seed_dev_rollback_fixture(handle, seed: DevRollbackSeed) -> None:
    reclog_capacity = RECLOG_LBA_COUNT * SECTOR_SIZE
    artstor_capacity = ARTSTOR_LBA_COUNT * SECTOR_SIZE
    if len(seed.reclog) > reclog_capacity or len(seed.artstor) > artstor_capacity:
        raise ValueError("dev rollback fixture exceeds persistence region")
    write_at(handle, SEED_DATA_START_LBA + RECLOG_START_LBA, seed.reclog)
    write_at(handle, SEED_DATA_START_LBA + ARTSTOR_START_LBA, seed.artstor)


def validate_dev_rollback_seed_image(image: Path, seed: DevRollbackSeed) -> None:
    with image.open("rb") as handle:
        reclog = read_at(
            handle,
            SEED_DATA_START_LBA + RECLOG_START_LBA,
            len(seed.reclog),
        )
        artstor = read_at(
            handle,
            SEED_DATA_START_LBA + ARTSTOR_START_LBA,
            len(seed.artstor),
        )
    if reclog != seed.reclog or artstor != seed.artstor:
        raise ValueError("dev rollback fixture image bytes differ from independently verified bundle")
    validate_dev_rollback_bundle(DevRollbackSeed(seed.artifact, reclog, artstor, {}), seed.artifact)


def seed_bootctl_fixture(handle, spec: str | None) -> None:
    if spec is None or spec == "" or spec.strip().lower() in {"none", "empty"}:
        return
    storage_slot_a, storage_slot_b = boot_control_fields_for_fixture(spec)
    if len(storage_slot_a) != BOOTCTL_SLOT_SIZE or len(storage_slot_b) != BOOTCTL_SLOT_SIZE:
        raise ValueError("BOOTCTL fixture slot length drift")
    handle.seek((SEED_DATA_START_LBA + BOOTCTL_START_LBA) * SECTOR_SIZE)
    handle.write(storage_slot_a)
    handle.write(storage_slot_b)


def boot_control_record_to_fields(record: dict[str, object] | None) -> dict[str, object]:
    if record is None:
        return normal_boot_control_fields()
    slots = record["slots"]
    boot_attempt = record["boot_attempt"]
    return {
        "active": record["active"],
        "last_good": record["last_good"],
        "pending": record["pending"],
        "safe_mode": bool(record["safe_mode"]),
        "boot_attempt_slot": boot_attempt["slot"],
        "boot_attempt_generation": int(boot_attempt["generation"]),
        "success_marked": bool(boot_attempt["success_marked"]),
        "slot_a_generation": int(slots["A"]["generation"]),
        "slot_a_state": slot_state_id(str(slots["A"]["state"])),
        "slot_a_failure_count": int(slots["A"]["failure_count"]),
        "slot_b_generation": int(slots["B"]["generation"]),
        "slot_b_state": slot_state_id(str(slots["B"]["state"])),
        "slot_b_failure_count": int(slots["B"]["failure_count"]),
    }


def boot_control_authority(scan: dict[str, object]) -> tuple[str | None, dict[str, object] | None, int, str]:
    decision = scan["decision"]
    authoritative = decision.get("authoritative_bootctl_slot")
    if authoritative in {"A", "B"}:
        record = scan[f"storage_slot_{str(authoritative).lower()}"]
        return str(authoritative), record, int(record["seq"]), "B" if authoritative == "A" else "A"
    reason = str(decision.get("reason"))
    if reason in {"no_valid_boot_control_slot", "both_slots_invalid"}:
        return None, None, 0, "A"
    raise ValueError(f"cannot choose loser BOOTCTL slot without an authoritative record: {reason}")


def payload_files(payload_dir: Path) -> list[tuple[str, Path]]:
    root = payload_dir.resolve()
    if not root.is_dir():
        raise ValueError(f"payload directory does not exist: {payload_dir}")
    files: list[tuple[str, Path]] = []
    for source in sorted(path for path in root.rglob("*") if path.is_file()):
        image_path = source.relative_to(root).as_posix()
        files.append((image_path, source))
    if not files:
        raise ValueError(f"payload directory is empty: {payload_dir}")
    return files


def build_payload_esp(payload_dir: Path) -> tuple[bytes, list[str]]:
    builder = load_fat32_builder()(ESP_SLOT_BYTES)
    staged: list[str] = []
    for image_path, source in payload_files(payload_dir):
        builder.add_file(image_path, source.read_bytes())
        staged.append(image_path)
    return builder.build(), staged


class _BoundedImageGpt:
    def __init__(self, image: Path):
        self.image = image
        self.size = image.stat().st_size

    def read_at(self, offset: int, length: int) -> bytes:
        with self.image.open("rb") as handle:
            handle.seek(offset)
            return handle.read(length)


def validated_seed_data_lba(image: Path) -> int:
    try:
        return ROLLBACK_SECURITY.validate_gpt(_BoundedImageGpt(image)).first_lba
    except (OSError, ROLLBACK_SECURITY.SecurityError) as exc:
        code = getattr(exc, "code", "gpt_source")
        raise ValueError(f"refusing invalid GPT image: {code}") from exc

def validate_existing_gpt_image(image: Path) -> dict[str, object]:
    seed_data_lba = validated_seed_data_lba(image)
    info = inspect_image(image)
    if not info.get("data_superblock_valid"):
        raise ValueError("refusing image with invalid SEED_DATA superblock")
    info["validated_seed_data_first_lba"] = seed_data_lba
    return info


def prepare_pending_fields(record: dict[str, object] | None, target_slot: str) -> dict[str, object]:
    fields = boot_control_record_to_fields(record)
    slot_prefix = "slot_a" if target_slot == "A" else "slot_b"
    fields["pending"] = target_slot
    fields["safe_mode"] = False
    fields["boot_attempt_slot"] = target_slot
    fields["boot_attempt_generation"] = int(fields[f"{slot_prefix}_generation"])
    fields["success_marked"] = False
    fields[f"{slot_prefix}_state"] = SLOT_STATE_PENDING
    fields[f"{slot_prefix}_failure_count"] = 0
    return fields


def bootctl_slot_relative_lba(storage_slot: str) -> int:
    return BOOTCTL_START_LBA + (0 if storage_slot == "A" else BOOTCTL_STORAGE_SLOT_SECTORS)


def esp_start_lba(payload_slot: str) -> int:
    return ESP_A_START_LBA if payload_slot == "A" else ESP_B_START_LBA


def apply_boot_slot_update(
    image: Path,
    stage_slot: str | None,
    payload_dir: Path | None,
    set_pending: str | None,
) -> dict[str, object]:
    assert_not_release_output(image)
    if not image.exists():
        raise FileNotFoundError(image)
    if stage_slot is not None and payload_dir is None:
        raise ValueError("--stage-slot requires --payload-dir")
    if payload_dir is not None and stage_slot is None:
        raise ValueError("--payload-dir requires --stage-slot")
    if stage_slot and set_pending and stage_slot != set_pending:
        raise ValueError("--stage-slot and --set-pending must name the same slot when combined")
    target_slot = stage_slot or set_pending
    if target_slot is None:
        raise ValueError("--stage-slot or --set-pending is required")

    pre = validate_existing_gpt_image(image)
    authoritative, record, authoritative_seq, loser = boot_control_authority(pre["bootctl_read"])
    new_seq = authoritative_seq + 1
    fields = prepare_pending_fields(record, target_slot)
    slot_bytes = build_boot_control_slot(new_seq, fields)
    payload_image = None
    staged_files: list[str] = []
    if payload_dir is not None:
        payload_image, staged_files = build_payload_esp(payload_dir)

    plan = {
        "image": str(image),
        "target_payload_slot": target_slot,
        "stage_payload": payload_dir is not None,
        "payload_dir": None if payload_dir is None else str(payload_dir),
        "payload_files": staged_files,
        "esp_start_lba": None if payload_image is None else esp_start_lba(target_slot),
        "authoritative_storage_slot": authoritative,
        "authoritative_seq": authoritative_seq,
        "written_storage_slot": loser,
        "new_seq": new_seq,
        "bootctl_relative_lba": bootctl_slot_relative_lba(loser),
        "bootctl_absolute_lba": SEED_DATA_START_LBA + bootctl_slot_relative_lba(loser),
        "pending": target_slot,
        "success_marked": False,
        "slot_state": "pending",
    }

    with image.open("r+b") as handle:
        if payload_image is not None:
            write_at(handle, esp_start_lba(target_slot), payload_image)
        write_at(handle, SEED_DATA_START_LBA + bootctl_slot_relative_lba(loser), slot_bytes)
        handle.flush()
        os.fsync(handle.fileno())

    post = inspect_image(image)
    post_bootctl = post["bootctl_read"]
    post_decision = post_bootctl["decision"]
    winner = post_bootctl[f"storage_slot_{loser.lower()}"]
    if (
        post_decision.get("authoritative_bootctl_slot") != loser
        or int(post_decision.get("seq") or 0) != new_seq
        or winner.get("pending") != target_slot
        or winner["slots"][target_slot]["state"] != "pending"
        or bool(winner["boot_attempt"]["success_marked"])
    ):
        raise ValueError("post-write BOOTCTL scan did not find the new pending record as winner")
    return {"operation": "stage_slot" if stage_slot else "set_pending", "plan": plan, "post_bootctl_read": post_bootctl}


def seed_data_first_lba_from_info(info: dict[str, object]) -> int:
    if "validated_seed_data_first_lba" not in info:
        raise ValueError("SEED_DATA LBA was not obtained from full GPT validation")
    return int(info["validated_seed_data_first_lba"])


def read_reclog_region(handle, seed_data_first_lba: int) -> bytes:
    handle.seek((seed_data_first_lba + RECLOG_START_LBA) * SECTOR_SIZE)
    data = handle.read(RECLOG_LBA_COUNT * SECTOR_SIZE)
    if len(data) != RECLOG_LBA_COUNT * SECTOR_SIZE:
        raise ValueError("short RECLOG read")
    return data


def parse_reclog_frames_for_inspection(data: bytes) -> list[dict[str, object]]:
    frames: list[dict[str, object]] = []
    offset = 0
    expected_seq = 1
    expected_prev_hash = b"\0" * 32
    while offset < len(data):
        if all_zero(data[offset:]):
            break
        parsed, reason = parse_reclog_frame(data, offset, expected_seq, expected_prev_hash)
        if parsed is None:
            raise ValueError(f"RECLOG frame parse failed at offset {offset}: {reason}")
        frame_len = int(parsed["frame_len"])
        payload_len = int(parsed["payload_len"])
        payload_start = offset + RECLOG_HEADER_LEN
        payload = data[payload_start : payload_start + payload_len]
        frames.append(
            {
                **parsed,
                "payload": payload,
                "frame": data[offset : offset + frame_len],
            }
        )
        expected_seq += 1
        expected_prev_hash = bytes.fromhex(str(parsed["frame_sha256"]))
        offset += frame_len
    return frames


def payload_json(payload: bytes) -> dict[str, object] | None:
    try:
        return json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return None


def reclog_frame_summary(frame: dict[str, object]) -> dict[str, object]:
    payload = frame["payload"]
    if not isinstance(payload, bytes):
        raise TypeError("RECLOG frame payload must be bytes")
    try:
        payload_text: str | None = payload.decode("utf-8")
    except UnicodeDecodeError:
        payload_text = None
    return {
        "offset": frame["offset"],
        "frame_len": frame["frame_len"],
        "payload_len": frame["payload_len"],
        "seq": frame["seq"],
        "frame_sha256": frame["frame_sha256"],
        "payload_sha256": frame["payload_sha256"],
        "payload_json": payload_json(payload),
        "payload_text": payload_text,
    }


def read_artstor_blob(handle, seed_data_first_lba: int, offset: int, frame_len: int) -> bytes:
    if offset < 0 or frame_len <= 0:
        raise ValueError("bad ARTSTOR blob range")
    if offset + frame_len > ARTSTOR_LBA_COUNT * SECTOR_SIZE:
        raise ValueError("ARTSTOR blob range out of bounds")
    handle.seek((seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE + offset)
    blob = handle.read(frame_len)
    if len(blob) != frame_len:
        raise ValueError("short ARTSTOR blob read")
    return blob


def resolve_buildfs_chunk_frame_from_image(
    handle,
    seed_data_first_lba: int,
    payload_sha256: bytes,
    payload_len: int,
) -> tuple[ResolvedBuildFsChunkFrame | None, str | None, int | None]:
    """Mirror seed-kernel resolve_buildfs_chunk_frame over an image handle."""
    region_bytes = ARTSTOR_LBA_COUNT * SECTOR_SIZE
    region_start = (seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE
    offset = 0
    while offset < region_bytes:
        handle.seek(region_start + offset)
        header = handle.read(SECTOR_SIZE)
        if len(header) != SECTOR_SIZE:
            raise ValueError(f"short ARTSTOR scan-window read at offset {offset}")
        if all_zero(header):
            return None, "Missing", None
        if header[: len(ARTIFACT_BLOB_MAGIC)] != ARTIFACT_BLOB_MAGIC:
            raise ValueError(f"malformed ARTSTOR magic at offset {offset}")
        frame_len = struct.unpack_from("<I", header, 8)[0]
        if (
            frame_len < ARTIFACT_BLOB_HEADER_LEN
            or frame_len % SECTOR_SIZE != 0
            or offset + frame_len > region_bytes
        ):
            raise ValueError(f"invalid ARTSTOR frame bounds at offset {offset}: frame_len={frame_len}")
        handle.seek(region_start + offset)
        frame = handle.read(frame_len)
        if len(frame) != frame_len:
            raise ValueError(f"short ARTSTOR frame read at offset {offset}: frame_len={frame_len}")
        parsed, reason = parse_artifact_blob_frame(frame, 0)
        if parsed is None:
            raise ValueError(f"malformed ARTSTOR frame at offset {offset}: {reason}")
        parsed_sha256 = bytes.fromhex(str(parsed["payload_sha256"]))
        parsed_payload_len = int(parsed["payload_len"])
        if parsed_sha256 == payload_sha256:
            if parsed_payload_len != payload_len:
                return None, "LengthMismatch", parsed_payload_len
            return (
                ResolvedBuildFsChunkFrame(
                    frame_offset=offset,
                    frame_len=frame_len,
                    payload_len=parsed_payload_len,
                    payload_sha256=parsed_sha256,
                ),
                None,
                parsed_payload_len,
            )
        offset += frame_len
    return None, "Missing", None


def read_resolved_buildfs_chunk_from_image(
    handle,
    seed_data_first_lba: int,
    resolved: ResolvedBuildFsChunkFrame,
) -> bytes:
    """Mirror read_buildfs_chunk_frame plus GrantedChunkReader's payload hash."""
    region_start = (seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE
    handle.seek(region_start + resolved.frame_offset)
    frame = handle.read(resolved.frame_len)
    if len(frame) != resolved.frame_len:
        raise ValueError(f"short resolved ARTSTOR frame read at offset {resolved.frame_offset}")
    if len(frame) < ARTIFACT_BLOB_HEADER_LEN or frame[:8] != ARTIFACT_BLOB_MAGIC:
        raise ValueError(f"resolved ARTSTOR frame is malformed at offset {resolved.frame_offset}")
    encoded_frame_len, encoded_payload_len = struct.unpack_from("<II", frame, 8)
    if encoded_frame_len != resolved.frame_len or encoded_payload_len != resolved.payload_len:
        raise ValueError(f"resolved ARTSTOR frame shape changed at offset {resolved.frame_offset}")
    payload_end = ARTIFACT_BLOB_HEADER_LEN + resolved.payload_len
    payload = frame[ARTIFACT_BLOB_HEADER_LEN:payload_end]
    if len(payload) != resolved.payload_len:
        raise ValueError(f"resolved ARTSTOR payload is truncated at offset {resolved.frame_offset}")
    if hashlib.sha256(payload).digest() != resolved.payload_sha256:
        raise ValueError(f"resolved ARTSTOR payload hash mismatch at offset {resolved.frame_offset}")
    return payload


def diagnose_buildfs_resolution(image: Path, manifest_path: Path) -> None:
    manifest = manifest_path.read_bytes()
    manifest_index = parse_buildfs_manifest_index(manifest)
    info = inspect_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    references = manifest_index.chunk_references
    chunk_lengths = [reference.length for reference in references]

    print("resolve-buildfs:")
    print(f"  image={image}")
    print(f"  manifest={manifest_path}")
    print(f"  manifest_sha256={hashlib.sha256(manifest).hexdigest()}")
    print(
        "  totals: "
        f"file_count={manifest_index.file_count} "
        f"unique_chunk_count={len(manifest_index.chunk_requirements)} "
        f"total_chunk_refs={len(references)} "
        f"min_chunk_len={min(chunk_lengths, default=0)} "
        f"max_chunk_len={max(chunk_lengths, default=0)}"
    )

    resolved_ok = 0
    with image.open("rb") as handle:
        for reference in references:
            resolved, failure, found_len = resolve_buildfs_chunk_frame_from_image(
                handle,
                seed_data_first_lba,
                reference.sha256,
                reference.length,
            )
            if failure is not None:
                print("  first_failure:")
                print(f"    manifest_index={reference.manifest_index}")
                print(f"    file_index={reference.file_index}")
                print(f"    file_path={reference.file_path}")
                print(f"    chunk_index={reference.chunk_index}")
                print(f"    expected_sha256={reference.sha256.hex()}")
                print(f"    expected_len={reference.length}")
                print(f"    reason={failure}")
                if found_len is not None:
                    print(f"    found_len={found_len}")
                print(f"    resolved_ok_before_failure={resolved_ok}")
                return
            if resolved is None:
                raise AssertionError("successful resolution did not return a frame")
            payload = read_resolved_buildfs_chunk_from_image(
                handle,
                seed_data_first_lba,
                resolved,
            )
            if len(payload) != reference.length:
                raise ValueError(
                    f"resolved payload length drift for manifest index {reference.manifest_index}"
                )
            resolved_ok += 1
    print(f"  status=ok resolved_chunk_refs={resolved_ok}")


def sha256_hex_body(value: object) -> object:
    # record-model Value::Sha256 serializes as "sha256:<64 hex>" (raios_core::record).
    # Strip the prefix so a stored field compares against a raw .hexdigest()/.hex()
    # recomputation; a raw-hex value never begins with "sha256:" so this is unambiguous.
    if isinstance(value, str) and value.startswith("sha256:"):
        return value[len("sha256:"):]
    return value


def inspect_artifact_persist_records(
    frames: list[dict[str, object]],
    handle,
    seed_data_first_lba: int,
) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    for frame in frames:
        record = payload_json(frame["payload"])
        if not record or record.get("schema") != "raios.artifact_persist.v0":
            continue
        view: dict[str, object] = {
            "seq": frame["seq"],
            "reclog_offset": frame["offset"],
            "reclog_frame_sha256": frame["frame_sha256"],
            "reclog_payload_sha256": frame["payload_sha256"],
            "artstor_blob_offset": record.get("artstor_blob_offset"),
            "artstor_blob_len": record.get("artstor_blob_len"),
            "artstor_blob_frame_sha256": record.get("artstor_blob_frame_sha256"),
            "artifact_sha256": record.get("artifact_sha256"),
            "manifest_hash": record.get("manifest_hash"),
            "vm_report_hash": record.get("vm_report_hash"),
            "grant_hash": record.get("grant_hash"),
            "promotion_transaction_sha256": record.get("promotion_transaction_sha256"),
            "authorizes_load": record.get("authorizes_load"),
            "binding_mismatches": [],
        }
        try:
            blob = read_artstor_blob(
                handle,
                seed_data_first_lba,
                int(record["artstor_blob_offset"]),
                int(record["artstor_blob_len"]),
            )
            blob_frame_sha = hashlib.sha256(blob).hexdigest()
            parsed_blob, reason = parse_artifact_blob_frame(blob, 0)
            view["actual_artstor_blob_frame_sha256"] = blob_frame_sha
            view["actual_blob_parse_reason"] = reason
            if parsed_blob is not None:
                view["actual_artifact_sha256"] = parsed_blob["payload_sha256"]
            if blob_frame_sha != sha256_hex_body(record.get("artstor_blob_frame_sha256")):
                view["binding_mismatches"].append("artstor_blob_frame_sha256")
            if parsed_blob is None:
                view["binding_mismatches"].append("artifact_blob_parse")
            elif parsed_blob["payload_sha256"] != sha256_hex_body(record.get("artifact_sha256")):
                view["binding_mismatches"].append("artifact_sha256")
        except (KeyError, TypeError, ValueError) as exc:
            view["binding_mismatches"].append("artstor_blob_read")
            view["actual_blob_parse_reason"] = str(exc)
        view["binding_ok"] = len(view["binding_mismatches"]) == 0
        records.append(view)
    return records


def inspect_promotion_transaction_records(frames: list[dict[str, object]]) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    for frame in frames:
        record = payload_json(frame["payload"])
        if not record or record.get("schema") != "raios.promotion_transaction.v0":
            continue
        records.append(
            {
                "seq": frame["seq"],
                "reclog_offset": frame["offset"],
                "reclog_frame_sha256": frame["frame_sha256"],
                "transaction_kind": record.get("transaction_kind"),
                "service_id": record.get("service_id"),
                "signature_verified": record.get("signature_verified"),
                "grant_binds_capability": record.get("grant_binds_capability"),
            }
        )
    return records


def parse_first_artstor_blob(handle, seed_data_first_lba: int) -> dict[str, object] | None:
    handle.seek((seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE)
    head = handle.read(SECTOR_SIZE)
    if len(head) != SECTOR_SIZE or head[:8] != ARTIFACT_BLOB_MAGIC:
        return None
    frame_len = struct.unpack_from("<I", head, 8)[0]
    blob = read_artstor_blob(handle, seed_data_first_lba, 0, frame_len)
    parsed, reason = parse_artifact_blob_frame(blob, 0)
    if parsed is None:
        return {"offset": 0, "reason": reason, "valid": False}
    return {**parsed, "valid": True}


def find_first_artstor_blob(handle, seed_data_first_lba: int) -> tuple[int, dict[str, object], bytes]:
    artstor_bytes = ARTSTOR_LBA_COUNT * SECTOR_SIZE
    for offset in range(0, artstor_bytes, SECTOR_SIZE):
        handle.seek((seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE + offset)
        head = handle.read(SECTOR_SIZE)
        if len(head) != SECTOR_SIZE:
            raise ValueError("short ARTSTOR scan read")
        if head[:8] != ARTIFACT_BLOB_MAGIC:
            if all_zero(head):
                break
            continue
        frame_len = struct.unpack_from("<I", head, 8)[0]
        blob = read_artstor_blob(handle, seed_data_first_lba, offset, frame_len)
        parsed, reason = parse_artifact_blob_frame(blob, 0)
        if parsed is None:
            raise ValueError(f"ARTSTOR blob parse failed at offset {offset}: {reason}")
        return offset, parsed, blob
    raise ValueError("no RAIOSAR0 ARTSTOR blob found")


def corrupt_artstor_blob(image: Path) -> dict[str, object]:
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        offset, blob, frame = find_first_artstor_blob(handle, seed_data_first_lba)
        if int(blob["payload_len"]) == 0:
            raise ValueError("cannot corrupt empty ARTSTOR blob payload")
        mutated = bytearray(frame)
        payload_offset = ARTIFACT_BLOB_HEADER_LEN
        mutated[payload_offset] ^= 0x01
        payload_end = payload_offset + int(blob["payload_len"])
        mutated[
            ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET : ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET + 32
        ] = hashlib.sha256(mutated[payload_offset:payload_end]).digest()
        parsed, reason = parse_artifact_blob_frame(mutated, 0)
        if parsed is None:
            raise ValueError(f"mutated ARTSTOR blob no longer self-parses: {reason}")
        handle.seek((seed_data_first_lba + ARTSTOR_START_LBA) * SECTOR_SIZE + offset)
        handle.write(mutated)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    records = post.get("artifact_persist_records") or []
    if not records or "artstor_blob_frame_sha256" not in records[0].get("binding_mismatches", []):
        raise ValueError("corrupt ARTSTOR blob did not create an artifact-persist blob hash mismatch")
    return {
        "operation": "corrupt_artstor_blob",
        "image": str(image),
        "artstor_blob_offset": offset,
        "old_frame_sha256": hashlib.sha256(frame).hexdigest(),
        "new_frame_sha256": hashlib.sha256(mutated).hexdigest(),
        "post_artifact_persist_record": records[0],
    }


def flip_hash_byte(hex_value: str) -> str:
    # record-model Sha256 values serialize as "sha256:<64 hex>"; preserve the prefix
    # and flip a byte in the hex body so the tampered field stays well-formed.
    prefix = ""
    if hex_value.startswith("sha256:"):
        prefix = "sha256:"
        hex_value = hex_value[len(prefix):]
    if len(hex_value) != 64:
        raise ValueError("bound hash field must be 64 hex characters")
    return f"{prefix}{int(hex_value[:2], 16) ^ 0x01:02x}{hex_value[2:]}"


def replace_json_hash(payload: bytes, field: str, old_value: str, new_value: str) -> bytes:
    old = f'"{field}": "{old_value}"'.encode("ascii")
    new = f'"{field}": "{new_value}"'.encode("ascii")
    if old not in payload:
        raise ValueError(f"could not find JSON hash field {field}")
    return payload.replace(old, new, 1)


def grant_target_snapshot_hash(
    service_id: str, artifact_sha256: str, entries: tuple[tuple[str, str], ...]
) -> str:
    artifact_hex = artifact_sha256.removeprefix("sha256:")
    artifact = bytes.fromhex(artifact_hex)
    if len(artifact) != 32 or not entries:
        raise ValueError("canonical grant target needs a 32-byte artifact and entries")
    body = bytearray(b"raios.grant_target_snapshot.v1\0")
    body.extend(service_id.encode("utf-8"))
    body.append(0)
    body.extend(artifact)
    body.extend(len(entries).to_bytes(8, "little"))
    for host_import_id, scope in entries:
        body.extend(host_import_id.encode("ascii"))
        body.append(0)
        body.extend(scope.encode("ascii"))
        body.append(0)
    return f"sha256:{hashlib.sha256(body).hexdigest()}"


def rewrite_reclog_frames(
    handle,
    seed_data_first_lba: int,
    frames: list[dict[str, object]],
    first_index: int,
    first_payload: bytes,
) -> None:
    prev_hash = (
        b"\0" * 32
        if first_index == 0
        else bytes.fromhex(str(frames[first_index - 1]["frame_sha256"]))
    )
    for idx in range(first_index, len(frames)):
        payload = first_payload if idx == first_index else frames[idx]["payload"]
        frame = build_reclog_frame(
            int(frames[idx]["seq"]),
            prev_hash,
            payload,
            int(frames[idx]["frame_len"]),
        )
        handle.seek((seed_data_first_lba + RECLOG_START_LBA) * SECTOR_SIZE + int(frames[idx]["offset"]))
        handle.write(frame)
        prev_hash = hashlib.sha256(frame).digest()


def rewrite_reclog_payloads(
    handle,
    seed_data_first_lba: int,
    frames: list[dict[str, object]],
    replacements: dict[int, bytes],
) -> None:
    if not replacements:
        raise ValueError("RECLOG rewrite requires at least one replacement payload")
    first_index = min(replacements)
    prev_hash = (
        b"\0" * 32
        if first_index == 0
        else bytes.fromhex(str(frames[first_index - 1]["frame_sha256"]))
    )
    for idx in range(first_index, len(frames)):
        payload = replacements.get(idx, frames[idx]["payload"])
        frame = build_reclog_frame(
            int(frames[idx]["seq"]),
            prev_hash,
            payload,
            int(frames[idx]["frame_len"]),
        )
        handle.seek(
            (seed_data_first_lba + RECLOG_START_LBA) * SECTOR_SIZE
            + int(frames[idx]["offset"])
        )
        handle.write(frame)
        prev_hash = hashlib.sha256(frame).digest()


def tamper_persist_record(image: Path) -> dict[str, object]:
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        target_index = None
        target_record = None
        for idx, frame in enumerate(frames):
            record = payload_json(frame["payload"])
            if record and record.get("schema") == "raios.artifact_persist.v0":
                target_index = idx
                target_record = record
                break
        if target_index is None or target_record is None:
            raise ValueError("no raios.artifact_persist.v0 RECLOG record found")
        field = "artifact_sha256"
        old_value = str(target_record[field])
        new_value = flip_hash_byte(old_value)
        new_payload = replace_json_hash(frames[target_index]["payload"], field, old_value, new_value)
        rewrite_reclog_frames(handle, seed_data_first_lba, frames, target_index, new_payload)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    records = post.get("artifact_persist_records") or []
    if scan.get("status") != "valid":
        raise ValueError("tampered RECLOG no longer scans as valid")
    if not records or "artifact_sha256" not in records[0].get("binding_mismatches", []):
        raise ValueError("tampered artifact_persist record did not create an artifact hash mismatch")
    return {
        "operation": "tamper_persist_record",
        "image": str(image),
        "field": field,
        "old_value": old_value,
        "new_value": new_value,
        "seq": frames[target_index]["seq"],
        "post_artifact_persist_record": records[0],
    }


def tamper_rollback_fixture_field(image: Path, mutation: str) -> dict[str, object]:
    """Mutate one signed predecessor binding while preserving the outer RECLOG chain."""
    assert_not_release_output(image)
    selectors = {
        "target_snapshot": ("raios.artifact_persist.v0", "grant_target_snapshot_sha256"),
        "artstor_link": ("raios.artifact_persist.v0", "artstor_blob_frame_sha256"),
        "promotion_signature": ("raios.install_authorization.v0", "install_signature_der"),
    }
    schema, field = selectors[mutation]
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        target_index = None
        target_record = None
        for idx, frame in enumerate(frames):
            record = payload_json(frame["payload"])
            if record and record.get("schema") == schema:
                target_index = idx
                target_record = record
                break
        if target_index is None or target_record is None:
            raise ValueError(f"no {schema} predecessor record found")
        old_value = str(target_record.get(field, ""))
        if field == "install_signature_der":
            if len(old_value) < 2 or len(old_value) % 2 != 0:
                raise ValueError("predecessor signature is not non-empty even-length hex")
            new_value = f"{int(old_value[:2], 16) ^ 0x01:02x}{old_value[2:]}"
        else:
            new_value = flip_hash_byte(old_value)
        new_payload = replace_json_hash(frames[target_index]["payload"], field, old_value, new_value)
        rewrite_reclog_frames(handle, seed_data_first_lba, frames, target_index, new_payload)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != len(frames):
        raise ValueError("rollback predecessor mutation damaged outer RECLOG validity")
    return {
        "operation": f"tamper_rollback_fixture_{mutation}",
        "image": str(image),
        "schema": schema,
        "field": field,
        "old_value": old_value,
        "new_value": new_value,
        "seq": frames[target_index]["seq"],
        "post_reclog_scan": scan,
    }


def reseal_rollback_fixture_snapshot(image: Path) -> dict[str, object]:
    """Replace only the unsigned artifact snapshot with a different canonical snapshot."""
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        target_index = None
        target_record = None
        for idx, frame in enumerate(frames):
            record = payload_json(frame["payload"])
            if record and record.get("schema") == "raios.artifact_persist.v0":
                target_index = idx
                target_record = record
                break
        if target_index is None or target_record is None:
            raise ValueError("no rollback artifact_persist predecessor record found")
        if target_record.get("grant_target_entries") != "env.log:host_call":
            raise ValueError("rollback predecessor is not the pinned log-only snapshot")
        signed_prefix = [
            (frames[index]["payload"], frames[index]["frame_sha256"])
            for index in range(target_index)
        ]
        old_snapshot = str(target_record.get("grant_target_snapshot_sha256", ""))
        entries = (
            ("env.log", "host_call"),
            ("env.counter_get", "host_call"),
        )
        import_set_sha256 = hashlib.sha256(b"env.log\nenv.counter_get\n").hexdigest()
        target_record["import_set_hash"] = f"sha256:{import_set_sha256}"
        target_record["grant_target_entries"] = (
            "env.log:host_call,env.counter_get:host_call"
        )
        target_record["grant_target_count"] = len(entries)
        target_record["grant_target_snapshot_sha256"] = grant_target_snapshot_hash(
            str(target_record.get("service_id", "")),
            str(target_record.get("artifact_sha256", "")),
            entries,
        )
        new_payload = (json.dumps(target_record, indent=2) + "\n").encode("utf-8")
        rewrite_reclog_frames(handle, seed_data_first_lba, frames, target_index, new_payload)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != len(frames):
        raise ValueError("canonical snapshot reseal damaged outer RECLOG validity")
    with image.open("rb") as handle:
        after_frames = parse_reclog_frames_for_inspection(
            read_reclog_region(handle, seed_data_first_lba)
        )
    for index, (payload, frame_sha256) in enumerate(signed_prefix):
        if (
            after_frames[index]["payload"] != payload
            or after_frames[index]["frame_sha256"] != frame_sha256
        ):
            raise ValueError("canonical snapshot reseal changed signed authorization/promotion")
    return {
        "operation": "reseal_rollback_fixture_snapshot",
        "image": str(image),
        "semantic_boundary": "signed_snapshot_context_mismatch",
        "old_snapshot": old_snapshot,
        "new_snapshot": target_record["grant_target_snapshot_sha256"],
        "new_entries": target_record["grant_target_entries"],
        "new_count": target_record["grant_target_count"],
        "signed_authorization_and_promotion_unchanged": True,
        "unsigned_artifact_and_reclog_resealed": True,
        "post_reclog_scan": scan,
    }


def duplicate_authenticated_install(image: Path) -> dict[str, object]:
    """Append an authentic install triple with byte-identical payloads and new framing."""
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(
            read_reclog_region(handle, seed_data_first_lba)
        )
        records = [payload_json(frame["payload"]) for frame in frames]
        triple_indices = None
        for persist_index, persist in enumerate(records):
            if not persist or persist.get("schema") != "raios.artifact_persist.v0":
                continue
            promotion_sha256 = str(
                persist.get("promotion_transaction_sha256", "")
            ).removeprefix("sha256:")
            promotion_index = next(
                (
                    index
                    for index, frame in enumerate(frames)
                    if frame["frame_sha256"] == promotion_sha256
                ),
                None,
            )
            if promotion_index is None:
                continue
            promotion = records[promotion_index]
            if (
                not promotion
                or promotion.get("schema") != "raios.promotion_transaction.v0"
                or promotion.get("transaction_kind") != "promote"
            ):
                continue
            authorization_sha256 = str(
                promotion.get("install_authorization_frame_sha256", "")
            ).removeprefix("sha256:")
            authorization_index = next(
                (
                    index
                    for index, frame in enumerate(frames)
                    if frame["frame_sha256"] == authorization_sha256
                ),
                None,
            )
            if authorization_index is None:
                continue
            authorization = records[authorization_index]
            if (
                not authorization
                or authorization.get("schema") != "raios.install_authorization.v0"
                or str(persist.get("install_authorization_frame_sha256", "")).removeprefix(
                    "sha256:"
                )
                != authorization_sha256
                or not (authorization_index < promotion_index < persist_index)
            ):
                continue
            triple_indices = (
                authorization_index,
                promotion_index,
                persist_index,
            )
            break
        if triple_indices is None:
            raise ValueError("linked authorization+promotion+persist install triple not found")

        original_frames = [frames[index] for index in triple_indices]
        append_offset = sum(int(frame["frame_len"]) for frame in frames)
        duplicate_len = sum(int(frame["frame_len"]) for frame in original_frames)
        region_len = RECLOG_LBA_COUNT * SECTOR_SIZE
        if append_offset + duplicate_len > region_len:
            raise ValueError("RECLOG has no room for duplicated authenticated install")

        prev_hash = bytes.fromhex(str(frames[-1]["frame_sha256"]))
        duplicate_frames = []
        for ordinal, original in enumerate(original_frames, start=1):
            seq = len(frames) + ordinal
            frame = build_reclog_frame(
                seq,
                prev_hash,
                original["payload"],
                int(original["frame_len"]),
            )
            handle.seek(
                (seed_data_first_lba + RECLOG_START_LBA) * SECTOR_SIZE
                + append_offset
            )
            handle.write(frame)
            duplicate_frames.append(
                {
                    "seq": seq,
                    "offset": append_offset,
                    "frame_sha256": hashlib.sha256(frame).hexdigest(),
                    "payload_sha256": hashlib.sha256(original["payload"]).hexdigest(),
                }
            )
            append_offset += len(frame)
            prev_hash = hashlib.sha256(frame).digest()
        handle.flush()
        os.fsync(handle.fileno())

    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != len(frames) + 3:
        raise ValueError("duplicated authenticated install damaged outer RECLOG validity")
    with image.open("rb") as handle:
        after_frames = parse_reclog_frames_for_inspection(
            read_reclog_region(handle, seed_data_first_lba)
        )
    for original_index, original in zip(triple_indices, original_frames):
        if after_frames[original_index]["frame"] != original["frame"]:
            raise ValueError("authenticated install duplication changed an original frame")
    for ordinal, original in enumerate(original_frames):
        duplicate = after_frames[len(frames) + ordinal]
        if duplicate["payload"] != original["payload"]:
            raise ValueError("authenticated install duplicate changed signed payload bytes")

    return {
        "operation": "duplicate_authenticated_install",
        "image": str(image),
        "semantic_boundary": "rollback_target_authenticated_order_duplicate",
        "original_seq": [int(frame["seq"]) for frame in original_frames],
        "duplicate_seq": [frame["seq"] for frame in duplicate_frames],
        "original_frame_sha256": [str(frame["frame_sha256"]) for frame in original_frames],
        "duplicate_frame_sha256": [frame["frame_sha256"] for frame in duplicate_frames],
        "original_payload_sha256": [str(frame["payload_sha256"]) for frame in original_frames],
        "duplicate_payload_sha256": [frame["payload_sha256"] for frame in duplicate_frames],
        "signed_install_payloads_unchanged": True,
        "duplicated_install_payloads_byte_identical": True,
        "unsigned_reclog_framing_rechained": True,
        "outer_reclog_chain_valid": True,
        "post_reclog_scan": scan,
    }


def select_exact_rollback_recovery(frames: list[dict[str, object]]) -> tuple[int, int, int, dict[str, object], dict[str, object]]:
    records = [payload_json(frame["payload"]) for frame in frames]
    if any(record is None for record in records):
        raise ValueError("rollback recovery contains malformed JSON")
    try:
        plan = ROLLBACK_SECURITY.derive_recovery_plan(
            records, ROLLBACK_SECURITY.DETERMINISTIC_ROLLBACK_TARGET)
    except ROLLBACK_SECURITY.SecurityError as exc:
        code = getattr(exc, "code", "recovery_malformed")
        raise ValueError(f"rollback recovery rejected: {code}") from exc
    intent, commit = records[plan.intent_index], records[plan.commit_index]
    return plan.intent_index, plan.revoke_index, plan.commit_index, intent, commit


def forge_rollback_recovery(image: Path, mutation: str) -> dict[str, object]:
    """Reseal one exact unsigned recovery record while preserving canonical record shape."""
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        intent_index, revoke_index, commit_index, intent_record, commit_record = select_exact_rollback_recovery(frames)

        signed_payloads = {
            idx: frame["payload"]
            for idx, frame in enumerate(frames[:intent_index])
            if (payload_json(frame["payload"]) or {}).get("schema")
            in {
                "raios.install_authorization.v0",
                "raios.promotion_transaction.v0",
                "raios.artifact_persist.v0",
            }
        }
        replacements: dict[int, bytes] = {}
        semantic_boundary = ""
        if mutation == "unchanged_projection":
            source_projection = str(
                (intent_record.get("value") or {}).get("source_projection_sha256", "")
            )
            if not source_projection.startswith("sha256:"):
                raise ValueError("rollback intent lacks a source projection hash")
            old_projection = str((commit_record.get("value") or {}).get("result_projection_sha256", ""))
            replacements[commit_index] = replace_json_hash(frames[commit_index]["payload"], "result_projection_sha256", old_projection, source_projection)
            commit_record["value"]["result_projection_sha256"] = source_projection
            semantic_boundary = "rollback_transition_delta_count_mismatch"
        elif mutation in {"pair_mismatch", "delta_count"}:
            old_count = int(commit_record["value"]["delta_count"]); new_count = old_count + 1
            old_token = f'"delta_count": {old_count}'.encode(); new_token = f'"delta_count": {new_count}'.encode()
            if len(old_token) != len(new_token) or frames[commit_index]["payload"].count(old_token) != 1:
                raise ValueError("rollback commit delta_count token is not one fixed-width canonical scalar")
            replacements[commit_index] = frames[commit_index]["payload"].replace(old_token, new_token, 1)
            commit_record["value"]["delta_count"] = new_count
            semantic_boundary = "rollback_recovery_pair_mismatch"
        else:
            raise ValueError(f"unknown rollback recovery mutation: {mutation}")

        rewrite_reclog_payloads(handle, seed_data_first_lba, frames, replacements)
        handle.flush()
        os.fsync(handle.fileno())

    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != len(frames):
        raise ValueError("rollback recovery reseal damaged outer RECLOG validity")
    with image.open("rb") as handle:
        after_frames = parse_reclog_frames_for_inspection(
            read_reclog_region(handle, seed_data_first_lba)
        )
    for idx, signed_payload in signed_payloads.items():
        if after_frames[idx]["payload"] != signed_payload:
            raise ValueError("rollback recovery reseal changed signed install payload bytes")
    for idx, before in enumerate(frames):
        if idx != commit_index and after_frames[idx]["payload"] != before["payload"]:
            raise ValueError("rollback recovery mutation changed a non-commit record")
    if payload_json(after_frames[commit_index]["payload"]) != commit_record:
        raise ValueError("rollback recovery mutation changed more than its named commit scalar")
    return {
        "operation": f"forge_rollback_recovery_{mutation}",
        "image": str(image),
        "semantic_boundary": semantic_boundary,
        "intent_seq": frames[intent_index]["seq"],
        "revoke_seq": frames[revoke_index]["seq"],
        "commit_seq": frames[commit_index]["seq"],
        "signed_install_payloads_unchanged": True,
        "revoke_record_byte_identical": after_frames[revoke_index]["payload"] == frames[revoke_index]["payload"],
        "canonical_scalar_shape_preserved": True,
        "unsigned_recovery_and_reclog_resealed": True,
        "post_reclog_scan": scan,
    }


def tamper_wasm_grant_event(image: Path) -> dict[str, object]:
    """Break the typed revoke->grant hash link while preserving RECLOG framing."""
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        target_index = None
        target_record = None
        for idx, frame in enumerate(frames):
            record = payload_json(frame["payload"])
            if record and record.get("predicate") == "wasm_import.revoked.v1":
                target_index = idx
                target_record = record
                break
        if target_index is None or target_record is None:
            raise ValueError("no wasm_import.revoked.v1 RECLOG record found")
        value = target_record.get("value") or {}
        field = "parent_grant_sha256"
        old_value = str(value.get(field, ""))
        new_value = flip_hash_byte(old_value)
        new_payload = replace_json_hash(frames[target_index]["payload"], field, old_value, new_value)
        rewrite_reclog_frames(handle, seed_data_first_lba, frames, target_index, new_payload)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != len(frames):
        raise ValueError("semantic grant-link tamper damaged outer RECLOG validity")
    return {
        "operation": "tamper_wasm_grant_event",
        "image": str(image),
        "semantic_boundary": "parent_grant_sha256",
        "old_value": old_value,
        "new_value": new_value,
        "seq": frames[target_index]["seq"],
        "post_reclog_scan": scan,
    }


def corrupt_memory_frame(image: Path) -> dict[str, object]:
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    with image.open("r+b") as handle:
        frames = parse_reclog_frames_for_inspection(read_reclog_region(handle, seed_data_first_lba))
        if not frames:
            raise ValueError("no RECLOG frame found")
        target = frames[-1]
        payload_len = int(target["payload_len"])
        if payload_len <= 0:
            raise ValueError("cannot corrupt empty RECLOG frame payload")
        reclog_payload_offset = int(target["offset"]) + RECLOG_HEADER_LEN
        disk_offset = (seed_data_first_lba + RECLOG_START_LBA) * SECTOR_SIZE + reclog_payload_offset
        handle.seek(disk_offset)
        old = handle.read(1)
        if len(old) != 1:
            raise ValueError("short RECLOG payload byte read")
        new = bytes([old[0] ^ 0x01])
        handle.seek(disk_offset)
        handle.write(new)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    scan = post.get("reclog_scan") or {}
    if scan.get("status") == "valid" and int(scan.get("count", -1)) >= len(frames):
        raise ValueError("corrupt RECLOG memory frame still scans as fully valid")
    return {
        "operation": "corrupt_memory_frame",
        "image": str(image),
        "seq": target["seq"],
        "reclog_offset": target["offset"],
        "reclog_payload_offset": reclog_payload_offset,
        "disk_offset": disk_offset,
        "old_byte": f"0x{old[0]:02x}",
        "new_byte": f"0x{new[0]:02x}",
        "old_frame_sha256": target["frame_sha256"],
        "old_payload_sha256": target["payload_sha256"],
        "post_reclog_scan": scan,
    }


def apply_bootctl_fixture_update(image: Path, spec: str) -> dict[str, object]:
    assert_not_release_output(image)
    info = validate_existing_gpt_image(image)
    seed_data_first_lba = seed_data_first_lba_from_info(info)
    storage_slot_a, storage_slot_b = boot_control_fields_for_fixture(spec)
    with image.open("r+b") as handle:
        handle.seek((seed_data_first_lba + BOOTCTL_START_LBA) * SECTOR_SIZE)
        handle.write(storage_slot_a)
        handle.write(storage_slot_b)
        handle.flush()
        os.fsync(handle.fileno())
    post = inspect_image(image)
    validate_boot_control_fixture_or_raise(post, spec)
    return {
        "operation": "seed_bootctl_existing",
        "image": str(image),
        "spec": spec,
        "post_bootctl_read": post["bootctl_read"],
    }


def build_image(
    output: Path,
    reclog_fixture_spec: str | None = None,
    bootctl_fixture_spec: str | None = None,
    seed_artstor_garbage: bool = False,
    buildfs_seed: BuildFsSeed | None = None,
    buildfs_seeds: tuple[BuildFsSeed, ...] | None = None,
    dev_rollback_seed: DevRollbackSeed | None = None,
) -> None:
    assert_not_release_output(output)
    if ARTSTOR_LBA_COUNT <= 0:
        raise RuntimeError("SEED_DATA is too small for ARTSTOR")
    if buildfs_seed is not None and buildfs_seeds is not None:
        raise ValueError("single and multi BuildFS seeds cannot both be supplied")
    effective_buildfs_seeds = (
        buildfs_seeds
        if buildfs_seeds is not None
        else ((buildfs_seed,) if buildfs_seed is not None else ())
    )

    if dev_rollback_seed is not None and (
        reclog_fixture_spec not in {None, "", "empty"}
        or seed_artstor_garbage
        or effective_buildfs_seeds
    ):
        raise ValueError("dev rollback fixture requires otherwise-empty RECLOG and ARTSTOR")

    reclog_fixture = parse_reclog_fixture(reclog_fixture_spec)
    output.parent.mkdir(parents=True, exist_ok=True)
    entries = gpt_entries()
    entries_crc = crc32(entries)
    primary_header = gpt_header(PRIMARY_GPT_HEADER_LBA, BACKUP_GPT_HEADER_LBA, PRIMARY_GPT_ENTRIES_LBA, entries_crc)
    backup_header = gpt_header(BACKUP_GPT_HEADER_LBA, PRIMARY_GPT_HEADER_LBA, BACKUP_GPT_ENTRIES_LBA, entries_crc)
    fat32_builder = load_fat32_builder()
    empty_esp = fat32_builder(ESP_SLOT_BYTES).build()
    sb = superblock_sector()

    with output.open("w+b") as handle:
        handle.truncate(TOTAL_BYTES)
        write_at(handle, 0, protective_mbr())
        write_at(handle, PRIMARY_GPT_HEADER_LBA, primary_header)
        write_at(handle, PRIMARY_GPT_ENTRIES_LBA, entries)
        write_at(handle, ESP_A_START_LBA, empty_esp)
        write_at(handle, ESP_B_START_LBA, empty_esp)
        write_at(handle, SEED_DATA_START_LBA, sb)
        write_at(handle, SEED_DATA_START_LBA + 1, sb)
        seed_reclog_fixture(handle, reclog_fixture)
        if dev_rollback_seed is not None:
            seed_dev_rollback_fixture(handle, dev_rollback_seed)
        if seed_artstor_garbage:
            seed_artstor_garbage_blob(handle)
        if effective_buildfs_seeds:
            seed_buildfs_trees(handle, effective_buildfs_seeds)
        seed_bootctl_fixture(handle, bootctl_fixture_spec)
        write_at(handle, BACKUP_GPT_ENTRIES_LBA, entries)
        write_at(handle, BACKUP_GPT_HEADER_LBA, backup_header)


def read_at(handle, lba: int, byte_count: int) -> bytes:
    handle.seek(lba * SECTOR_SIZE)
    data = handle.read(byte_count)
    if len(data) != byte_count:
        raise ValueError(f"short read at LBA {lba}: expected {byte_count}, got {len(data)}")
    return data


def parse_gpt_header(data: bytes) -> dict[str, object]:
    signature = data[:8]
    header_size = struct.unpack_from("<I", data, 12)[0]
    if header_size < 92 or header_size > SECTOR_SIZE:
        header_size = 92
    stored_crc = struct.unpack_from("<I", data, 16)[0]
    crc_data = bytearray(data[:header_size])
    struct.pack_into("<I", crc_data, 16, 0)
    current_lba, backup_lba, first_usable, last_usable = struct.unpack_from("<QQQQ", data, 24)
    entries_lba, entry_count, entry_size, entries_crc = struct.unpack_from("<QIII", data, 72)
    return {
        "signature": signature.decode("ascii", "replace"),
        "header_size": header_size,
        "stored_crc32": f"{stored_crc:08x}",
        "computed_crc32": f"{crc32(crc_data):08x}",
        "crc_ok": stored_crc == crc32(crc_data),
        "current_lba": current_lba,
        "backup_lba": backup_lba,
        "first_usable_lba": first_usable,
        "last_usable_lba": last_usable,
        "disk_guid": str(uuid.UUID(bytes_le=data[56:72])),
        "partition_entry_lba": entries_lba,
        "partition_entry_count": entry_count,
        "partition_entry_size": entry_size,
        "partition_entry_array_crc32": f"{entries_crc:08x}",
    }


def parse_partitions(entries: bytes) -> list[dict[str, object]]:
    partitions: list[dict[str, object]] = []
    for index in range(GPT_ENTRY_COUNT):
        start = index * GPT_ENTRY_SIZE
        raw = entries[start : start + GPT_ENTRY_SIZE]
        if raw[:16] == b"\0" * 16:
            continue
        first_lba, last_lba, attributes = struct.unpack_from("<QQQ", raw, 32)
        name = raw[56:128].decode("utf-16le", "replace").rstrip("\0")
        partitions.append(
            {
                "index": index + 1,
                "name": name,
                "type_guid": str(uuid.UUID(bytes_le=raw[0:16])),
                "unique_guid": str(uuid.UUID(bytes_le=raw[16:32])),
                "first_lba": first_lba,
                "last_lba": last_lba,
                "lba_count": last_lba - first_lba + 1,
                "attributes": attributes,
            }
        )
    return partitions


def parse_superblock(data: bytes, expected_data_lba_count: int) -> dict[str, object]:
    magic = data[:16]
    version, header_len, region_count, entry_len, data_lba_count = struct.unpack_from("<IIIIQ", data, 16)
    header = data[:header_len] if 0 < header_len <= SECTOR_SIZE else b""
    stored_hash = data[SUPERBLOCK_HASH_OFFSET : SUPERBLOCK_HASH_OFFSET + SUPERBLOCK_HASH_LEN]
    computed_hash = hashlib.sha256(header).digest() if header else b""
    regions = []
    offset = SUPERBLOCK_REGION_TABLE_OFFSET
    for _ in range(region_count if region_count <= 16 else 0):
        tag = data[offset : offset + 8].rstrip(b"\0").decode("ascii", "replace")
        start_lba, lba_count = struct.unpack_from("<QQ", data, offset + 8)
        regions.append({"tag": tag, "start_lba": start_lba, "lba_count": lba_count})
        offset += entry_len
    expected_regions = [
        {"tag": tag, "start_lba": start_lba, "lba_count": lba_count}
        for tag, start_lba, lba_count in REGIONS
    ]
    valid = (
        magic == SUPERBLOCK_MAGIC
        and version == SUPERBLOCK_VERSION
        and header_len == SUPERBLOCK_HEADER_LEN
        and region_count == len(REGIONS)
        and entry_len == SUPERBLOCK_REGION_ENTRY_LEN
        and data_lba_count == expected_data_lba_count
        and stored_hash == computed_hash
        and regions == expected_regions
    )
    return {
        "magic": magic.decode("ascii", "replace"),
        "version": version,
        "header_len": header_len,
        "region_count": region_count,
        "region_entry_len": entry_len,
        "data_lba_count": data_lba_count,
        "sha256": stored_hash.hex(),
        "computed_sha256": computed_hash.hex(),
        "sha256_ok": stored_hash == computed_hash,
        "regions": regions,
        "valid": valid,
        "hex_head": data[:160].hex(),
    }


def all_zero(data: bytes) -> bool:
    return all(byte == 0 for byte in data)


def parse_reclog_frame(data: bytes, offset: int, expected_seq: int, expected_prev_hash: bytes) -> tuple[dict[str, object] | None, str | None]:
    if offset + RECLOG_HEADER_LEN > len(data):
        return None, "truncated_frame_header"
    header = data[offset : offset + RECLOG_HEADER_LEN]
    if header[:8] != RECLOG_MAGIC:
        return None, "bad_magic"
    frame_len, payload_len, seq = struct.unpack_from("<IIQ", header, 8)
    if frame_len < SECTOR_SIZE or frame_len % SECTOR_SIZE != 0:
        return None, "bad_frame_len"
    if offset + frame_len > len(data):
        return None, "frame_len_out_of_bounds"
    if payload_len > frame_len - RECLOG_HEADER_LEN:
        return None, "payload_len_out_of_bounds"
    if seq != expected_seq:
        return None, "bad_seq"

    frame = data[offset : offset + frame_len]
    if frame[RECLOG_PREV_FRAME_SHA256_OFFSET : RECLOG_PREV_FRAME_SHA256_OFFSET + 32] != expected_prev_hash:
        return None, "bad_prev_frame_sha256"
    payload = frame[RECLOG_HEADER_LEN : RECLOG_HEADER_LEN + payload_len]
    payload_hash = hashlib.sha256(payload).digest()
    if frame[RECLOG_PAYLOAD_SHA256_OFFSET : RECLOG_PAYLOAD_SHA256_OFFSET + 32] != payload_hash:
        return None, "bad_payload_sha256"
    if not all_zero(frame[RECLOG_HEADER_LEN + payload_len :]):
        return None, "frame_padding_nonzero"

    frame_hash = hashlib.sha256(frame).digest()
    return {
        "offset": offset,
        "frame_len": frame_len,
        "payload_len": payload_len,
        "seq": seq,
        "frame_sha256": frame_hash.hex(),
        "payload_sha256": payload_hash.hex(),
    }, None


def parse_artifact_blob_frame(data: bytes, offset: int) -> tuple[dict[str, object] | None, str | None]:
    if offset + ARTIFACT_BLOB_HEADER_LEN > len(data):
        return None, "truncated_frame_header"
    header = data[offset : offset + ARTIFACT_BLOB_HEADER_LEN]
    if header[:8] != ARTIFACT_BLOB_MAGIC:
        return None, "bad_magic"
    frame_len, payload_len = struct.unpack_from("<II", header, 8)
    if frame_len < SECTOR_SIZE or frame_len % SECTOR_SIZE != 0:
        return None, "bad_frame_len"
    if offset + frame_len > len(data):
        return None, "frame_len_out_of_bounds"
    if payload_len > frame_len - ARTIFACT_BLOB_HEADER_LEN:
        return None, "payload_len_out_of_bounds"
    frame = data[offset : offset + frame_len]
    payload = frame[ARTIFACT_BLOB_HEADER_LEN : ARTIFACT_BLOB_HEADER_LEN + payload_len]
    payload_hash = hashlib.sha256(payload).digest()
    if frame[ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET : ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET + 32] != payload_hash:
        return None, "bad_payload_sha256"
    if not all_zero(frame[ARTIFACT_BLOB_HEADER_LEN + payload_len :]):
        return None, "frame_padding_nonzero"
    return {
        "offset": offset,
        "frame_len": frame_len,
        "payload_len": payload_len,
        "payload_sha256": payload_hash.hex(),
        "frame_sha256": hashlib.sha256(frame).hexdigest(),
        "status": "garbage",
        "reason": "blob_without_chained_reclog_record",
        "authorizes_load": False,
    }, None


def scan_artstor_garbage(data: bytes) -> dict[str, object]:
    blob, reason = parse_artifact_blob_frame(data, 0)
    return {
        "garbage_blob_present": blob is not None,
        "garbage_blob": blob,
        "reason": "blob_without_chained_reclog_record" if blob is not None else reason,
        "authorizes_load": False,
    }


def scan_reclog(data: bytes) -> dict[str, object]:
    if len(data) == 0:
        return {
            "status": "valid_empty",
            "head_seq": 0,
            "tail_seq": 0,
            "count": 0,
            "terminated_reason": "empty_region",
            "torn_tail": False,
            "first_invalid_offset": 0,
            "valid_prefix_chain": True,
            "full_region_valid": True,
            "head_frame_sha256": None,
            "tail_frame_sha256": None,
        }
    if len(data) % SECTOR_SIZE != 0:
        return {
            "status": "invalid",
            "head_seq": 0,
            "tail_seq": 0,
            "count": 0,
            "terminated_reason": "region_not_sector_aligned",
            "torn_tail": False,
            "first_invalid_offset": 0,
            "valid_prefix_chain": False,
            "full_region_valid": False,
            "head_frame_sha256": None,
            "tail_frame_sha256": None,
        }

    offset = 0
    expected_seq = 1
    expected_prev_hash = b"\0" * 32
    count = 0
    head_hash = None
    tail_hash = None
    while offset < len(data):
        if all_zero(data[offset:]):
            return {
                "status": "valid_empty" if count == 0 else "valid",
                "head_seq": 1 if count else 0,
                "tail_seq": count,
                "count": count,
                "terminated_reason": "zero_filled",
                "torn_tail": False,
                "first_invalid_offset": offset,
                "valid_prefix_chain": True,
                "full_region_valid": True,
                "head_frame_sha256": head_hash,
                "tail_frame_sha256": tail_hash,
            }
        frame, reason = parse_reclog_frame(data, offset, expected_seq, expected_prev_hash)
        if frame is None:
            return {
                "status": "torn_tail" if count else "invalid",
                "head_seq": 1 if count else 0,
                "tail_seq": count,
                "count": count,
                "terminated_reason": reason,
                "torn_tail": count > 0,
                "first_invalid_offset": offset,
                "valid_prefix_chain": True,
                "full_region_valid": False,
                "head_frame_sha256": head_hash,
                "tail_frame_sha256": tail_hash,
            }
        if count == 0:
            head_hash = str(frame["frame_sha256"])
        tail_hash = str(frame["frame_sha256"])
        count += 1
        expected_seq += 1
        expected_prev_hash = bytes.fromhex(str(frame["frame_sha256"]))
        offset += int(frame["frame_len"])

    return {
        "status": "valid_empty" if count == 0 else "valid",
        "head_seq": 1 if count else 0,
        "tail_seq": count,
        "count": count,
        "terminated_reason": "region_exhausted",
        "torn_tail": False,
        "first_invalid_offset": len(data),
        "valid_prefix_chain": True,
        "full_region_valid": True,
        "head_frame_sha256": head_hash,
        "tail_frame_sha256": tail_hash,
    }


def parse_slot_id_value(value: int, reason: str) -> tuple[str | None, str | None]:
    if value == SLOT_ID_NONE:
        return None, None
    if value == SLOT_ID_A:
        return "A", None
    if value == SLOT_ID_B:
        return "B", None
    return None, reason


def parse_slot_state_value(value: int, reason: str) -> tuple[str | None, str | None]:
    try:
        return slot_state_name(value), None
    except ValueError:
        return None, reason


def parse_bool_value(value: int, reason: str) -> tuple[bool | None, str | None]:
    if value == 0:
        return False, None
    if value == 1:
        return True, None
    return None, reason


def parse_boot_control_slot(data: bytes) -> tuple[dict[str, object] | None, str | None]:
    if len(data) != BOOTCTL_SLOT_SIZE:
        return None, "slot_size_mismatch"
    if all_zero(data):
        return None, "zero_filled_slot"
    if data[:8] != BOOTCTL_MAGIC:
        return None, "bad_magic"
    payload_len = struct.unpack_from("<I", data, 8)[0]
    seq = struct.unpack_from("<Q", data, 12)[0]
    if payload_len > BOOTCTL_SLOT_MAX_PAYLOAD_LEN:
        return None, "payload_len_out_of_bounds"
    if payload_len != BOOTCTL_PAYLOAD_LEN:
        return None, "payload_len_mismatch"
    payload_end = BOOTCTL_PAYLOAD_OFFSET + payload_len
    if payload_end > BOOTCTL_SLOT_SIZE:
        return None, "payload_len_out_of_bounds"
    payload = data[BOOTCTL_PAYLOAD_OFFSET:payload_end]
    payload_hash = hashlib.sha256(payload).digest()
    if data[BOOTCTL_PAYLOAD_SHA256_OFFSET : BOOTCTL_PAYLOAD_SHA256_OFFSET + 32] != payload_hash:
        return None, "bad_payload_sha256"
    if not all_zero(data[payload_end:]):
        return None, "slot_padding_nonzero"
    if payload[BOOTCTL_PAYLOAD_MAGIC_OFFSET : BOOTCTL_PAYLOAD_MAGIC_OFFSET + 8] != BOOTCTL_PAYLOAD_MAGIC:
        return None, "bad_payload_magic"
    if struct.unpack_from("<I", payload, BOOTCTL_PAYLOAD_VERSION_OFFSET)[0] != BOOT_CONTROL_VERSION:
        return None, "payload_version_mismatch"

    active, reason = parse_slot_id_value(payload[BOOTCTL_ACTIVE_SLOT_OFFSET], "bad_active_slot")
    if reason:
        return None, reason
    last_good, reason = parse_slot_id_value(payload[BOOTCTL_LAST_GOOD_SLOT_OFFSET], "bad_last_good_slot")
    if reason:
        return None, reason
    pending, reason = parse_slot_id_value(payload[BOOTCTL_PENDING_SLOT_OFFSET], "bad_pending_slot")
    if reason:
        return None, reason
    safe_mode, reason = parse_bool_value(payload[BOOTCTL_SAFE_MODE_OFFSET], "bad_safe_mode")
    if reason:
        return None, reason
    attempt_slot, reason = parse_slot_id_value(
        payload[BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET], "bad_boot_attempt_slot"
    )
    if reason:
        return None, reason
    success_marked, reason = parse_bool_value(
        payload[BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET], "bad_success_marked"
    )
    if reason:
        return None, reason
    slot_a_state, reason = parse_slot_state_value(payload[BOOTCTL_SLOT_A_STATE_OFFSET], "bad_slot_a_state")
    if reason:
        return None, reason
    slot_b_state, reason = parse_slot_state_value(payload[BOOTCTL_SLOT_B_STATE_OFFSET], "bad_slot_b_state")
    if reason:
        return None, reason

    return {
        "valid": True,
        "seq": seq,
        "payload_len": payload_len,
        "storage_sha256": hashlib.sha256(data).hexdigest(),
        "payload_sha256": payload_hash.hex(),
        "active": active,
        "last_good": last_good,
        "pending": pending,
        "safe_mode": safe_mode,
        "boot_attempt": {
            "slot": attempt_slot,
            "generation": struct.unpack_from("<Q", payload, BOOTCTL_BOOT_ATTEMPT_GENERATION_OFFSET)[0],
            "success_marked": success_marked,
        },
        "slots": {
            "A": {
                "generation": struct.unpack_from("<Q", payload, BOOTCTL_SLOT_A_GENERATION_OFFSET)[0],
                "state": slot_a_state,
                "failure_count": struct.unpack_from("<I", payload, BOOTCTL_SLOT_A_FAILURE_COUNT_OFFSET)[0],
            },
            "B": {
                "generation": struct.unpack_from("<Q", payload, BOOTCTL_SLOT_B_GENERATION_OFFSET)[0],
                "state": slot_b_state,
                "failure_count": struct.unpack_from("<I", payload, BOOTCTL_SLOT_B_FAILURE_COUNT_OFFSET)[0],
            },
        },
    }, None


def boot_slot_bootable(record: dict[str, object], slot: str | None) -> bool:
    if slot not in {"A", "B"}:
        return False
    state = record["slots"][slot]["state"]
    return state in {"candidate", "pending", "good"}


def evaluate_boot_control_record(storage_slot: str, record: dict[str, object]) -> dict[str, object]:
    pending = record["pending"]
    failure_count = None
    if pending in {"A", "B"}:
        failure_count = int(record["slots"][pending]["failure_count"])
    if record["safe_mode"]:
        return boot_control_decision(storage_slot, record, None, "Safe", "safe_mode_requested", failure_count)
    if pending is None:
        selected = record["active"] if boot_slot_bootable(record, record["active"]) else None
        reason = "active_slot_selected" if selected else "selected_payload_slot_not_bootable"
        posture = "Normal" if selected else "Safe"
        return boot_control_decision(storage_slot, record, selected, posture, reason, None)
    if failure_count is not None and failure_count >= MAX_PENDING_BOOT_ATTEMPTS:
        selected = record["last_good"] if boot_slot_bootable(record, record["last_good"]) else None
        reason = "pending_failure_count_exhausted" if selected else "selected_payload_slot_not_bootable"
        posture = "Normal" if selected else "Safe"
        return boot_control_decision(storage_slot, record, selected, posture, reason, failure_count)
    if not record["boot_attempt"]["success_marked"]:
        selected = pending if boot_slot_bootable(record, pending) else None
        reason = "pending_slot_selected_for_probation" if selected else "selected_payload_slot_not_bootable"
        posture = "Probation" if selected else "Safe"
        return boot_control_decision(storage_slot, record, selected, posture, reason, failure_count)
    selected = record["active"] if boot_slot_bootable(record, record["active"]) else None
    reason = "pending_success_already_marked_read_only" if selected else "selected_payload_slot_not_bootable"
    posture = "Normal" if selected else "Safe"
    return boot_control_decision(storage_slot, record, selected, posture, reason, failure_count)


def boot_control_decision(
    storage_slot: str | None,
    record: dict[str, object] | None,
    selected_payload_slot: str | None,
    posture: str,
    reason: str,
    failure_count: int | None,
) -> dict[str, object]:
    return {
        "selected_storage_slot": storage_slot,
        "selected_payload_slot": selected_payload_slot,
        "posture": posture,
        "authoritative_bootctl_slot": storage_slot,
        "seq": None if record is None else record["seq"],
        "pending_consumed": False,
        "would_mark_good": False,
        "failure_count": failure_count,
        "reason": reason,
        "max_pending_boot_attempts": MAX_PENDING_BOOT_ATTEMPTS,
    }


def scan_boot_control(data: bytes) -> dict[str, object]:
    if len(data) != BOOTCTL_SLOT_SIZE * BOOTCTL_SLOT_COUNT:
        raise ValueError(f"BOOTCTL region length drift: {len(data)}")
    slot_a, reason_a = parse_boot_control_slot(data[:BOOTCTL_SLOT_SIZE])
    slot_b, reason_b = parse_boot_control_slot(data[BOOTCTL_SLOT_SIZE:])
    if slot_a and slot_b:
        if slot_a["seq"] == slot_b["seq"] and slot_a["storage_sha256"] != slot_b["storage_sha256"]:
            decision = boot_control_decision(None, None, None, "Safe", "ambiguous_boot_control_slots", None)
        elif int(slot_a["seq"]) >= int(slot_b["seq"]):
            decision = evaluate_boot_control_record("A", slot_a)
        else:
            decision = evaluate_boot_control_record("B", slot_b)
    elif slot_a:
        decision = evaluate_boot_control_record("A", slot_a)
    elif slot_b:
        decision = evaluate_boot_control_record("B", slot_b)
    elif reason_a == "zero_filled_slot" and reason_b == "zero_filled_slot":
        decision = boot_control_decision(None, None, None, "Safe", "no_valid_boot_control_slot", None)
    else:
        decision = boot_control_decision(None, None, None, "Safe", "both_slots_invalid", None)
    return {
        "storage_slot_a": slot_a or {"valid": False, "reason": reason_a},
        "storage_slot_b": slot_b or {"valid": False, "reason": reason_b},
        "decision": decision,
    }


def inspect_image(path: Path) -> dict[str, object]:
    size = path.stat().st_size
    if size and size % SECTOR_SIZE != 0:
        raise ValueError(f"image is not sector-aligned: {size}")
    with path.open("rb") as handle:
        primary_header_bytes = read_at(handle, PRIMARY_GPT_HEADER_LBA, SECTOR_SIZE)
        primary = parse_gpt_header(primary_header_bytes)
        lba_count = size // SECTOR_SIZE
        if lba_count == 0:
            lba_count = int(primary["backup_lba"]) + 1
        backup_header_bytes = read_at(handle, lba_count - 1, SECTOR_SIZE)
        backup = parse_gpt_header(backup_header_bytes)

        primary_entries = read_at(
            handle,
            int(primary["partition_entry_lba"]),
            int(primary["partition_entry_count"]) * int(primary["partition_entry_size"]),
        )
        backup_entries = read_at(
            handle,
            int(backup["partition_entry_lba"]),
            int(backup["partition_entry_count"]) * int(backup["partition_entry_size"]),
        )
        primary_entries_crc_ok = f"{crc32(primary_entries):08x}" == primary["partition_entry_array_crc32"]
        backup_entries_crc_ok = f"{crc32(backup_entries):08x}" == backup["partition_entry_array_crc32"]
        partitions = parse_partitions(primary_entries)
        seed_data = next(
            (
                part
                for part in partitions
                if part["type_guid"].lower() == str(SEED_DATA_TYPE_GUID)
                and part["name"] == "SEED_DATA"
            ),
            None,
        )

        sb0 = sb1 = None
        sb0_bytes = sb1_bytes = b""
        reclog_scan = None
        reclog_frame_summaries: list[dict[str, object]] = []
        artifact_persist_records: list[dict[str, object]] = []
        promotion_transaction_records: list[dict[str, object]] = []
        bootctl_read = None
        artstor_scan = None
        first_artstor_blob = None
        if seed_data is not None:
            sb0_bytes = read_at(handle, int(seed_data["first_lba"]), SECTOR_SIZE)
            sb1_bytes = read_at(handle, int(seed_data["first_lba"]) + 1, SECTOR_SIZE)
            sb0 = parse_superblock(sb0_bytes, int(seed_data["lba_count"]))
            sb1 = parse_superblock(sb1_bytes, int(seed_data["lba_count"]))
            bootctl_bytes = read_at(
                handle,
                int(seed_data["first_lba"]) + BOOTCTL_START_LBA,
                BOOTCTL_LBA_COUNT * SECTOR_SIZE,
            )
            bootctl_read = scan_boot_control(bootctl_bytes)
            reclog_bytes = read_at(
                handle,
                int(seed_data["first_lba"]) + RECLOG_START_LBA,
                RECLOG_LBA_COUNT * SECTOR_SIZE,
            )
            reclog_scan = scan_reclog(reclog_bytes)
            if reclog_scan["status"] in {"valid", "valid_empty"}:
                reclog_frames = parse_reclog_frames_for_inspection(reclog_bytes)
                reclog_frame_summaries = [
                    reclog_frame_summary(frame) for frame in reclog_frames
                ]
                artifact_persist_records = inspect_artifact_persist_records(
                    reclog_frames,
                    handle,
                    int(seed_data["first_lba"]),
                )
                promotion_transaction_records = inspect_promotion_transaction_records(reclog_frames)
            artstor_head = read_at(
                handle,
                int(seed_data["first_lba"]) + ARTSTOR_START_LBA,
                SECTOR_SIZE,
            )
            artstor_scan = scan_artstor_garbage(artstor_head)
            try:
                first_artstor_blob = parse_first_artstor_blob(handle, int(seed_data["first_lba"]))
            except ValueError as exc:
                first_artstor_blob = {"valid": False, "reason": str(exc)}

    gpt_header_valid = primary["signature"] == "EFI PART" and backup["signature"] == "EFI PART"
    gpt_crc_checked = bool(
        primary["crc_ok"]
        and backup["crc_ok"]
        and primary_entries_crc_ok
        and backup_entries_crc_ok
    )
    data_superblock_valid = bool(sb0 and sb1 and sb0["valid"] and sb1["valid"] and sb0_bytes == sb1_bytes)
    return {
        "path": str(path),
        "size_bytes": size,
        "sector_size": SECTOR_SIZE,
        "lba_count": lba_count,
        "gpt_header_valid": gpt_header_valid,
        "gpt_crc_checked": gpt_crc_checked,
        "gpt_seed_data_found": seed_data is not None,
        "data_superblock_valid": data_superblock_valid,
        "gpt": {
            "primary": primary,
            "backup": backup,
            "primary_entries_crc_ok": primary_entries_crc_ok,
            "backup_entries_crc_ok": backup_entries_crc_ok,
        },
        "partitions": partitions,
        "superblock": sb0,
        "superblock_copy": sb1,
        "bootctl_read": bootctl_read,
        "reclog_scan": reclog_scan,
        "reclog_frames": reclog_frame_summaries,
        "artstor_scan": artstor_scan,
        "first_artstor_blob": first_artstor_blob,
        "artifact_persist_records": artifact_persist_records,
        "promotion_transaction_records": promotion_transaction_records,
        "constants": {
            "BOOTCTL": {
                "start_lba": BOOTCTL_START_LBA,
                "lba_count": BOOTCTL_LBA_COUNT,
                "slot_size": BOOTCTL_SLOT_SIZE,
                "slot_header_len": BOOTCTL_SLOT_HEADER_LEN,
                "payload_len": BOOTCTL_PAYLOAD_LEN,
                "storage_slot_sectors": BOOTCTL_STORAGE_SLOT_SECTORS,
                "max_pending_boot_attempts": MAX_PENDING_BOOT_ATTEMPTS,
            },
            "RECLOG": {"start_lba": RECLOG_START_LBA, "lba_count": RECLOG_LBA_COUNT},
            "ARTSTOR": {"start_lba": ARTSTOR_START_LBA, "lba_count": ARTSTOR_LBA_COUNT},
        },
    }


def print_summary(info: dict[str, object]) -> None:
    print(f"wrote {info['path']} ({info['size_bytes']} bytes)")
    print("partition table:")
    for part in info["partitions"]:
        print(
            "  {index} {name} type={type_guid} first_lba={first_lba} "
            "last_lba={last_lba} lba_count={lba_count}".format(**part)
        )
    sb = info["superblock"]
    print("seed_data superblock:")
    print(
        "  magic={magic} version={version} header_len={header_len} sha256={sha256}".format(
            **sb
        )
    )
    for region in sb["regions"]:
        print("  {tag} start_lba={start_lba} lba_count={lba_count}".format(**region))
    scan = info.get("reclog_scan")
    if scan:
        print(
            "reclog scan: status={status} count={count} head_seq={head_seq} "
            "tail_seq={tail_seq} torn_tail={torn_tail} terminated_reason={terminated_reason}".format(
                **scan
            )
        )
    bootctl = info.get("bootctl_read")
    if bootctl:
        decision = bootctl["decision"]
        print(
            "bootctl read: posture={posture} authoritative={authoritative_bootctl_slot} "
            "selected_payload={selected_payload_slot} reason={reason}".format(**decision)
        )
    artstor = info.get("artstor_scan")
    if artstor and artstor.get("garbage_blob_present"):
        blob = artstor["garbage_blob"]
        print(
            "artstor scan: status=garbage offset={offset} frame_len={frame_len} "
            "authorizes_load=false".format(**blob)
        )
    print(f"superblock hex head: {sb['hex_head']}")


def validate_or_raise(info: dict[str, object]) -> None:
    required = (
        "gpt_header_valid",
        "gpt_crc_checked",
        "gpt_seed_data_found",
        "data_superblock_valid",
    )
    failed = [name for name in required if not info[name]]
    if failed:
        raise ValueError("self-check failed: " + ", ".join(failed))


def validate_reclog_fixture_or_raise(info: dict[str, object], fixture: ReclogFixture) -> None:
    scan = info.get("reclog_scan")
    if not isinstance(scan, dict):
        raise ValueError("self-check failed: reclog_scan missing")
    expected_status = "torn_tail" if fixture.torn_tail else ("valid_empty" if fixture.frame_count == 0 else "valid")
    failures = []
    if scan.get("status") != expected_status:
        failures.append(f"status expected {expected_status} got {scan.get('status')}")
    if int(scan.get("count", -1)) != fixture.frame_count:
        failures.append(f"count expected {fixture.frame_count} got {scan.get('count')}")
    if bool(scan.get("torn_tail")) != fixture.torn_tail:
        failures.append(f"torn_tail expected {fixture.torn_tail} got {scan.get('torn_tail')}")
    if fixture.frame_count > 0 and int(scan.get("head_seq", 0)) != 1:
        failures.append(f"head_seq expected 1 got {scan.get('head_seq')}")
    if fixture.frame_count > 0 and int(scan.get("tail_seq", 0)) != fixture.frame_count:
        failures.append(f"tail_seq expected {fixture.frame_count} got {scan.get('tail_seq')}")
    if failures:
        raise ValueError("RECLOG fixture self-check failed: " + "; ".join(failures))


def validate_boot_control_fixture_or_raise(info: dict[str, object], spec: str | None) -> None:
    bootctl = info.get("bootctl_read")
    if not isinstance(bootctl, dict):
        raise ValueError("self-check failed: bootctl_read missing")
    decision = bootctl.get("decision")
    if not isinstance(decision, dict):
        raise ValueError("self-check failed: bootctl decision missing")
    normalized = "none" if spec is None or spec == "" else spec.strip().lower()
    expected = {
        "none": ("Safe", None, None, "no_valid_boot_control_slot"),
        "empty": ("Safe", None, None, "no_valid_boot_control_slot"),
        "valid-a": ("Normal", "A", "A", "active_slot_selected"),
        "ab": ("Normal", "B", "B", "active_slot_selected"),
        "both-invalid": ("Safe", None, None, "both_slots_invalid"),
        "pending": ("Probation", "A", "B", "pending_slot_selected_for_probation"),
        "pending-safe": ("Safe", "A", None, "safe_mode_requested"),
        "pending-exhausted": ("Normal", "A", "A", "pending_failure_count_exhausted"),
    }
    if normalized not in expected:
        raise ValueError(f"unsupported BOOTCTL fixture spec: {spec}")
    posture, authoritative, selected_payload, reason = expected[normalized]
    failures = []
    if decision.get("posture") != posture:
        failures.append(f"posture expected {posture} got {decision.get('posture')}")
    if decision.get("authoritative_bootctl_slot") != authoritative:
        failures.append(
            f"authoritative expected {authoritative} got {decision.get('authoritative_bootctl_slot')}"
        )
    if decision.get("selected_payload_slot") != selected_payload:
        failures.append(
            f"selected_payload expected {selected_payload} got {decision.get('selected_payload_slot')}"
        )
    if decision.get("reason") != reason:
        failures.append(f"reason expected {reason} got {decision.get('reason')}")
    if bool(decision.get("pending_consumed")):
        failures.append("pending_consumed expected false")
    if bool(decision.get("would_mark_good")):
        failures.append("would_mark_good expected false")
    if int(decision.get("max_pending_boot_attempts", -1)) != MAX_PENDING_BOOT_ATTEMPTS:
        failures.append(
            "max_pending_boot_attempts expected "
            f"{MAX_PENDING_BOOT_ATTEMPTS} got {decision.get('max_pending_boot_attempts')}"
        )
    if normalized == "pending-exhausted" and int(decision.get("failure_count", -1)) != MAX_PENDING_BOOT_ATTEMPTS:
        failures.append(
            "failure_count expected "
            f"{MAX_PENDING_BOOT_ATTEMPTS} got {decision.get('failure_count')}"
        )
    if failures:
        raise ValueError("BOOTCTL fixture self-check failed: " + "; ".join(failures))


def validate_artstor_garbage_fixture_or_raise(info: dict[str, object], expected: bool) -> None:
    scan = info.get("artstor_scan")
    if not isinstance(scan, dict):
        raise ValueError("self-check failed: artstor_scan missing")
    present = bool(scan.get("garbage_blob_present"))
    if present != expected:
        raise ValueError(
            f"ARTSTOR garbage fixture self-check failed: expected present={expected} got {present}"
        )
    if expected:
        blob = scan.get("garbage_blob")
        if not isinstance(blob, dict):
            raise ValueError("ARTSTOR garbage fixture self-check failed: blob missing")
        if blob.get("status") != "garbage" or bool(blob.get("authorizes_load")):
            raise ValueError("ARTSTOR garbage fixture self-check failed: blob is not inert garbage")
        reclog = info.get("reclog_scan")
        if not isinstance(reclog, dict) or int(reclog.get("count", -1)) != 0:
            raise ValueError("ARTSTOR garbage fixture self-check failed: RECLOG record unexpectedly present")


def run_standalone_self_check() -> None:
    frame1 = build_reclog_frame(1, b"\0" * 32, reclog_payload(1))
    frame2 = build_reclog_frame(2, hashlib.sha256(frame1).digest(), reclog_payload(2))
    region = frame1 + frame2 + (b"\0" * SECTOR_SIZE)
    scan = scan_reclog(region)
    if scan.get("status") != "valid" or int(scan.get("count", -1)) != 2:
        raise ValueError(f"standalone self-check failed: valid RECLOG scan {scan}")

    torn = bytearray(region)
    torn[len(frame1) + RECLOG_HEADER_LEN] ^= 0x01
    torn_scan = scan_reclog(bytes(torn))
    if torn_scan.get("status") != "torn_tail" or int(torn_scan.get("count", -1)) != 1:
        raise ValueError(f"standalone self-check failed: torn RECLOG scan {torn_scan}")

    slot_a, slot_b = boot_control_fields_for_fixture("valid-a")
    bootctl = scan_boot_control(slot_a + slot_b + (b"\0" * (BOOTCTL_LBA_COUNT * SECTOR_SIZE - len(slot_a) - len(slot_b))))
    decision = bootctl.get("decision") or {}
    if decision.get("posture") != "Normal" or decision.get("authoritative_bootctl_slot") != "A":
        raise ValueError(f"standalone self-check failed: BOOTCTL scan {bootctl}")


def run_buildfs_seed_self_check() -> None:
    repository = Path(__file__).resolve().parents[1]
    root = Path(tempfile.gettempdir()) / f"raios-buildfs-seed-{uuid.uuid4().hex}"
    root.mkdir()
    try:
        cargo_environment = os.environ.copy()
        cargo_environment["CARGO_HOME"] = str(Path.home() / ".cargo")
        cargo_environment["CARGO_TARGET_DIR"] = str(repository / "target")

        def pack_fixture(name: str, files: dict[str, bytes]) -> tuple[Path, str]:
            source = root / f"{name}-source"
            packed = root / f"{name}-packed"
            source.mkdir()
            for relative_path, payload in files.items():
                target = source / relative_path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(payload)
            packed_result = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "-p",
                    "buildfs-pack",
                    "--",
                    str(source),
                    "--output",
                    str(packed),
                ],
                cwd=repository,
                env=cargo_environment,
                capture_output=True,
                text=True,
                check=False,
            )
            if packed_result.returncode != 0:
                detail = packed_result.stderr.strip() or packed_result.stdout.strip()
                raise ValueError(f"buildfs-pack {name} self-check fixture failed: {detail}")
            expected = (packed / "manifest.sha256").read_text(encoding="ascii").strip()
            return packed, expected

        packed_one, expected_one = pack_fixture(
            "tree-one",
            {
                "alpha.txt": b"alpha-only\n",
                "shared.txt": b"shared-across-trees\n",
                "nested/multi.bin": bytes(range(256)) * 257,
            },
        )
        packed_two, expected_two = pack_fixture(
            "tree-two",
            {
                "beta.txt": b"beta-only\n",
                "shared.txt": b"shared-across-trees\n",
            },
        )
        seeds = prepare_buildfs_seeds(
            ((packed_one, expected_one), (packed_two, expected_two))
        )
        image = root / "persist.img"
        build_image(image, buildfs_seeds=seeds)
        validate_buildfs_seed_trees_image(image, seeds)

        expected_frames = b"".join(
            build_artifact_blob_frame(entry.payload, artifact_blob_frame_len(len(entry.payload)))
            for seed in seeds
            for entry in seed.payloads
        )
        with image.open("rb") as handle:
            actual_frames = read_artstor_blob(
                handle,
                SEED_DATA_START_LBA,
                0,
                len(expected_frames) + SECTOR_SIZE,
            )
        if actual_frames != expected_frames + (b"\0" * SECTOR_SIZE):
            raise ValueError("BuildFS multi-tree frames are not dense with one trailing zero sector")

        resolved_offsets: list[int] = []
        for tree_index, (packed, file_path, expected_payload) in enumerate(
            (
                (packed_one, "alpha.txt", b"alpha-only\n"),
                (packed_two, "beta.txt", b"beta-only\n"),
            )
        ):
            manifest_index = parse_buildfs_manifest_index((packed / "manifest.bin").read_bytes())
            reference = next(
                reference
                for reference in manifest_index.chunk_references
                if reference.file_path == file_path
            )
            with image.open("rb") as handle:
                resolved, failure, _found_len = resolve_buildfs_chunk_frame_from_image(
                    handle,
                    SEED_DATA_START_LBA,
                    reference.sha256,
                    reference.length,
                )
                if failure is not None or resolved is None:
                    raise ValueError(
                        f"BuildFS tree {tree_index + 1} self-check chunk did not resolve: {failure}"
                    )
                payload = read_resolved_buildfs_chunk_from_image(
                    handle,
                    SEED_DATA_START_LBA,
                    resolved,
                )
            if payload != expected_payload:
                raise ValueError(f"BuildFS tree {tree_index + 1} resolved the wrong chunk payload")
            if tree_index == 1 and resolved.frame_offset < seeds[0].frame_bytes:
                raise ValueError("BuildFS tree 2 chunk resolved before the tree 2 frame span")
            resolved_offsets.append(resolved.frame_offset)

        shared_sha256 = hashlib.sha256(b"shared-across-trees\n").digest()
        duplicate_frame_count = sum(
            entry.sha256 == shared_sha256
            for seed in seeds
            for entry in seed.payloads
        )
        if duplicate_frame_count != 2:
            raise ValueError("BuildFS cross-tree duplicate chunk was not written once per tree")

        single_image = root / "single-persist.img"
        build_image(single_image, buildfs_seed=seeds[0])
        validate_buildfs_seed_image(single_image, seeds[0])
        expected_single_frames = expected_frames[: seeds[0].frame_bytes]
        with single_image.open("rb") as handle:
            actual_single_frames = read_artstor_blob(
                handle,
                SEED_DATA_START_LBA,
                0,
                seeds[0].frame_bytes + SECTOR_SIZE,
            )
        if actual_single_frames != expected_single_frames + (b"\0" * SECTOR_SIZE):
            raise ValueError("BuildFS single-tree byte layout regressed")

        try:
            pair_buildfs_seed_arguments(
                [packed_one, packed_two],
                [expected_one],
            )
        except ValueError as exc:
            if "must be repeated equally" not in str(exc):
                raise
        else:
            raise ValueError("BuildFS self-check accepted unpaired multi-tree CLI arguments")

        try:
            prepare_buildfs_seed(packed_one, "00" * 32)
        except ValueError as exc:
            if "manifest SHA-256 mismatch" not in str(exc):
                raise
        else:
            raise ValueError("BuildFS self-check accepted the wrong manifest pin")

        first_chunk = sorted((packed_one / "chunks").iterdir(), key=lambda path: path.name)[0]
        tampered = bytearray(first_chunk.read_bytes())
        tampered[0] ^= 1
        first_chunk.write_bytes(tampered)
        try:
            prepare_buildfs_seed(packed_one, expected_one)
        except ValueError as exc:
            if "chunk SHA-256 mismatch" not in str(exc):
                raise
        else:
            raise ValueError("BuildFS self-check accepted a tampered chunk")

        try:
            ensure_artstor_seed_fits(ARTSTOR_LBA_COUNT * SECTOR_SIZE + SECTOR_SIZE)
        except ValueError as exc:
            if "exceed ARTSTOR region" not in str(exc):
                raise
        else:
            raise ValueError("BuildFS self-check accepted an oversized ARTSTOR span")

        tree_two_first_payload = (
            (SEED_DATA_START_LBA + ARTSTOR_START_LBA) * SECTOR_SIZE
            + seeds[0].frame_bytes
            + ARTIFACT_BLOB_HEADER_LEN
        )
        with image.open("r+b") as handle:
            handle.seek(tree_two_first_payload)
            original_byte = handle.read(1)
            if len(original_byte) != 1:
                raise ValueError("BuildFS self-check could not read the tree 2 tamper byte")
            handle.seek(tree_two_first_payload)
            handle.write(bytes((original_byte[0] ^ 1,)))
        try:
            validate_buildfs_seed_trees_image(image, seeds)
        except ValueError as exc:
            if "tree 2 ARTSTOR frame failed offline parse" not in str(exc):
                raise
        else:
            raise ValueError("BuildFS self-check accepted a tampered tree 2 image frame")

        print(
            "buildfs self-check: passed "
            f"trees={len(seeds)} manifests={expected_one},{expected_two} "
            f"frames={sum(len(seed.payloads) for seed in seeds)} "
            f"frame_bytes={sum(seed.frame_bytes for seed in seeds)} "
            f"resolved_tree1_offset={resolved_offsets[0]} "
            f"resolved_tree2_offset={resolved_offsets[1]} "
            f"duplicate_frames={duplicate_frame_count} trailing_zero=passed "
            "single_tree_regression=passed "
            "negative=pair_count_mismatch+manifest_mismatch+chunk_hash_mismatch+"
            "region_overflow+tree2_frame_tamper"
        )
    finally:
        shutil.rmtree(root)


def pair_buildfs_seed_arguments(
    buildfs_directories: list[Path] | None,
    expected_manifest_sha256: list[str] | None,
) -> tuple[tuple[Path, str], ...]:
    directories = tuple(buildfs_directories or ())
    expected_hashes = tuple(expected_manifest_sha256 or ())
    if len(directories) != len(expected_hashes):
        raise ValueError(
            "--seed-buildfs and --expect-manifest-sha256 must be repeated equally: "
            f"got {len(directories)} director{'y' if len(directories) == 1 else 'ies'} "
            f"and {len(expected_hashes)} manifest pin{'s' if len(expected_hashes) != 1 else ''}"
        )
    return tuple(zip(directories, expected_hashes))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-check", action="store_true", help="accepted for explicit self-check invocations")
    parser.add_argument(
        "--self-check-buildfs",
        action="store_true",
        help="pack a small synthetic BuildFS tree, seed it, and re-read every ARTSTOR frame",
    )
    parser.add_argument("--inspect-json", type=Path, help="inspect an existing image and print JSON")
    parser.add_argument(
        "--resolve-buildfs",
        type=Path,
        metavar="IMAGE",
        help="resolve every manifest chunk through the kernel-shaped ARTSTOR scan/read path",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="raios.buildfs_manifest.v1 input for --resolve-buildfs",
    )
    parser.add_argument("--assert-not-release", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--image", type=Path, help="existing GPT persist image for offline slot updates")
    parser.add_argument("--corrupt-artstor-blob", type=Path, help="mutate the first ARTSTOR blob payload in-place")
    parser.add_argument("--tamper-persist-record", type=Path, help="mutate the first artifact_persist RECLOG record in-place")
    parser.add_argument("--tamper-wasm-grant-event", type=Path, help="break a revoke parent hash while preserving a valid RECLOG chain")
    parser.add_argument("--tamper-rollback-target-snapshot", type=Path, help="raw-bitflip the unsigned predecessor snapshot hash while preserving RECLOG framing")
    parser.add_argument("--reseal-rollback-target-snapshot", type=Path, help="replace the unsigned predecessor snapshot with a different canonical snapshot and fully rechain RECLOG")
    parser.add_argument("--tamper-rollback-target-artstor-link", type=Path, help="mutate the predecessor ARTSTOR link while preserving RECLOG framing")
    parser.add_argument("--tamper-rollback-target-signature", type=Path, help="mutate the predecessor install signature while preserving RECLOG framing")
    parser.add_argument("--duplicate-authenticated-install", type=Path, help="duplicate one authentic authorization+promotion+persist triple and rechain only RECLOG framing")
    parser.add_argument("--forge-rollback-unchanged-projection", type=Path, help="remove the rollback revoke and reseal the unsigned commit to claim the unchanged source projection")
    parser.add_argument("--mismatch-rollback-intent-commit", type=Path, help="reseal a commit with the same transaction id but mismatched intent semantics")
    parser.add_argument("--corrupt-memory-frame", type=Path, help="flip one byte in the last RECLOG frame payload in-place")
    parser.add_argument("--stage-slot", choices=("A", "B"), help="stage --payload-dir into this ESP and set it pending")
    parser.add_argument("--payload-dir", type=Path, help="directory to stage into the selected ESP")
    parser.add_argument("--set-pending", choices=("A", "B"), help="set pending to this slot without staging files")
    parser.add_argument(
        "--seed-reclog-fixture",
        default="empty",
        help="seed RECLOG fixture: empty, valid:N, full, or valid:N,torn",
    )
    parser.add_argument(
        "--seed-bootctl",
        default=None,
        help="seed BOOTCTL fixture: valid-a, ab, both-invalid, pending, pending-safe, or pending-exhausted",
    )
    parser.add_argument(
        "--seed-artstor-garbage-blob",
        action="store_true",
        help="seed a bare RAIOSAR0 blob in ARTSTOR without a chained artifact_persist RECLOG record",
    )
    parser.add_argument(
        "--seed-dev-rollback-fixture",
        type=Path,
        metavar="WASM",
        help="seed the fixed signed log-only DEV rollback predecessor generated by dev-rollback-fixture",
    )
    parser.add_argument(
        "--dev-rollback-fixture-tool",
        type=Path,
        metavar="EXE",
        help="exact dev-rollback-fixture executable required with --seed-dev-rollback-fixture",
    )
    parser.add_argument(
        "--seed-buildfs",
        type=Path,
        action="append",
        metavar="DIR",
        help=(
            "seed a buildfs-pack output directory as dense RAIOSAR0 ARTSTOR frames; "
            "repeat with one --expect-manifest-sha256 per tree"
        ),
    )
    parser.add_argument(
        "--expect-manifest-sha256",
        action="append",
        metavar="HEX",
        help="required SHA-256 pin for the corresponding --seed-buildfs manifest.bin; repeat in order",
    )
    parser.add_argument("output", nargs="?", type=Path)
    args = parser.parse_args()
    mutation_targets = (
        args.corrupt_artstor_blob, args.tamper_persist_record,
        args.tamper_wasm_grant_event, args.tamper_rollback_target_snapshot,
        args.reseal_rollback_target_snapshot, args.tamper_rollback_target_artstor_link,
        args.tamper_rollback_target_signature, args.duplicate_authenticated_install,
        args.forge_rollback_unchanged_projection, args.mismatch_rollback_intent_commit,
        args.corrupt_memory_frame,
    )
    mutation_count = sum(target is not None for target in mutation_targets)
    seed_requested = bool(
        args.seed_dev_rollback_fixture or args.seed_buildfs
        or args.seed_artstor_garbage_blob
        or args.seed_reclog_fixture not in {None, "", "empty"}
    )
    if mutation_count > 1:
        parser.error("choose only one tamper subcommand")
    if mutation_count and (
        seed_requested or args.seed_bootctl or args.image or args.stage_slot
        or args.payload_dir or args.set_pending or args.output
        or args.inspect_json or args.resolve_buildfs or args.self_check
        or args.self_check_buildfs
    ):
        parser.error("tamper subcommands cannot be combined with seed, update, inspect, or create operations")
    if seed_requested and (
        args.inspect_json or args.resolve_buildfs or args.self_check_buildfs
        or args.image or args.stage_slot or args.payload_dir or args.set_pending
    ):
        parser.error("seed operations are valid only while creating a new positional output image")
    try:
        buildfs_seed_pairs = pair_buildfs_seed_arguments(
            args.seed_buildfs,
            args.expect_manifest_sha256,
        )
    except ValueError as exc:
        parser.error(str(exc))

    try:
        if (args.resolve_buildfs is None) != (args.manifest is None):
            parser.error("--resolve-buildfs and --manifest are required together")
        if args.seed_buildfs and args.seed_artstor_garbage_blob:
            parser.error("--seed-buildfs cannot be combined with --seed-artstor-garbage-blob")
        if (args.seed_dev_rollback_fixture is None) != (args.dev_rollback_fixture_tool is None):
            parser.error(
                "--seed-dev-rollback-fixture and --dev-rollback-fixture-tool are required together"
            )
        if args.seed_dev_rollback_fixture and (
            args.seed_buildfs
            or args.seed_artstor_garbage_blob
            or args.seed_reclog_fixture not in {None, "", "empty"}
        ):
            parser.error(
                "--seed-dev-rollback-fixture requires otherwise-empty RECLOG and ARTSTOR"
            )
        if args.seed_buildfs and (
            args.inspect_json
            or args.assert_not_release
            or args.image
            or args.corrupt_artstor_blob
            or args.tamper_persist_record
            or args.tamper_wasm_grant_event
            or args.forge_rollback_unchanged_projection
            or args.mismatch_rollback_intent_commit
            or args.corrupt_memory_frame
            or args.stage_slot
            or args.payload_dir
            or args.set_pending
        ):
            parser.error("--seed-buildfs is valid only while creating a new positional output image")
        if args.seed_buildfs and args.output is None:
            parser.error("--seed-buildfs requires a positional output image")
        if args.resolve_buildfs is not None:
            if (
                args.self_check
                or args.self_check_buildfs
                or args.inspect_json
                or args.assert_not_release
                or args.image
                or args.corrupt_artstor_blob
                or args.tamper_persist_record
                or args.tamper_wasm_grant_event
                or args.tamper_rollback_target_snapshot
                or args.reseal_rollback_target_snapshot
                or args.tamper_rollback_target_artstor_link
                or args.tamper_rollback_target_signature
                or args.duplicate_authenticated_install
                or args.forge_rollback_unchanged_projection
                or args.mismatch_rollback_intent_commit
                or args.corrupt_memory_frame
                or args.stage_slot
                or args.payload_dir
                or args.set_pending
                or args.seed_artstor_garbage_blob
                or args.seed_buildfs
                or args.expect_manifest_sha256
                or args.output
            ):
                parser.error("--resolve-buildfs cannot be combined with other operations")
            diagnose_buildfs_resolution(args.resolve_buildfs, args.manifest)
            return 0
        if args.self_check_buildfs:
            if (
                args.self_check
                or args.inspect_json
                or args.assert_not_release
                or args.image
                or args.corrupt_artstor_blob
                or args.tamper_persist_record
                or args.tamper_wasm_grant_event
                or args.tamper_rollback_target_snapshot
                or args.reseal_rollback_target_snapshot
                or args.tamper_rollback_target_artstor_link
                or args.tamper_rollback_target_signature
                or args.duplicate_authenticated_install
                or args.forge_rollback_unchanged_projection
                or args.mismatch_rollback_intent_commit
                or args.corrupt_memory_frame
                or args.stage_slot
                or args.payload_dir
                or args.set_pending
                or args.seed_buildfs
                or args.output
            ):
                parser.error("--self-check-buildfs cannot be combined with other operations")
            run_buildfs_seed_self_check()
            return 0
        if args.assert_not_release:
            assert_not_release_output(args.assert_not_release)
            print("release guard: passed")
            return 0
        if args.inspect_json:
            if (
                args.image
                or args.corrupt_artstor_blob
                or args.tamper_persist_record
                or args.duplicate_authenticated_install
                or args.forge_rollback_unchanged_projection
                or args.mismatch_rollback_intent_commit
                or args.corrupt_memory_frame
                or args.stage_slot
                or args.payload_dir
                or args.set_pending
                or args.seed_buildfs
            ):
                parser.error("--inspect-json cannot be combined with offline mutation flags")
            print(json.dumps(inspect_image(args.inspect_json), indent=2))
            return 0
        if (args.corrupt_artstor_blob or args.tamper_persist_record or
                args.tamper_wasm_grant_event or args.corrupt_memory_frame or
                args.tamper_rollback_target_snapshot or
                args.reseal_rollback_target_snapshot or
                args.tamper_rollback_target_artstor_link or
                args.tamper_rollback_target_signature or
                args.duplicate_authenticated_install or
                args.forge_rollback_unchanged_projection or
                args.mismatch_rollback_intent_commit):
            mutation_targets = [
                args.corrupt_artstor_blob,
                args.tamper_persist_record,
                args.tamper_wasm_grant_event,
                args.tamper_rollback_target_snapshot,
                args.reseal_rollback_target_snapshot,
                args.tamper_rollback_target_artstor_link,
                args.tamper_rollback_target_signature,
                args.duplicate_authenticated_install,
                args.forge_rollback_unchanged_projection,
                args.mismatch_rollback_intent_commit,
                args.corrupt_memory_frame,
            ]
            if sum(target is not None for target in mutation_targets) != 1:
                parser.error("choose only one tamper subcommand")
            if args.image or args.stage_slot or args.payload_dir or args.set_pending or args.output:
                parser.error("tamper subcommands cannot be combined with other image update flags")
            if args.corrupt_artstor_blob:
                result = corrupt_artstor_blob(args.corrupt_artstor_blob)
            elif args.tamper_persist_record:
                result = tamper_persist_record(args.tamper_persist_record)
            elif args.tamper_wasm_grant_event:
                result = tamper_wasm_grant_event(args.tamper_wasm_grant_event)
            elif args.tamper_rollback_target_snapshot:
                result = tamper_rollback_fixture_field(args.tamper_rollback_target_snapshot, "target_snapshot")
            elif args.reseal_rollback_target_snapshot:
                result = reseal_rollback_fixture_snapshot(args.reseal_rollback_target_snapshot)
            elif args.tamper_rollback_target_artstor_link:
                result = tamper_rollback_fixture_field(args.tamper_rollback_target_artstor_link, "artstor_link")
            elif args.tamper_rollback_target_signature:
                result = tamper_rollback_fixture_field(args.tamper_rollback_target_signature, "promotion_signature")
            elif args.duplicate_authenticated_install:
                result = duplicate_authenticated_install(args.duplicate_authenticated_install)
            elif args.forge_rollback_unchanged_projection:
                result = forge_rollback_recovery(
                    args.forge_rollback_unchanged_projection,
                    "unchanged_projection",
                )
            elif args.mismatch_rollback_intent_commit:
                result = forge_rollback_recovery(
                    args.mismatch_rollback_intent_commit,
                    "pair_mismatch",
                )
            else:
                result = corrupt_memory_frame(args.corrupt_memory_frame)
            print(json.dumps(result, indent=2))
            return 0
        if args.image and args.seed_bootctl:
            if args.stage_slot or args.payload_dir or args.set_pending or args.output:
                parser.error("--image --seed-bootctl cannot be combined with slot update flags")
            print(json.dumps(apply_bootctl_fixture_update(args.image, args.seed_bootctl), indent=2))
            return 0
        if args.image or args.stage_slot or args.payload_dir or args.set_pending:
            image = args.image or args.output
            if image is None:
                parser.error("--image or positional image path is required for offline slot updates")
            result = apply_boot_slot_update(image, args.stage_slot, args.payload_dir, args.set_pending)
            print(json.dumps(result, indent=2))
            return 0
        def build_validate_print(output: Path) -> int:
            reclog_fixture = parse_reclog_fixture(args.seed_reclog_fixture)
            dev_rollback_seed = (
                prepare_dev_rollback_seed(
                    args.dev_rollback_fixture_tool,
                    args.seed_dev_rollback_fixture,
                )
                if args.seed_dev_rollback_fixture
                else None
            )
            buildfs_seeds = (
                prepare_buildfs_seeds(buildfs_seed_pairs)
                if buildfs_seed_pairs
                else ()
            )
            build_image(
                output,
                args.seed_reclog_fixture,
                args.seed_bootctl,
                args.seed_artstor_garbage_blob,
                buildfs_seeds=buildfs_seeds or None,
                dev_rollback_seed=dev_rollback_seed,
            )
            info = inspect_image(output)
            validate_or_raise(info)
            if dev_rollback_seed is None:
                validate_reclog_fixture_or_raise(info, reclog_fixture)
            else:
                validate_dev_rollback_seed_image(output, dev_rollback_seed)
            validate_boot_control_fixture_or_raise(info, args.seed_bootctl)
            if buildfs_seeds:
                validate_buildfs_seed_trees_image(output, buildfs_seeds)
            elif dev_rollback_seed is None:
                validate_artstor_garbage_fixture_or_raise(info, args.seed_artstor_garbage_blob)
            print_summary(info)
            print(
                f"reclog fixture: frames_seeded={reclog_fixture.frame_count} "
                f"torn_tail={str(reclog_fixture.torn_tail).lower()} validation=passed"
            )
            print(
                "bootctl fixture: "
                f"spec={args.seed_bootctl or 'none'} "
                f"max_pending_boot_attempts={MAX_PENDING_BOOT_ATTEMPTS} validation=passed"
            )
            print(
                "artstor garbage fixture: "
                f"seeded={str(args.seed_artstor_garbage_blob).lower()} validation=passed"
            )
            if dev_rollback_seed is not None:
                print(
                    "dev rollback fixture: "
                    f"artifact={dev_rollback_seed.metadata['artifact_sha256']} "
                    "records=4 target=env.log signatures=2 validation=passed"
                )
            if len(buildfs_seeds) == 1:
                buildfs_seed = buildfs_seeds[0]
                print(
                    "buildfs seed: "
                    f"manifest={buildfs_seed.manifest_sha256.hex()} "
                    f"chunks={len(buildfs_seed.payloads) - 1} "
                    f"frames={len(buildfs_seed.payloads)} "
                    f"frame_bytes={buildfs_seed.frame_bytes} validation=passed"
                )
            elif buildfs_seeds:
                print(
                    "buildfs seed: "
                    f"trees={len(buildfs_seeds)} "
                    f"frames={sum(len(seed.payloads) for seed in buildfs_seeds)} "
                    f"frame_bytes={sum(seed.frame_bytes for seed in buildfs_seeds)} "
                    "validation=passed"
                )
                for tree_index, seed in enumerate(buildfs_seeds):
                    print(
                        f"buildfs tree {tree_index + 1}: "
                        f"manifest={seed.manifest_sha256.hex()} "
                        f"chunks={len(seed.payloads) - 1} "
                        f"frames={len(seed.payloads)} "
                        f"frame_bytes={seed.frame_bytes}"
                    )
            print("self-check: passed")
            return 0

        if args.output is None:
            if not args.self_check:
                parser.error("output path is required unless --inspect-json is used")
            run_standalone_self_check()
            print("self-check: passed")
            return 0
        return build_validate_print(args.output)
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
