import copy
import hashlib
import importlib.util
import json
import struct
import unittest
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("rollback_image_security", ROOT / "scripts" / "rollback_image_security.py")
SEC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SEC)

P = 0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
N = 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
G = (0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296, 0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5)
KEY = "sha256:698bea63dc44a344663ff1429aea10842df27b6b991ef25866b2c6c02cdcc5be"


def add(a, b):
    if a is None: return b
    if b is None: return a
    x, y = a; u, v = b
    if x == u and (y + v) % P == 0: return None
    m = ((3 * x * x - 3) * pow(2 * y, -1, P) if a == b else (v - y) * pow(u - x, -1, P)) % P
    q = (m * m - x - u) % P
    return q, (m * (x - q) - y) % P


def mul(k, point=G):
    out = None
    while k:
        if k & 1: out = add(out, point)
        point = add(point, point); k >>= 1
    return out


def integer(value):
    raw = value.to_bytes((value.bit_length() + 7) // 8 or 1, "big")
    if raw[0] & 128: raw = b"\0" + raw
    return b"\x02" + bytes((len(raw),)) + raw


def der(r, s):
    body = integer(r) + integer(s)
    return b"\x30" + bytes((len(body),)) + body


def sign(digest):
    r = mul(2)[0] % N
    s = (int.from_bytes(hashlib.sha256(digest).digest(), "big") + r) * pow(2, -1, N) % N
    if s > N // 2: s = N - s
    return der(r, s)


def canonical(value): return (json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True) + "\n").encode()
def q(value): return "sha256:" + value.hex()
def label(name): return hashlib.sha256(b"raios.dev_rollback_fixture.v1\0" + name.encode()).digest()


def model(canonicalization, schema, fields):
    lines = [f"canonicalization={canonicalization}", f"schema={schema}", "requested_capability=cap.module.load_ephemeral", "load_mode=ram_only", "subject=agent.session.serial", "resource=live_service_graph", "scope=current_boot"]
    for name, value in fields:
        value = f"event.current_boot.{value:08}" if type(value) is int else value.hex() if type(value) is bytes else str(value)
        lines.append(f"{name}={value}")
    return hashlib.sha256("\n".join(lines).encode()).digest()


def wasm():
    out = bytearray.fromhex("0061736d01000000010a0260027f7f006000017f020b0103656e76036c6f670000030201010503010001071f0206")
    out += b"memory" + bytes.fromhex("020012") + b"raios_service_main" + bytes.fromhex("00010a0c010a0041004118100041000b0b1e010041000b18") + b"rollback target log-only"
    return bytes(out)


def artifact_frame(artifact):
    head = struct.pack("<8sII32s", b"RAIOSAR0", 512, len(artifact), hashlib.sha256(artifact).digest())
    return head + artifact + bytes(512 - len(head) - len(artifact))


def reclog_frame(seq, previous, payload):
    length = ((88 + len(payload) + 511) // 512) * 512
    head = struct.pack("<8sIIQ32s32s", b"RAIOSRC0", length, len(payload), seq, previous, hashlib.sha256(payload).digest())
    frame = head + payload + bytes(length - len(head) - len(payload))
    return frame, hashlib.sha256(frame).digest()


def fixture_records(artifact, artstor):
    ah = hashlib.sha256(artifact).digest(); service = "svc.dev.granted_candidate"; trust = "dev_key_not_owner_sealed"
    names = ("manifest-log-only", "vm-report-log-only", "local-attestation-log-only", "manifest-reference-log-only", "artifact-reference-log-only", "vm-report-reference-log-only", "activation-approval", "w7-invocation", "w7-receipt", "physical-owner-authority-evidence", "physical-install-approval", "rollback-plan-log-only", "pre-load-inventory-log-only", "reprojected-inventory-log-only")
    h = {name: label(name) for name in names}
    grant = model("raios.computed_capability_grant.canonical.v0", "raios.computed_capability_grant.v0", [("manifest_sha256", h["manifest-log-only"]), ("candidate_artifact_sha256", ah), ("vm_test_report_sha256", h["vm-report-log-only"]), ("local_attestation_sha256", h["local-attestation-log-only"]), ("grants_load_now", "false"), ("authorizes_guest_load", "false"), ("service_inventory_change", "none"), ("load_attempted", "false")])
    attest = model("raios.module_local_attestation_reference.canonical.v0", "raios.module_local_attestation_reference.v0", [("retained_manifest_reference_event_id", 101), ("retained_artifact_reference_event_id", 102), ("retained_vm_report_reference_event_id", 103), ("retained_reference_event_id", 104), ("manifest_reference_sha256", h["manifest-reference-log-only"]), ("artifact_reference_sha256", h["artifact-reference-log-only"]), ("vm_test_report_reference_sha256", h["vm-report-reference-log-only"]), ("manifest_sha256", h["manifest-log-only"]), ("candidate_artifact_sha256", ah), ("computed_capability_grant_sha256", grant), ("vm_test_report_sha256", h["vm-report-log-only"]), ("local_attestation_sha256", h["local-attestation-log-only"]), ("accepts_local_attestation_json", "false"), ("accepts_artifact_bytes", "false"), ("loads_artifact", "false"), ("authorizes_guest_load", "false"), ("service_inventory_change", "none"), ("load_attempted", "false")])
    snapshot = hashlib.sha256(b"raios.grant_target_snapshot.v1\0" + service.encode() + b"\0" + ah + (1).to_bytes(8, "little") + b"env.log\0host_call\0").digest()
    text = lambda value: len(value.encode()).to_bytes(2, "little") + value.encode(); opt = lambda value: bytes((value is not None,)) + (value or bytes(32))
    envelope = hashlib.sha256(b"raios.granted_candidate_install_envelope.v2" + (2).to_bytes(2, "little") + text(service) + ah + (121).to_bytes(8, "little") + h["activation-approval"] + grant + attest + text("raios.grant_target_snapshot.v1") + (1).to_bytes(8, "little") + snapshot + h["w7-invocation"] + h["w7-receipt"] + ah * 3 + (1).to_bytes(8, "little") + b"\1" + text(trust)).digest()
    semantics = b"\1\1" + text(service) + (1).to_bytes(8, "little") * 2 + opt(None) + opt(envelope) + opt(None) + h["physical-owner-authority-evidence"] + opt(h["physical-install-approval"]) + opt(bytes.fromhex(KEY[7:]))
    message = hashlib.sha256(b"raios.project_install_action_signature.v1" + semantics).digest(); install_sig = sign(message); promo_sig = sign(attest)
    action = hashlib.sha256(b"raios.project_install_action.v1" + semantics + len(install_sig).to_bytes(2, "little") + install_sig).digest(); base = {"scope": "origin_boot", "classification": "local_only", "service_id": service}
    auth = {**base, "schema": "raios.install_authorization.v0", "id": "install_authorization.origin_boot.svc.dev.granted_candidate.v0", "record_kind": "install_authorization", "install_envelope_version": 2, "activation_approval_sha256": q(h["activation-approval"]), "computed_grant_sha256": q(grant), "attestation_reference_sha256": q(attest), "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_count": 1, "grant_target_snapshot_sha256": q(snapshot), "w7_invocation_sha256": q(h["w7-invocation"]), "w7_receipt_sha256": q(h["w7-receipt"]), "receiver_content_sha256": q(ah), "receiver_candidate_sha256": q(ah), "catalog_candidate_sha256": q(ah), "install_envelope_sha256": q(envelope), "install_action_sha256": q(action), "install_action_signature_message_sha256": q(message), "authority_evidence_sha256": q(h["physical-owner-authority-evidence"]), "physical_approval_sha256": q(h["physical-install-approval"]), "install_authority_key_sha256": KEY, "install_signature_der": install_sig.hex(), "install_signature_len": len(install_sig), "install_candidate_sha256": q(ah), "install_candidate_byte_len": 121, "install_generation": 1, "install_log_sequence": 1, "trust_tier": trust}
    frame0, hash0 = reclog_frame(1, bytes(32), canonical(auth))
    def promotion(kind):
        un = kind == "unpromote"
        return {**base, "schema": "raios.promotion_transaction.v0", "id": f"promotion_transaction.origin_boot.{kind}.svc.dev.granted_candidate.v0", "record_kind": "promotion_transaction", "transaction_kind": kind, "artifact_id": "wasm:external:granted_candidate", "requested_capability": "cap.module.load_ephemeral.dev_tier.current_boot", "load_mode": "wasmi_interpreter_ram_only", "computed_grant_hash": q(grant), "manifest_hash": q(h["manifest-log-only"]), "artifact_hash": q(ah), "vm_report_hash": q(h["vm-report-log-only"]), "local_attestation_hash": q(h["local-attestation-log-only"]), "retained_manifest_reference_event_id": "event.current_boot.00000101", "retained_artifact_reference_event_id": "event.current_boot.00000102", "retained_vm_report_reference_event_id": "event.current_boot.00000103", "retained_computed_grant_reference_event_id": "event.current_boot.00000104", "manifest_reference_hash": q(h["manifest-reference-log-only"]), "artifact_reference_hash": q(h["artifact-reference-log-only"]), "vm_report_reference_hash": q(h["vm-report-reference-log-only"]), "attestation_reference_hash": q(attest), "promotion_signature_der": promo_sig.hex(), "promotion_signature_len": len(promo_sig), "promotion_authority_key_sha256": KEY, "signature_verified": True, "grant_binds_capability": True, "install_authorization_present": True, "install_envelope_binds_activation": True, "install_action_signature_verified": True, "physical_install_approval_consumed": True, "install_authorization_frame_sha256": q(hash0), "install_envelope_version": 2, "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_count": 1, "grant_target_snapshot_sha256": q(snapshot), "trust_tier": trust, "promotion_authority_is_placeholder": True, "owner_sealed": False, "cross_reboot_proven": False, "persistence_claimed": False, "rollback_plan_hash": q(h["rollback-plan-log-only"]), "pre_load_inventory_hash": q(h["pre-load-inventory-log-only"]), "ram_only_service_slot_id": "ram.slot.granted_candidate.current_boot", "generation": 1, "load_event_id": "event.current_boot.00000105", "rollback_apply_event_id": "event.current_boot.00000106" if un else None, "reprojected_inventory_hash": q(h["reprojected-inventory-log-only"]) if un else None, "restore_hash_verified": un, "stopped": un, "drop_clear_bytes": un, "free_slot": un, "remove_inventory": un}
    promote = promotion("promote"); frame1, hash1 = reclog_frame(2, hash0, canonical(promote))
    persist = {**base, "schema": "raios.artifact_persist.v0", "id": "artifact_persist.origin_boot.svc.dev.granted_candidate.v0", "record_kind": "artifact_persist", "artstor_blob_offset": 0, "artstor_blob_len": len(artstor), "artstor_blob_frame_sha256": q(hashlib.sha256(artstor).digest()), "artifact_sha256": q(ah), "manifest_hash": q(h["manifest-log-only"]), "vm_report_hash": q(h["vm-report-log-only"]), "grant_hash": q(grant), "import_set_hash": q(hashlib.sha256(b"env.log\n").digest()), "grant_target_schema": "raios.grant_target_snapshot.v1", "grant_target_entries": "env.log:host_call", "grant_target_count": 1, "grant_target_snapshot_sha256": q(snapshot), "promotion_transaction_sha256": q(hash1), "blob_written_to_disk": True, "authorizes_load": False, "owner_sealed": False, "cross_reboot_proven": False, "persistence_claimed": False}
    return [auth, promote, persist, promotion("unpromote")]


def frame_records(records, override=None):
    out = b""; previous = bytes(32)
    for index, record in enumerate(records):
        payload = override[index] if override and index in override else canonical(record)
        frame, previous = reclog_frame(index + 1, previous, payload); out += frame
    return out


def gpt_parts():
    esp = uuid.UUID("c12a7328-f81f-11d2-ba4b-00a0c93ec93b"); data = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000001")
    return [["SEED_ESP_A", esp, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000a"), 2048, 264191], ["SEED_ESP_B", esp, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000b"), 264192, 526335], ["SEED_DATA", data, uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000d"), 526336, 1050623]]


def gpt(parts=None, geometry=(34, 1050623), tables=(2, 1050624)):
    entries = bytearray(16384)
    for index, (name, kind, unique, first, last) in enumerate(parts or gpt_parts()):
        entries[index * 128:(index + 1) * 128] = kind.bytes_le + unique.bytes_le + struct.pack("<QQQ", first, last, 0) + name.encode("utf-16le").ljust(72, b"\0")
    crc = __import__("binascii").crc32(entries) & 0xFFFFFFFF; disk = uuid.UUID("5eedda7a-c0de-4a55-9a15-000000000000").bytes_le
    def header(current, alternate, table):
        raw = bytearray(512); struct.pack_into("<8sIIIIQQQQ16sQIII", raw, 0, b"EFI PART", 0x10000, 92, 0, 0, current, alternate, *geometry, disk, table, 128, 128, crc)
        struct.pack_into("<I", raw, 16, __import__("binascii").crc32(raw[:92]) & 0xFFFFFFFF); return bytes(raw)
    return SEC.GptInput(537936384, header(1, 1050656, tables[0]), bytes(entries), bytes(entries), header(1050656, 1, tables[1]))


def recovery():
    h = lambda byte: "sha256:" + (f"{byte:02x}" * 32)
    tx = "ab" * 32; binding = SEC.RecoveryBinding(tx, "svc.dev.granted_candidate", 7, h(1), h(2), h(3), 1, h(4), "grant.env.log", h(5), h(6), "env.log", "host_call", 1, 2, f"rollback.intent.{tx}", f"rollback.commit.{tx}")
    common = {"transaction_id": tx, "service_id": binding.service_id, "domain_instance": 7, "source_projection_sha256": h(1), "target_snapshot_sha256": h(3), "delta_sha256": h(4), "delta_count": 1}
    rid = f"rollback.revoke.v1.{len(tx)}.{tx.encode().hex()}.{len(binding.parent_grant_id)}.{binding.parent_grant_id.encode().hex()}"
    outer = lambda rid, kind, entity, predicate, value, authority, sequence, method, source, tags: {"schema": "raios.memory_record.v0", "id": rid, "kind": kind, "entity": entity, "predicate": predicate, "value": value, "classification": "local_only", "authority": authority, "boot_id": "origin_boot", "sequence": sequence, "source": {"method": method, "record_id": source}, "evidence": [], "tags": tags, "supersedes": [], "created_at": {"clock": "boot_ticks", "ticks": 0}}
    marker = lambda phase, result: outer(getattr(binding, phase + "_id"), "rollback_tx_ref", binding.service_id, f"wasm_import.rollback_{phase}.v1", {**common, "result_projection_sha256": result}, "kernel_wasm_rollback_transaction.v1", 1 if phase == "intent" else 2, "service.rollback_apply", tx, ["wasm_import", "rollback_transaction"])
    member = {"event_schema": "raios.wasm_import_grant_event.v1", "service_id": binding.service_id, "domain_instance": 7, "binding_sha256": h(6), "host_import_id": "env.log", "scope": "host_call", "generation": 1, "epoch": 2, "parent_grant_id": binding.parent_grant_id, "parent_grant_sha256": h(5)}
    records = [marker("intent", None), {"id": "unrelated", "predicate": "audit.v1", "value": {}}, outer(rid, "capability_denial", binding.service_id, "wasm_import.revoked.v1", member, "kernel_wasm_import_grant_fold.v1", 2, "wasm_import.grant_event", rid, ["wasm_import", "capability"]), {"id": "unrelated.2", "predicate": "audit.v1", "value": {}}, marker("commit", h(2))]
    return binding, records


class ContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.artifact = wasm(); cls.artstor = artifact_frame(cls.artifact); cls.records = fixture_records(cls.artifact, cls.artstor); cls.reclog = frame_records(cls.records)

    def fixture_rejects(self, code, artifact=None, reclog=None, artstor=None):
        values = [bytearray(artifact or self.artifact), bytearray(reclog or self.reclog), bytearray(artstor or self.artstor)]; before = [bytes(value) for value in values]
        with self.assertRaisesRegex(SEC.SecurityError, f"^{code}$"): SEC.verify_fixture(*values)
        self.assertEqual(before, [bytes(value) for value in values])

    def gpt_rejects(self, code, image):
        before = tuple(bytes(value) if isinstance(value, (bytes, bytearray)) else value for value in image)
        with self.assertRaisesRegex(SEC.SecurityError, f"^{code}$"): SEC.validate_gpt(image)
        self.assertEqual(before, tuple(bytes(value) if isinstance(value, (bytes, bytearray)) else value for value in image))

    def recovery_rejects(self, code, binding, records):
        before = canonical(records)
        with self.assertRaisesRegex(SEC.SecurityError, f"^{code}$"): SEC.select_recovery(records, binding)
        self.assertEqual(before, canonical(records))

    def test_positive_fixture_contract(self):
        result = SEC.verify_fixture(self.artifact, self.reclog, self.artstor)
        self.assertEqual(result.artifact_sha256, hashlib.sha256(self.artifact).digest()); self.assertEqual(len(self.artifact), 121)

    def test_fixture_framing_json_artifact_and_payload_negatives(self):
        bad = bytearray(self.reclog); bad[24] ^= 1; self.fixture_rejects("fixture_reclog_frame", reclog=bad)
        compact = json.dumps(self.records[0], sort_keys=True).encode(); self.fixture_rejects("fixture_json_canonical", reclog=frame_records(self.records, {0: compact}))
        duplicate = canonical(self.records[0]).replace(b"{\n", b'{\n  "schema": "duplicate",\n', 1); self.fixture_rejects("fixture_json_duplicate", reclog=frame_records(self.records, {0: duplicate}))
        records = copy.deepcopy(self.records); records[1], records[2] = records[2], records[1]; self.fixture_rejects("fixture_record_order", reclog=frame_records(records))
        for index, field, value in ((0, "service_id", "foreign"), (0, "install_generation", 2), (0, "install_log_sequence", 2), (1, "generation", 2)):
            records = copy.deepcopy(self.records); records[index][field] = value; self.fixture_rejects("fixture_contract", reclog=frame_records(records))
        changed = bytearray(self.artstor); changed[48] ^= 1; changed[16:48] = hashlib.sha256(changed[48:169]).digest(); self.fixture_rejects("fixture_artstor_payload", artstor=changed)
        artifact = bytearray(self.artifact); artifact[-1] ^= 1; self.fixture_rejects("fixture_artifact", artifact=artifact)

    def test_fixture_every_security_link_and_claims_fail_closed(self):
        fields = ((0, ("activation_approval_sha256", "computed_grant_sha256", "attestation_reference_sha256", "grant_target_snapshot_sha256", "install_envelope_sha256", "install_action_sha256", "install_action_signature_message_sha256", "authority_evidence_sha256", "physical_approval_sha256", "install_authority_key_sha256", "install_candidate_sha256")), (1, ("computed_grant_hash", "manifest_hash", "artifact_hash", "vm_report_hash", "local_attestation_hash", "manifest_reference_hash", "artifact_reference_hash", "vm_report_reference_hash", "attestation_reference_hash", "install_authorization_frame_sha256", "grant_target_snapshot_sha256", "promotion_authority_key_sha256")), (2, ("artstor_blob_frame_sha256", "artifact_sha256", "manifest_hash", "vm_report_hash", "grant_hash", "import_set_hash", "grant_target_snapshot_sha256", "promotion_transaction_sha256")))
        for index, names in fields:
            for name in names:
                with self.subTest(index=index, name=name):
                    records = copy.deepcopy(self.records); records[index][name] = q(bytes(32)); self.fixture_rejects("fixture_contract", reclog=frame_records(records))
        before = (self.artifact, self.reclog, self.artstor)
        with self.assertRaises(TypeError): SEC.verify_fixture(*before, True)
        self.assertEqual(before, (self.artifact, self.reclog, self.artstor))

    def test_fixture_signature_der_scalar_and_hash_convention_negatives(self):
        def reject(signature, code, promotion=False):
            records = copy.deepcopy(self.records); indexes = (1, 3) if promotion else (0,)
            for index in indexes: records[index]["promotion_signature_der" if promotion else "install_signature_der"] = signature.hex(); records[index]["promotion_signature_len" if promotion else "install_signature_len"] = len(signature)
            self.fixture_rejects(code, reclog=frame_records(records))
        sig = bytes.fromhex(self.records[0]["install_signature_der"]); r, s = parse_der(sig)
        reject(der(r, N - s), "p256_high_s"); reject(der(N, s), "p256_scalar"); reject(b"\x30" + bytes((len(integer(r)) + 1 + len(integer(s)),)) + b"\x02" + bytes((len(integer(r)) - 1 + 1,)) + b"\0" + integer(r)[2:] + integer(s), "p256_der")
        changed = bytearray(sig); changed[-1] ^= 1; reject(bytes(changed), "p256_signature")
        promo = bytearray.fromhex(self.records[1]["promotion_signature_der"]); promo[-1] ^= 1; reject(bytes(promo), "p256_signature", True)

    def test_positive_deterministic_gpt_and_hostile_copies(self):
        image = gpt(); self.assertEqual(SEC.validate_gpt(image).name, "SEED_DATA")
        class Source:
            size = image.size
            def read_at(self, offset, length):
                return {512: image.primary_header, 1024: image.primary_entries, 1050624 * 512: image.backup_entries, 1050656 * 512: image.backup_header}[offset]
        self.assertEqual(SEC.validate_gpt(Source()).first_lba, 526336)
        image = gpt(); head = bytearray(image.primary_header); head[16] ^= 1; self.gpt_rejects("gpt_header_crc", image._replace(primary_header=head))
        head = bytearray(image.primary_header); head[0] ^= 1; self.gpt_rejects("gpt_header", image._replace(primary_header=head))
        entries = bytearray(image.backup_entries); entries[-1] ^= 1; self.gpt_rejects("gpt_entries_copy", image._replace(backup_entries=entries))
        entries = bytearray(image.primary_entries); entries[-1] ^= 1; self.gpt_rejects("gpt_entries_crc", image._replace(primary_entries=entries, backup_entries=entries))

    def test_gpt_layout_type_name_geometry_overlap_and_duplicate_negatives(self):
        parts = gpt_parts(); parts[2][1] = parts[0][1]; self.gpt_rejects("gpt_seed", gpt(parts))
        parts = gpt_parts(); parts[2][0] = "SEED_FAKE"; self.gpt_rejects("gpt_seed", gpt(parts))
        parts = gpt_parts(); parts.append(["SEED_DATA", parts[2][1], uuid.UUID("5eedda7a-c0de-4a55-9a15-00000000000e"), 40, 41]); self.gpt_rejects("gpt_seed", gpt(parts))
        parts = gpt_parts(); parts[1][3] = parts[0][4]; self.gpt_rejects("gpt_overlap", gpt(parts))
        parts = gpt_parts(); parts[1][2] = parts[0][2]; self.gpt_rejects("gpt_entry", gpt(parts))
        self.gpt_rejects("gpt_geometry", gpt(geometry=(35, 1050623))); self.gpt_rejects("gpt_geometry", gpt(tables=(3, 1050624)))

    def test_positive_exact_recovery_selection_without_adjacency(self):
        binding, records = recovery(); plan = SEC.select_recovery(records, binding)
        self.assertEqual((plan.intent_index, plan.revoke_index, plan.commit_index), (0, 2, 4))

    def test_recovery_crossed_foreign_ambiguous_member_and_tuple_negatives(self):
        binding, original = recovery()
        cases = []
        records = copy.deepcopy(original); records[4]["value"]["transaction_id"] = "tx.crossed"; cases.append(("recovery_binding", records))
        records = copy.deepcopy(original); records[0]["value"]["domain_instance"] = 8; records[4]["value"]["domain_instance"] = 8; cases.append(("recovery_binding", records))
        records = copy.deepcopy(original); records.insert(1, copy.deepcopy(records[0])); cases.append(("recovery_ambiguous", records))
        records = copy.deepcopy(original); records[2]["value"]["parent_grant_id"] = "wrong"; cases.append(("recovery_member", records))
        records = copy.deepcopy(original); records.insert(3, copy.deepcopy(records[2])); cases.append(("recovery_delta", records))
        records = [copy.deepcopy(original[4]), copy.deepcopy(original[0]), copy.deepcopy(original[2])]; cases.append(("recovery_order", records))
        records = copy.deepcopy(original); records[0]["value"] = []; cases.append(("recovery_binding", records))
        for field in ("source_projection_sha256", "target_snapshot_sha256", "delta_count", "delta_sha256"):
            records = copy.deepcopy(original); records[4]["value"][field] = 2 if field == "delta_count" else q(bytes(32)); cases.append(("recovery_binding", records))
        for index, path in ((0, "kind"), (0, "entity"), (0, "boot_id"), (0, "sequence"), (0, "source"), (0, "evidence"), (0, "tags"), (0, "created_at"), (2, "authority"), (2, "sequence")):
            records = copy.deepcopy(original); records[index][path] = "wrong"; cases.append(("recovery_binding" if index == 0 else "recovery_member", records))
        for field in ("event_schema", "epoch"):
            records = copy.deepcopy(original); records[2]["value"][field] = "wrong"; cases.append(("recovery_member", records))
        for index, field, code in ((0, "schema", "recovery_binding"), (0, "tags", "recovery_binding"), (2, "event_schema", "recovery_member"), (2, "epoch", "recovery_member")):
            records = copy.deepcopy(original); del (records[index]["value"] if index == 2 else records[index])[field]; cases.append((code, records))
        records = copy.deepcopy(original); records[4]["value"]["result_projection_sha256"] = q(b"\x09" * 32); cases.append(("recovery_binding", records))
        records = copy.deepcopy(original); records[2]["extra"] = True; cases.append(("recovery_member", records))
        for code, records in cases:
            with self.subTest(code=code): self.recovery_rejects(code, binding, records)


def parse_der(signature):
    p = 2; values = []
    for _ in range(2):
        length = signature[p + 1]; values.append(int.from_bytes(signature[p + 2:p + 2 + length], "big")); p += 2 + length
    return values


if __name__ == "__main__": unittest.main()
