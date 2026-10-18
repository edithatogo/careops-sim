"""Real file packaging refuses rehearsal substitutions and preserves revisions."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
from tools.package_native_release import package


class NativePackageTests(unittest.TestCase):
    def test_exact_bytes_modes_and_repeatability(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'binary'; binary.write_bytes(b'\x7fELF' + bytes(range(256))); binary.chmod(0o755)
            license_file = root / 'LICENSE'; license_file.write_text('Example licence fixture')
            scenario = root / 'scenario.json'; scenario.write_text('{}')
            first, second = root / 'one.zip', root / 'two.zip'
            for output in [first, second]:
                package(binary, license_file, output, 'a' * 40, 1700000000, scenario)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with zipfile.ZipFile(first) as archive:
                self.assertEqual(set(archive.namelist()), {'bin/careops-ed', 'LICENSE', 'build.json', 'examples/nominal_mvp.json'})
                self.assertEqual(archive.read('bin/careops-ed'), binary.read_bytes())
                self.assertEqual(archive.getinfo('bin/careops-ed').external_attr >> 16 & 0o777, 0o755)
                value = json.loads(archive.read('build.json'))
                self.assertEqual(value['binarySha256'], hashlib.sha256(binary.read_bytes()).hexdigest())
            with self.assertRaises(ValueError):
                package(binary, license_file, first, 'a' * 40, 1700000000, scenario)

    def test_missing_stub_link_nonexecutable_and_invalid_identity_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); binary = root / 'binary'; license_file = root / 'LICENSE'
            license_file.write_text('Example licence fixture')
            scenario = root / 'scenario.json'; scenario.write_text('{}')
            output = root / 'output.zip'
            def refused(source='a' * 40, epoch=1700000000):
                with self.assertRaises(ValueError):
                    package(binary, license_file, output, source, epoch, scenario)
                self.assertFalse(output.exists())
            refused()
            binary.write_text('#!/bin/sh\necho stub\n'); binary.chmod(0o755); refused()
            binary.write_bytes(b'\x7fELFfixture'); binary.chmod(0o644); refused()
            binary.chmod(0o755); refused('main'); refused(epoch=0)
            license_file.write_text(''); refused()
            license_file.write_text('Example licence fixture')
            target = root / 'target'; binary.rename(target); binary.symlink_to(target); refused()
