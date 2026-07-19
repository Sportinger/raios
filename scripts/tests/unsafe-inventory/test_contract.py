from __future__ import annotations

import copy
import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parents[1] / "unsafe-inventory.py"
SPEC = importlib.util.spec_from_file_location("unsafe_inventory", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
unsafe_inventory = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = unsafe_inventory
SPEC.loader.exec_module(unsafe_inventory)


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

    def test_added_removed_kind_rationale_and_hash_drift(self):
        added = copy.deepcopy(self.base_sites)
        extra = copy.deepcopy(added[0])
        extra["id"] = "unsafe.fixture.added"
        added.append(extra)
        self.assertIn("added_site", [d["code"] for d in unsafe_inventory.compare_inventory(inventory(self.base_sites), inventory(added))])
        self.assertIn("removed_site", self.drift(lambda sites: sites.pop()))
        self.assertIn("kind_drift", self.drift(lambda sites: sites[0].__setitem__("kind", "fn")))
        self.assertIn("rationale_drift", self.drift(lambda sites: sites[0].__setitem__("reason", "changed")))
        self.assertIn("syntax_hash_drift", self.drift(lambda sites: sites[0].__setitem__("syntax_hash", "sha256:changed")))

    def test_tag_rename_is_remove_plus_add(self):
        codes = self.drift(lambda sites: sites[0].__setitem__("id", "unsafe.fixture.renamed"))
        self.assertIn("removed_site", codes)
        self.assertIn("added_site", codes)


class CliFixtureTests(unittest.TestCase):
    def test_check_mode_is_non_overwriting_and_detects_stale_add(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "fixture"
            shutil.copytree(HERE, root, ignore=shutil.ignore_patterns("test_*.py", "__pycache__"))
            baseline = Path(temporary) / "inventory.json"
            create = subprocess.run(
                [sys.executable, str(SCRIPT), "--root", str(root), "--out", str(baseline), "--require-all-tagged"],
                text=True, capture_output=True, check=False,
            )
            self.assertEqual(0, create.returncode, create.stderr)
            before = baseline.read_bytes()
            green = subprocess.run(
                [sys.executable, str(SCRIPT), "--root", str(root), "--check", str(baseline), "--require-all-tagged"],
                text=True, capture_output=True, check=False,
            )
            self.assertEqual(0, green.returncode, green.stderr)
            rust = root / "fixture-crate" / "src" / "lib.rs"
            rust.write_text(rust.read_text(encoding="utf-8") + '''
// SAFETY[unsafe.fixture.added] Reason: added proof. Invariant: added invariant.
unsafe fn added() {}
''', encoding="utf-8")
            stale = subprocess.run(
                [sys.executable, str(SCRIPT), "--root", str(root), "--check", str(baseline), "--require-all-tagged"],
                text=True, capture_output=True, check=False,
            )
            self.assertNotEqual(0, stale.returncode)
            self.assertIn('"code": "added_site"', stale.stderr)
            self.assertEqual(before, baseline.read_bytes())


if __name__ == "__main__":
    unittest.main()
