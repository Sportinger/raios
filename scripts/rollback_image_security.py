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
