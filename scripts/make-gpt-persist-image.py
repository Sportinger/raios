#!/usr/bin/env python3
"""Build and validate the raiOS M7 GPT persistence test disk."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import importlib.util
import json
import os
import struct
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path

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


def validate_existing_gpt_image(image: Path) -> dict[str, object]:
    info = inspect_image(image)
    required = ("gpt_header_valid", "gpt_seed_data_found", "data_superblock_valid")
    failed = [name for name in required if not info.get(name)]
    if failed:
        raise ValueError(f"refusing non-GPT or unprovisioned persist image: {', '.join(failed)}")
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


def build_image(
    output: Path,
    reclog_fixture_spec: str | None = None,
    bootctl_fixture_spec: str | None = None,
) -> None:
    assert_not_release_output(output)
    if ARTSTOR_LBA_COUNT <= 0:
        raise RuntimeError("SEED_DATA is too small for ARTSTOR")

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
    if size % SECTOR_SIZE != 0:
        raise ValueError(f"image is not sector-aligned: {size}")
    lba_count = size // SECTOR_SIZE
    with path.open("rb") as handle:
        primary_header_bytes = read_at(handle, PRIMARY_GPT_HEADER_LBA, SECTOR_SIZE)
        backup_header_bytes = read_at(handle, lba_count - 1, SECTOR_SIZE)
        primary = parse_gpt_header(primary_header_bytes)
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
        bootctl_read = None
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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-check", action="store_true", help="accepted for explicit self-check invocations")
    parser.add_argument("--inspect-json", type=Path, help="inspect an existing image and print JSON")
    parser.add_argument("--assert-not-release", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--image", type=Path, help="existing GPT persist image for offline slot updates")
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
    parser.add_argument("output", nargs="?", type=Path)
    args = parser.parse_args()

    try:
        if args.assert_not_release:
            assert_not_release_output(args.assert_not_release)
            print("release guard: passed")
            return 0
        if args.inspect_json:
            if args.image or args.stage_slot or args.payload_dir or args.set_pending:
                parser.error("--inspect-json cannot be combined with offline slot update flags")
            print(json.dumps(inspect_image(args.inspect_json), indent=2))
            return 0
        if args.image or args.stage_slot or args.payload_dir or args.set_pending:
            image = args.image or args.output
            if image is None:
                parser.error("--image or positional image path is required for offline slot updates")
            result = apply_boot_slot_update(image, args.stage_slot, args.payload_dir, args.set_pending)
            print(json.dumps(result, indent=2))
            return 0
        if args.output is None:
            parser.error("output path is required unless --inspect-json is used")
        reclog_fixture = parse_reclog_fixture(args.seed_reclog_fixture)
        build_image(args.output, args.seed_reclog_fixture, args.seed_bootctl)
        info = inspect_image(args.output)
        validate_or_raise(info)
        validate_reclog_fixture_or_raise(info, reclog_fixture)
        validate_boot_control_fixture_or_raise(info, args.seed_bootctl)
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
        print("self-check: passed")
        return 0
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
