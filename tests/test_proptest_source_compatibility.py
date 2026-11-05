"""The bounded dev successor retains every historical and current binding."""
import json
from pathlib import Path
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
from tools import ed_source_compatibility as gate


class ProptestSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.original = gate.shared_bindings(ROOT, gate.SHARED_QUALIFICATION_COMMIT)
        self.current = gate.shared_bindings(ROOT)

    def verify(self):
        gate.verify_proptest_successor(ROOT, self.original, self.current)

    def test_current_profile_is_accepted(self):
        # This fails under the prior guard before its successor is implemented.
        gate.verify_binding(ROOT, json.loads((ROOT / gate.JOIN_PATH).read_bytes()))

    def test_all_31_current_sources_are_bound(self):
        self.assertEqual(len(gate.PROPTEST_SOURCES), 31)
        self.verify()
        original = Path.read_bytes
        for path in gate.PROPTEST_SOURCES:
            def changed(current, selected=ROOT / path):
                data = original(current)
                return data + b'\n' if current == selected else data
            with self.subTest(path=path), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaises(ValueError): self.verify()

    def test_historical_selector_is_fixed_and_never_default(self):
        previous = gate.shared_bindings(ROOT, gate.THISERROR_QUALIFICATION_COMMIT)
        gate.verify_thiserror_successor(ROOT, self.original, previous, source_checkpoint=gate.THISERROR_QUALIFICATION_COMMIT)
        with self.assertRaises(ValueError):
            gate.verify_thiserror_successor(ROOT, self.original, previous)
        for selector in (True, 'HEAD', gate.PROPTEST_QUALIFICATION_COMMIT):
            with self.subTest(selector=selector), self.assertRaises(ValueError):
                gate.verify_thiserror_successor(ROOT, self.original, previous, source_checkpoint=selector)
        with self.assertRaises(ValueError):
            gate.verify_proptest_successor(ROOT, self.original, previous)

    def test_receipt_omission_and_source_codrift_refuse(self):
        original = Path.read_bytes
        selected = ROOT / 'crates/careops-core/tests/r60_proptest_contract.rs'
        receipt_path = ROOT / gate.PROPTEST_RECEIPT
        for omit in (True, False):
            receipt = json.loads(original(receipt_path))
            if omit: receipt['source_bindings'].pop()
            else:
                for row in receipt['source_bindings']:
                    if row['path'] == str(selected.relative_to(ROOT)): row['sha256'] = gate.digest(original(selected) + b'\n')
            def changed(current):
                if current == receipt_path: return json.dumps(receipt).encode()
                if current == selected: return original(current) + b'\n'
                return original(current)
            with self.subTest(omit=omit), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'immutable proptest source'): self.verify()

    def test_both_immutable_anchors_refuse_drift(self):
        original = gate.git_blob
        for anchor, path in [(gate.PROPTEST_BASE_COMMIT, gate.PROPTEST_QUALIFICATION),
                             (gate.PROPTEST_QUALIFICATION_COMMIT, gate.PROPTEST_RECEIPT)]:
            def changed(root, commit, name):
                data = original(root, commit, name)
                return data + b' ' if (commit, name) == (anchor, path) else data
            with self.subTest(path=path), mock.patch.object(gate, 'git_blob', changed):
                with self.assertRaisesRegex(ValueError, 'immutable proptest'): self.verify()

    def test_closed_manifest_and_whole_lock_projection(self):
        old = gate.git_blob(ROOT, gate.THISERROR_QUALIFICATION_COMMIT, 'crates/careops-core/Cargo.toml')
        manifest = (ROOT / 'crates/careops-core/Cargo.toml').read_bytes()
        lock = (ROOT / 'Cargo.lock').read_bytes()
        q = json.loads((ROOT / gate.PROPTEST_QUALIFICATION).read_bytes())
        gate.validate_proptest_successor_profile(old, manifest, lock, q)
        for before, after in [(b'=1.12.0', b'=1.12.1'), (b'default-features = false', b'default-features = true'),
                              (b'features = ["std"]', b'features = ["std", "fork"]'),
                              (b'[dev-dependencies]', b'[build-dependencies]')]:
            altered = manifest.replace(before, after)
            self.assertNotEqual(altered, manifest)
            with self.subTest(after=after), self.assertRaises(ValueError):
                gate.validate_proptest_successor_profile(old, altered, lock, q)
        for altered in [lock + b'\n', lock.replace(b'checksum = "', b'checksum = "0', 1),
                        lock + b'\n[[package]]\nname="unknown"\nversion="1.0.0"\n']:
            with self.assertRaises(ValueError): gate.validate_proptest_successor_profile(old, manifest, altered, q)

    def test_qualification_and_lock_codrift_refuse(self):
        original = Path.read_bytes
        qpath, lockpath = ROOT / gate.PROPTEST_QUALIFICATION, ROOT / 'Cargo.lock'
        q = json.loads(original(qpath)); q['graph']['projected_lock_sha256'] = gate.digest(original(lockpath) + b'\n')
        def changed(current):
            if current == qpath: return json.dumps(q).encode()
            if current == lockpath: return original(current) + b'\n'
            return original(current)
        with mock.patch.object(Path, 'read_bytes', changed):
            with self.assertRaisesRegex(ValueError, 'immutable proptest qualification'): self.verify()


if __name__ == '__main__': unittest.main()
