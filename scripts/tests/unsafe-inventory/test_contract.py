from __future__ import annotations

import copy
import contextlib
import importlib.util
import hashlib
import json
import shutil
import subprocess
import sys
import unittest
import uuid
from pathlib import Path
from unittest import mock


HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parents[1] / "unsafe-inventory.py"
SPEC = importlib.util.spec_from_file_location("unsafe_inventory", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
unsafe_inventory = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = unsafe_inventory
SPEC.loader.exec_module(unsafe_inventory)

ROOT = SCRIPT.parents[1]
_CURRENT_INVENTORY = None


def current_inventory():
    global _CURRENT_INVENTORY
    if _CURRENT_INVENTORY is None:
        _CURRENT_INVENTORY = unsafe_inventory.build_inventory(ROOT)
    return _CURRENT_INVENTORY


@contextlib.contextmanager
def temporary_workspace():
    path = ROOT / "target" / f"unsafe-inventory-contract-{uuid.uuid4().hex}"
    path.mkdir(parents=True)
    try:
        yield path
    finally:
        shutil.rmtree(path)


def scan(source: str):
    return unsafe_inventory.scan_source(source, "fixture", "src/lib.rs")


def inventory(sites):
    return {"schema_version": 2, "sites": sites}


class LexingAndAssociationTests(unittest.TestCase):
    def test_positive_and_fake_unsafe_are_distinguished(self):
        source = r'''
// unsafe { unsafe fn fake() {} }
const A: &str = "unsafe { unsafe impl Fake {}";
const B: &str = r###"unsafe extern "C" fn fake() {}"###;
// SAFETY[unsafe.fixture.real] Reason: pointer is valid. Invariant: pointer remains live.
unsafe { read(); }
'''
        sites, diagnostics = scan(source)
        self.assertEqual(["block"], [site["kind"] for site in sites])
        self.assertEqual("unsafe.fixture.real", sites[0]["id"])
        self.assertFalse([d for d in diagnostics if d["code"] != "untagged_site"])
        self.assertNotIn("line", sites[0])
        self.assertTrue(sites[0]["syntax_hash"].startswith("sha256:"))
        self.assertTrue(sites[0]["association_hash"].startswith("sha256:"))

    def test_kinds_do_not_double_count_extern(self):
        sites, _ = scan('''
unsafe fn a() {} unsafe extern "C" fn b() {} unsafe impl X for Y {}
unsafe trait Z {} unsafe { x(); }
''')
        self.assertEqual(["fn", "extern", "impl", "trait", "block"], [s["kind"] for s in sites])

    def test_unsafe_extern_block_is_one_site_and_extern_fn_stays_distinct(self):
        source = r'''
// SAFETY[unsafe.fixture.ffi] Reason: declarations match the provider. Invariant: ABI stays C.
unsafe extern "C" {
    fn nested();
    pub safe static VALUE: i32;
}
unsafe extern "C" fn defined() {}
extern "C" { fn ordinary(); }
const FAKE: &str = r#"unsafe extern "C" { fn fake(); }"#;
'''
        sites, diagnostics = scan(source)
        self.assertEqual(["extern_block", "extern"], [site["kind"] for site in sites])
        self.assertEqual("unsafe.fixture.ffi", sites[0]["id"])
        self.assertEqual(1, sum(site["kind"] == "extern_block" for site in sites))
        self.assertFalse([item for item in diagnostics if item["code"] != "untagged_site"])

        moved, _ = scan("\n\n" + source)
        keys = ("id", "kind", "syntax_hash", "association_hash")
        self.assertEqual([{key: site[key] for key in keys} for site in sites], [{key: site[key] for key in keys} for site in moved])

    def test_untagged_orphan_wrong_adjacency_and_invalid_fields(self):
        source = '''
// SAFETY[unsafe.fixture.orphan] Reason: yes. Invariant: yes.
let not_unsafe = 1;
unsafe { a(); }
// SAFETY[unsafe.fixture.wrong] Reason: yes. Invariant: yes.
let intervening = 2;
unsafe { b(); }
// SAFETY[unsafe.fixture.empty] Reason: Invariant: proof.
unsafe { c(); }
'''
        _, diagnostics = scan(source)
        codes = [item["code"] for item in diagnostics]
        self.assertGreaterEqual(codes.count("orphan_tag"), 2)
        self.assertIn("invalid_tag", codes)
        self.assertEqual(3, codes.count("untagged_site"))

    def test_tag_inside_block_cannot_attach_to_later_site(self):
        source = '''
unsafe {
    work();
    // SAFETY[unsafe.fixture.inner] Reason: yes. Invariant: yes.
    unsafe { nested(); }
}
'''
        sites, diagnostics = scan(source)
        self.assertEqual(2, len(sites))
        self.assertTrue(all(site["id"] is None for site in sites))
        self.assertIn("orphan_tag", [d["code"] for d in diagnostics])

    def test_duplicate_id_is_rejected(self):
        source = '''
// SAFETY[unsafe.fixture.same] Reason: first. Invariant: held.
unsafe { a(); }
// SAFETY[unsafe.fixture.same] Reason: second. Invariant: held.
unsafe { b(); }
'''
        _, diagnostics = scan(source)
        duplicate = [d for d in diagnostics if d["code"] == "duplicate_id"]
        self.assertEqual(1, len(duplicate))
        self.assertIn("duplicate_tag", [d["code"] for d in diagnostics])

    def test_line_movement_does_not_change_identity_or_hashes(self):
        compact = '''
// SAFETY[unsafe.fixture.stable] Reason: proof. Invariant: held.
unsafe { operation(); }
'''
        moved = '''


// SAFETY[unsafe.fixture.stable] Reason: proof. Invariant: held.

unsafe {
    operation();
}
'''
        first, _ = scan(compact)
        second, _ = scan(moved)
        fields = ("id", "kind", "syntax_hash", "rationale_hash", "association_hash")
        self.assertEqual({key: first[0][key] for key in fields}, {key: second[0][key] for key in fields})


class StaleContractTests(unittest.TestCase):
    def setUp(self):
        self.base_sites, _ = scan('''
// SAFETY[unsafe.fixture.one] Reason: proof one. Invariant: invariant one.
unsafe { one(); }
// SAFETY[unsafe.fixture.two] Reason: proof two. Invariant: invariant two.
unsafe fn two() {}
''')

    def drift(self, mutate):
        changed = copy.deepcopy(self.base_sites)
        mutate(changed)
        return [d["code"] for d in unsafe_inventory.compare_inventory(inventory(self.base_sites), inventory(changed))]

    def test_added_removed_tagged_kind_path_item_rationale_and_hash_drift(self):
        added = copy.deepcopy(self.base_sites)
        extra = copy.deepcopy(added[0])
        extra["id"] = "unsafe.fixture.added"
        added.append(extra)
        self.assertIn("added_site", [d["code"] for d in unsafe_inventory.compare_inventory(inventory(self.base_sites), inventory(added))])
        self.assertIn("removed_site", self.drift(lambda sites: sites.pop()))
        self.assertIn("kind_drift", self.drift(lambda sites: sites[0].__setitem__("kind", "fn")))
        self.assertIn("path_drift", self.drift(lambda sites: sites[0].__setitem__("path", "src/other.rs")))
        self.assertIn("item_drift", self.drift(lambda sites: sites[0].__setitem__("enclosing_item", "fn other")))
        self.assertIn("rationale_drift", self.drift(lambda sites: sites[0].__setitem__("reason", "changed")))
        self.assertIn("rationale_drift", self.drift(lambda sites: sites[0].__setitem__("invariant", "changed")))
        self.assertIn("rationale_hash_drift", self.drift(lambda sites: sites[0].__setitem__("rationale_hash", "sha256:changed")))
        self.assertIn("syntax_hash_drift", self.drift(lambda sites: sites[0].__setitem__("syntax_hash", "sha256:changed")))
        self.assertIn("association_hash_drift", self.drift(lambda sites: sites[0].__setitem__("association_hash", "sha256:changed")))

    def test_untagged_set_drift(self):
        untagged = copy.deepcopy(self.base_sites)
        for field in ("id", "reason", "invariant", "rationale_hash", "association_hash"):
            untagged[0][field] = None
        changed = copy.deepcopy(untagged)
        changed[0]["syntax_hash"] = "sha256:changed"
        codes = [item["code"] for item in unsafe_inventory.compare_inventory(inventory(untagged), inventory(changed))]
        self.assertIn("untagged_set_drift", codes)

    def test_tag_rename_is_remove_plus_add(self):
        codes = self.drift(lambda sites: sites[0].__setitem__("id", "unsafe.fixture.renamed"))
        self.assertIn("removed_site", codes)
        self.assertIn("added_site", codes)


class CliFixtureTests(unittest.TestCase):
    def test_check_mode_is_non_overwriting_and_detects_stale_add(self):
        stale_value = copy.deepcopy(current_inventory())
        stale_value["sites"] = stale_value["sites"][1:]
        with temporary_workspace() as temporary:
            baseline = temporary / "stale.json"
            baseline.write_text(unsafe_inventory.render_json(stale_value), encoding="utf-8", newline="\n")
            before = baseline.read_bytes()
            stale = subprocess.run([sys.executable, str(SCRIPT), "--root", str(ROOT), "--check", str(baseline)], text=True, capture_output=True, check=False)
            self.assertNotEqual(0, stale.returncode)
            self.assertIn('"status": "stale"', stale.stderr)
            self.assertEqual(before, baseline.read_bytes())

    def test_build_evidence_hashes_counts_and_negative_immutability(self):
        with temporary_workspace() as temporary:
            baseline = temporary / "inventory.json"
            inventory_value = current_inventory()
            baseline.write_text(unsafe_inventory.render_json(inventory_value), encoding="utf-8", newline="\n")
            baseline_before = baseline.read_bytes()
            artifact = temporary / "seed-kernel"
            artifact.write_bytes(b"bounded fake kernel\x00")
            output = temporary / "evidence.json"
            with mock.patch.object(unsafe_inventory, "build_inventory", return_value=inventory_value):
                unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", output)
            evidence = json.loads(output.read_bytes())
            self.assertEqual(1, evidence["schema_version"])
            self.assertEqual("accepted", evidence["status"])
            self.assertEqual(artifact.resolve().as_posix(), evidence["artifact"]["path"])
            self.assertEqual("sha256:" + hashlib.sha256(artifact.read_bytes()).hexdigest(), evidence["artifact"]["sha256"])
            canonical = unsafe_inventory.render_json(inventory_value).encode()
            self.assertEqual("sha256:" + hashlib.sha256(canonical).hexdigest(), evidence["inventory_sha256"])
            self.assertEqual("sha256:" + hashlib.sha256(baseline_before).hexdigest(), evidence["baseline_sha256"])
            self.assertEqual(inventory_value["totals"]["total"], evidence["counts"]["total"])
            accepted = output.read_bytes()

            def preserved_failure(action, exception=(OSError, ValueError, json.JSONDecodeError)):
                with self.assertRaises(exception):
                    action()
                self.assertEqual(baseline_before, baseline.read_bytes())
                self.assertEqual(accepted, output.read_bytes())

            with mock.patch.object(unsafe_inventory, "build_inventory", return_value=inventory_value):
                preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, temporary / "missing-baseline.json", artifact, "debug", output), OSError)

                baseline.write_bytes(b"{invalid json")
                malformed_before = baseline.read_bytes()
                with self.assertRaises(json.JSONDecodeError):
                    unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", output)
                self.assertEqual(malformed_before, baseline.read_bytes())
                self.assertEqual(accepted, output.read_bytes())
                baseline.write_bytes(baseline_before)

                original_read_bytes = Path.read_bytes
                def deny_baseline(path):
                    if path == baseline:
                        raise PermissionError("injected unreadable baseline")
                    return original_read_bytes(path)
                with mock.patch.object(Path, "read_bytes", deny_baseline):
                    with self.assertRaises(PermissionError):
                        unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", output)
                    self.assertEqual(accepted, original_read_bytes(output))
                self.assertEqual(baseline_before, baseline.read_bytes())

                preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, baseline, temporary / "missing-kernel", "debug", output), OSError)
                artifact_directory = temporary / "artifact-directory"
                artifact_directory.mkdir()
                preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact_directory, "debug", output), ValueError)

                original_open = Path.open
                def deny_artifact(path, *args, **kwargs):
                    if path == artifact:
                        raise PermissionError("injected unreadable artifact")
                    return original_open(path, *args, **kwargs)
                with mock.patch.object(Path, "open", deny_artifact):
                    preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", output), PermissionError)

                bad_parent = temporary / "not-a-directory"
                bad_parent.write_bytes(b"sentinel")
                preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", bad_parent / "evidence.json"), OSError)

                with mock.patch.object(unsafe_inventory.os, "replace", side_effect=PermissionError("injected exact-destination replace denial")) as replace:
                    preserved_failure(lambda: unsafe_inventory.emit_build_evidence(ROOT, baseline, artifact, "debug", output), PermissionError)
                    replace.assert_called_once()
                    self.assertEqual(output, Path(replace.call_args.args[1]))

            invalid_profile = subprocess.run(
                [sys.executable, str(SCRIPT), "--build-evidence", "--check", str(baseline), "--artifact", str(artifact), "--profile", "invalid", "--evidence-out", str(output)],
                text=True, capture_output=True, check=False,
            )
            self.assertNotEqual(0, invalid_profile.returncode)
            self.assertEqual(baseline_before, baseline.read_bytes())
            self.assertEqual(accepted, output.read_bytes())
            self.assertFalse(list(temporary.glob(f".{output.name}.*.tmp")))


if __name__ == "__main__":
    unittest.main()
