"""Pure fail-closed validators for the deterministic rollback image contract."""

from __future__ import annotations

import binascii
import hashlib
import json
import struct
import uuid
from typing import NamedTuple


class SecurityError(ValueError):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


class FixtureResult(NamedTuple):
    artifact_sha256: bytes
    reclog_tail_sha256: bytes
    artstor_frame_sha256: bytes


class GptInput(NamedTuple):
    size: int
    primary_header: bytes
    primary_entries: bytes
    backup_entries: bytes
    backup_header: bytes


class PartitionIdentity(NamedTuple):
    name: str
    type_guid: uuid.UUID
    unique_guid: uuid.UUID
    first_lba: int
    last_lba: int


class RecoveryBinding(NamedTuple):
    transaction_id: str
    service_id: str
    domain_instance: int
    source_projection_sha256: str
    result_projection_sha256: str
    target_snapshot_sha256: str
    delta_count: int
    delta_sha256: str
    parent_grant_id: str
    parent_grant_sha256: str
    member_binding_sha256: str
    host_import_id: str
    scope: str
    generation: int
    epoch: int
    intent_id: str
    commit_id: str


class RecoveryPlan(NamedTuple):
    intent_index: int
    revoke_index: int
    commit_index: int
    transaction_id: str
    parent_grant_id: str
    delta_sha256: str


class RecoveryTarget(NamedTuple):
    service_id: str; domain_instance: int; generation: int; artifact_sha256: str
    artifact_byte_len: int; entries: tuple[tuple[str, str], ...]; source_generation: int
    source_artifact_sha256: str; source_artifact_byte_len: int
    source_entries: tuple[tuple[str, str], ...]


DETERMINISTIC_ROLLBACK_TARGET = RecoveryTarget(
    "svc.dev.granted_candidate", 1, 1,
    "sha256:33ea9dc8f8ecd039236673fafdffcf63e8691a1d352cffee4ddf47531cb5756c", 121,
    (("env.log", "host_call"),),
    2, "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2", 4205,
    (("env.log", "host_call"), ("env.counter_get", "host_call")),
)
def _need(condition: bool, code: str) -> None:
    if not condition:
        raise SecurityError(code)


def _canon(value: object) -> bytes:
    return (json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True, allow_nan=False) + "\n").encode()


def _json(payload: bytes) -> dict[str, object]:
    def unique(pairs):
        out = {}
        for key, value in pairs:
            _need(key not in out, "fixture_json_duplicate")
            out[key] = value
        return out
    try:
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=unique,
                           parse_constant=lambda _value: (_ for _ in ()).throw(ValueError()))
        rendered = _canon(value)
    except SecurityError:
        raise
    except (UnicodeError, json.JSONDecodeError, ValueError, RecursionError) as exc:
        raise SecurityError("fixture_json") from exc
    _need(type(value) is dict and rendered == payload, "fixture_json_canonical")
    return value


_P = 0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
_N = 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
_G = (0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296,
      0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5)
_Q = _G
_KEY_HASH = "sha256:698bea63dc44a344663ff1429aea10842df27b6b991ef25866b2c6c02cdcc5be"


def _add(left, right):
    if left is None:
        return right
    if right is None:
        return left
    x1, y1 = left
    x2, y2 = right
    if x1 == x2 and (y1 + y2) % _P == 0:
        return None
    if left == right:
        _need(y1 != 0, "p256_point")
        slope = (3 * x1 * x1 - 3) * pow(2 * y1, -1, _P) % _P
    else:
        slope = (y2 - y1) * pow((x2 - x1) % _P, -1, _P) % _P
    x3 = (slope * slope - x1 - x2) % _P
    return x3, (slope * (x1 - x3) - y1) % _P


def _mul(scalar: int, point):
    result = None
    while scalar:
        if scalar & 1:
            result = _add(result, point)
        point = _add(point, point)
        scalar >>= 1
    return result


def _der(signature: bytes) -> tuple[int, int]:
    _need(8 <= len(signature) <= 72 and signature[:1] == b"\x30" and signature[1] == len(signature) - 2,
          "p256_der")
    values = []
    pos = 2
    for _ in range(2):
        _need(pos + 2 <= len(signature) and signature[pos] == 2, "p256_der")
        length = signature[pos + 1]
        raw = signature[pos + 2:pos + 2 + length]
        _need(1 <= length <= 33 and len(raw) == length and not raw[0] & 0x80 and
              not (length > 1 and raw[0] == 0 and not raw[1] & 0x80), "p256_der")
        values.append(int.from_bytes(raw, "big"))
        pos += 2 + length
    _need(pos == len(signature), "p256_der")
    r, s = values
    _need(0 < r < _N and 0 < s < _N, "p256_scalar")
    _need(s <= _N // 2, "p256_high_s")
    return r, s


def _verify_p256(signature: bytes, contract_digest: bytes) -> None:
    _need(len(contract_digest) == 32 and all(type(v) is int and 0 <= v < _P for v in _Q) and
          (_Q[1] * _Q[1] - (_Q[0] ** 3 - 3 * _Q[0] +
           0x5AC635D8AA3A93E7B3EBBD55769886BC651D06B0CC53B0F63BCE3C3E27D2604B)) % _P == 0 and
          hashlib.sha256(b"\x04" + _Q[0].to_bytes(32, "big") + _Q[1].to_bytes(32, "big")).hexdigest()
          == _KEY_HASH[7:], "p256_point")
    r, s = _der(signature)
    z = int.from_bytes(hashlib.sha256(contract_digest).digest(), "big")
    inv = pow(s, -1, _N)
    point = _add(_mul(z * inv % _N, _G), _mul(r * inv % _N, _Q))
    _need(point is not None and point[0] % _N == r, "p256_signature")


def _label(label: str) -> bytes:
    return hashlib.sha256(b"raios.dev_rollback_fixture.v1\0" + label.encode()).digest()


def _model_hash(canonical: str, schema: str, fields) -> bytes:
    lines = [f"canonicalization={canonical}", f"schema={schema}",
             "requested_capability=cap.module.load_ephemeral", "load_mode=ram_only",
             "subject=agent.session.serial", "resource=live_service_graph", "scope=current_boot"]
    for name, value in fields:
        text = f"event.current_boot.{value:08}" if type(value) is int else value.hex() if type(value) is bytes else str(value)
        lines.append(f"{name}={text}")
    return hashlib.sha256("\n".join(lines).encode()).digest()


def _q(value: bytes) -> str:
    return "sha256:" + value.hex()


def _record_contract(artifact: bytes, artstor: bytes, frame_hashes: list[bytes], signatures) -> tuple[dict, ...]:
    ah = hashlib.sha256(artifact).digest()
    service = "svc.dev.granted_candidate"
    trust = "dev_key_not_owner_sealed"
    labels = {name: _label(name) for name in ("manifest-log-only", "vm-report-log-only", "local-attestation-log-only",
              "manifest-reference-log-only", "artifact-reference-log-only", "vm-report-reference-log-only", "activation-approval",
              "w7-invocation", "w7-receipt", "physical-owner-authority-evidence", "physical-install-approval",
              "rollback-plan-log-only", "pre-load-inventory-log-only", "reprojected-inventory-log-only")}
    grant = _model_hash("raios.computed_capability_grant.canonical.v0", "raios.computed_capability_grant.v0",
            [("manifest_sha256", labels["manifest-log-only"]), ("candidate_artifact_sha256", ah),
             ("vm_test_report_sha256", labels["vm-report-log-only"]), ("local_attestation_sha256", labels["local-attestation-log-only"]),
             ("grants_load_now", "false"), ("authorizes_guest_load", "false"), ("service_inventory_change", "none"), ("load_attempted", "false")])
    attest = _model_hash("raios.module_local_attestation_reference.canonical.v0", "raios.module_local_attestation_reference.v0",
            [("retained_manifest_reference_event_id", 101), ("retained_artifact_reference_event_id", 102),
             ("retained_vm_report_reference_event_id", 103), ("retained_reference_event_id", 104),
             ("manifest_reference_sha256", labels["manifest-reference-log-only"]), ("artifact_reference_sha256", labels["artifact-reference-log-only"]),
             ("vm_test_report_reference_sha256", labels["vm-report-reference-log-only"]), ("manifest_sha256", labels["manifest-log-only"]),
             ("candidate_artifact_sha256", ah), ("computed_capability_grant_sha256", grant),
             ("vm_test_report_sha256", labels["vm-report-log-only"]), ("local_attestation_sha256", labels["local-attestation-log-only"]),
             ("accepts_local_attestation_json", "false"), ("accepts_artifact_bytes", "false"), ("loads_artifact", "false"),
             ("authorizes_guest_load", "false"), ("service_inventory_change", "none"), ("load_attempted", "false")])
    snapshot = hashlib.sha256(b"raios.grant_target_snapshot.v1\0" + service.encode() + b"\0" + ah +
                              (1).to_bytes(8, "little") + b"env.log\0host_call\0").digest()
    text = lambda value: len(value.encode()).to_bytes(2, "little") + value.encode()
    optional = lambda value: bytes((value is not None,)) + (value or bytes(32))
    body = ((2).to_bytes(2, "little") + text(service) + ah + (121).to_bytes(8, "little") + labels["activation-approval"] +
            grant + attest + text("raios.grant_target_snapshot.v1") + (1).to_bytes(8, "little") + snapshot + labels["w7-invocation"] +
            labels["w7-receipt"] + ah * 3 + (1).to_bytes(8, "little") + b"\x01" + text(trust))
    envelope = hashlib.sha256(b"raios.granted_candidate_install_envelope.v2" + body).digest()
    action_body = b"\x01\x01" + text(service) + (1).to_bytes(8, "little") * 2 + optional(None) + optional(envelope) + optional(None) + labels["physical-owner-authority-evidence"] + optional(labels["physical-install-approval"]) + optional(bytes.fromhex(_KEY_HASH[7:]))
    message = hashlib.sha256(b"raios.project_install_action_signature.v1" + action_body).digest()
    install_sig, promotion_sig = signatures
    action = hashlib.sha256(b"raios.project_install_action.v1" + action_body + len(install_sig).to_bytes(2, "little") + install_sig).digest()
    base = {"scope": "origin_boot", "classification": "local_only", "service_id": service}
    auth = {**base, "schema": "raios.install_authorization.v0", "id": "install_authorization.origin_boot.svc.dev.granted_candidate.v0", "record_kind": "install_authorization", "install_envelope_version": 2,
        "activation_approval_sha256": _q(labels["activation-approval"]), "computed_grant_sha256": _q(grant), "attestation_reference_sha256": _q(attest), "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_count": 1, "grant_target_snapshot_sha256": _q(snapshot),
        "w7_invocation_sha256": _q(labels["w7-invocation"]), "w7_receipt_sha256": _q(labels["w7-receipt"]), "receiver_content_sha256": _q(ah), "receiver_candidate_sha256": _q(ah), "catalog_candidate_sha256": _q(ah), "install_envelope_sha256": _q(envelope), "install_action_sha256": _q(action),
        "install_action_signature_message_sha256": _q(message), "authority_evidence_sha256": _q(labels["physical-owner-authority-evidence"]), "physical_approval_sha256": _q(labels["physical-install-approval"]), "install_authority_key_sha256": _KEY_HASH,
        "install_signature_der": install_sig.hex(), "install_signature_len": len(install_sig), "install_candidate_sha256": _q(ah), "install_candidate_byte_len": 121, "install_generation": 1, "install_log_sequence": 1, "trust_tier": trust}
    def promotion(kind):
        un = kind == "unpromote"
        return {**base, "schema": "raios.promotion_transaction.v0", "id": f"promotion_transaction.origin_boot.{kind}.svc.dev.granted_candidate.v0", "record_kind": "promotion_transaction", "transaction_kind": kind, "artifact_id": "wasm:external:granted_candidate", "requested_capability": "cap.module.load_ephemeral.dev_tier.current_boot", "load_mode": "wasmi_interpreter_ram_only",
            "computed_grant_hash": _q(grant), "manifest_hash": _q(labels["manifest-log-only"]), "artifact_hash": _q(ah), "vm_report_hash": _q(labels["vm-report-log-only"]), "local_attestation_hash": _q(labels["local-attestation-log-only"]), "retained_manifest_reference_event_id": "event.current_boot.00000101", "retained_artifact_reference_event_id": "event.current_boot.00000102", "retained_vm_report_reference_event_id": "event.current_boot.00000103", "retained_computed_grant_reference_event_id": "event.current_boot.00000104",
            "manifest_reference_hash": _q(labels["manifest-reference-log-only"]), "artifact_reference_hash": _q(labels["artifact-reference-log-only"]), "vm_report_reference_hash": _q(labels["vm-report-reference-log-only"]), "attestation_reference_hash": _q(attest), "promotion_signature_der": promotion_sig.hex(), "promotion_signature_len": len(promotion_sig), "promotion_authority_key_sha256": _KEY_HASH,
            "signature_verified": True, "grant_binds_capability": True, "install_authorization_present": True, "install_envelope_binds_activation": True, "install_action_signature_verified": True, "physical_install_approval_consumed": True, "install_authorization_frame_sha256": _q(frame_hashes[0]), "install_envelope_version": 2,
            "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_count": 1, "grant_target_snapshot_sha256": _q(snapshot), "trust_tier": trust, "promotion_authority_is_placeholder": True, "owner_sealed": False, "cross_reboot_proven": False, "persistence_claimed": False,
            "rollback_plan_hash": _q(labels["rollback-plan-log-only"]), "pre_load_inventory_hash": _q(labels["pre-load-inventory-log-only"]), "ram_only_service_slot_id": "ram.slot.granted_candidate.current_boot", "generation": 1, "load_event_id": "event.current_boot.00000105", "rollback_apply_event_id": "event.current_boot.00000106" if un else None,
            "reprojected_inventory_hash": _q(labels["reprojected-inventory-log-only"]) if un else None, "restore_hash_verified": un, "stopped": un, "drop_clear_bytes": un, "free_slot": un, "remove_inventory": un}
    persist = {**base, "schema": "raios.artifact_persist.v0", "id": "artifact_persist.origin_boot.svc.dev.granted_candidate.v0", "record_kind": "artifact_persist", "artstor_blob_offset": 0, "artstor_blob_len": len(artstor), "artstor_blob_frame_sha256": _q(hashlib.sha256(artstor).digest()), "artifact_sha256": _q(ah),
        "manifest_hash": _q(labels["manifest-log-only"]), "vm_report_hash": _q(labels["vm-report-log-only"]), "grant_hash": _q(grant), "import_set_hash": _q(hashlib.sha256(b"env.log\n").digest()), "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_entries": "env.log:host_call", "grant_target_count": 1, "grant_target_snapshot_sha256": _q(snapshot),
        "promotion_transaction_sha256": _q(frame_hashes[1]), "blob_written_to_disk": True, "authorizes_load": False, "owner_sealed": False, "cross_reboot_proven": False, "persistence_claimed": False}
    return auth, promotion("promote"), persist, promotion("unpromote")


def verify_fixture(artifact, reclog, artstor) -> FixtureResult:
    _need(all(isinstance(value, (bytes, bytearray, memoryview)) for value in (artifact, reclog, artstor)), "fixture_input")
    _need(len(artifact) == 121 and 0 < len(reclog) <= 65536 and 0 < len(artstor) <= 65536, "fixture_input")
    artifact, reclog, artstor = bytes(artifact), bytes(reclog), bytes(artstor)
    ah = hashlib.sha256(artifact).digest()
    _need(len(artifact) == 121 and ah.hex() == "33ea9dc8f8ecd039236673fafdffcf63e8691a1d352cffee4ddf47531cb5756c", "fixture_artifact")
    _need(0 < len(reclog) <= 65536 and len(reclog) % 512 == 0, "fixture_reclog_bounds")
    payloads, hashes, offset, previous = [], [], 0, bytes(32)
    for seq in range(1, 5):
        _need(offset + 88 <= len(reclog), "fixture_reclog_frame")
        header = reclog[offset:offset + 88]
        frame_len, payload_len, actual_seq = struct.unpack_from("<IIQ", header, 8)
        _need(header[:8] == b"RAIOSRC0" and frame_len >= 512 and frame_len % 512 == 0 and offset + frame_len <= len(reclog) and payload_len <= frame_len - 88 and actual_seq == seq and header[24:56] == previous, "fixture_reclog_frame")
        frame = reclog[offset:offset + frame_len]
        payload = frame[88:88 + payload_len]
        _need(header[56:88] == hashlib.sha256(payload).digest() and not any(frame[88 + payload_len:]), "fixture_reclog_frame")
        previous = hashlib.sha256(frame).digest()
        payloads.append(payload)
        hashes.append(previous)
        offset += frame_len
    _need(offset == len(reclog), "fixture_reclog_count")
    records = [_json(payload) for payload in payloads]
    identities = (("raios.install_authorization.v0", "install_authorization", None), ("raios.promotion_transaction.v0", "promotion_transaction", "promote"), ("raios.artifact_persist.v0", "artifact_persist", None), ("raios.promotion_transaction.v0", "promotion_transaction", "unpromote"))
    _need(all(record.get("schema") == schema and record.get("record_kind") == kind and record.get("transaction_kind") == transaction for record, (schema, kind, transaction) in zip(records, identities)), "fixture_record_order")
    _need(0 < len(artstor) <= 65536 and len(artstor) % 512 == 0 and len(artstor) >= 48 and artstor[:8] == b"RAIOSAR0", "fixture_artstor_frame")
    frame_len, payload_len = struct.unpack_from("<II", artstor, 8)
    _need(frame_len == len(artstor) and payload_len == 121 and artstor[16:48] == ah and artstor[48:169] == artifact and not any(artstor[169:]), "fixture_artstor_payload")
    try:
        install_text = records[0]["install_signature_der"]
        promotion_text = records[1]["promotion_signature_der"]
        _need(type(install_text) is str and bytes.fromhex(install_text).hex() == install_text and type(promotion_text) is str and bytes.fromhex(promotion_text).hex() == promotion_text, "fixture_signature_encoding")
        signatures = bytes.fromhex(install_text), bytes.fromhex(promotion_text)
    except (KeyError, ValueError) as exc:
        raise SecurityError("fixture_signature_encoding") from exc
    expected = _record_contract(artifact, artstor, hashes, signatures)
    install_message = bytes.fromhex(expected[0]["install_action_signature_message_sha256"][7:])
    promotion_message = bytes.fromhex(expected[1]["attestation_reference_hash"][7:])
    _verify_p256(signatures[0], install_message)
    _verify_p256(signatures[1], promotion_message)
    _need(all(payload == _canon(wanted) for payload, wanted in zip(payloads, expected)), "fixture_contract")
    return FixtureResult(ah, hashes[-1], hashlib.sha256(artstor).digest())


_SECTOR = 512
_ENTRY_BYTES = 16384
_ESP_LBAS = 262144
_DATA_LBAS = 524288
_DATA_FIRST = 2048 + 2 * _ESP_LBAS
_TOTAL_LBAS = _DATA_FIRST + _DATA_LBAS + 32 + 1
_BACKUP = _TOTAL_LBAS - 1
_LAST_USABLE = _BACKUP - 33
_TOTAL_BYTES = _TOTAL_LBAS * _SECTOR
_DISK_GUID = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000000")
_DATA_TYPE = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000001")
_ESP_TYPE = uuid.UUID("c12a7328-f81f-11d2-ba4b-00a0c93ec93b")
_EXPECTED = (("SEED_ESP_A", _ESP_TYPE, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000a"), 2048, 2048 + _ESP_LBAS - 1),
             ("SEED_ESP_B", _ESP_TYPE, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000b"), 2048 + _ESP_LBAS, _DATA_FIRST - 1),
             ("SEED_DATA", _DATA_TYPE, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000d"), _DATA_FIRST, _DATA_FIRST + _DATA_LBAS - 1))


def _gpt_bytes(source) -> GptInput:
    if type(source) is GptInput:
        _need(source.size == _TOTAL_BYTES and all(isinstance(value, (bytes, bytearray, memoryview)) for value in source[1:]) and
              tuple(len(value) for value in source[1:]) == (512, _ENTRY_BYTES, _ENTRY_BYTES, 512), "gpt_source")
        return GptInput(source.size, *(bytes(value) for value in source[1:]))
    if isinstance(source, (bytes, bytearray, memoryview)):
        _need(len(source) == _TOTAL_BYTES, "gpt_size")
        view = memoryview(source)
        return GptInput(len(view), bytes(view[512:1024]), bytes(view[1024:1024 + _ENTRY_BYTES]), bytes(view[(_BACKUP - 32) * 512:_BACKUP * 512]), bytes(view[_BACKUP * 512:(_BACKUP + 1) * 512]))
    try:
        size, reader = getattr(source, "size"), getattr(source, "read_at")
        _need(type(size) is int and callable(reader), "gpt_source")
        chunks = (reader(512, 512), reader(1024, _ENTRY_BYTES), reader((_BACKUP - 32) * 512, _ENTRY_BYTES), reader(_BACKUP * 512, 512))
    except SecurityError:
        raise
    except Exception as exc:
        raise SecurityError("gpt_source") from exc
    _need(all(isinstance(value, (bytes, bytearray, memoryview)) for value in chunks) and
          tuple(len(value) for value in chunks) == (512, _ENTRY_BYTES, _ENTRY_BYTES, 512), "gpt_source")
    return GptInput(size, *(bytes(value) for value in chunks))


def validate_gpt(source) -> PartitionIdentity:
    image = _gpt_bytes(source)
    _need(image.size == _TOTAL_BYTES and len(image.primary_header) == 512 and len(image.backup_header) == 512 and len(image.primary_entries) == _ENTRY_BYTES and len(image.backup_entries) == _ENTRY_BYTES, "gpt_size")
    def header(raw, current, alternate, table):
        values = struct.unpack_from("<8sIIIIQQQQ16sQIII", raw)
        _need(values[0] == b"EFI PART" and values[1] == 0x10000 and values[2] == 92 and values[4] == 0 and not any(raw[92:]), "gpt_header")
        checked = bytearray(raw[:92])
        struct.pack_into("<I", checked, 16, 0)
        _need(values[3] == binascii.crc32(checked) & 0xFFFFFFFF, "gpt_header_crc")
        _need(values[5:9] == (current, alternate, 34, _LAST_USABLE) and values[10:13] == (table, 128, 128), "gpt_geometry")
        _need(uuid.UUID(bytes_le=values[9]) == _DISK_GUID, "gpt_disk_guid")
        return values[13]
    primary_crc = header(image.primary_header, 1, _BACKUP, 2)
    backup_crc = header(image.backup_header, _BACKUP, 1, _BACKUP - 32)
    _need(image.primary_entries == image.backup_entries, "gpt_entries_copy")
    actual_crc = binascii.crc32(image.primary_entries) & 0xFFFFFFFF
    _need(primary_crc == actual_crc and backup_crc == actual_crc, "gpt_entries_crc")
    partitions = []
    seen = set()
    for index in range(128):
        raw = image.primary_entries[index * 128:(index + 1) * 128]
        if raw[:16] == bytes(16):
            _need(not any(raw), "gpt_entry")
            continue
        try:
            decoded = raw[56:128].decode("utf-16le")
        except UnicodeDecodeError as exc:
            raise SecurityError("gpt_entry") from exc
        name = decoded.rstrip("\0")
        first, last, attributes = struct.unpack_from("<QQQ", raw, 32)
        unique = uuid.UUID(bytes_le=raw[16:32])
        _need(unique.int != 0 and unique not in seen and first <= last and 34 <= first <= last <= _LAST_USABLE and attributes == 0, "gpt_entry")
        seen.add(unique)
        partitions.append((name, uuid.UUID(bytes_le=raw[:16]), unique, first, last))
    ordered = sorted(partitions, key=lambda item: item[3])
    _need(all(left[4] < right[3] for left, right in zip(ordered, ordered[1:])), "gpt_overlap")
    seed = [part for part in partitions if part[0] == "SEED_DATA"]
    _need(len(seed) == 1 and seed[0][1] == _DATA_TYPE, "gpt_seed")
    _need(set(partitions) == set(_EXPECTED), "gpt_layout")
    return PartitionIdentity(*seed[0])


def _hash_text(value) -> bool:
    if type(value) is not str or len(value) != 71 or not value.startswith("sha256:"):
        return False
    try:
        return bytes.fromhex(value[7:]).hex() == value[7:]
    except ValueError:
        return False


def _bounded_plain(value, budget: list[int], depth: int = 0) -> None:
    _need(depth <= 16, "recovery_malformed")
    if value is None or type(value) is bool:
        return
    if type(value) is int:
        _need(-(1 << 63) <= value < 1 << 64, "recovery_malformed")
        return
    if type(value) is str:
        try:
            budget[0] += len(value.encode())
        except UnicodeError as exc:
            raise SecurityError("recovery_malformed") from exc
        _need(budget[0] <= 262144, "recovery_malformed")
        return
    _need(type(value) in (dict, list) and len(value) <= 128, "recovery_malformed")
    items = value.items() if type(value) is dict else enumerate(value)
    for key, item in items:
        if type(value) is dict:
            _need(type(key) is str, "recovery_malformed")
            _bounded_plain(key, budget, depth + 1)
        _bounded_plain(item, budget, depth + 1)


def _records(records) -> list[dict]:
    _need(type(records) in (list, tuple) and 1 <= len(records) <= 128, "recovery_malformed")
    out, total = [], 0
    for record in records:
        _need(type(record) is dict, "recovery_malformed")
        _bounded_plain(record, [0])
        try:
            encoded = json.dumps(record, ensure_ascii=False, sort_keys=True, allow_nan=False, separators=(",", ":"))
            copy = json.loads(encoded)
        except (TypeError, ValueError, json.JSONDecodeError) as exc:
            raise SecurityError("recovery_malformed") from exc
        total += len(encoded.encode())
        _need(total <= 262144, "recovery_malformed")
        out.append(copy)
    return out


_GRANT_PREDICATES = {"wasm_import.granted.v1", "wasm_import.revoked.v1"}
_EVENT_KEYS = {"event_schema", "service_id", "domain_instance", "binding_sha256", "host_import_id", "scope", "generation", "epoch", "parent_grant_id", "parent_grant_sha256"}
_OUTER_KEYS = {"schema", "id", "kind", "entity", "predicate", "value", "classification", "authority", "boot_id", "sequence", "source", "evidence", "tags", "supersedes", "created_at"}
def _hash_bytes(value: str, code: str) -> bytes:
    _need(_hash_text(value), code)
    return bytes.fromhex(value[7:])
def _grant_event(record: dict) -> dict:
    code = "recovery_grant_fold"
    _need(set(record) == _OUTER_KEYS and record.get("predicate") in _GRANT_PREDICATES, code)
    value = record.get("value")
    _need(type(value) is dict and set(value) == _EVENT_KEYS, code)
    predicate = record["predicate"]
    grant = predicate == "wasm_import.granted.v1"
    rid, service = record.get("id"), value.get("service_id")
    texts = (rid, service, value.get("host_import_id"), value.get("scope"), value.get("parent_grant_id"))
    _need(all(type(item) is str for item in texts), code)
    _need(record == {
        "schema": "raios.memory_record.v0", "id": rid,
        "kind": "capability_grant" if grant else "capability_denial",
        "entity": service, "predicate": predicate, "value": value,
        "classification": "local_only", "authority": "kernel_wasm_import_grant_fold.v1",
        "boot_id": "origin_boot", "sequence": value.get("epoch"),
        "source": {"method": "wasm_import.grant_event", "record_id": rid},
        "evidence": [], "tags": ["wasm_import", "capability"], "supersedes": [],
        "created_at": {"clock": "boot_ticks", "ticks": 0},
    }, code)
    _need(value.get("event_schema") == "raios.wasm_import_grant_event.v1"
          and value.get("host_import_id") in ("env.log", "env.counter_get")
          and value.get("scope") == "host_call", code)
    for field in ("domain_instance", "generation", "epoch"):
        _need(type(value.get(field)) is int and value[field] > 0, code)
    binding = _hash_bytes(value.get("binding_sha256"), code)
    parent_hash = _hash_bytes(value.get("parent_grant_sha256"), code)
    _need((not value["parent_grant_id"] and parent_hash == bytes(32)) if grant
          else (bool(value["parent_grant_id"]) and parent_hash != bytes(32)), code)
    return {**value, "id": rid, "grant": grant, "binding": binding, "parent_hash": parent_hash, "record_hash": hashlib.sha256(_canon(record)).digest()}
def _projection_hash(slots: list[dict]) -> bytes:
    body = bytearray()
    ordered = sorted(slots, key=lambda slot: (slot["service_id"], slot["domain_instance"],
                     slot["binding"], slot["host_import_id"], slot["scope"],
                     slot["generation"], slot["id"]))
    for slot in ordered:
        body.extend(slot["service_id"].encode()); body.append(0)
        body.extend(slot["domain_instance"].to_bytes(8, "little")); body.extend(slot["binding"])
        body.extend(slot["host_import_id"].encode()); body.append(0)
        body.extend(slot["scope"].encode()); body.append(0)
        body.extend(slot["generation"].to_bytes(8, "little")); body.extend(slot["id"].encode())
        body.append(0); body.extend(slot["record_hash"]); body.extend(slot["epoch"].to_bytes(8, "little"))
        if slot["revoke_id"] is not None:
            body.extend(slot["revoke_id"].encode())
        body.append(0); body.append(slot["revoked"])
    return hashlib.sha256(body).digest()


def _target_snapshot(target: RecoveryTarget) -> bytes:
    code = "recovery_target"
    _need(type(target) is RecoveryTarget and target == DETERMINISTIC_ROLLBACK_TARGET, code)
    artifact = _hash_bytes(target.artifact_sha256, code)
    body = bytearray(b"raios.grant_target_snapshot.v1\0" + target.service_id.encode() + b"\0")
    body.extend(artifact); body.extend(len(target.entries).to_bytes(8, "little"))
    for host_import_id, scope in target.entries:
        body.extend(host_import_id.encode() + b"\0" + scope.encode() + b"\0")
    return hashlib.sha256(body).digest()


_AUTH_KEYS = {"schema", "id", "scope", "classification", "service_id", "record_kind",
    "install_envelope_version", "activation_approval_sha256", "computed_grant_sha256",
    "attestation_reference_sha256", "grant_target_schema", "grant_target_count",
    "grant_target_snapshot_sha256", "w7_invocation_sha256", "w7_receipt_sha256",
    "receiver_content_sha256", "receiver_candidate_sha256", "catalog_candidate_sha256",
    "install_envelope_sha256", "install_action_sha256", "install_action_signature_message_sha256",
    "authority_evidence_sha256", "physical_approval_sha256", "install_authority_key_sha256",
    "install_signature_der", "install_signature_len", "install_candidate_sha256",
    "install_candidate_byte_len", "install_generation", "install_log_sequence", "trust_tier"}


def _source_authorization(record: dict, target: RecoveryTarget, version) -> None:
    code = "recovery_source_authorization"
    _need(type(record) is dict and set(record) == _AUTH_KEYS, code)
    generation, artifact_text, artifact_len, entries = version
    service, artifact = target.service_id, _hash_bytes(artifact_text, code)
    _need(record["schema"] == "raios.install_authorization.v0" and
          record["record_kind"] == "install_authorization" and record["service_id"] == service and
          record["id"] == f"install_authorization.origin_boot.{service}.{generation}.v0" and
          record["scope"] == "origin_boot" and record["classification"] == "local_only" and
          record["install_envelope_version"] == 2 and record["install_generation"] == generation and
          type(record["install_log_sequence"]) is int and record["install_log_sequence"] == generation and
          type(record["install_candidate_byte_len"]) is int and record["install_candidate_byte_len"] == artifact_len and
          record["trust_tier"] == "dev_key_not_owner_sealed", code)
    hashes = {name: _hash_bytes(record[name], code) for name in _AUTH_KEYS if name.endswith("_sha256")}
    _need(all(hashes[name] == artifact for name in ("install_candidate_sha256", "receiver_content_sha256",
          "receiver_candidate_sha256", "catalog_candidate_sha256")), code)
    snapshot_body = (b"raios.grant_target_snapshot.v1\0" + service.encode() + b"\0" + artifact +
                      len(entries).to_bytes(8, "little") +
                      b"".join(host.encode() + b"\0" + scope.encode() + b"\0"
                               for host, scope in entries))
    snapshot = hashlib.sha256(snapshot_body).digest()
    _need(record["grant_target_schema"] == "raios.grant_target_snapshot.v1" and
          type(record["grant_target_count"]) is int and record["grant_target_count"] == len(entries) and
          hashes["grant_target_snapshot_sha256"] == snapshot, code)
    text = lambda value: len(value.encode()).to_bytes(2, "little") + value.encode()
    opt = lambda value: bytes((value is not None,)) + (value or bytes(32))
    envelope_body = ((2).to_bytes(2, "little") + text(service) + artifact +
        record["install_candidate_byte_len"].to_bytes(8, "little") + hashes["activation_approval_sha256"] +
        hashes["computed_grant_sha256"] + hashes["attestation_reference_sha256"] +
        text(record["grant_target_schema"]) + len(entries).to_bytes(8, "little") + snapshot +
        hashes["w7_invocation_sha256"] + hashes["w7_receipt_sha256"] + artifact * 3 +
        generation.to_bytes(8, "little") + b"\x01" + text(record["trust_tier"]))
    envelope = hashlib.sha256(b"raios.granted_candidate_install_envelope.v2" + envelope_body).digest()
    _need(envelope == hashes["install_envelope_sha256"], code)
    semantics = (b"\x01\x01" + text(service) + generation.to_bytes(8, "little") +
        record["install_log_sequence"].to_bytes(8, "little") + opt(None) + opt(envelope) + opt(None) +
        hashes["authority_evidence_sha256"] + opt(hashes["physical_approval_sha256"]) +
        opt(hashes["install_authority_key_sha256"]))
    message = hashlib.sha256(b"raios.project_install_action_signature.v1" + semantics).digest()
    try:
        signature = bytes.fromhex(record["install_signature_der"])
    except (TypeError, ValueError) as exc:
        raise SecurityError(code) from exc
    _need(message == hashes["install_action_signature_message_sha256"] and
          hashes["install_authority_key_sha256"] == bytes.fromhex(_KEY_HASH[7:]) and
          type(record["install_signature_len"]) is int and record["install_signature_len"] == len(signature), code)
    _verify_p256(signature, message)
    action = hashlib.sha256(b"raios.project_install_action.v1" + semantics +
                            len(signature).to_bytes(2, "little") + signature).digest()
    _need(action == hashes["install_action_sha256"], code)


def derive_recovery_plan(records, target: RecoveryTarget = DETERMINISTIC_ROLLBACK_TARGET) -> RecoveryPlan:
    """Derive rollback authority solely from the typed grant prefix and fixed target."""
    values = _records(records)
    intents = [i for i, item in enumerate(values) if item.get("predicate") == "wasm_import.rollback_intent.v1"]
    commits = [i for i, item in enumerate(values) if item.get("predicate") == "wasm_import.rollback_commit.v1"]
    _need(len(intents) == len(commits) == 1 and intents[0] < commits[0], "recovery_ambiguous")
    intent_index, commit_index = intents[0], commits[0]
    authorizations = [(i, item) for i, item in enumerate(values) if
                      item.get("schema") == "raios.install_authorization.v0" or
                      item.get("record_kind") == "install_authorization"]
    versions = ((target.generation, target.artifact_sha256, target.artifact_byte_len, target.entries),
                (target.source_generation, target.source_artifact_sha256,
                 target.source_artifact_byte_len, target.source_entries))
    _need(len(authorizations) == len(versions) and all(i < intent_index for i, _ in authorizations),
          "recovery_source_authorization")
    auth_indices = {}
    for (index, record), version in zip(authorizations, versions):
        _source_authorization(record, target, version)
        generation = version[0]
        auth_indices[generation] = index
    _need(tuple(auth_indices) == (target.generation, target.source_generation), "recovery_source_authorization")
    shaped = [i for i, item in enumerate(values)
              if item.get("predicate") in _GRANT_PREDICATES
              or item.get("kind") in ("capability_grant", "capability_denial")]
    _need(not any(i > commit_index for i in shaped), "recovery_grant_fold")
    prefix = [{**_grant_event(values[i]), "record_index": i} for i in shaped if i < intent_index]
    _need(1 <= len(prefix) <= 128, "recovery_grant_fold")
    slots, ids, prior_epoch = [], set(), 0
    for event in prefix:
        _need(event["epoch"] > prior_epoch and event["id"] not in ids, "recovery_grant_fold")
        prior_epoch = event["epoch"]; ids.add(event["id"])
        identity = (event["service_id"], event["domain_instance"], event["binding"],
                    event["host_import_id"], event["scope"])
        matches = [slot for slot in slots if slot["identity"] == identity]
        if event["grant"]:
            _need(len(slots) < 8 and not any(not slot["revoked"] for slot in matches)
                  and not any(slot["generation"] >= event["generation"] for slot in matches),
                  "recovery_grant_fold")
            slots.append({**event, "identity": identity, "revoke_id": None, "revoked": 0})
        else:
            parents = [slot for slot in slots if slot["id"] == event["parent_grant_id"]]
            _need(len(parents) == 1, "recovery_grant_fold")
            parent = parents[0]
            _need(not parent["revoked"] and parent["identity"] == identity
                  and parent["generation"] == event["generation"]
                  and parent["record_hash"] == event["parent_hash"], "recovery_grant_fold")
            parent["revoked"] = 1; parent["revoke_id"] = event["id"]
    expected_binding = hashlib.sha256(target.service_id.encode()).digest()
    _need(len(slots) == 2 and all(not slot["revoked"] for slot in slots), "recovery_grant_fold")
    live = [slot for slot in slots if not slot["revoked"] and slot["service_id"] == target.service_id
            and slot["domain_instance"] == target.domain_instance and slot["binding"] == expected_binding]
    _need({(slot["host_import_id"], slot["scope"]) for slot in live}
          == {("env.log", "host_call"), ("env.counter_get", "host_call")}, "recovery_delta")
    generations = {slot["host_import_id"]: slot["generation"] for slot in live}
    _need(generations["env.counter_get"] == target.source_generation and
          generations["env.log"] in (target.generation, target.source_generation), "recovery_grant_fold")
    _need(all(auth_indices[slot["generation"]] < slot["record_index"] for slot in live),
          "recovery_source_authorization")
    ordered = {slot["host_import_id"]: slot["record_index"] for slot in live}
    _need((ordered["env.log"] < ordered["env.counter_get"] and
           (generations["env.log"] == target.source_generation or
            ordered["env.log"] < auth_indices[target.source_generation])), "recovery_source_authorization")
    _need(all(slot["service_id"] == target.service_id and slot["domain_instance"] == target.domain_instance
              and slot["binding"] == expected_binding for slot in slots), "recovery_grant_fold")
    removed = [slot for slot in live if (slot["host_import_id"], slot["scope"]) not in target.entries]
    _need(len(removed) == 1 and removed[0]["host_import_id"] == "env.counter_get", "recovery_delta")
    removed = removed[0]
    source_hash = _projection_hash(slots)
    delta_body = (removed["id"].encode() + b"\0" + removed["record_hash"]
                  + removed["epoch"].to_bytes(8, "little") + removed["binding"]
                  + removed["host_import_id"].encode() + b"\0" + removed["scope"].encode()
                  + b"\0" + removed["generation"].to_bytes(8, "little"))
    delta_hash, target_hash = hashlib.sha256(delta_body).digest(), _target_snapshot(target)
    transaction_id = hashlib.sha256(source_hash + target_hash + delta_hash).hexdigest()
    revoke_id = (f"rollback.revoke.v1.{len(transaction_id.encode())}.{transaction_id.encode().hex()}."
                 f"{len(removed['id'].encode())}.{removed['id'].encode().hex()}")
    result_slots = [dict(slot) for slot in slots]
    result = next(slot for slot in result_slots if slot["id"] == removed["id"])
    result["revoked"] = 1; result["revoke_id"] = revoke_id
    binding = RecoveryBinding(transaction_id, target.service_id, target.domain_instance,
        "sha256:" + source_hash.hex(), "sha256:" + _projection_hash(result_slots).hex(),
        "sha256:" + target_hash.hex(), 1, "sha256:" + delta_hash.hex(), removed["id"],
        "sha256:" + removed["record_hash"].hex(), "sha256:" + removed["binding"].hex(),
        removed["host_import_id"], removed["scope"], removed["generation"], len(prefix) + 1,
        f"rollback.intent.{transaction_id}", f"rollback.commit.{transaction_id}")
    plan = select_recovery(values, binding)
    _need(plan.intent_index == intent_index and plan.commit_index == commit_index, "recovery_order")
    _need(not any(i != plan.revoke_index for i in shaped if intent_index < i < commit_index),
          "recovery_delta")
    return plan


def select_recovery(records, binding: RecoveryBinding) -> RecoveryPlan:
    texts = (binding.transaction_id, binding.service_id, binding.parent_grant_id, binding.host_import_id, binding.scope, binding.intent_id, binding.commit_id) if type(binding) is RecoveryBinding else ()
    try:
        text_ok = all(type(value) is str and 1 <= len(value.encode()) <= 256 for value in texts)
    except UnicodeError:
        text_ok = False
    _need(type(binding) is RecoveryBinding and text_ok and len(binding.transaction_id) == 64 and all(c in "0123456789abcdef" for c in binding.transaction_id) and type(binding.domain_instance) is int and binding.domain_instance > 0 and type(binding.delta_count) is int and binding.delta_count == 1 and type(binding.generation) is int and binding.generation > 0 and type(binding.epoch) is int and binding.epoch > 0 and all(_hash_text(value) for value in (binding.source_projection_sha256, binding.result_projection_sha256, binding.target_snapshot_sha256, binding.delta_sha256, binding.parent_grant_sha256, binding.member_binding_sha256)), "recovery_malformed")
    values = _records(records)
    intents = [i for i, record in enumerate(values) if record.get("predicate") == "wasm_import.rollback_intent.v1"]
    commits = [i for i, record in enumerate(values) if record.get("predicate") == "wasm_import.rollback_commit.v1"]
    _need(len(intents) == len(commits) == 1, "recovery_ambiguous")
    intent_index, commit_index = intents[0], commits[0]
    _need(intent_index < commit_index and values[intent_index].get("id") == binding.intent_id and values[commit_index].get("id") == binding.commit_id, "recovery_order")
    common = {"transaction_id": binding.transaction_id, "service_id": binding.service_id, "domain_instance": binding.domain_instance, "source_projection_sha256": binding.source_projection_sha256, "target_snapshot_sha256": binding.target_snapshot_sha256, "delta_sha256": binding.delta_sha256, "delta_count": binding.delta_count}
    def marker(phase, sequence, result):
        rid = binding.intent_id if phase == "intent" else binding.commit_id
        return {"schema": "raios.memory_record.v0", "id": rid, "kind": "rollback_tx_ref", "entity": binding.service_id, "predicate": f"wasm_import.rollback_{phase}.v1", "value": {**common, "result_projection_sha256": result}, "classification": "local_only", "authority": "kernel_wasm_rollback_transaction.v1", "boot_id": "origin_boot", "sequence": sequence, "source": {"method": "service.rollback_apply", "record_id": binding.transaction_id}, "evidence": [], "tags": ["wasm_import", "rollback_transaction"], "supersedes": [], "created_at": {"clock": "boot_ticks", "ticks": 0}}
    _need(values[intent_index] == marker("intent", 1, None) and values[commit_index] == marker("commit", 2, binding.result_projection_sha256), "recovery_binding")
    revokes = [(i, record) for i, record in enumerate(values) if intent_index < i < commit_index and record.get("predicate") == "wasm_import.revoked.v1"]
    _need(len(revokes) == binding.delta_count, "recovery_delta")
    revoke_index, revoke = revokes[0]
    expected_id = f"rollback.revoke.v1.{len(binding.transaction_id.encode())}.{binding.transaction_id.encode().hex()}.{len(binding.parent_grant_id.encode())}.{binding.parent_grant_id.encode().hex()}"
    member = {"event_schema": "raios.wasm_import_grant_event.v1", "service_id": binding.service_id, "domain_instance": binding.domain_instance, "binding_sha256": binding.member_binding_sha256, "host_import_id": binding.host_import_id, "scope": binding.scope, "generation": binding.generation, "epoch": binding.epoch, "parent_grant_id": binding.parent_grant_id, "parent_grant_sha256": binding.parent_grant_sha256}
    expected = {"schema": "raios.memory_record.v0", "id": expected_id, "kind": "capability_denial", "entity": binding.service_id, "predicate": "wasm_import.revoked.v1", "value": member, "classification": "local_only", "authority": "kernel_wasm_import_grant_fold.v1", "boot_id": "origin_boot", "sequence": binding.epoch, "source": {"method": "wasm_import.grant_event", "record_id": expected_id}, "evidence": [], "tags": ["wasm_import", "capability"], "supersedes": [], "created_at": {"clock": "boot_ticks", "ticks": 0}}
    _need(revoke == expected, "recovery_member")
    return RecoveryPlan(intent_index, revoke_index, commit_index, binding.transaction_id, binding.parent_grant_id, binding.delta_sha256)
