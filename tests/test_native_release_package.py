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
                package(binary, license_file, output, 'a' * 40, 1700000000, scenario, notices={'project': license_file})
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with zipfile.ZipFile(first) as archive:
                self.assertEqual(set(archive.namelist()), {'bin/careops-ed', 'LICENSE', 'build.json', 'examples/nominal_mvp.json', 'notices/project'})
                self.assertEqual(archive.read('bin/careops-ed'), binary.read_bytes())
                self.assertEqual(archive.read('notices/project'), license_file.read_bytes())
                self.assertEqual(archive.getinfo('bin/careops-ed').external_attr >> 16 & 0o777, 0o755)
                value = json.loads(archive.read('build.json'))
                self.assertEqual(value['binarySha256'], hashlib.sha256(binary.read_bytes()).hexdigest())
            with self.assertRaises(ValueError):
                package(binary, license_file, first, 'a' * 40, 1700000000, scenario, notices={'project': license_file})

    def test_missing_stub_link_nonexecutable_and_invalid_identity_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); binary = root / 'binary'; license_file = root / 'LICENSE'
            license_file.write_text('Example licence fixture')
            scenario = root / 'scenario.json'; scenario.write_text('{}')
            output = root / 'output.zip'
            def refused(source='a' * 40, epoch=1700000000):
                with self.assertRaises(ValueError):
                    package(binary, license_file, output, source, epoch, scenario, notices={'project': license_file})
                self.assertFalse(output.exists())
            refused()
            binary.write_text('#!/bin/sh\necho stub\n'); binary.chmod(0o755); refused()
            binary.write_bytes(b'\x7fELFfixture'); binary.chmod(0o644); refused()
            binary.chmod(0o755); refused('main'); refused(epoch=0)
            license_file.write_text(''); refused()
            license_file.write_text('Example licence fixture')
            target = root / 'target'; binary.rename(target); binary.symlink_to(target); refused()

    def test_notices_are_exact_and_unsafe_or_missing_notices_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'binary'; binary.write_bytes(b'\x7fELFfixture'); binary.chmod(0o755)
            license_file = root / 'LICENSE'; license_file.write_text('Project licence')
            scenario = root / 'scenario.json'; scenario.write_text('{}')
            notice = root / 'NOTICE'; notice.write_text('Original upstream attribution\n')
            output = root / 'package.zip'
            package(binary, license_file, output, 'a' * 40, 1700000000, scenario,
                    notices={'project': license_file, 'kairos/NOTICE': notice})
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(archive.read('notices/kairos/NOTICE'), notice.read_bytes())
                manifest = json.loads(archive.read('build.json'))
                self.assertEqual(manifest['noticeSha256']['kairos/NOTICE'], hashlib.sha256(notice.read_bytes()).hexdigest())
            missing = root / 'missing'; link = root / 'link'; link.symlink_to(notice)
            empty = root / 'empty'; empty.write_bytes(b'')
            for index, notices in enumerate([{}, {'project': missing}, {'project': link}, {'project': empty},
                                             {'project': notice, '.': notice},
                                             {'project': notice, 'windows' + chr(92) + 'escape': notice},
                                             {'project': notice, '../escape': notice},
                                             {'project': notice, '/absolute': notice}]):
                candidate = root / f'refused-{index}.zip'
                with self.assertRaises(ValueError):
                    package(binary, license_file, candidate, 'a' * 40, 1700000000, scenario, notices=notices)
                self.assertFalse(candidate.exists())
